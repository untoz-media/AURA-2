# M004.6 — Start / Stop Streaming

M004.6 adds direct OBS streaming control to AURA-2.

## Supported actions

AURA can now:

- start the configured OBS stream
- stop the active OBS stream

The implementation uses OBS WebSocket v5 through `obws 0.15.0`:

- `Streaming::status`
- `Streaming::start`
- `Streaming::stop`

## Safety model

Starting a stream can publish audio and video outside the computer.

For that reason, natural-language **Start Streaming / Go Live** commands are classified as `PermissionClass::Sensitive`.

The default AURA policy therefore requires confirmation before going live, and Sensitive actions cannot be permanently configured as Allow.

Stopping a stream is classified as `PermissionClass::Act`, so the default policy can stop a live output immediately.

UI buttons are explicit user actions and call the OBS integration directly.

## State validation

AURA reads the live streaming status before every action:

- Start rejects if OBS is already streaming.
- Stop rejects if OBS is not currently streaming.

After sending Start/Stop, AURA polls `Streaming::status` through a short bounded retry window and only reports success after OBS confirms the expected state.

## AURA commands

Examples include:

- `Go live`
- `Start streaming`
- `Start the stream`
- `Stop the stream`
- `End stream`
- `Entra em direto`
- `Inicia a transmissão`
- `Para a transmissão`
- `Termina o direto`

## Settings UI

The OBS live-state panel now exposes a contextual **Streaming Control** section:

- **Go Live** while offline
- **Stop Stream** while live

The UI disables concurrent stream actions while a request is in flight and refreshes the runtime state immediately after completion.

## Scope boundary

M004.6 controls stream start/stop only.

Stream duration remains isolated to **M004.7 — Read Stream Duration**. Production metrics and health remain in M004.10.

## Validation

Regression coverage includes:

- English and Portuguese stream routing
- Sensitive classification for Start
- Act classification for Stop
- confirmation defaults
- prevention of permanent Sensitive Allow
- streaming result serialization

The repository's GitHub Actions runner provisioning problem still prevents the Windows workflow from reaching its first step.
