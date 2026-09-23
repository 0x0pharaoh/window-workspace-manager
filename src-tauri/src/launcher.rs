//! Async launch orchestration.
//!
//! `run_workspace_launch` creates a session and runs it to completion,
//! returning the session id. `run_session_launch` continues a pre-created
//! session (used by the `launch_workspace` command so it can return the id
//! immediately while the work continues on a background task).
//!
//! Every app is attempted independently: one failure is recorded and the
//! launch continues. Progress is emitted on the `launch-progress` channel.

use std::path::Path;

use tauri::{AppHandle, Emitter, Manager};

use crate::coords::{monitor_work_rect, normalized_to_native, NativeRect};
use crate::database::{now_iso, Db};
use crate::error::WorksetError;
use crate::models::{
    validate_workspace, LaunchProgressEvent, MatchRules, MonitorInfo, SessionWindowRow,
    WorkspaceApp,
};

fn emit(app: &AppHandle, ev: &LaunchProgressEvent) {
    let _ = app.emit("launch-progress", ev);
}

fn event(
    session_id: &str,
    workspace_id: &str,
    app: &WorkspaceApp,
    status: &str,
    message: &str,
    progress: f64,
) -> LaunchProgressEvent {
    LaunchProgressEvent {
        session_id: session_id.to_owned(),
        workspace_id: workspace_id.to_owned(),
        app_id: app.id.clone(),
        app_name: app.name.clone(),
        status: status.to_owned(),
        message: message.to_owned(),
        progress,
    }
}

