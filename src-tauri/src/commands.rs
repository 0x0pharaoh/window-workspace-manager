//! Tauri command wrappers. All commands are JSON-friendly and return
//! `Result<T, String>`. Argument names match the frontend service layer
//! (`workspaceId`, `appId`, ...) exactly.

use std::collections::HashMap;

use serde_json::{json, Value};
use tauri::{AppHandle, State};

use crate::database::{now_iso, Db};
use crate::error::WorksetError;
use crate::models::{
    join_args, validate_workspace, AppSettings, SessionInfo, Workspace, WorkspaceApp,
};

// ---------- frontend shape helpers ----------

fn app_to_frontend(a: &WorkspaceApp) -> Value {
    json!({
        "id": a.id,
        "workspace_id": a.workspace_id,
        "name": a.name,
        "exe_path": a.exe_path,
        "args": join_args(&a.args),
        "cwd": a.cwd,
        "url": a.url,
        "delay_ms": a.launch_delay_ms,
        "monitor_id": a.target_monitor,
        "x": a.x, "y": a.y, "w": a.w, "h": a.h,
        "state": a.window_state,
        "policy": a.launch_policy,
        "match_rules": {
            "title_contains": a.match_rules.title_contains,
            "title_regex": a.match_rules.title_regex,
            "window_class": a.match_rules.window_class,
        },
        "sort_order": a.sort_order,
    })
}

fn ws_to_frontend(ws: &Workspace) -> Value {
    json!({
        "id": ws.id,
        "name": ws.name,
        "description": ws.description,
        "icon": ws.icon,
        "color": ws.color,
        "hotkey": ws.hotkey,
        "favorite": ws.is_favorite,
        "created_at": ws.created_at,
        "updated_at": ws.updated_at,
        "last_launched_at": null,
        "apps": ws.apps.iter().map(app_to_frontend).collect::<Vec<_>>(),
    })
}

fn session_to_frontend(db: &Db, s: &SessionInfo) -> Value {
    let app_names: HashMap<String, String> = db
        .get_workspace(&s.workspace_id)
        .map(|w| w.apps.into_iter().map(|a| (a.id, a.name)).collect())
        .unwrap_or_default();
    let workspace_name = db
        .get_workspace(&s.workspace_id)
        .map(|w| w.name)
        .unwrap_or_default();
    json!({
        "id": s.id,
        "workspace_id": s.workspace_id,
        "workspace_name": workspace_name,
        "started_at": s.started_at,
        "status": s.status,
        "windows": s.windows.iter().map(|r| json!({
            "id": r.id,
            "session_id": r.session_id,
            "app_id": r.app_id,
            "app_name": app_names.get(&r.app_id).cloned().unwrap_or_default(),
            "pid": r.pid,
            "hwnd": r.hwnd,
            "status": r.status,
            "message": r.message,
            "last_x": null,
            "last_y": null,
        })).collect::<Vec<_>>(),
    })
}

fn apply_ws_patch(ws: &mut Workspace, patch: &Value) {
    let Some(o) = patch.as_object() else { return };
    if let Some(s) = o.get("name").and_then(|v| v.as_str()) {
        ws.name = s.to_owned();
    }
    for k in ["description", "icon", "color", "hotkey"] {
        if let Some(s) = o.get(k).and_then(|v| v.as_str()) {
            match k {
                "description" => ws.description = s.to_owned(),
                "icon" => ws.icon = s.to_owned(),
                "color" => ws.color = s.to_owned(),
                _ => ws.hotkey = s.to_owned(),
            }
        }
    }
    if let Some(b) = o
        .get("favorite")
        .or_else(|| o.get("is_favorite"))
        .and_then(|v| v.as_bool())
    {
        ws.is_favorite = b;
    }
    if let Some(ls) = o.get("launch_settings") {
        ws.launch_settings = ls.clone();
    }
    if let Some(apps) = o.get("apps").and_then(|v| v.as_array()) {
        if let Ok(parsed) = serde_json::from_value::<Vec<WorkspaceApp>>(Value::Array(apps.clone()))
        {
            ws.apps = parsed;
        }
    }
}

