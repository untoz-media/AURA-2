# M009.3 — Stability Validation

M009.3 is the final gate before the Public Beta installer is published.

## Automated checks

The shared Beta validation now covers:

- synchronized version metadata
- TypeScript compilation
- Vite frontend build
- embedded Python worker syntax
- Rust formatting

The Windows gate additionally runs:

- Rust unit/regression tests
- Tauri release build
- NSIS packaging
- installer artifact presence validation
- SHA-256 checksum generation

The same source gate is available locally:

```bash
npm install
npm run beta:validate
```

On Windows, the full local installer fallback is:

```powershell
npm run beta:build:windows
```

That command validates the source, runs the Rust regression suite, builds the Tauri/NSIS installer, copies the candidate into `artifacts/beta/<version>/`, creates `AURA-2-Windows-x64.sha256`, reports the Authenticode signature state and writes `AURA-2-Beta-Build.json` with the version, source commit, installer size, SHA-256 and build timestamp.

The finished artifact can then be independently re-checked with:

```powershell
npm run beta:verify:artifact
```

The manual end-to-end validation matrix lives in `docs/M009-3-WINDOWS-SMOKE-CHECKLIST.md`.

## GitHub-hosted runner incident — 2026-10-04

The repository workflows are currently blocked before runner assignment.

A controlled probe was performed with an Ubuntu workflow containing only one inline shell step:

```yaml
runs-on: ubuntu-latest
steps:
  - run: echo "runner-assigned"
```

The result was still:

- job created
- conclusion: `failure`
- steps: none / 0 executed
- no job logs
- failure within a few seconds

The same startup failure occurs on both `ubuntu-latest` and `windows-2022`.

This rules out AURA build commands, third-party actions, `actions/checkout`, Node, Python, Rust and NSIS as the cause of the current startup failure. The remaining likely class is a GitHub-hosted runner entitlement/billing/budget/account restriction outside repository code.

Before retrying, check the Untoz organization:

1. **Settings → Billing & Licensing → Budgets and alerts**
2. Verify the Actions budget is not exhausted with usage stopping enabled.
3. Verify there is no failed payment or payment-method warning.
4. If billing is healthy and jobs still fail before step 1, contact GitHub Support and provide the failed run IDs.

## Persistence hardening

The Beta candidate now writes its Beta preference/session JSON through a temporary file, flushes it to disk, and only then replaces the active state file. This reduces the chance of a forced shutdown leaving a partially-written JSON document.

Session recovery is conservative:

- a missing session marker means no prior recovery condition;
- a valid marker with `cleanExit: false` means recovered;
- a corrupt/unreadable existing session marker is treated as an unclean prior exit rather than silently reported as clean;
- marking a session clean now fails explicitly if the current marker is corrupt instead of masking the problem;
- after an unclean/recovered session, AURA starts paused and keeps Agents/Automations paused until the user explicitly resumes it;
- the main Chat workspace surfaces a Recovery Safe Mode banner with direct access to diagnostics and an explicit Resume AURA action.

Regression tests cover missing markers, corrupt markers, atomic state writes and the diagnostics schema privacy boundary. Settings → Beta & Diagnostics also computes an in-app integrity self-check covering the privacy invariants, diagnostics identity/schema and internal Agent/Automation counter consistency.

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

M009.3 remains pending until both of these are true:

1. a Windows installer candidate is actually built and smoke-tested on Windows;
2. the hosted Windows CI can execute real steps and pass again.

A runner failure before step 1 is infrastructure failure and does not count as successful validation.
