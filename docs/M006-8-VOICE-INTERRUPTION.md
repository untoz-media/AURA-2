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

When the optional wake phrase is enabled, AURA keeps listening during its own speech so the phrase can interrupt playback. That path uses a louder energy floor than idle listening and rejects a transcript that is contained in the text AURA is speaking, including a few seconds after playback ends. There is no acoustic echo canceller. Quiet speaker bleed is ignored; a phrase that merely repeats the spoken reply does not stop playback. Wake Phrase stays off by default. Push-to-talk still interrupts speech immediately.
