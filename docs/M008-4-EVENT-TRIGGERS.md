# M008.4 — Event triggers

M008.4 adds local event triggers for trusted automations.

## Startup trigger

A saved Action can run once when the current AURA process starts.

If AURA is paused at startup, the trigger remains pending until AURA is resumed.

## App Focused trigger

A saved Action can run when a supported external application becomes the Windows foreground process.

Supported application aliases use the same deterministic AppTarget registry as Computer Control.

The trigger fires on a focus transition rather than continuously while the app remains focused.

AURA ignores its own process for external app-focus detection.

## Execution rules

Event triggers execute saved Actions only.

They cannot ask an LLM to invent a new plan at trigger time.
