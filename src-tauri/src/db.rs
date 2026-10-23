//! SQLite 持久化。每次操作使用独立连接，避免长连接跨 await 持有。

use std::fs;
use std::path::PathBuf;

use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::de::DeserializeOwned;

use crate::models::{
    ApiRequest, Collection, Environment, ExtractRule, HistoryEntry, ParamField, RequestAuth,
    RequestBody, ResponseLabel,
    Workspace,
};
use crate::seed;

const HISTORY_LIMIT: u32 = 200;

pub struct Database {
    path: PathBuf,
}

impl Database {
    pub fn open(path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|err| format!("创建数据目录失败：{err}"))?;
        }

        let database = Self { path };
        let mut conn = database.connect()?;
        init_schema(&conn)?;
        seed::seed_if_empty(&mut conn).map_err(|err| format!("初始化示例数据失败：{err}"))?;
        Ok(database)
    }

    fn connect(&self) -> Result<Connection, String> {
        let conn = Connection::open(&self.path).map_err(|err| format!("打开数据库失败：{err}"))?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;",
        )
        .map_err(|err| format!("初始化数据库连接失败：{err}"))?;
        Ok(conn)
    }

    pub fn load_workspace(&self) -> Result<Workspace, String> {
        let conn = self.connect()?;
        let collections = load_collections(&conn)?;
        let requests = load_requests(&conn)?;
        let environments = load_environments(&conn)?;
        let active_environment_id = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'active_environment'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| format!("读取激活环境失败：{err}"))?
            .filter(|value| !value.trim().is_empty());

        Ok(Workspace {
            collections,
            requests,
            environments,
            active_environment_id,
        })
    }

    pub fn save_collection(&self, collection: &Collection) -> Result<(), String> {
        if collection.id.trim().is_empty() {
            return Err("集合 ID 不能为空".to_string());
        }
        let conn = self.connect()?;
        let auth_json = serde_json::to_string(&collection.auth)
            .map_err(|err| format!("保存集合失败：鉴权序列化失败（{err}）"))?;
        conn.execute(
            "INSERT INTO collections
                (id, parent_id, name, description, auth_json, sort_order,
                 created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                parent_id = excluded.parent_id,
                name = excluded.name,
                description = excluded.description,
                auth_json = excluded.auth_json,
                sort_order = excluded.sort_order,
                created_at = excluded.created_at,
                updated_at = excluded.updated_at",
            params![
                collection.id,
                collection.parent_id,
                collection.name,
                collection.description,
                auth_json,
                collection.sort_order,
                collection.created_at,
                collection.updated_at
            ],
        )
        .map_err(|err| format!("保存集合失败：{err}"))?;
        Ok(())
    }

    pub fn delete_collection(&self, id: &str) -> Result<(), String> {
        let mut conn = self.connect()?;
        let tx = conn
            .transaction()
            .map_err(|err| format!("删除集合失败：{err}"))?;
        // 递归删掉整棵子树里的请求与文件夹
        tx.execute(
            "WITH RECURSIVE doomed(id) AS (
                SELECT ?1
                UNION ALL
                SELECT c.id FROM collections c JOIN doomed d ON c.parent_id = d.id
             )
             DELETE FROM requests WHERE collection_id IN (SELECT id FROM doomed)",
            params![id],
        )
        .map_err(|err| format!("删除集合中的请求失败：{err}"))?;
        tx.execute(
            "WITH RECURSIVE doomed(id) AS (
                SELECT ?1
                UNION ALL
                SELECT c.id FROM collections c JOIN doomed d ON c.parent_id = d.id
             )
             DELETE FROM collections WHERE id IN (SELECT id FROM doomed)",
            params![id],
        )
        .map_err(|err| format!("删除集合失败：{err}"))?;
        tx.commit().map_err(|err| format!("删除集合失败：{err}"))?;
        Ok(())
    }

    pub fn save_request(&self, request: &ApiRequest) -> Result<(), String> {
        if request.id.trim().is_empty() {
            return Err("请求 ID 不能为空".to_string());
        }
        let conn = self.connect()?;
        let collection_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM collections WHERE id = ?1)",
                params![request.collection_id],
                |row| row.get(0),
            )
            .map_err(|err| format!("保存请求失败：{err}"))?;
        if !collection_exists {
            return Err("保存请求失败：所属集合不存在".to_string());
        }

        let params_json = serde_json::to_string(&request.params)
            .map_err(|err| format!("保存请求失败：参数序列化失败（{err}）"))?;
        let headers_json = serde_json::to_string(&request.headers)
            .map_err(|err| format!("保存请求失败：Header 序列化失败（{err}）"))?;
        let body_json = serde_json::to_string(&request.body)
            .map_err(|err| format!("保存请求失败：请求体序列化失败（{err}）"))?;
        let auth_json = serde_json::to_string(&request.auth)
            .map_err(|err| format!("保存请求失败：鉴权信息序列化失败（{err}）"))?;
        let extractors_json = serde_json::to_string(&request.extractors)
            .map_err(|err| format!("保存请求失败：提取规则序列化失败（{err}）"))?;
        let response_labels_json = serde_json::to_string(&request.response_labels)
            .map_err(|err| format!("保存请求失败：响应字段序列化失败（{err}）"))?;

        conn.execute(
            "INSERT INTO requests
                (id, collection_id, name, method, url, params_json, headers_json, body_json,
                 auth_json, extractors_json, response_labels_json, description, sort_order,
                 created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
             ON CONFLICT(id) DO UPDATE SET
                collection_id = excluded.collection_id,
                name = excluded.name,
                method = excluded.method,
                url = excluded.url,
                params_json = excluded.params_json,
                headers_json = excluded.headers_json,
                body_json = excluded.body_json,
                auth_json = excluded.auth_json,
                extractors_json = excluded.extractors_json,
                response_labels_json = excluded.response_labels_json,
                description = excluded.description,
                sort_order = excluded.sort_order,
                updated_at = excluded.updated_at",
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
                extractors_json,
                response_labels_json,
                request.description,
                request.sort_order,
                request.updated_at,
                request.updated_at
            ],
        )
        .map_err(|err| format!("保存请求失败：{err}"))?;
        Ok(())
    }

    pub fn delete_request(&self, id: &str) -> Result<(), String> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM requests WHERE id = ?1", params![id])
            .map_err(|err| format!("删除请求失败：{err}"))?;
        Ok(())
    }

    pub fn get_request(&self, id: &str) -> Result<Option<ApiRequest>, String> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT id, collection_id, name, method, url, params_json, headers_json,
                    body_json, auth_json, description, sort_order, updated_at, extractors_json,
                    response_labels_json
             FROM requests WHERE id = ?1",
            params![id],
            row_to_request,
        )
        .optional()
        .map_err(|err| format!("读取请求失败：{err}"))
    }

    pub fn save_environment(&self, environment: &Environment) -> Result<(), String> {
        if environment.id.trim().is_empty() {
            return Err("环境 ID 不能为空".to_string());
        }
        let conn = self.connect()?;
        let variables_json = serde_json::to_string(&environment.variables)
            .map_err(|err| format!("保存环境失败：变量序列化失败（{err}）"))?;
        conn.execute(
            "INSERT INTO environments (id, name, base_url, variables_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                base_url = excluded.base_url,
                variables_json = excluded.variables_json,
                created_at = excluded.created_at,
                updated_at = excluded.updated_at",
            params![
                environment.id,
                environment.name,
                environment.base_url,
                variables_json,
                environment.created_at,
                environment.updated_at
            ],
        )
        .map_err(|err| format!("保存环境失败：{err}"))?;
        Ok(())
    }

    pub fn delete_environment(&self, id: &str) -> Result<(), String> {
        let mut conn = self.connect()?;
        let tx = conn
            .transaction()
            .map_err(|err| format!("删除环境失败：{err}"))?;
        tx.execute("DELETE FROM environments WHERE id = ?1", params![id])
            .map_err(|err| format!("删除环境失败：{err}"))?;
        tx.execute(
            "UPDATE settings SET value = '' WHERE key = 'active_environment' AND value = ?1",
            params![id],
        )
        .map_err(|err| format!("清理激活环境失败：{err}"))?;
        tx.commit().map_err(|err| format!("删除环境失败：{err}"))?;
        Ok(())
    }

    pub fn set_active_environment(&self, id: Option<&str>) -> Result<(), String> {
        let conn = self.connect()?;
        let value = id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("");
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('active_environment', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![value],
        )
        .map_err(|err| format!("设置激活环境失败：{err}"))?;
        Ok(())
    }

    /// 外部工具（MCP/CLI）导入后递增的版本号；应用自身保存不会改变它。
    pub fn workspace_revision(&self) -> Result<Option<String>, String> {
        let conn = self.connect()?;
        let result = conn.query_row(
            "SELECT value FROM settings WHERE key = 'external_revision'",
            [],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(value) if !value.trim().is_empty() => Ok(Some(value)),
            Ok(_) => Ok(None),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(format!("读取工作台版本失败：{err}")),
        }
    }

    /// MCP sidecar 导入成功后调用，通知正在运行的桌面应用刷新。
    pub fn bump_workspace_revision(&self) -> Result<String, String> {
        let conn = self.connect()?;
        let revision = format!(
            "{}-{}",
            chrono::Utc::now().timestamp_millis(),
            uuid::Uuid::new_v4().simple()
        );
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('external_revision', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![revision],
        )
        .map_err(|err| format!("写入工作台版本失败：{err}"))?;
        Ok(revision)
    }

    pub fn insert_history(&self, entry: &HistoryEntry) -> Result<(), String> {
        let mut conn = self.connect()?;
        let request_json = serde_json::to_string(&entry.request)
            .map_err(|err| format!("写入历史记录失败：请求快照序列化失败（{err}）"))?;
        let tx = conn
            .transaction()
            .map_err(|err| format!("写入历史记录失败：{err}"))?;
        tx.execute(
            "INSERT INTO history
                (id, request_json, request_id, request_name, method, url, status,
                 duration_ms, ok, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                entry.id,
                request_json,
                entry.request_id,
                entry.request_name,
                entry.method,
                entry.url,
                entry.status.map(i64::from),
                entry
                    .duration_ms
                    .map(|value| value.min(i64::MAX as u64) as i64),
                i64::from(entry.ok),
                entry.created_at
            ],
        )
        .map_err(|err| format!("写入历史记录失败：{err}"))?;
        tx.execute(
            "DELETE FROM history WHERE id NOT IN (
                SELECT id FROM history ORDER BY created_at DESC, rowid DESC LIMIT ?1
             )",
            params![HISTORY_LIMIT],
        )
        .map_err(|err| format!("清理历史记录失败：{err}"))?;
        tx.commit()
            .map_err(|err| format!("写入历史记录失败：{err}"))?;
        Ok(())
    }

    pub fn list_history(
        &self,
        limit: Option<u32>,
        query: Option<&str>,
    ) -> Result<Vec<HistoryEntry>, String> {
        let conn = self.connect()?;
        let limit = limit.unwrap_or(50).min(200) as i64;
        let query = query.unwrap_or("").trim();

        if query.is_empty() {
            let mut stmt = conn
                .prepare(
                    "SELECT id, request_id, request_name, method, url, status, duration_ms,
                            ok, created_at, request_json
                     FROM history ORDER BY created_at DESC, rowid DESC LIMIT ?1",
                )
                .map_err(|err| format!("读取历史记录失败：{err}"))?;
            let rows = stmt
                .query_map(params![limit], row_to_history)
                .map_err(|err| format!("读取历史记录失败：{err}"))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|err| format!("读取历史记录失败：{err}"))
        } else {
            let pattern = like_pattern(query);
            let mut stmt = conn
                .prepare(
                    "SELECT id, request_id, request_name, method, url, status, duration_ms,
                            ok, created_at, request_json
                     FROM history
                     WHERE UPPER(request_name) LIKE UPPER(?1) ESCAPE '\\'
                        OR UPPER(method) LIKE UPPER(?1) ESCAPE '\\'
                        OR UPPER(url) LIKE UPPER(?1) ESCAPE '\\'
                     ORDER BY created_at DESC, rowid DESC LIMIT ?2",
                )
                .map_err(|err| format!("读取历史记录失败：{err}"))?;
            let rows = stmt
                .query_map(params![pattern, limit], row_to_history)
                .map_err(|err| format!("读取历史记录失败：{err}"))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|err| format!("读取历史记录失败：{err}"))
        }
    }

    pub fn delete_history(&self, id: &str) -> Result<(), String> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM history WHERE id = ?1", params![id])
            .map_err(|err| format!("删除历史记录失败：{err}"))?;
        Ok(())
    }

    pub fn clear_history(&self) -> Result<(), String> {
        let conn = self.connect()?;
        conn.execute("DELETE FROM history", [])
            .map_err(|err| format!("清空历史记录失败：{err}"))?;
        Ok(())
    }
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS collections (
            id TEXT PRIMARY KEY NOT NULL,
            parent_id TEXT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            auth_json TEXT NOT NULL DEFAULT '{}',
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS requests (
            id TEXT PRIMARY KEY NOT NULL,
            collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            method TEXT NOT NULL,
            url TEXT NOT NULL,
            params_json TEXT NOT NULL,
            headers_json TEXT NOT NULL,
            body_json TEXT NOT NULL,
            auth_json TEXT NOT NULL,
            extractors_json TEXT NOT NULL DEFAULT '[]',
            response_labels_json TEXT NOT NULL DEFAULT '[]',
            description TEXT NOT NULL DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS environments (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            variables_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS history (
            id TEXT PRIMARY KEY NOT NULL,
            request_json TEXT NOT NULL,
            request_id TEXT NOT NULL,
            request_name TEXT NOT NULL,
            method TEXT NOT NULL,
            url TEXT NOT NULL,
            status INTEGER NULL,
            duration_ms INTEGER NULL,
            ok INTEGER NOT NULL,
            created_at TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS idx_requests_collection ON requests(collection_id);
         CREATE INDEX IF NOT EXISTS idx_history_created ON history(created_at DESC);",
    )
    .map_err(|err| format!("初始化数据库表失败：{err}"))?;

    // 老库升级：CREATE TABLE IF NOT EXISTS 不会补列，这里逐列检查
    ensure_column(conn, "collections", "parent_id", "parent_id TEXT NULL")?;
    ensure_column(
        conn,
        "collections",
        "auth_json",
        "auth_json TEXT NOT NULL DEFAULT '{}'",
    )?;
    ensure_column(
        conn,
        "environments",
        "base_url",
        "base_url TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "requests",
        "extractors_json",
        "extractors_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column(
        conn,
        "requests",
        "response_labels_json",
        "response_labels_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    Ok(())
}

/// 表里没有这一列时才 ALTER，重复启动不会报错。
fn ensure_column(conn: &Connection, table: &str, column: &str, ddl: &str) -> Result<(), String> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2)",
            params![table, column],
            |row| row.get(0),
        )
        .map_err(|err| format!("检查列 {table}.{column} 失败：{err}"))?;
    if exists {
        return Ok(());
    }
    conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {ddl}"), [])
        .map_err(|err| format!("升级列 {table}.{column} 失败：{err}"))?;
    Ok(())
}

