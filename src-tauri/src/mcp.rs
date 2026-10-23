//! 随安装包分发的 MCP sidecar：通过 stdio 暴露信使本地数据库。

use std::env;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::db::Database;
use crate::models::{
    ApiRequest, Collection, EnvVariable, Environment, ExtractRule, ParamField, RequestAuth,
    RequestBody, ResponseLabel, Workspace, now_iso,
};

const PROTOCOL_VERSION: &str = "2025-11-25";
const METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];
const PARAM_TYPES: &[&str] = &["string", "number", "boolean", "enum", "file"];
const BODY_MODES: &[&str] = &["none", "form-data", "urlencoded", "json", "xml", "text"];
const AUTH_TYPES: &[&str] = &["inherit", "none", "apikey", "bearer", "basic"];

pub fn sidecar_file_name() -> &'static str {
    if cfg!(windows) {
        "xingshi-mcp.exe"
    } else {
        "xingshi-mcp"
    }
}

pub fn default_db_path() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("XINGSHI_DB") {
        if !path.trim().is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    let home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .map_err(|_| "无法定位用户主目录".to_string())?;
    if cfg!(windows) {
        let base = env::var("APPDATA").unwrap_or_else(|_| format!("{home}\\AppData\\Roaming"));
        Ok(PathBuf::from(base).join("com.zx.rustone").join("messenger.db"))
    } else if cfg!(target_os = "macos") {
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("com.zx.rustone")
            .join("messenger.db"))
    } else {
        let base = env::var("XDG_DATA_HOME")
            .unwrap_or_else(|_| format!("{home}/.local/share"));
        Ok(PathBuf::from(base).join("com.zx.rustone").join("messenger.db"))
    }
}

pub fn run_stdio() -> Result<(), String> {
    let db_path = default_db_path()?;
    eprintln!("[xingshi-mcp] db={}", db_path.display());
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    for line in stdin.lock().lines() {
        let line = line.map_err(|err| format!("读取 stdin 失败：{err}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("[xingshi-mcp] 无法解析 JSON：{err}");
                continue;
            }
        };
        // MCP notification 不需要响应
        if request.get("id").is_none() {
            continue;
        }
        let response = dispatch(&request, &db_path);
        writeln!(stdout, "{}", response).map_err(|err| format!("写入 stdout 失败：{err}"))?;
        stdout
            .flush()
            .map_err(|err| format!("刷新 stdout 失败：{err}"))?;
    }
    Ok(())
}

fn dispatch(request: &Value, db_path: &Path) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let params = request.get("params").cloned().unwrap_or(Value::Null);

    let result = match method {
        "initialize" => Ok(initialize_result(&params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(tools_list()),
        "tools/call" => Ok(call_tool(&params, db_path)),
        "notifications/initialized" => Ok(json!({})),
        _ => Err(format!("Method not found: {method}")),
    };

    match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(message) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": message }
        }),
    }
}

fn initialize_result(params: &Value) -> Value {
    let protocol = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or(PROTOCOL_VERSION);
    json!({
        "protocolVersion": protocol,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "xingshi-api", "version": env!("CARGO_PKG_VERSION") },
        "instructions": "信使本地 MCP：先 list_workspace 查看已有项目；再用 import_api_tree 批量写入接口、参数和设计视图字段。导入后切回信使会自动刷新。"
    })
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "list_workspace",
                "description": "列出信使里的项目、模块、接口摘要和环境。导入前先调用，按路径和名称避免重复。",
                "inputSchema": { "type": "object", "properties": {} },
                "annotations": { "readOnlyHint": true }
            },
            {
                "name": "get_status",
                "description": "查看信使数据库路径、外部导入版本号、当前环境和项目/接口数量。",
                "inputSchema": { "type": "object", "properties": {} },
                "annotations": { "readOnlyHint": true }
            },
            {
                "name": "import_api_tree",
                "description": "按「项目 → 模块/文件夹 → 接口」批量写入信使。按路径+名称幂等 upsert，不会删除已有内容。每个接口可包含 params、headers、body、auth、extractors、responseLabels；可带 environment；dryRun=true 只预览。",
                "inputSchema": import_schema(),
                "annotations": { "readOnlyHint": false, "destructiveHint": false, "idempotentHint": true }
            },
            {
                "name": "upsert_environment",
                "description": "新建或更新信使环境：name、baseUrl、variables、activate。",
                "inputSchema": environment_schema(),
                "annotations": { "readOnlyHint": false, "destructiveHint": false, "idempotentHint": true }
            }
        ]
    })
}

