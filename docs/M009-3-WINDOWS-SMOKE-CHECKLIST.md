# M009.3 — Windows Beta Smoke Checklist

Use this checklist against the exact installer produced by:

```powershell
npm run beta:build:windows
npm run beta:verify:artifact
```

Record the installer SHA-256 and source commit from `AURA-2-Beta-Build.json` before testing.

## A. Clean install

- [ ] Install as a standard/current user without an Administrator prompt.
- [ ] Confirm AURA-2 appears in the Start Menu under Untoz.
- [ ] Confirm the installed version is `0.9.0-beta.1`.
- [ ] Confirm launching from Start Menu opens the main window.
- [ ] Confirm the first-run Public Beta onboarding appears.
- [ ] Confirm onboarding states local-first behavior, explicit model installation, permission boundaries and zero automatic telemetry/crash uploads.

## B. Core desktop lifecycle

- [ ] Main window opens and can be closed to tray when Background Mode is enabled.
- [ ] Tray menu can reopen AURA.
- [ ] Overlay opens from the configured shortcut.
- [ ] Pause/Resume changes runtime state correctly.
- [ ] Optional Windows autostart can be enabled and disabled.
- [ ] A clean exit is reported as Clean on the next launch.

## C. Recovery

- [ ] Force-close AURA while it is running.
- [ ] Relaunch and confirm Previous session reports Recovered.
- [ ] Confirm AURA starts in the paused state after the unclean exit.
- [ ] Confirm no Agent/Automation action executes until AURA is explicitly resumed.
- [ ] Confirm no crash report is uploaded.
- [ ] Confirm interrupted Agent runs are recovered as interrupted/failed rather than left running.
- [ ] Resume AURA manually and confirm normal execution returns.
- [ ] Relaunch after a normal exit and confirm the recovery warning clears.

## D. Permissions and safety

- [ ] Review Read, Act, Modify, Sensitive and Destructive permission levels.
- [ ] Verify a Never decision blocks the corresponding action.
- [ ] Verify Ask creates a one-shot confirmation.
- [ ] Verify cancelling a confirmation performs no action.
- [ ] Verify confirmation IDs cannot be reused.
- [ ] Verify destructive/sensitive safety floors cannot be bypassed by a lower-risk policy.

## E. Computer Control

- [ ] Launch a known application.
- [ ] Switch to an existing application window.
- [ ] Type controlled text into a safe test field.
- [ ] Execute a safe keyboard shortcut.
- [ ] Move/click/scroll with bounded mouse actions.
- [ ] Read and change system volume.
- [ ] Execute a safe Windows system action.
- [ ] Confirm high-impact actions remain permission gated.

## F. Models and local runtime

- [ ] Open Models without a managed runtime installed.
- [ ] Install or repair the private AURA runtime.
- [ ] Confirm runtime status reaches Ready.
- [ ] Download an available model.
- [ ] Pause and resume a model download.
- [ ] Restart AURA during a partial download and confirm resume state survives.
- [ ] Select an installed model.
- [ ] Run a free-form local prompt.
- [ ] Start a new conversation and confirm chat context clears without deleting the model.
- [ ] Remove a model and confirm active-model state remains valid.

## G. Voice

- [ ] Select a microphone.
- [ ] Run the microphone input test.
- [ ] Use Push-to-Talk.
- [ ] Confirm local STT returns text.
- [ ] Confirm TTS can speak a response.
- [ ] Interrupt active speech.
- [ ] Toggle Conversation Mode.
- [ ] Verify wake phrase behavior only when enabled.

## H. Vision

- [ ] Capture the full screen with Read permission allowed.
- [ ] Capture the active window.
- [ ] Capture a bounded region.
- [ ] Analyze a capture locally.
- [ ] Verify Read = Never blocks every Vision capture path.
- [ ] Clear Vision history and confirm it is removed locally.

## I. Memory and context

- [ ] Create and delete a local Memory item.
- [ ] Confirm current-app awareness updates.
- [ ] Confirm recent-file context can be refreshed.
- [ ] Save and activate Project Memory.
- [ ] Create and run a user routine.

## J. Agents and Automations

- [ ] Plan a multi-step goal.
- [ ] Start the plan and observe live run status.
- [ ] Pause/cancel a run.
- [ ] Save and run an AURA Action.
- [ ] Create an event-triggered Automation.
- [ ] Create a scheduled Automation.
- [ ] Disable and re-enable an Automation.
- [ ] Pause AURA and confirm background automation execution respects pause state.

## K. Director Mode / OBS

- [ ] Connect to a local OBS WebSocket instance.
- [ ] Read current Program/Preview scenes.
- [ ] Switch Program scene.
- [ ] Toggle source visibility.
- [ ] Mute/unmute an audio input.
- [ ] Change an audio level.
- [ ] Start/stop a short local recording.
- [ ] Run a saved Director Mode preset.
- [ ] Disconnect cleanly.

OBS testing may be marked N/A only for a Beta validation machine where OBS is intentionally unavailable; it does not replace a dedicated Director Mode validation pass.

## L. Beta self-check and diagnostics

- [ ] Launch AURA and confirm the startup health report runs without blocking normal startup.
- [ ] Open Settings → Beta & Diagnostics.
- [ ] Confirm diagnostics schema version 2 is displayed through a valid snapshot.
- [ ] Confirm configuration storage and Local Data write probes pass.
- [ ] Confirm Session marker and Beta preferences checks pass.
- [ ] Confirm Permission safety floor passes.
- [ ] Confirm Model catalog, Agent run store, Saved Actions store and Automation store checks pass.
- [ ] Confirm Managed runtime state, Privacy boundary and Runtime counters checks pass.
- [ ] Confirm Beta self-check reports Healthy when every subsystem check passes.
- [ ] Confirm telemetry is Off.
- [ ] Confirm automatic crash uploads are Off.
- [ ] Export diagnostics JSON.
- [ ] Inspect the JSON and confirm it contains no chat messages, prompts, responses, transcripts, screenshots, passwords, Memory contents or arbitrary file contents.
- [ ] Corrupt a disposable test store in a controlled test profile and confirm the report becomes Degraded without making the entire diagnostics command unavailable.
- [ ] Confirm a degraded startup health report is surfaced in Chat with a link to the local report.

## M. Installer lifecycle

- [ ] Run the same installer again and confirm repair/reinstall behavior is safe.
- [ ] Attempt to install an older build and confirm downgrade protection blocks it.
- [ ] Uninstall AURA-2.
- [ ] Confirm the application executable and Start Menu entry are removed.
- [ ] Reinstall the same Beta candidate successfully.

## Exit criteria

M009.3 can be marked complete only when:

1. `npm run beta:validate` passes;
2. Rust regression tests pass;
3. `npm run beta:build:windows` produces the NSIS installer;
4. `npm run beta:verify:artifact` passes;
5. this smoke checklist has no unresolved release-blocking failures;
6. GitHub-hosted Windows CI executes real steps and passes.

Any failure involving permissions, corrupt persistence, installer integrity, unexpected data upload, unrecoverable startup, or destructive-action bypass is a release blocker.
