# AURA-2 0.9.0-beta.1 — Beta Candidate

0.9.0-beta.1 is the first AURA-2 Beta candidate.

It combines the complete Alpha milestones:

- Desktop Foundation
- Computer Control
- OBS / Director Mode
- Memory & Context
- Voice
- Vision
- Agents & Automations
- AURA Create local image generation

with new Beta-readiness work.

## Beta candidate additions

- hardened per-user NSIS installer configuration
- downgrade protection
- installer publisher/product metadata
- Windows CI formatting/test/build/package gates
- installer SHA-256 artifact
- first-run Public Beta onboarding
- permission review entry point
- local previous-session recovery detection
- local diagnostics snapshot/export
- diagnostics schema v2 with structured subsystem health checks
- automatic startup health report with degraded-state Chat warning
- crash recovery safe mode that starts AURA paused after an unclean session
- atomic Beta/session state persistence
- explicit zero-upload telemetry policy
- synchronized 0.9.0-beta.1 version metadata across the monorepo
- named app window minimize/maximize/restore controls
- App Skills V3 with a Core-owned dynamic Skill Registry
- focus-verified Brave/Chrome browser skills
- focus-verified Notepad New note / Find / Select all / Undo / Redo skills
- Windows Terminal navigation/UI skills with no arbitrary shell execution
- Windows Calculator Standard/Scientific/Programmer/Date/Graphing mode skills
- File Explorer Skills for safe Desktop/Documents/Downloads/Pictures/Videos/Music access
- native Drag & Drop intake with an in-memory opaque-id registry
- safe dropped-image handoff to Vision through normalized cache copies
- bounded File Intelligence across personal Windows folders without content scanning
- recent-file/category queries for videos, images, audio, documents, archives and Downloads
- canonical-path-safe reveal in File Explorer without executing files
- privacy-first Clipboard Intelligence for explicit text read/write/clear
- voice privacy suppression for sensitive clipboard reads
- local AURA Create text-to-image generation
- pinned SafeTensors-only Create model manifest
- Diffusers managed-runtime support
- persistent image-generation worker with recovery
- local PNG output + in-app preview

## Privacy

0.9.0-beta.1 does not implement usage telemetry or automatic crash uploads.

Diagnostics stay local. A lightweight health report is generated automatically at startup, while JSON export only happens when explicitly requested. No diagnostics are uploaded automatically.

## Release status

This commit is a Beta Candidate, not yet the published Public Beta.

Public Beta publication is gated on a successful Windows installer CI run with real build steps and a verified NSIS artifact.
