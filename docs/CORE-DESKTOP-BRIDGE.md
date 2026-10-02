# AURA-2 Core ↔ Desktop Bridge

**Milestone:** M002.4  
**Status:** Initial bridge complete

This layer connects the React desktop UI with the Rust/Tauri core.

## Design

AURA uses two IPC patterns:

1. **Commands** for request/response operations.
2. **Events** for lifecycle updates, state changes, progress and errors.

This follows Tauri's intended separation: commands provide a typed request/response path while events provide asynchronous updates.

## Frontend → Core

The frontend calls:

- `get_app_status`
- `process_user_command`

The command request contains:

```ts
{
  text: string;
  source: "desktop" | "overlay" | "voice";
}
```

The core immediately returns a command acknowledgement containing an ID and current status.

## Core → Frontend

The Rust core emits:

- `aura:core-event`
- `aura:core-error`

A core event contains:

- command/event ID
- event kind
- AURA runtime status
- human-readable activity message
- original command when relevant
- timestamp

## Current lifecycle

The M002.4 prototype proves the complete round trip:

```text
React command input
       │
       ▼
Tauri invoke()
       │
       ▼
Rust process_user_command
       │
       ├── command.accepted → Thinking
       ├── command.processing → Working
       └── command.completed → Idle
       │
       ▼
Tauri events
       │
       ▼
React status + activity UI
```

No Windows actions are executed yet. M003 will replace the prototype processing stage with the real Action Router and Computer Control tools.

## Source files

Frontend:

- `src/bridge/types.ts`
- `src/bridge/aura.ts`
- `src/bridge/useAuraBridge.ts`

Backend:

- `src-tauri/src/lib.rs`

## Rules for future modules

- UI components do not call platform APIs directly.
- System actions go through the AURA Core.
- Every action gets an ID.
- Long-running work reports lifecycle state.
- Failures use structured error codes.
- Destructive or sensitive actions will pass through the permissions layer before execution.
