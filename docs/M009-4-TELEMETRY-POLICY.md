# M009.4 — Beta Telemetry Policy

AURA-2 Beta does not upload usage telemetry.

## Disabled by design

In 0.9.0-beta.1:

- usage analytics uploads: OFF
- automatic crash uploads: OFF
- automatic diagnostics uploads: OFF
- chat-content telemetry: OFF
- Voice transcript telemetry: OFF
- Vision screenshot telemetry: OFF
- Memory-content telemetry: OFF

This is not an opt-out switch hiding an active telemetry client. The Beta does not implement a telemetry upload pipeline.

## Local diagnostics

Users can manually generate a local diagnostics snapshot.

The snapshot contains operational metadata such as:

- app version
- OS and architecture
- paused/background/autostart state
- active model ID
- installed model IDs
- managed runtime state
- aggregate Agent run counts
- Saved Action count
- Automation count

The diagnostics snapshot intentionally excludes:

- chat messages
- model prompts/responses
- Voice transcripts/audio
- screenshots
- Memory content
- project notes
- OBS passwords
- arbitrary file paths/content

The JSON file is written locally only after an explicit Export action.

Before writing, AURA applies a fail-closed privacy guard:

- diagnostics schema fields are allowlisted;
- health-check fields are allowlisted;
- unexpected fields block export instead of being written;
- the exported payload is capped at 64 KiB;
- export failures are surfaced locally through the AURA bridge/activity state.

This means future code cannot silently expand the diagnostics export schema without also updating and reviewing the privacy boundary.
