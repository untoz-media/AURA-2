# M004.5 — Start / Stop Recording

M004.5 adds direct OBS recording control to AURA-2.

## Supported actions

AURA can now:

- start recording
- stop recording
- pause recording
- resume recording

The implementation uses the OBS WebSocket v5 record requests exposed by `obws 0.15.0`:

- `Recording::start`
- `Recording::stop`
- `Recording::pause`
- `Recording::resume`
- `Recording::status`

## State validation

Every action checks the current recording state before execution.

Examples:

- Start fails if OBS is already recording.
- Stop fails if no recording is active.
- Pause fails if recording is not running or is already paused.
- Resume fails if recording is not paused.

After each action, AURA polls the OBS recording status with a short bounded retry window and confirms the expected active/paused state.

This avoids treating a request acknowledgement as proof that the final OBS state changed.

## Stop result

`Recording::stop` returns the recording output path.

AURA exposes that path through `ObsRecordingActionResult.output_path`, displays the last saved recording in Settings and includes the path in AURA activity/command feedback when available.

## AURA commands

Deterministic English and Portuguese commands include:

- `Start recording`
- `Stop recording`
- `Pause recording`
- `Resume recording`
- `Inicia a gravação`
- `Começa a gravar`
- `Para a gravação`
- `Pausa a gravação`
- `Retoma a gravação`

Recording actions use the existing **Act** permission class and therefore respect the user's AURA permission policy and confirmation flow.

## Settings UI

The live OBS state panel now exposes contextual recording actions:

- **Start Recording** when stopped
- **Pause** while recording
- **Resume** while paused
- **Stop Recording** while recording or paused

The UI disables concurrent recording actions while a request is in flight and refreshes the OBS runtime state immediately after completion.

## Scope boundary

M004.5 controls recording only.

Streaming start/stop remains isolated to **M004.6 — Start / Stop Streaming**.

## Validation

Regression coverage includes:

- recording command routing
- English and Portuguese commands
- Act permission classification
- Act policy overrides
- recording result serialization

The GitHub Actions runner provisioning issue still prevents the Windows Rust/TypeScript/NSIS workflow from reaching its first step.
