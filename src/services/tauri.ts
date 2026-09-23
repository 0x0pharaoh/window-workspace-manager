import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  CapturedWindow,
  DiscoveredApp,
  LaunchProgressItem,
  MonitorInfo,
  SessionInfo,
  UserSettings,
  Workspace,
  WorkspaceApp,
  WorkspaceAppDraft,
  WorkspaceExportEnvelope,
} from "@/types";

function uid(): string {
  try {
    return crypto.randomUUID();
  } catch {
    return `id-${Date.now()}-${Math.floor(Math.random() * 1_000_000)}`;
  }
}

function nowIso(): string {
  return new Date().toISOString();
}

export const DEFAULT_SETTINGS: UserSettings = {
  theme: "system",
  default_timeout_ms: 15000,
  match_policy: "exe_and_title",
  confirm_launch: false,
  notifications: true,
  autostart: false,
  launch_on_startup_workspace: "",
  storage_path: "",
  hotkeys: {},
};

async function safeInvoke<T>(cmd: string, args: Record<string, unknown> | undefined, fallback: T): Promise<T> {
  try {
    const v = await invoke<T>(cmd, args);
    return v;
  } catch (err) {
    console.warn(`[tauri] invoke "${cmd}" failed, using fallback:`, err);
    return fallback;
  }
}

function blankApp(workspaceId: string, draft: Partial<WorkspaceAppDraft>, index: number): WorkspaceApp {
  return {
    id: uid(),
    workspace_id: workspaceId,
    name: draft.name ?? "New app",
    exe_path: draft.exe_path ?? "",
    args: draft.args ?? "",
    cwd: draft.cwd ?? "",
    url: draft.url ?? "",
    delay_ms: draft.delay_ms ?? 0,
    monitor_id: draft.monitor_id ?? "",
    x: draft.x ?? 0,
    y: draft.y ?? 0,
    w: draft.w ?? 0.5,
    h: draft.h ?? 0.5,
    state: draft.state ?? "normal",
    policy: draft.policy ?? "if_not_running",
    match_rules: draft.match_rules ?? {},
    sort_order: draft.sort_order ?? index,
  };
}

// ---------- Monitors & windows ----------

export function getMonitors(): Promise<MonitorInfo[]> {
  return safeInvoke<MonitorInfo[]>("get_monitors", undefined, []);
}

export function enumWindows(): Promise<CapturedWindow[]> {
  return safeInvoke<CapturedWindow[]>("enum_windows", undefined, []);
}

export function captureDesktop(): Promise<CapturedWindow[]> {
  return safeInvoke<CapturedWindow[]>("capture_desktop", undefined, []);
}

export function discoverApps(): Promise<DiscoveredApp[]> {
  return safeInvoke<DiscoveredApp[]>("discover_apps", undefined, []);
}

// ---------- Workspaces ----------

export function getWorkspaces(): Promise<Workspace[]> {
  return safeInvoke<Workspace[]>("get_workspaces", undefined, []);
}

export function getWorkspace(id: string): Promise<Workspace | null> {
  return safeInvoke<Workspace | null>("get_workspace", { id }, null);
}

export async function createWorkspace(name: string): Promise<Workspace> {
  const fallback: Workspace = {
    id: uid(),
    name,
    description: "",
    icon: "",
    color: "#0078d4",
    hotkey: "",
    favorite: false,
    created_at: nowIso(),
    updated_at: nowIso(),
    last_launched_at: null,
    apps: [],
  };
  return safeInvoke<Workspace>("create_workspace", { name }, fallback);
}

export async function updateWorkspace(id: string, patch: Partial<Workspace>): Promise<Workspace> {
  const fallback: Workspace = {
    id,
    name: patch.name ?? "Workspace",
    description: patch.description ?? "",
    icon: patch.icon ?? "",
    color: patch.color ?? "#0078d4",
    hotkey: patch.hotkey ?? "",
    favorite: patch.favorite ?? false,
    created_at: nowIso(),
    updated_at: nowIso(),
    last_launched_at: patch.last_launched_at ?? null,
    apps: patch.apps ?? [],
  };
  return safeInvoke<Workspace>("update_workspace", { id, patch }, fallback);
}

export async function deleteWorkspace(id: string): Promise<void> {
  await safeInvoke<void>("delete_workspace", { id }, undefined);
}

export async function duplicateWorkspace(id: string): Promise<Workspace> {
  const fallback: Workspace = {
    id: uid(),
    name: "Copy",
    description: "",
    icon: "",
    color: "#0078d4",
    hotkey: "",
    favorite: false,
    created_at: nowIso(),
    updated_at: nowIso(),
    last_launched_at: null,
    apps: [],
  };
  return safeInvoke<Workspace>("duplicate_workspace", { id }, fallback);
}

