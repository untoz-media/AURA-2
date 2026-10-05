# AURA-2

<p align="center">
  <strong>Your PC. Now it understands you.</strong>
</p>

**AURA-2** is the next generation of AURA, a local-first personal AI assistant developed by **Untoz**.

AURA is not designed to compete with general-purpose AI chatbots. Its purpose is different: to become an intelligent layer for your computer — able to understand context, control applications, automate workflows, assist with production tasks, and act on your behalf with explicit permissions.

The 0.9 Beta candidate adds first-run onboarding, explicit no-telemetry policy, local diagnostics export, session recovery detection and a hardened per-user Windows installer pipeline.

> **Current stage:** M009 Beta Candidate · AURA-2 0.9.0-beta.1 · Windows installer validation pending

A guarded **AURA-2 Alpha v1 — Testing Preview** release pipeline is now prepared. It will publish only as a GitHub pre-release after a real Windows build passes the release gate; the current hosted-runner outage does not count as validation. Preview builds embed privacy-safe build provenance (commit/source/label) in local diagnostics and ship with installer checksum + build manifest.

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
- App Skills ✅
- File Intelligence ✅
- Clipboard Intelligence ✅
- Drag & Drop actions ✅
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

### M005 — Memory & Context ✅
Persistent local memory, app/window/recent-file context, user routines and project-scoped memory.

### M006 — Voice ✅
Local microphone capture, Push-to-Talk, Whisper STT, Piper TTS, Conversation Mode, interruption and wake phrase support.

### M007 — Vision ✅
Explicit Windows screenshot/region capture, active-window understanding, local multimodal UI analysis and privacy-first history.

### M008 — Agents & Automations ✅
Local multi-step planning, deterministic Agent execution, reusable AURA Actions, event triggers, schedules, background status and failure recovery.

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

AURA-2 is now at **M009 — Beta Candidate** as **0.9.0-beta.1**. The candidate combines deterministic Windows control, named window state control, OBS Director Mode, persistent local memory and context, local Voice, local Vision, local image generation through AURA Create, bounded local Agent planning, reusable AURA Actions, event/scheduled automations, recovery-aware background execution, first-run Beta onboarding, local health diagnostics and an explicit zero-telemetry policy.

The remaining Public Beta gates are stability validation of a real Windows installer and restoration of GitHub-hosted runner execution. Drag & Drop intake is now included in the candidate and remains subject to the same Windows smoke gate. APIs, architecture and product behaviour may still change during the Beta cycle.

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

AURA Create now has a real local image-generation backend. Video remains deliberately unavailable until a real hardware-aware video engine is connected; AURA does not simulate unavailable generation features.


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






### Drag & Drop Actions

AURA now accepts local files dropped onto the main desktop window through Tauri's native drag/drop events.

Drag & Drop V2 is an **explicit intake + bounded inspection layer**, not automatic execution.

Current behavior:

- up to 8 files accepted per drop
- files are canonicalized locally before entering the temporary registry
- directories are rejected
- duplicate canonical paths are deduplicated
- React receives an opaque drop id + safe metadata, not the real filesystem path
- the canonical path remains inside an in-memory Core registry
- nothing is opened, executed, uploaded, remembered or analyzed automatically
- the temporary drop session can be dismissed without modifying the original files
- any accepted file can be revealed explicitly in File Explorer
- any accepted file can be inspected explicitly without exposing its real path to React
- allowlisted text/code formats can expose a temporary preview bounded to 64 KiB / 12,000 characters
- image inspection can expose local dimensions without creating a Vision capture
- **Inspect all** applies the same bounded rules across the temporary batch
- **Attach to Chat** / **Attach all to Chat** adds selected opaque drop IDs to the next desktop message
- attached files appear as removable chips above the composer and safe filename labels in the local chat UI
- attachments are one-turn: after an accepted send they are automatically detached from the next message
- sending attachments with an empty composer uses an explicit local "Analyze the attached local files." request
- **Analyze with AURA** remains available as a one-click batch shortcut
- model attachment context is capped to 6,000 characters total and never contains canonical filesystem paths
- multi-file text excerpts share that budget fairly so later attachments are not starved by earlier files
- the bounded text allowlist includes common source/config/subtitle formats plus known extensionless text files such as `.env`, `.gitignore`, `Dockerfile`, `Makefile` and `README`
- attachment requests bypass deterministic computer-action, Routine and Director routing
- attached text is explicitly marked as untrusted data for prompt-injection resistance
- PDFs, Office files, video, audio and archives remain metadata-only in this Beta step
- PNG/JPEG/WebP/GIF/BMP images can be staged explicitly for AURA Vision
- Vision works from a normalized PNG copy in AURA's cache, never from the original image
- Vision image import is bounded to 40 MB and the existing 24-million-pixel capture limit
- dropped-file paths/content are not added to Beta diagnostics

The drop tray classifies metadata as image, video, audio, document, archive or other. V2 adds explicit bounded inspection for allowlisted UTF-8 text/code files while keeping complex binary formats metadata-only.

