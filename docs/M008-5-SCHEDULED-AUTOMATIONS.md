# M008.5 — Scheduled automations

M008.5 adds persistent local scheduling.

## Trigger types

Automations can run:

- at AURA startup
- every N minutes
- once at a future date/time
- repeatedly every 24 hours after a selected date/time
- when a supported app receives focus

Interval range:

- minimum: 1 minute
- maximum: 7 days

## Persistence

Automations are stored locally in:

`aura-automations.json`

Each automation stores:

- enabled state
- referenced saved Action
- trigger
- last run time
- next scheduled run
- last result
- update timestamp

The scheduler checks local state in the AURA background process.

No external automation service is required.