export async function toggleFavorite(id: string): Promise<Workspace> {
  const fallback: Workspace = {
    id,
    name: "Workspace",
    description: "",
    icon: "",
    color: "#0078d4",
    hotkey: "",
    favorite: true,
    created_at: nowIso(),
    updated_at: nowIso(),
    last_launched_at: null,
    apps: [],
  };
  return safeInvoke<Workspace>("toggle_favorite", { id }, fallback);
}

// ---------- Apps ----------

export async function createApp(workspaceId: string, input: WorkspaceAppDraft): Promise<WorkspaceApp> {
  const fallback = blankApp(workspaceId, input, 0);
  return safeInvoke<WorkspaceApp>("create_app", { workspaceId, input }, fallback);
}

export async function updateApp(appId: string, patch: Partial<WorkspaceApp>): Promise<WorkspaceApp> {
  const fallback: WorkspaceApp = {
    id: appId,
    workspace_id: patch.workspace_id ?? "",
    name: patch.name ?? "App",
    exe_path: patch.exe_path ?? "",
    args: patch.args ?? "",
    cwd: patch.cwd ?? "",
    url: patch.url ?? "",
    delay_ms: patch.delay_ms ?? 0,
    monitor_id: patch.monitor_id ?? "",
    x: patch.x ?? 0,
    y: patch.y ?? 0,
    w: patch.w ?? 0.5,
    h: patch.h ?? 0.5,
    state: patch.state ?? "normal",
    policy: patch.policy ?? "if_not_running",
    match_rules: patch.match_rules ?? {},
    sort_order: patch.sort_order ?? 0,
  };
  return safeInvoke<WorkspaceApp>("update_app", { appId, patch }, fallback);
}

export async function deleteApp(appId: string): Promise<void> {
  await safeInvoke<void>("delete_app", { appId }, undefined);
}

export async function reorderApps(workspaceId: string, orderedIds: string[]): Promise<void> {
  await safeInvoke<void>("reorder_apps", { workspaceId, orderedIds }, undefined);
}

// ---------- Launch & sessions ----------

export function launchWorkspace(id: string): Promise<string> {
  return safeInvoke<string>("launch_workspace", { id }, `session-${uid()}`);
}

export async function cancelLaunch(workspaceId: string): Promise<void> {
  await safeInvoke<void>("cancel_launch", { workspaceId }, undefined);
}

export function getSessions(): Promise<SessionInfo[]> {
  return safeInvoke<SessionInfo[]>("get_sessions", undefined, []);
}

export async function closeSessionApp(sessionId: string, appId: string): Promise<void> {
  await safeInvoke<void>("close_session_app", { sessionId, appId }, undefined);
}

export async function closeWorkspaceSession(sessionId: string): Promise<void> {
  await safeInvoke<void>("close_workspace_session", { sessionId }, undefined);
}

export function recoverOffscreen(): Promise<number> {
  return safeInvoke<number>("recover_offscreen", undefined, 0);
}

// ---------- Import / export ----------

export function serializeWorkspace(ws: Workspace): string {
  const env: WorkspaceExportEnvelope = {
    version: 1,
    exported_at: nowIso(),
    workspace: ws,
  };
  return JSON.stringify(env, null, 2);
}

export function validateWorkspaceImport(data: unknown): { ok: boolean; error?: string } {
  if (typeof data !== "object" || data === null) return { ok: false, error: "Not an object" };
  const rec = data as Record<string, unknown>;
  const payload = rec["workspace"] ?? data;
  if (typeof payload !== "object" || payload === null) return { ok: false, error: "Missing workspace" };
  const ws = payload as Record<string, unknown>;
  if (typeof ws["name"] !== "string" || (ws["name"] as string).trim() === "") {
    return { ok: false, error: "Workspace name is required" };
  }
  if ("apps" in ws && !Array.isArray(ws["apps"])) {
    return { ok: false, error: "apps must be an array" };
  }
  if (Array.isArray(ws["apps"])) {
    for (const a of ws["apps"] as unknown[]) {
      if (typeof a !== "object" || a === null) return { ok: false, error: "Invalid app entry" };
      const app = a as Record<string, unknown>;
      if (typeof app["name"] !== "string" || (app["name"] as string).trim() === "") {
        return { ok: false, error: "Each app needs a name" };
      }
      for (const k of ["x", "y", "w", "h"] as const) {
        const v = app[k];
        if (v !== undefined && (typeof v !== "number" || Number.isNaN(v) || v < 0 || v > 1)) {
          return { ok: false, error: `App "${String(app["name"])}" has invalid ${k}` };
        }
      }
    }
  }
  if ("version" in rec && rec["version"] !== 1 && rec["version"] !== undefined) {
    return { ok: false, error: "Unsupported version" };
  }
  return { ok: true };
}