### App Skills

AURA now has a real deterministic **App Skills V2** layer backed by a Core-owned Skill Registry.

The desktop UI no longer hard-codes which app capabilities exist. AURA Core publishes skill metadata, permission class and contextual availability, and the Computer workspace renders that catalog dynamically.

V1 includes Browser Skills for **Brave** and **Google Chrome** with:

- New tab
- Next tab
- Previous tab
- Reload tab
- Focus address bar
- Reopen closed tab

Every Browser Skill:

1. resolves a known browser target;
2. brings that browser window to the foreground;
3. waits briefly for Windows to complete the focus transition;
4. verifies the foreground process really is the requested browser;
5. sends only the fixed shortcut associated with that skill.

If foreground verification fails, no shortcut is injected.

File Explorer Skills V1 can also open the known personal folders Desktop, Documents, Downloads, Pictures, Videos and Music using paths resolved by Windows/Tauri. They do not accept arbitrary paths or use a shell.

Notepad Skills V1 are also available contextually:

- New note → Act
- Find → Act
- Select all → Act
- Undo → Modify
- Redo → Modify

Windows Terminal Skills V1:

- New tab
- Next tab
- Previous tab
- Command palette
- Find
- Tab/profile dropdown

Terminal Skills never accept arbitrary shell command text. They use the documented default Windows Terminal key bindings and therefore respect the product boundary that AURA is navigating the Terminal UI, not executing a shell.

Calculator Skills V1:

- Standard mode
- Scientific mode
- Programmer mode
- Date Calculation
- Graphing mode

Calculator Skills use the keyboard accelerators exercised by the official Microsoft Calculator repository's manual test plan.

Notepad shortcuts use the same foreground-process verification as Browser Skills. Save/Close are intentionally not exposed in this pass because they can write files or risk unsaved work.

App Skills remain deterministic and do not require an LLM.

### File Intelligence

AURA can now perform bounded filename/metadata search across the user's personal Windows libraries without scanning the whole machine or reading file contents.

V2 also supports deterministic recent-file queries by filesystem modified time, including videos, images, audio, documents, archives and Downloads.

Current V1 boundary:

- Desktop
- Documents
- Downloads
- Pictures
- Videos
- Music
- maximum recursion depth: 4
- maximum scanned entries: 8,000
- maximum returned results: 20
- symlinks are not followed
- hidden dot-prefixed entries are skipped
- only names and filesystem metadata are inspected
- file contents are never opened by the search engine
- a returned path can be revealed in File Explorer after a separate Act action
- reveal validates the canonical path remains inside an allowed personal folder
- AURA does not execute/open the matched file in V1

Examples:

- `Find file Artemis`
- `Search files WorldUnited`
- `Procura ficheiro thumbnail`
- `Latest video`
- `Latest video I exported`
- `Imagens recentes`
- `Último download`
- `Recent documents`
- `Reveal file "C:\\Users\\…\\Documents\\report.pdf"`

For phrases such as “latest video I exported”, AURA explicitly reports the **most recently modified matching file**. It does not claim to know which application created or exported the file.

### Clipboard Intelligence

AURA can now interact with Windows text clipboard data through explicit commands and Computer workspace controls.

Privacy rules:

- clipboard reading is Sensitive and requires confirmation by default
- clipboard writing is Modify and requires confirmation by default
- clearing the clipboard is Destructive and requires confirmation
- AURA does not poll or monitor clipboard changes in the background
- clipboard text is never added to Beta diagnostics
- voice-triggered clipboard reads are shown visually but are not spoken aloud by TTS
- large clipboard reads are bounded before being returned to the conversation

Examples:

- `Read clipboard`
- `Copy to clipboard Hello AURA`
- `Clear clipboard`
- `Lê o clipboard`
- `Copia para o clipboard Olá`

### AURA Create

The Create workspace now includes real local text-to-image generation.

Current behavior:

- dedicated `AURA Create · Image` model managed separately from the assistant model
- pinned model revision and SafeTensors-only weight manifest
- one-time model download with pause/resume/cancel support
- persistent local Diffusers worker using only installed files during inference
- square, landscape and portrait presets
- configurable inference steps, optional negative prompt and reproducible seed
- CUDA-aware execution with CPU offload when CUDA is available
- CPU fallback
- generated PNG files saved under `Pictures/AURA Create` when the Pictures directory is available
- preview remains in the current desktop session
- removing or repairing the managed runtime stops all local AI workers first

Video generation remains planned and is not simulated by the interface.


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
- Transformers / Diffusers / Accelerate / BitsAndBytes / Safetensors
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
- installs Transformers, Diffusers, Accelerate, BitsAndBytes and Safetensors
- verifies the final Python/AI stack before marking it Ready
- exposes install, repair, reinstall and removal controls in the desktop UI

This moves the normal user flow toward **Install AURA → Install Runtime → Download model → Chat**, without manual Python setup.
