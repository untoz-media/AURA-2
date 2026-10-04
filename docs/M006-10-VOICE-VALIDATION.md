# M006.10 — Voice Validation

M006.10 closes the Voice milestone and releases AURA-2 0.6.0-alpha.1.

## Covered Voice stack

- Audio input foundation
- Push-to-Talk
- local Whisper STT
- Voice → AURA Core
- local Piper TTS
- PT-PT / English UK voice selection
- persistent Voice settings
- Conversation Mode
- Stop speaking
- Push-to-Talk barge-in
- optional local Wake Phrase
- safety guards and regression tests

## Safety boundaries

Voice commands use the existing Permission Engine.

Voice does not gain extra authority.

Existing safeguards include:

- paused runtime rejection
- permission classes
- Ask confirmation
- Never policy
- quiet / too-short capture rejection
- wake monitoring disabled by default
- wake monitoring suspended during TTS
- manual Push-to-Talk preemption

## Regression coverage

Tests cover:

- wake-prefix normalization
- configured wake phrase extraction
- wake-name-only handling
- Voice preference sanitization
- invalid voice selection fallback
- microphone stereo → mono conversion
- resampling to 16 kHz
- RMS calculation
- PT-PT and English UK Piper voice path resolution

## Release

Version:

`0.6.0-alpha.1`

The authoritative integration validation remains the Windows GitHub Actions build because the full stack crosses Rust, Tauri, CPAL/WASAPI, Managed Python, PyTorch, Transformers, Whisper, Piper and Windows audio playback.
