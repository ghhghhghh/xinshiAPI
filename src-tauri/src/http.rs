//! HTTP 请求引擎：共享连接池与 Cookie、可取消、流式限读。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use futures_util::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use reqwest::redirect::Policy;
use reqwest::{Method, Url};
use tokio_util::sync::CancellationToken;

use crate::models::{
    HttpRequestSpec, HttpResponseData, ParamField, RequestAuth, RequestBody, ResponseHeader,
    now_iso,
};

const MAX_BODY_BYTES: usize = 8 * 1024 * 1024;
const MAX_IMAGE_BYTES: usize = 2 * 1024 * 1024;
const MIN_TIMEOUT_MS: u64 = 1_000;
const MAX_TIMEOUT_MS: u64 = 300_000;

pub struct HttpEngine {
    client: reqwest::Client,
    no_redirect_client: reqwest::Client,
    cancellations: Mutex<HashMap<String, Arc<CancellationToken>>>,
}

impl HttpEngine {
    pub fn new() -> Result<Self, String> {
        // reqwest 的重定向策略只在 Client 上配置；两个 Client 共享同一个 Cookie Jar，
        // 这样既保留连接池，也能按请求切换是否跟随重定向。
        let cookie_jar = Arc::new(reqwest::cookie::Jar::default());
        let client = build_client(cookie_jar.clone(), Policy::limited(10))?;
        let no_redirect_client = build_client(cookie_jar, Policy::none())?;

        Ok(Self {
            client,
            no_redirect_client,
            cancellations: Mutex::new(HashMap::new()),
        })
    }

    pub async fn send(
        &self,
        request_id: &str,
        spec: HttpRequestSpec,
    ) -> Result<HttpResponseData, String> {
        let timeout_ms = normalize_timeout_ms(spec.timeout_ms);
        let builder = self.prepare(&spec, timeout_ms)?;

        let token = Arc::new(CancellationToken::new());
        {
            let mut map = self.cancellation_map();
            if let Some(previous) = map.insert(request_id.to_string(), token.clone()) {
                previous.cancel();
            }
        }

        let result = tokio::select! {
            biased;
            _ = token.cancelled() => Err("请求已取消".to_string()),
            result = self.perform(builder, timeout_ms) => result,
        };

        {
            let mut map = self.cancellation_map();
            let still_current = map
                .get(request_id)
                .is_some_and(|current| Arc::ptr_eq(current, &token));
            if still_current {
                map.remove(request_id);
            }
        }

        result
    }

    pub fn cancel(&self, request_id: &str) -> bool {
        let map = self.cancellation_map();
        if let Some(token) = map.get(request_id) {
            token.cancel();
            true
        } else {
            false
        }
    }

    fn cancellation_map(&self) -> MutexGuard<'_, HashMap<String, Arc<CancellationToken>>> {
        self.cancellations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn prepare(
        &self,
        spec: &HttpRequestSpec,
        timeout_ms: u64,
    ) -> Result<reqwest::RequestBuilder, String> {
        let method = parse_method(&spec.method)?;
        let url = build_url(&spec.url, &spec.params)?;
        let headers = build_headers(spec);
        let body = build_body(&spec.body);
        let client = if spec.follow_redirects {
            &self.client
        } else {
            &self.no_redirect_client
        };

        let mut builder = client
            .request(method, url)
            .headers(headers)
            .timeout(Duration::from_millis(timeout_ms));

        if let Some(body) = body {
            builder = builder.body(body);
        }
        Ok(builder)
    }

    async fn perform(
        &self,
        builder: reqwest::RequestBuilder,
        timeout_ms: u64,
    ) -> Result<HttpResponseData, String> {
        let started = Instant::now();
        let response = builder
            .send()
            .await
            .map_err(|err| map_request_error(err, timeout_ms))?;

        let status = response.status();
        let status_text = status.canonical_reason().unwrap_or_default().to_string();
        let headers: Vec<ResponseHeader> = response
            .headers()
            .iter()
            .map(|(name, value)| ResponseHeader {
                name: name.as_str().to_string(),
                value: String::from_utf8_lossy(value.as_bytes()).into_owned(),
            })
            .collect();
        let content_type = headers
            .iter()
            .find(|header| header.name.eq_ignore_ascii_case("content-type"))
            .map(|header| header.value.clone())
            .unwrap_or_default();

        let mut stream = response.bytes_stream();
        let mut body = Vec::new();
        let mut truncated = false;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|err| map_request_error(err, timeout_ms))?;
            if body.len().saturating_add(chunk.len()) > MAX_BODY_BYTES {
                let remaining = MAX_BODY_BYTES.saturating_sub(body.len());
                body.extend_from_slice(&chunk[..remaining]);
                truncated = true;
                break;
            }
            body.extend_from_slice(&chunk);
        }

        let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let size_bytes = body.len() as u64;
        let body_kind = classify_body(&content_type, &body).to_string();
        let body_text = if matches!(body_kind.as_str(), "image" | "binary") {
            None
        } else {
            Some(String::from_utf8_lossy(&body).into_owned())
        };
        let body_base64 = if body_kind == "image" && !truncated && body.len() <= MAX_IMAGE_BYTES {
            Some(STANDARD.encode(&body))
        } else {
            None
        };

        Ok(HttpResponseData {
            status: status.as_u16(),
            status_text,
            headers,
            body_text,
            body_base64,
            body_kind,
            content_type,
            duration_ms,
            size_bytes,
            truncated,
            received_at: now_iso(),
        })
    }
}

