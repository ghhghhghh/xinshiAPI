//! 信使 API 工作台 —— Tauri 命令入口。

mod db;
mod http;
mod mcp;
mod models;
mod seed;

pub fn run_mcp_stdio() -> Result<(), String> {
    mcp::run_stdio()
}

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

use db::Database;
use http::HttpEngine;
use models::{
    ApiRequest, Collection, Environment, HistoryEntry, HttpRequestSpec, HttpResponseData,
    Workspace, now_iso,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AiSetupInfo {
    available: bool,
    sidecar_path: String,
    skill_path: String,
    db_path: String,
    qoder_settings_path: String,
    codex_config_path: String,
    message: String,
}

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
fn get_workspace_revision(state: State<'_, AppState>) -> Result<Option<String>, String> {
    state.db.workspace_revision()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .map(PathBuf::from)
}

fn find_sidecar(app: &AppHandle) -> Option<PathBuf> {
    let name = mcp::sidecar_file_name();
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(name));
            candidates.push(dir.join("resources").join(name));
        }
    }
    if let Ok(dir) = app.path().resource_dir() {
        candidates.push(dir.join(name));
        candidates.push(dir.join("binaries").join(name));
    }
    candidates.into_iter().find(|path| path.is_file())
}

fn find_skill(app: &AppHandle) -> Option<PathBuf> {
    let relative = PathBuf::from("skills").join("xingshi-api-import");
    let mut candidates = vec![
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(&relative),
    ];
    if let Ok(dir) = app.path().resource_dir() {
        candidates.push(dir.join("resources").join(&relative));
        candidates.push(dir.join(&relative));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("resources").join(&relative));
            candidates.push(dir.join(&relative));
        }
    }
    candidates
        .into_iter()
        .find(|path| path.join("SKILL.md").is_file())
}

#[tauri::command(rename_all = "camelCase")]
fn get_ai_setup_info(app: AppHandle) -> Result<AiSetupInfo, String> {
    let sidecar = find_sidecar(&app);
    let skill = find_skill(&app);
    let home = home_dir().ok_or_else(|| "无法定位用户主目录".to_string())?;
    let db_path = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("无法定位应用数据目录：{err}"))?
        .join("messenger.db");
    let available = sidecar.is_some() && skill.is_some();
    Ok(AiSetupInfo {
        available,
        sidecar_path: sidecar
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_default(),
        skill_path: skill
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_default(),
        db_path: db_path.display().to_string(),
        qoder_settings_path: home.join(".qoder").join("settings.json").display().to_string(),
        codex_config_path: home.join(".codex").join("config.toml").display().to_string(),
        message: if available {
            String::new()
        } else {
            "安装包缺少 MCP sidecar 或 Skill 资源，请重新安装应用。".to_string()
        },
    })
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum CloseAction {
    Tray,
    Quit,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct ClosePreferences {
    ask: bool,
    action: CloseAction,
}

impl Default for ClosePreferences {
    fn default() -> Self {
        Self { ask: true, action: CloseAction::Tray }
    }
}

fn read_close_preferences(db: &Database) -> Result<ClosePreferences, String> {
    match db.get_setting("window_close")? {
        Some(value) => serde_json::from_str(&value).map_err(|err| format!("读取关闭设置失败：{err}")),
        None => Ok(ClosePreferences::default()),
    }
}

#[tauri::command]
fn get_close_preferences(state: State<'_, AppState>) -> Result<ClosePreferences, String> {
    read_close_preferences(&state.db)
}

#[tauri::command]
fn set_close_preferences(
    app: AppHandle,
    state: State<'_, AppState>,
    preferences: ClosePreferences,
) -> Result<(), String> {
    let value = serde_json::to_string(&preferences).map_err(|err| err.to_string())?;
    state.db.set_setting("window_close", &value)?;
    sync_tray_ask(&app, preferences.ask).map_err(|err| err.to_string())?;
    app.emit_to("main", "close-preferences-changed", ())
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn complete_window_close(app: AppHandle, action: CloseAction) -> Result<(), String> {
    app.save_window_state(StateFlags::all() & !StateFlags::VISIBLE)
        .map_err(|err| format!("保存窗口状态失败：{err}"))?;
    match action {
        CloseAction::Tray => {
            let window = app.get_webview_window("main").ok_or("找不到主窗口")?;
            window.hide().map_err(|err| format!("隐藏窗口失败：{err}"))?;
        }
        CloseAction::Quit => app.exit(0),
    }
    Ok(())
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn build_tray_menu(app: &AppHandle, ask_checked: bool) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder};
    MenuBuilder::new(app)
        .item(&MenuItemBuilder::with_id("open", "打开信使").build(app)?)
        .item(
            &CheckMenuItemBuilder::with_id("close-ask", "关闭时询问")
                .checked(ask_checked)
                .build(app)?,
        )
        .item(&MenuItemBuilder::with_id("quit", "退出").build(app)?)
        .build()
}

fn persist_tray_ask(app: &AppHandle) {
    let state = app.state::<AppState>();
    let result = (|| {
        let mut preferences = read_close_preferences(&state.db)?;
        preferences.ask = !preferences.ask;
        set_close_preferences(app.clone(), state, preferences)
    })();
    if let Err(error) = result {
        show_main_window(app);
        let _ = app.emit_to("main", "close-preferences-error", error);
        let _ = sync_tray_ask(app, read_close_preferences(&app.state::<AppState>().db)
            .map(|preferences| preferences.ask).unwrap_or(true));
    }
}

fn sync_tray_ask(app: &AppHandle, ask: bool) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(build_tray_menu(app, ask)?))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default()
            .with_state_flags(StateFlags::all() & !StateFlags::VISIBLE)
            .build())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|err| std::io::Error::other(format!("无法获取应用数据目录：{err}")))?;
            let db =
                Database::open(data_dir.join("messenger.db")).map_err(std::io::Error::other)?;
            let http = HttpEngine::new().map_err(std::io::Error::other)?;
            let _ = app.manage(AppState { db, http });

            {
                use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
                let ask = read_close_preferences(&app.state::<AppState>().db)
                    .map(|preferences| preferences.ask).unwrap_or(true);
                let menu = build_tray_menu(app.handle(), ask)?;
                let icon = app.default_window_icon().ok_or_else(|| std::io::Error::other("缺少应用图标"))?.clone();
                TrayIconBuilder::with_id("main")
                    .icon(icon)
                    .tooltip("信使 API 工作台")
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "open" => show_main_window(app),
                        "close-ask" => persist_tray_ask(app),
                        "quit" => {
                            show_main_window(app);
                            let _ = app.emit_to("main", "quit-requested", ());
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if matches!(event,
                            TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. }
                            | TrayIconEvent::DoubleClick { button: MouseButton::Left, .. }
                        ) {
                            show_main_window(tray.app_handle());
                        }
                    })
                    .build(app)?;
            }

            if let Some(window) = app.get_webview_window("main") {
                let handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = handle.emit_to("main", "window-close-requested", ());
                    }
                });
            }

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
            get_workspace_revision,
            get_ai_setup_info,
            send_request,
            cancel_request,
            list_history,
            delete_history,
            clear_history,
            get_close_preferences,
            set_close_preferences,
            complete_window_close,
        ])
        .run(tauri::generate_context!())
        .expect("启动信使失败");
}
