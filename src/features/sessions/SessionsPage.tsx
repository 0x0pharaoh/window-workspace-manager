import { useEffect, useState } from "react";
import { LifeBuoy, RotateCcw, ScrollText, Square, X } from "lucide-react";
import { useSessions } from "@/stores/useSessions";
import { useWorkspaces } from "@/stores/useWorkspaces";
import { getLogs, recoverOffscreen } from "@/services/tauri";
import { Badge, Button, Card, CardContent, CardHeader, CardTitle } from "@/components/ui";

export function SessionsPage() {
  const {
    sessions,
    progress,
    fetchSessions,
    launchWorkspace,
    closeSessionApp,
    closeWorkspaceSession,
  } = useSessions();
  const { workspaces } = useWorkspaces();
  const [logs, setLogs] = useState<string[]>([]);
  const [logsOpen, setLogsOpen] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);

  useEffect(() => {
    void fetchSessions();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function loadLogs(): Promise<void> {
    setLogsOpen((v) => !v);
    if (!logsOpen && logs.length === 0) {
      setLogs(await getLogs(200));
    }
  }

  async function restart(workspaceId: string): Promise<void> {
    setBusy(workspaceId);
    try {
      await launchWorkspace(workspaceId);
    } finally {
      setBusy(null);
    }
  }

  async function reposition(): Promise<void> {
    await recoverOffscreen();
    await fetchSessions();
  }

  if (sessions.length === 0) {
    return (
      <div className="space-y-4">
        <h1 className="text-xl font-bold">Active Sessions</h1>
        <Card>
          <CardContent className="py-8 text-center text-sm text-slate-500">
            No active sessions. Launch a workspace to see tracked windows here.
          </CardContent>
        </Card>
        <Button variant="outline" size="sm" onClick={() => void loadLogs()}>
          <ScrollText size={14} aria-hidden /> {logsOpen ? "Hide logs" : "View logs"}
        </Button>
        {logsOpen ? (
          <Card>
            <CardContent className="max-h-64 overflow-auto pt-4 font-mono text-xs">
              {logs.length === 0 ? "No logs." : logs.map((l, i) => <div key={i}>{l}</div>)}
            </CardContent>
          </Card>
        ) : null}
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h1 className="text-xl font-bold">Active Sessions ({sessions.length})</h1>
        <div className="flex gap-2">
          <Button size="sm" variant="outline" onClick={() => void reposition()}>
            <LifeBuoy size={13} aria-hidden /> Reposition all
          </Button>
          <Button size="sm" variant="outline" onClick={() => void loadLogs()}>
            <ScrollText size={13} aria-hidden /> {logsOpen ? "Hide logs" : "Logs"}
          </Button>
        </div>
      </div>

      {logsOpen ? (
        <Card>
          <CardContent className="max-h-64 overflow-auto pt-4 font-mono text-xs">
            {logs.length === 0 ? "No logs." : logs.map((l, i) => <div key={i}>{l}</div>)}
          </CardContent>
        </Card>
      ) : null}

      <div className="space-y-3">
        {sessions.map((s) => {
          const ws = workspaces.find((w) => w.id === s.workspace_id);
          const progItems = Object.values(progress).filter((p) => p.workspace_id === s.workspace_id);
          return (
            <Card key={s.id}>
              <CardHeader>
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <CardTitle>
                    {s.workspace_name}{" "}
                    <span className="ml-2">
                      <Badge tone={s.status === "running" ? "green" : "slate"}>{s.status}</Badge>
                    </span>
                  </CardTitle>
                  <div className="flex gap-1.5">
                    <Button
                      size="sm"
                      variant="outline"
                      disabled={busy === s.workspace_id}
                      onClick={() => void restart(s.workspace_id)}
                    >
                      <RotateCcw size={13} aria-hidden /> Restart
                    </Button>
                    <Button size="sm" variant="destructive" onClick={() => void closeWorkspaceSession(s.id)}>
                      <Square size={13} aria-hidden /> Close all
                    </Button>
                  </div>
                </div>
                <p className="text-xs text-slate-500">Started {new Date(s.started_at).toLocaleString()}</p>
              </CardHeader>
              <CardContent className="space-y-1.5">
                {progItems.length > 0 ? (
                  <div className="space-y-1" aria-label="Launch progress">
                    {progItems.map((p) => (
                      <div key={p.app_id} className="flex items-center gap-2 text-xs">
                        <span className="w-40 truncate">{p.app_name}</span>
                        <div
                          role="progressbar"
                          aria-valuenow={Math.round(p.progress * 100)}
                          aria-valuemin={0}
                          aria-valuemax={100}
                          aria-label={`${p.app_name} progress`}
                          className="h-1.5 flex-1 overflow-hidden rounded bg-slate-200 dark:bg-slate-700"
                        >
                          <div
                            className="h-full bg-sky-600"
                            style={{ width: `${Math.round(p.progress * 100)}%` }}
                          />
                        </div>
                        <Badge
                          tone={
                            p.status === "done" ? "green" : p.status === "failed" ? "red" : "blue"
                          }
                        >
                          {p.status}
                        </Badge>
                      </div>
                    ))}
                  </div>
                ) : null}
                {s.windows.length === 0 ? (
                  <p className="text-sm text-slate-500">No tracked windows.</p>
                ) : (
                  <ul className="divide-y divide-slate-100 dark:divide-slate-800">
                    {s.windows.map((w) => (
                      <li key={w.id} className="flex items-center justify-between gap-2 py-2">
                        <div className="min-w-0">
                          <div className="truncate text-sm font-medium">{w.app_name}</div>
                          <div className="text-xs text-slate-500">
                            {w.status}
                            {w.message ? ` — ${w.message}` : ""}
                            {w.pid ? ` · pid ${w.pid}` : ""}
                          </div>
                        </div>
                        <div className="flex gap-1.5">
                          {ws ? (
                            <Button size="sm" variant="outline" onClick={() => void restart(s.workspace_id)}>
                              Reposition
                            </Button>
                          ) : null}
                          <Button
                            size="sm"
                            variant="ghost"
                            aria-label={`Close ${w.app_name}`}
                            onClick={() => void closeSessionApp(s.id, w.app_id)}
                          >
                            <X size={14} aria-hidden />
                          </Button>
                        </div>
                      </li>
                    ))}
                  </ul>
                )}
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