fn build_client(
    cookie_jar: Arc<reqwest::cookie::Jar>,
    redirect: Policy,
) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .cookie_provider(cookie_jar)
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .redirect(redirect)
        .user_agent(concat!("Messenger/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|err| format!("初始化 HTTP 客户端失败：{err}"))
}

pub(crate) fn normalize_timeout_ms(value: u64) -> u64 {
    value.clamp(MIN_TIMEOUT_MS, MAX_TIMEOUT_MS)
}

pub(crate) fn active_fields(fields: &[ParamField]) -> impl Iterator<Item = &ParamField> {
    fields
        .iter()
        .filter(|field| field.enabled && !field.name.trim().is_empty())
}

fn parse_method(method: &str) -> Result<Method, String> {
    let method = method.trim();
    if method.is_empty() {
        return Err("请求方法不能为空".to_string());
    }
    Method::from_bytes(method.to_uppercase().as_bytes())
        .map_err(|_| format!("不支持的请求方法：{method}"))
}

fn build_url(raw: &str, params: &[ParamField]) -> Result<Url, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("请求地址不能为空".to_string());
    }
    let mut url =
        Url::parse(raw).map_err(|_| "请求地址无效，请检查格式或环境变量是否已替换".to_string())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("仅支持 http:// 或 https:// 地址".to_string());
    }
    if url.host_str().is_none() {
        return Err("请求地址缺少主机名".to_string());
    }

    {
        let mut fields = active_fields(params).peekable();
        if fields.peek().is_some() {
            let mut query = url.query_pairs_mut();
            for field in fields {
                query.append_pair(field.name.trim(), &field.value);
            }
        }
    }
    Ok(url)
}

fn build_headers(spec: &HttpRequestSpec) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for field in active_fields(&spec.headers) {
        let name = HeaderName::from_bytes(field.name.trim().as_bytes());
        let value = HeaderValue::from_str(&field.value);
        if let (Ok(name), Ok(value)) = (name, value) {
            headers.insert(name, value);
        }
    }

    if !headers.contains_key(CONTENT_TYPE) {
        if let Some(content_type) = body_content_type(&spec.body) {
            if let Ok(value) = HeaderValue::from_str(&content_type) {
                headers.insert(CONTENT_TYPE, value);
            }
        }
    }

    if !headers.contains_key(AUTHORIZATION) {
        if let Some(value) = authorization_value(&spec.auth) {
            if let Ok(value) = HeaderValue::from_str(&value) {
                headers.insert(AUTHORIZATION, value);
            }
        }
    }

    headers
}

pub(crate) fn authorization_value(auth: &RequestAuth) -> Option<String> {
    match auth.r#type.trim().to_ascii_lowercase().as_str() {
        "bearer" => {
            let token = auth.bearer_token.trim();
            if token.is_empty() {
                None
            } else {
                Some(format!("Bearer {token}"))
            }
        }
        "basic" => {
            let credentials = format!("{}:{}", auth.username, auth.password);
            Some(format!("Basic {}", STANDARD.encode(credentials)))
        }
        _ => None,
    }
}

/// multipart 的边界串，同一个请求内使用同一个值即可
const MULTIPART_BOUNDARY: &str = "----rustoneBoundary7MA4YWxkTrZu0gW";

