# M006.7 — Conversation Mode

Conversation Mode enables one automatic follow-up listening window after AURA finishes a voice-originated response.

## Flow

```
Voice command
→ AURA Core
→ optional spoken response
→ Conversation Mode listening
→ speech detected
→ silence detected
→ Whisper
→ AURA Core
```

## Local VAD

AURA uses the existing microphone amplitude meter as a lightweight local voice-activity detector.

Thresholds:

- speech onset: level >= 0.015
- continuing speech: updates while speech is present
- end of utterance: level < 0.008 for ~900 ms
- timeout: configurable, 3–20 seconds

No cloud VAD service is used.

If no speech is detected before the timeout, Conversation Mode closes cleanly.

Manual Push-to-Talk can preempt an automatic follow-up capture.
