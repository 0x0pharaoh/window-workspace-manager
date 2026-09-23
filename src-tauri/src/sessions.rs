//! Session helpers: listing, closing windows, restart and
//! off-screen recovery.

use tauri::AppHandle;

use crate::coords::{fallback_monitor, is_rect_visible_on_any_monitor, normalized_to_native};
use crate::database::Db;
use crate::error::WorksetError;
use crate::models::SessionInfo;

/// Recent sessions, newest first.
pub fn list(db: &Db, workspace_id: Option<&str>) -> Result<Vec<SessionInfo>, WorksetError> {
    db.list_sessions(workspace_id, 100)
}

/// Sessions that have not finished yet.
pub fn active(db: &Db) -> Result<Vec<SessionInfo>, WorksetError> {
    Ok(db
        .list_sessions(None, 100)?
        .into_iter()
        .filter(|s| s.status == "running")
        .collect())
}

/// Close a tracked window: validate the handle, post WM_CLOSE, and only
/// terminate the process when `force` is set and the window survives.
pub fn close_window_by_hwnd(
    hwnd: i64,
    force_kill: bool,
    pid: Option<u32>,
) -> Result<(), WorksetError> {
    if hwnd == 0 || !crate::windows::is_window_valid(hwnd) {
        return Err(WorksetError::NotFound("window no longer exists".to_owned()));
    }
    crate::windows::close_window(hwnd)?;
    if force_kill {
        std::thread::sleep(std::time::Duration::from_millis(600));
        if crate::windows::is_window_valid(hwnd) {
            match pid {
                Some(p) if p != 0 => crate::processes::kill_pid(p)?,
                _ => {
                    return Err(WorksetError::Validation(
                        "window did not close and no pid is known for a force kill".to_owned(),
                    ))
                }
            }
        }
    }
    Ok(())
}

/// Close one app window of a session.
pub fn close_session_app(
    db: &Db,
    session_id: &str,
    app_id: &str,
    force: bool,
) -> Result<(), WorksetError> {
    let rows = db.list_session_windows(session_id)?;
    let row = rows
        .iter()
        .find(|r| r.app_id == app_id)
        .ok_or_else(|| WorksetError::NotFound(format!("no tracked window for app {app_id}")))?;
    let hwnd = row.hwnd.unwrap_or(0);
    let pid = row.pid.unwrap_or(0) as u32;
    close_window_by_hwnd(hwnd, force, if pid == 0 { None } else { Some(pid) })?;
    db.update_session_window_status(&row.id, "closed", "closed by user")?;
    Ok(())
}

/// Close every tracked window of a session. One failure does not abort the
/// rest; an error is returned only when *all* closes failed.
pub fn close_workspace_session(db: &Db, session_id: &str, force: bool) -> Result<(), WorksetError> {
    let rows = db.list_session_windows(session_id)?;
    if rows.is_empty() {
        return Err(WorksetError::NotFound(format!(
            "session {session_id} has no tracked windows"
        )));
    }
    let mut failures = 0usize;
    for row in &rows {
        let hwnd = row.hwnd.unwrap_or(0);
        if hwnd == 0 {
            failures += 1;
            continue;
        }
        let pid = row.pid.unwrap_or(0) as u32;
        match close_window_by_hwnd(hwnd, force, if pid == 0 { None } else { Some(pid) }) {
            Ok(()) => {
                let _ = db.update_session_window_status(&row.id, "closed", "closed by user");
            }
            Err(_) => failures += 1,
        }
    }
    if failures == rows.len() {
        return Err(WorksetError::Windows(
            "could not close any window of the session".to_owned(),
        ));
    }
    Ok(())
}

/// Restart a session: cancel the old one, launch the same workspace again
/// in the background, and return the new session id immediately.
pub fn restart(app: &AppHandle, db: &Db, session_id: &str) -> Result<String, WorksetError> {
    let old = db.get_session(session_id)?;
    let _ = db.update_session_status(session_id, "cancelled", Some(crate::database::now_iso()));
    let workspace_id = old.workspace_id.clone();
    let new_session = db.create_session(&workspace_id)?;
    let sid = new_session.id.clone();
    let app2 = app.clone();
    let sid2 = sid.clone();
    tokio::spawn(async move {
        let _ = crate::launcher::run_session_launch(app2, sid2, workspace_id, None).await;
    });
    Ok(sid)
}

/// Move every tracked window that is fully off-screen back onto a visible
/// monitor (90% of the fallback work area). Returns the number of windows
/// moved. No-ops on platforms without window management.
pub fn recover_offscreen(db: &Db) -> Result<usize, WorksetError> {
    let monitors = crate::monitors::get_monitors();
    if monitors.is_empty() {
        return Ok(0);
    }
    let fb_id = fallback_monitor(&monitors).unwrap_or_default();
    let fb = monitors
        .iter()
        .find(|m| m.id == fb_id)
        .or(monitors.first())
        .cloned();
    let fb = match fb {
        Some(m) => m,
        None => return Ok(0),
    };
    let mut fixed = 0usize;
    for s in db.list_sessions(None, 50)? {
        if s.status != "running" && s.status != "done" {
            continue;
        }
        for row in db.list_session_windows(&s.id)? {
            let hwnd = row.hwnd.unwrap_or(0);
            if hwnd == 0 || !crate::windows::is_window_valid(hwnd) {
                continue;
            }
            let Some(win) = crate::windows::find_window(hwnd) else {
                continue;
            };
            let rect = crate::coords::NativeRect {
                x: win.x,
                y: win.y,
                w: win.w,
                h: win.h,
            };
            if is_rect_visible_on_any_monitor(&monitors, rect) {
                continue;
            }
            let target = normalized_to_native(
                0.05, 0.05, 0.9, 0.9, fb.work_x, fb.work_y, fb.work_w, fb.work_h,
            );
            if crate::windows::move_window(hwnd, target.x, target.y, target.w, target.h, "normal")
                .is_ok()
            {
                fixed += 1;
            }
        }
    }
    Ok(fixed)
}