fn import_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "projects": {
                "type": "array",
                "description": "顶层项目数组",
                "items": { "$ref": "#/$defs/node" }
            },
            "environment": { "$ref": "#/$defs/environment" },
            "dryRun": { "type": "boolean", "description": "只返回创建/更新清单，不写入" }
        },
        "required": ["projects"],
        "$defs": {
            "param": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "enabled": { "type": "boolean" },
                    "name": { "type": "string", "description": "参数名" },
                    "type": { "type": "string", "enum": PARAM_TYPES },
                    "value": { "type": "string", "description": "示例值/实际值" },
                    "fileName": { "type": "string" },
                    "description": { "type": "string" },
                    "options": { "type": "array", "items": { "type": "string" }, "description": "enum 的自定义候选项" }
                },
                "required": ["name"]
            },
            "body": {
                "type": "object",
                "properties": {
                    "mode": { "type": "string", "enum": BODY_MODES },
                    "raw": { "type": "string" },
                    "contentType": { "type": "string" },
                    "fields": { "type": "array", "items": { "$ref": "#/$defs/param" } }
                }
            },
            "auth": {
                "type": "object",
                "properties": {
                    "type": { "type": "string", "enum": AUTH_TYPES },
                    "bearerToken": { "type": "string" },
                    "username": { "type": "string" },
                    "password": { "type": "string" },
                    "keyName": { "type": "string" },
                    "keyValue": { "type": "string" },
                    "keyIn": { "type": "string", "enum": ["header", "query"] }
                }
            },
            "extractor": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "enabled": { "type": "boolean" },
                    "path": { "type": "string", "description": "响应 JSON 路径，如 data.token" },
                    "variable": { "type": "string", "description": "写回的环境变量名" }
                },
                "required": ["path", "variable"]
            },
            "responseLabel": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "字段路径，数组用 data.list[].name" },
                    "label": { "type": "string", "description": "中文名" },
                    "type": { "type": "string" },
                    "description": { "type": "string" },
                    "example": { "type": "string" }
                },
                "required": ["path", "label"]
            },
            "request": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "name": { "type": "string" },
                    "method": { "type": "string", "enum": METHODS },
                    "url": { "type": "string", "description": "绝对地址或相对路径；相对路径配合环境 baseUrl" },
                    "path": { "type": "string", "description": "url 的别名" },
                    "description": { "type": "string" },
                    "params": { "type": "array", "items": { "$ref": "#/$defs/param" } },
                    "headers": { "type": "array", "items": { "$ref": "#/$defs/param" } },
                    "body": { "$ref": "#/$defs/body" },
                    "auth": { "$ref": "#/$defs/auth" },
                    "extractors": { "type": "array", "items": { "$ref": "#/$defs/extractor" } },
                    "responseLabels": { "type": "array", "items": { "$ref": "#/$defs/responseLabel" } }
                },
                "required": ["name"]
            },
            "node": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "name": { "type": "string" },
                    "description": { "type": "string" },
                    "auth": { "$ref": "#/$defs/auth" },
                    "folders": { "type": "array", "items": { "$ref": "#/$defs/node" } },
                    "requests": { "type": "array", "items": { "$ref": "#/$defs/request" } }
                },
                "required": ["name"]
            },
            "envVar": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "enabled": { "type": "boolean" },
                    "name": { "type": "string" },
                    "value": { "type": "string" },
                    "description": { "type": "string" }
                },
                "required": ["name"]
            },
            "environment": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "name": { "type": "string" },
                    "baseUrl": { "type": "string", "description": "环境前置 URL" },
                    "activate": { "type": "boolean" },
                    "variables": { "type": "array", "items": { "$ref": "#/$defs/envVar" } }
                },
                "required": ["name"]
            }
        }
    })
}

