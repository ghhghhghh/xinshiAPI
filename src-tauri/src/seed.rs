//! 首次启动写入的示例数据。ID 固定，便于前端 mock 对齐。

use rusqlite::{Connection, params};
use serde::Serialize;

use crate::models::{
    ApiRequest, Collection, EnvVariable, Environment, ParamField, RequestAuth, RequestBody,
};

const SEED_TIME: &str = "2026-10-22T08:00:00.000Z";

pub fn seed_if_empty(conn: &mut Connection) -> rusqlite::Result<bool> {
    let collection_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM collections", [], |row| row.get(0))?;
    if collection_count > 0 {
        return Ok(false);
    }

    let tx = conn.transaction()?;

    for environment in seed_environments() {
        let variables_json = to_json(&environment.variables);
        tx.execute(
            "INSERT OR IGNORE INTO environments (id, name, variables_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                environment.id,
                environment.name,
                variables_json,
                environment.created_at,
                environment.updated_at
            ],
        )?;
    }

    for collection in seed_collections() {
        tx.execute(
            "INSERT OR IGNORE INTO collections (id, name, description, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                collection.id,
                collection.name,
                collection.description,
                collection.sort_order,
                collection.created_at,
                collection.updated_at
            ],
        )?;
    }

    for request in seed_requests() {
        let params_json = to_json(&request.params);
        let headers_json = to_json(&request.headers);
        let body_json = to_json(&request.body);
        let auth_json = to_json(&request.auth);
        tx.execute(
            "INSERT OR IGNORE INTO requests
                (id, collection_id, name, method, url, params_json, headers_json, body_json,
                 auth_json, description, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                request.id,
                request.collection_id,
                request.name,
                request.method,
                request.url,
                params_json,
                headers_json,
                body_json,
                auth_json,
                request.description,
                request.sort_order,
                request.updated_at,
                request.updated_at
            ],
        )?;
    }

    tx.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('active_environment', 'env-demo')",
        [],
    )?;

    tx.commit()?;
    Ok(true)
}

fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("示例数据序列化失败")
}

fn seed_collections() -> Vec<Collection> {
    vec![
        Collection {
            id: "col-demo".into(),
            parent_id: None,
            headers: Vec::new(),
            auth: RequestAuth::default(),
            name: "入门示例".into(),
            description: "带参数的 GET、JSON POST 与鉴权示例".into(),
            sort_order: 0,
            created_at: SEED_TIME.into(),
            updated_at: SEED_TIME.into(),
        },
        Collection {
            id: "col-jsonplaceholder".into(),
            parent_id: None,
            headers: Vec::new(),
            auth: RequestAuth::default(),
            name: "JSONPlaceholder".into(),
            description: "公开测试接口，无需鉴权".into(),
            sort_order: 1,
            created_at: SEED_TIME.into(),
            updated_at: SEED_TIME.into(),
        },
    ]
}

fn seed_environments() -> Vec<Environment> {
    vec![
        Environment {
            id: "env-demo".into(),
            name: "示例环境".into(),
            base_url: "https://httpbin.org".into(),
            variables: vec![
                EnvVariable {
                    id: "v-base".into(),
                    enabled: true,
                    name: "baseUrl".into(),
                    value: "https://httpbin.org".into(),
                    description: "示例接口地址".into(),
                },
                EnvVariable {
                    id: "v-token".into(),
                    enabled: true,
                    name: "token".into(),
                    value: "demo-token-123".into(),
                    description: "演示用 Token".into(),
                },
                EnvVariable {
                    id: "v-page".into(),
                    enabled: true,
                    name: "pageSize".into(),
                    value: "10".into(),
                    description: "分页大小".into(),
                },
            ],
            created_at: SEED_TIME.into(),
            updated_at: SEED_TIME.into(),
        },
        Environment {
            id: "env-local".into(),
            name: "本地开发".into(),
            base_url: "http://localhost:3000".into(),
            variables: vec![EnvVariable {
                id: "v-local-base".into(),
                enabled: true,
                name: "baseUrl".into(),
                value: "http://localhost:3000".into(),
                description: "本地服务地址".into(),
            }],
            created_at: SEED_TIME.into(),
            updated_at: SEED_TIME.into(),
        },
    ]
}

fn param(
    id: &str,
    name: &str,
    r#type: &str,
    value: &str,
    description: &str,
    options: &[&str],
) -> ParamField {
    ParamField {
        id: id.into(),
        enabled: true,
        name: name.into(),
        r#type: r#type.into(),
        value: value.into(),
        file_name: String::new(),
        description: description.into(),
        options: options.iter().map(|item| (*item).to_string()).collect(),
    }
}

fn auth_none() -> RequestAuth {
    RequestAuth::default()
}

fn body_none() -> RequestBody {
    RequestBody::default()
}

fn json_body(raw: &str) -> RequestBody {
    RequestBody {
        mode: "json".into(),
        raw: raw.into(),
        content_type: String::new(),
        fields: Vec::new(),
    }
}

