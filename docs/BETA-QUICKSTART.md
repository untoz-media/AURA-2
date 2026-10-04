# AURA-2 Beta Quick Start

## 1. Install

Run the verified AURA-2 Windows x64 NSIS installer.

Normal Beta installation is per-user and does not require administrator privileges.

## 2. Read the first-run guide

AURA explains the Beta privacy model and Permission Engine before normal use.

## 3. Review permissions

Recommended safe defaults:

- Read: Allow
- Act: Allow
- Modify: Ask
- Sensitive: Ask
- Destructive: Ask

Sensitive and Destructive cannot be permanently set to Allow.

## 4. Install the Managed AI Runtime

Open Models and install AURA's private managed Python/AI runtime.

## 5. Install an assistant model

Install AURA-1 if you want free-form local model responses and Agent planning.

AURA-2 model remains a future model line until its own weights are available.

## 6. Optional feature models

Install only the local features you want:

- Whisper Base for Voice STT
- Piper voices for local TTS
- SmolVLM2 for Vision
- AURA Create · Image for local text-to-image generation

## 7. Try deterministic controls first

Examples:

- open Brave
- switch to OBS
- minimize Brave
- maximize OBS
- restore Brave
- find file Artemis
- read clipboard
- what app am I using?
- run a saved routine

## 8. Try Voice and Vision

- Ctrl + Shift + F8: Push-to-Talk
- Ctrl + Shift + F9 twice: select a Vision region
- Ctrl + Shift + Space: Overlay

## 9. Try Agents

Open Agents, describe a bounded goal, review the proposed steps and approve only if the plan matches your intent.

## 9.5 Create a local image

Open **Create → Image**.

If required:

1. Install or repair the AURA Runtime.
2. Download **AURA Create · Image**.
3. Enter a prompt.
4. Choose square, landscape or portrait.
5. Optionally set a negative prompt, inference steps or seed.
6. Choose **Generate image**.

The first generation takes longer because the local Diffusers pipeline must load. Later generations reuse the resident worker.

Generated PNGs are saved locally under **Pictures → AURA Create** when Windows exposes the Pictures directory.

The model download requires network access once. Inference is configured to use the installed local files only.

## 10. Diagnostics

AURA runs a lightweight local health report on startup. You can inspect it at:

Settings → Beta & Diagnostics

A healthy Beta should show the local subsystem checks as Passed. If Chat shows a degraded-health warning, open the report before relying on the affected subsystem.

For a support snapshot:

Settings → Beta & Diagnostics → Refresh → Export JSON

Nothing is uploaded automatically.

If AURA detects that the previous session ended unexpectedly, it starts paused in Recovery Safe Mode. Review the recovery/health state and explicitly choose Resume AURA when you are ready.