fn body_content_type(body: &RequestBody) -> Option<String> {
    match body.mode.trim().to_ascii_lowercase().as_str() {
        "json" => Some("application/json".to_string()),
        "xml" => Some("application/xml".to_string()),
        "form" | "urlencoded" => Some("application/x-www-form-urlencoded".to_string()),
        "form-data" => Some(format!("multipart/form-data; boundary={MULTIPART_BOUNDARY}")),
        "text" | "raw" => {
            let content_type = body.content_type.trim();
            Some(if content_type.is_empty() {
                "text/plain".to_string()
            } else {
                content_type.to_string()
            })
        }
        _ => None,
    }
}

fn build_body(body: &RequestBody) -> Option<Vec<u8>> {
    match body.mode.trim().to_ascii_lowercase().as_str() {
        "json" | "xml" | "text" | "raw" => Some(body.raw.clone().into_bytes()),
        "form" | "urlencoded" => Some(encode_form(&body.fields).into_bytes()),
        "form-data" => Some(encode_multipart(&body.fields)),
        _ => None,
    }
}

/// 手工拼 multipart，避免给 reqwest 打开 multipart feature。
/// file 类型的字段 value 是 base64，这里解码成真正的二进制。
fn encode_multipart(fields: &[ParamField]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for field in active_fields(fields) {
        out.extend_from_slice(b"--");
        out.extend_from_slice(MULTIPART_BOUNDARY.as_bytes());
        out.extend_from_slice(b"\r\n");

        let is_file = field.r#type.eq_ignore_ascii_case("file") && !field.file_name.trim().is_empty();
        if is_file {
            out.extend_from_slice(
                format!(
                    "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                    field.name.trim(),
                    field.file_name.trim()
                )
                .as_bytes(),
            );
            out.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
            match STANDARD.decode(field.value.as_bytes()) {
                Ok(bytes) => out.extend_from_slice(&bytes),
                Err(_) => out.extend_from_slice(field.value.as_bytes()),
            }
            out.extend_from_slice(b"\r\n");
        } else {
            out.extend_from_slice(
                format!(
                    "Content-Disposition: form-data; name=\"{}\"\r\n\r\n",
                    field.name.trim()
                )
                .as_bytes(),
            );
            out.extend_from_slice(field.value.as_bytes());
            out.extend_from_slice(b"\r\n");
        }
    }
    out.extend_from_slice(b"--");
    out.extend_from_slice(MULTIPART_BOUNDARY.as_bytes());
    out.extend_from_slice(b"--\r\n");
    out
}

pub(crate) fn encode_form(fields: &[ParamField]) -> String {
    let mut url = Url::parse("http://localhost/").expect("固定的表单编码基准地址必须有效");
    {
        let mut query = url.query_pairs_mut();
        for field in active_fields(fields) {
            query.append_pair(field.name.trim(), &field.value);
        }
    }
    url.query().unwrap_or_default().to_string()
}

pub(crate) fn classify_body(content_type: &str, body: &[u8]) -> &'static str {
    let content_type = content_type.to_ascii_lowercase();
    if content_type.contains("json") {
        return "json";
    }
    if !body.is_empty() && serde_json::from_slice::<serde_json::Value>(body).is_ok() {
        return "json";
    }
    if content_type.contains("html") {
        return "html";
    }
    if content_type.contains("xml") {
        return "xml";
    }
    if content_type.trim_start().starts_with("image/") {
        return "image";
    }
    if content_type.starts_with("text/")
        || content_type.contains("javascript")
        || content_type.contains("x-www-form-urlencoded")
    {
        return "text";
    }
    if std::str::from_utf8(body).is_ok() {
        return "text";
    }
    "binary"
}

fn map_request_error(err: reqwest::Error, timeout_ms: u64) -> String {
    if err.is_timeout() {
        format!("请求超时（{} 秒）", timeout_seconds(timeout_ms))
    } else if err.is_connect() {
        "无法连接到服务器，请检查地址或网络".to_string()
    } else {
        format!("请求失败：{err}")
    }
}

