# M003.7 — System Commands

M003.7 adds deterministic Windows-level commands to AURA-2.

## Commands available now

Read-only:

- `System status`
- `PC status`
- `Battery status`
- `Estado do sistema`
- `Estado da bateria`

Immediate reversible actions:

- `Show desktop`
- `Open Task Manager`
- `Open Settings`
- `Open Display Settings`
- `Open Bluetooth Settings`
- `Open Network Settings`
- `Open Sound Settings`

High-impact actions routed for confirmation:

- `Lock PC`
- `Sleep PC`
- `Restart PC`
- `Shut down PC`

## Native implementation

AURA uses fixed native Windows paths/APIs:

- `GlobalMemoryStatusEx` for physical-memory state
- `GetTickCount64` for system uptime
- `GetSystemPowerStatus` for AC/battery state
- `LockWorkStation` for session locking
- `SetSuspendState` for sleep
- `taskmgr.exe` for Task Manager
- `explorer.exe ms-settings:...` for known Windows Settings pages
- `shutdown.exe` with fixed arguments for restart/shutdown
- the existing keyboard controller for Win+D / Show Desktop

No user-provided executable names or command-line arguments reach these system actions.

## Permissions

- system/battery status → `Read`
- Show Desktop → `Act`
- open Task Manager / Settings pages → `Act`
- Lock PC → `Sensitive`
- Sleep PC → `Sensitive`
- Restart PC → `Destructive`
- Shut down PC → `Destructive`

Under the current default policy, Read and Act execute immediately. Sensitive and Destructive actions enter `Waiting` and will become executable only through the M003.9 confirmation flow.

## Status response

The system-status response can include:

- battery percentage when available
- charging state
- AC power state
- physical RAM usage
- uptime

Desktop PCs without a battery are handled explicitly rather than reporting a fake percentage.

## Safety

Restart/shutdown arguments are fixed in code; AURA does not construct arbitrary `shutdown.exe` arguments from the user's text.

Bluetooth/Wi-Fi toggling is intentionally not included here. Opening the relevant Settings surface is safe and deterministic; direct radio toggling requires a separate permissions and device-control design.
