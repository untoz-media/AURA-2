# AURA-2

<p align="center">
  <strong>Your PC. Now it understands you.</strong>
</p>

**AURA-2** is the next generation of AURA, a local-first personal AI assistant developed by **Untoz**.

AURA is not designed to compete with general-purpose AI chatbots. Its purpose is different: to become an intelligent layer for your computer — able to understand context, control applications, automate workflows, assist with production tasks, and act on your behalf with explicit permissions.

> **Current stage:** M004 OBS Control / Director Mode in progress · AURA-2 0.3.0-alpha.1 · pre-Beta

---

## Vision

AURA-2 is being built around a simple idea:

> **Do less clicking. Tell your computer what you want done.**

Instead of only answering questions, AURA-2 is designed to:

- Understand what is happening on your computer
- Control Windows and supported applications
- Work with files and local context
- Listen and respond through voice
- See the screen when permission is granted
- Run reusable actions and automations
- Assist with complex workflows such as OBS productions
- Keep local execution at the center of the experience

---

## Core pillars

### Vision
AURA can understand the screen and use visual context when authorized.

### Voice
Natural voice interaction with optional wake-word support.

### Computer Control
Open apps, switch windows, type, click, manage system settings and execute actions.

### Director Mode
Deep OBS integration for live production workflows, scene switching, recording, audio monitoring and production automation.

---

## Planned modules

```text
AURA-2/
├── apps/
│   └── desktop/
├── aura/
│   ├── core/
│   ├── models/
│   ├── memory/
│   ├── tools/
│   ├── vision/
│   ├── voice/
│   ├── computer/
│   ├── agents/
│   ├── automations/
│   ├── permissions/
│   └── integrations/
│       └── obs/
├── docs/
├── tests/
├── assets/
└── README.md
```

---

## Product principles

- **Local-first** by default
- **Action-oriented**, not chatbot-first
- **Fast commands should stay fast**
- **LLMs should not be required for every action**
- **Permissions must be explicit and understandable**
- **Irreversible actions require confirmation**
- **Modular architecture**
- **Windows-first**
- **Cloud assistance should remain optional where possible**

---

## Planned AURA-2 capabilities

- AURA Overlay
- Context Awareness
- AURA Vision
- AURA Voice
- Computer Control
- App Skills
- File Intelligence
- Clipboard Intelligence
- Drag & Drop actions
- AURA Memory
- Agent Mode
- AURA Automations
- Background Tasks
- Proactive system alerts
- Permissions & Safety controls
- Local + optional cloud model routing
- Instant Commands
- OBS Control
- Director Mode
- Plugin / Skill architecture

---

## Development stages

### M001 — Project Initialization
Repository, architecture, documentation and development foundations.

### M002 — Desktop Foundation ✅
Native Windows app, Core bridge, tray, Overlay, Settings, background mode, autostart and packaged NSIS build.

### M003 — Computer Control ✅
Deterministic Windows control for applications, windows, keyboard, mouse, audio/media and system actions, with persistent permissions and one-shot confirmations.

### M004 — OBS Integration 🚧
OBS WebSocket v5 connection and live production-state detection are implemented; scene discovery and Director Mode controls are next.

### M005 — Memory & Context
Persistent local memory, app context and system awareness.

### M006 — Voice
Speech input, speech output and low-latency interaction.

### M007 — Vision
Screen understanding and permission-based visual context.

### M008 — Agents & Automations
Multi-step tasks, reusable actions and event-based workflows.

### M009 — AURA-2 Beta
Public testing release.

### M010 — AURA-2
Stable next-generation release.

---

## AURA-1

AURA-2 builds on the lessons of the first generation while introducing a substantially different product architecture.

The original project remains available at:

https://github.com/untoz-media/AURA-1

---

## Status

AURA-2 has completed **M003 — Computer Control** and has started **M004 — OBS Control / Director Mode** with the OBS WebSocket connection foundation.

APIs, architecture, features, compatibility and product behaviour may change significantly before Beta.

---

## About Untoz

AURA is developed by **Untoz** as part of its technology and AI projects.
