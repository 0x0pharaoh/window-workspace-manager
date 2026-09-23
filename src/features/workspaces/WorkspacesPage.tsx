import { useMemo, useRef, useState } from "react";
import {
  Copy,
  Download,
  Heart,
  Monitor,
  Pencil,
  Play,
  Plus,
  Square,
  Trash2,
  Upload,
} from "lucide-react";
import type { Workspace, WorkspaceApp } from "@/types";
import { useWorkspaces } from "@/stores/useWorkspaces";
import { useMonitors } from "@/stores/useMonitors";
import { useSessions } from "@/stores/useSessions";
import { exportWorkspace, saveFile, serializeWorkspace } from "@/services/tauri";
import { AppForm } from "@/features/workspaces/AppForm";
import { Badge, Button, Card, CardContent, CardHeader, CardTitle, Dialog, Input } from "@/components/ui";

function blankWorkspaceApp(workspaceId: string, sortOrder: number): WorkspaceApp {
  return {
    id: `new-${Date.now()}-${sortOrder}`,
    workspace_id: workspaceId,
    name: "",
    exe_path: "",
    args: "",
    cwd: "",
    url: "",
    delay_ms: 0,
    monitor_id: "",
    x: 0,
    y: 0,
    w: 0.5,
    h: 0.5,
    state: "normal",
    policy: "if_not_running",
    match_rules: {},
    sort_order: sortOrder,
  };
}

