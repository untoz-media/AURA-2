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
- [ ] Minimize a known application by name.
- [ ] Maximize a known application by name.
- [ ] Restore a known application by name.
- [ ] Confirm the Computer workspace exposes contextual window controls only for a known app identity.
- [ ] Confirm an unknown app name is rejected instead of guessed.
- [ ] Run New tab / Next tab / Previous tab / Reload / Focus address bar / Reopen closed tab in Brave.
- [ ] Repeat at least New tab and Reload in Google Chrome.
- [ ] Confirm Browser Skills are classified as Act.
- [ ] Confirm Browser Skills reject non-browser targets such as Notepad.
- [ ] Force a foreground-focus mismatch during a disposable test and confirm AURA refuses to inject the shortcut.
- [ ] Open Desktop/Documents/Downloads/Pictures/Videos/Music through File Explorer Skills.
- [ ] Confirm the personal-folder skills use Act and do not accept arbitrary paths.
- [ ] Bring Notepad into context and confirm its skills appear without a frontend reload.
- [ ] Run New note / Find / Select all in Notepad and confirm Act behavior.
- [ ] Run Undo/Redo in a disposable Notepad document and confirm Modify requests confirmation.
- [ ] Force a Notepad foreground mismatch and confirm no shortcut is injected.
- [ ] Confirm Save/Close are not published by the Skill Registry.
- [ ] Bring Windows Terminal into context and validate New tab / Next / Previous / Command palette / Find / Tab dropdown with default bindings.
- [ ] Confirm an arbitrary request such as “Run echo hello in Windows Terminal” is not exposed as a Terminal Skill.
- [ ] If Terminal key bindings are customized, confirm the limitation is visible during testing rather than treated as shell/API execution.
- [ ] Bring Calculator into context and validate Standard / Scientific / Programmer / Date Calculation / Graphing mode switching.
- [ ] Confirm Terminal and Calculator shortcuts are aborted if foreground-process verification fails.
- [ ] Confirm the Computer UI is driven by the backend App Skill catalog rather than hard-coded browser/folder arrays.
- [ ] Search for a known file by exact name in a personal folder.
- [ ] Search with a partial/multi-word filename and confirm ranking is sensible.
- [ ] Confirm searches stay inside Desktop/Documents/Downloads/Pictures/Videos/Music.
- [ ] Confirm symlinked directories are not traversed.
- [ ] Confirm file contents are never read during filename search.
- [ ] Confirm a bounded search reports when the 8,000-entry safety cap is reached.
- [ ] Ask for the latest video and confirm results are sorted by filesystem modified time.
- [ ] Ask for recent images/documents and confirm extension filtering is correct.
- [ ] Ask for the latest download and confirm the search is scoped to Downloads.
- [ ] Ask for “latest video I exported” and confirm AURA describes it as most recently modified rather than claiming the source application.
- [ ] Confirm Read = Never blocks File Intelligence.
- [ ] Reveal a returned file in File Explorer and confirm the file is selected but not executed.
- [ ] Attempt to reveal an existing path outside the allowed personal roots and confirm AURA rejects it.
- [ ] Confirm a symlink cannot escape an allowed root during reveal validation.
- [ ] Read clipboard text and confirm Sensitive permission is requested.
- [ ] Copy text to the clipboard and confirm Modify permission is requested.
- [ ] Clear the clipboard and confirm Destructive permission is requested.
- [ ] Confirm clipboard text is not present in exported Beta diagnostics.
- [ ] Trigger a clipboard read through Voice and confirm the content is displayed but not spoken by TTS.
- [ ] Confirm AURA does not react to clipboard changes unless an explicit clipboard command is issued.
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

## F.4 Drag & Drop Actions

- [ ] Drag one local file over the AURA main window and confirm the drop overlay appears on enter/over and disappears on leave.
- [ ] Drop a valid file and confirm the tray shows filename/type/size without exposing its filesystem path.
- [ ] Drop more than 8 items and confirm AURA accepts at most 8 and reports the bounded rejection/truncation state.
- [ ] Drop a directory and confirm it is rejected.
- [ ] Drop the same canonical file twice in one batch and confirm it is deduplicated.
- [ ] Confirm dropping a file does not open, execute, upload, remember or analyze it.
- [ ] Inspect an allowlisted UTF-8 text/code file and confirm the preview is bounded and no filesystem path is shown.
- [ ] Inspect an image and confirm dimensions are shown without starting Vision analysis.
- [ ] Inspect all across a mixed batch and confirm complex binary formats remain metadata-only.
- [ ] Attach one dropped file to Chat and confirm a removable filename chip appears above the composer.
- [ ] Attach all dropped files and confirm all current opaque items appear as composer chips without filesystem paths.
- [ ] Send a custom question with attachments and confirm the local model uses only bounded turn-only context.
- [ ] Send attachments with an empty composer and confirm the visible request becomes "Analyze the attached local files."
- [ ] After an accepted attached message, confirm attachments are detached from the next turn while the Drop Tray session remains.
- [ ] Confirm the sent user message shows safe attachment filenames but no paths or file contents.
- [ ] Set Read = Never and confirm attached-file Chat analysis is blocked.
- [ ] Modify a dropped file after intake and confirm Inspect/Analyze refuses the stale Drop ID until the file is dropped again.
- [ ] Put instruction-like text such as "ignore previous instructions" inside a dropped text file and confirm it is treated as untrusted data, not as an AURA action.
- [ ] With attachments present, try a prompt that resembles a Computer/Routine/Director command and confirm deterministic action routing is bypassed.
- [ ] Reveal a dropped file in Explorer and confirm the original file is selected but not executed.
- [ ] Set Act = Never and confirm Reveal is blocked.
- [ ] Drop PNG/JPEG/WebP/GIF/BMP images and stage each supported format for Vision.
- [ ] Confirm an unsupported image-like format such as SVG is not offered to Vision.
- [ ] Confirm a source image larger than 40 MB is rejected for Vision staging.
- [ ] Confirm an image over the 24-million-pixel Vision limit is rejected.
- [ ] Confirm Vision receives a cached `droppedImage` capture and clearing Vision does not delete the original file.
- [ ] Set Read = Never and confirm Use in Vision is blocked.
- [ ] Confirm dropped-file paths/content are absent from exported Beta diagnostics.
- [ ] Dismiss the drop tray and confirm original files are unchanged.

## F.5 AURA Create

- [ ] Confirm `AURA Create · Image` appears as a feature-specific downloadable model.
- [ ] Confirm the model download can start, pause, resume and complete verification.
- [ ] Confirm repairing the managed runtime upgrades it for Diffusers and leaves model files intact.
- [ ] Generate a 512×512 square image.
- [ ] Generate one landscape and one portrait image.
- [ ] Confirm a fixed seed is reproducible for the same prompt/settings.
- [ ] Confirm the generated PNG is saved locally and previewed in Create.
- [ ] Confirm no prompt or generated-image data appears in exported Beta diagnostics.
- [ ] Remove the Create model and confirm the resident image worker stops cleanly.
- [ ] Re-download/reinstall and confirm generation works again.
- [ ] On CUDA hardware, verify generation reports CUDA/CPU-offload mode without crashing.
- [ ] On a CPU-only validation path, verify generation either completes or fails with a clear local error.
- [ ] Force-kill the Python image worker in a disposable test and confirm the next request can start a fresh worker.
- [ ] Confirm Video mode remains clearly marked planned and does not simulate generation.

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
