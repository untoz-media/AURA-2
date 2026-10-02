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