fn seed_requests() -> Vec<ApiRequest> {
    vec![
        ApiRequest {
            id: "req-demo-get".into(),
            collection_id: "col-demo".into(),
            name: "查询用户列表".into(),
            method: "GET".into(),
            url: "{{baseUrl}}/get".into(),
            description: "带查询参数、Header 与枚举选项的入门 GET 示例".into(),
            params: vec![
                param(
                    "p-get-keyword",
                    "keyword",
                    "string",
                    "rust",
                    "搜索关键词",
                    &[],
                ),
                param(
                    "p-get-sort",
                    "sort",
                    "enum",
                    "asc",
                    "排序方向",
                    &["asc", "desc"],
                ),
                param(
                    "p-get-role",
                    "role",
                    "enum",
                    "admin",
                    "角色筛选",
                    &["admin", "editor", "viewer"],
                ),
                param("p-get-page", "page", "number", "1", "页码", &[]),
                param(
                    "p-get-verbose",
                    "verbose",
                    "boolean",
                    "false",
                    "返回调试信息",
                    &[],
                ),
            ],
            headers: vec![
                param(
                    "h-get-accept",
                    "Accept",
                    "string",
                    "application/json",
                    "期望响应类型",
                    &[],
                ),
                param(
                    "h-get-source",
                    "X-Demo-Source",
                    "string",
                    "信使",
                    "示例自定义头",
                    &[],
                ),
            ],
            body: body_none(),
            auth: auth_none(),
            extractors: Vec::new(),
            response_labels: Vec::new(),
            sort_order: 0,
            updated_at: SEED_TIME.into(),
        },
        ApiRequest {
            id: "req-demo-post".into(),
            collection_id: "col-demo".into(),
            name: "创建订单（JSON）".into(),
            method: "POST".into(),
            url: "{{baseUrl}}/post".into(),
            description: "提交 JSON 订单并查看响应回显".into(),
            params: Vec::new(),
            headers: vec![param(
                "h-post-content-type",
                "Content-Type",
                "string",
                "application/json",
                "JSON 请求体",
                &[],
            )],
            body: json_body(
                r#"{"orderNo":"A-1001","amount":128.5,"channel":"web","items":[{"sku":"RUST-01","qty":2}]}"#,
            ),
            auth: auth_none(),
            extractors: Vec::new(),
            response_labels: Vec::new(),
            sort_order: 1,
            updated_at: SEED_TIME.into(),
        },
        ApiRequest {
            id: "req-demo-auth".into(),
            collection_id: "col-demo".into(),
            name: "Bearer 鉴权示例".into(),
            method: "GET".into(),
            url: "{{baseUrl}}/bearer".into(),
            description: "自动附加 Bearer Token（来自 {{token}}）".into(),
            params: Vec::new(),
            headers: Vec::new(),
            body: body_none(),
            auth: RequestAuth {
                r#type: "bearer".into(),
                bearer_token: "{{token}}".into(),
                username: String::new(),
                password: String::new(),
                key_name: "Authorization".into(),
                key_value: String::new(),
                key_in: "header".into(),
            },
            extractors: Vec::new(),
            response_labels: Vec::new(),
            sort_order: 2,
            updated_at: SEED_TIME.into(),
        },
        ApiRequest {
            id: "req-demo-404".into(),
            collection_id: "col-demo".into(),
            name: "错误状态示例".into(),
            method: "GET".into(),
            url: "{{baseUrl}}/status/404".into(),
            description: "httpbin 返回 404，用于观察错误状态与历史记录".into(),
            params: Vec::new(),
            headers: Vec::new(),
            body: body_none(),
            auth: auth_none(),
            extractors: Vec::new(),
            response_labels: Vec::new(),
            sort_order: 3,
            updated_at: SEED_TIME.into(),
        },
        ApiRequest {
            id: "req-jp-user".into(),
            collection_id: "col-jsonplaceholder".into(),
            name: "获取用户 1".into(),
            method: "GET".into(),
            url: "https://jsonplaceholder.typicode.com/users/1".into(),
            description: "公开接口，返回用户资料与嵌套地址信息".into(),
            params: Vec::new(),
            headers: Vec::new(),
            body: body_none(),
            auth: auth_none(),
            extractors: Vec::new(),
            response_labels: Vec::new(),
            sort_order: 0,
            updated_at: SEED_TIME.into(),
        },
        ApiRequest {
            id: "req-jp-create".into(),
            collection_id: "col-jsonplaceholder".into(),
            name: "创建文章".into(),
            method: "POST".into(),
            url: "https://jsonplaceholder.typicode.com/posts".into(),
            description: "向公开接口提交一篇文章并查看返回结果".into(),
            params: Vec::new(),
            headers: Vec::new(),
            body: json_body(r#"{"title":"信使测试","body":"hello from messenger","userId":1}"#),
            auth: auth_none(),
            extractors: Vec::new(),
            response_labels: Vec::new(),
            sort_order: 1,
            updated_at: SEED_TIME.into(),
        },
    ]
}
