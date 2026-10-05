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
- native Drag & Drop V2 intake with an in-memory opaque-id registry
- explicit per-file and batch inspection actions gated by Read permissions
- bounded 64 KiB / 12,000-character previews for allowlisted UTF-8 text/code files
- image-dimension inspection without exposing canonical paths
- one-turn **Attach to Chat** / **Attach all to Chat** composer attachments
- removable attachment chips before send and safe filename labels in local chat history
- attachment-only Send falls back to an explicit "Analyze the attached local files." request
- **Analyze with AURA** one-click drop analysis shortcut
- 6,000-character total model attachment cap with fair multi-file excerpt budgeting
- expanded bounded text-context support for source/config/subtitle formats and allowlisted extensionless developer files
- attachment requests bypass action/Routine/Director routing
- attached content is explicitly treated as untrusted data for prompt-injection resistance
- attachment context stays outside visible Chat messages and model conversation history
- metadata-only handling for PDFs, Office files, video, audio and archives
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

## Testing Preview release hardening

The Beta candidate now includes the guarded distribution path for **AURA-2 Alpha v1 — Testing Preview** without changing the real application version.

New release-readiness work:

- diagnostics schema v3 adds privacy-safe build provenance: source commit, build source and build label
- Settings → Beta & Diagnostics shows the exact build identity used for bug reports
- Windows CI artifacts include a SHA-256 checksum and traceable build manifest
- the Testing Preview release pipeline resolves and attaches the exact npm and Cargo lockfiles used for the build
- the GitHub release is explicitly marked as a pre-release and not Latest
- manual release dispatch requires explicit `RELEASE_ALPHA_V1` confirmation
- security reports are routed to private GitHub Security Advisories
- testing-preview bug reports ask for build provenance and privacy-safe diagnostics

Publication remains blocked until a real Windows runner executes the source validation, Rust tests, NSIS build and smoke-test gate. A hosted runner failure before step 1 is not treated as release validation.

## Crash Loop Guard v1

Repeated unclean launches now escalate beyond the normal one-session Recovery Safe Mode.

- the session marker tracks a backward-compatible unclean-session streak
- one unclean exit still starts AURA paused in normal Recovery Safe Mode
- two consecutive unclean sessions activate Crash Loop Guard
- Crash Loop Guard suppresses `--background` hiding so the recovery state is visible
- the Chat recovery banner removes one-click Resume while the guard is active
- local health diagnostics expose the repeated recovery condition without uploading anything
- a subsequent clean shutdown resets the recovery streak

Settings → Beta & Diagnostics also adds **Testing Preview readiness**, separating required core/safety, Windows target and traceable-build checks from optional Managed Runtime/local-model setup.

## Fail-closed recovery resume

Recovery Safe Mode can no longer be bypassed by a generic resume call when Crash Loop Guard or a degraded local startup health report is active.

- Core rejects normal unpause requests while the recovery gate is active
- General Settings redirects the user to Beta & Diagnostics instead of silently unpausing
- Beta & Diagnostics exposes **Review & Resume** only in the relevant paused/degraded state
- the override requires an explicit confirmation
- an acknowledged override is recorded as a local lifecycle event
- normal one-session recovery remains lightweight when the stronger gate is not required

## Beta Test Session v1

The Beta & Diagnostics workspace now includes a structured local test pass for the first AURA-2 testing builds.

- 15 fixed validation areas cover install, lifecycle, recovery, permissions, Computer Control, models, attachments, Create, Voice, Vision, Memory, Agents/Automations, OBS, diagnostics and installer lifecycle
- test progress persists locally between app views/restarts
- each item stores only completion state and timestamp
- arbitrary test-area ids are discarded by backend normalization
- no free-text notes are stored
- the test-session store participates in local health checks
- **Export report** writes a size-bounded JSON bundle containing the checklist plus the existing privacy-validated diagnostics snapshot
- nothing is uploaded automatically