fn apply_app_patch(app: &mut WorkspaceApp, patch: &Value) {
    let Some(o) = patch.as_object() else { return };
    for k in [
        "name",
        "exe_path",
        "cwd",
        "url",
        "app_icon",
        "target_monitor",
        "monitor_id",
    ] {
        if let Some(s) = o.get(k).and_then(|v| v.as_str()) {
            match k {
                "name" => app.name = s.to_owned(),
                "exe_path" => app.exe_path = s.to_owned(),
                "cwd" => app.cwd = s.to_owned(),
                "url" => app.url = s.to_owned(),
                "app_icon" => app.app_icon = s.to_owned(),
                _ => app.target_monitor = s.to_owned(),
            }
        }
    }
    if let Some(args) = o.get("args") {
        if let Ok(v) = serde_json::from_value::<Vec<String>>(args.clone()) {
            app.args = v;
        } else if let Some(s) = args.as_str() {
            app.args = crate::models::split_args(s);
        }
    }
    if let Some(n) = o
        .get("delay_ms")
        .or_else(|| o.get("launch_delay_ms"))
        .and_then(|v| v.as_i64())
    {
        app.launch_delay_ms = n;
    }
    for (k, slot) in [
        ("x", &mut app.x),
        ("y", &mut app.y),
        ("w", &mut app.w),
        ("h", &mut app.h),
    ] {
        if let Some(n) = o.get(k).and_then(|v| v.as_f64()) {
            *slot = n;
        }
    }
    if let Some(s) = o
        .get("state")
        .or_else(|| o.get("window_state"))
        .and_then(|v| v.as_str())
    {
        app.window_state = s.to_owned();
    }
    if let Some(s) = o
        .get("policy")
        .or_else(|| o.get("launch_policy"))
        .and_then(|v| v.as_str())
    {
        app.launch_policy = s.to_owned();
    }
    if let Some(mr) = o.get("match_rules") {
        if let Ok(parsed) = serde_json::from_value(mr.clone()) {
            app.match_rules = parsed;
        }
    }
    if let Some(n) = o.get("sort_order").and_then(|v| v.as_i64()) {
        app.sort_order = n;
    }
}

// ---------- monitors / windows / discovery ----------

#[tauri::command]
pub fn get_monitors() -> Vec<crate::models::MonitorInfo> {
    crate::monitors::get_monitors()
}

#[tauri::command]
pub fn enum_windows() -> Vec<crate::models::CapturedWindow> {
    crate::windows::enum_windows()
}

#[tauri::command]
pub fn capture_desktop() -> Vec<crate::models::CapturedWindow> {
    crate::capture::capture_desktop()
}

#[tauri::command]
pub fn discover_apps(db: State<'_, Db>) -> Vec<crate::models::DiscoveredApp> {
    let found = crate::discovery::discover_apps();
    for app in &found {
        let _ = db.upsert_catalog(app);
    }
    found
}

// ---------- workspaces ----------

