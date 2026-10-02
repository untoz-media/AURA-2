# M003.6 — Volume & Media Controls

M003.6 adds Windows system-volume and global media control to AURA-2.

## Commands

Volume state:

- `What is the volume?`
- `What's the volume?`
- `Qual é o volume?`

Exact volume:

- `Set volume to 30%`
- `Volume 30%`
- `Define o volume para 30%`
- `Mete o volume a 30%`

Volume step:

- `Volume up`
- `Volume down`
- `Aumenta o volume`
- `Baixa o volume`

Mute:

- `Mute`
- `Unmute`
- `Silencia`
- `Tira o mute`

Media:

- `Play pause`
- `Pause music`
- `Next track`
- `Previous track`
- `Stop media`
- `Pausa a música`
- `Próxima música`
- `Música anterior`

## Exact volume

Exact system volume uses Windows Core Audio:

- `IMMDeviceEnumerator`
- default render endpoint
- `IAudioEndpointVolume`
- `SetMasterVolumeLevelScalar`
- `GetMasterVolumeLevelScalar`
- `SetMute`
- `GetMute`
- `VolumeStepUp / VolumeStepDown`

The scalar volume API uses the Windows normalized range from 0.0 to 1.0. AURA exposes this as 0–100%.

## Media controls

Playback controls use the Windows global media virtual keys through `SendInput`:

- Play/Pause
- Next Track
- Previous Track
- Stop

They behave like the media keys on a physical keyboard, so the currently active media session/application decides how to respond.

## Permissions

- read volume/mute state → `Read`
- set volume → `Act`
- step volume → `Act`
- mute/unmute → `Act`
- play/pause/next/previous/stop → `Act`

These are reversible system/media actions and are allowed by the current default policy.

## Safety

- exact volume is validated to 0–100
- no app-specific audio-session manipulation is included yet
- no microphone controls are included in M003.6
- no shell or external command-line audio utilities are used
