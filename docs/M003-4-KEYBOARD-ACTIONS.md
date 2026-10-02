# M003.4 — Keyboard Actions

M003.4 adds controlled keyboard synthesis to AURA-2 through the native Windows `SendInput` API.

## Supported command shapes

Shortcut/key actions:

- `Press F11`
- `Press Escape`
- `Press Ctrl+S`
- `Carrega em Tab`
- `Pressiona Alt+F4`

Text actions:

- `Type "Hello world"`
- `Write "Untoz"`
- `Escreve "Olá"`
- `Digita "AURA-2"`

## Supported shortcut vocabulary

Modifiers:

- Ctrl
- Shift
- Alt
- Win

Keys:

- A–Z
- 0–9
- Enter
- Escape
- Tab
- Space
- Backspace
- Delete
- Arrow keys
- Home / End
- PageUp / PageDown
- F1–F12

A shortcut may contain up to four unique modifiers plus one key.

## Permission classification

Keyboard synthesis is intentionally conservative.

### Act

Navigation-only keys without modifiers:

- Escape
- Tab
- arrows
- Home / End
- PageUp / PageDown
- F1–F12

### Modify

- typed text
- Enter
- Backspace
- Space
- letters/numbers
- any shortcut containing modifiers, including Ctrl+S and Alt+F4

### Destructive

- Delete

The default policy therefore executes only navigation-only keyboard actions immediately. Modify and Destructive actions remain in `Waiting` until M003.9 adds explicit confirmation.

## Overlay execution

Immediate keyboard injection is only permitted from the AURA Overlay in M003.4.

Before injection:

1. AURA hides the Overlay.
2. focus returns to the user's previous app.
3. AURA waits briefly for focus restoration.
4. `SendInput` injects the key events.

Commands submitted from the full desktop app are not injected into the focused AURA input field.

## Text safety

Text injection:

- is limited to 500 Unicode characters
- uses `KEYEVENTF_UNICODE`
- rejects control characters such as newline, Enter and Tab
- never invokes PowerShell, cmd.exe or a shell

## Windows security boundary

Windows may block `SendInput` when the target application has a higher integrity level. AURA treats a partial/zero input count as a controlled failure and does not attempt to bypass Windows UIPI protections.
