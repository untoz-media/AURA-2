# M005.2 — Current App Awareness

M005.2 adds a live, local snapshot of the application the user is currently working in.

This milestone intentionally does **not** capture the active window title, document name, browser tab title or other window text. That richer context remains scoped to **M005.3 — Active Window Context**.

## Windows detection

AURA uses native Windows APIs:

- `GetForegroundWindow`
- `GetWindowThreadProcessId`
- `OpenProcess`
- `QueryFullProcessImageNameW`

The result includes:

- friendly app name
- process image name
- process ID
- whether the process maps to an AURA-known application
- context source
- capture timestamp

## Known applications

Existing M003 application metadata is reused.

Known process images currently map to:

- OBS Studio
- Brave
- Google Chrome
- File Explorer
- Windows Terminal
- Notepad
- Calculator

Unknown applications are still supported. AURA falls back to a readable name derived from the executable image.

Example:

`Discord.exe` → `Discord`

## AURA focus protection

Opening the AURA desktop window or overlay can itself become the Windows foreground process.

To preserve useful user context, M005.2 keeps the most recently observed **external** foreground application in memory.

When AURA itself owns foreground focus:

- if a previous external app exists, AURA returns that app with `contextSource = lastExternal`;
- otherwise AURA reports its own foreground process normally.

This cache is in RAM only.

It is not written to persistent memory and is not an app-usage history.

## Polling

The desktop bridge refreshes current-app context once per second while AURA is not paused.

When AURA is paused:

- current-app polling stops;
- the last valid snapshot remains visible;
- Home labels awareness as paused.

No timeline or historical record is stored.

## Natural-language commands

Examples:

- `What app am I using?`
- `Current app`
- `Which application is active?`
- `Que app estou a usar?`
- `Que aplicação estou a usar?`
- `Em que app estou?`

Current-app awareness is classified as **Read**.

## Desktop UI

Home now shows a **Current app** block inside Current Activity.

It displays:

- friendly app name
- process image
- `Foreground`, `Last external context`, or paused/waiting status

## Scope boundary

M005.2 knows **which application** is current.

It deliberately does not expose:

- window title
- document title
- browser tab title
- file path inferred from window text
- UI contents

Those belong to:

**M005.3 — Active Window Context**

## Validation

Regression coverage includes:

- known process-image matching
- generic executable-name fallback
- English and Portuguese current-app routing
- Read permission classification
- Read policy overrides
- distinct foreground / last-external context source

The repository's GitHub Actions runner provisioning issue still requires the real Windows build gate before merge.