#[tauri::command]
pub fn get_workspaces(db: State<'_, Db>) -> Result<Vec<Value>, String> {
    db.get_workspaces()
        .map(|all| all.iter().map(ws_to_frontend).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_workspace(db: State<'_, Db>, id: String) -> Result<Value, String> {
    db.get_workspace(&id)
        .map(|ws| ws_to_frontend(&ws))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_workspace(db: State<'_, Db>, name: String) -> Result<Value, String> {
    let now = now_iso();
    let ws = Workspace {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        description: String::new(),
        icon: String::new(),
        color: "#0078d4".to_owned(),
        is_favorite: false,
        hotkey: String::new(),
        launch_settings: json!({}),
        created_at: now.clone(),
        updated_at: now,
        apps: Vec::new(),
    };
    validate_workspace(&ws).map_err(|e| e.to_string())?;
    db.create_workspace(ws)
        .map(|ws| ws_to_frontend(&ws))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_workspace(db: State<'_, Db>, id: String, patch: Value) -> Result<Value, String> {
    let mut ws = db.get_workspace(&id).map_err(|e| e.to_string())?;
    apply_ws_patch(&mut ws, &patch);
    db.update_workspace(ws)
        .map(|ws| ws_to_frontend(&ws))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_workspace(db: State<'_, Db>, id: String) -> Result<(), String> {
    db.delete_workspace(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn duplicate_workspace(db: State<'_, Db>, id: String) -> Result<Value, String> {
    db.duplicate_workspace(&id)
        .map(|ws| ws_to_frontend(&ws))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_favorite(db: State<'_, Db>, id: String) -> Result<Value, String> {
    db.toggle_favorite(&id)
        .map(|ws| ws_to_frontend(&ws))
        .map_err(|e| e.to_string())
}

// ---------- apps ----------

#[tauri::command]
#[allow(non_snake_case)]
pub fn create_app(db: State<'_, Db>, workspaceId: String, input: Value) -> Result<Value, String> {
    db.get_workspace(&workspaceId).map_err(|e| e.to_string())?;
    let mut obj = input.as_object().cloned().unwrap_or_default();
    obj.entry("id")
        .or_insert_with(|| json!(uuid::Uuid::new_v4().to_string()));
    obj.entry("workspace_id")
        .or_insert_with(|| json!(workspaceId));
    let app: WorkspaceApp =
        serde_json::from_value(Value::Object(obj)).map_err(|e| format!("invalid app: {e}"))?;
    db.create_app(app)
        .map(|a| app_to_frontend(&a))
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn update_app(db: State<'_, Db>, appId: String, patch: Value) -> Result<Value, String> {
    let mut current = find_app(&db, &appId)?;
    apply_app_patch(&mut current, &patch);
    db.update_app(current)
        .map(|a| app_to_frontend(&a))
        .map_err(|e| e.to_string())
}

fn find_app(db: &Db, app_id: &str) -> Result<WorkspaceApp, String> {
    for ws in db.get_workspaces().map_err(|e| e.to_string())? {
        if let Some(a) = ws.apps.into_iter().find(|a| a.id == app_id) {
            return Ok(a);
        }
    }
    Err(WorksetError::NotFound(format!("app {app_id}")).to_string())
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn delete_app(db: State<'_, Db>, appId: String) -> Result<(), String> {
    db.delete_app(&appId).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn reorder_apps(
    db: State<'_, Db>,
    workspaceId: String,
    orderedIds: Vec<String>,
) -> Result<(), String> {
    db.reorder_apps(&workspaceId, &orderedIds)
        .map_err(|e| e.to_string())
}

// ---------- launch & sessions ----------

fn start_launch(app: &AppHandle, workspace_id: &str) -> Result<String, String> {
    use tauri::Manager;
    let session_id = {
        let db = app.state::<Db>();
        db.create_session(workspace_id)
            .map(|s| s.id)
            .map_err(|e| e.to_string())?
    };
    let app2 = app.clone();
    let ws = workspace_id.to_owned();
    let sid = session_id.clone();
    tokio::spawn(async move {
        if let Err(e) = crate::launcher::run_session_launch(app2, sid, ws, None).await {
            eprintln!("[workset] launch failed: {e}");
        }
    });
    Ok(session_id)
}

#[tauri::command]
pub async fn launch_workspace(app: AppHandle, id: String) -> Result<String, String> {
    start_launch(&app, &id)
}

/// Run a launch to completion and return the finished session id.
/// Unlike `launch_workspace` this awaits the whole run (automation/tests).
#[tauri::command]
pub async fn launch_workspace_wait(
    app: AppHandle,
    id: String,
    timeout_ms: Option<u64>,
) -> Result<String, String> {
    crate::launcher::run_workspace_launch(app, id, timeout_ms)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn cancel_launch(db: State<'_, Db>, workspaceId: String) -> Result<(), String> {
    let active = crate::sessions::active(&db).map_err(|e| e.to_string())?;
    for s in active.iter().filter(|s| s.workspace_id == workspaceId) {
        let _ = db.update_session_status(&s.id, "cancelled", Some(now_iso()));
        let _ = db.push_log(&s.id, "", "cancelled", "launch cancelled by user");
    }
    Ok(())
}

#[tauri::command]
pub fn get_sessions(db: State<'_, Db>) -> Result<Vec<Value>, String> {
    crate::sessions::list(&db, None)
        .map(|all| all.iter().map(|s| session_to_frontend(&db, s)).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_session(db: State<'_, Db>, id: String) -> Result<Value, String> {
    db.get_session(&id)
        .map(|s| session_to_frontend(&db, &s))
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn restart_session(app: AppHandle, sessionId: String) -> Result<String, String> {
    use tauri::Manager;
    let db = app.state::<Db>();
    crate::sessions::restart(&app, &db, &sessionId).map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn close_session_app(
    db: State<'_, Db>,
    sessionId: String,
    appId: String,
    force: Option<bool>,
) -> Result<(), String> {
    crate::sessions::close_session_app(&db, &sessionId, &appId, force.unwrap_or(false))
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn close_workspace_session(
    db: State<'_, Db>,
    sessionId: String,
    force: Option<bool>,
) -> Result<(), String> {
    crate::sessions::close_workspace_session(&db, &sessionId, force.unwrap_or(false))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn recover_offscreen(db: State<'_, Db>) -> Result<usize, String> {
    crate::sessions::recover_offscreen(&db).map_err(|e| e.to_string())
}

// ---------- import / export ----------

#[tauri::command]
pub fn import_workspace(
    db: State<'_, Db>,
    json: String,
    confirmed: Option<bool>,
) -> Result<Value, String> {
    crate::import_export::import_workspace(&db, &json, confirmed.unwrap_or(false))
        .map(|ws| ws_to_frontend(&ws))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_workspace(db: State<'_, Db>, id: String) -> Result<String, String> {
    crate::import_export::export_workspace(&db, &id).map_err(|e| e.to_string())
}

// ---------- settings & logs ----------

fn value_to_bool(v: &Value) -> Option<bool> {
    match v {
        Value::Bool(b) => Some(*b),
        Value::Number(n) => n.as_i64().map(|i| i != 0),
        Value::String(s) => match s.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn parse_stored_bool(db: &Db, key: &str) -> Option<bool> {
    db.get_setting(key)
        .unwrap_or(None)
        .and_then(|v| match v.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
}

#[tauri::command]
pub fn get_settings(db: State<'_, Db>) -> Result<Value, String> {
    let typed = crate::settings::load(&db);
    let raw = |k: &str| db.get_setting(k).unwrap_or(None).unwrap_or_default();
    let hotkeys: Value = db
        .get_setting("hotkeys")
        .unwrap_or(None)
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_else(|| json!({}));
    Ok(json!({
        "theme": typed.theme,
        "default_timeout_ms": typed.default_timeout_ms,
        "match_policy": typed.default_match_policy,
        "confirm_launch": typed.confirm_before_launch,
        "notifications": typed.notifications_enabled,
        "autostart": parse_stored_bool(&db, "autostart").unwrap_or(false),
        "launch_on_startup_workspace": raw("launch_on_startup_workspace"),
        "storage_path": raw("storage_path"),
        "hotkeys": hotkeys,
    }))
}

#[tauri::command]
pub fn set_setting(
    app: AppHandle,
    db: State<'_, Db>,
    key: String,
    value: Value,
) -> Result<(), String> {
    let stored = match &value {
        Value::String(s) => s.clone(),
        _ => value.to_string(),
    };
    db.set_setting(&key, &stored).map_err(|e| e.to_string())?;
    // Mirror frontend key names onto the typed settings keys.
    let mirror = match key.as_str() {
        "match_policy" => Some(("default_match_policy", stored.clone())),
        "confirm_launch" => Some((
            "confirm_before_launch",
            value_to_bool(&value).unwrap_or(false).to_string(),
        )),
        "notifications" => Some((
            "notifications_enabled",
            value_to_bool(&value).unwrap_or(true).to_string(),
        )),
        _ => None,
    };
    if let Some((k, v)) = mirror {
        db.set_setting(k, &v).map_err(|e| e.to_string())?;
    }
    if key == "autostart" {
        use tauri_plugin_autostart::ManagerExt;
        let enable = value_to_bool(&value).unwrap_or(false);
        let manager = app.autolaunch();
        let _ = if enable {
            manager.enable()
        } else {
            manager.disable()
        };
    }
    // Keep the typed cache consistent.
    if [
        "theme",
        "default_timeout_ms",
        "default_match_policy",
        "confirm_before_launch",
        "notifications_enabled",
        "match_policy",
        "confirm_launch",
        "notifications",
    ]
    .contains(&key.as_str())
    {
        let _ = crate::settings::save(
            &db,
            &AppSettings {
                theme: db
                    .get_setting("theme")
                    .unwrap_or(None)
                    .unwrap_or_else(|| "system".to_owned()),
                default_timeout_ms: db
                    .get_setting("default_timeout_ms")
                    .unwrap_or(None)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(15000),
                default_match_policy: db
                    .get_setting("default_match_policy")
                    .unwrap_or(None)
                    .unwrap_or_else(|| "exe_and_title".to_owned()),
                confirm_before_launch: parse_stored_bool(&db, "confirm_before_launch")
                    .unwrap_or(false),
                notifications_enabled: parse_stored_bool(&db, "notifications_enabled")
                    .unwrap_or(true),
            },
        );
    }
    Ok(())
}

#[tauri::command]
pub fn get_logs(db: State<'_, Db>, limit: Option<u64>) -> Result<Vec<String>, String> {
    db.list_logs(None, limit.unwrap_or(200) as i64)
        .map(|logs| {
            logs.into_iter()
                .map(|l| {
                    let scope = if l.app_id.is_empty() {
                        String::new()
                    } else {
                        format!("({}) ", l.app_id)
                    };
                    format!("[{}] [{}] {}{}", l.created_at, l.level, scope, l.message)
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

// ---------- OS integrations ----------

#[tauri::command]
pub fn register_hotkey(app: AppHandle, shortcut: String) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    app.global_shortcut()
        .register(shortcut.as_str())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unregister_hotkey(app: AppHandle, shortcut: String) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    app.global_shortcut()
        .unregister(shortcut.as_str())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enable: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    if enable {
        manager.enable()
    } else {
        manager.disable()
    }
    .map_err(|e| e.to_string())
}
