//! SQLite persistence layer (`rusqlite`, bundled).
//!
//! [`Db`] wraps a single `Mutex<Connection>`; every method locks briefly,
//! so it is safe to share via Tauri state. Workspace+app writes use
//! transactions. Schema is embedded from `migrations/001_init.sql`.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::WorksetError;
use crate::models::*;

const INIT_SQL: &str = include_str!("../migrations/001_init.sql");

pub struct Db {
    conn: Mutex<Connection>,
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn parse_object(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap_or_else(|_| serde_json::json!({}))
}

fn parse_args_cell(s: &str) -> Vec<String> {
    let t = s.trim();
    if t.is_empty() {
        return Vec::new();
    }
    if let Ok(v) = serde_json::from_str::<Vec<String>>(t) {
        return v;
    }
    if let Ok(one) = serde_json::from_str::<String>(t) {
        return split_args(&one);
    }
    split_args(t)
}

fn parse_match_rules(s: &str) -> MatchRules {
    serde_json::from_str(s).unwrap_or_default()
}

fn row_to_workspace(row: &Row) -> rusqlite::Result<Workspace> {
    let fav: i64 = row.get("is_favorite")?;
    let launch_settings: String = row.get("launch_settings")?;
    Ok(Workspace {
        id: row.get("id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        icon: row.get("icon")?,
        color: row.get("color")?,
        is_favorite: fav != 0,
        hotkey: row.get::<_, Option<String>>("hotkey")?.unwrap_or_default(),
        launch_settings: parse_object(&launch_settings),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        apps: Vec::new(),
    })
}

fn row_to_app(row: &Row) -> rusqlite::Result<WorkspaceApp> {
    let args: String = row.get("args")?;
    let match_rules: String = row.get("match_rules")?;
    Ok(WorkspaceApp {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        sort_order: row.get("sort_order")?,
        name: row.get("name")?,
        exe_path: row.get("exe_path")?,
        args: parse_args_cell(&args),
        cwd: row.get("cwd")?,
        url: row.get("url")?,
        app_icon: row.get("app_icon")?,
        match_rules: parse_match_rules(&match_rules),
        launch_delay_ms: row.get("launch_delay_ms")?,
        target_monitor: row.get("target_monitor")?,
        x: row.get("x")?,
        y: row.get("y")?,
        w: row.get("w")?,
        h: row.get("h")?,
        window_state: row.get("window_state")?,
        launch_policy: row.get("launch_policy")?,
    })
}

fn row_to_session(row: &Row) -> rusqlite::Result<SessionInfo> {
    Ok(SessionInfo {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        status: row.get("status")?,
        started_at: row.get("started_at")?,
        finished_at: row.get("finished_at")?,
        windows: Vec::new(),
    })
}

fn row_to_session_window(row: &Row) -> rusqlite::Result<SessionWindowRow> {
    let last_rect: String = row.get("last_rect")?;
    Ok(SessionWindowRow {
        id: row.get("id")?,
        session_id: row.get("session_id")?,
        app_id: row.get("app_id")?,
        pid: row.get("pid")?,
        hwnd: row.get("hwnd")?,
        status: row.get("status")?,
        message: row.get("message")?,
        last_rect: parse_object(&last_rect),
        launched_at: row.get("launched_at")?,
    })
}

fn row_to_log(row: &Row) -> rusqlite::Result<LaunchLog> {
    Ok(LaunchLog {
        id: row.get("id")?,
        session_id: row.get("session_id")?,
        app_id: row.get("app_id")?,
        level: row.get("level")?,
        message: row.get("message")?,
        created_at: row.get("created_at")?,
    })
}

impl Db {
    fn lock(&self) -> Result<MutexGuard<'_, Connection>, WorksetError> {
        self.conn
            .lock()
            .map_err(|e| WorksetError::Db(format!("database lock poisoned: {e}")))
    }

    fn init_schema(conn: &Connection) -> Result<(), WorksetError> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(INIT_SQL)?;
        conn.execute_batch("PRAGMA user_version = 1;")?;
        Ok(())
    }

