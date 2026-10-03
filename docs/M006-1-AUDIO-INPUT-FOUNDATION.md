# M006.1 — Audio Input Foundation

M006.1 starts AURA Voice with a local microphone input layer.

## Audio backend

AURA uses CPAL 0.18.2.

On Windows, CPAL uses WASAPI for the default audio backend.

The audio foundation can:

- enumerate microphone/input devices
- identify the Windows default input
- select a microphone
- open a local input stream
- read stream sample rate, channel count and sample format
- calculate a live input level
- stop and release the stream cleanly

## Privacy boundary

M006.1 does not:

- record microphone audio to disk
- send audio to the AURA model
- transcribe speech
- upload audio
- keep an audio history
- listen in the background

The microphone is opened only while the explicit microphone test is running.

The callback calculates peak amplitude and discards the sample buffer after processing.

## Desktop UI

Settings → Voice now exposes:

- microphone selector
- default-device indicator
- Refresh
- Test microphone
- Stop test
- live input meter
- sample rate
- channel count
- sample format
- microphone stream errors

The meter refreshes while testing and stops polling when the stream closes.

## Architecture

```
Windows microphone
      ↓
CPAL / WASAPI
      ↓
AudioInputManager
      ↓
local amplitude meter
```

No speech model is connected in this milestone.

## Next

M006.2 — Push-to-talk will reuse AudioInputManager as the capture source.

M006.3 will then add local speech-to-text with an open-source model.

## Roadmap

- M006.1 ✅ Audio input foundation
- M006.2 Push-to-talk
- M006.3 Local speech-to-text
- M006.4 Voice → AURA Core
- M006.5 Local text-to-speech
- M006.6 Voice selection & settings
- M006.7 Conversation mode
- M006.8 Voice interruption / stop speaking
- M006.9 Optional wake word
- M006.10 Voice validation
