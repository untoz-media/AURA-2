# M008.7 — Failure recovery

M008.7 adds bounded failure recovery without blindly replaying side effects.

## Retry policy

AURA automatically retries only steps that are safe enough to repeat:

- launchApp
- switchToApp

They receive at most one retry.

AURA does NOT automatically retry:

- user routines
- Director Mode presets
- saved composite Actions
- arbitrary side-effecting steps

This avoids repeating a workflow that may have partially succeeded before reporting an error.

## Failure behavior

Execution is fail-fast.

When a non-recoverable step fails:

- the failed step is recorded
- the Agent run becomes failed
- later steps are not executed
- the error remains in local run history

## Crash/interruption recovery

Run progress is persisted after state changes.

A non-terminal persisted run with no live runtime is presented as interrupted after restart.

## Pause and cancellation

Global AURA Pause suspends Agents and Automations.

Scheduled automations that become due during Pause remain pending instead of being consumed as failures.