    /// Open (creating parent dirs) and migrate a file database.
    pub fn open(path: &Path) -> Result<Self, WorksetError> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(path)?;
        Self::init_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// In-memory database, used by unit tests.
    pub fn open_in_memory() -> Result<Self, WorksetError> {
        let conn = Connection::open_in_memory()?;
        Self::init_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // ---------- workspaces ----------

    pub fn get_workspaces(&self) -> Result<Vec<Workspace>, WorksetError> {
        let ids: Vec<String> = {
            let conn = self.lock()?;
            let mut stmt = conn.prepare(
                "SELECT id FROM workspaces ORDER BY is_favorite DESC, updated_at DESC, name ASC",
            )?;
            let rows: Vec<String> = stmt
                .query_map([], |r| r.get("id"))?
                .collect::<Result<Vec<_>, _>>()?;
            rows
        };
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            out.push(self.get_workspace(&id)?);
        }
        Ok(out)
    }

    pub fn get_workspace(&self, id: &str) -> Result<Workspace, WorksetError> {
        let mut ws = {
            let conn = self.lock()?;
            conn.query_row(
                "SELECT id, name, description, icon, color, is_favorite, hotkey,
                        launch_settings, created_at, updated_at
                 FROM workspaces WHERE id = ?1",
                params![id],
                row_to_workspace,
            )
            .optional()?
            .ok_or_else(|| WorksetError::NotFound(format!("workspace {id}")))?
        };
        ws.apps = self.get_apps(&ws.id)?;
        Ok(ws)
    }

    fn insert_apps_tx(
        tx: &rusqlite::Transaction,
        workspace_id: &str,
        apps: &[WorkspaceApp],
        now: &str,
    ) -> Result<(), WorksetError> {
        for app in apps {
            tx.execute(
                "INSERT INTO workspace_apps
                 (id, workspace_id, sort_order, name, exe_path, args, cwd, url,
                  app_icon, match_rules, launch_delay_ms, target_monitor,
                  x, y, w, h, window_state, launch_policy, created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
                params![
                    app.id,
                    workspace_id,
                    app.sort_order,
                    app.name,
                    app.exe_path,
                    serde_json::to_string(&app.args).unwrap_or_else(|_| "[]".to_owned()),
                    app.cwd,
                    app.url,
                    app.app_icon,
                    serde_json::to_string(&app.match_rules).unwrap_or_else(|_| "{}".to_owned()),
                    app.launch_delay_ms,
                    app.target_monitor,
                    app.x,
                    app.y,
                    app.w,
                    app.h,
                    app.window_state,
                    app.launch_policy,
                    now,
                ],
            )?;
        }
        Ok(())
    }

    pub fn create_workspace(&self, mut ws: Workspace) -> Result<Workspace, WorksetError> {
        validate_workspace(&ws)?;
        if ws.id.trim().is_empty() {
            ws.id = new_id();
        }
        let now = now_iso();
        if ws.created_at.is_empty() {
            ws.created_at = now.clone();
        }
        ws.updated_at = now.clone();
        for (i, app) in ws.apps.iter_mut().enumerate() {
            if app.id.trim().is_empty() {
                app.id = new_id();
            }
            app.workspace_id = ws.id.clone();
            app.sort_order = i as i64;
            validate_app(app)?;
        }
        let guard = self.lock()?;
        let tx = guard.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO workspaces
             (id, name, description, icon, color, is_favorite, hotkey,
              launch_settings, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                ws.id,
                ws.name,
                ws.description,
                ws.icon,
                ws.color,
                i64::from(ws.is_favorite),
                ws.hotkey,
                serde_json::to_string(&ws.launch_settings).unwrap_or_else(|_| "{}".to_owned()),
                ws.created_at,
                ws.updated_at,
            ],
        )?;
        Self::insert_apps_tx(&tx, &ws.id, &ws.apps, &now)?;
        tx.commit()?;
        Ok(ws)
    }

    pub fn update_workspace(&self, mut ws: Workspace) -> Result<Workspace, WorksetError> {
        validate_workspace(&ws)?;
        if ws.id.trim().is_empty() {
            return Err(WorksetError::Validation("workspace id is empty".to_owned()));
        }
        ws.updated_at = now_iso();
        for (i, app) in ws.apps.iter_mut().enumerate() {
            if app.id.trim().is_empty() {
                app.id = new_id();
            }
            app.workspace_id = ws.id.clone();
            app.sort_order = i as i64;
            validate_app(app)?;
        }
        let guard = self.lock()?;
        let tx = guard.unchecked_transaction()?;
        let changed = tx.execute(
            "UPDATE workspaces SET name=?1, description=?2, icon=?3, color=?4,
                    is_favorite=?5, hotkey=?6, launch_settings=?7, updated_at=?8
             WHERE id=?9",
            params![
                ws.name,
                ws.description,
                ws.icon,
                ws.color,
                i64::from(ws.is_favorite),
                ws.hotkey,
                serde_json::to_string(&ws.launch_settings).unwrap_or_else(|_| "{}".to_owned()),
                ws.updated_at,
                ws.id,
            ],
        )?;
        if changed == 0 {
            return Err(WorksetError::NotFound(format!("workspace {}", ws.id)));
        }
        tx.execute(
            "DELETE FROM workspace_apps WHERE workspace_id=?1",
            params![ws.id],
        )?;
        Self::insert_apps_tx(&tx, &ws.id, &ws.apps, &ws.updated_at)?;
        tx.commit()?;
        Ok(ws)
    }

