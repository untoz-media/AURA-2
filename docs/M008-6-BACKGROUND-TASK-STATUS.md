# M008.6 — Background task status

M008.6 makes Agent and Automation activity visible.

## Agent status

The Agents workspace shows:

- active run count
- current run state
- current step
- completed / total steps
- progress bar
- pause/resume controls
- cancel control
- recent completed/failed/cancelled/interrupted runs

Agent state changes are emitted through:

`aura:agent-event`

## Automation status

Automation trigger results are emitted through:

`aura:automation-event`

The UI shows the latest automation result and persists each automation's last result.

## Persistence

Agent progress is persisted during execution, not only after completion.

If AURA closes before an active run finishes, the next startup presents that run as:

`interrupted`

instead of silently losing it.
