# Build a Production-Ready Windows Workspace Manager

## Role

Act as a senior Windows desktop application engineer, Rust developer, React/TypeScript engineer, and product designer.

Build a complete, production-ready Windows desktop application that allows users to create, save, and launch groups of applications together, automatically arrange their windows, and restore predefined desktop workspaces.

**Do not stop at planning, scaffolding, mockups, or a prototype. Implement the complete application in the existing repository.**

---

# 1. Product Overview

Build a Windows application named **Workset** (keep the application name configurable).

Workset allows users to create reusable workspaces containing multiple applications, define where each application should open, and launch the entire workspace with one action.

Example:

A user creates a workspace named `Development`.

When launched, Workset:

1. Opens Visual Studio Code with a predefined project directory.
2. Opens Chrome with specified URLs.
3. Opens Windows Terminal with a configured working directory and command.
4. Opens Postman.
5. Opens Slack.
6. Waits for the application windows to become available.
7. Moves and resizes each window according to the saved layout.
8. Places applications on the correct monitors.
9. Displays the workspace's launch status.

The user must also be able to save their current desktop arrangement as a reusable workspace.

The application must work as a genuine Windows desktop application, not a browser-only dashboard.

---

# 2. Technology Stack

Use the following architecture:

### Desktop application
- Tauri 2
- Rust for Windows-native functionality
- React
- TypeScript
- Vite

### UI
- Tailwind CSS
- shadcn/ui where compatible
- Lucide icons

### Persistence
- SQLite for local application data
- SQL migrations
- Versioned workspace configuration

### Windows integration
Use native Windows APIs through Rust, with appropriate Windows crates and documented API requirements.

Prioritize Windows 10 and Windows 11 compatibility.

Do not use Docker.

Do not build a separate backend server. All core functionality must work locally without an internet connection.

Use stable, maintained dependencies and pin compatible versions.

---

# 3. Core Application Features

## 3.1 Workspace Management

Implement complete CRUD functionality.

Users must be able to:

- Create a workspace.
- Rename a workspace.
- Duplicate a workspace.
- Delete a workspace with confirmation.
- Add applications to a workspace.
- Remove applications.
- Reorder applications.
- Save workspace changes.
- Import and export workspace configurations.
- Mark a workspace as a favorite.
- Assign an optional keyboard shortcut.
- Launch a workspace.
- Close applications belonging to a workspace.
- Restart a workspace.
- View recent workspace activity.

Each workspace should support a name, description, icon, color, application list, display layout, and launch settings.

Persist all data locally.

## 3.2 Application Management

Allow users to add applications using:

- File picker for executable files.
- Drag-and-drop executable files.
- Application discovery.
- Existing running application windows.
- Custom executable paths.
- Windows applications launched through URI protocols where supported.

Each application entry must support:

- Display name.
- Executable path or launch target.
- Command-line arguments.
- Working directory.
- Optional application icon.
- Optional window matching rules.
- Launch delay.
- Target monitor.
- Window position and size.
- Window state.
- Optional startup command.
- Optional URL or URI.
- Launch behavior when already running.

Support applications such as:

- Visual Studio Code
- Google Chrome
- Microsoft Edge
- Windows Terminal
- PowerShell
- Command Prompt
- Notepad
- Slack
- Discord
- Postman
- Other ordinary Windows desktop applications

Do not hardcode support for only these applications.

Users must be able to configure arbitrary compatible Windows applications.

## 3.3 Application Discovery

Implement a native application picker.

Discover applications through appropriate Windows mechanisms, including installed application shortcuts, Start Menu entries, and common application locations.

Show:

- Application name.
- Icon where available.
- Executable or launch target.
- Search and filtering.
- Installed application suggestions.

Handle missing executables gracefully.

Do not assume every Start Menu shortcut directly points to an executable.

---

# 4. Visual Workspace Layout Editor

Build a fully functional visual layout editor.

Users must be able to arrange application windows through an interactive desktop preview.

## Layout editor requirements

- Display connected monitors.
- Show each monitor's resolution and orientation.
- Display a scaled representation of each monitor.
- Add application windows to the layout.
- Drag windows to reposition them.
- Resize windows interactively.
- Snap windows to edges and grid regions.
- Support overlapping windows where appropriate.
- Switch between monitor layouts.
- Configure window dimensions and coordinates manually.
- Support percentage-based and pixel-based positioning.
- Preview the final workspace arrangement.
- Save and restore layouts.

Each application must be represented as a movable and resizable window card.

Show its application name and assigned monitor.

