# M009.3 — Stability Validation

M009.3 is the final gate before the Public Beta installer is published.

## Automated checks

The Windows CI pipeline is configured to run:

- Rust formatting check
- Rust unit/regression tests
- TypeScript compilation
- Vite frontend build
- Tauri release build
- NSIS packaging
- installer artifact presence validation
- SHA-256 checksum generation

## Existing recovery systems

The Beta candidate also includes:

- Agent run recovery as interrupted after abnormal shutdown
- model/runtime fail-closed behavior
- corrupt local JSON fail-closed/left unchanged in key persisted stores
- explicit local Beta session marker
- local diagnostics export
- permission gating for Agents/Automations
- background automation pause when AURA is paused

## Public Beta gate

This milestone remains pending until a Windows CI attempt executes real build steps and passes.

A runner failure before step 1 is an infrastructure failure and does not count as a successful validation.
