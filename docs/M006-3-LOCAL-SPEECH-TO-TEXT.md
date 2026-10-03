# M006.3 — Local Speech-to-Text

M006.3 adds on-device speech recognition to AURA Voice.

## Model

AURA Voice STT uses:

- Source: `openai/whisper-base`
- Architecture: Whisper
- Role: speech-to-text only
- License: Apache-2.0
- Approximate local model size: 295 MB
- Languages: multilingual

The speech model is separate from AURA-1 / AURA-2.

It cannot be selected as the main assistant model.

## Installation

The Voice settings page exposes an explicit STT model install flow.

AURA does not silently download the model when Push-to-Talk is used.

The Model Manager handles the download, verification marker, partial/resumable transfer and removal using the existing local model infrastructure.

The main Models page filters the speech model out of the assistant model picker.

## Runtime

M006.3 adds a dedicated persistent `SpeechRuntime`.

The runtime:

1. locates AURA's Managed Python runtime
2. loads the installed Whisper model using Transformers
3. uses CUDA when PyTorch reports CUDA availability
4. otherwise uses CPU
5. stays alive after the first transcription
6. receives future audio captures through stdin
7. returns transcription text through NDJSON stdout

No network access is used by the runtime after the model is installed.

All Transformers loads use `local_files_only=True`.

## Audio pipeline

Push-to-Talk captures the microphone in its native sample rate/channel layout.

Before STT:

- interleaved channels are downmixed to mono
- audio is resampled to 16 kHz
- samples remain normalized `f32`
- the converted audio is passed directly to the local Python process

No WAV or temporary voice file is written.

## Push-to-Talk flow

```
Ctrl + Shift + F8 held
        ↓
local microphone capture
        ↓
release
        ↓
Captured
        ↓
16 kHz mono conversion
        ↓
Transcribing
        ↓
Whisper Base
        ↓
Transcribed text
```

M006.3 stops at producing text.

M006.4 will submit that text to AURA Core as a voice command.

## Voice events

`aura:voice-capture` now supports:

- `listening`
- `captured`
- `transcribing`
- `transcribed`
- `error`

The transcribed event contains the recognized text.

## Settings → Voice

The Voice settings screen now includes:

- AURA Voice STT status
- model install / pause / resume / cancel / remove
- download progress
- runtime device
- CUDA / CPU state
- speech runtime state
- latest transcription
- STT errors

## Privacy

M006.3:

- does not upload microphone audio
- does not send audio to AURA-1 / Qwen
- does not write voice captures to disk
- does not persist transcripts automatically
- does not execute transcribed text yet
- does not listen without explicit Push-to-Talk

The audio capture is consumed from process memory by the local STT runtime.

## Validation

Audio conversion regression tests cover:

- stereo → mono downmix
- sample-rate conversion to 16 kHz

The authoritative integration gate remains the Windows build because the full path combines CPAL/WASAPI, Tauri global shortcuts, the managed Python runtime, PyTorch and Transformers.

## Roadmap

- M006.1 ✅ Audio input foundation
- M006.2 ✅ Push-to-talk
- M006.3 ✅ Local speech-to-text
- M006.4 Voice → AURA Core
- M006.5 Local text-to-speech
- M006.6 Voice selection & settings
- M006.7 Conversation mode
- M006.8 Voice interruption / stop speaking
- M006.9 Optional wake word
- M006.10 Voice validation
