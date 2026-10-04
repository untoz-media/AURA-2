# AURA-2 0.9.0-beta.1 — Public Beta Candidate

**Status:** Release candidate prepared; Public Beta publication is still gated on successful CI/build validation.

AURA-2 0.9.0-beta.1 is the first Beta candidate for the new AURA desktop architecture.

## What AURA-2 is

AURA-2 is a local-first personal computer assistant for Windows. Its goal is to become an intelligent layer for the PC: understanding context, controlling supported applications, assisting with production workflows and executing bounded multi-step work under explicit permissions.

## Highlights

### Desktop foundation

- contextual First Local Setup card for Runtime/model onboarding

- native Tauri 2 + React + TypeScript desktop app
- system tray and Background Mode
- optional Windows autostart
- global AURA Overlay
- persistent themes and Settings
- per-user Windows NSIS installer path

### Local AI & Model Manager

- local Model Manager
- private Managed Python/AI Runtime
- GPU-aware PyTorch setup with CPU fallback
- resumable model downloads
- installed-model verification
- persistent local assistant runtime
- current assistant profile: AURA-1 / Qwen3-4B-Instruct-2507

The dedicated AURA-2 model checkpoint is not defined in this candidate yet. The Model Manager and architecture are ready for it.

### Computer Control

- deterministic app launching and lifecycle
- window discovery/switching
- keyboard and mouse actions
- volume/media controls
- system actions
- persistent permission policy
- explicit confirmation flow

### OBS Director Mode

- OBS WebSocket connection and live state
- scene and source control
- audio control
- recording and streaming controls
- production health
- reusable Director presets

### Memory & Context

- persistent local memory
- active app/window context
- recent-file context
- reusable routines
- project memory

### Voice

- microphone selection and testing
- Push-to-Talk (`Ctrl + Shift + F8`)
- local Whisper Base STT
- local Piper TTS
- Conversation Mode
- interruption / Stop Speaking
- optional wake phrase

### Vision

- explicit screen capture
- active-window capture
- region selection (`Ctrl + Shift + F9`)
- local SmolVLM2 500M analysis
- permission indicators
- local Vision history controls

### Agents & Automations

- bounded local multi-step planning
- deterministic Action execution
- Saved AURA Actions
- scheduled/event Automations
- pause/resume/cancel
- run history and interrupted-run recovery
- limited retry policy for idempotent app actions
- background Automations restricted to Read/Act + current Allow

### Beta diagnostics

- in-app **Settings → Diagnostics** readiness center
- one-click local Core self-test for stores, permission policy, model catalog and runtime health
- runtime/model/Voice/Vision/OBS technical state
- Agent and Automation health summary
- privacy-safe diagnostics report for bug reports
- excludes prompts, memories, screenshots, audio, file paths and credentials

### Beta stability hardening

- independent Beta quality/source gates
- repaired Agent regression-test annotations
- fail-safe Action Router parser paths
- shared tested Beta self-test finalization logic
- fail-closed Diagnostics readiness for unexpected states
- persistent Global Pause with direct Vision/Routine/Director/OBS execution guards
- stop/escape production actions remain available while paused
- privacy-safe Diagnostics report for bug reports
- atomic crash-resistant JSON persistence for critical local state
- integrity checks for user stores and desktop configuration
### Beta safety & privacy

- redesigned Privacy and Safety & Permissions UI
- Sensitive and Destructive permanent Allow blocked by Core
- explicit Agent plan review
- Global Pause covers Agents and Automations
- automatic product telemetry Off
- automatic crash reporting Off
- SHA-256 installer artifact path
- Beta source and release guard scripts

## Installation flow

`Install AURA → review Permissions → install AURA Runtime → download/select AURA-1 → install optional Voice/Vision models`

See `docs/BETA-GUIDE.md` for the tester guide.

For bugs and feature requests, see `docs/BETA-FEEDBACK.md` and use the structured GitHub issue forms.

## Known limitations

- dedicated AURA-2 checkpoint is not defined yet
- pre-release installer may be unsigned and trigger SmartScreen
- first-time local AI setup requires substantial disk/network usage
- CPU inference is slower than supported GPU acceleration
- cloud assistance is intentionally not configured

See `docs/BETA-KNOWN-ISSUES.md` for the current blocker list.

## Release gate

This candidate must not be tagged/published until:

- M009.3 stability gate is marked complete
- `package-lock.json` is generated and committed for reproducible npm installs
- Beta Quality executes successfully
- Windows Rust/Core tests execute successfully
- Windows NSIS installer is produced
- checksum artifact is produced
- critical Windows Beta checklist items pass

## Intended tag

`aura-v0.9.0-beta.1`
