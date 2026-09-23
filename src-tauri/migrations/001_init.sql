-- Workset schema v1
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS workspaces (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  icon TEXT NOT NULL DEFAULT '',
  color TEXT NOT NULL DEFAULT '#0078d4',
  is_favorite INTEGER NOT NULL DEFAULT 0,
  hotkey TEXT,
  launch_settings TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_workspaces_fav ON workspaces(is_favorite);
CREATE INDEX IF NOT EXISTS idx_workspaces_updated ON workspaces(updated_at);

CREATE TABLE IF NOT EXISTS workspace_apps (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  sort_order INTEGER NOT NULL DEFAULT 0,
  name TEXT NOT NULL,
  exe_path TEXT NOT NULL DEFAULT '',
  args TEXT NOT NULL DEFAULT '[]',
  cwd TEXT NOT NULL DEFAULT '',
  url TEXT NOT NULL DEFAULT '',
  app_icon TEXT NOT NULL DEFAULT '',
  match_rules TEXT NOT NULL DEFAULT '{}',
  launch_delay_ms INTEGER NOT NULL DEFAULT 0,
  target_monitor TEXT NOT NULL DEFAULT '',
  x REAL NOT NULL DEFAULT 0,
  y REAL NOT NULL DEFAULT 0,
  w REAL NOT NULL DEFAULT 0.5,
  h REAL NOT NULL DEFAULT 0.5,
  window_state TEXT NOT NULL DEFAULT 'normal',
  launch_policy TEXT NOT NULL DEFAULT 'if_not_running',
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_apps_ws ON workspace_apps(workspace_id, sort_order);

CREATE TABLE IF NOT EXISTS monitor_configs (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  monitor_id TEXT NOT NULL,
  x INTEGER NOT NULL, y INTEGER NOT NULL,
  w INTEGER NOT NULL, h INTEGER NOT NULL,
  work_x INTEGER NOT NULL, work_y INTEGER NOT NULL,
  work_w INTEGER NOT NULL, work_h INTEGER NOT NULL,
  scale REAL NOT NULL DEFAULT 1.0,
  is_primary INTEGER NOT NULL DEFAULT 0,
  name TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS layout_configs (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  name TEXT NOT NULL DEFAULT 'default',
  preset TEXT NOT NULL DEFAULT 'custom',
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_catalog (
  exe_path TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  icon TEXT NOT NULL DEFAULT '',
  source TEXT NOT NULL DEFAULT 'discovered',
  last_seen TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS launch_sessions (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  status TEXT NOT NULL DEFAULT 'running',
  started_at TEXT NOT NULL,
  finished_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_sessions_ws ON launch_sessions(workspace_id, started_at DESC);

CREATE TABLE IF NOT EXISTS session_windows (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES launch_sessions(id) ON DELETE CASCADE,
  app_id TEXT NOT NULL,
  pid INTEGER,
  hwnd INTEGER,
  status TEXT NOT NULL DEFAULT 'pending',
  message TEXT NOT NULL DEFAULT '',
  last_rect TEXT NOT NULL DEFAULT '{}',
  launched_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_swin_session ON session_windows(session_id);

CREATE TABLE IF NOT EXISTS user_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS launch_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id TEXT NOT NULL,
  app_id TEXT NOT NULL DEFAULT '',
  level TEXT NOT NULL DEFAULT 'info',
  message TEXT NOT NULL,
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_logs_session ON launch_logs(session_id);
