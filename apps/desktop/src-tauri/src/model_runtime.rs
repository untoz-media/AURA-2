use crate::model_manager::ModelManager;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    env,
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
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

const RUNTIME_SCRIPT: &str = include_str!("model_runtime.py");
const MAX_CONVERSATION_MESSAGES: usize = 20;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelRuntimeStatus {
    pub state: String,
    pub loaded_model_id: Option<String>,
    pub python_executable: Option<String>,
    pub device: Option<String>,
    pub cuda: Option<bool>,
    pub last_error: Option<String>,
    pub refreshed_at_ms: u64,
}

impl Default for ModelRuntimeStatus {
    fn default() -> Self {
        Self {
            state: "stopped".to_string(),
            loaded_model_id: None,
            python_executable: None,
            device: None,
            cuda: None,
            last_error: None,
            refreshed_at_ms: timestamp_ms(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct RuntimeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeEnvelope {
    #[serde(rename = "type")]
    kind: String,
    id: Option<String>,
    text: Option<String>,
    message: Option<String>,
    python: Option<String>,
    device: Option<String>,
    cuda: Option<bool>,
}

struct RuntimeProcess {
    model_id: String,
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    python_executable: String,
    device: Option<String>,
    cuda: Option<bool>,
}

#[derive(Clone)]
struct PythonCommand {
    program: String,
    prefix_args: Vec<String>,
    display: String,
}

pub struct ModelRuntime {
    process: Mutex<Option<RuntimeProcess>>,
    conversation: Mutex<Vec<RuntimeMessage>>,
    status: Mutex<ModelRuntimeStatus>,
    request_counter: AtomicU64,
}

impl Default for ModelRuntime {
    fn default() -> Self {
        Self {
            process: Mutex::new(None),
            conversation: Mutex::new(Vec::new()),
            status: Mutex::new(ModelRuntimeStatus::default()),
            request_counter: AtomicU64::new(1),
        }
    }
}

impl ModelRuntime {
    pub fn status(&self) -> ModelRuntimeStatus {
        self.status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn clear_conversation(&self) {
        self.conversation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }

    pub fn stop(&self) {
        let mut process = self
            .process
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(mut running) = process.take() {
            let request = json!({
                "type": "shutdown",
                "id": "shutdown",
            });
            let _ = writeln!(running.stdin, "{request}");
            let _ = running.stdin.flush();
            let _ = running.child.kill();
            let _ = running.child.wait();
        }

        self.clear_conversation();
        self.set_status(ModelRuntimeStatus::default());
    }

    pub fn generate(
        &self,
        app: &AppHandle,
        manager: &ModelManager,
        user_text: &str,
    ) -> Result<String, String> {
        let user_text = user_text.trim();
        if user_text.is_empty() {
            return Err("The model runtime received an empty message.".to_string());
        }

        let (model_id, model_path) = manager.active_installation(app)?;

        let mut process_guard = self
            .process
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let needs_restart = process_guard
            .as_ref()
            .is_some_and(|process| process.model_id != model_id);

        if needs_restart {
            if let Some(mut running) = process_guard.take() {
                let _ = running.child.kill();
                let _ = running.child.wait();
            }
            self.clear_conversation();
        }

        if process_guard.is_none() {
            let process = self.start_process(app, &model_id, &model_path)?;
            *process_guard = Some(process);
        }

        let request_id = format!(
            "generation-{}-{}",
            timestamp_ms(),
            self.request_counter.fetch_add(1, Ordering::Relaxed)
        );

        let messages = {
            let conversation = self
                .conversation
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut messages = conversation.clone();
            messages.push(RuntimeMessage {
                role: "user".to_string(),
                content: user_text.to_string(),
            });
            messages
        };

        let request = json!({
            "type": "generate",
            "id": request_id.clone(),
            "messages": messages,
        });

        {
            let running = process_guard
                .as_ref()
                .ok_or_else(|| "The model runtime failed to start.".to_string())?;

            self.set_status(ModelRuntimeStatus {
                state: "generating".to_string(),
                loaded_model_id: Some(running.model_id.clone()),
                python_executable: Some(running.python_executable.clone()),
                device: running.device.clone(),
                cuda: running.cuda,
                last_error: None,
                refreshed_at_ms: timestamp_ms(),
            });
        }

        let send_result = {
            let running = process_guard
                .as_mut()
                .ok_or_else(|| "The model runtime failed to start.".to_string())?;
            writeln!(running.stdin, "{request}").and_then(|_| running.stdin.flush())
        };

        if let Err(error) = send_result {
            let message = format!("Could not send the message to the local model runtime: {error}");
            self.mark_process_failed(&mut process_guard, &message);
            return Err(message);
        }

        loop {
            let envelope_result = {
                let running = process_guard
                    .as_mut()
                    .ok_or_else(|| "The model runtime is no longer available.".to_string())?;
                read_envelope(&mut running.stdout)
            };

            let envelope = match envelope_result {
                Ok(envelope) => envelope,
                Err(error) => {
                    let message =
                        format!("The local model runtime stopped responding: {error}");
                    self.mark_process_failed(&mut process_guard, &message);
                    return Err(message);
                }
            };

            if envelope.id.as_deref() != Some(request_id.as_str()) {
                continue;
            }

            match envelope.kind.as_str() {
                "response" => {
                    let response = envelope
                        .text
                        .unwrap_or_default()
                        .trim()
                        .to_string();

                    if response.is_empty() {
                        let message =
                            "The local model returned an empty response.".to_string();
                        if let Some(running) = process_guard.as_ref() {
                            self.set_ready_with_error(running, Some(message.clone()));
                        }
                        return Err(message);
                    }

                    {
                        let mut conversation = self
                            .conversation
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        conversation.push(RuntimeMessage {
                            role: "user".to_string(),
                            content: user_text.to_string(),
                        });
                        conversation.push(RuntimeMessage {
                            role: "assistant".to_string(),
                            content: response.clone(),
                        });

                        while conversation.len() > MAX_CONVERSATION_MESSAGES {
                            let remove_count = if conversation.len() >= 2 { 2 } else { 1 };
                            conversation.drain(0..remove_count);
                        }
                    }

                    if let Some(running) = process_guard.as_ref() {
                        self.set_ready_with_error(running, None);
                    }
                    return Ok(response);
                }
                "error" | "fatal" => {
                    let message = envelope
                        .message
                        .unwrap_or_else(|| "Local generation failed.".to_string());

                    if envelope.kind == "fatal" {
                        self.mark_process_failed(&mut process_guard, &message);
                    } else if let Some(running) = process_guard.as_ref() {
                        self.set_ready_with_error(running, Some(message.clone()));
                    }
                    return Err(message);
                }
                _ => {}
            }
        }
    }

    fn start_process(
        &self,
        app: &AppHandle,
        model_id: &str,
        model_path: &Path,
    ) -> Result<RuntimeProcess, String> {
        self.set_status(ModelRuntimeStatus {
            state: "loading".to_string(),
            loaded_model_id: Some(model_id.to_string()),
            python_executable: None,
            device: None,
            cuda: None,
            last_error: None,
            refreshed_at_ms: timestamp_ms(),
        });

        let script_path = write_runtime_script(app)?;
        let python = find_python(app).ok_or_else(|| {
            let message = "Python 3 was not found for the local model runtime. Install Python 3 or set AURA_PYTHON to a compatible Python executable.".to_string();
            self.set_error_status(Some(model_id.to_string()), &message);
            message
        })?;

        let mut command = Command::new(&python.program);
        command
            .args(&python.prefix_args)
            .arg("-u")
            .arg(&script_path)
            .arg("--model-path")
            .arg(model_path)
            .arg("--model-id")
            .arg(model_id)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        #[cfg(windows)]
        {
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = command.spawn().map_err(|error| {
            let message = format!(
                "Could not start the local model runtime with {}: {error}",
                python.display
            );
            self.set_error_status(Some(model_id.to_string()), &message);
            message
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Could not open model runtime input.".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Could not open model runtime output.".to_string())?;
        let mut stdout = BufReader::new(stdout);

        loop {
            let envelope = read_envelope(&mut stdout).map_err(|error| {
                let _ = child.kill();
                let message = format!(
                    "The local model runtime exited while loading: {error}"
                );
                self.set_error_status(Some(model_id.to_string()), &message);
                message
            })?;

            match envelope.kind.as_str() {
                "boot" => {
                    self.set_status(ModelRuntimeStatus {
                        state: "loading".to_string(),
                        loaded_model_id: Some(model_id.to_string()),
                        python_executable: envelope
                            .python
                            .clone()
                            .or_else(|| Some(python.display.clone())),
                        device: None,
                        cuda: None,
                        last_error: None,
                        refreshed_at_ms: timestamp_ms(),
                    });
                }
                "ready" => {
                    let process = RuntimeProcess {
                        model_id: model_id.to_string(),
                        child,
                        stdin,
                        stdout,
                        python_executable: envelope
                            .python
                            .unwrap_or_else(|| python.display.clone()),
                        device: envelope.device,
                        cuda: envelope.cuda,
                    };

                    self.set_status(ModelRuntimeStatus {
                        state: "ready".to_string(),
                        loaded_model_id: Some(model_id.to_string()),
                        python_executable: Some(process.python_executable.clone()),
                        device: process.device.clone(),
                        cuda: process.cuda,
                        last_error: None,
                        refreshed_at_ms: timestamp_ms(),
                    });

                    return Ok(process);
                }
                "fatal" | "error" => {
                    let _ = child.kill();
                    let message = envelope
                        .message
                        .unwrap_or_else(|| "The local model runtime failed to load.".to_string());
                    self.set_error_status(Some(model_id.to_string()), &message);
                    return Err(message);
                }
                _ => {}
            }
        }
    }

    fn set_ready_with_error(&self, process: &RuntimeProcess, error: Option<String>) {
        self.set_status(ModelRuntimeStatus {
            state: "ready".to_string(),
            loaded_model_id: Some(process.model_id.clone()),
            python_executable: Some(process.python_executable.clone()),
            device: process.device.clone(),
            cuda: process.cuda,
            last_error: error,
            refreshed_at_ms: timestamp_ms(),
        });
    }

    fn mark_process_failed(
        &self,
        process_guard: &mut Option<RuntimeProcess>,
        message: &str,
    ) {
        let model_id = process_guard
            .as_ref()
            .map(|process| process.model_id.clone());

        if let Some(mut process) = process_guard.take() {
            let _ = process.child.kill();
            let _ = process.child.wait();
        }

        self.set_error_status(model_id, message);
    }

    fn set_error_status(&self, model_id: Option<String>, message: &str) {
        self.set_status(ModelRuntimeStatus {
            state: "error".to_string(),
            loaded_model_id: model_id,
            python_executable: None,
            device: None,
            cuda: None,
            last_error: Some(message.to_string()),
            refreshed_at_ms: timestamp_ms(),
        });
    }

    fn set_status(&self, status: ModelRuntimeStatus) {
        *self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = status;
    }
}

fn read_envelope(reader: &mut BufReader<ChildStdout>) -> Result<RuntimeEnvelope, String> {
    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .map_err(|error| error.to_string())?;

    if bytes == 0 {
        return Err("runtime process closed its output stream".to_string());
    }

    serde_json::from_str::<RuntimeEnvelope>(line.trim())
        .map_err(|error| format!("invalid runtime response: {error}"))
}

fn write_runtime_script(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_local_data_dir()
        .map_err(|error| error.to_string())?
        .join("runtime");
    fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create runtime directory: {error}"))?;

    let path = directory.join("model_runtime.py");
    let should_write = fs::read_to_string(&path)
        .map(|current| current != RUNTIME_SCRIPT)
        .unwrap_or(true);

    if should_write {
        fs::write(&path, RUNTIME_SCRIPT)
            .map_err(|error| format!("Could not prepare model runtime script: {error}"))?;
    }

    Ok(path)
}

fn find_python(app: &AppHandle) -> Option<PythonCommand> {
    let mut candidates = Vec::new();

    if let Ok(value) = env::var("AURA_PYTHON") {
        let value = value.trim();
        if !value.is_empty() {
            candidates.push(PythonCommand {
                program: value.to_string(),
                prefix_args: Vec::new(),
                display: value.to_string(),
            });
        }
    }

    if let Ok(local_data) = app.path().app_local_data_dir() {
        let bundled = local_data.join("runtime").join("python").join("python.exe");
        if bundled.exists() {
            let display = bundled.to_string_lossy().to_string();
            candidates.push(PythonCommand {
                program: display.clone(),
                prefix_args: Vec::new(),
                display,
            });
        }
    }

    candidates.extend([
        PythonCommand {
            program: "python".to_string(),
            prefix_args: Vec::new(),
            display: "python".to_string(),
        },
        PythonCommand {
            program: "py".to_string(),
            prefix_args: vec!["-3".to_string()],
            display: "py -3".to_string(),
        },
        PythonCommand {
            program: "python3".to_string(),
            prefix_args: Vec::new(),
            display: "python3".to_string(),
        },
    ]);

    candidates.into_iter().find(|candidate| {
        let mut command = Command::new(&candidate.program);
        command
            .args(&candidate.prefix_args)
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        #[cfg(windows)]
        {
            command.creation_flags(CREATE_NO_WINDOW);
        }

        command.status().map(|status| status.success()).unwrap_or(false)
    })
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
    fn runtime_status_defaults_to_stopped() {
        let status = ModelRuntimeStatus::default();
        assert_eq!(status.state, "stopped");
        assert!(status.loaded_model_id.is_none());
    }

    #[test]
    fn embedded_runtime_is_local_only() {
        assert!(RUNTIME_SCRIPT.contains("local_files_only=True"));
        assert!(RUNTIME_SCRIPT.contains("AutoModelForCausalLM"));
        assert!(RUNTIME_SCRIPT.contains("BitsAndBytesConfig"));
    }

    #[test]
    fn conversation_limit_is_even_for_turn_pairs() {
        assert_eq!(MAX_CONVERSATION_MESSAGES % 2, 0);
        assert!(MAX_CONVERSATION_MESSAGES >= 10);
    }
}
