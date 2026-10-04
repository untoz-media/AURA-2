# AURA-2 0.8.0-alpha.1 — Agents & Automations

AURA-2 0.8.0-alpha.1 completes M008 — Agents & Automations.

## Agent planning

AURA can now receive a goal and ask the selected local assistant model for a bounded structured plan.

Planning is isolated from normal Chat history.

Plans are validated before execution and expose their highest permission class.

## Deterministic Agent execution

Agent plans execute through AURA Core, not through unrestricted model tool access.

Supported plan steps include:

- launch app
- switch app
- run routine
- run Director Mode preset
- wait
- run saved AURA Action

## Saved AURA Actions

Users can save trusted reusable actions and run them manually or use them as automation targets.

## Event triggers

Initial local triggers:

- AURA startup
- supported app focused

## Scheduled automations

Initial schedule types:

- interval
- future date/time
- optional 24-hour repeat

## Background safety

Automations execute saved Actions only.

Background execution requires the relevant permission class to be Allow.

Sensitive routines and Director presets cannot run unattended.

AURA Pause suspends the automation scheduler.

## Background task status

The Agents workspace shows live execution state, progress, current step and recent run history.

## Failure recovery

- safe launch/focus actions get one retry
- multi-step routines/presets are not blindly retried
- execution is fail-fast
- state is persisted during runs
- unfinished runs become Interrupted after restart
- Wait steps respond to pause/cancel

## Desktop workspace

New primary navigation item:

`Agents`

The workspace contains:

- local Planner
- execution status
- Saved AURA Actions
- Automations
- trigger configuration
- permission/safety messaging

## Release version

`0.8.0-alpha.1`

Next milestone:

`M009 — AURA-2 Beta`