    pub fn delete_workspace(&self, id: &str) -> Result<(), WorksetError> {
        let conn = self.lock()?;
        let changed = conn.execute("DELETE FROM workspaces WHERE id=?1", params![id])?;
        if changed == 0 {
            return Err(WorksetError::NotFound(format!("workspace {id}")));
        }
        Ok(())
    }

    pub fn duplicate_workspace(&self, id: &str) -> Result<Workspace, WorksetError> {
        let mut ws = self.get_workspace(id)?;
        ws.id = new_id();
        ws.name = format!("{} (copy)", ws.name);
        ws.is_favorite = false;
        ws.hotkey.clear();
        let now = now_iso();
        ws.created_at = now.clone();
        ws.updated_at = now;
        for app in ws.apps.iter_mut() {
            app.id = new_id();
            app.workspace_id = ws.id.clone();
        }
        self.create_workspace(ws)
    }

    pub fn toggle_favorite(&self, id: &str) -> Result<Workspace, WorksetError> {
        {
            let conn = self.lock()?;
            let changed = conn.execute(
                "UPDATE workspaces SET is_favorite = 1 - is_favorite,
                        updated_at = ?1 WHERE id = ?2",
                params![now_iso(), id],
            )?;
            if changed == 0 {
                return Err(WorksetError::NotFound(format!("workspace {id}")));
            }
        }
        self.get_workspace(id)
    }

    // ---------- apps ----------

