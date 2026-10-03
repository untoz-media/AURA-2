# M005.5 — User-defined Routines

M005.5 adds persistent, user-defined multi-step routines to AURA.

The goal is to let users teach AURA repeatable workflows without giving the language model unrestricted control of the PC.

## Architecture

A routine is an explicit list of deterministic AURA actions stored locally in:

`<AppConfig>/routines.json`

The local model is not the executor.

A model may later help interpret natural language or propose a routine, but the saved routine itself remains a structured list of auditable actions executed by AURA Core.

## Supported steps

Initial routine actions:

- Launch app
- Switch to app
- Run Director Mode preset
- Wait

Supported app targets reuse the existing M003 safe application catalog:

- OBS Studio
- Brave
- Google Chrome
- File Explorer
- Windows Terminal
- Notepad
- Calculator

This keeps M005.5 on top of already validated computer-control capabilities instead of introducing arbitrary executable paths.

## Persistence limits

- maximum 64 routines
- maximum 24 steps per routine
- maximum 12 aliases per routine
- name length: 64 characters
- description length: 240 characters
- wait steps: maximum 30 seconds

Routine names and aliases are unique case-insensitively.

Invalid/corrupt routine files fail closed rather than being silently replaced.

## Natural-language execution

Saved routine names and aliases can be run directly from Chat.

Explicit forms include:

- `Run routine Start Editing`
- `Execute routine Prepare Live`
- `Start routine Editing Mode`
- `Executa a rotina Preparar Live`
- `Inicia rotina Edição`

A routine alias may also resolve directly when the normal deterministic router has no stronger match.

Existing explicit computer and OBS commands keep priority.

## Permissions

Normal routines are classified as **Act**.

If a routine contains a Director Mode preset that can start streaming, the routine is dynamically classified as **Sensitive**.

AURA revalidates sensitivity immediately before execution.

If a routine was routed as Act and is edited to contain a Sensitive action before execution, AURA aborts and requires the command to be submitted again so the correct confirmation can be shown.

The Tasks UI does not bypass this boundary. A Sensitive routine must be run through Chat so the normal confirmation flow can apply.

## Execution behavior

Routines run sequentially and fail fast.

- steps execute in saved order
- the first real failure stops execution
- completed actions are not automatically rolled back
- each step records applied/failed status
- the final result records completed steps, failed step and error

Automatic rollback is deliberately avoided because application launches and production actions can have side effects that are unsafe to guess.

## Tasks workspace

The previous Tasks placeholder now includes a real **AURA Routines** workspace.

Users can:

- create routines
- edit routines
- delete routines
- add aliases
- add/reorder/remove steps
- run non-Sensitive routines directly
- refresh the local routine library
- inspect the last run result

Scheduling and autonomous long-running background jobs remain scoped to M008.

## Open-source model boundary

The routine engine is model-agnostic.

AURA-1 currently uses the local Qwen-based model path, but AURA-2 can later use other open-source models for:

- natural-language intent understanding
- plan proposals
- tool selection suggestions
- coding/reasoning
- vision

The deterministic routine representation remains the safety boundary regardless of which local model is selected.

## Roadmap

- M005.1 ✅ Local persistent memory
- M005.2 ✅ Current app awareness
- M005.3 ✅ Active window context
- M005.4 ✅ Recent files context
- M005.5 ✅ User-defined routines
- M005.6 Project memory
