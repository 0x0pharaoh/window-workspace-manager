import { useEffect, useMemo, useState } from "react";
import { AlertTriangle, FolderOpen, Plus, Search } from "lucide-react";
import type { DiscoveredApp } from "@/types";
import { useWorkspaces } from "@/stores/useWorkspaces";
import { discoverApps, enumWindows, pickExeFile } from "@/services/tauri";
import { Badge, Button, Card, CardContent, Input, Select } from "@/components/ui";

type Tab = "installed" | "running" | "browse" | "custom";

export function ApplicationsPage() {
  const { workspaces, selectedId, selectWorkspace, createApp } = useWorkspaces();
  const [tab, setTab] = useState<Tab>("installed");
  const [query, setQuery] = useState("");
  const [installed, setInstalled] = useState<DiscoveredApp[]>([]);
  const [running, setRunning] = useState<DiscoveredApp[]>([]);
  const [loading, setLoading] = useState(false);
  const [customPath, setCustomPath] = useState("");
  const [customName, setCustomName] = useState("");
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    setLoading(true);
    Promise.all([discoverApps(), enumWindows()])
      .then(([inst, wins]) => {
        if (!alive) return;
        setInstalled(inst);
        setRunning(
          wins.map((w) => ({
            name: w.title || w.exe_path || "Unknown window",
            exe_path: w.exe_path,
            source: "running" as const,
          })),
        );
      })
      .catch(() => undefined)
      .finally(() => {
        if (alive) setLoading(false);
      });
    return () => {
      alive = false;
    };
  }, []);

  const targetWs = workspaces.find((w) => w.id === selectedId) ?? workspaces[0] ?? null;

  const list = useMemo(() => {
    const base = tab === "installed" ? installed : running;
    const q = query.trim().toLowerCase();
    if (!q) return base;
    return base.filter(
      (a) => a.name.toLowerCase().includes(q) || a.exe_path.toLowerCase().includes(q),
    );
  }, [installed, running, tab, query]);

  async function addToWorkspace(app: DiscoveredApp): Promise<void> {
    if (!targetWs) {
      setNotice("Create a workspace first.");
      return;
    }
    if (!app.exe_path) {
      setNotice(`"${app.name}" has no executable path and cannot be added reliably.`);
      return;
    }
    await createApp(targetWs.id, {
      name: app.name,
      exe_path: app.exe_path,
      args: app.args ?? "",
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
    });
    setNotice(`Added "${app.name}" to ${targetWs.name}.`);
  }

  async function handleBrowse(): Promise<void> {
    const p = await pickExeFile();
    if (!p) return;
    const name = p.split(/[\\/]/).pop()?.replace(/\.(exe|bat|cmd|lnk)$/i, "") ?? p;
    await addToWorkspace({ name, exe_path: p, source: "custom" });
  }

  const tabs: { key: Tab; label: string }[] = [
    { key: "installed", label: "Installed" },
    { key: "running", label: "Running" },
    { key: "browse", label: "Browse" },
    { key: "custom", label: "Custom" },
  ];

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h1 className="text-xl font-bold">Applications</h1>
        <Select
          aria-label="Target workspace"
          value={targetWs?.id ?? ""}
          onChange={(e) => selectWorkspace(e.target.value)}
          className="max-w-xs"
        >
          {workspaces.length === 0 ? <option value="">No workspaces</option> : null}
          {workspaces.map((w) => (
            <option key={w.id} value={w.id}>
              Add to: {w.name}
            </option>
          ))}
        </Select>
      </div>

      <div role="tablist" aria-label="Application sources" className="flex gap-1.5">
        {tabs.map((t) => (
          <button
            key={t.key}
            role="tab"
            aria-selected={tab === t.key}
            onClick={() => setTab(t.key)}
            className={
              tab === t.key
                ? "rounded-lg bg-sky-600 px-3 py-1.5 text-sm font-medium text-white"
                : "rounded-lg bg-slate-100 px-3 py-1.5 text-sm text-slate-600 hover:bg-slate-200 dark:bg-slate-800 dark:text-slate-300"
            }
          >
            {t.label}
          </button>
        ))}
      </div>

      {notice ? (
        <p role="status" className="text-sm text-slate-600 dark:text-slate-300">
          {notice}
        </p>
      ) : null}

      {tab === "browse" ? (
        <Card>
          <CardContent className="space-y-3 pt-4">
            <p className="text-sm text-slate-500">Pick an .exe from disk via the system dialog.</p>
            <Button onClick={() => void handleBrowse()}>
              <FolderOpen size={14} aria-hidden /> Browse executable…
            </Button>
          </CardContent>
        </Card>
      ) : null}

      {tab === "custom" ? (
        <Card>
          <CardContent className="space-y-3 pt-4">
            <Input label="Display name" value={customName} onChange={(e) => setCustomName(e.target.value)} placeholder="My tool" />
            <Input
              label="Executable path or URI"
              value={customPath}
              onChange={(e) => setCustomPath(e.target.value)}
              placeholder="C:\Tools\app.exe or ms-settings:"
            />
            <Button
              onClick={() => {
                if (!customPath.trim()) {
                  setNotice("Enter an executable path or URI.");
                  return;
                }
                void addToWorkspace({
                  name: customName.trim() || customPath,
                  exe_path: customPath.trim(),
                  source: "custom",
                });
              }}
            >
              <Plus size={14} aria-hidden /> Add custom app
            </Button>
          </CardContent>
        </Card>
      ) : null}

      {tab === "installed" || tab === "running" ? (
        <>
          <Input
            label="Search applications"
            aria-label="Search applications"
            placeholder="Search name or path…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <p className="flex items-center gap-1 text-xs text-slate-500">
            <Search size={12} aria-hidden /> {loading ? "Loading…" : `${list.length} result${list.length === 1 ? "" : "s"}`}
          </p>
          {list.length === 0 && !loading ? (
            <Card>
              <CardContent className="py-8 text-center text-sm text-slate-500">
                No applications found. {tab === "running" ? "Open some windows first." : "Try Browse instead."}
              </CardContent>
            </Card>
          ) : (
            <ul className="space-y-2">
              {list.slice(0, 100).map((a, i) => (
                <li key={`${a.exe_path}-${i}`}>
                  <Card>
                    <CardContent className="flex items-center justify-between gap-3 pt-4">
                      <div className="min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="truncate text-sm font-medium">{a.name}</span>
                          <Badge tone="slate">{a.source}</Badge>
                          {!a.exe_path ? (
                            <Badge tone="amber">
                              <AlertTriangle size={11} aria-hidden className="mr-1" /> missing exe
                            </Badge>
                          ) : null}
                        </div>
                        <div className="truncate text-xs text-slate-500">{a.exe_path || "unknown target"}</div>
                      </div>
                      <Button size="sm" variant="outline" onClick={() => void addToWorkspace(a)} disabled={!targetWs}>
                        <Plus size={13} aria-hidden /> Add
                      </Button>
                    </CardContent>
                  </Card>
                </li>
              ))}
            </ul>
          )}
        </>
      ) : null}
    </div>
  );
}
