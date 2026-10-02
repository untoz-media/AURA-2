# AURA-2 System Tray

**Milestone:** M002.5  
**Status:** Initial implementation complete

AURA-2 is designed to remain available in the Windows notification area even when its main window is hidden.

## Behaviour

### Close button

Closing the main AURA window does not terminate the application.

Instead:

```text
Close requested
      ↓
Prevent close
      ↓
Hide main window
      ↓
AURA remains resident
```

The application only exits when the user explicitly chooses **Quit AURA** from the tray menu.

### Left click

A left click on the tray icon restores and focuses the main AURA window.

### Tray menu

The initial menu contains:

- Open AURA
- Pause AURA
- Settings
- Quit AURA

## Pause state

Pause is a real Core state, not just a visual toggle.

When paused:

- the runtime state is stored in the Rust Core
- the tray check item is checked
- the desktop UI receives `aura:runtime-state`
- new user commands are rejected before routing or execution

This will later connect to voice listening, vision access, automations and other background capabilities.

## Tray icon

M002.5 uses a small procedurally generated AURA-spectrum icon so the tray works without requiring binary icon assets in the repository workflow.

The final Windows application icon set will be added during M002.12.

## Future

M002.9 will make **Settings** navigate to a real settings surface.

M002.10 will expand background lifecycle behaviour.

M002.11 will add optional Windows autostart.
