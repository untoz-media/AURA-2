# M004.2 — Detect OBS State

M004.2 makes AURA aware of the current OBS production state instead of only knowing whether the WebSocket connection exists.

## Detected state

AURA now reads:

- streaming active / inactive
- recording active / inactive
- recording paused
- Studio Mode enabled / disabled
- current Program scene
- current Preview scene when Studio Mode is enabled

## Refresh model

The desktop bridge polls OBS every two seconds while the integration is connected.

A manual **Refresh** action is also available in **Settings → Integrations → OBS Control**.

If a runtime request fails, AURA marks the OBS integration unavailable and exposes the error instead of continuing to display stale production state.

## Backend

The OBS controller exposes an `ObsRuntimeState` snapshot through:

- `get_obs_runtime_state`

The snapshot intentionally focuses on production state. Stream/record controls remain in M004.5/M004.6, stream duration remains in M004.7 and production health metrics remain in M004.10.

## UI

The live state panel displays:

- Streaming
- Recording
- Studio Mode
- Program Scene
- Preview Scene

This creates the state foundation required for scene discovery and scene switching in M004.3 and M004.4.