export function parseWorkspaceImport(json: string): Workspace {
  let data: unknown;
  try {
    data = JSON.parse(json) as unknown;
  } catch {
    throw new Error("Invalid JSON");
  }
  const res = validateWorkspaceImport(data);
  if (!res.ok) throw new Error(res.error ?? "Invalid workspace file");
  const rec = data as Record<string, unknown>;
  const raw = (rec["workspace"] ?? data) as Record<string, unknown>;
  const appsRaw = Array.isArray(raw["apps"]) ? (raw["apps"] as Record<string, unknown>[]) : [];
  const wsId = typeof raw["id"] === "string" ? (raw["id"] as string) : uid();
  const apps: WorkspaceApp[] = appsRaw.map((a, i) => ({
    id: typeof a["id"] === "string" ? (a["id"] as string) : uid(),
    workspace_id: wsId,
    name: String(a["name"] ?? "App"),
    exe_path: String(a["exe_path"] ?? ""),
    args: String(a["args"] ?? ""),
    cwd: String(a["cwd"] ?? ""),
    url: String(a["url"] ?? ""),
    delay_ms: typeof a["delay_ms"] === "number" ? (a["delay_ms"] as number) : 0,
    monitor_id: String(a["monitor_id"] ?? ""),
    x: typeof a["x"] === "number" ? (a["x"] as number) : 0,
    y: typeof a["y"] === "number" ? (a["y"] as number) : 0,
    w: typeof a["w"] === "number" ? (a["w"] as number) : 0.5,
    h: typeof a["h"] === "number" ? (a["h"] as number) : 0.5,
    state: (a["state"] as WorkspaceApp["state"]) ?? "normal",
    policy: (a["policy"] as WorkspaceApp["policy"]) ?? "if_not_running",
    match_rules: (a["match_rules"] as WorkspaceApp["match_rules"]) ?? {},
    sort_order: typeof a["sort_order"] === "number" ? (a["sort_order"] as number) : i,
  }));
  return {
    id: wsId,
    name: String(raw["name"] ?? "Imported"),
    description: String(raw["description"] ?? ""),
    icon: String(raw["icon"] ?? ""),
    color: String(raw["color"] ?? "#0078d4"),
    hotkey: String(raw["hotkey"] ?? ""),
    favorite: Boolean(raw["favorite"] ?? false),
    created_at: typeof raw["created_at"] === "string" ? (raw["created_at"] as string) : nowIso(),
    updated_at: nowIso(),
    last_launched_at: null,
    apps,
  };
}

export async function importWorkspace(json: string, confirmed = false): Promise<Workspace> {
  const parsed = parseWorkspaceImport(json);
  return safeInvoke<Workspace>("import_workspace", { json, confirmed }, parsed);
}

export async function exportWorkspace(id: string): Promise<string> {
  return safeInvoke<string>("export_workspace", { id }, JSON.stringify({ version: 1, workspace: { id } }));
}

// ---------- Settings & logs ----------

export function getSettings(): Promise<UserSettings> {
  return safeInvoke<UserSettings>("get_settings", undefined, { ...DEFAULT_SETTINGS });
}

export async function setSetting(key: string, value: unknown): Promise<void> {
  await safeInvoke<void>("set_setting", { key, value }, undefined);
}

export function getLogs(limit = 200): Promise<string[]> {
  return safeInvoke<string[]>("get_logs", { limit }, []);
}

// ---------- Dialogs ----------

export async function pickExeFile(): Promise<string | null> {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const sel = await open({
      multiple: false,
      filters: [{ name: "Executables", extensions: ["exe", "bat", "cmd", "lnk"] }],
    });
    if (typeof sel === "string") return sel;
    return null;
  } catch (err) {
    console.warn("[tauri] pickExeFile fallback:", err);
    return null;
  }
}

export async function saveFile(defaultName: string, content: string): Promise<void> {
  const triggerDownload = (): void => {
    try {
      const blob = new Blob([content], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = defaultName;
      document.body.appendChild(a);
      a.click();
      a.remove();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
    } catch (err) {
      console.warn("[tauri] saveFile download failed:", err);
    }
  };
  try {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const path = await save({ defaultPath: defaultName });
    if (!path) {
      triggerDownload();
      return;
    }
    // No fs plugin in scope; fall back to browser download so data is never lost.
    triggerDownload();
  } catch {
    triggerDownload();
  }
}

// ---------- Events ----------

export async function onLaunchProgress(
  cb: (p: LaunchProgressItem) => void,
): Promise<() => void> {
  try {
    const unlisten = await listen<LaunchProgressItem>("launch-progress", (e) => {
      cb(e.payload);
    });
    return unlisten;
  } catch (err) {
    console.warn("[tauri] launch-progress listen failed:", err);
    return () => undefined;
  }
}