Support configurable layout presets:

- Two-column layout.
- Three-column layout.
- Main window plus sidebar.
- Main window plus bottom panel.
- Four-quadrant layout.
- Full-screen application.
- Custom freeform layout.

The user must be able to edit presets instead of being locked into fixed templates.

### Layout coordinate system

Use normalized coordinates relative to the usable monitor work area.

Store:

- x
- y
- width
- height

Support conversion between normalized coordinates and native Windows screen coordinates.

Account for:

- Negative monitor coordinates.
- Multiple monitor arrangements.
- Different resolutions.
- Different DPI scaling.
- Taskbar work areas.
- Portrait monitors.
- Monitor disconnection.
- Monitor resolution changes.

Do not assume the primary monitor begins at coordinate `(0, 0)`.

---

# 5. Capture Current Desktop

Implement a complete **Capture Workspace** feature.

When the user clicks Capture:

1. Enumerate currently visible top-level application windows.
2. Identify their owning processes.
3. Retrieve application names and executable paths where possible.
4. Retrieve window titles and window classes.
5. Retrieve window positions and dimensions.
6. Determine the monitor containing each window.
7. Capture the window's current state.
8. Display detected windows in a selection interface.
9. Allow users to choose which windows to include.
10. Allow users to edit names and matching rules.
11. Save the selected windows into a new or existing workspace.

Exclude Workset's own windows, invisible windows, and inappropriate system windows.

Handle applications with multiple windows.

Provide a clear interface when a window cannot be identified reliably.

Do not silently create broken workspace entries.

---

# 6. Native Windows Window Management

Implement a dedicated Rust module for native Windows window management.

Use suitable Win32 APIs, including where appropriate:

- `EnumWindows`
- `EnumDisplayMonitors`
- `GetMonitorInfoW`
- `GetWindowRect`
- `GetWindowThreadProcessId`
- `SetWindowPos`
- `ShowWindow`
- `IsWindow`
- `IsWindowVisible`
- `GetForegroundWindow`
- `RegisterHotKey`
- `UnregisterHotKey`
- DPI-awareness APIs
- DWM window attribute APIs

Use suitable process-launching APIs, such as `CreateProcessW` or equivalent safe Rust abstractions.

### Window management requirements

Implement:

- Enumerating top-level windows.
- Identifying application-owned windows.
- Launching applications.
- Waiting for windows to appear.
- Matching launched processes to their windows.
- Moving windows between monitors.
- Resizing windows.
- Restoring minimized windows.
- Handling maximized windows.
- Applying saved window states.
- Detecting closed or missing windows.
- Detecting inaccessible or protected windows.
- Handling windows that refuse to resize.
- Handling applications that launch asynchronously.
- Avoiding accidental manipulation of unrelated system windows.

Do not rely exclusively on process IDs.

Applications such as Chrome may use multiple processes or reuse an existing process.

Implement configurable window matching using appropriate combinations of:

- Executable identity.
- Process identity.
- Window title.
- Window class.
- Launch timestamp.
- Existing versus newly created windows.

Avoid fragile matching based solely on exact window titles.

Do not forcibly terminate unrelated processes.

Do not silently elevate privileges.

If elevated applications cannot be manipulated, report the limitation clearly.

---

# 7. Workspace Launch Engine

Build a robust launch orchestration engine.

Each workspace launch must follow a reliable sequence:

1. Validate the workspace configuration.
2. Detect connected monitors.
3. Resolve application paths.
4. Check whether applications are already running.
5. Apply each application's configured launch policy.
6. Launch applications that need to be opened.
7. Wait for their windows.
8. Match the correct windows.
9. Restore minimized windows when needed.
10. Move windows to their target monitors.
11. Apply saved sizes and positions.
12. Apply configured window states.
13. Report completion or partial failure.

### Launch policies

Support:

- Launch only if not running.
- Always launch a new instance when supported.
- Reuse an existing matching window.
- Focus an existing matching window.
- Skip an application.
- Ask the user when matching is ambiguous.

### Reliability

Implement:

- Configurable launch delays.
- Window detection timeouts.
- Retry handling.
- Cancellation.
- Progress reporting.
- Per-application status.
- Partial failure recovery.
- Detailed error messages.
- Launch logs.

One application's failure must not prevent unrelated applications from launching unless the user explicitly configures a dependency.

Do not block the UI thread while launching applications or waiting for windows.

---

# 8. Multi-Monitor Support

Implement genuine multi-monitor support.

Users must be able to:

