use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, Stream, StreamConfig,
};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioInputDevice {
    pub name: String,
    pub is_default: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioInputSnapshot {
    pub devices: Vec<AudioInputDevice>,
    pub selected_device: Option<String>,
    pub testing: bool,
    pub level: f32,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub sample_format: Option<String>,
    pub last_error: Option<String>,
}

pub struct AudioInputManager {
    selected_device: Mutex<Option<String>>,
    stream: Mutex<Option<Stream>>,
    level_milli: Arc<AtomicU32>,
    stream_metadata: Mutex<Option<(u32, u16, String)>>,
    last_error: Arc<Mutex<Option<String>>>,
}

impl Default for AudioInputManager {
    fn default() -> Self {
        Self {
            selected_device: Mutex::new(None),
            stream: Mutex::new(None),
            level_milli: Arc::new(AtomicU32::new(0)),
            stream_metadata: Mutex::new(None),
            last_error: Arc::new(Mutex::new(None)),
        }
    }
}

fn input_devices() -> Result<(Vec<AudioInputDevice>, Option<String>), String> {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|device| device.name().ok());

    let mut devices = host
        .input_devices()
        .map_err(|error| format!("Could not enumerate audio input devices: {error}"))?
        .filter_map(|device| device.name().ok())
        .map(|name| AudioInputDevice {
            is_default: default_name.as_deref() == Some(name.as_str()),
            name,
        })
        .collect::<Vec<_>>();

    devices.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    devices.dedup_by(|left, right| left.name.eq_ignore_ascii_case(&right.name));

    Ok((devices, default_name))
}

fn find_input_device(name: Option<&str>) -> Result<cpal::Device, String> {
    let host = cpal::default_host();

    if let Some(name) = name {
        return host
            .input_devices()
            .map_err(|error| format!("Could not enumerate audio input devices: {error}"))?
            .find(|device| {
                device
                    .name()
                    .ok()
                    .is_some_and(|candidate| candidate.eq_ignore_ascii_case(name))
            })
            .ok_or_else(|| format!("Audio input device “{name}” is no longer available."));
    }

    host.default_input_device()
        .ok_or_else(|| "Windows did not expose a default audio input device.".to_string())
}

fn update_level_f32(level: &AtomicU32, data: &[f32]) {
    let peak = data
        .iter()
        .copied()
        .map(f32::abs)
        .fold(0.0_f32, f32::max)
        .clamp(0.0, 1.0);
    level.store((peak * 1000.0).round() as u32, Ordering::Relaxed);
}

fn update_level_i16(level: &AtomicU32, data: &[i16]) {
    let peak = data
        .iter()
        .copied()
        .map(|sample| (sample as f32 / i16::MAX as f32).abs())
        .fold(0.0_f32, f32::max)
        .clamp(0.0, 1.0);
    level.store((peak * 1000.0).round() as u32, Ordering::Relaxed);
}

fn update_level_u16(level: &AtomicU32, data: &[u16]) {
    let midpoint = u16::MAX as f32 / 2.0;
    let peak = data
        .iter()
        .copied()
        .map(|sample| ((sample as f32 - midpoint) / midpoint).abs())
        .fold(0.0_f32, f32::max)
        .clamp(0.0, 1.0);
    level.store((peak * 1000.0).round() as u32, Ordering::Relaxed);
}

impl AudioInputManager {
    pub fn snapshot(&self) -> Result<AudioInputSnapshot, String> {
        let (devices, default_name) = input_devices()?;
        let mut selected = self
            .selected_device
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();

        if selected
            .as_ref()
            .is_some_and(|name| !devices.iter().any(|device| device.name.eq_ignore_ascii_case(name)))
        {
            selected = None;
        }

        let selected_device = selected.or(default_name);
        let testing = self
            .stream
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some();
        let level = self.level_milli.load(Ordering::Relaxed) as f32 / 1000.0;
        let metadata = self
            .stream_metadata
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let last_error = self
            .last_error
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();

        Ok(AudioInputSnapshot {
            devices,
            selected_device,
            testing,
            level,
            sample_rate: metadata.as_ref().map(|value| value.0),
            channels: metadata.as_ref().map(|value| value.1),
            sample_format: metadata.map(|value| value.2),
            last_error,
        })
    }

    pub fn select_device(&self, device_name: Option<String>) -> Result<AudioInputSnapshot, String> {
        self.stop_test();

        if let Some(name) = device_name.as_deref() {
            let _ = find_input_device(Some(name))?;
        }

        *self
            .selected_device
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = device_name;
        *self
            .last_error
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;

        self.snapshot()
    }

    pub fn start_test(&self) -> Result<AudioInputSnapshot, String> {
        self.stop_test();

        let selected = self
            .selected_device
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let device = find_input_device(selected.as_deref())?;
        let supported = device
            .default_input_config()
            .map_err(|error| format!("Could not read microphone configuration: {error}"))?;

        let sample_rate = supported.sample_rate().0;
        let channels = supported.channels();
        let sample_format = supported.sample_format();
        let config: StreamConfig = supported.into();

        let level = self.level_milli.clone();
        let last_error = self.last_error.clone();
        let error_callback = move |error: cpal::StreamError| {
            *last_error
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                Some(format!("Microphone stream error: {error}"));
        };

        let stream = match sample_format {
            SampleFormat::F32 => {
                let level = level.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[f32], _| update_level_f32(&level, data),
                    error_callback,
                    None,
                )
            }
            SampleFormat::I16 => {
                let level = level.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[i16], _| update_level_i16(&level, data),
                    error_callback,
                    None,
                )
            }
            SampleFormat::U16 => {
                let level = level.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[u16], _| update_level_u16(&level, data),
                    error_callback,
                    None,
                )
            }
            other => {
                return Err(format!(
                    "The selected microphone uses unsupported sample format {other:?}."
                ))
            }
        }
        .map_err(|error| format!("Could not open microphone input stream: {error}"))?;

        stream
            .play()
            .map_err(|error| format!("Could not start microphone input stream: {error}"))?;

        self.level_milli.store(0, Ordering::Relaxed);
        *self
            .stream_metadata
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
            Some((sample_rate, channels, format!("{sample_format:?}")));
        *self
            .last_error
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
        *self
            .stream
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(stream);

        self.snapshot()
    }

    pub fn stop_test(&self) -> AudioInputSnapshot {
        self.stream
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        self.level_milli.store(0, Ordering::Relaxed);

        self.snapshot().unwrap_or(AudioInputSnapshot {
            devices: Vec::new(),
            selected_device: None,
            testing: false,
            level: 0.0,
            sample_rate: None,
            channels: None,
            sample_format: None,
            last_error: Some("Could not refresh microphone state.".to_string()),
        })
    }
}
