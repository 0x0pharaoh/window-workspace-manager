import { useMemo, useState } from "react";
import { Rnd } from "react-rnd";
import { AlertTriangle, LifeBuoy, Magnet } from "lucide-react";
import type { WorkspaceApp } from "@/types";
import { useWorkspaces } from "@/stores/useWorkspaces";
import { useMonitors } from "@/stores/useMonitors";
import { recoverOffscreen } from "@/services/tauri";
import { PRESETS, PRESET_LABELS, applyPreset, type PresetKind } from "@/lib/layoutPresets";
import {
  clampRect,
  isOffscreen,
  normalizedToPreview,
  previewSizeForMonitor,
  previewToNormalized,
  snapRect,
} from "@/lib/coords";
import { Badge, Button, Card, CardContent, CardHeader, CardTitle, Input, Select, Switch } from "@/components/ui";

const PREVIEW_MAX_W = 720;

export function LayoutEditorPage() {
  const { workspaces, selectedId, selectWorkspace, updateApp } = useWorkspaces();
  const { monitors } = useMonitors();
  const [monitorId, setMonitorId] = useState<string>("");
  const [snap, setSnap] = useState(true);
  const [recovering, setRecovering] = useState(false);
  const [recovered, setRecovered] = useState<number | null>(null);

  const workspace = useMemo(
    () => workspaces.find((w) => w.id === selectedId) ?? workspaces[0] ?? null,
    [workspaces, selectedId],
  );

  const monitor = useMemo(() => {
    if (monitors.length === 0) return null;
    return monitors.find((m) => m.id === monitorId) ?? monitors[0] ?? null;
  }, [monitors, monitorId]);

  const activeMonitorId = monitor?.id ?? "";
  const apps = useMemo(() => {
    if (!workspace) return [];
    if (!activeMonitorId) return workspace.apps;
    const onMon = workspace.apps.filter((a) => (a.monitor_id || "") === activeMonitorId || a.monitor_id === "");
    return onMon.length > 0 ? onMon : workspace.apps;
  }, [workspace, activeMonitorId]);

  const preview = useMemo(() => {
    if (!monitor) return { width: PREVIEW_MAX_W, height: 405 };
    return previewSizeForMonitor(monitor.width, monitor.height, PREVIEW_MAX_W);
  }, [monitor]);

  const offscreen = useMemo(() => (workspace?.apps ?? []).filter((a) => isOffscreen(a)), [workspace]);

  async function persist(app: WorkspaceApp, patch: Partial<WorkspaceApp>): Promise<void> {
    const next = snap
      ? (() => {
          const r = clampRect({
            x: patch.x ?? app.x,
            y: patch.y ?? app.y,
            w: patch.w ?? app.w,
            h: patch.h ?? app.h,
          });
          const s = snapRect(r, 0.05);
          return { ...patch, x: s.x, y: s.y, w: s.w, h: s.h };
        })()
      : patch;
    await updateApp(app.id, next);
  }

  async function applyPresetToWorkspace(preset: PresetKind): Promise<void> {
    if (!workspace) return;
    const updated = applyPreset(workspace.apps, preset);
    for (const a of updated) {
      await updateApp(a.id, { x: a.x, y: a.y, w: a.w, h: a.h });
    }
  }

  async function handleRecover(): Promise<void> {
    setRecovering(true);
    try {
      const n = await recoverOffscreen();
      setRecovered(n);
      if (workspace && offscreen.length > 0) {
        for (const [i, a] of offscreen.entries()) {
          const cols = Math.ceil(Math.sqrt(offscreen.length));
          const rows = Math.ceil(offscreen.length / cols);
          await updateApp(a.id, {
            x: (i % cols) * (1 / cols),
            y: Math.floor(i / cols) * (1 / rows),
            w: 1 / cols,
            h: 1 / rows,
          });
        }
      }
    } finally {
      setRecovering(false);
    }
  }

  if (!workspace) {
    return (
      <Card>
        <CardContent className="py-8 text-center text-sm text-slate-500">
          No workspace selected. Create one in Workspaces first.
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h1 className="text-xl font-bold">Layout Editor</h1>
        <div className="flex items-center gap-3">
          <label className="flex items-center gap-1.5 text-sm">
            <Magnet size={14} aria-hidden />
            <Switch checked={snap} onCheckedChange={setSnap} label="Snap to grid" />
            <span>Snap</span>
          </label>
        </div>
      </div>

      <div className="grid gap-3 sm:grid-cols-3">
        <Select label="Workspace" value={workspace.id} onChange={(e) => selectWorkspace(e.target.value)}>
          {workspaces.map((w) => (
            <option key={w.id} value={w.id}>
              {w.name}
            </option>
          ))}
        </Select>
        <Select
          label="Monitor"
          value={activeMonitorId}
          onChange={(e) => setMonitorId(e.target.value)}
        >
          {monitors.length === 0 ? <option value="">No monitors detected</option> : null}
          {monitors.map((m) => (
            <option key={m.id} value={m.id}>
              {m.name} — {m.width}×{m.height}{m.is_primary ? " (primary)" : ""}
            </option>
          ))}
        </Select>
        <div className="flex items-end">
          <Badge tone={monitor ? "green" : "amber"}>
            {monitor
              ? `${monitor.width}×${monitor.height} @${monitor.scale_factor}x`
              : "backend offline — preview only"}
          </Badge>
        </div>
      </div>

      {offscreen.length > 0 ? (
        <Card>
          <CardContent className="flex flex-wrap items-center gap-2 pt-4 text-sm">
            <AlertTriangle size={16} className="text-amber-500" aria-hidden />
            <span role="alert">
              {offscreen.length} app{offscreen.length === 1 ? " is" : "s are"} off-screen and may be
              invisible after launch.
            </span>
            <Button size="sm" variant="outline" onClick={() => void handleRecover()} disabled={recovering}>
              <LifeBuoy size={13} aria-hidden /> {recovering ? "Recovering…" : "Recover off-screen"}
            </Button>
            {recovered !== null ? <Badge tone="green">Fixed {recovered}</Badge> : null}
          </CardContent>
        </Card>
      ) : null}

      <div className="flex flex-wrap gap-1.5" role="group" aria-label="Layout presets">
        {PRESETS.map((p) => (
          <Button
            key={p}
            size="sm"
            variant="outline"
            onClick={() => void applyPresetToWorkspace(p)}
            aria-label={`Apply ${PRESET_LABELS[p]} preset`}
          >
            {PRESET_LABELS[p]}
          </Button>
        ))}
      </div>

      <Card>
        <CardHeader>
          <CardTitle>
            Preview — {monitor?.name ?? "monitor"} ({preview.width}×{preview.height}px)
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div
            data-testid="layout-preview"
            className="relative overflow-hidden rounded-lg border border-slate-300 bg-slate-100 dark:border-slate-700 dark:bg-slate-950"
            style={{ width: preview.width, height: preview.height, maxWidth: "100%" }}
          >
            {apps.length === 0 ? (
              <p className="p-6 text-center text-sm text-slate-500">
                No apps on this monitor. Add apps in Workspaces.
              </p>
            ) : null}
            {apps.map((app) => {
              const px = normalizedToPreview(app, preview.width, preview.height);
              return (
                <Rnd
                  key={app.id}
                  data-testid={`layout-card-${app.id}`}
                  size={{ width: Math.max(60, px.width), height: Math.max(40, px.height) }}
                  position={{ x: px.x, y: px.y }}
                  bounds="parent"
                  onDragStop={(_e, d) => {
                    const r = previewToNormalized(
                      { x: d.x, y: d.y, width: px.width, height: px.height },
                      preview.width,
                      preview.height,
                    );
                    void persist(app, { x: r.x, y: r.y });
                  }}
                  onResizeStop={(_e, _dir, ref, _delta, pos) => {
                    const r = previewToNormalized(
                      { x: pos.x, y: pos.y, width: ref.offsetWidth, height: ref.offsetHeight },
                      preview.width,
                      preview.height,
                    );
                    void persist(app, r);
                  }}
                  className="flex flex-col rounded-md border-2 bg-white/95 text-xs shadow dark:bg-slate-800/95"
                  style={{ borderColor: workspace.color || "#0078d4", zIndex: 1 }}
                >
                  <div className="truncate bg-slate-200/70 px-1.5 py-0.5 font-semibold dark:bg-slate-700/70">
                    {app.name}
                  </div>
                  <div className="px-1.5 text-[10px] text-slate-500">
                    {Math.round(app.x * 100)},{Math.round(app.y * 100)} · {Math.round(app.w * 100)}×
                    {Math.round(app.h * 100)}%
                  </div>
                </Rnd>
              );
            })}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Manual coordinates (% of work area)</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          {apps.map((app) => (
            <div key={app.id} className="grid grid-cols-2 items-end gap-2 sm:grid-cols-6">
              <span className="truncate text-sm font-medium sm:col-span-2">{app.name}</span>
              {(["x", "y", "w", "h"] as const).map((k) => (
                <Input
                  key={k}
                  label={k.toUpperCase()}
                  type="number"
                  min={0}
                  max={100}
                  aria-label={`${app.name} ${k} percent`}
                  value={Math.round(app[k] * 100)}
                  onChange={(e) => {
                    const v = Math.min(100, Math.max(0, Number(e.target.value) || 0)) / 100;
                    void persist(app, { [k]: v } as Partial<WorkspaceApp>);
                  }}
                />
              ))}
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  );
}