fn environment_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": { "type": "string" },
            "name": { "type": "string" },
            "baseUrl": { "type": "string" },
            "activate": { "type": "boolean" },
            "variables": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "enabled": { "type": "boolean" },
                        "name": { "type": "string" },
                        "value": { "type": "string" },
                        "description": { "type": "string" }
                    },
                    "required": ["name"]
                }
            }
        },
        "required": ["name"]
    })
}

fn call_tool(params: &Value, db_path: &Path) -> Value {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let result = (|| -> Result<Value, String> {
        let db = Database::open(db_path.to_path_buf())?;
        match name {
            "list_workspace" => list_workspace_value(&db, db_path),
            "get_status" => get_status_value(&db, db_path),
            "import_api_tree" => {
                let input: ImportInput = serde_json::from_value(arguments)
                    .map_err(|err| format!("import_api_tree 参数无效：{err}"))?;
                import_tree(&db, db_path, input)
            }
            "upsert_environment" => {
                let input: EnvironmentInput = serde_json::from_value(arguments)
                    .map_err(|err| format!("upsert_environment 参数无效：{err}"))?;
                upsert_environment_tool(&db, db_path, input)
            }
            _ => Err(format!("未知工具：{name}")),
        }
    })();

    match result {
        Ok(value) => json!({
            "content": [{ "type": "text", "text": serde_json::to_string_pretty(&value).unwrap_or_default() }],
            "isError": false
        }),
        Err(message) => json!({
            "content": [{ "type": "text", "text": message }],
            "isError": true
        }),
    }
}

fn list_workspace_value(db: &Database, db_path: &Path) -> Result<Value, String> {
    let workspace = db.load_workspace()?;
    let roots: Vec<Value> = workspace
        .collections
        .iter()
        .filter(|collection| collection.parent_id.is_none())
        .map(|collection| folder_node(collection, &workspace))
        .collect();
    let environments: Vec<Value> = workspace
        .environments
        .iter()
        .map(|environment| {
            json!({
                "id": environment.id,
                "name": environment.name,
                "baseUrl": environment.base_url,
                "active": workspace.active_environment_id.as_deref() == Some(environment.id.as_str()),
                "variables": environment.variables.iter().map(|variable| json!({
                    "name": variable.name,
                    "enabled": variable.enabled,
                    "description": variable.description
                })).collect::<Vec<_>>()
            })
        })
        .collect();
    Ok(json!({
        "dbPath": db_path.display().to_string(),
        "revision": db.workspace_revision()?,
        "activeEnvironmentId": workspace.active_environment_id,
        "environments": environments,
        "projects": roots
    }))
}

fn folder_node(collection: &Collection, workspace: &Workspace) -> Value {
    let folders: Vec<Value> = workspace
        .collections
        .iter()
        .filter(|child| child.parent_id.as_deref() == Some(collection.id.as_str()))
        .map(|child| folder_node(child, workspace))
        .collect();
    let requests: Vec<Value> = workspace
        .requests
        .iter()
        .filter(|request| request.collection_id == collection.id)
        .map(|request| {
            json!({
                "id": request.id,
                "name": request.name,
                "method": request.method,
                "url": request.url,
                "paramCount": request.params.len(),
                "responseLabelCount": request.response_labels.len()
            })
        })
        .collect();
    json!({
        "id": collection.id,
        "name": collection.name,
        "description": collection.description,
        "folders": folders,
        "requests": requests
    })
}

