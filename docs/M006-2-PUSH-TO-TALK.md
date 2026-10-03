# M006.2 — Push-to-Talk

M006.2 adds a global hold-to-talk gesture on top of the M006.1 microphone foundation.

## Shortcut

Default Alpha shortcut:

`Ctrl + Shift + F8`

Behavior:

- press and hold → AURA starts local microphone capture
- while held → AURA status becomes Listening
- release → capture stops immediately
- completed audio remains in memory for the future M006.3 speech-to-text stage

The shortcut is intentionally fixed for M006.2. Shortcut customization is planned for M006.6.

## Capture

Push-to-talk reuses `AudioInputManager`.

Captured microphone samples are normalized to `f32` in memory regardless of the original CPAL input format:

- F32
- I16
- U16

A capture stores:

- normalized samples
- sample rate
- channel count
- start timestamp
- completion timestamp

Maximum capture duration is 30 seconds.

This prevents an accidental stuck key from causing unbounded memory growth.

## Events

AURA emits `aura:voice-capture` events:

- `listening`
- `captured`
- `error`

The desktop uses these events to update:

- AURA runtime status
- activity message
- microphone state
- Settings → Voice status

## Settings

Settings → Voice now shows Push-to-Talk as active functionality.

It displays:

- the global shortcut
- Listening state while held
- last capture duration
- last sample count
- readiness for M006.3

## Privacy

M006.2 still does not:

- save captures to disk
- transcribe speech
- send microphone audio to a cloud service
- send microphone audio to the Qwen conversation model
- listen without an explicit held shortcut

The completed audio buffer exists only in process memory and is reserved for the next local STT stage.

## Next

M006.3 — Local speech-to-text will consume the completed in-memory capture and convert it into text with an on-device open-source model.

## Roadmap

- M006.1 ✅ Audio input foundation
- M006.2 ✅ Push-to-talk
- M006.3 Local speech-to-text
- M006.4 Voice → AURA Core
- M006.5 Local text-to-speech
- M006.6 Voice selection & settings
- M006.7 Conversation mode
- M006.8 Voice interruption / stop speaking
- M006.9 Optional wake word
- M006.10 Voice validation
