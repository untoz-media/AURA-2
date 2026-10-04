# M006.5 — Local Text-to-Speech

M006.5 gives AURA an entirely local spoken response path on Windows.

## Voice engine

AURA Voice TTS uses:

- Piper
- pt_PT-tugão-medium
- Portuguese (Portugal)
- ONNX voice model
- approximately 63 MB voice files

The voice files are sourced from `rhasspy/piper-voices`.

The voice-model repository is MIT licensed.

The current Piper runtime is GPL-3.0 and is installed explicitly into AURA's private Managed Python environment only when the user requests TTS setup.

## Architecture

```
Voice command
  ↓
Whisper STT
  ↓
AURA Core
  ↓
Core result
  ↓
TTS Router
  ↓
Piper
  ↓
pt_PT-tugão-medium
  ↓
WAV in memory
  ↓
Windows speakers
```

## Voice-only automatic replies

Automatic TTS is scoped to commands that originated from `source="voice"`.

Typed desktop and overlay commands remain silent by default.

AURA Core tracks voice command IDs until a terminal event:

- command.completed
- command.failed
- command.cancelled

Completed and failed voice commands can trigger local speech.

Cancelled commands do not speak.

## Permission and confirmation compatibility

Voice command IDs remain associated with the command through confirmation flows.

If a voice command requires confirmation:

1. the original voice command enters Waiting
2. permission confirmation remains required
3. approval reuses the same command ID
4. the final result can be spoken only after the safe Core path finishes

TTS does not bypass the Permission Engine.

## Runtime

M006.5 adds a persistent `TtsRuntime`.

The runtime:

- uses AURA Managed Python
- installs `piper-tts>=1.8,<2` only through explicit user action
- loads the local PT-PT ONNX voice
- remains resident after first use
- communicates with Rust over NDJSON stdin/stdout
- generates WAV data fully in memory
- uses Windows in-memory audio playback
- writes no temporary WAV file

## Settings → Voice

The TTS panel exposes:

- Piper runtime state
- PT-PT voice-model state
- runtime installation
- voice-model download / pause / resume / cancel / remove
- test voice
- sample rate
- latest spoken response
- error state
- license disclosure

Default test phrase:

`Olá. Eu sou a AURA, o teu assistente pessoal.`

## Model separation

Feature models remain separate from main assistant models.

Roles now include:

- `assistant`
- `speechToText`
- `textToSpeech`

Only `assistant` models appear in the main assistant-model picker.

The TTS voice cannot become AURA's reasoning model.

## Privacy

M006.5 remains local-first:

- response text stays local
- synthesis runs locally
- audio is generated in memory
- audio is not uploaded
- generated WAV is not persisted automatically

## Interruption

Playback still runs inside the dedicated TTS worker, keeping the desktop UI responsive.

M006.8 adds explicit Stop speaking and Push-to-Talk barge-in by terminating and cleanly restarting the isolated TTS worker when needed.

## Roadmap

- M006.1 ✅ Audio input foundation
- M006.2 ✅ Push-to-talk
- M006.3 ✅ Local speech-to-text
- M006.4 ✅ Voice → AURA Core
- M006.5 ✅ Local text-to-speech
- M006.6 ✅ Voice selection & settings
- M006.7 ✅ Conversation mode
- M006.8 ✅ Voice interruption / stop speaking
- M006.9 ✅ Optional wake word
- M006.10 ✅ Voice validation
