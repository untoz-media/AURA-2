# M004.11 — Director Mode Presets

M004.11 completes the first AURA-2 OBS Control / Director Mode milestone.

Director Mode turns the individual OBS controls implemented in M004.1–M004.10 into reusable multi-step production routines.

## Persistent local presets

Presets are stored locally in AURA's application configuration directory as:

`director-presets.json`

No cloud service is required.

Each preset contains:

- stable local ID
- name
- optional description
- optional command aliases
- ordered actions
- last-updated timestamp

AURA supports up to 64 saved presets, 32 actions per preset and 12 aliases per preset.

## Supported actions

A Director Mode preset can contain:

1. **Program Scene**
2. **Preview Scene**
3. **Source Visibility**
4. **Audio Mute / Unmute**
5. **Audio Volume**
6. **Recording Start / Stop / Pause / Resume**
7. **Streaming Start / Stop**
8. **Wait**

Wait steps are limited to 30 seconds each.

## Ordered execution

Preset actions run sequentially in the saved order.

This matters for source visibility. A source visibility action targets the OBS Program scene that is active at that point in the sequence.

Example:

1. Program → Match
2. Show Scoreboard
3. Unmute Commentary
4. Commentary → 80%
5. Recording → Start

## Fail-fast behavior

Director Mode is deliberately fail-fast.

If a step fails:

- the preset stops immediately,
- later steps are not executed,
- AURA reports the failed step,
- completed steps remain completed.

AURA does **not** attempt automatic rollback.

Production actions such as scene changes, recording stops and external stream state can have irreversible real-world effects, so guessing a rollback sequence would be unsafe.

## Idempotent recording and streaming steps

Recording and streaming actions avoid failing when the output is already in the requested state.

Examples:

- Start Recording while already recording → skipped
- Stop Recording while already stopped → skipped
- Start Streaming while already live → skipped
- Stop Streaming while already offline → skipped

Pause and Resume still fail when no recording exists because there is no valid state transition.

## Permissions

Director Mode permissions are derived from the saved actions.

A preset containing **Streaming → Start** is classified as **Sensitive** for natural-language execution because it can publish audio/video externally.

Presets without a stream-start action are classified as **Act**.

The normal AURA confirmation policy therefore applies automatically.

### Permission revalidation

AURA re-loads the preset immediately before execution.

If a preset was originally routed as Act but is edited to include Streaming → Start before execution begins, AURA stops and asks the user to run the command again so that Sensitive confirmation can occur.

## Natural-language execution

Existing deterministic commands always have priority.

Only when no normal M003/M004 command matches does AURA try to resolve the text as a Director Mode preset.

Presets can be executed by name:

- `Run preset Prepare Match`
- `Run director preset Go to Break`
- `Execute preset End Broadcast`
- `Executa o preset Preparar Jogo`

Aliases can also be invoked directly.

Example preset:

- Name: `Prepare Match`
- Alias: `prepare the match`

The user can simply say:

`prepare the match`

## Preset editor

Settings → Integrations → OBS Control now contains **Director Mode → Production Presets**.

The editor supports:

- create preset
- edit preset
- delete preset
- name and description
- comma-separated aliases
- add actions
- reorder actions
- remove actions
- connected-OBS suggestions for scenes, sources and audio inputs
- run preset
- last-run result

The last-run panel shows every executed step as:

- applied
- skipped
- failed

along with the result message.

## Explicit UI execution

The Run button is an explicit user action and is only enabled while AURA is connected to OBS.

Presets that can start streaming are visibly marked **Can go live / Sensitive**.

## Validation

Preset validation covers:

- non-empty names
- name length
- alias normalization and deduplication
- conflicts between names and aliases across presets
- action count limits
- source/scene/input text validation
- audio volume 0–100%
- wait duration limit
- Sensitive detection
- bare alias resolution
- explicit preset command resolution

## Release

M004.11 completes:

**M004 — OBS Control / Director Mode**

Release line:

**AURA-2 0.4.0-alpha.1**

The next roadmap block is:

**M005 — Memory & Context**
