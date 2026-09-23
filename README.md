# Workset — Windows Workspace Manager

Workset lets you save groups of Windows applications as reusable **workspaces**,
launch them together with one click, and have every window moved to its saved
monitor, position, size and state. It also captures your current desktop into a
new workspace.

Local-first: React + TypeScript + Tailwind CSS v4 frontend, Tauri 2 + Rust
backend, SQLite persistence. No server, no telemetry, works offline.

## Features

- Workspace CRUD: create, rename, duplicate, delete (confirmed), favorites,
  colors, icons, per-workspace hotkey, import/export (versioned JSON)
- Apps per workspace: exe path or URL/URI, args, working dir, launch delay,
  target monitor, normalized layout rect, window state, launch policy
  (`if_not_running`, `always_new`, `reuse`, `focus`, `skip`), window matching
  rules (exe/title/regex/class)
- Visual layout editor: scaled multi-monitor preview, drag/resize with
  snapping, 7 editable presets, manual %/px editing
- Capture desktop: enumerate visible windows, pick which to keep, save into a
  new or existing workspace
- Launch engine: validation → monitor detection → policy → spawn → wait/retry
  with timeout → move/resize/state, per-app status, progress events,
  cancellation, partial-failure recovery, launch logs
- Multi-monitor: negative origins, portrait, DPI scale, work-area coordinates,
  fallback when a monitor is gone, one-click off-screen recovery
- Automation: global hotkeys (per workspace), tray menu, opt-in Windows
  autostart, launch-on-startup workspace
- Sessions: active session tracking, close/restart, missing-window detection

## Architecture

```
src/                  React 18 + TS (strict) + Vite 6 + Tailwind v4 + Zustand
  app / components / features/{home,workspaces,layout-editor,applications,
  capture,sessions,settings} / hooks / lib / services / stores / types
src-tauri/            Tauri 2 + Rust (rusqlite bundled, tokio, windows 0.58)
  src/{main,commands,models,database,coords,matching,windows,monitors,
       processes,discovery,capture,launcher,sessions,import_export,
       settings,tray,error}.rs
  migrations/001_init.sql   capabilities/main.json (least privilege)
```

The frontend never touches the OS directly: every native operation goes
through typed `invoke()` calls defined in `src/services/tauri.ts`.
Coordinates are stored normalized (0..1 of the monitor **work area**) and
converted to native pixels in Rust (`coords.rs`), so layouts survive DPI and
resolution changes.

## Prerequisites (Windows 10/11)

- Node.js 20+ and npm
- Rust stable (MSVC target) + **Visual Studio C++ build tools**
  (Desktop development with C++ workload) + WebView2 runtime
- WiX Toolset v3 (only for the MSI target; NSIS works without it)

## Development

```cmd
npm install
npm run tauri:dev
```

Useful commands:

```cmd
npm run dev          - Vite frontend only (backend calls fall back gracefully)
npm run build        - typecheck + production frontend bundle
npm run test         - frontend unit tests (vitest)
npm run check        - tsc --noEmit
cd src-tauri && cargo test        - Rust unit tests (needs MSVC linker)
cd src-tauri && cargo fmt --check - Rust formatting
npm run tauri:build  - production installer (NSIS + MSI)
```

Database lives at `%LOCALAPPDATA%\Workset\workset.db` (WAL mode, migrations
run automatically). Uninstalling keeps your workspaces.

## Testing

- Frontend: `npx vitest run` — 23 tests (layout presets, coordinate scaling,
  workspace store CRUD, layout editor interactions, import validation).
- Rust: `cargo test` — 26 tests (coordinate conversion incl. negative-origin
  and portrait monitors, monitor picking, matching scores, validation,
  arg quoting, SQLite CRUD/sessions/logs/settings/catalog, import/export
  schema + confirmation gate).
- Windows E2E (manual, runnable procedure in `tests/e2e-windows.md`):
  create workspace → add Notepad/Calculator → save/reload → launch →
  verify positions → restart → missing-app error → unplug-monitor recovery.

## Windows API notes / limitations

- Uses `EnumWindows`, `GetWindowRect`, `GetWindowThreadProcessId`,
  `SetWindowPos`, `ShowWindow`, `EnumDisplayMonitors`, `GetMonitorInfoW`,
  `GetDpiForMonitor`, `RegisterHotKey` (via Tauri global-shortcut),
  `CreateProcess` (via `std::process::Command`, never a shell).
- Matching never relies on PID alone (Chrome etc. reuse processes); it scores
  exe + class + title/regex + freshness, and never moves a window on a
  title-substring hit alone.
- UWP/Store apps: add their `.lnk` from the Start Menu; the shortcut itself
  is the launch target.
- Elevated (admin) windows cannot be moved/resized from a non-elevated
  Workset — the error names the limitation instead of failing silently.
- `fullscreen` state is approximated with maximize; true exclusive
  fullscreen needs app cooperation.

## Workspace file format

Versioned JSON envelope (`schema_version: 1`):

```json
{
  "schema_version": 1,
  "exported_at": "2026-09-23T12:00:00.000Z",
  "workspace": { "name": "Development", "apps": [ ... ] }
}
```

Imports with shell-like tokens (`;`, `&&`, `$(`, …) in exe paths/args require
an explicit confirmation click. A bare workspace object (no envelope) is also
accepted.

## Troubleshooting

| Symptom | Fix |
|---|---|
| Window not found after launch | Increase launch delay/timeout; tighten match rules (class or title regex) |
| Apps land off-screen | Settings → Recovery → “Move windows back on-screen”; reconnect the monitor |
| “Executable not found” | Re-browse the exe (path moved) or remove the entry |
| Hotkey does nothing/conflict | Change it in workspace settings; OS-reserved combos stay with the OS |
| DB corrupted | Delete `%LOCALAPPDATA%\Workset\workset.db` (workspaces are lost; imports restore from `.workset.json` backups) |

## Release

Bump `version` in `package.json` + `src-tauri/tauri.conf.json` (+ `Cargo.toml`),
push a tag — `.github/workflows/ci.yml` runs frontend tests, `cargo test`,
and `tauri build` on `windows-latest`, uploading the NSIS/MSI artifacts.
The installer is **unsigned** unless you configure Tauri code signing.
