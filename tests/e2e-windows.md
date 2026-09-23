# Workset Windows E2E procedure (run on a real Windows 10/11 machine)
#
# Prerequisites: production build installed (`npm run tauri:build`),
# a second monitor if you want to exercise multi-monitor cases.

1. Create workspace "E2E" (Workspaces -> New). Expect: appears in list, persists.
2. Add apps: Notepad (`notepad.exe`), Calculator (`calc.exe`), a Chrome/Edge URL.
   Set working dir + args on Notepad. Expect: validation errors for bogus paths.
3. Restart Workset. Expect: workspace + apps intact (SQLite persistence).
4. Launch "E2E". Expect: all three open; progress reaches done; logs show per-app lines.
5. Move/resize via Layout Editor, relaunch. Expect: windows land on saved rects.
6. Close one app, press Restart on the session. Expect: only the missing app relaunches
   under `if_not_running` (default).
7. Point an entry at `C:\does-not-exist\app.exe`, launch. Expect: clear per-app error,
   other apps still launch.
8. Disconnect the secondary monitor, launch a two-monitor workspace.
   Expect: fallback to primary, nothing permanently off-screen; run
   Settings -> Recovery -> Move windows back on-screen afterwards.
9. Capture desktop with Notepad open. Expect: Notepad listed, selectable, saved into workspace.
10. Set a workspace hotkey, press it from another app. Expect: workspace launches.
