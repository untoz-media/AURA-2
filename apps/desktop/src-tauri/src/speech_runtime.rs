use crate::{
    audio_input::CapturedAudio,
    managed_runtime::managed_python_path,
    model_manager::ModelManager,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const SPEECH_MODEL_ID: &str = "voice-whisper-base";
const SPEECH_RUNTIME_SCRIPT: &str = include_str!("speech_runtime.py");

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechRuntimeStatus {
    pub state: String,
    pub model_id: String,
    pub device: Option<String>,
    pub cuda: Option<bool>,
    pub last_text: Option<String>,
    pub last_error: Option<String>,
    pub refreshed_at_ms: u64,
}

impl Default for SpeechRuntimeStatus {
    fn default() -> Self {
        Self {
            state: "stopped".to_string(),
            model_id: SPEECH_MODEL_ID.to_string(),
            device: None,
            cuda: None,
            last_text: None,
            last_error: None,
            refreshed_at_ms: timestamp_ms(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechTranscriptionResult {
    pub text: String,
    pub duration_ms: u64,
    pub source_sample_rate: u32,
    pub source_channels: u16,
    pub input_samples_16khz: usize,
    pub input_rms: f32,
    pub completed_at_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpeechEnvelope {
    #[serde(rename = "type")]
    kind: String,
    id: Option<String>,
    text: Option<String>,
    message: Option<String>,
    device: Option<String>,
    cuda: Option<bool>,
}

struct SpeechProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    device: Option<String>,
    cuda: Option<bool>,
}

pub struct SpeechRuntime {
    process: Mutex<Option<SpeechProcess>>,
    status: Mutex<SpeechRuntimeStatus>,
    request_counter: AtomicU64,
}

impl Default for SpeechRuntime {
    fn default() -> Self {
        Self {
            process: Mutex::new(None),
            status: Mutex::new(SpeechRuntimeStatus::default()),
            request_counter: AtomicU64::new(1),
        }
    }
}

impl SpeechRuntime {
    pub fn status(&self) -> SpeechRuntimeStatus {
        self.status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn stop(&self) {
        let mut process = self
            .process
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(mut running) = process.take() {
            let _ = writeln!(running.stdin, "{}", json!({
                "type": "shutdown",
                "id": "shutdown",
            }));
            let _ = running.stdin.flush();
            let _ = running.child.kill();
            let _ = running.child.wait();
        }

        self.set_status(SpeechRuntimeStatus::default());
    }

    pub fn transcribe(
        &self,
        app: &AppHandle,
        manager: &ModelManager,
        capture: CapturedAudio,
    ) -> Result<SpeechTranscriptionResult, String> {
        let source_sample_rate = capture.sample_rate;
        let source_channels = capture.channels;
        let duration_ms = capture.completed_at_ms.saturating_sub(capture.started_at_ms);
        let input_rms = capture.rms();
        let samples = capture.mono_16khz();

        if samples.is_empty() {
            return Err("The voice capture is empty.".to_string());
        }

        let model_path = manager.installation_path(app, SPEECH_MODEL_ID)?;
        let mut process_guard = self
            .process
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if process_guard.is_none() {
            *process_guard = Some(self.start_process(app, &model_path)?);
        }

        let request_id = format!(
            "speech-{}-{}",
            timestamp_ms(),
            self.request_counter.fetch_add(1, Ordering::Relaxed)
        );

        if let Some(running) = process_guard.as_ref() {
            self.set_status(SpeechRuntimeStatus {
                state: "transcribing".to_string(),
                model_id: SPEECH_MODEL_ID.to_string(),
                device: running.device.clone(),
                cuda: running.cuda,
                last_text: self.status().last_text,
                last_error: None,
                refreshed_at_ms: timestamp_ms(),
            });
        }

        let input_samples_16khz = samples.len();
        let request = json!({
            "type": "transcribe",
            "id": request_id.clone(),
            "samples": samples,
        });

        {
            let running = process_guard
                .as_mut()
                .ok_or_else(|| "Speech runtime failed to start.".to_string())?;
            writeln!(running.stdin, "{request}")
                .and_then(|_| running.stdin.flush())
                .map_err(|error| format!("Could not send audio to local speech runtime: {error}"))?;
        }

        loop {
            let envelope = {
                let running = process_guard
                    .as_mut()
                    .ok_or_else(|| "Speech runtime is no longer available.".to_string())?;
                read_envelope(&mut running.stdout)?
            };

            if envelope.id.as_deref() != Some(request_id.as_str()) {
                continue;
            }

            match envelope.kind.as_str() {
                "transcription" => {
                    let text = envelope.text.unwrap_or_default().trim().to_string();
                    if text.is_empty() {
                        let message = "Speech-to-text returned an empty transcription.".to_string();
                        self.set_error(process_guard.as_ref(), message.clone());
                        return Err(message);
                    }

                    if let Some(running) = process_guard.as_ref() {
                        self.set_status(SpeechRuntimeStatus {
                            state: "ready".to_string(),
                            model_id: SPEECH_MODEL_ID.to_string(),
                            device: running.device.clone(),
                            cuda: running.cuda,
                            last_text: Some(text.clone()),
                            last_error: None,
                            refreshed_at_ms: timestamp_ms(),
                        });
                    }

                    return Ok(SpeechTranscriptionResult {
                        text,
                        duration_ms,
                        source_sample_rate,
                        source_channels,
                        input_samples_16khz,
                        input_rms,
                        completed_at_ms: timestamp_ms(),
                    });
                }
                "error" | "fatal" => {
                    let message = envelope
                        .message
                        .unwrap_or_else(|| "Local speech runtime failed.".to_string());
                    self.set_error(process_guard.as_ref(), message.clone());
                    return Err(message);
                }
                _ => {}
            }
        }
    }

    fn start_process(
        &self,
        app: &AppHandle,
        model_path: &std::path::Path,
    ) -> Result<SpeechProcess, String> {
        let python = managed_python_path(app)?;
        if !python.exists() {
            return Err(
                "AURA Managed Runtime is not installed. Install or repair it from Models first."
                    .to_string(),
            );
        }

        let runtime_dir = app
            .path()
            .app_cache_dir()
            .map_err(|error| error.to_string())?
            .join("speech-runtime");
        std::fs::create_dir_all(&runtime_dir)
            .map_err(|error| format!("Could not create speech runtime directory: {error}"))?;
        let script_path = runtime_dir.join("speech_runtime.py");
        std::fs::write(&script_path, SPEECH_RUNTIME_SCRIPT)
            .map_err(|error| format!("Could not write speech runtime script: {error}"))?;

        self.set_status(SpeechRuntimeStatus {
            state: "loading".to_string(),
            model_id: SPEECH_MODEL_ID.to_string(),
            refreshed_at_ms: timestamp_ms(),
            ..SpeechRuntimeStatus::default()
        });

        let mut command = Command::new(&python);
        command
            .arg("-u")
            .arg(&script_path)
            .arg("--model-path")
            .arg(model_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);

        let mut child = command
            .spawn()
            .map_err(|error| format!("Could not start local speech runtime: {error}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Speech runtime stdin is unavailable.".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Speech runtime stdout is unavailable.".to_string())?;

        let mut process = SpeechProcess {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            device: None,
            cuda: None,
        };

        loop {
            let envelope = read_envelope(&mut process.stdout)?;
            match envelope.kind.as_str() {
                "ready" => {
                    process.device = envelope.device;
                    process.cuda = envelope.cuda;
                    self.set_status(SpeechRuntimeStatus {
                        state: "ready".to_string(),
                        model_id: SPEECH_MODEL_ID.to_string(),
                        device: process.device.clone(),
                        cuda: process.cuda,
                        last_text: None,
                        last_error: None,
                        refreshed_at_ms: timestamp_ms(),
                    });
                    return Ok(process);
                }
                "fatal" | "error" => {
                    return Err(envelope
                        .message
                        .unwrap_or_else(|| "Speech runtime failed to initialize.".to_string()));
                }
                _ => {}
            }
        }
    }

    fn set_error(
        &self,
        process: Option<&SpeechProcess>,
        message: String,
    ) {
        self.set_status(SpeechRuntimeStatus {
            state: "error".to_string(),
            model_id: SPEECH_MODEL_ID.to_string(),
            device: process.and_then(|running| running.device.clone()),
            cuda: process.and_then(|running| running.cuda),
            last_text: self.status().last_text,
            last_error: Some(message),
            refreshed_at_ms: timestamp_ms(),
        });
    }

    fn set_status(&self, status: SpeechRuntimeStatus) {
        *self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = status;
    }
}

fn read_envelope(reader: &mut BufReader<ChildStdout>) -> Result<SpeechEnvelope, String> {
    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .map_err(|error| format!("Could not read local speech runtime: {error}"))?;

    if bytes == 0 {
        return Err("Local speech runtime exited unexpectedly.".to_string());
    }

    serde_json::from_str(line.trim())
        .map_err(|error| format!("Local speech runtime returned invalid data: {error}"))
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