/// Effective match rules: fall back to the exe file stem when the user did
/// not configure `exe_contains` explicitly.
fn effective_rules(app: &WorkspaceApp) -> MatchRules {
    let mut r = app.match_rules.clone();
    let empty = r
        .exe_contains
        .as_deref()
        .map(|s| s.trim().is_empty())
        .unwrap_or(true);
    if empty && !app.exe_path.trim().is_empty() {
        let stem = Path::new(&app.exe_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .trim();
        // A `.lnk` stem rarely matches the real process; strip the extension
        // hint anyway and let title rules dominate.
        let stem = stem.strip_suffix(".lnk").unwrap_or(stem);
        if !stem.is_empty() {
            r.exe_contains = Some(stem.to_owned());
        }
    }
    r
}

fn is_cancelled(db: &Db, session_id: &str) -> bool {
    db.get_session(session_id)
        .map(|s| s.status == "cancelled")
        .unwrap_or(false)
}

fn notify(app: &AppHandle, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification().builder().title(title).body(body).show();
}

fn record(
    db: &Db,
    session_id: &str,
    app: &WorkspaceApp,
    pid: Option<u32>,
    hwnd: Option<i64>,
    status: &str,
    message: &str,
    rect: Option<NativeRect>,
) {
    let _ = db.upsert_session_window(&SessionWindowRow {
        id: format!("{session_id}:{}", app.id),
        session_id: session_id.to_owned(),
        app_id: app.id.clone(),
        pid: pid.map(|p| p as i64),
        hwnd,
        status: status.to_owned(),
        message: message.to_owned(),
        last_rect: rect
            .map(|r| serde_json::to_value(r).unwrap_or_default())
            .unwrap_or_else(|| serde_json::json!({})),
        launched_at: now_iso(),
    });
    let _ = db.push_log(session_id, &app.id, status, message);
}

/// Launch (or reuse) a single app and move its window into place.
async fn launch_single_app(
    app: &AppHandle,
    db: &Db,
    session_id: &str,
    workspace_id: &str,
    wa: &WorkspaceApp,
    monitors: &[MonitorInfo],
    default_timeout_ms: u64,
    progress: f64,
) -> Result<(), WorksetError> {
    let policy = wa.launch_policy.trim();
    if policy == "skip" {
        record(
            db,
            session_id,
            wa,
            None,
            None,
            "done",
            "skipped by policy",
            None,
        );
        emit(
            app,
            &event(
                session_id,
                workspace_id,
                wa,
                "done",
                "Skipped (policy)",
                progress,
            ),
        );
        return Ok(());
    }

    let target = monitors
        .iter()
        .find(|m| m.id == wa.target_monitor)
        .or_else(|| monitors.iter().find(|m| m.is_primary))
        .or(monitors.first())
        .cloned()
        .ok_or_else(|| WorksetError::Windows("no monitors available".to_owned()))?;
    let work = monitor_work_rect(&target);
    let rules = effective_rules(wa);

    // Policies that reuse an existing window.
    if matches!(policy, "if_not_running" | "reuse" | "focus") {
        let current = crate::windows::enum_windows();
        if let Some(hit) = crate::matching::find_best(&current, &rules) {
            if policy == "focus" {
                let _ = crate::windows::focus_window(hit.hwnd);
            } else {
                // Enforce the saved layout on the reused window.
                let rect =
                    normalized_to_native(wa.x, wa.y, wa.w, wa.h, work.x, work.y, work.w, work.h);
                let _ = crate::windows::move_window(
                    hit.hwnd,
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    &wa.window_state,
                );
            }
            let msg = format!("reusing window '{}' (hwnd {})", hit.title, hit.hwnd);
            record(
                db,
                session_id,
                wa,
                Some(hit.pid),
                Some(hit.hwnd),
                "done",
                &msg,
                None,
            );
            emit(
                app,
                &event(session_id, workspace_id, wa, "done", &msg, progress),
            );
            return Ok(());
        }
        if policy == "reuse" || policy == "focus" {
            let msg = "no matching window found (policy only matches)".to_owned();
            record(db, session_id, wa, None, None, "failed", &msg, None);
            emit(
                app,
                &event(session_id, workspace_id, wa, "failed", &msg, progress),
            );
            return Err(WorksetError::NotFound(msg));
        }
        // if_not_running without a hit falls through to spawning.
    }

    emit(
        app,
        &event(
            session_id,
            workspace_id,
            wa,
            "launching",
            "Starting…",
            progress,
        ),
    );

    if wa.launch_delay_ms > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(
            wa.launch_delay_ms.max(0) as u64
        ))
        .await;
    }
    if is_cancelled(db, session_id) {
        return Err(WorksetError::Timeout("launch cancelled".to_owned()));
    }

    if wa.exe_path.trim().is_empty() && wa.url.trim().is_empty() {
        let msg = "nothing to launch: exe_path and url are both empty".to_owned();
        record(db, session_id, wa, None, None, "failed", &msg, None);
        emit(
            app,
            &event(session_id, workspace_id, wa, "failed", &msg, progress),
        );
        return Err(WorksetError::Validation(msg));
    }
    if !wa.exe_path.trim().is_empty() {
        if let Err(e) = crate::processes::validate_exe(&wa.exe_path) {
            let msg = format!("invalid executable: {e}");
            record(db, session_id, wa, None, None, "failed", &msg, None);
            emit(
                app,
                &event(session_id, workspace_id, wa, "failed", &msg, progress),
            );
            return Err(e);
        }
    }

    let snapshot = crate::windows::snapshot_before_launch();
    let pid = crate::processes::launch_app(&wa.exe_path, &wa.args, &wa.cwd, &wa.url)?;
    record(
        db,
        session_id,
        wa,
        Some(pid),
        None,
        "waiting",
        &format!("spawned pid {pid}, waiting for window…"),
        None,
    );
    emit(
        app,
        &event(
            session_id,
            workspace_id,
            wa,
            "waiting",
            "Waiting for window…",
            progress,
        ),
    );

    let found = crate::processes::wait_for_new_window(&snapshot, &rules, default_timeout_ms).await;
    let win = match found {
        Ok(w) => w,
        Err(e) => {
            let msg = format!("launched pid {pid} but {e}");
            record(db, session_id, wa, Some(pid), None, "failed", &msg, None);
            emit(
                app,
                &event(session_id, workspace_id, wa, "failed", &msg, progress),
            );
            return Err(e);
        }
    };

    emit(
        app,
        &event(
            session_id,
            workspace_id,
            wa,
            "positioning",
            "Positioning…",
            progress,
        ),
    );
    let rect = normalized_to_native(wa.x, wa.y, wa.w, wa.h, work.x, work.y, work.w, work.h);
    if let Err(e) =
        crate::windows::move_window(win.hwnd, rect.x, rect.y, rect.w, rect.h, &wa.window_state)
    {
        let msg = format!(
            "window found (hwnd {}) but positioning failed: {e}",
            win.hwnd
        );
        record(
            db,
            session_id,
            wa,
            Some(pid),
            Some(win.hwnd),
            "failed",
            &msg,
            Some(rect),
        );
        emit(
            app,
            &event(session_id, workspace_id, wa, "failed", &msg, progress),
        );
        return Err(e);
    }
    let msg = format!("launched on {} (hwnd {})", target.name, win.hwnd);
    record(
        db,
        session_id,
        wa,
        Some(pid),
        Some(win.hwnd),
        "done",
        &msg,
        Some(rect),
    );
    emit(
        app,
        &event(session_id, workspace_id, wa, "done", &msg, progress),
    );
    Ok(())
}