fn get_status_value(db: &Database, db_path: &Path) -> Result<Value, String> {
    let workspace = db.load_workspace()?;
    let projects = workspace
        .collections
        .iter()
        .filter(|collection| collection.parent_id.is_none())
        .count();
    Ok(json!({
        "dbPath": db_path.display().to_string(),
        "revision": db.workspace_revision()?,
        "activeEnvironmentId": workspace.active_environment_id,
        "counts": {
            "projects": projects,
            "folders": workspace.collections.len(),
            "requests": workspace.requests.len(),
            "environments": workspace.environments.len()
        }
    }))
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct BodyInput {
    mode: Option<String>,
    raw: Option<String>,
    content_type: Option<String>,
    fields: Option<Vec<ParamField>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct AuthInput {
    #[serde(rename = "type")]
    r#type: Option<String>,
    bearer_token: Option<String>,
    username: Option<String>,
    password: Option<String>,
    key_name: Option<String>,
    key_value: Option<String>,
    key_in: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestInput {
    #[serde(default)]
    id: Option<String>,
    name: String,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    params: Option<Vec<ParamField>>,
    #[serde(default)]
    headers: Option<Vec<ParamField>>,
    #[serde(default)]
    body: Option<BodyInput>,
    #[serde(default)]
    auth: Option<AuthInput>,
    #[serde(default)]
    extractors: Option<Vec<ExtractRule>>,
    #[serde(default)]
    response_labels: Option<Vec<ResponseLabel>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NodeInput {
    #[serde(default)]
    id: Option<String>,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    auth: Option<AuthInput>,
    #[serde(default)]
    folders: Option<Vec<NodeInput>>,
    #[serde(default)]
    requests: Option<Vec<RequestInput>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnvVariableInput {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
    name: String,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnvironmentInput {
    #[serde(default)]
    id: Option<String>,
    name: String,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    activate: Option<bool>,
    #[serde(default)]
    variables: Option<Vec<EnvVariableInput>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportInput {
    projects: Vec<NodeInput>,
    #[serde(default)]
    environment: Option<EnvironmentInput>,
    #[serde(default)]
    dry_run: Option<bool>,
}

fn import_tree(db: &Database, db_path: &Path, input: ImportInput) -> Result<Value, String> {
    let dry_run = input.dry_run.unwrap_or(false);
    let mut workspace = db.load_workspace()?;
    let mut actions: Vec<Value> = Vec::new();
    let mut changed = false;

    if let Some(environment) = input.environment {
        let (action, did_change) = upsert_environment(db, &mut workspace, environment, dry_run)?;
        actions.push(action);
        changed |= did_change;
    }

    for project in &input.projects {
        let (project_actions, did_change) =
            upsert_node(db, &mut workspace, project, None, "project", &[], dry_run)?;
        actions.extend(project_actions);
        changed |= did_change;
    }

    let revision = if changed && !dry_run {
        Some(db.bump_workspace_revision()?)
    } else {
        db.workspace_revision()?
    };

    Ok(json!({
        "dryRun": dry_run,
        "dbPath": db_path.display().to_string(),
        "revision": revision,
        "changed": changed,
        "actions": actions
    }))
}

fn upsert_environment_tool(
    db: &Database,
    db_path: &Path,
    input: EnvironmentInput,
) -> Result<Value, String> {
    let mut workspace = db.load_workspace()?;
    let (action, changed) = upsert_environment(db, &mut workspace, input, false)?;
    let revision = if changed {
        Some(db.bump_workspace_revision()?)
    } else {
        db.workspace_revision()?
    };
    Ok(json!({
        "dbPath": db_path.display().to_string(),
        "revision": revision,
        "action": action
    }))
}

fn upsert_environment(
    db: &Database,
    workspace: &mut Workspace,
    input: EnvironmentInput,
    dry_run: bool,
) -> Result<(Value, bool), String> {
    let existing_index = workspace
        .environments
        .iter()
        .position(|environment| environment.name == input.name);
    let mut changed = true;
    let id;

    if let Some(index) = existing_index {
        let mut environment = workspace.environments[index].clone();
        if let Some(base_url) = &input.base_url {
            environment.base_url = base_url.clone();
        }
        if let Some(variables) = &input.variables {
            environment.variables = normalize_variables(variables);
        }
        environment.updated_at = now_iso();
        id = environment.id.clone();
        if !dry_run {
            db.save_environment(&environment)?;
        }
        workspace.environments[index] = environment;
    } else {
        let at = now_iso();
        let environment = Environment {
            id: input.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
            name: input.name.clone(),
            base_url: input.base_url.clone().unwrap_or_default(),
            variables: normalize_variables(input.variables.as_deref().unwrap_or(&[])),
            created_at: at.clone(),
            updated_at: at,
        };
        id = environment.id.clone();
        if !dry_run {
            db.save_environment(&environment)?;
        }
        workspace.environments.push(environment);
        changed = true;
    }

    if input.activate.unwrap_or(false) {
        if !dry_run {
            db.set_active_environment(Some(id.as_str()))?;
        }
        workspace.active_environment_id = Some(id.clone());
        changed = true;
    }

    Ok((
        json!({
            "action": if existing_index.is_some() { "update" } else { "create" },
            "kind": "environment",
            "path": input.name
        }),
        changed,
    ))
}

fn upsert_node(
    db: &Database,
    workspace: &mut Workspace,
    node: &NodeInput,
    parent_id: Option<&str>,
    kind: &str,
    parent_path: &[String],
    dry_run: bool,
) -> Result<(Vec<Value>, bool), String> {
    let mut path_parts = parent_path.to_vec();
    path_parts.push(node.name.clone());
    let path_text = path_parts.join(" / ");
    let mut actions = Vec::new();
    let mut changed = false;

    let existing_index = workspace.collections.iter().position(|collection| {
        collection.parent_id.as_deref() == parent_id && collection.name == node.name
    });

    let collection_id = if let Some(index) = existing_index {
        let mut collection = workspace.collections[index].clone();
        let mut did_change = false;
        if let Some(description) = &node.description {
            if collection.description != *description {
                collection.description = description.clone();
                did_change = true;
            }
        }
        if let Some(auth) = &node.auth {
            let merged = normalize_auth((*auth).clone(), Some(&collection.auth))?;
            collection.auth = merged;
            did_change = true;
        }
        if did_change {
            collection.updated_at = now_iso();
            if !dry_run {
                db.save_collection(&collection)?;
            }
            workspace.collections[index] = collection.clone();
            actions.push(json!({ "action": "update", "kind": kind, "path": path_text.clone() }));
            changed = true;
        }
        collection.id
    } else {
        let at = now_iso();
        let auth = if let Some(auth) = &node.auth {
            normalize_auth((*auth).clone(), Some(&default_auth()))?
        } else {
            default_auth()
        };
        let collection = Collection {
            id: node.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
            parent_id: parent_id.map(ToString::to_string),
            name: node.name.clone(),
            description: node.description.clone().unwrap_or_default(),
            auth,
            sort_order: next_collection_sort(workspace, parent_id),
            created_at: at.clone(),
            updated_at: at,
        };
        if !dry_run {
            db.save_collection(&collection)?;
        }
        let id = collection.id.clone();
        workspace.collections.push(collection);
        actions.push(json!({ "action": "create", "kind": kind, "path": path_text.clone() }));
        changed = true;
        id
    };

    for request in node.requests.as_deref().unwrap_or(&[]) {
        let (action, did_change) =
            upsert_request(db, workspace, request, &collection_id, &path_text, dry_run)?;
        actions.push(action);
        changed |= did_change;
    }

    for folder in node.folders.as_deref().unwrap_or(&[]) {
        let (folder_actions, did_change) = upsert_node(
            db,
            workspace,
            folder,
            Some(collection_id.as_str()),
            "folder",
            &path_parts,
            dry_run,
        )?;
        actions.extend(folder_actions);
        changed |= did_change;
    }

    Ok((actions, changed))
}

fn upsert_request(
    db: &Database,
    workspace: &mut Workspace,
    input: &RequestInput,
    collection_id: &str,
    folder_path: &str,
    dry_run: bool,
) -> Result<(Value, bool), String> {
    let path_text = format!("{folder_path} / {}", input.name);
    let existing_index = if let Some(id) = &input.id {
        workspace.requests.iter().position(|request| request.id == *id)
    } else {
        workspace
            .requests
            .iter()
            .position(|request| request.collection_id == collection_id && request.name == input.name)
    };

    if let Some(index) = existing_index {
        let mut request = workspace.requests[index].clone();
        request.collection_id = collection_id.to_string();
        request.name = input.name.clone();
        if let Some(method) = &input.method {
            request.method = normalize_method(method)?;
        }
        if let Some(url) = input.url.as_ref().or(input.path.as_ref()) {
            if !url.trim().is_empty() {
                request.url = url.clone();
            }
        }
        if let Some(description) = &input.description {
            request.description = description.clone();
        }
        if let Some(params) = &input.params {
            request.params = normalize_params(params);
        }
        if let Some(headers) = &input.headers {
            request.headers = normalize_params(headers);
        }
        if let Some(body) = &input.body {
            let merged = normalize_body((*body).clone(), Some(&request.body))?;
            request.body = merged;
        }
        if let Some(auth) = &input.auth {
            let merged = normalize_auth((*auth).clone(), Some(&request.auth))?;
            request.auth = merged;
        }
        if let Some(extractors) = &input.extractors {
            request.extractors = normalize_extractors(extractors);
        }
        if let Some(labels) = &input.response_labels {
            request.response_labels = normalize_labels(labels);
        }
        request.updated_at = now_iso();
        if !dry_run {
            db.save_request(&request)?;
        }
        workspace.requests[index] = request;
        return Ok((
            json!({ "action": "update", "kind": "request", "path": path_text }),
            true,
        ));
    }

    let url = input
        .url
        .clone()
        .or_else(|| input.path.clone())
        .unwrap_or_default();
    if url.trim().is_empty() {
        return Err(format!("接口「{}」缺少 url/path", input.name));
    }
    let request = ApiRequest {
        id: input.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
        collection_id: collection_id.to_string(),
        name: input.name.clone(),
        method: normalize_method(input.method.as_deref().unwrap_or("GET"))?,
        url,
        description: input.description.clone().unwrap_or_default(),
        params: normalize_params(input.params.as_deref().unwrap_or(&[])),
        headers: normalize_params(input.headers.as_deref().unwrap_or(&[])),
        body: normalize_body(input.body.clone().unwrap_or_default(), None)?,
        auth: normalize_auth(
            input.auth.clone().unwrap_or_default(),
            Some(&default_auth()),
        )?,
        extractors: normalize_extractors(input.extractors.as_deref().unwrap_or(&[])),
        response_labels: normalize_labels(input.response_labels.as_deref().unwrap_or(&[])),
        sort_order: next_request_sort(workspace, collection_id),
        updated_at: now_iso(),
    };
    if !dry_run {
        db.save_request(&request)?;
    }
    workspace.requests.push(request);
    Ok((
        json!({ "action": "create", "kind": "request", "path": path_text }),
        true,
    ))
}

fn normalize_params(fields: &[ParamField]) -> Vec<ParamField> {
    fields.iter().cloned().map(normalize_param).collect()
}

fn normalize_param(mut field: ParamField) -> ParamField {
    if field.id.trim().is_empty() {
        field.id = Uuid::new_v4().to_string();
    }
    field.name = field.name.trim().to_string();
    field.r#type = match field.r#type.trim().to_ascii_lowercase().as_str() {
        "integer" | "int" => "number".to_string(),
        "bool" => "boolean".to_string(),
        value if PARAM_TYPES.contains(&value) => value.to_string(),
        _ => "string".to_string(),
    };
    let mut seen = std::collections::HashSet::new();
    let options = field
        .options
        .iter()
        .map(|option| option.trim().to_string())
        .filter(|option| !option.is_empty() && seen.insert(option.clone()))
        .collect();
    field.options = options;
    field
}

fn normalize_body(mut input: BodyInput, base: Option<&RequestBody>) -> Result<RequestBody, String> {
    let mut body = base.cloned().unwrap_or_default();
    if let Some(mode) = input.mode.take() {
        let mode = match mode.trim().to_ascii_lowercase().as_str() {
            "form" => "urlencoded".to_string(),
            "raw" => "text".to_string(),
            value if BODY_MODES.contains(&value) => value.to_string(),
            _ => return Err(format!("不支持的 Body 类型：{mode}")),
        };
        body.mode = mode;
    }
    if let Some(raw) = input.raw.take() {
        body.raw = raw;
    }
    if let Some(content_type) = input.content_type.take() {
        body.content_type = content_type;
    }
    if let Some(fields) = input.fields.as_deref() {
        body.fields = normalize_params(fields);
    }
    if body.mode.trim().is_empty() {
        body.mode = "none".to_string();
    }
    Ok(body)
}

fn normalize_auth(input: AuthInput, base: Option<&RequestAuth>) -> Result<RequestAuth, String> {
    let mut auth = base.cloned().unwrap_or_else(default_auth);
    if let Some(auth_type) = input.r#type {
        let auth_type = auth_type.trim().to_ascii_lowercase();
        if !AUTH_TYPES.contains(&auth_type.as_str()) {
            return Err(format!("不支持的鉴权类型：{auth_type}"));
        }
        auth.r#type = auth_type;
    }
    if let Some(value) = input.bearer_token {
        auth.bearer_token = value;
    }
    if let Some(value) = input.username {
        auth.username = value;
    }
    if let Some(value) = input.password {
        auth.password = value;
    }
    if let Some(value) = input.key_name {
        auth.key_name = value;
    }
    if let Some(value) = input.key_value {
        auth.key_value = value;
    }
    if let Some(value) = input.key_in {
        if value == "header" || value == "query" {
            auth.key_in = value;
        }
    }
    Ok(auth)
}

fn normalize_extractors(rules: &[ExtractRule]) -> Vec<ExtractRule> {
    rules
        .iter()
        .cloned()
        .map(|mut rule| {
            if rule.id.trim().is_empty() {
                rule.id = Uuid::new_v4().to_string();
            }
            rule
        })
        .collect()
}

fn normalize_labels(labels: &[ResponseLabel]) -> Vec<ResponseLabel> {
    labels.to_vec()
}

fn normalize_variables(variables: &[EnvVariableInput]) -> Vec<EnvVariable> {
    variables
        .iter()
        .map(|variable| EnvVariable {
            id: variable
                .id
                .clone()
                .filter(|id| !id.trim().is_empty())
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            enabled: variable.enabled.unwrap_or(true),
            name: variable.name.trim().to_string(),
            value: variable.value.clone().unwrap_or_default(),
            description: variable.description.clone().unwrap_or_default(),
        })
        .collect()
}

fn default_auth() -> RequestAuth {
    RequestAuth {
        r#type: "inherit".to_string(),
        ..RequestAuth::default()
    }
}

fn normalize_method(method: &str) -> Result<String, String> {
    let method = method.trim().to_ascii_uppercase();
    if METHODS.contains(&method.as_str()) {
        Ok(method)
    } else {
        Err(format!("不支持的请求方法：{method}"))
    }
}

fn next_collection_sort(workspace: &Workspace, parent_id: Option<&str>) -> i64 {
    workspace
        .collections
        .iter()
        .filter(|collection| collection.parent_id.as_deref() == parent_id)
        .map(|collection| collection.sort_order)
        .max()
        .unwrap_or(-1)
        + 1
}

fn next_request_sort(workspace: &Workspace, collection_id: &str) -> i64 {
    workspace
        .requests
        .iter()
        .filter(|request| request.collection_id == collection_id)
        .map(|request| request.sort_order)
        .max()
        .unwrap_or(-1)
        + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_database() -> (PathBuf, Database) {
        let dir = env::temp_dir().join(format!("xingshi-mcp-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("messenger.db");
        let db = Database::open(path.clone()).unwrap();
        (dir, db)
    }

    fn sample_import() -> ImportInput {
        serde_json::from_value(json!({
            "dryRun": false,
            "environment": {
                "name": "MCP环境",
                "baseUrl": "http://localhost:3000",
                "activate": true,
                "variables": [
                    { "name": "token", "value": "demo-token", "description": "测试 Token" }
                ]
            },
            "projects": [{
                "name": "MCP测试项目",
                "folders": [{
                    "name": "人员管理",
                    "requests": [{
                        "name": "获取人员列表",
                        "method": "GET",
                        "url": "/api/staffs",
                        "description": "获取人员列表",
                        "params": [
                            { "name": "page", "type": "number", "value": "1", "description": "页码" },
                            { "name": "role", "type": "enum", "value": "admin", "options": ["admin", "editor"], "description": "角色" }
                        ],
                        "responseLabels": [
                            { "path": "data.list[].name", "label": "姓名", "type": "string", "example": "张三" }
                        ]
                    }]
                }]
            }]
        }))
        .unwrap()
    }

    #[test]
    fn imports_tree_environments_and_design_fields() {
        let (dir, db) = temp_database();
        let path = dir.join("messenger.db");
        let result = import_tree(&db, &path, sample_import()).unwrap();
        assert!(result["revision"].is_string());
        assert!(result["actions"].as_array().unwrap().iter().any(|action| action["action"] == "create"));

        let listing = list_workspace_value(&db, &path).unwrap();
        let project = listing["projects"]
            .as_array()
            .unwrap()
            .iter()
            .find(|project| project["name"] == "MCP测试项目")
            .unwrap();
        let folder = &project["folders"][0];
        assert_eq!(folder["name"], "人员管理");
        assert_eq!(folder["requests"][0]["paramCount"], 2);
        assert_eq!(folder["requests"][0]["responseLabelCount"], 1);
        let environment = listing["environments"]
            .as_array()
            .unwrap()
            .iter()
            .find(|environment| environment["name"] == "MCP环境")
            .unwrap();
        assert_eq!(listing["activeEnvironmentId"], environment["id"]);

        let workspace = db.load_workspace().unwrap();
        let request = workspace
            .requests
            .iter()
            .find(|request| request.name == "获取人员列表")
            .unwrap();
        assert_eq!(request.params[1].r#type, "enum");
        assert_eq!(request.params[1].options, vec!["admin", "editor"]);
        assert_eq!(request.response_labels[0].label, "姓名");
        assert!(workspace.environments.iter().any(|env| env.name == "MCP环境"));
        assert!(db.workspace_revision().unwrap().is_some());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn imports_are_idempotent_and_dry_run_does_not_write() {
        let (dir, db) = temp_database();
        let path = dir.join("messenger.db");
        import_tree(&db, &path, sample_import()).unwrap();
        let before = db.load_workspace().unwrap();
        import_tree(&db, &path, sample_import()).unwrap();
        let after = db.load_workspace().unwrap();
        assert_eq!(before.collections.len(), after.collections.len());
        assert_eq!(before.requests.len(), after.requests.len());

        let mut dry = sample_import();
        dry.dry_run = Some(true);
        dry.projects[0].name = "干跑项目".to_string();
        import_tree(&db, &path, dry).unwrap();
        let final_workspace = db.load_workspace().unwrap();
        assert!(!final_workspace
            .collections
            .iter()
            .any(|collection| collection.name == "干跑项目"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn dispatch_supports_handshake_and_tools() {
        let (dir, db) = temp_database();
        let path = dir.join("messenger.db");
        let initialize = dispatch(
            &json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": { "protocolVersion": "2025-11-25" }
            }),
            &path,
        );
        assert_eq!(initialize["result"]["serverInfo"]["name"], "xingshi-api");
        assert_eq!(initialize["result"]["capabilities"]["tools"], json!({}));

        let tools = dispatch(
            &json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }),
            &path,
        );
        let names: Vec<_> = tools["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|tool| tool["name"].as_str())
            .collect();
        assert!(names.contains(&"import_api_tree"));
        assert!(names.contains(&"list_workspace"));

        let listing = dispatch(
            &json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": { "name": "list_workspace", "arguments": {} }
            }),
            &path,
        );
        assert_eq!(listing["result"]["isError"], false);
        assert!(listing["result"]["content"][0]["text"].is_string());
        drop(db);
        fs::remove_dir_all(dir).unwrap();
    }
}
