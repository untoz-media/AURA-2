# M003.8 — Permission Engine

M003.8 replaces AURA's hard-coded permission defaults with a persistent, user-configurable policy engine.

## Permission classes

AURA keeps five permission classes:

- **Read** — inspect permitted local/system information
- **Act** — reversible actions such as opening apps, switching windows or changing volume
- **Modify** — actions that may change app state or content
- **Sensitive** — session/privacy-sensitive actions
- **Destructive** — actions with potentially significant loss or interruption

## Decisions

Each class can resolve to:

- **Allow** — execute immediately
- **Ask** — queue for explicit confirmation
- **Never** — block the action

## Safety floor

Read, Act and Modify can be configured as Allow, Ask or Never.

Sensitive and Destructive intentionally support only:

- Ask
- Never

AURA rejects attempts to permanently set these classes to Allow.

Persisted policy data is sanitized on load, so an edited/corrupt configuration cannot bypass this safety floor.

## Defaults

```text
Read         → Allow
Act          → Allow
Modify       → Ask
Sensitive    → Ask
Destructive  → Ask
```

## Persistence

The policy is stored locally in the Tauri app configuration directory as:

`permission-policy.json`

No permission policy data is sent to a cloud service.

## Core API

The desktop bridge exposes:

- `get_permission_policy`
- `set_permission_decision`
- `reset_permission_policy`

The Action Router receives a snapshot of the current persisted policy for every command.

## Settings

Settings → Permissions is now functional.

Changes are written to disk immediately and become authoritative for subsequent commands.

## M003.9 dependency

An `Ask` decision does not yet execute the action.

M003.9 will add the pending-action store and explicit confirmation UI that can approve or reject an individual action.
