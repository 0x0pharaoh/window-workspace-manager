import { useCallback, useEffect, useState } from "react";
import type { ViewKey } from "@/types";
import { Layout } from "@/components/Layout";
import { HomePage } from "@/features/home/HomePage";
import { WorkspacesPage } from "@/features/workspaces/WorkspacesPage";
import { LayoutEditorPage } from "@/features/layout-editor/LayoutEditorPage";
import { ApplicationsPage } from "@/features/applications/ApplicationsPage";
import { SessionsPage } from "@/features/sessions/SessionsPage";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { CaptureDialog } from "@/features/capture/CaptureDialog";
import { useWorkspaces } from "@/stores/useWorkspaces";
import { useMonitors } from "@/stores/useMonitors";
import { useSessions } from "@/stores/useSessions";
import { useSettings } from "@/stores/useSettings";
import { onLaunchProgress } from "@/services/tauri";
import { useWorkspaceHotkeys } from "@/hooks/useWorkspaceHotkeys";

function initialDark(): boolean {
  try {
    const saved = localStorage.getItem("workset-theme");
    if (saved === "dark") return true;
    if (saved === "light") return false;
  } catch {
    // ignore
  }
  return window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false;
}

export default function App() {
  const [view, setView] = useState<ViewKey>("home");
  const [dark, setDark] = useState<boolean>(() => initialDark());
  const [captureOpen, setCaptureOpen] = useState(false);

  const fetchWorkspaces = useWorkspaces((s) => s.fetchWorkspaces);
  const fetchMonitors = useMonitors((s) => s.fetchMonitors);
  const fetchSessions = useSessions((s) => s.fetchSessions);
  const setProgress = useSessions((s) => s.setProgress);
  const loadSettings = useSettings((s) => s.load);
  const settings = useSettings((s) => s.settings);
  useWorkspaceHotkeys();

  useEffect(() => {
    void fetchWorkspaces();
    void fetchMonitors();
    void fetchSessions();
    void loadSettings();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void onLaunchProgress((item) => setProgress(item)).then((u) => {
      unlisten = u;
    });
    return () => unlisten?.();
  }, [setProgress]);

  useEffect(() => {
    const root = document.documentElement;
    const effective = settings?.theme === "light" ? false : settings?.theme === "dark" ? true : dark;
    root.classList.toggle("dark", effective);
    try {
      localStorage.setItem("workset-theme", effective ? "dark" : "light");
    } catch {
      // ignore
    }
  }, [dark, settings?.theme]);

  const toggleDark = useCallback(() => setDark((d) => !d), []);

  return (
    <Layout view={view} onNavigate={setView} dark={dark} onToggleDark={toggleDark}>
      {view === "home" ? (
        <HomePage onNavigate={setView} onOpenCapture={() => setCaptureOpen(true)} />
      ) : null}
      {view === "workspaces" ? <WorkspacesPage /> : null}
      {view === "editor" ? <LayoutEditorPage /> : null}
      {view === "apps" ? <ApplicationsPage /> : null}
      {view === "sessions" ? <SessionsPage /> : null}
      {view === "settings" ? <SettingsPage /> : null}
      <CaptureDialog
        open={captureOpen}
        onClose={() => setCaptureOpen(false)}
        onSaved={() => {
          void fetchWorkspaces();
        }}
      />
    </Layout>
  );
}
