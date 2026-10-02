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
