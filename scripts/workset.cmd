@echo off
REM Workset developer scripts (run from the repo root)
if "%1"=="dev" npm run tauri:dev
if "%1"=="build" npm run tauri:build
if "%1"=="test" (
  call npx vitest run
  cd src-tauri && cargo test && cd ..
)
if "%1"=="" echo Usage: scripts\workset.cmd [dev^|build^|test]
