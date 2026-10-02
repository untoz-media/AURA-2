# AURA Overlay

**Milestone:** M002.7  
**Status:** Initial implementation complete

The AURA Overlay is the fast, keyboard-first surface for interacting with AURA without opening the full desktop application.

## Shortcut

Default development shortcut:

`Ctrl + Shift + Space`

The shortcut now toggles the Overlay instead of the full AURA window.

## Window behaviour

The Overlay is a dedicated Tauri window:

- label: `overlay`
- 720 × 150 logical pixels
- hidden at startup
- always on top
- excluded from the Windows taskbar
- undecorated
- non-resizable
- centered on the active display by the window manager
- automatically hidden when it loses focus

Pressing `Escape` also hides it.

## Shared architecture

The Overlay is not a separate assistant.

It reuses:

- AURA Design System
- Core ↔ Desktop bridge
- runtime Pause state
- Core status events
- command validation
- the same Rust Core

Commands sent from the Overlay are tagged with:

```text
source = "overlay"
```

This allows future routing, telemetry and permissions logic to distinguish quick commands from full-app, voice or automation requests without duplicating the execution system.

## UX

The Overlay contains:

- AURA mark
- current runtime activity
- status pill
- one command input
- Open app button
- Escape hint

The goal is to keep the surface extremely fast and minimal.

## Next

M002.9 will add Settings.

M002.10 will formalize background lifecycle.

M003 will connect Overlay commands to real Windows actions.
