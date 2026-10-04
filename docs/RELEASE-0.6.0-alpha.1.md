# AURA-2 0.6.0-alpha.1 — Voice

AURA-2 0.6.0-alpha.1 completes M006 — Voice.

## What is new

AURA can now:

- detect and select Windows microphones
- test live microphone input locally
- capture voice with global Push-to-Talk
- transcribe locally with Whisper Base
- route transcribed commands through the existing AURA Core and Permission Engine
- speak voice-originated responses locally with Piper
- switch between Portuguese (Portugal) Tugão and English (UK) Alan voices
- adjust TTS speed
- continue with Conversation Mode follow-ups
- interrupt speech with Stop speaking
- barge in by pressing Push-to-Talk while AURA is talking
- optionally use an experimental local Wake Phrase
- persist Voice preferences locally

## Default privacy posture

- Push-to-Talk remains the primary interaction mode
- Wake Phrase is off by default
- microphone audio is not uploaded
- voice captures are not persisted automatically
- STT runs locally
- TTS runs locally
- typed commands remain silent by default
- Voice commands keep the same permissions as typed commands

## Models

### Speech-to-text

`openai/whisper-base`

Role: `speechToText`

### Text-to-speech

`voice-piper-ptpt`

Tugão Medium · Portuguese (Portugal)

`voice-piper-engb-alan`

Alan Medium · English (UK)

Role: `textToSpeech`

Feature models cannot become the main AURA assistant model.

## Voice flow

```
Microphone
→ Push-to-Talk / Conversation / Wake Phrase
→ Whisper
→ AURA Core
→ Permission Engine
→ action / local assistant model
→ optional Piper response
→ speakers
```

## Safety

Voice execution preserves:

- paused-state rejection
- permission classes
- confirmations
- Never policy
- silence / short-capture rejection
- TTS self-trigger prevention
- manual Push-to-Talk preemption
- explicit Wake Phrase opt-in

## Validation

The release includes regression tests for audio conversion, RMS, wake phrase parsing, Voice preference sanitization and local TTS voice path selection.

The final integration gate is the Windows GitHub Actions build.
