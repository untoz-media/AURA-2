# M009.3 — Stability Testing

**Milestone:** M009.3  
**Stage:** AURA-2 Beta hardening  
**Base build:** 0.8.0-alpha.1

M009.3 adds repeatable regression gates before the Public Beta instead of relying on a successful Windows installer job as the only quality signal.

## Regression bug found during the audit

The M008 Agent test module contained two annotation defects:

- `only_idempotent_app_steps_are_retried` had duplicate `#[test]` attributes
- `action_alias_conflicts_are_case_insensitive` was missing `#[test]`, so that regression case was not executed

M009.3 repairs both annotations and expands pure regression coverage for:

- retry allowlisting
- non-retryable Wait and Saved Action steps
- Automation interval lower/upper bounds
- invalid non-JSON planner output
- case-insensitive Action alias collisions

## Additional hardening findings

Continued Beta audit found and fixed additional issues:

- Action Router parsing no longer relies on runtime `unreachable!` / `expect()` assumptions; unexpected internal parser shapes now fail safely instead of panicking.
- The Beta self-test finalization helper accidentally called itself recursively. The runtime command had duplicated inline finalization, so the UI path did not invoke that recursion, but Rust tests using the helper could stack-overflow once CI actually executed.
- Runtime and tests now share one `finish_beta_self_test` implementation, so the tested logic is the production logic.
- Unknown required self-test statuses now fail closed and count as failures.
- Diagnostics renders every non-ready self-test as critical instead of allowing a visually green `Attention` state.
- Stale Alpha fallback copy (`Desktop Foundation`, `0.2.0`, planned Model Router) was removed from Settings.
## Beta source safety checks

`npm run beta:source-check` performs fast source-level release guards.

It fails on:

- unresolved Git merge-conflict markers
- known stale Alpha UI copy
- missing Core permission sanitization invariants
- missing Agent plan/wait limits
- missing unattended Automation permission constraints
- missing sensitive background Routine/Director guards
- retry policy drift
- duplicate Agent `#[test]` attributes
- critical Agent regression functions that are no longer marked as tests

These checks do not replace Rust tests or a security review. They are a cheap additional tripwire for Beta-critical invariants.

## Permission corruption fails closed

The persisted permission policy previously fell back to normal defaults when its JSON could not be read or parsed. Because normal defaults allow Read and Act, a corrupted file could unintentionally become less restrictive than the user's previous policy.

Beta recovery now distinguishes first run from corruption:

- missing policy file → normal recommended defaults
- unreadable/invalid existing policy → fail-closed recovery policy
- fail-closed policy → Read, Act, Modify, Sensitive and Destructive all require Ask

This also prevents background Automations from continuing silently because unattended execution requires current Allow.
## Crash-resistant local persistence

The Beta audit found that several JSON stores still used direct whole-file writes. A process or OS interruption during a direct write could leave truncated local state.

M009.3 now provides a shared atomic JSON persistence primitive:

`serialize → write temp file in the same directory → flush/sync → rename over destination`

It is used for:

- Memory
- Project Memory
- Routines
- Saved AURA Actions
- Automations and Agent run state
- Director Mode presets
- Model Manager configuration
- Vision preferences/history
- permission policy
- Voice preferences
- desktop preferences

The Beta self-test also performs a privacy-safe atomic write/read-back probe and validates the main user stores and configuration files for parse integrity.

The source gate rejects a return to direct `fs::write` for these critical JSON stores.
## Independent Beta Quality workflow

`.github/workflows/beta-quality.yml` runs on `ubuntu-latest` and performs:

1. dependency installation
2. release metadata validation
3. Beta source safety checks
4. TypeScript compilation and Vite frontend build

This workflow is intentionally independent from the Windows NSIS runner.

## Windows runner incident

Windows Build runs #74 and #75 both failed before the first workflow step was recorded.

In both cases GitHub reported an empty step list, meaning Checkout, npm, TypeScript, Rust, Tauri and NSIS did not execute. M009.3 therefore treats that condition as a runner/infrastructure failure rather than a product test failure.

The Windows workflow still remains required for a real Beta installer and now also runs the Beta source guard when the runner starts normally.

## Root quality commands

```powershell
npm run release:check
npm run beta:source-check
npm run beta:quality
```

`beta:quality` combines release metadata checks, source guards and the frontend production build.

## In-app Beta Diagnostics

The Public Beta candidate now includes **Settings → Diagnostics**.

It gives testers a privacy-safe readiness snapshot for runtime, assistant model, Voice, Vision, OBS, permissions, Agents and Automations, plus a sanitized report that can be copied into bug reports.

No prompts, memories, screenshots, audio, file paths or credentials are included in the copied report.

The Diagnostics Center also exposes a local **Core self-test** that validates Local Data readability, permission sanitization, Saved Actions/Automations stores, Agent history, model catalog and runtime health without executing PC or OBS actions.

## Release gate

M009.3 should only be marked complete after the independent Beta Quality workflow passes. A successful Windows build is still required before the Public Beta in M009.6.

## Next

**M009.4 — Telemetry Policy**

AURA should define a clear Beta telemetry/crash-reporting policy before public distribution, with local-first defaults and no silent collection.
