use crate::{
    managed_runtime::managed_python_path,
    model_manager::ModelManager,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
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

const VISION_MODEL_ID: &str = "vision-smolvlm2-500m";
const VISION_RUNTIME_SCRIPT: &str = include_str!("vision_runtime.py");

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionRuntimeStatus {
    pub state: String,
    pub model_id: String,
    pub device: Option<String>,
    pub cuda: Option<bool>,
    pub last_prompt: Option<String>,
    pub last_analysis: Option<String>,
    pub last_error: Option<String>,
    pub refreshed_at_ms: u64,
}

impl Default for VisionRuntimeStatus {
    fn default() -> Self {
        Self {
            state: "stopped".to_string(),
            model_id: VISION_MODEL_ID.to_string(),
            device: None,
            cuda: None,
            last_prompt: None,
            last_analysis: None,
            last_error: None,
            refreshed_at_ms: timestamp_ms(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionAnalysisResult {
    pub text: String,
    pub prompt: String,
    pub completed_at_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VisionEnvelope {
    #[serde(rename = "type")]
    kind: String,
    id: Option<String>,
    text: Option<String>,
    message: Option<String>,
    device: Option<String>,
    cuda: Option<bool>,
}

struct VisionProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    device: Option<String>,
    cuda: Option<bool>,
}

pub struct VisionRuntime {
    process: Mutex<Option<VisionProcess>>,
    status: Mutex<VisionRuntimeStatus>,
    request_counter: AtomicU64,
}

impl Default for VisionRuntime {
    fn default() -> Self {
        Self {
            process: Mutex::new(None),
            status: Mutex::new(VisionRuntimeStatus::default()),
            request_counter: AtomicU64::new(1),
        }
    }
}

impl VisionRuntime {
    pub fn status(&self) -> VisionRuntimeStatus {
        self.status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn analyze(
        &self,
        app: &AppHandle,
        manager: &ModelManager,
        image_path: &Path,
        prompt: &str,
    ) -> Result<VisionAnalysisResult, String> {
        let prompt = prompt.trim();
        if prompt.is_empty() {
            return Err("Vision prompt cannot be empty.".to_string());
        }
        if !image_path.exists() {
            return Err("Vision capture file no longer exists.".to_string());
        }

        let model_path = manager.installation_path(app, VISION_MODEL_ID)?;
        let mut process_guard = self
            .process
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if process_guard.is_none() {
            *process_guard = Some(self.start_process(app, &model_path)?);
        }

        let request_id = format!(
            "vision-{}-{}",
            timestamp_ms(),
            self.request_counter.fetch_add(1, Ordering::Relaxed)
        );

        if let Some(running) = process_guard.as_ref() {
            self.set_status(VisionRuntimeStatus {
                state: "analyzing".to_string(),
                model_id: VISION_MODEL_ID.to_string(),
                device: running.device.clone(),
                cuda: running.cuda,
                last_prompt: Some(prompt.to_string()),
                last_analysis: self.status().last_analysis,
                last_error: None,
                refreshed_at_ms: timestamp_ms(),
            });
        }

        let request = json!({
            "type": "analyze",
            "id": request_id.clone(),
            "imagePath": image_path.to_string_lossy(),
            "prompt": prompt,
        });

        {
            let running = process_guard
                .as_mut()
                .ok_or_else(|| "Vision runtime failed to start.".to_string())?;
            writeln!(running.stdin, "{request}")
                .and_then(|_| running.stdin.flush())
                .map_err(|error| format!("Could not send screenshot to local Vision runtime: {error}"))?;
        }

        loop {
            let envelope = {
                let running = process_guard
                    .as_mut()
                    .ok_or_else(|| "Vision runtime is no longer available.".to_string())?;
                read_envelope(&mut running.stdout)?
            };

            if envelope.id.as_deref() != Some(request_id.as_str()) {
                continue;
            }

            match envelope.kind.as_str() {
                "analysis" => {
                    let text = envelope.text.unwrap_or_default().trim().to_string();
                    if text.is_empty() {
                        let message = "Vision model returned an empty analysis.".to_string();
                        self.set_error(process_guard.as_ref(), message.clone());
                        return Err(message);
                    }

                    if let Some(running) = process_guard.as_ref() {
                        self.set_status(VisionRuntimeStatus {
                            state: "ready".to_string(),
                            model_id: VISION_MODEL_ID.to_string(),
                            device: running.device.clone(),
                            cuda: running.cuda,
                            last_prompt: Some(prompt.to_string()),
                            last_analysis: Some(text.clone()),
                            last_error: None,
                            refreshed_at_ms: timestamp_ms(),
                        });
                    }

                    return Ok(VisionAnalysisResult {
                        text,
                        prompt: prompt.to_string(),
                        completed_at_ms: timestamp_ms(),
                    });
                }
                "error" | "fatal" => {
                    let message = envelope
                        .message
                        .unwrap_or_else(|| "Local Vision runtime failed.".to_string());
                    self.set_error(process_guard.as_ref(), message.clone());
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

        self.set_status(VisionRuntimeStatus::default());
    }

    fn start_process(
        &self,
        app: &AppHandle,
        model_path: &Path,
    ) -> Result<VisionProcess, String> {
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
            .join("vision-runtime");
        std::fs::create_dir_all(&runtime_dir)
            .map_err(|error| format!("Could not create Vision runtime directory: {error}"))?;
        let script_path = runtime_dir.join("vision_runtime.py");
        std::fs::write(&script_path, VISION_RUNTIME_SCRIPT)
            .map_err(|error| format!("Could not write Vision runtime script: {error}"))?;

        self.set_status(VisionRuntimeStatus {
            state: "loading".to_string(),
            model_id: VISION_MODEL_ID.to_string(),
            refreshed_at_ms: timestamp_ms(),
            ..VisionRuntimeStatus::default()
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
            .map_err(|error| format!("Could not start local Vision runtime: {error}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Vision runtime stdin is unavailable.".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Vision runtime stdout is unavailable.".to_string())?;

        let mut process = VisionProcess {
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
                    self.set_status(VisionRuntimeStatus {
                        state: "ready".to_string(),
                        model_id: VISION_MODEL_ID.to_string(),
                        device: process.device.clone(),
                        cuda: process.cuda,
                        last_prompt: None,
                        last_analysis: None,
                        last_error: None,
                        refreshed_at_ms: timestamp_ms(),
                    });
                    return Ok(process);
                }
                "fatal" | "error" => {
                    return Err(envelope
                        .message
                        .unwrap_or_else(|| "Vision runtime failed to initialize.".to_string()));
                }
                _ => {}
            }
        }
    }

    fn set_error(&self, process: Option<&VisionProcess>, message: String) {
        let previous = self.status();
        self.set_status(VisionRuntimeStatus {
            state: "error".to_string(),
            model_id: VISION_MODEL_ID.to_string(),
            device: process.and_then(|running| running.device.clone()),
            cuda: process.and_then(|running| running.cuda),
            last_prompt: previous.last_prompt,
            last_analysis: previous.last_analysis,
            last_error: Some(message),
            refreshed_at_ms: timestamp_ms(),
        });
    }

    fn set_status(&self, status: VisionRuntimeStatus) {
        *self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = status;
    }
}

fn read_envelope(reader: &mut BufReader<ChildStdout>) -> Result<VisionEnvelope, String> {
    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .map_err(|error| format!("Could not read local Vision runtime: {error}"))?;

    if bytes == 0 {
        return Err("Local Vision runtime exited unexpectedly.".to_string());
    }

    serde_json::from_str(line.trim())
        .map_err(|error| format!("Local Vision runtime returned invalid data: {error}"))
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
