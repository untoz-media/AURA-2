# M003.3 — Window Discovery & Switching

M003.3 gives AURA direct awareness of visible Windows desktop windows.

## Win32 path

AURA now uses native Win32 APIs to:

- enumerate top-level windows
- ignore invisible/titleless windows
- read window titles
- resolve owning process IDs
- resolve executable names where permitted
- restore minimized windows
- request foreground activation

Core APIs:

- `EnumWindows`
- `GetWindowTextW`
- `GetWindowThreadProcessId`
- `QueryFullProcessImageNameW`
- `IsIconic`
- `ShowWindow(SW_RESTORE)`
- `SetForegroundWindow`

## Commands

Switching:

- `Switch to OBS`
- `Focus Brave`
- `Go to Chrome`
- `Vai para o OBS`
- `Muda para o Brave`
- `Troca para o Terminal`

Discovery:

- `List windows`
- `What windows are open?`
- `Lista as janelas`
- `Que janelas estão abertas?`

## Matching

AURA first matches a visible window by the known executable image associated with an `AppTarget`.

If process information cannot be read, AURA falls back to conservative window-title hints.

This allows the window manager to reuse the same application identity used by launching and lifecycle control.

## Permissions

- list visible windows → `Read`
- switch/focus a known app window → `Act`

Both are allowed by the current default policy.

## Windows foreground restrictions

Windows intentionally restricts when an application may force another window into the foreground.

AURA calls `SetForegroundWindow` and treats a false return as a normal controlled failure. It does not bypass Windows focus-stealing protections.

## Scope

M003.3 does not implement commands like `minimize this window`.

When AURA Overlay has focus, “this window” is not necessarily the user's previous application. That will be implemented after AURA has explicit foreground-context tracking rather than guessing.
