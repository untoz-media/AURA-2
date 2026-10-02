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
