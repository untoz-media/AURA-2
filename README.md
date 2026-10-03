# AURA-2

<p align="center">
  <strong>Your PC. Now it understands you.</strong>
</p>

**AURA-2** is the next generation of AURA, a local-first personal AI assistant developed by **Untoz**.

AURA is not designed to compete with general-purpose AI chatbots. Its purpose is different: to become an intelligent layer for your computer — able to understand context, control applications, automate workflows, assist with production tasks, and act on your behalf with explicit permissions.

> **Current stage:** M005 Memory & Context in progress · AURA-2 0.4.0-alpha.1 · pre-Beta

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

### M004 — OBS Integration ✅
OBS WebSocket v5 connection, live state detection, scene/source/audio control, recording/streaming control, live duration, production health monitoring and persistent Director Mode presets are implemented.

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

AURA-2 has completed **M004 — OBS Control / Director Mode** as **0.4.0-alpha.1** and is progressing through **M005 — Memory & Context** with explicit local persistent memory, current-app awareness, active-window context, Windows Recent Items context and persistent user-defined routines, project-scoped memory with an explicit active project, and the local microphone foundation, global hold-to-talk capture path and on-device Whisper speech-to-text runtime for M006 Voice.

APIs, architecture, features, compatibility and product behaviour may change significantly before Beta.

---

## About Untoz

AURA is developed by **Untoz** as part of its technology and AI projects.


## AURA-1 → AURA-2 desktop evolution

The AURA-2 desktop app deliberately keeps the product identity and primary interaction model established by AURA-1 while moving the implementation to the newer Tauri + React architecture.

The evolved desktop shell keeps:

- the left navigation rail
- the AURA Core / orb identity
- a central `How can I help?` assistant surface
- a bottom composer
- local-computer status
- local capability shortcuts
- local-first / private-by-design messaging

AURA-2 extends that shell with permanent workspaces for:

- Memory
- Models
- Create
- Computer
- Tasks
- Director Mode
- Settings

The desktop also supports persistent visual themes:

- AURA
- Midnight
- OLED
- Aurora
- Light

Model downloads and media-generation engines are intentionally not simulated by the UI foundation. Their buttons remain disabled until the real backend engines are implemented.


### Model Manager

The evolved AURA-2 desktop now includes a real local Model Manager.

Current behavior:

- AURA-1 can be downloaded from its existing upstream runtime source
- large downloads support staging, pause, resume and cancel
- interrupted partial downloads can resume after restart
- available disk space is checked before transfer
- required runtime files are verified before installation is finalized
- installed models can be selected and removed
- selected-model state is persisted locally
- AURA-2 remains unavailable until a real checkpoint is defined

The selected verified model is now connected to free-form local chat through a persistent Python/Transformers runtime. Deterministic computer and OBS controls still route directly and do not depend on the LLM.


### Local Model Runtime

Free-form Chat now falls back to the selected verified local model when no deterministic AURA action matches.

Current runtime behavior:

- selected model loads on the first free-form prompt
- the Python/Transformers worker remains resident between prompts
- model loading uses `local_files_only=True`
- AURA-1 keeps its existing 4-bit BitsAndBytes profile when CUDA is available
- conversation context remains in RAM for the current conversation
- **New conversation** clears model context without unloading the model
- changing/removing the active model stops the old worker
- Settings exposes loading / ready / generating / error runtime state
- Chat now renders real user/AURA message history

The desktop now includes a Managed Runtime installer that can prepare a private Python/PyTorch/Transformers environment under AURA Local Data. A compatible system Python or AURA_PYTHON override remains available for development, but is no longer the intended end-user path.


### Managed Runtime

Local model users no longer need to configure Python manually.

From **Models**, AURA can install its own private Windows AI environment:

- Python 3.12.10 x64 from python.org
- Authenticode verification before installer execution
- private per-user TargetDir
- no PATH changes, launcher, file associations or shortcuts
- GPU-aware PyTorch installation
- CUDA 12.8 wheels when NVIDIA is detected
- CPU wheels otherwise
- Transformers / Accelerate / BitsAndBytes / Safetensors
- final import/version/CUDA verification
- repair and removal controls

The intended local setup is now:

`Install runtime → Download AURA-1 → Use model → Chat`

Model weights and the managed runtime are stored separately, so repairing/removing Python does not delete downloaded models.


### Managed AI Runtime

AURA can now prepare its own private Windows AI runtime from the Models workspace.

The managed setup:

- installs private Python 3.12 under AURA Local Data
- does not modify the user's PATH or system Python
- verifies the official Python installer with Windows Authenticode
- checks for at least 10 GB of free runtime space
- selects CUDA or CPU PyTorch based on NVIDIA detection
- installs Transformers, Accelerate, BitsAndBytes and Safetensors
- verifies the final Python/AI stack before marking it Ready
- exposes install, repair, reinstall and removal controls in the desktop UI

This moves the normal user flow toward **Install AURA → Install Runtime → Download model → Chat**, without manual Python setup.
