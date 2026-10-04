# M008.1 — Multi-step task planning

M008.1 adds bounded local task planning for AURA Agents.

## Planner

The Agent planner uses the currently selected local assistant model through an isolated generation path.

Planning does not append messages to the normal Chat conversation.

The planner receives:

- the user goal
- supported Windows app aliases
- available user routines
- available Director Mode presets
- available saved AURA Actions
- a strict JSON schema
- a hard maximum of 12 steps

The model can only propose supported typed steps:

- launchApp
- switchToApp
- runRoutine
- directorPreset
- wait
- savedAction

Unsupported capabilities such as shell commands, arbitrary typing, file deletion, shutdown, purchases, messages and uploads are explicitly excluded from the Agent planner.

## Validation

Planner output is parsed as JSON and then revalidated by AURA Core.

The model cannot create a new executable capability merely by writing it in JSON.

A plan records:

- goal
- summary
- typed steps
- highest permission class
- whether confirmation is required
- whether current permissions block execution
- creation timestamp
