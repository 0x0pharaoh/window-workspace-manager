import { useMemo, useState } from "react";
import { Camera, Heart, Play, Plus, Rocket, Star } from "lucide-react";
import type { ViewKey } from "@/types";
import { useWorkspaces } from "@/stores/useWorkspaces";
import { useSessions } from "@/stores/useSessions";
import { Badge, Button, Card, CardContent, CardHeader, CardTitle } from "@/components/ui";

export function HomePage({
  onNavigate,
  onOpenCapture,
}: {
  onNavigate: (v: ViewKey) => void;
  onOpenCapture: () => void;
}) {
  const { workspaces, createWorkspace } = useWorkspaces();
  const { sessions, launchWorkspace } = useSessions();
  const [busyId, setBusyId] = useState<string | null>(null);

  const favorites = useMemo(() => workspaces.filter((w) => w.favorite).slice(0, 6), [workspaces]);
  const recent = useMemo(
    () =>
      [...workspaces]
        .sort((a, b) =>
          (b.last_launched_at ?? b.updated_at).localeCompare(a.last_launched_at ?? a.updated_at),
        )
        .slice(0, 4),
    [workspaces],
  );

  async function quickLaunch(id: string): Promise<void> {
    setBusyId(id);
    try {
      await launchWorkspace(id);
    } finally {
      setBusyId(null);
    }
  }

  async function handleCreate(): Promise<void> {
    await createWorkspace("Untitled workspace");
    onNavigate("workspaces");
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-bold">Good day — launch your workspace</h1>
          <p className="text-sm text-slate-500 dark:text-slate-400">
            {workspaces.length === 0
              ? "Create your first workspace or capture your desktop."
              : `${workspaces.length} workspace${workspaces.length === 1 ? "" : "s"} · ${sessions.length} active session${sessions.length === 1 ? "" : "s"}`}
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={onOpenCapture} aria-label="Capture current desktop">
            <Camera size={16} aria-hidden /> Capture
          </Button>
          <Button onClick={handleCreate} aria-label="Create workspace">
            <Plus size={16} aria-hidden /> New workspace
          </Button>
        </div>
      </div>

      {workspaces.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center gap-3 py-10 text-center">
            <Rocket size={32} className="text-sky-600" aria-hidden />
            <p className="font-medium">No workspaces yet</p>
            <p className="max-w-sm text-sm text-slate-500 dark:text-slate-400">
              Create a workspace to group apps together, or capture the windows you already have open.
            </p>
            <div className="flex gap-2">
              <Button onClick={handleCreate}>Create workspace</Button>
              <Button variant="outline" onClick={onOpenCapture}>
                Capture desktop
              </Button>
            </div>
          </CardContent>
        </Card>
      ) : null}

      <section aria-label="Quick launch">
        <h2 className="mb-2 flex items-center gap-1.5 text-sm font-semibold">
          <Play size={14} aria-hidden /> Quick launch
        </h2>
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          {recent.map((w) => (
            <Card key={w.id}>
              <CardHeader>
                <CardTitle>{w.name}</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <div className="flex gap-1.5">
                  <Badge tone="slate">{w.apps.length} apps</Badge>
                  {w.favorite ? <Badge tone="amber">Favorite</Badge> : null}
                </div>
                <Button
                  size="sm"
                  className="w-full"
                  disabled={busyId === w.id}
                  onClick={() => void quickLaunch(w.id)}
                  aria-label={`Launch ${w.name}`}
                >
                  {busyId === w.id ? "Launching…" : "Launch"}
                </Button>
              </CardContent>
            </Card>
          ))}
        </div>
      </section>

      <div className="grid gap-4 lg:grid-cols-2">
        <section aria-label="Favorites">
          <h2 className="mb-2 flex items-center gap-1.5 text-sm font-semibold">
            <Star size={14} aria-hidden /> Favorites
          </h2>
          {favorites.length === 0 ? (
            <Card>
              <CardContent className="py-6 text-center text-sm text-slate-500 dark:text-slate-400">
                <Heart size={18} className="mx-auto mb-2" aria-hidden />
                No favorites yet. Star a workspace to pin it here.
              </CardContent>
            </Card>
          ) : (
            <div className="space-y-2">
              {favorites.map((w) => (
                <Card key={w.id}>
                  <CardContent className="flex items-center justify-between gap-2 pt-4">
                    <div className="min-w-0">
                      <div className="truncate text-sm font-medium">{w.name}</div>
                      <div className="text-xs text-slate-500">{w.apps.length} apps</div>
                    </div>
                    <Button size="sm" variant="outline" onClick={() => void quickLaunch(w.id)}>
                      Launch
                    </Button>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </section>

        <section aria-label="Active sessions">
          <h2 className="mb-2 text-sm font-semibold">Active sessions</h2>
          {sessions.length === 0 ? (
            <Card>
              <CardContent className="py-6 text-center text-sm text-slate-500 dark:text-slate-400">
                No active sessions. Launch a workspace to track its windows here.
              </CardContent>
            </Card>
          ) : (
            <div className="space-y-2">
              {sessions.slice(0, 5).map((s) => (
                <Card key={s.id}>
                  <CardContent className="flex items-center justify-between gap-2 pt-4">
                    <div className="min-w-0">
                      <div className="truncate text-sm font-medium">{s.workspace_name}</div>
                      <div className="text-xs text-slate-500">
                        {s.windows.length} windows · {s.status}
                      </div>
                    </div>
                    <Button size="sm" variant="outline" onClick={() => onNavigate("sessions")}>
                      View
                    </Button>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
          <Button variant="ghost" size="sm" className="mt-2" onClick={() => onNavigate("workspaces")}>
            Manage workspaces →
          </Button>
        </section>
      </div>
    </div>
  );
}
