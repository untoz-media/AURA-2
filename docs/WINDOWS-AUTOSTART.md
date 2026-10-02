# AURA-2 Windows Autostart

**Milestone:** M002.11  
**Status:** Initial implementation complete

AURA-2 can optionally register itself to launch when the user signs in to Windows.

## Implementation

Autostart uses the official Tauri autostart plugin.

The React UI does not access the plugin directly. Instead:

```text
Settings UI
    ↓
AURA bridge
    ↓
Rust Core command
    ↓
Tauri autostart plugin
    ↓
Windows startup registration
```

This keeps operating-system actions behind the AURA Core.

## Start with Windows

The setting is available in:

`Settings → General → Start with Windows`

When enabled, AURA registers itself with the operating system.

When disabled, the registration is removed.

## Background startup

Autostart launches AURA with:

`--background`

When this argument is present:

- the Core and tray start normally
- the global Overlay shortcut is registered
- the main AURA window is hidden immediately
- AURA remains available without interrupting the Windows sign-in experience

## Source of truth

The operating-system autostart registration is the source of truth.

At startup AURA asks the Tauri autostart manager whether registration is currently enabled and exposes that value through the shared runtime snapshot.

## Runtime state

The desktop runtime snapshot now includes:

```ts
{
  paused: boolean;
  backgroundEnabled: boolean;
  autostartEnabled: boolean;
}
```

## Notes

Autostart is optional and disabled unless the user explicitly enables it.

The user can always disable it again from AURA Settings.
