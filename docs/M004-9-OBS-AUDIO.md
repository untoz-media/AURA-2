# M004.9 — Control Audio Levels

M004.9 adds per-input audio control to the AURA OBS integration.

## OBS API

The implementation targets OBS WebSocket v5 through `obws 0.15.0`:

- `Inputs::list`
- `Inputs::muted`
- `Inputs::set_muted`
- `Inputs::volume`
- `Inputs::set_volume`
- `Volume::Mul`

OBS returns both multiplier and dB values for an input volume. AURA uses the multiplier as the canonical writable value and maps:

- 0% → 0.0 mul
- 50% → 0.5 mul
- 100% → 1.0 mul

The OBS-provided dB value remains visible for diagnostics.

## Audio input discovery

AURA lists OBS inputs and keeps only inputs that successfully expose both mute and volume state.

Each controllable audio input includes:

- input name
- input UUID
- input kind
- muted state
- volume percentage
- volume multiplier
- volume in dB

Inputs are refreshed every five seconds while OBS is connected.

## UI mixer

Settings → Integrations → OBS Control now includes **Audio Inputs**.

Each input exposes:

- name and OBS input kind
- current dB value
- 0–100% volume slider
- explicit **Set** button
- Muted / Live state badge
- **Mute / Unmute** button

Moving the slider only changes a local draft. OBS volume is not modified until **Set** is pressed.

After a successful mute or volume action, AURA refreshes the audio-input snapshot immediately.

## Natural-language commands

Examples:

- `Mute Mic/Aux`
- `Unmute Desktop Audio`
- `Set Mic/Aux to 70%`
- `Set Commentary volume to 45%`
- `Set volume of Desktop Audio to 60%`
- `Silencia Mic/Aux`
- `Define Desktop Audio para 45%`

Input names are matched exactly, case-insensitively.

Plain `Mute` / `Unmute` remains part of M003 system-media control and is not intercepted by OBS audio routing.

## Permission model

OBS audio changes are classified as **Act** and therefore respect the existing AURA permission policy and confirmation flow.

## Validation

Before UI actions, AURA re-lists the OBS inputs and resolves the displayed UUID again.

After changing mute state, AURA reads the state back and verifies it.

After changing volume, AURA reads the volume back and verifies the requested multiplier within a small tolerance before reporting success.

Natural-language actions resolve the input by exact name and verify that it exposes volume control.

## Scope boundary

M004.9 controls configured input volume and mute state.

Real-time VU meters, clipping analysis, dropped frames, bitrate and broader production monitoring remain part of **M004.10 — Production Health Checks**.

## Regression coverage

Coverage includes:

- multiplier → percentage conversion
- audio result serialization
- English and Portuguese routing
- Act permission classification
- Act policy overrides
- preservation of plain system `Mute`

The repository's GitHub Actions runner provisioning issue still prevents the Windows workflow from reaching its first step.
