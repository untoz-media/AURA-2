# M005.6 — Project Memory

M005.6 completes the first AURA-2 Memory & Context milestone with structured, project-scoped memory.

## Why project memory exists

General Memory is for user-level facts such as preferences.

Project Memory is for context that belongs to one specific project:

- project name and aliases
- description
- project notes
- linked AURA routines
- explicit active-project selection

This prevents unrelated project context from being mixed into global personal memory.

## Persistence

Project memory is stored locally in:

`<AppConfig>/projects.json`

The file contains:

- saved projects
- the currently active project ID

No project memory is synchronized to a cloud service.

## Project shape

Each project contains:

- ID
- name
- description
- aliases
- notes
- linked routine IDs
- created timestamp
- updated timestamp

Limits:

- 128 projects
- 12 aliases per project
- 64 notes per project
- 32 linked routines per project
- 80-character name
- 600-character description
- 500 characters per note

Names and aliases are unique case-insensitively.

## Active project

AURA has at most one explicit active project.

Selecting a project does not open files or applications. It only changes the project context supplied to AURA.

Clearing the active project removes project-specific context from future local-model turns.

## Local model context

The selected active project is appended to the same ephemeral per-turn desktop context used for:

- current app
- active window
- recent Windows items

The active project can provide:

- project name
- description
- notes
- linked routine IDs

This context is not added to the user message and is not stored as a model conversation turn.

Changing projects therefore updates future context without polluting conversation history.

## Memory workspace

The existing Memory page now contains a Project Memory workspace.

Users can:

- create projects
- edit projects
- delete projects
- select/clear the active project
- add aliases
- add project notes
- link saved AURA routines
- inspect project counts and active state

Deleting the active project automatically clears active-project selection.

## Safety/privacy boundary

Project Memory does not automatically:

- scan project folders
- index file contents
- open linked resources
- infer projects from browsing history
- upload project data
- execute linked routines

Routine links are references only. Execution remains explicit and permission-controlled.

## Release

Completing M005 advances the desktop release line to:

`AURA-2 0.5.0-alpha.1`

M005 status:

- M005.1 ✅ Local persistent memory
- M005.2 ✅ Current app awareness
- M005.3 ✅ Active window context
- M005.4 ✅ Recent files context
- M005.5 ✅ User-defined routines
- M005.6 ✅ Project memory

The next roadmap milestone is M006 — Voice.
