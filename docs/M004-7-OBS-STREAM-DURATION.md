# M004.7 — Read Stream Duration

M004.7 gives AURA awareness of how long the current OBS stream has been live.

## OBS source

AURA reads `GetStreamStatus` through `obws::client::Streaming::status`.

The OBS WebSocket v5 payload exposes:

- `outputActive`
- `outputDuration` — current output duration in milliseconds
- `outputTimecode` — formatted OBS output timecode

AURA treats `outputDuration` as the canonical numeric value and also exposes the OBS timecode for diagnostics.

## Runtime state

`ObsRuntimeState` now includes:

- `stream_duration_ms`
- `stream_timecode`

The existing two-second runtime poll therefore synchronizes stream duration without adding another high-frequency OBS request loop.

## Dedicated query

The OBS controller also exposes `ObsStreamDuration` through:

- `get_obs_stream_duration`

This is used by direct duration queries and contains:

- whether the stream is active
- duration in milliseconds
- OBS timecode
- refresh timestamp

## AURA commands

Stream-duration questions route as **Read** permission:

- `How long have we been live?`
- `How long have we been streaming?`
- `What is the stream duration?`
- `Stream duration`
- `Há quanto tempo estamos em direto?`
- `Há quanto tempo estamos a transmitir?`
- `Qual é a duração do direto?`

When live, AURA responds using a normalized `HH:MM:SS` duration. When offline, it reports that OBS is not currently live.

## UI clock

Settings → Integrations → OBS Control now shows the live duration inside the Streaming state:

`LIVE · HH:MM:SS`

The backend remains authoritative and refreshes every two seconds. Between those snapshots, the frontend advances the clock locally once per second using the backend refresh timestamp. This keeps the display smooth without increasing OBS WebSocket traffic.

## Upstream accuracy note

AURA reports the duration OBS itself provides.

OBS/obs-websocket has upstream reports where `outputDuration` and `outputTimecode` can be inaccurate with some Enhanced Broadcasting configurations. AURA does not guess or apply an undocumented correction factor; it preserves the OBS-provided value.

## Scope boundary

M004.7 reads stream duration only.

- source visibility → M004.8
- audio levels → M004.9
- bitrate, dropped frames and production health → M004.10

## Validation

Regression coverage includes:

- stream duration protocol-field extraction
- default runtime duration state
- English and Portuguese duration commands
- Read permission classification
- Read policy overrides

The repository's GitHub Actions runner provisioning issue still prevents the Windows workflow from reaching its first step.
