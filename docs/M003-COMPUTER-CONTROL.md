# M003 — Computer Control

AURA-2 now moves from desktop infrastructure into real Windows actions.

## M003 sub-milestones

- **M003.1 — Action Router & App Launching**
- **M003.2 — Application Lifecycle (close apps)**
- **M003.3 — Window Discovery & Switching**
- **M003.4 — Keyboard Actions**
- **M003.5 — Mouse Actions**
- **M003.6 — Volume & Media Controls**
- **M003.7 — System Commands**
- **M003.8 — Permission Engine**
- **M003.9 — Confirmation Flow**
- **M003.10 — Computer Control validation**

## M003.1

M003.1 introduces the first deterministic AURA action path.

```text
"Open OBS"
    ↓
Command Router
    ↓
ActionIntent::LaunchApp(ObsStudio)
    ↓
PermissionClass::Act
    ↓
PermissionPolicy
    ↓
AppLauncher
    ↓
OBS Studio
```

No language model is required for this flow.

### Supported commands

English examples:

- `Open OBS`
- `Launch Brave`
- `Start Chrome`
- `Open File Explorer`
- `Open Windows Terminal`
- `Open Notepad`
- `Open Calculator`

Portuguese examples:

- `Abre o OBS`
- `Abrir Brave`
- `Abre o Explorador de Ficheiros`
- `Abre o Terminal`
- `Abre o Bloco de Notas`
- `Abre a Calculadora`

### Safety

Application launching uses `std::process::Command` directly.

AURA does not construct a shell command from user text and does not pass arbitrary input to `cmd.exe` or PowerShell.

Only known application targets can be launched by M003.1.

### Permissions

M003.1 also creates the first internal permission policy.

Default policy:

- Read → Allow
- Act → Allow
- Modify → Ask
- Destructive → Ask
- Sensitive → Ask

Only the `Act` class is executed in M003.1. The confirmation UI for `Ask` decisions arrives later in M003.


## M003.2

M003.2 adds application lifecycle control for known app targets.

Close commands are deterministic and bilingual, but are classified as `PermissionClass::Modify`. Under the default permission policy this produces a `Waiting` state until M003.9 provides the explicit confirmation UI.

The Windows lifecycle implementation is intentionally conservative:

- known executable image names only
- no arbitrary PIDs
- no wildcard targets
- no shell interpolation
- no PowerShell or cmd.exe
- no forced `/F` termination
- File Explorer is protected from generic termination

The close engine is implemented in `computer/app_lifecycle.rs`.


## M003.3

M003.3 adds native top-level window discovery and foreground switching.

AURA can now enumerate visible titled windows, associate them with known application targets and request focus changes without simulating Alt+Tab.

Examples:

- `Switch to OBS`
- `Vai para o Brave`
- `List windows`
- `Que janelas estão abertas?`

Window listing is classified as `Read`. Switching to a known application window is classified as `Act`.

AURA respects Windows foreground restrictions: if `SetForegroundWindow` is denied, the action fails cleanly rather than attempting to bypass OS focus protections.


## M003.4

M003.4 adds controlled native Windows keyboard synthesis using `SendInput`.

AURA understands key/shortcut commands such as `Press F11`, `Press Escape` and `Press Ctrl+S`, plus text intents such as `Type "Hello"`.

Permission classification is intentionally conservative:

- navigation-only keys → `Act`
- shortcuts, letters, Enter, Backspace and typed text → `Modify`
- Delete → `Destructive`

With the default policy, only navigation-only keys execute immediately. Immediate injection is restricted to commands submitted through the Overlay, which is hidden before `SendInput` runs so focus can return to the user's previous application.

Text is sent through Unicode keyboard events, capped at 500 characters and control characters are rejected.


## M003.5

M003.5 adds controlled native Windows pointer and mouse-wheel actions.

AURA can parse absolute cursor movement, bounded scrolling and click intents. Coordinates are validated against the Windows virtual desktop so multi-monitor layouts with negative coordinates are supported.

Permission classification:

