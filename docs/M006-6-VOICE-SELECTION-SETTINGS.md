# M006.6 — Voice Selection & Settings

M006.6 turns Voice into a configurable local subsystem instead of a fixed demo path.

## Persistent preferences

AURA stores Voice settings locally in `voice-preferences.json`.

Preferences include:

- automatic spoken replies
- TTS speed
- selected TTS voice
- Conversation Mode
- conversation timeout
- optional Wake Phrase
- wake phrase text

Values are sanitized before persistence.

## Local voice selection

AURA ships model profiles for two Piper voices:

- `voice-piper-ptpt` — Tugão Medium · Português (Portugal)
- `voice-piper-engb-alan` — Alan Medium · English (UK)

Both remain feature models with role `textToSpeech` and cannot become the reasoning model.

Only the selected voice is used for synthesis. Switching voice stops the active Piper worker so the next response reloads the correct ONNX model.

## Voice settings

Settings → Voice now exposes:

- microphone selection
- Push-to-Talk shortcut
- STT model setup
- TTS runtime setup
- local voice selection
- automatic spoken replies
- speech speed from 0.60× to 1.50×
- conversation timeout
- Conversation Mode
- Wake Phrase
- test voice
- Stop speaking

All settings remain local.
