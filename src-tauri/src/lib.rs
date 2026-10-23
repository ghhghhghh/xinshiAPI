//! 信使 API 工作台 —— Tauri 命令入口。

mod db;
mod http;
mod models;
mod seed;

use tauri::{Manager, State};

use db::Database;
use http::HttpEngine;
use models::{
    ApiRequest, Collection, Environment, HistoryEntry, HttpRequestSpec, HttpResponseData,
    Workspace, now_iso,
};

struct AppState {
    db: Database,
    http: HttpEngine,
}

#[tauri::command(rename_all = "camelCase")]
fn load_workspace(state: State<'_, AppState>) -> Result<Workspace, String> {
    state.db.load_workspace()
}

#[tauri::command(rename_all = "camelCase")]
fn save_collection(state: State<'_, AppState>, collection: Collection) -> Result<(), String> {
    state.db.save_collection(&collection)
}

#[tauri::command(rename_all = "camelCase")]
fn delete_collection(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.db.delete_collection(&id)
}

#[tauri::command(rename_all = "camelCase")]
fn save_request(state: State<'_, AppState>, request: ApiRequest) -> Result<(), String> {
    state.db.save_request(&request)
}

#[tauri::command(rename_all = "camelCase")]
fn delete_request(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.db.delete_request(&id)
}

#[tauri::command(rename_all = "camelCase")]
fn save_environment(state: State<'_, AppState>, environment: Environment) -> Result<(), String> {
    state.db.save_environment(&environment)
}

#[tauri::command(rename_all = "camelCase")]
fn delete_environment(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.db.delete_environment(&id)
}

#[tauri::command(rename_all = "camelCase")]
fn set_active_environment(state: State<'_, AppState>, id: Option<String>) -> Result<(), String> {
    state.db.set_active_environment(id.as_deref())
}

#[tauri::command(rename_all = "camelCase")]
async fn send_request(
    state: State<'_, AppState>,
    request_id: String,
    spec: HttpRequestSpec,
) -> Result<HttpResponseData, String> {
    let saved = state.db.get_request(&request_id)?;
    let (request_name, snapshot) = match saved {
        Some(request) => {
            let name = if request.name.trim().is_empty() {
                "历史请求".to_string()
            } else {
                request.name.clone()
            };
            (name, request)
        }
        None => {
            let name = "历史请求".to_string();
            let snapshot = ApiRequest {
                id: request_id.clone(),
                name: name.clone(),
                method: spec.method.clone(),
                url: spec.url.clone(),
                params: spec.params.clone(),
                headers: spec.headers.clone(),
                body: spec.body.clone(),
                auth: spec.auth.clone(),
                updated_at: now_iso(),
                ..Default::default()
            };
            (name, snapshot)
        }
    };

    let result = state.http.send(&request_id, spec.clone()).await;

    let created_at = now_iso();
    let (status, duration_ms, ok) = match &result {
        Ok(response) => (
            Some(response.status),
            Some(response.duration_ms),
            (200..400).contains(&response.status),
        ),
        Err(_) => (None, None, false),
    };
    let entry = HistoryEntry {
        id: uuid::Uuid::new_v4().to_string(),
        request_id: request_id.clone(),
        request_name,
        method: spec.method,
        url: spec.url,
        status,
        duration_ms,
        ok,
        created_at,
        request: snapshot,
    };
    if let Err(err) = state.db.insert_history(&entry) {
        eprintln!("写入历史记录失败：{err}");
    }

    result
}

#[tauri::command(rename_all = "camelCase")]
fn cancel_request(state: State<'_, AppState>, request_id: String) -> Result<(), String> {
    state.http.cancel(&request_id);
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
fn list_history(
    state: State<'_, AppState>,
    limit: Option<u32>,
    query: Option<String>,
) -> Result<Vec<HistoryEntry>, String> {
    state.db.list_history(limit, query.as_deref())
}

#[tauri::command(rename_all = "camelCase")]
fn delete_history(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.db.delete_history(&id)
}

#[tauri::command(rename_all = "camelCase")]
fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    state.db.clear_history()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|err| std::io::Error::other(format!("无法获取应用数据目录：{err}")))?;
            let db =
                Database::open(data_dir.join("messenger.db")).map_err(std::io::Error::other)?;
            let http = HttpEngine::new().map_err(std::io::Error::other)?;
            let _ = app.manage(AppState { db, http });

            // 记住窗口大小：插件本身在关闭时保存，这里额外在拖动结束时存一次，
            // 免得用户直接结束进程导致下次还是默认尺寸。
            {
                use std::sync::{Arc, Mutex};
                use std::time::{Duration, Instant};
                use tauri_plugin_window_state::{AppHandleExt, StateFlags};

                if let Some(window) = app.get_webview_window("main") {
                    let handle = app.handle().clone();
                    let last = Arc::new(Mutex::new(Instant::now() - Duration::from_secs(5)));
                    window.on_window_event(move |event| {
                        if !matches!(
                            event,
                            tauri::WindowEvent::Resized(_) | tauri::WindowEvent::Moved(_)
                        ) {
                            return;
                        }
                        let should_save = {
                            let mut guard = match last.lock() {
                                Ok(guard) => guard,
                                Err(_) => return,
                            };
                            if guard.elapsed() >= Duration::from_millis(700) {
                                *guard = Instant::now();
                                true
                            } else {
                                false
                            }
                        };
                        if should_save {
                            let _ = handle.save_window_state(StateFlags::all());
                        }
                    });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_workspace,
            save_collection,
            delete_collection,
            save_request,
            delete_request,
            save_environment,
            delete_environment,
            set_active_environment,
            send_request,
            cancel_request,
            list_history,
            delete_history,
            clear_history,
        ])
        .run(tauri::generate_context!())
        .expect("启动信使失败");
}
