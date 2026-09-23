export type WindowState = "normal" | "maximized" | "minimized" | "fullscreen";

export type LaunchPolicy =
  | "if_not_running"
  | "always_new"
  | "reuse"
  | "focus"
  | "skip"
  | "ask";

export type ExportFormat = "json";

export type ViewKey =
  | "home"
  | "workspaces"
  | "editor"
  | "apps"
  | "sessions"
  | "settings";

export interface MatchRules {
  title_contains?: string;
  title_regex?: string;
  window_class?: string;
}

export interface MonitorInfo {
  id: string;
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  work_x: number;
  work_y: number;
  work_width: number;
  work_height: number;
  scale_factor: number;
  is_primary: boolean;
}

export interface CapturedWindow {
  hwnd?: number;
  title: string;
  exe_path: string;
  window_class: string;
  x: number;
  y: number;
  width: number;
  height: number;
  monitor_id: string;
  state: WindowState;
}

export interface DiscoveredApp {
  name: string;
  exe_path: string;
  icon?: string;
  source: "installed" | "running" | "start_menu" | "custom";
  args?: string;
}

export interface WorkspaceApp {
  id: string;
  workspace_id: string;
  name: string;
  exe_path: string;
  args: string;
  cwd: string;
  url: string;
  delay_ms: number;
  monitor_id: string;
  /** Normalized 0..1 relative to monitor work area */
  x: number;
  y: number;
  w: number;
  h: number;
  state: WindowState;
  policy: LaunchPolicy;
  match_rules: MatchRules;
  sort_order: number;
}

export type WorkspaceAppDraft = Omit<WorkspaceApp, "id" | "workspace_id" | "sort_order"> & {
  sort_order?: number;
};

export interface Workspace {
  id: string;
  name: string;
  description: string;
  icon: string;
  color: string;
  hotkey: string;
  favorite: boolean;
  created_at: string;
  updated_at: string;
  last_launched_at?: string | null;
  apps: WorkspaceApp[];
}

export type ProgressStatus =
  | "pending"
  | "launching"
  | "waiting"
  | "positioning"
  | "done"
  | "failed"
  | "cancelled";

export interface LaunchProgressItem {
  workspace_id: string;
  app_id: string;
  app_name: string;
  status: ProgressStatus;
  message: string;
  progress: number;
}

export interface SessionWindowRow {
  id: string;
  session_id: string;
  app_id: string;
  app_name: string;
  pid?: number | null;
  hwnd?: number | null;
  status: string;
  message?: string;
  last_x?: number | null;
  last_y?: number | null;
}

export interface SessionInfo {
  id: string;
  workspace_id: string;
  workspace_name: string;
  started_at: string;
  status: string;
  windows: SessionWindowRow[];
}

export type ThemeChoice = "light" | "dark" | "system";

export interface UserSettings {
  theme: ThemeChoice;
  default_timeout_ms: number;
  match_policy: string;
  confirm_launch: boolean;
  notifications: boolean;
  autostart: boolean;
  launch_on_startup_workspace: string;
  storage_path: string;
  hotkeys: Record<string, string>;
}

export interface WorkspaceExportEnvelope {
  version: 1;
  exported_at: string;
  workspace: Workspace;
}

export const NAV_VIEWS: ViewKey[] = [
  "home",
  "workspaces",
  "editor",
  "apps",
  "sessions",
  "settings",
];
