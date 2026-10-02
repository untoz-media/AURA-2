# AURA-2 Background Mode

**Milestone:** M002.10  
**Status:** Initial implementation complete

Background Mode defines whether AURA remains resident after the main desktop window is closed.

## Behaviour

### Run in background enabled

This is the default.

When the user closes the main window:

1. AURA intercepts the window close request.
2. The window is hidden.
3. The Core, tray and global shortcut remain alive.
4. The AURA Overlay remains available.
5. Future background modules can continue running subject to their permissions.

### Run in background disabled

When the user closes the main window:

1. AURA allows the user intent to terminate the desktop session.
2. The application explicitly exits.
3. The tray, Overlay and Core stop with the process.

## Persistent preference

The background-mode preference is stored locally in AURA's application config directory as:

`desktop-preferences.json`

The file currently contains:

```json
{
  "backgroundEnabled": true
}
```

This uses the Tauri application config directory rather than a project-folder file.

## Runtime state

The shared runtime snapshot now contains:

```ts
{
  paused: boolean;
  backgroundEnabled: boolean;
}
```

The same state is consumed by:

- Settings
- desktop UI
- Overlay
- close-window lifecycle logic
- future background services

## Architecture

Background Mode is event-driven.

AURA does **not** add a polling loop just to remain resident. The Tauri event loop, tray, global shortcut and future explicit workers keep the application available without an artificial busy loop.

## Lifecycle events

AURA emits:

- `aura:lifecycle-event`

Initial lifecycle event kinds:

- `foreground.entered`
- `background.entered`
- `background.disabled`

These events provide a foundation for future automations and background modules.

## Relationship to Pause

**Pause AURA** and **Run in background** are separate concepts.

- Pause: AURA stays running but rejects new commands/background actions.
- Background disabled: closing the main window exits the application.

## Next

M002.11 adds optional Windows autostart.

M002.12 creates the first packaged Windows build.
