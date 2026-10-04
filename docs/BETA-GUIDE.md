# AURA-2 Beta Guide

## 1. What AURA-2 is

AURA-2 is designed as a personal computer assistant rather than a chatbot-first app. It can combine local language models with deterministic Windows actions, Voice, Vision, Memory, OBS control, reusable Actions, Agents and Automations.

Local execution is the default, and the model does not receive unrestricted direct control of the PC.

## 2. Platform

Initial Beta target:

- Windows x64
- per-user NSIS installation
- Microsoft WebView2 (the installer embeds the bootstrapper)

A discrete NVIDIA GPU is optional. The managed runtime can use CUDA when supported and otherwise falls back to CPU execution; CPU inference can be substantially slower.

## 3. Install

1. Obtain the AURA-2 Windows installer and its matching `.sha256` file from the approved Beta build.
2. Verify the installer in PowerShell:

```powershell
Get-FileHash .\AURA-2*.exe -Algorithm SHA256
```

3. Compare the result with the provided `.sha256` file.
4. Run the installer.
5. Windows SmartScreen may warn about unsigned pre-release builds until production code signing is configured.

## 4. First-run setup

On a fresh install, the Chat home shows a **First Local Setup** card until the essential local AI setup is complete. It tracks:

1. Managed AURA Runtime
2. installed + selected assistant model

The card links directly to Models, Permissions and Diagnostics and disappears automatically when both essential steps are ready.

Recommended order:

`Install AURA → review Permissions → install AURA Runtime → download/select AURA-1 → add Voice/Vision models as needed`

### Permissions

The recommended defaults are:

- Read — Allow
- Act — Allow
- Modify — Ask every time
- Sensitive — Ask every time
- Destructive — Ask every time

Sensitive and Destructive cannot be permanently set to Allow. AURA Core enforces this even if unsafe values are injected into stored settings.

### Managed AI Runtime

From **Models**, install the AURA Runtime. It creates a private Python/AI environment for AURA without adding Python to the system PATH.

The runtime installs the local AI dependencies used by assistant, Voice and Vision models. Runtime files and model weights are stored separately.

### Assistant model

For the current Beta hardening build, install and select **AURA-1**. It uses `Qwen/Qwen3-4B-Instruct-2507` and is approximately 8 GB before runtime/cache overhead.

The **AURA-2** model slot is visible but unavailable until a dedicated checkpoint is defined.

## 5. Everyday use

### Chat

Free-form messages use the selected verified local assistant model when no deterministic AURA command matches.

### Computer

AURA can route supported Windows actions through its deterministic Action Router and Permission Engine. Higher-impact actions are confirmed or blocked according to the current policy.

### Memory

Memory, project memory, routines and relevant context are stored locally.

### Voice

- Push-to-Talk: `Ctrl + Shift + F8`
- local speech-to-text: Whisper Base
- local text-to-speech: Piper
- Conversation Mode and optional wake phrase are available from Voice settings

### Vision

AURA Vision can analyze explicit captures of:

- the screen
- the active window
- a selected region

Vision region shortcut: `Ctrl + Shift + F9` to mark the first and opposite corners.

Vision uses the local `SmolVLM2-500M-Video-Instruct` profile.

### Overlay

Open the compact AURA Overlay with:

`Ctrl + Shift + Space`

### OBS / Director Mode

Connect AURA to the OBS WebSocket server from Settings. Director Mode can control supported OBS state through deterministic presets and permission checks.

Do not test streaming controls against a real production destination unless you intend to go live.

## 6. Agents

Agents turn a goal into a bounded plan. The local planner may propose only supported AURA steps such as:

- launch app
- switch app
- run routine
- run Director preset
- wait
- run Saved AURA Action

The Core revalidates every plan before execution. Plans may be runnable, require confirmation or be blocked by policy.

Agents can be paused, resumed or cancelled. Global Pause suspends active Agents at safe boundaries.

## 7. Automations

Automations run a pre-saved AURA Action from a local trigger. They do not ask the model to invent a new background plan at execution time.

Supported trigger families include:

- AURA startup
- interval
- future date/time
- daily repeat
- supported app focus

Background execution is stricter than manual execution: only Read/Act actions are eligible, and the relevant permission must still be Allow when the automation fires.

## 8. Privacy

AURA-2 Beta sends no automatic product analytics, usage telemetry or crash reports to Untoz.

Prompts, memory, screenshots, audio/transcripts, file context, Agent history and Automation state are not automatically transmitted as telemetry.

See [Telemetry Policy](./TELEMETRY-POLICY.md) for the full policy.

## 9. Beta Diagnostics

Open **Settings → Diagnostics** to review AURA's Beta readiness without exposing personal content.

The Diagnostics Center shows:

It also includes **Run self-test**, which asks AURA Core to validate local state without executing computer actions. The self-test checks Local Data readability, permission sanitization, Saved Actions, Automations, Agent history, the model catalog and runtime health.

Voice, TTS and Vision are optional Beta capabilities: an unavailable optional runtime can produce a warning without failing the required Core checks.

- AURA build/version
- Managed Runtime state
- selected assistant model readiness
- Voice STT/TTS state
- Vision model/runtime state
- OBS connection state
- Global Pause and permission guardrails
- Agent failure/interruption count
- enabled Automation count
- automatic product telemetry status

The **Copy diagnostics** action creates a privacy-safe text report for bug reports. It intentionally excludes prompts, memories, screenshots, audio, file paths, OBS passwords and other personal content.

## 10. Troubleshooting

### Local model will not start

- confirm the Managed Runtime reports Ready
- confirm an assistant model is installed and selected
- repair the Managed Runtime from Models if needed
- on lower-memory GPUs, try CPU fallback if the GPU path cannot load the model

### Voice transcription is unavailable

- confirm AURA Voice STT is installed
- confirm the microphone device is selected
- test input level from Voice settings
- confirm AURA is not globally paused

### Vision analysis is unavailable

- confirm AURA Vision is installed
- create a fresh explicit capture
- verify Read permission is not Block

### Agent plan is blocked

Open **Settings → Permissions** and inspect the permission class reported by the plan. Do not loosen a permission merely to make a plan run unless you understand the action.

### OBS will not connect

- confirm OBS is running
- confirm the OBS WebSocket server is enabled
- verify host, port and password
- use `127.0.0.1` for the normal same-PC setup

## 11. Reporting Beta problems

When reporting a bug, include only the information you choose to share:

- AURA version
- Windows version
- feature involved
- exact steps to reproduce
- expected result
- actual result/error text

Do not share OBS passwords, private prompts, personal files or other secrets in a bug report.
