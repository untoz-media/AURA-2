# M003.5 — Mouse Actions

M003.5 adds controlled native Windows mouse actions to AURA-2.

## Commands

Movement:

- `Move mouse to 500, 300`
- `Move cursor to 500 300`
- `Move o rato para 500, 300`

Scrolling:

- `Scroll down`
- `Scroll down 3`
- `Scroll up 2`
- `Scroll para baixo 3`
- `Scroll para cima 2`

Clicks:

- `Click`
- `Double click`
- `Right click`
- `Click at 500, 300`
- `Right click at 500, 300`
- `Clica em 500, 300`
- `Clique direito em 500, 300`

## Native Windows implementation

AURA uses:

- `SetCursorPos` for absolute pointer movement
- `SendInput` + `MOUSEINPUT` for click and wheel events
- `GetSystemMetrics` virtual-screen metrics to validate coordinates

This means coordinate validation supports the Windows virtual desktop, including layouts where a secondary monitor uses negative coordinates.

## Permissions

Mouse actions are classified by effect.

### Act

- move pointer
- scroll

These are reversible navigation actions and may execute immediately.

### Modify

- left click
- right click
- double click
- click at coordinates

A click can confirm dialogs, submit forms, activate controls or change application state, so the default permission policy routes clicks to `Waiting` until M003.9 adds explicit confirmation.

## Bounds and rate limits

- absolute coordinates must fall inside the Windows virtual desktop
- scroll actions are limited to 1–20 wheel notches per command
- malformed coordinates are rejected before execution

## Overlay

Immediate mouse actions submitted through the Overlay hide the Overlay before execution so it does not obstruct the target application.

Unlike keyboard input, moving the cursor itself does not require keyboard focus, but AURA keeps Overlay behaviour consistent across Computer Control actions.

## Safety scope

M003.5 does not implement:

- drag-and-drop
- click-by-text / click-by-visual-label
- automatic UI targeting
- repeated click loops
- arbitrary macros

Those capabilities require stronger context, Vision and permission controls.