- pointer movement → `Act`
- scrolling → `Act`
- left/right/double click → `Modify`
- click at coordinates → `Modify`

Under the current default policy, movement and scrolling can execute immediately. Click actions stay in `Waiting` until M003.9 provides explicit confirmation.

The implementation uses `SetCursorPos`, `GetSystemMetrics` and `SendInput/MOUSEINPUT`; no click loops, drag-and-drop or visual auto-targeting are included in M003.5.


## M003.6

M003.6 adds bidirectional Windows audio state and global media controls.

Exact volume and mute state use the Windows Core Audio default render endpoint through `IMMDeviceEnumerator` and `IAudioEndpointVolume`. AURA can read the current master level, set an exact 0–100% volume, step volume up/down and mute/unmute.

Global Play/Pause, Next, Previous and Stop actions use Windows media virtual keys through `SendInput`.

All media/volume mutations are classified as `Act`; volume-state reads are `Read`.


## M003.7

M003.7 adds deterministic Windows system commands and read-only PC status.

AURA can now read RAM usage, uptime and power/battery state; show the desktop; open Task Manager; and open known Windows Settings pages.

High-impact system actions are also routed now:

- Lock → `Sensitive`
- Sleep → `Sensitive`
- Restart → `Destructive`
- Shutdown → `Destructive`

The current default policy keeps all of those in `Waiting` until the M003.9 confirmation flow is implemented.

System status uses native Win32 APIs including `GlobalMemoryStatusEx`, `GetTickCount64` and `GetSystemPowerStatus`. Known Settings pages use fixed `ms-settings:` URIs and no arbitrary command-line arguments are accepted.


## M003.8

M003.8 turns the initial static permission defaults into a persistent runtime policy engine.

Settings → Permissions can now configure Read, Act and Modify as `Allow`, `Ask` or `Never`. Sensitive and Destructive actions intentionally expose only `Ask` and `Never`; permanent Allow is rejected by the Core even if an invalid configuration is injected manually.

The policy is stored locally as `permission-policy.json` and loaded at startup. Every command is routed against a snapshot of the current runtime policy rather than `PermissionPolicy::default()`.

The bridge exposes policy read/update/reset commands. M003.9 will consume `Ask` decisions through a real pending-action confirmation workflow.


## M003.9

M003.9 adds explicit one-shot confirmation for actions whose permission decision is `Ask`.

When the Action Router resolves to `Ask`, AURA now:

1. creates a pending confirmation associated with the command ID, command text, source and permission class
2. stores it only in runtime memory
3. enters `Waiting`
4. renders an Allow / Cancel confirmation card in the main app and Overlay
5. executes the original deterministic action only after explicit approval

Confirmations expire after 60 seconds and are single-use. Approval IDs cannot be replayed.

Before executing an approved action, AURA routes the command again against the current permission policy. If the policy has changed to `Never`, the action remains blocked.

Only one pending confirmation is retained per command source. A newer request replaces the older pending request from that same source.

Cancellation removes the pending action and returns AURA to `Idle`.

The confirmation flow does not permanently change permission settings; it approves one specific action only.


## M003.10

M003.10 closes the Computer Control milestone with a dedicated validation suite and Windows packaging gate.

Automated validation now covers:

- English and Portuguese deterministic command routing
- permission-class regressions across Read / Act / Modify / Sensitive / Destructive
- persistent policy overrides
- fail-closed handling for invalid volume, mouse, keyboard and unknown-app input
- Unicode text intent preservation
- confirmation expiry and command/source binding
- the safety floor that prevents Sensitive or Destructive from becoming permanent Allow

The Windows CI gate runs Rust Core tests before the frontend/Tauri build and NSIS packaging.

A manual hardware checklist is documented in `docs/M003-10-VALIDATION.md` for behaviours that should not be executed automatically in CI, including actual window focus, multi-monitor pointer movement, audio endpoint changes and power/session actions.

**M003 release marker:** `AURA-2 0.3.0-alpha.1`

With M003 complete, AURA-2 has a deterministic, permission-aware Windows Computer Control foundation. M004 moves into direct OBS WebSocket control and Director Mode.