fn load_collections(conn: &Connection) -> Result<Vec<Collection>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, sort_order, created_at, updated_at,
                    parent_id, auth_json
             FROM collections ORDER BY sort_order ASC, name ASC",
        )
        .map_err(|err| format!("读取集合失败：{err}"))?;
    let rows = stmt
        .query_map([], |row| {
            let auth_json: Option<String> = row.get(7)?;
            Ok(Collection {
                id: row.get(0)?,
                parent_id: row.get(6)?,
                name: row.get(1)?,
                description: row.get(2)?,
                auth: auth_json
                    .as_deref()
                    .map(parse_json)
                    .unwrap_or_else(|| RequestAuth {
                        r#type: "inherit".to_string(),
                        ..Default::default()
                    }),
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|err| format!("读取集合失败：{err}"))?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|err| format!("读取集合失败：{err}"))
}

fn load_requests(conn: &Connection) -> Result<Vec<ApiRequest>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, collection_id, name, method, url, params_json, headers_json,
                    body_json, auth_json, description, sort_order, updated_at, extractors_json,
                    response_labels_json
             FROM requests ORDER BY collection_id ASC, sort_order ASC, name ASC",
        )
        .map_err(|err| format!("读取请求失败：{err}"))?;
    let rows = stmt
        .query_map([], row_to_request)
        .map_err(|err| format!("读取请求失败：{err}"))?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|err| format!("读取请求失败：{err}"))
}