export function WorkspacesPage() {
  const {
    workspaces,
    selectedId,
    selectWorkspace,
    createWorkspace,
    renameWorkspace,
    duplicateWorkspace,
    deleteWorkspace,
    toggleFavorite,
    updateWorkspaceMeta,
    createApp,
    updateApp,
    deleteApp,
    importWorkspace,
  } = useWorkspaces();
  const { monitors } = useMonitors();
  const { launchWorkspace, closeWorkspaceSession, sessions } = useSessions();

  const [newName, setNewName] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [confirmDelete, setConfirmDelete] = useState<Workspace | null>(null);
  const [formApp, setFormApp] = useState<WorkspaceApp | null>(null);
  const [savingApp, setSavingApp] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);
  const [importError, setImportError] = useState<string | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  const selected = useMemo(
    () => workspaces.find((w) => w.id === selectedId) ?? workspaces[0] ?? null,
    [workspaces, selectedId],
  );

  async function handleCreate(e: React.FormEvent): Promise<void> {
    e.preventDefault();
    await createWorkspace(newName.trim() || "Untitled workspace");
    setNewName("");
  }

  async function handleRename(id: string): Promise<void> {
    await renameWorkspace(id, editName);
    setEditingId(null);
  }

  async function handleLaunch(id: string): Promise<void> {
    setBusy(id);
    try {
      await launchWorkspace(id);
    } finally {
      setBusy(null);
    }
  }

  async function handleExport(ws: Workspace): Promise<void> {
    try {
      const json = await exportWorkspace(ws.id);
      let out = json;
      try {
        const parsed = JSON.parse(json) as { workspace?: Workspace };
        if (!parsed.workspace) out = serializeWorkspace(ws);
      } catch {
        out = serializeWorkspace(ws);
      }
      await saveFile(`${ws.name.replace(/[^\w\-]+/g, "_")}.workset.json`, out);
    } catch (err) {
      console.warn("export failed", err);
    }
  }

  async function handleImportFile(file: File): Promise<void> {
    setImportError(null);
    try {
      const text = await file.text();
      try {
        await importWorkspace(text);
      } catch (err) {
        // The backend rejects imports with shell-like tokens unless the user
        // explicitly confirms. Offer the confirmation and retry once.
        const msg = err instanceof Error ? err.message : "Import failed";
        if (!/explicit confirmation/i.test(msg)) throw err;
        const ok = window.confirm(
          "This workspace file contains shell-like characters in an executable path or arguments.\n\n" +
            "Import it anyway? Only proceed with files from sources you trust.",
        );
        if (!ok) return;
        await importWorkspace(text, true);
      }
    } catch (err) {
      setImportError(err instanceof Error ? err.message : "Import failed");
    }
  }

  async function handleSaveApp(patch: Partial<WorkspaceApp>): Promise<void> {
    if (!formApp || !selected) return;
    setSavingApp(true);
    try {
      if (formApp.id.startsWith("new-")) {
        await createApp(selected.id, {
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
        });
      } else {
        await updateApp(formApp.id, patch);
      }
      setFormApp(null);
    } finally {
      setSavingApp(false);
    }
  }

  const sessionForSelected = sessions.find((s) => s.workspace_id === selected?.id);

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h1 className="text-xl font-bold">Workspaces</h1>
        <div className="flex gap-2">
          <input
            ref={fileRef}
            type="file"
            accept=".json,application/json"
            className="hidden"
            aria-label="Import workspace file"
            onChange={(e) => {
              const f = e.target.files?.[0];
              if (f) void handleImportFile(f);
              e.target.value = "";
            }}
          />
          <Button variant="outline" size="sm" onClick={() => fileRef.current?.click()}>
            <Upload size={14} aria-hidden /> Import
          </Button>
        </div>
      </div>
      {importError ? (
        <p role="alert" className="text-sm text-red-600">
          Import failed: {importError}
        </p>
      ) : null}

      <form onSubmit={handleCreate} className="flex gap-2">
        <Input
          aria-label="New workspace name"
          placeholder="New workspace name…"
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
        />
        <Button type="submit">
          <Plus size={15} aria-hidden /> Create
        </Button>
      </form>

      {workspaces.length === 0 ? (
        <Card>
          <CardContent className="py-8 text-center text-sm text-slate-500 dark:text-slate-400">
            No workspaces yet. Create one above to get started.
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          {workspaces.map((w) => (
            <Card key={w.id} className={w.id === selected?.id ? "ring-2 ring-sky-500" : undefined}>
              <CardHeader>
                <div className="flex items-start justify-between gap-2">
                  <div className="min-w-0">
                    {editingId === w.id ? (
                      <span className="flex gap-1.5">
                        <Input
                          aria-label="Workspace name"
                          value={editName}
                          onChange={(e) => setEditName(e.target.value)}
                        />
                        <Button size="sm" onClick={() => void handleRename(w.id)}>
                          Save
                        </Button>
                      </span>
                    ) : (
                      <CardTitle>{w.name}</CardTitle>
                    )}
                    <div className="mt-1 flex flex-wrap gap-1">
                      <Badge tone="slate">{w.apps.length} apps</Badge>
                      {w.hotkey ? <Badge tone="blue">{w.hotkey}</Badge> : null}
                    </div>
                  </div>
                  <button
                    type="button"
                    aria-label={w.favorite ? `Unfavorite ${w.name}` : `Favorite ${w.name}`}
                    aria-pressed={w.favorite}
                    onClick={() => void toggleFavorite(w.id)}
                    className="rounded-md p-1.5 hover:bg-slate-100 dark:hover:bg-slate-800"
                  >
                    <Heart
                      size={16}
                      aria-hidden
                      className={w.favorite ? "fill-amber-400 text-amber-400" : "text-slate-400"}
                    />
                  </button>
                </div>
              </CardHeader>
              <CardContent className="flex flex-wrap gap-1.5">
                <Button size="sm" onClick={() => selectWorkspace(w.id)} variant="outline">
                  Open
                </Button>
                <Button size="sm" onClick={() => void handleLaunch(w.id)} disabled={busy === w.id}>
                  <Play size={13} aria-hidden /> {busy === w.id ? "…" : "Launch"}
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  aria-label={`Rename ${w.name}`}
                  onClick={() => {
                    setEditingId(w.id);
                    setEditName(w.name);
                  }}
                >
                  <Pencil size={13} aria-hidden />
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  aria-label={`Duplicate ${w.name}`}
                  onClick={() => void duplicateWorkspace(w.id)}
                >
                  <Copy size={13} aria-hidden />
                </Button>
                <Button size="sm" variant="ghost" aria-label={`Export ${w.name}`} onClick={() => void handleExport(w)}>
                  <Download size={13} aria-hidden />
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  aria-label={`Delete ${w.name}`}
                  onClick={() => setConfirmDelete(w)}
                >
                  <Trash2 size={13} aria-hidden />
                </Button>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {selected ? (
        <Card>
          <CardHeader>
            <CardTitle>
              {selected.name} — applications
              <span className="ml-2 align-middle">
                <Input
                  aria-label="Workspace hotkey"
                  placeholder="Hotkey (e.g. Ctrl+Alt+1)"
                  value={selected.hotkey}
                  onChange={(e) => void updateWorkspaceMeta(selected.id, { hotkey: e.target.value })}
                  className="mt-2 max-w-xs"
                />
              </span>
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <div className="flex flex-wrap gap-2">
              <Button size="sm" onClick={() => void handleLaunch(selected.id)}>
                <Play size={13} aria-hidden /> Launch
              </Button>
              {sessionForSelected ? (
                <Button size="sm" variant="outline" onClick={() => void closeWorkspaceSession(sessionForSelected.id)}>
                  <Square size={13} aria-hidden /> Close session
                </Button>
              ) : null}
              <Button size="sm" variant="outline" onClick={() => setFormApp(blankWorkspaceApp(selected.id, selected.apps.length))}>
                <Plus size={13} aria-hidden /> Add app
              </Button>
            </div>
            {selected.apps.length === 0 ? (
              <p className="py-4 text-center text-sm text-slate-500 dark:text-slate-400">
                No applications in this workspace yet.
              </p>
            ) : (
              <ul className="divide-y divide-slate-100 dark:divide-slate-800">
                {selected.apps.map((a) => {
                  const mon = monitors.find((m) => m.id === a.monitor_id);
                  return (
                    <li key={a.id} className="flex flex-wrap items-center justify-between gap-2 py-2.5">
                      <div className="min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="truncate text-sm font-medium">{a.name}</span>
                          {a.exe_path.trim() === "" ? (
                            <Badge tone="amber">missing exe</Badge>
                          ) : null}
                        </div>
                        <div className="truncate text-xs text-slate-500 dark:text-slate-400">
                          {a.exe_path || "no target"} · {Math.round(a.x * 100)},{Math.round(a.y * 100)} ·{" "}
                          {Math.round(a.w * 100)}×{Math.round(a.h * 100)}% · {a.state}
                        </div>
                        <div className="mt-1 flex gap-1">
                          <Badge tone="blue">
                            <Monitor size={11} aria-hidden className="mr-1" />
                            {mon ? mon.name : "Auto"}
                          </Badge>
                          <Badge tone="slate">{a.policy}</Badge>
                        </div>
                      </div>
                      <div className="flex gap-1.5">
                        <Button size="sm" variant="outline" onClick={() => setFormApp({ ...a })}>
                          Edit
                        </Button>
                        <Button size="sm" variant="ghost" onClick={() => void deleteApp(a.id)} aria-label={`Remove ${a.name}`}>
                          <Trash2 size={13} aria-hidden />
                        </Button>
                      </div>
                    </li>
                  );
                })}
              </ul>
            )}
          </CardContent>
        </Card>
      ) : null}

      {formApp ? (
        <AppForm
          app={formApp}
          saving={savingApp}
          onCancel={() => setFormApp(null)}
          onSave={handleSaveApp}
        />
      ) : null}

      <Dialog open={confirmDelete !== null} onClose={() => setConfirmDelete(null)} title="Delete workspace">
        <p className="text-sm">
          Delete <strong>{confirmDelete?.name}</strong>? This removes its {confirmDelete?.apps.length ?? 0}{" "}
          app entries. This cannot be undone.
        </p>
        <div className="mt-4 flex justify-end gap-2">
          <Button variant="ghost" onClick={() => setConfirmDelete(null)}>
            Cancel
          </Button>
          <Button
            variant="destructive"
            onClick={() => {
              if (confirmDelete) void deleteWorkspace(confirmDelete.id);
              setConfirmDelete(null);
            }}
          >
            Delete
          </Button>
        </div>
      </Dialog>
    </div>
  );
}