/// Continue a pre-created session to completion.
pub async fn run_session_launch(
    app: AppHandle,
    session_id: String,
    workspace_id: String,
    timeout_ms: Option<u64>,
) -> Result<(), WorksetError> {
    let (workspace, monitors, timeout, notify_on) = {
        let db = app.state::<Db>();
        let ws = db.get_workspace(&workspace_id)?;
        let settings = crate::settings::load(&db);
        let mons = crate::monitors::get_monitors();
        let t = timeout_ms.unwrap_or_else(|| settings.default_timeout_ms.max(1000) as u64);
        (ws, mons, t, settings.notifications_enabled)
    };

    if let Err(e) = validate_workspace(&workspace) {
        let db = app.state::<Db>();
        let _ = db.update_session_status(&session_id, "failed", Some(now_iso()));
        let _ = db.push_log(&session_id, "", "failed", &e.to_string());
        return Err(e);
    }

    {
        let db = app.state::<Db>();
        let _ = db.push_log(
            &session_id,
            "",
            "info",
            &format!(
                "starting launch of workspace '{}' ({} app(s))",
                workspace.name,
                workspace.apps.len()
            ),
        );
    }

    let total = workspace.apps.len().max(1) as f64;
    for (idx, wa) in workspace.apps.iter().enumerate() {
        let progress = (idx as f64 + 1.0) / total;
        {
            let db = app.state::<Db>();
            if is_cancelled(&db, &session_id) {
                let _ = db.push_log(&session_id, "", "cancelled", "launch cancelled by user");
                let _ = db.update_session_status(&session_id, "cancelled", Some(now_iso()));
                emit(
                    &app,
                    &LaunchProgressEvent {
                        session_id: session_id.clone(),
                        workspace_id: workspace_id.clone(),
                        app_id: String::new(),
                        app_name: String::new(),
                        status: "cancelled".to_owned(),
                        message: "Launch cancelled".to_owned(),
                        progress,
                    },
                );
                return Ok(());
            }
        }
        let db = app.state::<Db>();
        // One app must never abort the rest.
        let _ = launch_single_app(
            &app,
            &db,
            &session_id,
            &workspace_id,
            wa,
            &monitors,
            timeout,
            progress,
        )
        .await;
    }

    {
        let db = app.state::<Db>();
        let _ = db.update_session_status(&session_id, "done", Some(now_iso()));
        let _ = db.push_log(&session_id, "", "done", "launch finished");
    }
    if notify_on {
        notify(
            &app,
            "Workset",
            &format!("Workspace '{}' launched", workspace.name),
        );
    }
    Ok(())
}

/// Create a session and run it to completion, returning the session id.
pub async fn run_workspace_launch(
    app: AppHandle,
    workspace_id: String,
    timeout_ms: Option<u64>,
) -> Result<String, WorksetError> {
    let session_id = {
        let db = app.state::<Db>();
        db.create_session(&workspace_id).map(|s| s.id)?
    };
    run_session_launch(app, session_id.clone(), workspace_id, timeout_ms).await?;
    Ok(session_id)
}