- Assign applications to individual monitors.
- Arrange applications across several monitors.
- Configure layouts for different monitor setups.
- Identify each monitor.
- Detect changes in connected displays.
- Recover when a monitor is disconnected.
- Recalculate layouts when resolution changes.
- Restore windows after docking or undocking a laptop.

Provide a fallback strategy when a configured monitor is unavailable.

Never allow a workspace to silently place every application off-screen.

Include a recovery action to move Workset-managed windows back into visible screen areas.

---

# 9. Window Capture and Matching

Implement reliable window identification.

A workspace application should optionally have matching rules such as:

- Process executable.
- Window title contains.
- Window title regular expression.
- Window class.
- Launch instance.
- Custom matching conditions.

Validate regular expressions.

Handle multiple matching windows through a selection interface.

Never arbitrarily move an unrelated window because its title happens to contain a common word.

Persist stable matching configuration rather than assuming process IDs remain valid across sessions.

---

# 10. Workspace Automation

Implement useful workspace launch automation.

Support:

- Launch workspace on application startup.
- Launch workspace on Windows login, with explicit user permission.
- Global keyboard shortcuts.
- System tray menu.
- Optional launch confirmation.
- Configurable launch behavior.
- Launch workspace from a tray menu.
- Close a workspace's tracked applications.
- Restart a workspace.

Global hotkeys must be registered and unregistered safely.

Handle shortcut conflicts gracefully.

Do not implement automatic login startup without clearly informing the user and providing a way to disable it.

---

# 11. Workspace Session Tracking

Track which application windows belong to each launched workspace.

Maintain a session model containing:

- Workspace ID.
- Launch session ID.
- Application configuration ID.
- Process ID where applicable.
- Matched window handle.
- Launch timestamp.
- Current status.
- Last known window position.
- Last known window state.

Support:

- Viewing active workspace sessions.
- Identifying missing applications.
- Repositioning tracked windows.
- Closing tracked applications.
- Restarting failed applications.
- Recovering session state after application restart.

Never assume a stored window handle is valid indefinitely.

Validate handles before using them.

Do not kill an application process unless the user explicitly requests termination and the application can be safely identified.

---

# 12. UI/UX Requirements

Build a polished, modern Windows desktop application.

The interface must feel like a professional productivity tool, not a generic admin dashboard.

### Design direction

- Minimal and clean.
- Desktop-first.
- Modern Windows-inspired interface.
- Strong visual hierarchy.
- Excellent spacing and typography.
- Clear workspace cards.
- Useful empty states.
- Consistent icons.
- Light and dark themes.
- Subtle animations.
- Accessible controls.
- Responsive application window layout.

Avoid excessive gradients, oversized decorative sections, and unnecessary dashboard charts.

### Main navigation

Implement:

1. Home
2. Workspaces
3. Layout Editor
4. Applications
5. Active Sessions
6. Settings

### Home

Display:

- Recent workspaces.
- Favorite workspaces.
- Quick launch buttons.
- Currently active workspace sessions.
- Recently used applications.
- Create workspace action.
- Capture current desktop action.

### Workspace details

Display:

- Workspace name.
- Application list.
- Monitor assignments.
- Layout preview.
- Launch settings.
- Edit workspace action.
- Launch workspace button.
- Close workspace button.

### Application picker

Implement a searchable application selection dialog with:

- Installed applications.
- Running applications.
- Browse executable.
- Custom launch target.
- Application details.

### Settings

Include:

- Theme.
- Launch behavior.
- Default launch timeout.
- Default window matching policy.
- Startup behavior.
- Global hotkeys.
- Notification settings.
- Workspace storage.
- Import/export.
- Application logs.
- Recovery tools.

All controls must work.

Do not use static mock data as a substitute for real application functionality.

---

# 13. Database Design

Design and implement a normalized SQLite schema.

Include appropriate tables for:

- Workspaces.
- Workspace applications.
- Monitor configurations.
- Layout configurations.
- Application catalog.
- Launch sessions.
- Session windows.
- User settings.
- Launch logs.
- Database schema migrations.

Use appropriate primary keys, foreign keys, indexes, and constraints.

Use transactions for operations that modify multiple related records.

Persist data across application restarts.

Avoid duplicating workspace application configuration unnecessarily.

Support database upgrades without destroying user data.

Provide a workspace JSON import/export format with schema versioning and validation.

Never execute arbitrary imported commands without explicit user approval.

---

# 14. Security and Reliability

Implement appropriate Windows desktop security practices.

Requirements:

