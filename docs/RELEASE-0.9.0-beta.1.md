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
- diagnostics schema v3 with build provenance and structured subsystem health checks
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

Public Beta publication stays closed until a Windows machine builds the NSIS installer with `npm run beta:build:windows` and `npm run beta:verify:artifact` passes on that artifact. GitHub-hosted Actions cannot provide that evidence while the organization account is locked.

## What 0.9.0-beta.1 actually does

These capabilities are implemented in the desktop app. They still need a real Windows smoke pass before publication.

- **Voice.** Push-to-talk is Ctrl+Shift+F8. Conversation mode listens for one follow-up after a spoken reply. The wake phrase is optional, local, and off by default. Pressing push-to-talk stops speech that is already playing. When the wake phrase is enabled, AURA can also interrupt its own speech: the microphone must be louder than the idle gate, the transcript must start with the phrase, and a transcript that repeats the current spoken text is treated as playback echo and ignored. That louder gate also covers a few seconds after playback. There is no acoustic echo canceller.
- **Projects.** “Switch to project …”, “Use project …”, and “Ativa o projeto …” select a saved project by name or alias. An unknown name does not create a project.
- **Agents.** Plans, progress, pause, resume, and cancel are local. Planning needs an installed local model. Background automations run only when that action's permission is already Allow. Ask and Never do not pop a confirmation for a scheduled run.
- **Saved Actions.** Named actions and aliases are stored locally and can be run again.
- **Memory.** Explicit memories can be viewed, created, edited, and deleted. They stay on disk and are not uploaded. Chat can see the foreground app, recent file names, and the active project. That metadata is not written into the memory list and does not include file contents, screenshots, or clipboard text.
- **Vision.** Full-screen, active-window, and region capture use Windows graphics APIs and the Read permission. Read set to Never blocks capture.
- **Files.** Search looks at personal folder names and metadata, not file contents. Dropped files are copied into a local image/text cache for Vision or inspection. Dropping a file does not open or execute it. Reveal in Explorer is a separate explicit action.
- **Computer Control and App Skills.** Typed intents call fixed application targets: Brave, Chrome, Notepad, Windows Terminal, Calculator, and File Explorer. Skills send focus-verified shortcuts. Explorer opens known folders only. There is no synthesized PowerShell, cmd, or arbitrary shell command.
- **OBS / Director.** Chat actions and the Settings/Director controls use the same permission policy. Going live is Sensitive and always asks, including when the saved policy says Allow. Other mutating OBS actions follow Act. Mutating OBS and Director actions are blocked while AURA is paused. The OBS password is kept for the session only and is not written into diagnostics.
- **Diagnostics.** Export is manual. The JSON export is size-bounded and uses an allowlist. Telemetry and automatic crash uploads are hardcoded off. Session recovery and Crash Loop Guard stay local.
- **Image generation.** Local text-to-image accepts prompt, negative prompt, steps, seed, and aspect, then previews the PNG. Video generation is planned and has no generate command.

## Dependency lock

`package-lock.json` is a real npm lockfile (lockfileVersion 3) produced by `npm install`. Windows builds should use that lock instead of generating a floating one at release time. `Cargo.lock` for the desktop crate is committed from the same resolution so the Rust graph is reproducible too.

Node 24 refuses to start `npm.cmd` from `spawnSync` when `shell` is false. `scripts/validate-beta.mjs` launches that shim through `cmd.exe /d /s /c` with the fixed argument list. `cargo.exe`, `node.exe`, and the PowerShell `npm` calls in the Windows build scripts are real executables or shell commands, so they stay as they are.

Windows PowerShell 5.1 reads a UTF-8 script without a BOM using the system ANSI code page, so an em dash in `windows-build-doctor.ps1` was parsed as a stray quote. The Windows scripts are ASCII-only and do not use `&&` or `||`. They also do not assign automatic variables such as `$Host`; the Rust target triple is stored in `$rustHost`. The Visual Studio environment import writes a temporary `.cmd` and runs `set` on the next line. `scripts/build-beta-windows.ps1` still runs `cargo test --locked` after the doctor.

`apps/desktop/src-tauri/icons/icon.ico` and the PNG sizes named by `tauri.conf.json` are committed. A fresh checkout can compile and test without running `npm run icons` first. `npm run build` still refreshes the set from `aura-app-icon.svg` before the installer is produced.

`package.json` `allowScripts` permits the locked `esbuild@0.28.2` postinstall and the optional macOS `fsevents@2.3.3` install script. Those are the only two packages in the lockfile with install scripts. esbuild's script checks the platform binary that npm already installs as an optional dependency. No other dependency install script is approved.

## Local validation on Linux

Linux can typecheck the frontend, build the Vite bundle, format-check Rust, syntax-check the embedded Python workers, and cross-compile the Windows GNU target. It cannot execute the Windows test binary or produce the NSIS installer. Host `cargo test` / `cargo clippy` for the Linux target do not compile: `windows-threading` is empty off Windows, so `windows-future` fails before AURA tests start. The Windows GNU target is the compile check used for this candidate.

Do not treat a green Linux source gate as a substitute for `npm run beta:build:windows` and `npm run beta:verify:artifact`.

## Open tracker

GitHub's open count mixes issues and pull requests. The repository currently has three open issues:

- #56 is addressed by the committed `package-lock.json`. The Windows and static workflows already install with `npm ci`.
- #55 remains an organization billing lock. Jobs still fail before step 1. No repository change starts a runner.
- #57 remains a repository admin setting. `SECURITY.md` and the issue template already point at private advisory reports. Application code cannot enable Private vulnerability reporting.

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

## Panic-free Core hardening

A source-level gate now blocks panic-prone constructs from the production Rust surface.

- Action Router invariants fail closed through `RouteResult::InternalError`
- parser mismatches no longer call `expect()` or `unreachable!()`
- browser Skill Registry no longer assumes an impossible target with `unreachable!()`
- clipboard global-memory ownership returns a controlled error instead of `expect()`
- fatal Tauri runtime errors are handled explicitly instead of panicking
- `npm run beta:validate` includes the production panic-surface scan

## Recovery State repair

Recovery Safe Mode now has an explicit repair path for a damaged/stuck Beta session marker.

- tray Resume uses the same fail-closed recovery guard as the desktop UI
- blocked tray Resume opens Beta & Diagnostics directly
- **Repair recovery state** preserves the old marker as a timestamped local backup
- only recovery metadata/streak is reset
- the runtime remains paused after repair
- user Memories, models, projects, Agents and Automations are left untouched
