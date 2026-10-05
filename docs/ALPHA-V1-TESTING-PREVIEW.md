# AURA-2 Alpha v1 — Testing Preview

This document defines the first downloadable AURA-2 testing preview.

> Product version: `0.9.0-beta.1`  
> Distribution label: **AURA-2 Alpha v1 — Testing Preview**  
> Platform: Windows x64  
> Release type: GitHub pre-release

The distribution label is intentionally separate from the internal semantic version. AURA-2 has already progressed beyond the earlier Alpha milestones, so the application itself remains on the real `0.9.0-beta.1` version line.

## Release gate

Do not publish the Testing Preview until all of the following are true:

- `npm run beta:doctor:windows` reports no blocking Windows build prerequisites
- source validation passes
- Rust regression tests pass
- a real NSIS installer is produced on Windows
- the installer checksum matches
- the build manifest matches the installer
- build provenance is present in Settings → Beta & Diagnostics
- the critical Windows smoke checklist is completed
- no release-blocking permission, startup, data-loss or crash issue is known

## Build provenance

Testing Preview builds embed three non-personal compile-time identifiers:

- source commit
- build source
- build label

These values are included in the local diagnostics snapshot so a tester can identify the exact binary when reporting a problem. They do not contain prompts, files, screenshots, transcripts, account tokens or other user data.

## Download bundle

The release pipeline must attach:

- Windows NSIS installer
- SHA-256 checksum
- `AURA-2-Testing-Preview-Build.json`
- the generated npm `package-lock.json` used for that build
- the generated Rust `Cargo.lock` used for that build

The manifest records the product/version, source commit, build label, installer size, checksum, signature status, npm lock hash, Cargo lock hash and build timestamp.

## Local build fallback

While hosted runners are unavailable, a local Windows smoke candidate can be produced with:

```powershell
npm run beta:doctor:windows
npm run beta:build:windows
npm run beta:verify:artifact
```

The local build path loads the Visual Studio developer environment when available and embeds the exact source commit with the `local-beta-build / beta-local-smoke` provenance pair. A locally verified installer is useful for M009.3 smoke testing, but it does not by itself satisfy the separate hosted-CI release gate.

## First test pass

Prioritize:

1. install/uninstall as a normal Windows user
2. first-run onboarding
3. pause/resume and Recovery Safe Mode
4. local model/runtime installation
5. Chat with and without file context
6. Computer Control permission prompts
7. Voice and Vision permission boundaries
8. Agents and Automations pause/cancel behaviour
9. OBS Director Mode connection and safe controls
10. Settings → Beta & Diagnostics self-check and export

Use `docs/M009-3-WINDOWS-SMOKE-CHECKLIST.md` for the complete matrix.

## Known release infrastructure blocker

At the time this preview pipeline was prepared, GitHub-hosted Actions jobs were failing before the first workflow step with no runner logs. That is an infrastructure blocker, not a successful release validation. A preview must not be published merely because source changes are complete.

## Feedback

Use the repository bug/feature issue forms for normal testing feedback. Security vulnerabilities must use GitHub Security Advisories rather than a public issue.
