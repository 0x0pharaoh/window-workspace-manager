import { useEffect, useState } from "react";
import type { CapturedWindow } from "@/types";
import { useWorkspaces } from "@/stores/useWorkspaces";
import { enumWindows } from "@/services/tauri";
import { Badge, Button, Card, Dialog, Input, Select } from "@/components/ui";

interface EditableRow extends CapturedWindow {
  selected: boolean;
  customName: string;
}

export function CaptureDialog({
  open,
  onClose,
  onSaved,
}: {
  open: boolean;
  onClose: () => void;
  onSaved?: () => void;
}) {
  const { workspaces, createWorkspace, createApp } = useWorkspaces();
  const [rows, setRows] = useState<EditableRow[]>([]);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [target, setTarget] = useState<string>("__new__");
  const [newName, setNewName] = useState("Captured workspace");

  useEffect(() => {
    if (!open) return;
    setLoading(true);
    enumWindows()
      .then((wins) => {
        const filtered = wins.filter((w) => (w.title || "").trim() !== "");
        setRows(
          filtered.map((w) => ({
            ...w,
            selected: true,
            customName: w.title || "Untitled",
          })),
        );
      })
      .catch(() => setRows([]))
      .finally(() => setLoading(false));
  }, [open ]);

  function toggle(i: number): void {
    setRows((r) => r.map((row, idx) => (idx === i ? { ...row, selected: !row.selected } : row)));
  }

  function edit(i: number, name: string): void {
    setRows((r) => r.map((row, idx) => (idx === i ? { ...row, customName: name } : row)));
  }

  async function save(): Promise<void> {
    const chosen = rows.filter((r) => r.selected);
    if (chosen.length === 0) return;
    setSaving(true);
    try {
      let wsId = target;
      if (target === "__new__") {
        const ws = await createWorkspace(newName.trim() || "Captured workspace");
        wsId = ws.id;
      }
      for (const [i, w] of chosen.entries()) {
        await createApp(wsId, {
          name: w.customName.trim() || w.title || "Captured app",
          exe_path: w.exe_path,
          args: "",
          cwd: "",
          url: "",
          delay_ms: i * 500,
          monitor_id: w.monitor_id,
          x: 0.05,
          y: 0.05,
          w: 0.6,
          h: 0.6,
          state: w.state ?? "normal",
          policy: "if_not_running",
          match_rules: {
            ...(w.title ? { title_contains: w.title.slice(0, 80) } : {}),
            ...(w.window_class ? { window_class: w.window_class } : {}),
          },
        });
      }
      onSaved?.();
      onClose();
    } finally {
      setSaving(false);
    }
  }

  const selectedCount = rows.filter((r) => r.selected).length;

  return (
    <Dialog open={open} onClose={onClose} title="Capture current desktop" wide>
      {loading ? (
        <p className="py-6 text-center text-sm text-slate-500">Enumerating windows…</p>
      ) : rows.length === 0 ? (
        <Card>
          <div className="p-6 text-center text-sm text-slate-500">
            No capturable windows found. Open some applications first.
          </div>
        </Card>
      ) : (
        <div className="space-y-3">
          <div className="grid gap-3 sm:grid-cols-2">
            <Select label="Save into" value={target} onChange={(e) => setTarget(e.target.value)}>
              <option value="__new__">New workspace</option>
              {workspaces.map((w) => (
                <option key={w.id} value={w.id}>
                  {w.name}
                </option>
              ))}
            </Select>
            {target === "__new__" ? (
              <Input label="New workspace name" value={newName} onChange={(e) => setNewName(e.target.value)} />
            ) : null}
          </div>
          <ul className="max-h-80 space-y-2 overflow-auto">
            {rows.map((r, i) => (
              <li key={`${r.hwnd ?? i}-${r.title}`}>
                <Card>
                  <div className="flex items-start gap-2 p-3">
                    <input
                      type="checkbox"
                      aria-label={`Include ${r.title}`}
                      checked={r.selected}
                      onChange={() => toggle(i)}
                      className="mt-1 h-4 w-4 accent-sky-600"
                    />
                    <div className="min-w-0 flex-1">
                      <Input
                        aria-label={`Name for ${r.title}`}
                        value={r.customName}
                        onChange={(e) => edit(i, e.target.value)}
                      />
                      <div className="mt-1 truncate text-xs text-slate-500">
                        {r.exe_path || "unknown exe"} · class {r.window_class || "?"}
                      </div>
                      <div className="mt-1">
                        {!r.exe_path ? <Badge tone="amber">unreliable — no exe path</Badge> : null}
                      </div>
                    </div>
                  </div>
                </Card>
              </li>
            ))}
          </ul>
          <div className="flex items-center justify-between">
            <span className="text-sm text-slate-500">
              {selectedCount} of {rows.length} selected
            </span>
            <div className="flex gap-2">
              <Button variant="ghost" onClick={onClose}>
                Cancel
              </Button>
              <Button onClick={() => void save()} disabled={saving || selectedCount === 0}>
                {saving ? "Saving…" : `Save ${selectedCount} window${selectedCount === 1 ? "" : "s"}`}
              </Button>
            </div>
          </div>
        </div>
      )}
    </Dialog>
  );
}
