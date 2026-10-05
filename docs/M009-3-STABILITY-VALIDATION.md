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

The latest re-check on the expanded Beta candidate reproduced the same infrastructure signature:

- Windows Build run `37237151569`: `runner_name` empty, `steps: []`, failure in ~2 seconds
- Static Checks run `37237151580`: `runner_name` empty, `steps: []`, failure in ~1 second

This confirms that the newer Create, Clipboard and File Intelligence code was not executed by those failed hosted jobs.

A further re-check after File Intelligence V2 and App Skills V1 produced the same signature:

- Static Checks run `37238848202`: empty runner name, `steps: []`, failed in ~2 seconds
- Windows Build run `37238848232`: empty runner name, `steps: []`, failed in ~2 seconds

Those runs did not execute the new File Intelligence V2 or App Skills code either.

After the App Skills V2 registry + Notepad pass, the failure signature remains unchanged:

- Static Checks run `37239755921`: empty runner name, `steps: []`, failed before source validation
- Windows Build run `37239755865`: empty runner name, `steps: []`, failed before installer build

Therefore the App Skills V2/Notepad changes were not executed by GitHub-hosted CI.

After App Skills V3 (Windows Terminal + Calculator), the hosted jobs still fail before runner assignment:

- Windows Build run `37240255952`: empty runner name, `steps: []`, failed before the NSIS job began
- Static Checks run `37240255914`: empty runner name, `steps: []`, failed before Beta source validation began

The App Skills V3 pass therefore also has no GitHub-hosted compile/test evidence yet.

After Drag & Drop Actions V1, the same pre-runner failure reproduced again:

- Windows Build run `37241609285`: empty runner name, `steps: []`, failed before the NSIS installer job executed
- Static Checks run `37241609275`: empty runner name, `steps: []`, failed before Beta source validation executed

The runs briefly appeared queued, then completed as failures within about two seconds without receiving a runner. Drag & Drop V1 therefore has no hosted compile/test evidence yet.

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

Regression tests cover missing markers, corrupt markers, atomic state writes and the diagnostics schema privacy boundary.

Diagnostics schema v2 adds structured local health checks. The health report probes configuration/local-data writeability and validates the active session marker, Beta preferences, permission safety floor, model catalog, Agent run store, Saved Actions store, Automation store, managed-runtime state, privacy boundary and aggregate runtime counters. Each check is reported independently as Passed/Failed so one damaged subsystem no longer makes the entire diagnostic snapshot unavailable.

The desktop runs this health report automatically during startup without uploading anything. Settings → Beta & Diagnostics shows the full per-subsystem report, and Chat surfaces a local warning when the overall health state is degraded.

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

## Windows Build Doctor

Before attempting a local NSIS build, run:

```powershell
npm run beta:doctor:windows
```

The doctor performs a fail-fast prerequisite pass for:

- PowerShell
- Git
- Node.js 22+
- npm
- Rust `x86_64-pc-windows-msvc`
- Visual Studio 2022 C++ Build Tools
- MSVC `cl.exe` / `link.exe`
- Windows 10/11 SDK x64 libraries
- Python used by source validation
- minimum build disk space
- WebView2 presence as a non-blocking runtime warning

When invoked by `npm run beta:build:windows`, the doctor also loads `VsDevCmd.bat` into the build process when available so an installed MSVC toolchain does not have to be manually exposed in the user's normal PowerShell PATH.

Local Beta builds now embed traceable build provenance:

- exact Git commit
- `local-beta-build` source
- `beta-local-smoke` label

The local build manifest uses schema v3. Before compilation, the local build resolves an npm `package-lock.json` and Rust `Cargo.lock`, uses those locks for the validation/test pass, copies both into the artifact directory and records both SHA-256 hashes. Artifact verification rejects an untraceable source commit or a changed dependency lock.

