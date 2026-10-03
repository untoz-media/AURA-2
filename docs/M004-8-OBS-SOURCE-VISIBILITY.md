# M004.8 — Control Source Visibility

M004.8 gives AURA direct control over scene-item visibility in the current OBS Program scene.

## OBS API

The implementation targets OBS WebSocket v5 through `obws 0.15.0`:

- `SceneItems::list`
- `SceneItems::enabled`
- `SceneItems::set_enabled`

A visibility update uses `SetEnabled { scene, item_id, enabled }`.

## Program Sources

AURA exposes a structured list of scene items for the current Program scene.

Each item includes:

- scene name
- numeric scene item ID
- positional index
- source name
- current enabled/visible state
- input kind when available
- whether the item is a group

The source panel follows the current Program scene and refreshes every five seconds.

## UI controls

Settings → Integrations → OBS Control now includes **Program Sources**.

Every source item shows:

- source name
- input kind / group type
- scene item ID
- Visible / Hidden status
- Show / Hide action

After an explicit UI action, AURA refreshes the source list immediately.

## Stale-state protection

UI actions include the Program scene name and scene item ID that were displayed.

Before changing visibility, AURA:

1. validates that the requested scene is still the current Program scene,
2. re-reads the live scene item list,
3. confirms the requested item ID still exists,
4. sends `SetSceneItemEnabled`,
5. reads the enabled state again and verifies the result.

If Program changed between refresh and click, AURA refuses the stale action and asks for a refresh instead of modifying the old scene.

## Natural-language commands

Natural-language source visibility always targets the current Program scene.

Examples:

- `Show Lower Third`
- `Hide Scoreboard`
- `Enable Camera 2`
- `Disable Sponsor Overlay`
- `Mostra Lower Third`
- `Esconde Marcador`
- `Ativa Câmara 2`
- `Desativa Sponsor Overlay`

Names are matched exactly, case-insensitively.

If no matching source exists, the command fails closed. If more than one scene item with the same source name exists in the Program scene, AURA reports the ambiguity instead of choosing one.

## Permission model

Source visibility commands are classified as **Act** and therefore respect the existing AURA permission policy and confirmation flow.

Existing deterministic commands keep priority. For example, `Show windows` still routes to Windows window discovery rather than an OBS source called “windows”.

## Scope boundary

M004.8 controls scene-item visibility only.

- audio levels and mute state → M004.9
- bitrate, dropped frames and production health → M004.10
- Director Mode multi-action presets → M004.11

## Validation

Regression coverage includes:

- English and Portuguese show/hide routing
- Act permission classification
- Act policy overrides
- preservation of existing Windows routing
- source-name validation
- visibility-result serialization

The repository's GitHub Actions runner provisioning issue still prevents the Windows workflow from reaching its first step.
