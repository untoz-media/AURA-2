# Desktop UI Evolution — AURA-1 → AURA-2

This work keeps AURA-2 visually and structurally connected to the AURA-1 desktop experience instead of turning the new assistant into an unrelated product.

## Design direction

The AURA-1 application established the core desktop identity:

- dark local-first workspace
- left sidebar
- AURA Core / orb
- central assistant welcome
- quick prompts
- bottom composer
- computer/model status
- local-capability chips
- blue/violet AURA spectrum

AURA-2 preserves those ideas while moving the desktop implementation to the Tauri + React foundation built in M002–M005.

## New shell

Primary navigation now contains:

1. Chat
2. Memory
3. Models
4. Create
5. Computer
6. Tasks
7. Director Mode
8. Settings

The sidebar also contains:

- New conversation entry point
- runtime status card
- current-app context
- local/hybrid mode state
- local capability shortcuts
- local-first privacy message

## Chat

The Chat workspace returns to the AURA-1 interaction pattern:

- AURA Core centered in the workspace
- `AURA-2 BY UNTOZ`
- `How can I help?`
- `AI that lives on your computer.`
- quick-start cards
- compact current-context strip
- composer docked at the bottom

Existing AURA Core commands, confirmation cards and permission routing remain connected.

## Models

The Models workspace establishes the permanent Model Manager surface for:

- AURA-1
- AURA-2
- download state
- install progress
- verification
- activation
- updates
- removal

The current UI deliberately leaves Download disabled until the real downloader is implemented.

## Create

The Create workspace is prepared for:

- image generation
- video generation
- local engines when hardware supports them
- connected engines where required

Generate is deliberately disabled until an actual generation backend is connected.

## Computer

The Computer workspace already uses real AURA-2 state:

- current foreground app context
- AURA runtime state
- OBS connection/output state
- direct quick commands through the Core action router

## Director Mode

Director Mode now has a first-class workspace showing:

- OBS connection
- output state
- production health
- saved Director Mode presets
- explicit preset execution
- path into full OBS settings

## Tasks

Tasks is now a permanent navigation destination for the future M008 agent/task system.

No fake background task runtime is exposed before that backend exists.

## Themes

Appearance now supports persistent themes:

- **AURA** — primary blue/violet identity
- **Midnight** — deeper navy treatment
- **OLED** — true-black surfaces
- **Aurora** — brighter cyan/blue/violet spectrum
- **Light** — bright UI preserving AURA accents

Theme state is stored locally and initialized before the main or overlay React view renders.

## Scope boundary

This foundation changes desktop information architecture and visual identity only.

It does not yet implement:

- AURA model downloads
- image-generation engines
- video-generation engines
- task/agent scheduling
- conversation-history persistence

Those capabilities will be added on top of this permanent shell rather than requiring another redesign.
