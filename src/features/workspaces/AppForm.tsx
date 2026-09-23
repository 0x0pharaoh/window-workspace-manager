import { useState } from "react";
import type { WorkspaceApp } from "@/types";
import { useMonitors } from "@/stores/useMonitors";
import { pickExeFile } from "@/services/tauri";
import { Button, Dialog, Input, Select } from "@/components/ui";

const STATES: WorkspaceApp["state"][] = ["normal", "maximized", "minimized", "fullscreen"];
const POLICIES: WorkspaceApp["policy"][] = [
  "if_not_running",
  "always_new",
  "reuse",
  "focus",
  "skip",
  "ask",
];

function toPct(v: number): number {
  return Math.round(v * 100);
}

export function AppForm({
  app,
  onSave,
  onCancel,
  saving,
}: {
  app: WorkspaceApp;
  onSave: (patch: Partial<WorkspaceApp>) => Promise<void> | void;
  onCancel: () => void;
  saving?: boolean;
}) {
  const { monitors } = useMonitors();
  const [form, setForm] = useState({
    name: app.name,
    exe_path: app.exe_path,
    args: app.args,
    cwd: app.cwd,
    url: app.url,
    delay_ms: app.delay_ms,
    monitor_id: app.monitor_id,
    x: toPct(app.x),
    y: toPct(app.y),
    w: toPct(app.w),
    h: toPct(app.h),
    state: app.state,
    policy: app.policy,
    title_contains: app.match_rules.title_contains ?? "",
    title_regex: app.match_rules.title_regex ?? "",
    window_class: app.match_rules.window_class ?? "",
  });
  const [regexError, setRegexError] = useState<string | null>(null);

  function set<K extends keyof typeof form>(key: K, value: (typeof form)[K]): void {
    setForm((f) => ({ ...f, [key]: value }));
  }

  async function browse(): Promise<void> {
    const p = await pickExeFile();
    if (p) set("exe_path", p);
  }

  async function submit(e: React.FormEvent): Promise<void> {
    e.preventDefault();
    if (form.title_regex.trim() !== "") {
      try {
        new RegExp(form.title_regex);
        setRegexError(null);
      } catch {
        setRegexError("Invalid regular expression");
        return;
      }
    } else {
      setRegexError(null);
    }
    await onSave({
      name: form.name.trim() || "Untitled app",
      exe_path: form.exe_path.trim(),
      args: form.args,
      cwd: form.cwd.trim(),
      url: form.url.trim(),
      delay_ms: Math.max(0, Math.floor(Number(form.delay_ms) || 0)),
      monitor_id: form.monitor_id,
      x: Math.min(100, Math.max(0, form.x)) / 100,
      y: Math.min(100, Math.max(0, form.y)) / 100,
      w: Math.min(100, Math.max(5, form.w)) / 100,
      h: Math.min(100, Math.max(5, form.h)) / 100,
      state: form.state,
      policy: form.policy,
      match_rules: {
        ...(form.title_contains.trim() ? { title_contains: form.title_contains.trim() } : {}),
        ...(form.title_regex.trim() ? { title_regex: form.title_regex.trim() } : {}),
        ...(form.window_class.trim() ? { window_class: form.window_class.trim() } : {}),
      },
    });
  }

  return (
    <Dialog open onClose={onCancel} title={app.id.startsWith("new-") ? "Add application" : "Edit application"} wide>
      <form onSubmit={submit} className="space-y-3">
        <Input label="Display name" value={form.name} onChange={(e) => set("name", e.target.value)} required />
        <div>
          <Input
            label="Executable path or launch target"
            value={form.exe_path}
            placeholder="C:\Program Files\App\app.exe or protocol URI"
            onChange={(e) => set("exe_path", e.target.value)}
          />
          <div className="mt-1.5 flex gap-2">
            <Button type="button" size="sm" variant="outline" onClick={() => void browse()}>
              Browse…
            </Button>
            {form.exe_path.trim() === "" ? (
              <span role="alert" className="text-xs text-amber-600 dark:text-amber-400">
                No executable set — this entry may fail to launch.
              </span>
            ) : null}
          </div>
        </div>
        <div className="grid gap-3 sm:grid-cols-2">
          <Input label="Arguments" value={form.args} onChange={(e) => set("args", e.target.value)} placeholder="--new-window https://…" />
          <Input label="Working directory" value={form.cwd} onChange={(e) => set("cwd", e.target.value)} placeholder="C:\Projects\demo" />
        </div>
        <div className="grid gap-3 sm:grid-cols-2">
          <Input label="URL / URI (optional)" value={form.url} onChange={(e) => set("url", e.target.value)} placeholder="https://…" />
          <Input
            label="Launch delay (ms)"
            type="number"
            min={0}
            step={100}
            value={form.delay_ms}
            onChange={(e) => set("delay_ms", Number(e.target.value))}
          />
        </div>
        <div className="grid gap-3 sm:grid-cols-3">
          <Select label="Monitor" value={form.monitor_id} onChange={(e) => set("monitor_id", e.target.value)}>
            <option value="">Auto (primary)</option>
            {monitors.map((m) => (
              <option key={m.id} value={m.id}>
                {m.name} ({m.width}×{m.height})
              </option>
            ))}
          </Select>
          <Select label="Window state" value={form.state} onChange={(e) => set("state", e.target.value as WorkspaceApp["state"])}>
            {STATES.map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </Select>
          <Select label="Launch policy" value={form.policy} onChange={(e) => set("policy", e.target.value as WorkspaceApp["policy"])}>
            {POLICIES.map((p) => (
              <option key={p} value={p}>
                {p}
              </option>
            ))}
          </Select>
        </div>
        <fieldset>
          <legend className="mb-1 text-xs font-medium text-slate-600 dark:text-slate-300">
            Position & size (% of monitor work area)
          </legend>
          <div className="grid grid-cols-4 gap-3">
            {(["x", "y", "w", "h"] as const).map((k) => (
              <Input
                key={k}
                label={k.toUpperCase()}
                type="number"
                min={0}
                max={100}
                value={form[k]}
                onChange={(e) => set(k, Number(e.target.value))}
              />
            ))}
          </div>
        </fieldset>
        <fieldset>
          <legend className="mb-1 text-xs font-medium text-slate-600 dark:text-slate-300">
            Window matching rules
          </legend>
          <div className="grid gap-3 sm:grid-cols-3">
            <Input label="Title contains" value={form.title_contains} onChange={(e) => set("title_contains", e.target.value)} />
            <Input label="Window class" value={form.window_class} onChange={(e) => set("window_class", e.target.value)} />
            <Input label="Title regex" value={form.title_regex} onChange={(e) => set("title_regex", e.target.value)} placeholder="^My App.*$" />
          </div>
          {regexError ? (
            <p role="alert" className="mt-1 text-xs text-red-600">
              {regexError}
            </p>
          ) : null}
        </fieldset>
        <div className="flex justify-end gap-2 pt-1">
          <Button type="button" variant="ghost" onClick={onCancel}>
            Cancel
          </Button>
          <Button type="submit" disabled={saving}>
            {saving ? "Saving…" : "Save"}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}

