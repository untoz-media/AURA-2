use crate::{
    managed_runtime::{
        install_managed_python_package, managed_python_path, python_module_available,
    },
    model_manager::ModelManager,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const DEFAULT_TTS_MODEL_ID: &str = "voice-piper-ptpt";
const TTS_RUNTIME_PACKAGE: &str = "piper-tts>=1.8,<2";
const TTS_RUNTIME_SCRIPT: &str = include_str!("tts_runtime.py");

fn voice_relative_path(model_id: &str) -> Option<&'static str> {
    match model_id {
        "voice-piper-ptpt" => Some("pt/pt_PT/tugão/medium/pt_PT-tugão-medium.onnx"),
        "voice-piper-engb-alan" => Some("en/en_GB/alan/medium/en_GB-alan-medium.onnx"),
        _ => None,
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsRuntimeStatus {
    pub state: String,
    pub model_id: String,
    pub dependency_ready: bool,
    pub voice_installed: bool,
    pub sample_rate: Option<u32>,
    pub last_text: Option<String>,
    pub last_error: Option<String>,
    pub refreshed_at_ms: u64,
}

impl Default for TtsRuntimeStatus {
    fn default() -> Self {
        Self {
            state: "stopped".to_string(),
            model_id: DEFAULT_TTS_MODEL_ID.to_string(),
            dependency_ready: false,
            voice_installed: false,
            sample_rate: None,
            last_text: None,
            last_error: None,
            refreshed_at_ms: timestamp_ms(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TtsEnvelope {
    #[serde(rename = "type")]
    kind: String,
    id: Option<String>,
    text: Option<String>,
    message: Option<String>,
    sample_rate: Option<u32>,
}

struct TtsProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    sample_rate: Option<u32>,
}

pub struct TtsRuntime {
    process: Mutex<Option<TtsProcess>>,
    status: Mutex<TtsRuntimeStatus>,
    request_counter: AtomicU64,
    process_id: AtomicU32,
}

impl Default for TtsRuntime {
    fn default() -> Self {
        Self {
            process: Mutex::new(None),
            status: Mutex::new(TtsRuntimeStatus::default()),
            request_counter: AtomicU64::new(1),
            process_id: AtomicU32::new(0),
        }
    }
}

impl TtsRuntime {
    pub fn is_speaking(&self) -> bool {
        self.status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .state
            == "speaking"
    }

    pub fn status(&self, app: &AppHandle, manager: &ModelManager) -> TtsRuntimeStatus {
        let mut current = self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();

        if !current.dependency_ready {
            current.dependency_ready =
                python_module_available(app, "piper").unwrap_or(false);
        }
        current.voice_installed =
            manager.installation_path(app, DEFAULT_TTS_MODEL_ID).is_ok();
        current.refreshed_at_ms = timestamp_ms();

        current
    }

    pub fn prepare_dependency(
        &self,
        app: &AppHandle,
        manager: &ModelManager,
    ) -> Result<TtsRuntimeStatus, String> {
        self.stop();

        self.set_status(TtsRuntimeStatus {
            state: "installingDependency".to_string(),
            model_id: DEFAULT_TTS_MODEL_ID.to_string(),
            dependency_ready: false,
            voice_installed: manager.installation_path(app, DEFAULT_TTS_MODEL_ID).is_ok(),
            refreshed_at_ms: timestamp_ms(),
            ..TtsRuntimeStatus::default()
        });

        install_managed_python_package(app, TTS_RUNTIME_PACKAGE)?;

        if !python_module_available(app, "piper")? {
            let message =
                "Piper installation finished but the managed Python runtime cannot import it."
                    .to_string();
            self.set_error(app, manager, None, message.clone());
            return Err(message);
        }

        let status = TtsRuntimeStatus {
            state: "ready".to_string(),
            model_id: DEFAULT_TTS_MODEL_ID.to_string(),
            dependency_ready: true,
            voice_installed: manager.installation_path(app, DEFAULT_TTS_MODEL_ID).is_ok(),
            sample_rate: None,
            last_text: None,
            last_error: None,
            refreshed_at_ms: timestamp_ms(),
        };
        self.set_status(status.clone());
        Ok(status)
    }

    pub fn speak(
        &self,
        app: &AppHandle,
        manager: &ModelManager,
        text: &str,
        speed: f32,
        model_id: &str,
    ) -> Result<TtsRuntimeStatus, String> {
        let text = text.trim();
        if text.is_empty() {
            return Err("Text-to-speech received empty text.".to_string());
        }

        if !python_module_available(app, "piper")? {
            return Err(
                "Piper TTS is not installed. Prepare the local TTS runtime from Voice settings."
                    .to_string(),
            );
        }

        let relative_path = voice_relative_path(model_id)
            .ok_or_else(|| format!("Unknown TTS voice model: {model_id}."))?;
        let voice_root = manager.installation_path(app, model_id)?;
        let voice_path = voice_root.join(relative_path);
        if !voice_path.exists() {
            return Err("The selected local TTS voice is incomplete.".to_string());
        }

        let mut process_guard = self
            .process
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if self.process_id.load(Ordering::Relaxed) == 0 {
            process_guard.take();
        }

        if process_guard.is_none() {
            *process_guard = Some(self.start_process(app, &voice_path)?);
        }

        let request_id = format!(
            "tts-{}-{}",
            timestamp_ms(),
            self.request_counter.fetch_add(1, Ordering::Relaxed)
        );

        let sample_rate = process_guard
            .as_ref()
            .and_then(|running| running.sample_rate);

        self.set_status(TtsRuntimeStatus {
            state: "speaking".to_string(),
            model_id: model_id.to_string(),
            dependency_ready: true,
            voice_installed: true,
            sample_rate,
            last_text: Some(text.to_string()),
            last_error: None,
            refreshed_at_ms: timestamp_ms(),
        });

        let request = json!({
            "type": "speak",
            "id": request_id.clone(),
            "text": text,
            "speed": speed.clamp(0.6, 1.5),
        });

        {
            let running = process_guard
                .as_mut()
                .ok_or_else(|| "TTS runtime failed to start.".to_string())?;
            writeln!(running.stdin, "{request}")
                .and_then(|_| running.stdin.flush())
                .map_err(|error| format!("Could not send text to local TTS runtime: {error}"))?;
        }

        loop {
            let envelope = {
                let running = process_guard
                    .as_mut()
                    .ok_or_else(|| "TTS runtime is no longer available.".to_string())?;
                read_envelope(&mut running.stdout)?
            };

            if envelope.id.as_deref() != Some(request_id.as_str()) {
                continue;
            }

            match envelope.kind.as_str() {
                "spoken" => {
                    let status = TtsRuntimeStatus {
                        state: "ready".to_string(),
                        model_id: DEFAULT_TTS_MODEL_ID.to_string(),
                        dependency_ready: true,
                        voice_installed: true,
                        sample_rate: process_guard
                            .as_ref()
                            .and_then(|running| running.sample_rate),
                        last_text: envelope.text.or_else(|| Some(text.to_string())),
                        last_error: None,
                        refreshed_at_ms: timestamp_ms(),
                    };
                    self.set_status(status.clone());
                    return Ok(status);
                }
                "error" | "fatal" => {
                    let message = envelope
                        .message
                        .unwrap_or_else(|| "Local text-to-speech failed.".to_string());
                    let sample_rate = process_guard
                        .as_ref()
                        .and_then(|running| running.sample_rate);
                    self.set_error(app, manager, sample_rate, message.clone());
                    return Err(message);
                }
                _ => {}
            }
        }
    }

    pub fn stop(&self) {
        let mut process = self
            .process
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(mut running) = process.take() {
            let _ = writeln!(
                running.stdin,
                "{}",
                json!({"type": "shutdown", "id": "shutdown"})
            );
            let _ = running.stdin.flush();
            let _ = running.child.kill();
            let _ = running.child.wait();
        }

        self.process_id.store(0, Ordering::Relaxed);
        self.set_status(TtsRuntimeStatus::default());
    }

    pub fn interrupt(&self) -> Result<TtsRuntimeStatus, String> {
        let pid = self.process_id.swap(0, Ordering::Relaxed);
        if pid != 0 {
            #[cfg(windows)]
            {
                let status = Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/T", "/F"])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .creation_flags(CREATE_NO_WINDOW)
                    .status()
                    .map_err(|error| format!("Could not stop local TTS playback: {error}"))?;
                if !status.success() {
                    return Err("Windows could not stop the TTS worker.".to_string());
                }
            }
        }

        let status = TtsRuntimeStatus {
            state: "stopped".to_string(),
            model_id: DEFAULT_TTS_MODEL_ID.to_string(),
            dependency_ready: true,
            voice_installed: true,
            sample_rate: None,
            last_text: self
                .status
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .last_text
                .clone(),
            last_error: None,
            refreshed_at_ms: timestamp_ms(),
        };
        self.set_status(status.clone());
        Ok(status)
    }

    fn start_process(
        &self,
        app: &AppHandle,
        voice_path: &Path,
    ) -> Result<TtsProcess, String> {
        let python = managed_python_path(app)?;
        if !python.exists() {
            return Err(
                "AURA Managed Runtime is not installed. Install it from Models first."
                    .to_string(),
            );
        }

        let runtime_dir = app
            .path()
            .app_cache_dir()
            .map_err(|error| error.to_string())?
            .join("tts-runtime");
        std::fs::create_dir_all(&runtime_dir)
            .map_err(|error| format!("Could not create TTS runtime directory: {error}"))?;
        let script_path = runtime_dir.join("tts_runtime.py");
        std::fs::write(&script_path, TTS_RUNTIME_SCRIPT)
            .map_err(|error| format!("Could not write TTS runtime script: {error}"))?;

        self.set_status(TtsRuntimeStatus {
            state: "loading".to_string(),
            model_id: DEFAULT_TTS_MODEL_ID.to_string(),
            dependency_ready: true,
            voice_installed: true,
            refreshed_at_ms: timestamp_ms(),
            ..TtsRuntimeStatus::default()
        });

        let mut command = Command::new(&python);
        command
            .arg("-u")
            .arg(&script_path)
            .arg("--voice-path")
            .arg(voice_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);

        let mut child = command
            .spawn()
            .map_err(|error| format!("Could not start local TTS runtime: {error}"))?;
        self.process_id.store(child.id(), Ordering::Relaxed);
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "TTS runtime stdin is unavailable.".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "TTS runtime stdout is unavailable.".to_string())?;

        let mut process = TtsProcess {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            sample_rate: None,
        };

        loop {
            let envelope = read_envelope(&mut process.stdout)?;
            match envelope.kind.as_str() {
                "ready" => {
                    process.sample_rate = envelope.sample_rate;
                    return Ok(process);
                }
                "fatal" | "error" => {
                    return Err(envelope
                        .message
                        .unwrap_or_else(|| "TTS runtime failed to initialize.".to_string()));
                }
                _ => {}
            }
        }
    }

    fn set_error(
        &self,
        app: &AppHandle,
        manager: &ModelManager,
        sample_rate: Option<u32>,
        message: String,
    ) {
        let status = TtsRuntimeStatus {
            state: "error".to_string(),
            model_id: DEFAULT_TTS_MODEL_ID.to_string(),
            dependency_ready: python_module_available(app, "piper").unwrap_or(false),
            voice_installed: manager.installation_path(app, DEFAULT_TTS_MODEL_ID).is_ok(),
            sample_rate,
            last_text: self
                .status
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .last_text
                .clone(),
            last_error: Some(message),
            refreshed_at_ms: timestamp_ms(),
        };
        self.set_status(status);
    }

    fn set_status(&self, status: TtsRuntimeStatus) {
        *self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = status;
    }
}

fn read_envelope(reader: &mut BufReader<ChildStdout>) -> Result<TtsEnvelope, String> {
    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .map_err(|error| format!("Could not read local TTS runtime: {error}"))?;

    if bytes == 0 {
        return Err("Local TTS runtime exited unexpectedly.".to_string());
    }

    serde_json::from_str(line.trim())
        .map_err(|error| format!("Local TTS runtime returned invalid data: {error}"))
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_supported_local_voice_paths() {
        assert!(voice_relative_path("voice-piper-ptpt")
            .is_some_and(|path| path.ends_with("pt_PT-tugão-medium.onnx")));
        assert!(voice_relative_path("voice-piper-engb-alan")
            .is_some_and(|path| path.ends_with("en_GB-alan-medium.onnx")));
        assert_eq!(voice_relative_path("unknown"), None);
    }
}
