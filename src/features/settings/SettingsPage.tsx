import { useEffect, useRef, useState } from "react";
import { Download, LifeBuoy, Plus, Trash2, Upload } from "lucide-react";
import { useSettings } from "@/stores/useSettings";
import { useWorkspaces } from "@/stores/useWorkspaces";
import { getLogs, recoverOffscreen, saveFile, serializeWorkspace } from "@/services/tauri";
import { Badge, Button, Card, CardContent, CardHeader, CardTitle, Input, Select, Slider, Switch } from "@/components/ui";

export function SettingsPage() {
  const { settings, load, updateSetting } = useSettings();
  const { workspaces, importWorkspace } = useWorkspaces();
  const [logs, setLogs] = useState<string[]>([]);
  const [hotKey, setHotKey] = useState("");
  const [hotValue, setHotValue] = useState("");
  const [recovered, setRecovered] = useState<number | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    void load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function handleExportAll(): Promise<void> {
    const payload = JSON.stringify(
      { version: 1, exported_at: new Date().toISOString(), workspaces },
      null,
      2,
    );
    await saveFile("workset-backup.json", payload);
  }

  async function handleImportFile(file: File): Promise<void> {
    const text = await file.text();
    try {
      const parsed = JSON.parse(text) as { workspaces?: unknown[]; workspace?: unknown };
      if (Array.isArray(parsed.workspaces)) {
        for (const w of parsed.workspaces) {
          await importWorkspace(JSON.stringify({ version: 1, workspace: w }));
        }
      } else {
        await importWorkspace(text);
      }
    } catch (err) {
      console.warn("import failed", err);
    }
  }

  if (!settings) {
    return <p className="text-sm text-slate-500">Loading settings…</p>;
  }

  const hotEntries = Object.entries(settings.hotkeys);

  return (
    <div className="space-y-4">
      <h1 className="text-xl font-bold">Settings</h1>

      <Card>
        <CardHeader>
          <CardTitle>Appearance & launch</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <Select
            label="Theme"
            value={settings.theme}
            onChange={(e) => void updateSetting("theme", e.target.value as typeof settings.theme)}
          >
            <option value="light">Light</option>
            <option value="dark">Dark</option>
            <option value="system">System</option>
          </Select>
          <Slider
            label="Default launch timeout (ms)"
            value={settings.default_timeout_ms}
            min={2000}
            max={60000}
            step={1000}
            onChange={(v) => void updateSetting("default_timeout_ms", v)}
          />
          <Select
            label="Default window matching policy"
            value={settings.match_policy}
            onChange={(e) => void updateSetting("match_policy", e.target.value)}
          >
            <option value="exe_and_title">Executable + title</option>
            <option value="exe_only">Executable only</option>
            <option value="title_only">Title only</option>
            <option value="strict">Strict (exe + title + class)</option>
          </Select>
          <div className="flex items-center justify-between gap-2">
            <span className="text-sm">Confirm before launch</span>
            <Switch
              checked={settings.confirm_launch}
              onCheckedChange={(v) => void updateSetting("confirm_launch", v)}
              label="Confirm before launch"
            />
          </div>
          <div className="flex items-center justify-between gap-2">
            <span className="text-sm">Notifications</span>
            <Switch
              checked={settings.notifications}
              onCheckedChange={(v) => void updateSetting("notifications", v)}
              label="Notifications"
            />
          </div>
          <div className="flex items-center justify-between gap-2">
            <span className="text-sm">Start with Windows (autostart)</span>
            <Switch
              checked={settings.autostart}
              onCheckedChange={(v) => void updateSetting("autostart", v)}
              label="Autostart with Windows"
            />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Global hotkeys</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          {hotEntries.length === 0 ? (
            <p className="text-sm text-slate-500">No hotkeys configured.</p>
          ) : (
            <ul className="space-y-1.5">
              {hotEntries.map(([k, v]) => (
                <li key={k} className="flex items-center gap-2">
                  <Badge tone="blue">{k}</Badge>
                  <span className="flex-1 truncate font-mono text-xs">{v}</span>
                  <Button
                    size="sm"
                    variant="ghost"
                    aria-label={`Remove hotkey ${k}`}
                    onClick={() => {
                      const next = { ...settings.hotkeys };
                      delete next[k];
                      void updateSetting("hotkeys", next);
                    }}
                  >
                    <Trash2 size={13} aria-hidden />
                  </Button>
                </li>
              ))}
            </ul>
          )}
          <div className="flex gap-2">
            <Input aria-label="Hotkey name" placeholder="e.g. launch-dev" value={hotKey} onChange={(e) => setHotKey(e.target.value)} />
            <Input aria-label="Hotkey binding" placeholder="e.g. Ctrl+Alt+D" value={hotValue} onChange={(e) => setHotValue(e.target.value)} />
            <Button
              size="sm"
              variant="outline"
              onClick={() => {
                if (!hotKey.trim() || !hotValue.trim()) return;
                void updateSetting("hotkeys", { ...settings.hotkeys, [hotKey.trim()]: hotValue.trim() });
                setHotKey("");
                setHotValue("");
              }}
            >
              <Plus size={13} aria-hidden /> Add
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Storage, backup & recovery</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="text-sm text-slate-500">
            Storage:{" "}
            <span className="font-mono text-xs">{settings.storage_path || "(default app data directory)"}</span>
          </p>
          <div className="flex flex-wrap gap-2">
            <Button size="sm" variant="outline" onClick={() => void handleExportAll()}>
              <Download size={13} aria-hidden /> Export all
            </Button>
            <Button size="sm" variant="outline" onClick={() => fileRef.current?.click()}>
              <Upload size={13} aria-hidden /> Import backup
            </Button>
            <input
              ref={fileRef}
              type="file"
              accept=".json,application/json"
              className="hidden"
              aria-label="Import backup file"
              onChange={(e) => {
                const f = e.target.files?.[0];
                if (f) void handleImportFile(f);
                e.target.value = "";
              }}
            />
            <Button
              size="sm"
              variant="outline"
              onClick={() =>
                void (async () => {
                  setRecovered(await recoverOffscreen());
                })()
              }
            >
              <LifeBuoy size={13} aria-hidden /> Recover off-screen windows
            </Button>
            {recovered !== null ? <Badge tone="green">Recovered {recovered}</Badge> : null}
          </div>
          {workspaces.length > 0 ? (
            <div className="flex flex-wrap gap-1.5">
              {workspaces.map((w) => (
                <Button
                  key={w.id}
                  size="sm"
                  variant="ghost"
                  onClick={() => void saveFile(`${w.name}.workset.json`, serializeWorkspace(w))}
                >
                  <Download size={12} aria-hidden /> {w.name}
                </Button>
              ))}
            </div>
          ) : null}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Application logs</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <Button size="sm" variant="outline" onClick={() => void getLogs(200).then(setLogs)}>
            Load logs
          </Button>
          <div className="max-h-56 overflow-auto rounded-md bg-slate-950 p-3 font-mono text-xs text-slate-100" role="log" aria-label="Application logs">
            {logs.length === 0 ? "No logs loaded." : logs.map((l, i) => <div key={i}>{l}</div>)}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
