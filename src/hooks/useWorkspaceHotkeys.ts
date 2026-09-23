import { useEffect } from "react";
import { useWorkspaces } from "@/stores/useWorkspaces";
import { launchWorkspace } from "@/services/tauri";

/**
 * Registers each workspace's global hotkey via the Tauri global-shortcut
 * JS API and launches the workspace when it fires. Re-registers whenever
 * the workspace list changes; unregisters everything on cleanup.
 * No-ops outside Tauri (browser dev) with a console warning.
 */
export function useWorkspaceHotkeys(): void {
  const workspaces = useWorkspaces((s) => s.workspaces);

  useEffect(() => {
    let cancelled = false;
    const registered: string[] = [];

    async function sync(): Promise<void> {
      try {
        const mod = await import("@tauri-apps/plugin-global-shortcut");
        if (cancelled) return;
        await mod.unregisterAll().catch(() => undefined);
        if (cancelled) return;
        const seen = new Set<string>();
        for (const ws of workspaces) {
          const hotkey = ws.hotkey.trim();
          if (!hotkey || seen.has(hotkey.toLowerCase())) continue;
          seen.add(hotkey.toLowerCase());
          const id = ws.id;
          try {
            await mod.register(hotkey, () => {
              void launchWorkspace(id);
            });
            registered.push(hotkey);
          } catch (err) {
            console.warn(`[hotkeys] could not register "${hotkey}":`, err);
          }
        }
      } catch (err) {
        console.warn("[hotkeys] global shortcuts unavailable:", err);
      }
    }

    void sync();
    return () => {
      cancelled = true;
      void (async () => {
        try {
          const mod = await import("@tauri-apps/plugin-global-shortcut");
          await mod.unregisterAll().catch(() => undefined);
        } catch {
          // ignore teardown failures
        }
      })();
    };
  }, [workspaces]);
}
