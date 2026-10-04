# AURA-2 Beta

AURA-2 is a Windows-first, local-first personal computer assistant. The Beta combines deterministic computer control with local AI, Voice, Vision, Memory, OBS Director Mode, Agents and Automations.

> **Release status:** Beta hardening is in progress. This documentation is being prepared before the Public Beta is published.

## Start here

- [Beta Guide](./BETA-GUIDE.md) — install, first-run setup and feature overview
- [Beta Test Checklist](./BETA-TEST-CHECKLIST.md) — structured validation before release
- [Known Issues](./BETA-KNOWN-ISSUES.md) — current limitations and release blockers
- [Telemetry Policy](./TELEMETRY-POLICY.md) — what AURA does and does not send
- [M009.3 Stability Testing](./M009-3-STABILITY-TESTING.md) — quality gates and CI status

## Beta principles

- local execution first
- explicit permissions
- no automatic product telemetry
- no permanent Allow for Sensitive or Destructive actions
- background Automations remain bounded
- model/runtime downloads happen only when requested
- Windows x64 is the initial supported desktop target

## Current model status

The desktop Model Manager is ready for multiple assistant models.

- **AURA-1** is currently the usable local assistant profile and is backed by `Qwen/Qwen3-4B-Instruct-2507`.
- **AURA-2** has a Model Manager slot, but its dedicated checkpoint has not yet been defined.
- **AURA Voice STT** uses local Whisper Base.
- **AURA Voice TTS** includes local Piper voices.
- **AURA Vision** uses local SmolVLM2 500M for image/UI understanding.

The AURA-2 product architecture, computer-control stack and agent system do not depend on an unrestricted model having direct access to Windows.

## Public Beta gate

The Public Beta must not be published until:

1. Beta quality checks pass
2. a real Windows installer build succeeds
3. installer checksum is produced
4. critical Beta checklist items pass on a Windows machine
5. known release blockers are reviewed
