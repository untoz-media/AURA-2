# M004.3 — List Scenes

M004.3 gives AURA a structured view of every scene in the connected OBS Studio instance.

## Scene discovery

AURA now uses the OBS WebSocket `GetSceneList` request through `obws::client::Scenes::list`.

The response includes the full scene array plus the current Program and Preview scene identifiers.

For every scene AURA stores:

- scene name
- scene UUID
- positional index
- whether the scene is currently Program
- whether the scene is currently Preview

## Backend

The OBS controller exposes an `ObsSceneList` snapshot through:

- `get_obs_scenes`

The snapshot contains the ordered scene list, current Program/Preview names, refresh timestamp and any scene-list-specific error.

## Refresh model

Scene discovery refreshes automatically every five seconds while OBS is connected.

A manual **Refresh scenes** action is also available in **Settings → Integrations → OBS Control**.

The scene list uses a slower refresh cadence than the two-second production-state polling because scene creation, deletion and renaming are less frequent than live stream/record state changes.

## UI

The OBS integration now shows a Scenes section containing:

- scene position
- scene name
- scene UUID
- Program badge
- Preview badge

No scene-switch action is performed in M004.3. Scene switching remains isolated to **M004.4 — Change Scenes**.

## API compatibility

The implementation targets `obws 0.15.0` / OBS WebSocket v5. The crate's `Scenes::list` response exposes `current_program_scene`, `current_preview_scene` and the ordered `scenes` collection.