fn load_environments(conn: &Connection) -> Result<Vec<Environment>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, variables_json, created_at, updated_at, base_url
             FROM environments ORDER BY created_at ASC, rowid ASC",
        )
        .map_err(|err| format!("读取环境失败：{err}"))?;
    let rows = stmt
        .query_map([], |row| {
            let variables_json: String = row.get(2)?;
            Ok(Environment {
                id: row.get(0)?,
                name: row.get(1)?,
                base_url: row.get(5)?,
                variables: parse_json(&variables_json),
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|err| format!("读取环境失败：{err}"))?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|err| format!("读取环境失败：{err}"))
}

fn row_to_request(row: &Row<'_>) -> rusqlite::Result<ApiRequest> {
    let params_json: String = row.get(5)?;
    let headers_json: String = row.get(6)?;
    let body_json: String = row.get(7)?;
    let auth_json: String = row.get(8)?;
    let extractors_json: String = row.get(12)?;
    let response_labels_json: String = row.get(13)?;
    Ok(ApiRequest {
        id: row.get(0)?,
        collection_id: row.get(1)?,
        name: row.get(2)?,
        method: row.get(3)?,
        url: row.get(4)?,
        params: parse_json::<Vec<ParamField>>(&params_json),
        headers: parse_json::<Vec<ParamField>>(&headers_json),
        body: parse_json::<RequestBody>(&body_json),
        auth: parse_json::<RequestAuth>(&auth_json),
        extractors: parse_json::<Vec<ExtractRule>>(&extractors_json),
        response_labels: parse_json::<Vec<ResponseLabel>>(&response_labels_json),
        description: row.get(9)?,
        sort_order: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn row_to_history(row: &Row<'_>) -> rusqlite::Result<HistoryEntry> {
    let request_json: String = row.get(9)?;
    let status: Option<i64> = row.get(5)?;
    let duration_ms: Option<i64> = row.get(6)?;
    Ok(HistoryEntry {
        id: row.get(0)?,
        request_id: row.get(1)?,
        request_name: row.get(2)?,
        method: row.get(3)?,
        url: row.get(4)?,
        status: status.map(|value| value.clamp(0, u16::MAX as i64) as u16),
        duration_ms: duration_ms.and_then(|value| u64::try_from(value).ok()),
        ok: row.get::<_, i64>(7)? != 0,
        created_at: row.get(8)?,
        request: parse_json(&request_json),
    })
}

/// JSON 列损坏时退回默认值，保证整个工作区仍可打开。
fn parse_json<T: DeserializeOwned + Default>(value: &str) -> T {
    serde_json::from_str(value).unwrap_or_default()
}

fn like_pattern(query: &str) -> String {
    let mut pattern = String::with_capacity(query.len() + 2);
    pattern.push('%');
    for ch in query.chars() {
        if matches!(ch, '\\' | '%' | '_') {
            pattern.push('\\');
        }
        pattern.push(ch);
    }
    pattern.push('%');
    pattern
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ApiRequest, EnvVariable, Environment, HistoryEntry};
    use std::time::{SystemTime, UNIX_EPOCH};
    use uuid::Uuid;

    struct TempDb {
        db: Database,
        path: PathBuf,
    }

    impl TempDb {
        fn new() -> Self {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path =
                std::env::temp_dir().join(format!("messenger-test-{}-{stamp}.db", Uuid::new_v4()));
            let db = Database::open(path.clone()).expect("创建测试数据库");
            Self { db, path }
        }
    }

    impl Drop for TempDb {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
            let _ = fs::remove_file(self.path.with_extension("db-wal"));
            let _ = fs::remove_file(self.path.with_extension("db-shm"));
        }
    }

    fn sample_request(id: &str) -> ApiRequest {
        ApiRequest {
            id: id.into(),
            collection_id: "col-demo".into(),
            name: "测试请求".into(),
            method: "GET".into(),
            url: "https://example.com".into(),
            updated_at: "2026-01-01T00:00:00.000Z".into(),
            ..Default::default()
        }
    }

    fn history_entry(id: &str, name: &str, created_at: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.into(),
            request_id: "req-demo-get".into(),
            request_name: name.into(),
            method: "GET".into(),
            url: "https://example.com".into(),
            status: Some(200),
            duration_ms: Some(12),
            ok: true,
            created_at: created_at.into(),
            request: sample_request("req-demo-get"),
        }
    }

    #[test]
    fn seeds_empty_workspace_with_one_default_environment() {
        let temp = TempDb::new();
        let workspace = temp.db.load_workspace().unwrap();
        assert!(workspace.collections.is_empty());
        assert!(workspace.requests.is_empty());
        assert_eq!(workspace.environments.len(), 1);
        assert_eq!(workspace.environments[0].id, "env-default");
        assert_eq!(workspace.environments[0].name, "本地开发");
        assert!(workspace.environments[0].variables.is_empty());
        assert_eq!(
            workspace.active_environment_id.as_deref(),
            Some("env-default")
        );

        // 再次加载不会重复写种子
        let again = temp.db.load_workspace().unwrap();
        assert_eq!(again.environments.len(), 1);
        assert!(again.collections.is_empty());
    }

    #[test]
    fn workspace_revision_is_read_from_settings() {
        let temp = TempDb::new();
        assert_eq!(temp.db.workspace_revision().unwrap(), None);
        temp.db
            .connect()
            .unwrap()
            .execute(
                "INSERT INTO settings (key, value) VALUES ('external_revision', 'rev-1')
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [],
            )
            .unwrap();
        assert_eq!(
            temp.db.workspace_revision().unwrap().as_deref(),
            Some("rev-1")
        );
    }

    #[test]
    fn save_and_delete_collection_cascades_requests() {
        let temp = TempDb::new();
        let collection = Collection {
            id: "col-test".into(),
            parent_id: None,
            name: "测试集合".into(),
            description: String::new(),
            auth: RequestAuth::default(),
            sort_order: 9,
            created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-01T00:00:00.000Z".into(),
        };
        temp.db.save_collection(&collection).unwrap();
        let mut request = sample_request("req-test");
        request.collection_id = "col-test".into();
        temp.db.save_request(&request).unwrap();
        assert!(temp.db.get_request("req-test").unwrap().is_some());

        temp.db.delete_collection("col-test").unwrap();
        assert!(temp.db.get_request("req-test").unwrap().is_none());
        let workspace = temp.db.load_workspace().unwrap();
        assert!(
            !workspace
                .collections
                .iter()
                .any(|item| item.id == "col-test")
        );
    }

    #[test]
    fn history_is_trimmed_and_filtered() {
        let temp = TempDb::new();
        for index in 0..205 {
            let id = format!("h-{index:03}");
            let created_at = format!("2026-01-01T00:{:02}:{:02}.000Z", index / 60, index % 60);
            let name = if index == 204 {
                "特殊请求"
            } else {
                "普通请求"
            };
            temp.db
                .insert_history(&history_entry(&id, name, &created_at))
                .unwrap();
        }
        let all = temp.db.list_history(None, None).unwrap();
        assert_eq!(all.len(), 50);
        assert_eq!(all[0].request_name, "特殊请求");

        let capped = temp.db.list_history(Some(999), None).unwrap();
        assert_eq!(capped.len(), 200);

        let filtered = temp.db.list_history(Some(10), Some("特殊")).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].request_name, "特殊请求");
        assert_eq!(filtered[0].request.id, "req-demo-get");

        let insensitive = temp.db.list_history(Some(10), Some("get")).unwrap();
        assert_eq!(insensitive.len(), 10);
    }

    #[test]
    fn active_environment_can_be_cleared_and_deleted() {
        let temp = TempDb::new();
        temp.db.set_active_environment(Some("")).unwrap();
        assert!(
            temp.db
                .load_workspace()
                .unwrap()
                .active_environment_id
                .is_none()
        );

        temp.db.set_active_environment(Some("env-demo")).unwrap();
        temp.db.delete_environment("env-demo").unwrap();
        assert!(
            temp.db
                .load_workspace()
                .unwrap()
                .active_environment_id
                .is_none()
        );
    }

    #[test]
    fn environment_and_history_records_round_trip() {
        let temp = TempDb::new();
        let environment = Environment {
            id: "env-test".into(),
            name: "测试环境".into(),
            base_url: "http://localhost:9000".into(),
            variables: vec![EnvVariable {
                id: "v-test".into(),
                enabled: true,
                name: "baseUrl".into(),
                value: "http://localhost:9000".into(),
                description: "测试地址".into(),
            }],
            created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-01T00:00:00.000Z".into(),
        };
        temp.db.save_environment(&environment).unwrap();
        let workspace = temp.db.load_workspace().unwrap();
        let stored = workspace
            .environments
            .iter()
            .find(|item| item.id == "env-test")
            .unwrap();
        assert_eq!(stored.variables[0].value, "http://localhost:9000");

        temp.db
            .insert_history(&history_entry(
                "h-one",
                "普通请求",
                "2026-01-01T00:00:01.000Z",
            ))
            .unwrap();
        assert_eq!(temp.db.list_history(None, None).unwrap().len(), 1);
        temp.db.delete_history("h-one").unwrap();
        assert!(temp.db.list_history(None, None).unwrap().is_empty());

        temp.db
            .insert_history(&history_entry(
                "h-two",
                "普通请求",
                "2026-01-01T00:00:02.000Z",
            ))
            .unwrap();
        temp.db.clear_history().unwrap();
        assert!(temp.db.list_history(None, None).unwrap().is_empty());
    }
}
