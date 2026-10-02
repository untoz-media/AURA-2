# AURA Settings

**Milestone:** M002.9  
**Status:** Foundation complete

The Settings experience is the control surface for AURA's desktop behaviour, privacy model and future capabilities.

## Sections

The initial information architecture is:

- General
- Appearance
- Privacy
- Permissions
- Models
- Voice
- Overlay
- Shortcuts
- Integrations

## Implementation philosophy

Settings must distinguish between:

1. capabilities that already work
2. capabilities that are planned but not yet implemented

M002.9 intentionally avoids fake toggles.

For example:

- Pause AURA is functional.
- Background tray behaviour is marked Active.
- Overlay behaviour is marked Enabled.
- Start with Windows is marked M002.11.
- Voice features are marked M006.
- OBS integration is marked M004.

## Tray integration

Choosing **Settings** in the system tray:

1. restores the main AURA window
2. emits `aura:open-settings`
3. switches the frontend to the Settings screen

## Permissions foundation

The UI introduces five future permission classes:

- Read
- Act
- Modify
- Destructive
- Sensitive

Destructive and sensitive capabilities are presented as requiring confirmation.

The actual permission engine is implemented with Computer Control in M003 and expanded by later modules.

## Navigation

Settings is an application-level screen, not a separate window.

This keeps:

- one Core connection
- one runtime state
- one permissions context
- one consistent design system

## Persistence

M002.9 defines the Settings surface and working runtime controls.

Persistent user preferences are added as each underlying capability becomes real, avoiding settings that appear configurable but have no effect.
