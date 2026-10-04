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