- Validate executable paths.
- Validate imported workspace data.
- Avoid shell injection.
- Avoid unsafe command concatenation.
- Use argument arrays where possible.
- Sanitize user-controlled paths.
- Restrict Tauri capabilities and permissions.
- Keep the frontend isolated from unrestricted operating-system access.
- Expose only necessary Rust commands.
- Handle application crashes.
- Handle corrupted workspace data.
- Handle missing applications.
- Handle disconnected monitors.
- Handle permission errors.
- Handle unexpected process termination.
- Avoid leaking sensitive command-line arguments into logs.
- Never collect or transmit user data.

No telemetry, analytics, cloud account, or remote server is required.

The application must remain useful offline.

---

# 15. Testing Requirements

Implement automated tests throughout the project.

### Rust tests

Test:

- Coordinate conversion.
- Monitor selection.
- Layout validation.
- Window matching rules.
- Workspace configuration validation.
- Application launch configuration.
- Session state transitions.
- Error handling.
- Database operations.
- Import/export serialization.

Mock native Windows APIs where required for deterministic unit testing.

### Frontend tests

Test:

- Workspace CRUD flows.
- Application picker.
- Layout editor interactions.
- Window resizing and repositioning.
- Monitor selection.
- Launch progress.
- Error states.
- Settings.
- Import/export validation.

### End-to-end tests

Implement practical Windows E2E tests using test applications.

Verify:

1. A workspace can be created.
2. Applications can be added.
3. A workspace can be saved and reloaded.
4. Multiple test applications can be launched.
5. Their windows are positioned correctly.
6. The layout survives application restart.
7. Missing applications produce useful errors.
8. Monitor changes do not leave windows permanently off-screen.
9. Existing applications are handled according to launch policies.
10. Workspace sessions can be closed safely.

Do not claim tests passed unless they were actually executed.

If the current environment cannot execute Windows-native tests, document exactly which tests require Windows and provide a runnable Windows test procedure.

---

# 16. Project Structure

Use a clean, maintainable architecture similar to:

```text
workset/
├── src/
│   ├── app/
│   ├── components/
│   ├── features/
│   │   ├── workspaces/
│   │   ├── applications/
│   │   ├── layout-editor/
│   │   ├── sessions/
│   │   └── settings/
│   ├── hooks/
│   ├── lib/
│   ├── services/
│   ├── types/
│   └── styles/
├── src-tauri/
│   ├── src/
│   │   ├── commands/
│   │   ├── windows/
│   │   ├── processes/
│   │   ├── monitors/
│   │   ├── launcher/
│   │   ├── sessions/
│   │   ├── database/
│   │   └── main.rs
│   ├── capabilities/
│   ├── migrations/
│   └── tauri.conf.json
├── tests/
├── scripts/
├── README.md
└── package.json
```

Adapt the structure to the actual project when necessary.

Keep modules focused and avoid creating unnecessary abstractions.

Use strict TypeScript.

Use typed Rust structures for commands and persisted configuration.

Keep Windows-native operations out of React components.

Keep UI state separate from persistent database state.

Use a clear service boundary between the frontend and Rust backend.

---

# 17. Code Quality Rules

Follow these rules throughout the codebase:

- Strictly follow the DRY principle.
- Keep functions small and focused.
- Use descriptive names.
- Avoid unnecessary comments.
- Add comments only when they explain non-obvious behavior, with a maximum of 1–2 lines.
- Avoid unnecessary abstractions.
- Avoid duplicated business logic.
- Avoid `any` in TypeScript unless absolutely necessary and justified.
- Handle errors explicitly.
- Avoid silent failures.
- Avoid hardcoded machine-specific paths.
- Avoid placeholder implementations.
- Avoid fake functionality.
- Avoid unused dependencies.
- Avoid dead code.
- Avoid unnecessary global state.
- Avoid suppressing compiler errors to force a successful build.

Use proper logging and meaningful error types.

Do not replace a difficult feature with a mock implementation while claiming it is complete.

---

# 18. Performance Requirements

The application must remain responsive while:

- Discovering installed applications.
- Enumerating windows.
- Launching multiple applications.
- Waiting for application windows.
- Updating launch progress.
- Capturing layouts.
- Monitoring session changes.

Use asynchronous execution where appropriate.

Avoid excessive window enumeration and unnecessary UI re-renders.

Do not continuously poll at an unnecessarily high frequency.

Use event-driven mechanisms where practical, with bounded polling as a fallback.

Ensure the application behaves correctly when many windows and multiple monitors are present.

---

# 19. Build and Distribution

