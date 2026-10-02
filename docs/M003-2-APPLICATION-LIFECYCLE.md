# M003.2 — Application Lifecycle

M003.2 adds safe application-closing support to the Computer Control layer.

## Commands

English:

- `Close OBS`
- `Close Brave`
- `Close Chrome`
- `Close Windows Terminal`
- `Close Notepad`
- `Close Calculator`

Portuguese:

- `Fecha o OBS`
- `Fechar Brave`
- `Fecha o Chrome`
- `Fecha o Terminal`
- `Fecha o Bloco de Notas`
- `Fecha a Calculadora`

## Routing

```text
"Close OBS"
    ↓
Action Router
    ↓
ActionIntent::CloseApp(ObsStudio)
    ↓
PermissionClass::Modify
    ↓
PermissionPolicy::Ask
```

Closing an application is classified as **Modify**, not Act, because it can affect application state or unsaved work.

Until the confirmation flow is implemented in M003.9, the default policy places close actions into the Waiting state instead of executing them automatically.

The actual lifecycle controller is implemented now and is ready to execute once a policy decision becomes Allow.

## Windows implementation

The controller uses two fixed Windows utilities:

- `tasklist.exe` to check whether a known image is running
- `taskkill.exe /IM <known-image>` to request termination

AURA does **not** use:

- `/F` forced termination
- user-provided process names
- arbitrary PIDs
- wildcard process names
- shell interpolation
- `cmd.exe`
- PowerShell

Only hard-coded process-image names associated with known `AppTarget` values can reach the lifecycle controller.

## Protected applications

**File Explorer is intentionally protected.**

AURA will not terminate `explorer.exe` through the generic application lifecycle tool because Explorer also hosts important Windows shell functionality.

A future Windows-specific system command may provide a separate, explicit shell-restart workflow if needed.

## Force close

Force termination is intentionally outside M003.2.

A future `Force close …` action should receive a higher permission level and explicit confirmation.
