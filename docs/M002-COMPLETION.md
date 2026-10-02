# M002 Completion — First Packaged Windows Build

**Milestone:** M002.12  
**Build:** AURA-2 0.2.0-alpha.1  
**Target:** Windows x64  
**Installer:** NSIS setup executable

M002 turns AURA-2 from a repository skeleton into a real Windows desktop foundation.

## Packaged build

The desktop package now enables Tauri bundling and targets NSIS on Windows.

A local build can be created from the repository root with:

```powershell
npm install
npm run build
```

The build script first generates the complete Tauri icon set from:

`apps/desktop/src-tauri/aura-app-icon.svg`

and then builds the desktop application and NSIS installer.

Expected installer output:

```text
apps/desktop/src-tauri/target/release/bundle/nsis/
```

## GitHub Actions

`.github/workflows/windows-build.yml` provides a Windows-native build pipeline.

It runs on:

- pull requests that change the desktop app
- manual workflow dispatch
- tags matching `aura-v*`

Successful runs upload the NSIS installer as the artifact:

`AURA-2-Windows-x64`

## M002 deliverables

M002 now includes:

- Tauri 2 + React + TypeScript + Vite desktop stack
- AURA Design System
- Core ↔ Desktop bridge
- system tray
- global shortcut
- AURA Overlay
- runtime states
- Settings
- persistent Background Mode
- optional Windows autostart
- application icon source and generated icon pipeline
- Windows NSIS packaging
- CI build pipeline

## Versioning

The first packaged Desktop Foundation build is:

`0.2.0-alpha.1`

This is an internal/pre-Beta development build, not the AURA-2 Beta release.

## Next

Development continues with:

**M003 — Computer Control**

M003 connects the existing command bridge and Overlay to real deterministic Windows actions and the first permission engine.
