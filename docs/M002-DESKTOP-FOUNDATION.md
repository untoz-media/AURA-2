# M002 — Desktop Foundation

## Goal

Turn AURA-2 from a repository architecture into a real Windows desktop application.

## Stack decision

AURA-2 Desktop uses:

- Tauri 2
- Rust
- React
- TypeScript
- Vite

The application remains Windows-first and local-first.

## Current implementation

### M002.1 — Desktop stack
Status: **complete**

Tauri 2 + React + TypeScript + Vite selected.

### M002.2 — Base desktop application
Status: **initial implementation complete**

The first shell includes:

- Native Tauri desktop window
- AURA-2 application shell
- Local command input prototype
- AURA status states
- Module overview
- Rust command bridge foundation
- Global shortcut foundation

Default development shortcut:

`Ctrl + Shift + Space`

It toggles the main AURA window. This shortcut will become user-configurable.

## Next

### M002.3 — AURA Design System
- Final logo assets
- Typography
- Component tokens
- Motion language
- Window chrome
- Overlay-specific UI

### M002.4 — Core communication
- Typed frontend/Rust bridge
- Event bus
- Action requests
- Status updates
- Errors and approvals

### M002.5 — System tray
- Background resident process
- Open AURA
- Pause AURA
- Settings
- Quit

### M002.6 — Global shortcut
- User-configurable shortcut
- Conflict handling
- Settings persistence

### M002.7 — AURA Overlay
- Compact window
- Fast show/hide
- Keyboard-first UX
- Expansion into full app

### M002.8 — Runtime states
- Idle
- Listening
- Thinking
- Working
- Waiting for approval

### M002.9 — Settings
- General
- Privacy
- Permissions
- Models
- Integrations

### M002.10 — Background mode
- Keep AURA available after the main window closes

### M002.11 — Start with Windows
- Optional autostart

### M002.12 — First packaged build
- Windows installer
- Version metadata
- App icon
- First internal build

## Running locally

From the repository root:

```powershell
npm install
npm run dev
```

For a frontend-only preview:

```powershell
npm run frontend:dev
```

Tauri development on Windows requires Rust, Microsoft C++ Build Tools and WebView2.
