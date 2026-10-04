# M006.8 — Voice Interruption / Stop Speaking

M006.8 adds explicit speech interruption and barge-in.

## Stop Speaking

Settings → Voice exposes a Stop speaking action.

Piper playback lives in its own Python worker process. AURA can terminate that worker immediately and restart it cleanly for the next synthesis request.

## Barge-in

Pressing the Push-to-Talk shortcut while AURA is speaking:

1. interrupts the active TTS worker
2. stops spoken output
3. cancels automatic wake monitoring for that instant
4. gives the microphone to the user's Push-to-Talk capture

This allows natural interruption instead of waiting for AURA to finish speaking.

## Wake safety

Wake monitoring is suspended while TTS is speaking so AURA cannot activate from its own synthetic voice.