    pub fn get_apps(&self, workspace_id: &str) -> Result<Vec<WorkspaceApp>, WorksetError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, workspace_id, sort_order, name, exe_path, args, cwd, url,
                    app_icon, match_rules, launch_delay_ms, target_monitor,
                    x, y, w, h, window_state, launch_policy
             FROM workspace_apps WHERE workspace_id = ?1
             ORDER BY sort_order ASC, rowid ASC",
        )?;
        let apps = stmt
            .query_map(params![workspace_id], row_to_app)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(apps)
    }

    fn next_sort_order(&self, workspace_id: &str) -> Result<i64, WorksetError> {
        let conn = self.lock()?;
        let max: Option<i64> = conn
            .query_row(
                "SELECT MAX(sort_order) FROM workspace_apps WHERE workspace_id=?1",
                params![workspace_id],
                |r| r.get(0),
            )
            .unwrap_or(None);
        Ok(max.unwrap_or(-1) + 1)
    }

    pub fn create_app(&self, mut app: WorkspaceApp) -> Result<WorkspaceApp, WorksetError> {
        validate_app(&app)?;
        if app.id.trim().is_empty() {
            app.id = new_id();
        }
        if app.workspace_id.trim().is_empty() {
            return Err(WorksetError::Validation(
                "app workspace_id is empty".to_owned(),
            ));
        }
        app.sort_order = self.next_sort_order(&app.workspace_id)?;
        let now = now_iso();
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO workspace_apps
             (id, workspace_id, sort_order, name, exe_path, args, cwd, url,
              app_icon, match_rules, launch_delay_ms, target_monitor,
              x, y, w, h, window_state, launch_policy, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
            params![
                app.id,
                app.workspace_id,
                app.sort_order,
                app.name,
                app.exe_path,
                serde_json::to_string(&app.args).unwrap_or_else(|_| "[]".to_owned()),
                app.cwd,
                app.url,
                app.app_icon,
                serde_json::to_string(&app.match_rules).unwrap_or_else(|_| "{}".to_owned()),
                app.launch_delay_ms,
                app.target_monitor,
                app.x,
                app.y,
                app.w,
                app.h,
                app.window_state,
                app.launch_policy,
                now,
            ],
        )?;
        Ok(app)
    }

    pub fn update_app(&self, app: WorkspaceApp) -> Result<WorkspaceApp, WorksetError> {
        validate_app(&app)?;
        let conn = self.lock()?;
        let changed = conn.execute(
            "UPDATE workspace_apps SET workspace_id=?1, sort_order=?2, name=?3,
                    exe_path=?4, args=?5, cwd=?6, url=?7, app_icon=?8,
                    match_rules=?9, launch_delay_ms=?10, target_monitor=?11,
                    x=?12, y=?13, w=?14, h=?15, window_state=?16, launch_policy=?17
             WHERE id=?18",
            params![
                app.workspace_id,
                app.sort_order,
                app.name,
                app.exe_path,
                serde_json::to_string(&app.args).unwrap_or_else(|_| "[]".to_owned()),
                app.cwd,
                app.url,
                app.app_icon,
                serde_json::to_string(&app.match_rules).unwrap_or_else(|_| "{}".to_owned()),
                app.launch_delay_ms,
                app.target_monitor,
                app.x,
                app.y,
                app.w,
                app.h,
                app.window_state,
                app.launch_policy,
                app.id,
            ],
        )?;
        if changed == 0 {
            return Err(WorksetError::NotFound(format!("app {}", app.id)));
        }
        Ok(app)
    }

    pub fn delete_app(&self, id: &str) -> Result<(), WorksetError> {
        let conn = self.lock()?;
        let changed = conn.execute("DELETE FROM workspace_apps WHERE id=?1", params![id])?;
        if changed == 0 {
            return Err(WorksetError::NotFound(format!("app {id}")));
        }
        Ok(())
    }

    pub fn reorder_apps(
        &self,
        workspace_id: &str,
        ordered_ids: &[String],
    ) -> Result<(), WorksetError> {
        let guard = self.lock()?;
        let tx = guard.unchecked_transaction()?;
        for (i, id) in ordered_ids.iter().enumerate() {
            tx.execute(
                "UPDATE workspace_apps SET sort_order=?1 WHERE id=?2 AND workspace_id=?3",
                params![i as i64, id, workspace_id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    // ---------- sessions ----------

    pub fn create_session(&self, workspace_id: &str) -> Result<SessionInfo, WorksetError> {
        {
            let conn = self.lock()?;
            let exists: Option<String> = conn
                .query_row(
                    "SELECT id FROM workspaces WHERE id=?1",
                    params![workspace_id],
                    |r| r.get(0),
                )
                .optional()?;
            if exists.is_none() {
                return Err(WorksetError::NotFound(format!("workspace {workspace_id}")));
            }
        }
        let s = SessionInfo {
            id: new_id(),
            workspace_id: workspace_id.to_owned(),
            status: "running".to_owned(),
            started_at: now_iso(),
            finished_at: None,
            windows: Vec::new(),
        };
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO launch_sessions (id, workspace_id, status, started_at, finished_at)
             VALUES (?1,?2,?3,?4,?5)",
            params![s.id, s.workspace_id, s.status, s.started_at, s.finished_at],
        )?;
        Ok(s)
    }

    pub fn update_session_status(
        &self,
        id: &str,
        status: &str,
        finished_at: Option<String>,
    ) -> Result<(), WorksetError> {
        let conn = self.lock()?;
        let changed = conn.execute(
            "UPDATE launch_sessions SET status=?1, finished_at=COALESCE(?2, finished_at)
             WHERE id=?3",
            params![status, finished_at, id],
        )?;
        if changed == 0 {
            return Err(WorksetError::NotFound(format!("session {id}")));
        }
        Ok(())
    }

    pub fn get_session(&self, id: &str) -> Result<SessionInfo, WorksetError> {
        let mut s = {
            let conn = self.lock()?;
            conn.query_row(
                "SELECT id, workspace_id, status, started_at, finished_at
                 FROM launch_sessions WHERE id=?1",
                params![id],
                row_to_session,
            )
            .optional()?
            .ok_or_else(|| WorksetError::NotFound(format!("session {id}")))?
        };
        s.windows = self.list_session_windows(&s.id)?;
        Ok(s)
    }

    pub fn list_sessions(
        &self,
        workspace_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SessionInfo>, WorksetError> {
        let ids: Vec<String> = {
            let conn = self.lock()?;
            let mut out = Vec::new();
            if let Some(ws) = workspace_id {
                let mut stmt = conn.prepare(
                    "SELECT id FROM launch_sessions WHERE workspace_id=?1
                     ORDER BY started_at DESC LIMIT ?2",
                )?;
                for id in stmt.query_map(params![ws, limit], |r| r.get(0))? {
                    out.push(id?);
                }
            } else {
                let mut stmt = conn
                    .prepare("SELECT id FROM launch_sessions ORDER BY started_at DESC LIMIT ?1")?;
                for id in stmt.query_map(params![limit], |r| r.get(0))? {
                    out.push(id?);
                }
            }
            out
        };
        let mut sessions = Vec::with_capacity(ids.len());
        for id in ids {
            sessions.push(self.get_session(&id)?);
        }
        Ok(sessions)
    }

    pub fn upsert_session_window(&self, row: &SessionWindowRow) -> Result<(), WorksetError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO session_windows
             (id, session_id, app_id, pid, hwnd, status, message, last_rect, launched_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
             ON CONFLICT(id) DO UPDATE SET
               session_id=excluded.session_id, app_id=excluded.app_id,
               pid=excluded.pid, hwnd=excluded.hwnd, status=excluded.status,
               message=excluded.message, last_rect=excluded.last_rect,
               launched_at=excluded.launched_at",
            params![
                row.id,
                row.session_id,
                row.app_id,
                row.pid,
                row.hwnd,
                row.status,
                row.message,
                serde_json::to_string(&row.last_rect).unwrap_or_else(|_| "{}".to_owned()),
                row.launched_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_session_window_status(
        &self,
        id: &str,
        status: &str,
        message: &str,
    ) -> Result<(), WorksetError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE session_windows SET status=?1, message=?2 WHERE id=?3",
            params![status, message, id],
        )?;
        Ok(())
    }

    pub fn list_session_windows(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionWindowRow>, WorksetError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, app_id, pid, hwnd, status, message, last_rect, launched_at
             FROM session_windows WHERE session_id=?1 ORDER BY launched_at ASC, rowid ASC",
        )?;
        let rows = stmt
            .query_map(params![session_id], row_to_session_window)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // ---------- settings ----------

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, WorksetError> {
        let conn = self.lock()?;
        let v: Option<String> = conn
            .query_row(
                "SELECT value FROM user_settings WHERE key=?1",
                params![key],
                |r| r.get(0),
            )
            .optional()?;
        Ok(v)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), WorksetError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO user_settings (key, value) VALUES (?1,?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    // ---------- logs ----------

    pub fn push_log(
        &self,
        session_id: &str,
        app_id: &str,
        level: &str,
        message: &str,
    ) -> Result<(), WorksetError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO launch_logs (session_id, app_id, level, message, created_at)
             VALUES (?1,?2,?3,?4,?5)",
            params![session_id, app_id, level, message, now_iso()],
        )?;
        Ok(())
    }

    pub fn list_logs(
        &self,
        session_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<LaunchLog>, WorksetError> {
        let conn = self.lock()?;
        let mut out = Vec::new();
        if let Some(sid) = session_id {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, app_id, level, message, created_at
                 FROM launch_logs WHERE session_id=?1 ORDER BY id DESC LIMIT ?2",
            )?;
            for row in stmt.query_map(params![sid, limit], row_to_log)? {
                out.push(row?);
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, app_id, level, message, created_at
                 FROM launch_logs ORDER BY id DESC LIMIT ?1",
            )?;
            for row in stmt.query_map(params![limit], row_to_log)? {
                out.push(row?);
            }
        }
        Ok(out)
    }

    // ---------- app catalog ----------

    pub fn upsert_catalog(&self, entry: &crate::models::DiscoveredApp) -> Result<(), WorksetError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO app_catalog (exe_path, name, icon, source, last_seen)
             VALUES (?1,?2,?3,?4,?5)
             ON CONFLICT(exe_path) DO UPDATE SET
               name=excluded.name, icon=excluded.icon,
               source=excluded.source, last_seen=excluded.last_seen",
            params![
                entry.exe_path,
                entry.name,
                entry.icon,
                entry.source,
                now_iso()
            ],
        )?;
        Ok(())
    }

    pub fn list_catalog(&self) -> Result<Vec<crate::models::DiscoveredApp>, WorksetError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT exe_path, name, icon, source FROM app_catalog
             ORDER BY last_seen DESC LIMIT 2000",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok(crate::models::DiscoveredApp {
                    exe_path: r.get(0)?,
                    name: r.get(1)?,
                    icon: r.get(2)?,
                    source: r.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_workspace() -> Workspace {
        Workspace {
            id: String::new(),
            name: "Work".to_owned(),
            description: "d".to_owned(),
            icon: String::new(),
            color: "#0078d4".to_owned(),
            is_favorite: false,
            hotkey: String::new(),
            launch_settings: serde_json::json!({}),
            created_at: String::new(),
            updated_at: String::new(),
            apps: vec![WorkspaceApp {
                id: String::new(),
                workspace_id: String::new(),
                sort_order: 0,
                name: "App".to_owned(),
                exe_path: "C:\\a.exe".to_owned(),
                args: vec![],
                cwd: String::new(),
                url: String::new(),
                app_icon: String::new(),
                match_rules: MatchRules::default(),
                launch_delay_ms: 0,
                target_monitor: String::new(),
                x: 0.0,
                y: 0.0,
                w: 0.5,
                h: 0.5,
                window_state: "normal".to_owned(),
                launch_policy: "if_not_running".to_owned(),
            }],
        }
    }

    #[test]
    fn workspace_crud_roundtrip() {
        let db = Db::open_in_memory().unwrap();
        let ws = db.create_workspace(sample_workspace()).unwrap();
        assert!(!ws.id.is_empty());
        assert_eq!(ws.apps.len(), 1);
        assert_eq!(ws.apps[0].workspace_id, ws.id);

        let fetched = db.get_workspace(&ws.id).unwrap();
        assert_eq!(fetched.name, "Work");
        assert_eq!(fetched.apps.len(), 1);

        let all = db.get_workspaces().unwrap();
        assert_eq!(all.len(), 1);

        let fav = db.toggle_favorite(&ws.id).unwrap();
        assert!(fav.is_favorite);

        let dup = db.duplicate_workspace(&ws.id).unwrap();
        assert_ne!(dup.id, ws.id);
        assert!(dup.name.contains("copy"));
        assert_eq!(dup.apps.len(), 1);
        assert_eq!(db.get_workspaces().unwrap().len(), 2);

        db.delete_workspace(&ws.id).unwrap();
        assert!(db.get_workspace(&ws.id).is_err());
    }

    #[test]
    fn app_crud_and_reorder() {
        let db = Db::open_in_memory().unwrap();
        let ws = db.create_workspace(sample_workspace()).unwrap();
        let mut second = ws.apps[0].clone();
        second.id = String::new();
        second.name = "Second".to_owned();
        let created = db.create_app(second).unwrap();
        assert_eq!(created.sort_order, 1);

        db.reorder_apps(&ws.id, &[created.id.clone(), ws.apps[0].id.clone()])
            .unwrap();
        let apps = db.get_apps(&ws.id).unwrap();
        assert_eq!(apps[0].id, created.id);

        db.delete_app(&created.id).unwrap();
        assert_eq!(db.get_apps(&ws.id).unwrap().len(), 1);
    }

    #[test]
    fn sessions_logs_settings_catalog() {
        let db = Db::open_in_memory().unwrap();
        let ws = db.create_workspace(sample_workspace()).unwrap();
        let s = db.create_session(&ws.id).unwrap();
        db.push_log(&s.id, "a1", "info", "hello").unwrap();
        db.upsert_session_window(&SessionWindowRow {
            id: "sw1".to_owned(),
            session_id: s.id.clone(),
            app_id: "a1".to_owned(),
            pid: Some(123),
            hwnd: Some(456),
            status: "done".to_owned(),
            message: String::new(),
            last_rect: serde_json::json!({"x":0}),
            launched_at: now_iso(),
        })
        .unwrap();
        let got = db.get_session(&s.id).unwrap();
        assert_eq!(got.windows.len(), 1);
        db.update_session_status(&s.id, "done", Some(now_iso()))
            .unwrap();
        assert_eq!(db.list_sessions(Some(&ws.id), 10).unwrap().len(), 1);
        assert_eq!(db.list_logs(Some(&s.id), 10).unwrap().len(), 1);

        assert!(db.get_setting("theme").unwrap().is_none());
        db.set_setting("theme", "dark").unwrap();
        assert_eq!(db.get_setting("theme").unwrap().as_deref(), Some("dark"));

        db.upsert_catalog(&DiscoveredApp {
            name: "X".to_owned(),
            exe_path: "C:\\x.exe".to_owned(),
            source: "installed".to_owned(),
            icon: String::new(),
        })
        .unwrap();
        assert_eq!(db.list_catalog().unwrap().len(), 1);
    }
}