fn timeout_seconds(timeout_ms: u64) -> u64 {
    timeout_ms.div_ceil(1_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ParamField;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    fn field(id: &str, enabled: bool, name: &str, value: &str) -> ParamField {
        ParamField {
            id: id.into(),
            enabled,
            name: name.into(),
            value: value.into(),
            ..Default::default()
        }
    }

    fn read_request_line(stream: &mut TcpStream) -> String {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];
        while buffer.len() < 64 * 1024 {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(read) => {
                    buffer.extend_from_slice(&chunk[..read]);
                    if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        String::from_utf8_lossy(&buffer)
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    fn write_response(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) {
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
    }

    fn spawn_json_server(body: &'static str) -> (String, JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let request_line = read_request_line(&mut stream);
            write_response(&mut stream, "200 OK", "application/json", body);
            request_line
        });
        (format!("http://{address}/hello"), handle)
    }

    #[test]
    fn active_fields_skip_disabled_and_empty_names() {
        let fields = vec![
            field("1", true, "q", "rust"),
            field("2", false, "disabled", "x"),
            field("3", true, "   ", "x"),
            field("4", true, "page", "2"),
        ];
        let names: Vec<_> = active_fields(&fields)
            .map(|item| item.name.as_str())
            .collect();
        assert_eq!(names, vec!["q", "page"]);
    }

    #[test]
    fn query_params_are_appended_to_existing_url() {
        let params = vec![
            field("1", true, "keyword", "rust"),
            field("2", false, "skip", "1"),
            field("3", true, "sort", "asc"),
        ];
        let url = build_url("https://example.com/api?x=1", &params).unwrap();
        assert_eq!(url.query(), Some("x=1&keyword=rust&sort=asc"));
    }

    #[test]
    fn invalid_urls_report_chinese_errors() {
        assert_eq!(build_url("", &[]).unwrap_err(), "请求地址不能为空");
        assert!(build_url("{{baseUrl}}/get", &[]).is_err());
        assert!(build_url("ftp://example.com", &[]).is_err());
    }

    #[test]
    fn empty_params_do_not_add_a_trailing_question_mark() {
        let url = build_url("https://example.com/api", &[]).unwrap();
        assert_eq!(url.as_str(), "https://example.com/api");
        let url = build_url(
            "https://example.com/api",
            &[field("1", false, "disabled", "1")],
        )
        .unwrap();
        assert_eq!(url.as_str(), "https://example.com/api");
    }

    #[test]
    fn form_fields_are_url_encoded() {
        let fields = vec![
            field("1", true, "a b", "1&2"),
            field("2", true, "empty", ""),
            field("3", false, "skip", "x"),
        ];
        assert_eq!(encode_form(&fields), "a+b=1%262&empty=");
    }

    #[test]
    fn body_kind_detects_common_content() {
        assert_eq!(classify_body("application/json", b"{bad"), "json");
        assert_eq!(classify_body("text/plain", b"{\"a\":1}"), "json");
        assert_eq!(
            classify_body("text/html; charset=utf-8", b"<p>x</p>"),
            "html"
        );
        assert_eq!(classify_body("application/xml", b"<x/>"), "xml");
        assert_eq!(classify_body("image/png", b"\x89PNG"), "image");
        assert_eq!(classify_body("text/plain", b"hello"), "text");
        assert_eq!(
            classify_body("application/octet-stream", b"\xff\xfe\xfd"),
            "binary"
        );
        assert_eq!(classify_body("application/octet-stream", b"hello"), "text");
    }

    #[test]
    fn authorization_headers_are_built_from_auth_config() {
        assert_eq!(
            authorization_value(&RequestAuth {
                r#type: "bearer".into(),
                bearer_token: "abc".into(),
                ..Default::default()
            }),
            Some("Bearer abc".into())
        );
        assert_eq!(
            authorization_value(&RequestAuth {
                r#type: "bearer".into(),
                bearer_token: "   ".into(),
                ..Default::default()
            }),
            None
        );
        assert_eq!(
            authorization_value(&RequestAuth {
                r#type: "basic".into(),
                username: "user".into(),
                password: "pass".into(),
                ..Default::default()
            }),
            Some("Basic dXNlcjpwYXNz".into())
        );
        assert_eq!(authorization_value(&RequestAuth::default()), None);
    }

    #[test]
    fn explicit_headers_win_over_body_and_auth_defaults() {
        let spec = HttpRequestSpec {
            headers: vec![
                field("1", true, "Authorization", "Custom abc"),
                field("2", true, "Content-Type", "text/custom"),
            ],
            auth: RequestAuth {
                r#type: "bearer".into(),
                bearer_token: "token".into(),
                ..Default::default()
            },
            body: RequestBody {
                mode: "json".into(),
                raw: "{}".into(),
                ..Default::default()
            },
            ..Default::default()
        };
        let headers = build_headers(&spec);
        assert_eq!(headers.get(AUTHORIZATION).unwrap(), "Custom abc");
        assert_eq!(headers.get(CONTENT_TYPE).unwrap(), "text/custom");
    }

    #[test]
    fn content_type_defaults_follow_body_mode() {
        let json = RequestBody {
            mode: "json".into(),
            ..Default::default()
        };
        assert_eq!(
            body_content_type(&json).as_deref(),
            Some("application/json")
        );

        let form = RequestBody {
            mode: "form".into(),
            ..Default::default()
        };
        assert_eq!(
            body_content_type(&form).as_deref(),
            Some("application/x-www-form-urlencoded")
        );

        let raw = RequestBody {
            mode: "raw".into(),
            ..Default::default()
        };
        assert_eq!(body_content_type(&raw).as_deref(), Some("text/plain"));

        let none = RequestBody::default();
        assert_eq!(body_content_type(&none), None);
        assert_eq!(build_body(&none), None);
    }

    #[test]
    fn timeout_is_clamped_to_supported_range() {
        assert_eq!(normalize_timeout_ms(0), 1_000);
        assert_eq!(normalize_timeout_ms(500), 1_000);
        assert_eq!(normalize_timeout_ms(30_000), 30_000);
        assert_eq!(normalize_timeout_ms(300_000), 300_000);
        assert_eq!(normalize_timeout_ms(999_999), 300_000);
        assert_eq!(timeout_seconds(1_001), 2);
    }

    #[test]
    fn empty_bearer_does_not_create_authorization() {
        let spec = HttpRequestSpec {
            auth: RequestAuth {
                r#type: "bearer".into(),
                bearer_token: String::new(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(build_headers(&spec).get(AUTHORIZATION).is_none());
    }

    #[tokio::test]
    async fn sends_get_and_reads_json_response() {
        let body = r#"{"ok":true}"#;
        let (base, server) = spawn_json_server(body);
        let engine = HttpEngine::new().unwrap();
        let spec = HttpRequestSpec {
            method: "GET".into(),
            url: format!("{base}?existing=1"),
            params: vec![field("1", true, "q", "rust")],
            timeout_ms: 5_000,
            ..Default::default()
        };

        let response = engine.send("test-get", spec).await.unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.status_text, "OK");
        assert_eq!(response.body_kind, "json");
        assert_eq!(response.body_text.as_deref(), Some(body));
        assert_eq!(response.content_type, "application/json");
        assert_eq!(response.size_bytes, body.len() as u64);
        assert!(!response.truncated);

        let request_line = server.join().unwrap();
        assert!(
            request_line.starts_with("GET /hello?existing=1&q=rust "),
            "{request_line}"
        );
    }

    #[tokio::test]
    async fn redirect_following_can_be_toggled() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for _ in 0..3 {
                let (mut stream, _) = listener.accept().unwrap();
                let request_line = read_request_line(&mut stream);
                if request_line.starts_with("GET /start ") {
                    let response = "HTTP/1.1 302 Found\r\nLocation: /final\r\nContent-Type: text/plain\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                } else {
                    write_response(&mut stream, "200 OK", "text/plain", "ok");
                }
            }
        });

        let engine = HttpEngine::new().unwrap();
        let mut spec = HttpRequestSpec {
            method: "GET".into(),
            url: format!("http://{address}/start"),
            timeout_ms: 5_000,
            ..Default::default()
        };
        let followed = engine.send("follow", spec.clone()).await.unwrap();
        assert_eq!(followed.status, 200);
        assert_eq!(followed.body_text.as_deref(), Some("ok"));

        spec.follow_redirects = false;
        let stopped = engine.send("stop", spec).await.unwrap();
        assert_eq!(stopped.status, 302);
        let location = stopped
            .headers
            .iter()
            .find(|header| header.name.eq_ignore_ascii_case("location"))
            .map(|header| header.value.as_str());
        assert_eq!(location, Some("/final"));

        server.join().unwrap();
    }

    #[tokio::test]
    async fn cancellation_returns_chinese_error() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let _ = read_request_line(&mut stream);
            thread::sleep(Duration::from_millis(500));
            let _ = write_response(&mut stream, "200 OK", "text/plain", "late");
        });

        let engine = Arc::new(HttpEngine::new().unwrap());
        let spec = HttpRequestSpec {
            method: "GET".into(),
            url: format!("http://{address}/slow"),
            timeout_ms: 30_000,
            ..Default::default()
        };
        let sender = Arc::clone(&engine);
        let task = tokio::spawn(async move { sender.send("cancel-me", spec).await });

        for _ in 0..200 {
            if engine
                .cancellations
                .lock()
                .unwrap()
                .contains_key("cancel-me")
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        assert!(engine.cancel("cancel-me"));
        let result = task.await.unwrap();
        assert_eq!(result.unwrap_err(), "请求已取消");
        server.join().unwrap();
    }
}
