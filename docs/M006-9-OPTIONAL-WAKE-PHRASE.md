# M006.9 — Optional Wake Phrase

M006.9 adds an experimental fully local hands-free activation mode.

## Default

Wake Phrase is OFF by default.

Default phrase:

`AURA`

The phrase is configurable up to 32 characters.

## Alpha implementation

This first implementation reuses the existing local Whisper STT runtime for phrase spotting.

AURA:

1. captures short local microphone windows
2. rejects very quiet windows
3. transcribes locally
4. checks whether the transcription starts with the configured wake phrase
5. if a command follows the phrase, submits it to AURA Core
6. if only the wake phrase is heard, opens a Conversation Mode follow-up window

Example:

`AURA, abre o Brave`

can route directly to `abre o Brave`.

## Trade-off

Whisper phrase spotting is more computationally expensive than a dedicated tiny wake-word model.

For that reason:

- it is opt-in
- it is clearly labelled experimental
- it requires the local STT model
- it pauses while AURA is speaking
- manual Push-to-Talk preempts it

A later release can replace this detector with a dedicated OpenWakeWord-compatible model without changing the Voice → AURA Core boundary.
