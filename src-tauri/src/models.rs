//! 前后端共用的数据契约。
//!
//! 所有结构体的 JSON 字段均为 camelCase；命令参数由 `tauri::command(rename_all = "camelCase")`
//! 负责转换。`type` 是 Rust 关键字，因此使用 `r#type`，序列化后仍是 `type`。

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

/// 统一使用毫秒精度的 UTC ISO8601 时间。
pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ParamField {
    pub id: String,
    pub enabled: bool,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    /// file 类型时是文件内容的 base64
    pub value: String,
    /// file 类型的原始文件名
    pub file_name: String,
    pub description: String,
    pub options: Vec<String>,
}

impl Default for ParamField {
    fn default() -> Self {
        Self {
            id: String::new(),
            enabled: true,
            name: String::new(),
            r#type: "string".to_string(),
            value: String::new(),
            file_name: String::new(),
            description: String::new(),
            options: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RequestBody {
    pub mode: String,
    pub raw: String,
    pub content_type: String,
    pub fields: Vec<ParamField>,
}

impl Default for RequestBody {
    fn default() -> Self {
        Self {
            mode: "none".to_string(),
            raw: String::new(),
            content_type: String::new(),
            fields: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RequestAuth {
    #[serde(rename = "type")]
    pub r#type: String,
    pub bearer_token: String,
    pub username: String,
    pub password: String,
    /// API Key 的请求头名
    pub key_name: String,
    /// API Key 的值
    pub key_value: String,
    /// API Key 放哪：header / query
    pub key_in: String,
}

impl Default for RequestAuth {
    fn default() -> Self {
        Self {
            r#type: "none".to_string(),
            bearer_token: String::new(),
            username: String::new(),
            password: String::new(),
            key_name: "Authorization".to_string(),
            key_value: String::new(),
            key_in: "header".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ResponseLabel {
    pub path: String,
    pub label: String,
    pub r#type: String,
    pub description: String,
    pub example: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ExtractRule {
    pub id: String,
    pub enabled: bool,
    /// 响应 JSON 中的取值路径，如 data.token
    pub path: String,
    /// 写回哪个环境变量
    pub variable: String,
}

impl Default for ExtractRule {
    fn default() -> Self {
        Self {
            id: String::new(),
            enabled: true,
            path: String::new(),
            variable: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ApiRequest {
    pub id: String,
    pub collection_id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub description: String,
    pub params: Vec<ParamField>,
    pub headers: Vec<ParamField>,
    pub body: RequestBody,
    pub auth: RequestAuth,
    pub extractors: Vec<ExtractRule>,
    pub response_labels: Vec<ResponseLabel>,
    pub sort_order: i64,
    pub updated_at: String,
}

impl Default for ApiRequest {
    fn default() -> Self {
        Self {
            id: String::new(),
            collection_id: String::new(),
            name: String::new(),
            method: "GET".to_string(),
            url: String::new(),
            description: String::new(),
            params: Vec::new(),
            headers: Vec::new(),
            body: RequestBody::default(),
            auth: RequestAuth::default(),
            extractors: Vec::new(),
            response_labels: Vec::new(),
            sort_order: 0,
            updated_at: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Collection {
    pub id: String,
    /// 上级文件夹；None 表示顶层（一个「项目」）
    pub parent_id: Option<String>,
    pub name: String,
    pub description: String,
    /// 文件夹级鉴权，子级请求选「继承」时生效
    pub auth: RequestAuth,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for Collection {
    fn default() -> Self {
        Self {
            id: String::new(),
            parent_id: None,
            name: String::new(),
            description: String::new(),
            auth: RequestAuth::default(),
            sort_order: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EnvVariable {
    pub id: String,
    pub enabled: bool,
    pub name: String,
    pub value: String,
    pub description: String,
}

impl Default for EnvVariable {
    fn default() -> Self {
        Self {
            id: String::new(),
            enabled: true,
            name: String::new(),
            value: String::new(),
            description: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Environment {
    pub id: String,
    pub name: String,
    /// 前置 URL：请求地址不是完整地址时拼在路径前面
    pub base_url: String,
    pub variables: Vec<EnvVariable>,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            base_url: String::new(),
            variables: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Workspace {
    pub collections: Vec<Collection>,
    pub requests: Vec<ApiRequest>,
    pub environments: Vec<Environment>,
    pub active_environment_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HttpRequestSpec {
    pub method: String,
    pub url: String,
    pub params: Vec<ParamField>,
    pub headers: Vec<ParamField>,
    pub body: RequestBody,
    pub auth: RequestAuth,
    pub timeout_ms: u64,
    pub follow_redirects: bool,
}

impl Default for HttpRequestSpec {
    fn default() -> Self {
        Self {
            method: "GET".to_string(),
            url: String::new(),
            params: Vec::new(),
            headers: Vec::new(),
            body: RequestBody::default(),
            auth: RequestAuth::default(),
            timeout_ms: 30_000,
            follow_redirects: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpResponseData {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<ResponseHeader>,
    pub body_text: Option<String>,
    pub body_base64: Option<String>,
    /// json | text | html | xml | image | binary
    pub body_kind: String,
    pub content_type: String,
    pub duration_ms: u64,
    pub size_bytes: u64,
    pub truncated: bool,
    pub received_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub request_id: String,
    pub request_name: String,
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub duration_ms: Option<u64>,
    pub ok: bool,
    pub created_at: String,
    pub request: ApiRequest,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn camel_case_contract_uses_type_keys() {
        let field = ParamField {
            id: "p1".into(),
            enabled: true,
            name: "role".into(),
            r#type: "enum".into(),
            value: "admin".into(),
            file_name: String::new(),
            description: "角色".into(),
            options: vec!["admin".into(), "viewer".into()],
        };
        let value = serde_json::to_value(&field).unwrap();
        assert_eq!(value["type"], json!("enum"));
        assert_eq!(value["options"], json!(["admin", "viewer"]));
        assert!(value.get("r#type").is_none());

        let request = ApiRequest {
            id: "r1".into(),
            collection_id: "c1".into(),
            name: "请求".into(),
            ..Default::default()
        };
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["collectionId"], json!("c1"));
        assert_eq!(value["sortOrder"], json!(0));
        assert_eq!(value["body"]["mode"], json!("none"));
        assert_eq!(value["auth"]["type"], json!("none"));
    }

    #[test]
    fn missing_optional_fields_fall_back_to_defaults() {
        let field: ParamField = serde_json::from_value(json!({ "name": "q" })).unwrap();
        assert!(field.enabled);
        assert_eq!(field.r#type, "string");
        assert!(field.options.is_empty());

        let spec: HttpRequestSpec =
            serde_json::from_value(json!({ "url": "https://example.com" })).unwrap();
        assert_eq!(spec.timeout_ms, 30_000);
        assert!(spec.follow_redirects);
        assert_eq!(spec.method, "GET");
    }
}