Implement a complete Windows build pipeline.

Provide:

- Development setup.
- Production build.
- Windows installer.
- Application icon and metadata.
- Version configuration.
- Release build instructions.
- Clean installation and upgrade behavior.
- Local data persistence.
- Uninstallation behavior that does not unexpectedly delete user workspace data.

Configure a suitable Windows installer format supported by Tauri.

Provide scripts for development, testing, and production builds.

If GitHub Actions is available in the repository, implement a Windows CI workflow that installs dependencies, runs tests, and builds the installer.

Do not claim a signed installer unless code signing is actually configured.

---

# 20. Documentation

Create a complete README containing:

- Product overview.
- Features.
- Architecture.
- Prerequisites.
- Installation.
- Development setup.
- Build commands.
- Testing commands.
- Windows API limitations.
- Workspace configuration format.
- Troubleshooting.
- Known limitations.
- Release instructions.

Include `.env.example` only if environment variables are genuinely required.

Do not include secrets.

Document any native Windows limitations accurately.

---

# 21. Implementation Workflow

Execute the work in the following order without stopping after each phase.

### Phase 1: Inspect and plan

- Inspect the repository.
- Identify existing code and conventions.
- Check the available development environment.
- Define the architecture.
- Identify Windows API constraints.
- Create an implementation checklist.

Do not overwrite existing work without understanding it.

### Phase 2: Foundation

- Configure Tauri.
- Configure React and TypeScript.
- Configure styling.
- Set up SQLite.
- Implement migrations.
- Establish typed frontend-to-Rust communication.
- Implement application settings.

### Phase 3: Native functionality

- Implement monitor detection.
- Implement application discovery.
- Implement process launching.
- Implement window enumeration.
- Implement window matching.
- Implement native positioning and resizing.
- Implement DPI-aware coordinate conversion.

### Phase 4: Core product

- Implement workspace CRUD.
- Implement application management.
- Implement layout editor.
- Implement desktop capture.
- Implement workspace launch orchestration.
- Implement session tracking.
- Implement multi-monitor recovery.

### Phase 5: Advanced functionality

- Implement global hotkeys.
- Implement system tray.
- Implement startup options.
- Implement workspace import/export.
- Implement application launch policies.
- Implement session controls.

### Phase 6: Quality

- Add automated tests.
- Run available tests.
- Run TypeScript checks.
- Run Rust formatting and linting.
- Run production builds.
- Fix actual errors.
- Verify database persistence.
- Verify native functionality on Windows where available.

### Phase 7: Delivery

- Finalize UI.
- Add documentation.
- Configure installer.
- Configure CI.
- Report completed features and limitations.

**Continue through all phases in one execution. Do not ask for approval between phases.**

If a decision is not specified, make a sensible, documented engineering decision and continue.

If a feature depends on unavailable Windows APIs, permissions, or hardware, implement the supported portion, provide a clear fallback, and document the limitation.

If the environment is not Windows, continue implementing the application and use cross-platform tests or mocks where possible. Do not falsely claim native Windows behavior was verified.

---

# 22. Definition of Done

The project is complete only when:

- The application starts successfully.
- The UI is connected to real Rust functionality.
- Users can create and save workspace groups.
- Users can add real installed applications.
- Users can configure launch arguments and working directories.
- Users can visually arrange application windows.
- Users can capture their existing desktop layout.
- Users can launch multiple applications together.
- Windows are matched and positioned using native Windows functionality.
- Multiple monitors are supported.
- Workspace data persists across restarts.
- Launch errors are displayed clearly.
- Users can recover from missing applications and disconnected monitors.
- Core functionality is tested.
- The production build is configured.
- Documentation is complete.
- No critical feature is left as a fake button, placeholder, or mock.

---

# 23. Final Agent Report

At the end of implementation, provide a concise report containing:

1. Features implemented.
2. Architecture and major design decisions.
3. Important files created or modified.
4. Database schema and persistence details.
5. Windows APIs used.
6. Tests actually executed and their results.
7. Build status.
8. Installer location if successfully generated.
9. Known limitations.
10. Remaining work, if any.

Clearly distinguish between:

- Implemented and verified.
- Implemented but not verified.
- Not implemented.

Do not claim completion for functionality that only exists in the UI.

## Final instruction

Build Workset as a real, reliable Windows productivity application that users can depend on for their daily development and work routines.

Prioritize functional correctness, native Windows integration, reliable window positioning, multi-monitor support, and maintainable architecture.

**Start inspecting the repository and implement the complete application now.**