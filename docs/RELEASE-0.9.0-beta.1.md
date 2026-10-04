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
- explicit zero-upload telemetry policy
- synchronized 0.9.0-beta.1 version metadata across the monorepo

## Privacy

0.9.0-beta.1 does not implement usage telemetry or automatic crash uploads.

Diagnostics are generated and exported locally only when requested.

## Release status

This commit is a Beta Candidate, not yet the published Public Beta.

Public Beta publication is gated on a successful Windows installer CI run with real build steps and a verified NSIS artifact.
