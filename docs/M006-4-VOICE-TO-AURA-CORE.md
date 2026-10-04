# M006.4 — Voice → AURA Core

M006.4 connects local speech recognition to the same deterministic AURA Core command path used by typed commands.

## Flow

```
Push-to-Talk
  ↓
Whisper STT
  ↓
transcription
  ↓
voice normalization
  ↓
AURA Core
  ↓
Action Router
  ↓
Permission Engine
  ↓
Computer / OBS / Memory / Routine / Model fallback
```

Voice commands use `source="voice"`.

They do not bypass any existing permission or confirmation rule.

## Wake-prefix normalization

For natural speech, AURA removes an optional leading invocation phrase before routing:

- AURA
- Hey AURA
- OK AURA
- Okay AURA
- Olá AURA

Examples:

- `AURA, abre o Brave` → `abre o Brave`
- `Hey AURA: open Brave` → `open Brave`

The original transcription remains visible in Voice UI. Only the command passed to Core is normalized.

A wake name with no command is rejected.

## Safety guard

Before a transcript can be executed, the capture must pass a minimum signal check.

AURA rejects voice commands when:

- capture duration is below 250 ms
- input RMS is below 0.003

This is specifically intended to reduce accidental command execution from silence/noise or Whisper hallucinations.

## Permissions

Voice reuses the exact same `process_user_command` path as desktop and overlay commands.

Therefore:

- Read permissions remain Read
- Act permissions remain Act
- destructive actions remain permission-controlled
- Sensitive actions still require confirmation when policy is Ask
- Never remains Never
- paused AURA rejects voice commands

Voice receives no extra authority.

## UI state

Voice now transitions through:

- Listening
- Transcribing
- Transcribed
- Submitted to AURA Core

After submission, AURA Core owns runtime state such as Thinking, Working, Waiting and Idle.

Voice commands also appear in chat as user messages so their execution/result remains visible.

## Tests

M006.4 adds regression coverage for:

- AURA wake-prefix stripping
- commands without wake prefixes
- wake-word-only rejection
- audio RMS calculation

## Privacy

The voice path remains local:

- microphone capture stays in memory
- STT runs locally
- normalized text is sent only to local AURA Core
- audio is not uploaded
- audio is not persisted

## Roadmap

- M006.1 ✅ Audio input foundation
- M006.2 ✅ Push-to-talk
- M006.3 ✅ Local speech-to-text
- M006.4 ✅ Voice → AURA Core
- M006.5 Local text-to-speech
- M006.6 Voice selection & settings
- M006.7 Conversation mode
- M006.8 Voice interruption / stop speaking
- M006.9 Optional wake word
- M006.10 Voice validation
