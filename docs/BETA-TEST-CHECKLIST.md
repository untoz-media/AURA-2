# AURA-2 Beta Test Checklist

Use this checklist on a clean or representative Windows x64 machine before the Public Beta.

## Installer

- [ ] NSIS installer opens
- [ ] current-user installation completes without Administrator elevation
- [ ] AURA launches after installation
- [ ] application icon and Start Menu entry are correct
- [ ] installed version matches release metadata
- [ ] SHA-256 matches the published checksum
- [ ] downgrade protection rejects an older installer over a newer build

## Core lifecycle

- [ ] main window opens and closes correctly
- [ ] tray behaviour works
- [ ] Background Mode works
- [ ] optional Start with Windows setting persists
- [ ] Global Pause blocks new actions
- [ ] resuming AURA restores normal execution

## Settings & permissions

- [ ] Read / Act / Modify can be changed
- [ ] Sensitive has no permanent Allow option
- [ ] Destructive has no permanent Allow option
- [ ] Block prevents the corresponding action
- [ ] Ask every time produces an explicit confirmation flow
- [ ] Restore defaults requires confirmation
- [ ] policy survives app restart

## Managed runtime & models

- [ ] Managed Runtime installs
- [ ] runtime verification reports Python/PyTorch/Transformers state
- [ ] runtime repair works
- [ ] AURA-1 download starts
- [ ] model download pause/resume works
- [ ] model download cancel leaves no finalized broken install
- [ ] interrupted partial download can resume
- [ ] AURA-1 can be selected
- [ ] free-form local Chat produces a response
- [ ] New conversation clears conversation context

## Computer Control

- [ ] launch a supported app
- [ ] switch to a supported app/window
- [ ] keyboard action obeys Modify policy
- [ ] mouse action obeys Modify policy
- [ ] media/audio actions work
- [ ] sensitive/destructive system actions require the expected confirmation

## Memory & context

- [ ] create and delete a memory
- [ ] current app/window context refreshes
- [ ] recent-file context loads
- [ ] create/edit/run a user routine
- [ ] project memory can be created and selected

## Voice

- [ ] microphone list loads
- [ ] input meter/test works
- [ ] Whisper STT model installs
- [ ] `Ctrl + Shift + F8` captures and transcribes speech
- [ ] Piper voice installs and speaks
- [ ] Stop Speaking interrupts TTS
- [ ] Conversation Mode can be enabled/disabled
- [ ] optional wake phrase respects settings

## Vision

- [ ] AURA Vision model installs
- [ ] full-screen capture works only after explicit action
- [ ] active-window capture works
- [ ] `Ctrl + Shift + F9` region selection works
- [ ] local analysis returns an answer
- [ ] Read=Block prevents the expected Vision access path
- [ ] Vision history can be cleared

## OBS / Director Mode

- [ ] OBS connects/disconnects
- [ ] scene list refreshes
- [ ] Program/Preview scene changes work
- [ ] source visibility works
- [ ] audio mute/volume controls work
- [ ] recording start/pause/resume/stop works in a test profile
- [ ] production health check refreshes
- [ ] Director preset can be created and run
- [ ] sensitive Director operations are not silently executed in background

## Agents

- [ ] planner returns only supported step types
- [ ] unsupported goals fail safely
- [ ] plan shows highest permission
- [ ] confirmation-required plan cannot start without approval
- [ ] Blocked plan cannot run
- [ ] Pause/Resume works
- [ ] Cancel stops at a safe boundary
- [ ] Launch/Switch retry at most once on transient failure
- [ ] Routine/Director steps are not blindly retried
- [ ] interrupted run is recovered as Interrupted after restart

## Saved Actions & Automations

- [ ] create/edit/delete Saved Action
- [ ] duplicate name/alias is rejected
- [ ] Action referenced by Automation cannot be deleted
- [ ] startup trigger works
- [ ] interval trigger works
- [ ] future date/time trigger works
- [ ] app-focus trigger works
- [ ] unsafe Modify/Sensitive/Destructive background Action cannot be saved
- [ ] changing permission from Allow to Ask/Block stops silent execution
- [ ] Global Pause suspends scheduled execution

## Privacy

- [ ] Privacy page states Product telemetry Off
- [ ] no analytics SDK is present in release manifests
- [ ] crash reporting is manual only
- [ ] cloud assistance remains Off/not configured

## Regression / release gates

- [ ] `npm run release:check` passes
- [ ] `npm run telemetry:check` passes
- [ ] `npm run beta:source-check` passes
- [ ] `npm run frontend:build` passes
- [ ] Rust Core tests pass on the Windows build runner
- [ ] Windows NSIS build succeeds

## Sign-off

- [ ] no Critical release blocker remains
- [ ] known limitations are documented
- [ ] Beta version/release notes are final
- [ ] installer + checksum are attached to the approved release
