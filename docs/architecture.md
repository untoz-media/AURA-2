# AURA-2 Architecture

## Goal

AURA-2 is designed as an intelligent control layer for the user's computer rather than a chatbot application.

The architecture should separate reasoning from execution so that deterministic system actions do not depend on a language model when they do not need to.

## High-level architecture

```text
Desktop UI / Overlay / Voice
           │
           ▼
      AURA Core
           │
    ┌──────┼───────────┐
    ▼      ▼           ▼
 Context  Router   Permissions
    │      │           │
    └──────┼───────────┘
           ▼
      Action Layer
           │
 ┌─────────┼───────────────┐
 ▼         ▼               ▼
Computer   Skills       Automations
Control    / Apps
           │
           ├── OBS
           ├── Browser
           ├── Files
           ├── Drop Intake
           └── Future integrations

Optional intelligence services:

- Local language model
- Local speech models
- Local / hybrid vision
- Optional cloud models
```

## Routing principle

AURA should always prefer the cheapest and most deterministic path.

Examples:

- "Set volume to 40%" → direct system action
- "Open OBS" → app launcher
- "Switch to Match scene" → OBS skill
- "Find the image I used yesterday" → file intelligence + context
- drag local files → bounded Drop Intake → explicit Inspect / Reveal / Vision handoff
- "Prepare my stream" → agent / action workflow
- "What is wrong with this window?" → vision + reasoning

## Permission model

Every tool should declare its risk category.

Suggested levels:

- READ
- ACT
- MODIFY
- DESTRUCTIVE
- SENSITIVE

Users should be able to choose whether an action is always allowed, always blocked, or requires confirmation.

## OBS integration

OBS should use the OBS WebSocket API whenever possible.

Visual computer control should only be used as a fallback when an integration does not expose the required functionality.

## Local-first

Local execution remains the default architecture.

Cloud services may be used optionally for tasks where they provide a substantial capability improvement, but AURA should remain useful without requiring constant cloud access.


## Drag & Drop intake boundary

Native drag/drop is treated as temporary local context, not as implicit permission to execute or analyze content.

The frontend forwards native dropped paths to a Core intake command once. Core canonicalizes accepted files and stores their real paths only in an in-memory registry keyed by opaque drop IDs. React receives only safe metadata and capability flags.

Actions on dropped files resolve the opaque ID back inside Core.

Inspect is a Read-class action. Only allowlisted UTF-8 text/code formats may return a bounded preview (64 KiB / 12,000 characters), while image inspection may return dimensions. Complex binary formats remain metadata-only. Inspect never returns the canonical path.

Chat attachments and Analyze with AURA share a separate ephemeral context-attachment path. The frontend can bind selected opaque drop IDs to exactly one desktop Chat turn. Core resolves and re-inspects them only when that message is submitted, then builds a model-only context capped at 6,000 characters. This context is not placed inside the visible user message and is not persisted in model conversation history. After an accepted send, the frontend detaches the IDs for the next turn while keeping the temporary drop session available.

Attachment requests bypass deterministic action routing, user Routines and Director presets. Attached text is marked as untrusted data before it reaches the model, providing a hard product boundary between file content and computer actions.

Image handoff to Vision always creates a cache copy first so Vision cleanup cannot delete the user's original file.
