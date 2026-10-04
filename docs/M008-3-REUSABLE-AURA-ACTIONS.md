# M008.3 — Reusable AURA Actions

M008.3 introduces Saved AURA Actions.

Saved Actions are small trusted building blocks that can be reused by:

- users
- Agent plans
- automations

## Supported Action shapes

A saved Action can wrap one supported Agent step:

- launch application
- switch to application
- run user routine
- run Director Mode preset
- wait

Actions have:

- ID
- name
- description
- aliases
- typed step
- updated timestamp

## Safety

Saved Actions do not store arbitrary code.

AURA validates referenced apps, routines and presets before saving.

An Action cannot be deleted while an automation still references it.

Manual Action execution still respects the current AURA permission policy.
