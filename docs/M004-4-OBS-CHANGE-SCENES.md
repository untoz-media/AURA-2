# M004.4 — Change Scenes

M004.4 turns OBS scene awareness into real production control.

## Program switching

AURA can now set any discovered OBS scene as the current Program scene.

Before every switch, AURA re-reads the OBS scene list and verifies that the requested scene still exists. After the request is sent, it reads the current Program scene again and verifies that OBS actually accepted the change.

This prevents stale UI state from sending a blind scene switch.

## Preview switching

When OBS Studio Mode is enabled, AURA can also set the current Preview scene.

Preview switching is rejected when Studio Mode is disabled.

## Scene identification

UI actions use the scene UUID discovered in M004.3.

The backend resolves that UUID against the current OBS scene list immediately before the action, then sends the actual OBS scene identifier to `Scenes::set_current_program_scene` or `Scenes::set_current_preview_scene`.

Natural-language actions use exact case-insensitive scene-name matching against the current OBS scene list.

## AURA commands

M004.4 adds deterministic scene routing through the AURA Core permission engine.

Examples:

- `Switch scene to Camera 2`
- `Switch to Camera 2`
- `Set program scene to Intro`
- `Set preview scene to Interview`
- `Muda para a cena Câmara 1`
- `Prepara a cena Entrevista`

Known application targets retain priority. For example, `Switch to OBS` still switches to the OBS Studio application. An unknown switch target falls back to exact OBS Program-scene matching.

Scene commands use the **Act** permission class.

## Settings UI

Each discovered scene now exposes:

- **Take Program** when it is not currently Program
- **Set Preview** when Studio Mode is enabled and it is not currently Preview
- Program / Preview status badges

After a successful UI switch, AURA immediately refreshes runtime and scene state instead of waiting for the next polling interval.

## Validation

M004.4 adds regression coverage for:

- explicit Program scene commands
- Preview scene commands
- `Switch to <scene>` fallback routing
- Act permission classification
- Act policy overrides
- scene UUID validation
- scene-name validation

The repository's GitHub Actions runner provisioning issue still prevents the real Windows Rust/TypeScript/NSIS gate from starting.
