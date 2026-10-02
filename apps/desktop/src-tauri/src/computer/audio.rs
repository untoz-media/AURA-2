use std::{mem::size_of, ptr};

use windows::Win32::{
        Media::Audio::{
            eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
            Endpoints::IAudioEndpointVolume,
        },
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL,
            COINIT_MULTITHREADED,
        },
    };
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK,
    VK_MEDIA_STOP,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaAction {
    GetVolume,
    SetVolume(u8),
    VolumeUp,
    VolumeDown,
    Mute,
    Unmute,
    PlayPause,
    NextTrack,
    PreviousTrack,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioState {
    pub volume_percent: u8,
    pub muted: bool,
}

#[derive(Debug)]
pub enum AudioError {
    InvalidVolume,
    ComInitialization(String),
    CoreAudio(String),
    MediaKeyInjection { expected: u32, sent: u32 },
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidVolume => write!(formatter, "Volume must be between 0 and 100 percent."),
            Self::ComInitialization(message) => {
                write!(formatter, "Windows audio COM initialization failed: {message}")
            }
            Self::CoreAudio(message) => write!(formatter, "Windows Core Audio failed: {message}"),
            Self::MediaKeyInjection { expected, sent } => write!(
                formatter,
                "Windows accepted {sent} of {expected} media-key events."
            ),
        }
    }
}

struct ComGuard;

impl ComGuard {
    fn initialize() -> Result<Self, AudioError> {
        let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        result
            .ok()
            .map_err(|error| AudioError::ComInitialization(error.to_string()))?;
        Ok(Self)
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

fn endpoint_volume() -> Result<(ComGuard, IAudioEndpointVolume), AudioError> {
    let guard = ComGuard::initialize()?;

    let enumerator: IMMDeviceEnumerator = unsafe {
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .map_err(|error| AudioError::CoreAudio(error.to_string()))?
    };

    let device = unsafe {
        enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .map_err(|error| AudioError::CoreAudio(error.to_string()))?
    };

    let endpoint: IAudioEndpointVolume = unsafe {
        device
            .Activate(CLSCTX_ALL, None)
            .map_err(|error| AudioError::CoreAudio(error.to_string()))?
    };

    Ok((guard, endpoint))
}

pub fn get_audio_state() -> Result<AudioState, AudioError> {
    let (_guard, endpoint) = endpoint_volume()?;

    let scalar = unsafe {
        endpoint
            .GetMasterVolumeLevelScalar()
            .map_err(|error| AudioError::CoreAudio(error.to_string()))?
    };

    let muted = unsafe {
        endpoint
            .GetMute()
            .map_err(|error| AudioError::CoreAudio(error.to_string()))?
            .as_bool()
    };

    let volume_percent = (scalar.clamp(0.0, 1.0) * 100.0).round() as u8;

    Ok(AudioState {
        volume_percent,
        muted,
    })
}

pub fn set_volume(percent: u8) -> Result<AudioState, AudioError> {
    if percent > 100 {
        return Err(AudioError::InvalidVolume);
    }

    let (_guard, endpoint) = endpoint_volume()?;
    let scalar = percent as f32 / 100.0;

    unsafe {
        endpoint
            .SetMasterVolumeLevelScalar(scalar, ptr::null())
            .map_err(|error| AudioError::CoreAudio(error.to_string()))?;
    }

    get_audio_state()
}

pub fn set_mute(muted: bool) -> Result<AudioState, AudioError> {
    let (_guard, endpoint) = endpoint_volume()?;

    unsafe {
        endpoint
            .SetMute(muted, ptr::null())
            .map_err(|error| AudioError::CoreAudio(error.to_string()))?;
    }

    get_audio_state()
}

pub fn volume_step(up: bool) -> Result<AudioState, AudioError> {
    let (_guard, endpoint) = endpoint_volume()?;

    unsafe {
        if up {
            endpoint
                .VolumeStepUp(ptr::null())
                .map_err(|error| AudioError::CoreAudio(error.to_string()))?;
        } else {
            endpoint
                .VolumeStepDown(ptr::null())
                .map_err(|error| AudioError::CoreAudio(error.to_string()))?;
        }
    }

    get_audio_state()
}

fn key_input(vk: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send_media_key(vk: u16) -> Result<(), AudioError> {
    let inputs = [
        key_input(vk, 0),
        key_input(vk, KEYEVENTF_KEYUP),
    ];

    let expected = inputs.len() as u32;
    let sent = unsafe {
        SendInput(
            expected,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    };

    if sent == expected {
        Ok(())
    } else {
        Err(AudioError::MediaKeyInjection { expected, sent })
    }
}

pub fn execute_media_action(action: MediaAction) -> Result<Option<AudioState>, AudioError> {
    match action {
        MediaAction::GetVolume => get_audio_state().map(Some),
        MediaAction::SetVolume(percent) => set_volume(percent).map(Some),
        MediaAction::VolumeUp => volume_step(true).map(Some),
        MediaAction::VolumeDown => volume_step(false).map(Some),
        MediaAction::Mute => set_mute(true).map(Some),
        MediaAction::Unmute => set_mute(false).map(Some),
        MediaAction::PlayPause => {
            send_media_key(VK_MEDIA_PLAY_PAUSE)?;
            Ok(None)
        }
        MediaAction::NextTrack => {
            send_media_key(VK_MEDIA_NEXT_TRACK)?;
            Ok(None)
        }
        MediaAction::PreviousTrack => {
            send_media_key(VK_MEDIA_PREV_TRACK)?;
            Ok(None)
        }
        MediaAction::Stop => {
            send_media_key(VK_MEDIA_STOP)?;
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_percent_bounds_are_valid() {
        assert!(matches!(MediaAction::SetVolume(0), MediaAction::SetVolume(0)));
        assert!(matches!(MediaAction::SetVolume(100), MediaAction::SetVolume(100)));
    }

    #[test]
    fn media_actions_are_copyable() {
        let action = MediaAction::PlayPause;
        let copied = action;
        assert_eq!(action, copied);
    }
}
