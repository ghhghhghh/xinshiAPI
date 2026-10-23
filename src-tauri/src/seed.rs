//! 首次启动的初始数据：只建一个空环境，并把它设为当前环境。

use rusqlite::{Connection, params};

use crate::models::{EnvVariable, Environment};

const SEED_TIME: &str = "2026-10-22T08:00:00.000Z";

pub fn seed_if_empty(conn: &mut Connection) -> rusqlite::Result<bool> {
    let environment_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM environments", [], |row| row.get(0))?;
    if environment_count > 0 {
        return Ok(false);
    }

    let environment = Environment {
        id: "env-default".to_string(),
        name: "本地开发".to_string(),
        base_url: String::new(),
        variables: Vec::<EnvVariable>::new(),
        created_at: SEED_TIME.to_string(),
        updated_at: SEED_TIME.to_string(),
    };

    let variables_json = serde_json::to_string(&environment.variables)
        .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;

    conn.execute(
        "INSERT OR IGNORE INTO environments (id, name, base_url, variables_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            environment.id,
            environment.name,
            environment.base_url,
            variables_json,
            environment.created_at,
            environment.updated_at
        ],
    )?;

    // 与前端 mock 的初始状态保持一致：默认环境处于激活状态。
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('active_environment', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![environment.id],
    )?;

    Ok(true)
}
