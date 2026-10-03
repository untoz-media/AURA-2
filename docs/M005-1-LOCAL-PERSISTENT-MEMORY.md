# M005.1 — Local Persistent Memory

M005.1 introduces the first persistent context layer in AURA-2.

The scope is deliberately explicit and user-controlled: AURA only stores a memory when the user asks it to remember something or adds an entry manually in the Memory screen.

Automatic context capture is not enabled in this milestone.

## Storage

Explicit memories are stored locally in AURA's application configuration directory as:

`memory.json`

No cloud service is required.

Each memory contains:

- stable local ID
- content
- source (`command` or `ui`)
- created timestamp
- updated timestamp

## Limits

- up to 1,000 explicit memories
- up to 2,000 characters per memory

Memory text is normalized for duplicate detection.

If the same memory is saved again with differences only in casing or repeated whitespace, AURA refreshes the existing record instead of creating a duplicate.

## Corruption safety

AURA fails closed if an existing `memory.json` cannot be parsed.

It does not silently treat an invalid file as an empty memory store, which prevents a later write from unintentionally overwriting corrupted-but-recoverable data.

A memory load failure is isolated from the rest of the desktop bootstrap so the main AURA application can still start.

## Natural-language commands

### Save

- `Remember that I prefer Brave`
- `Remember my default browser is Brave`
- `Lembra-te que uso Brave`
- `Guarda na memória ...`

Saving through natural language is classified as **Modify**.

Under the default AURA permission policy this requires confirmation.

### Read

- `What do you remember?`
- `List memories`
- `Show memories`
- `O que te lembras?`
- `Mostra as memórias`

Reading memory is classified as **Read**.

The command response returns the eight most recently updated memories and indicates when more entries exist.

### Forget

- `Forget that I prefer Brave`
- `Forget my default browser is Brave`
- `Esquece que uso Brave`
- `Apaga da memória ...`

Natural-language deletion requires an exact normalized text match and is classified as **Destructive**.

The default permission policy therefore requires confirmation.

## Memory screen

The existing **Memory** item in the desktop sidebar is now functional.

The page provides:

- saved-memory count
- manual memory creation
- 2,000-character counter
- local text search
- source indicator
- created / updated timestamps
- manual refresh
- explicit per-entry Forget action
- local-first / explicit-memory disclosure

Explicit UI actions directly represent user intent and therefore call the local memory API without routing through natural-language permissions.

## Bridge API

Tauri commands:

- `get_memories`
- `create_memory_command`
- `delete_memory_command`

Desktop bridge helpers:

- `getMemories`
- `createMemory`
- `deleteMemory`

The React bridge keeps the Memory screen synchronized after:

- app startup
- explicit UI create/delete operations
- completed AURA Core commands

## Permission model

| Operation | Permission |
| --- | --- |
| List / read memory | Read |
| Save explicit memory | Modify |
| Forget explicit memory by command | Destructive |

M005.1 does not infer facts from conversations, files, apps or activity.

## Scope boundary

The following remain separate milestones:

- **M005.2** — Current app awareness
- **M005.3** — Active window context
- **M005.4** — Recent files context
- **M005.5** — User-defined routines
- **M005.6** — Project memory

Those later milestones may consume the memory/context foundation, but M005.1 remains an explicit persistent-memory store.

## Validation

Regression coverage includes:

- memory content normalization
- content length validation
- duplicate-safe IDs
- English and Portuguese routing
- Read / Modify / Destructive permission boundaries
- permission overrides
- fail-closed storage reads

The repository's GitHub Actions runner provisioning issue still prevents the Windows workflow from reaching its first step.
