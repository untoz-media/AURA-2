use crate::{
    managed_runtime::managed_python_path,
    model_manager::ModelManager,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    fs,
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

const IMAGE_MODEL_ID: &str = "create-tiny-sd";
const IMAGE_RUNTIME_SCRIPT: &str = include_str!("image_runtime.py");

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageRuntimeStatus {
    pub state: String,
    pub model_id: String,
    pub device: Option<String>,
    pub cuda: Option<bool>,
    pub last_error: Option<String>,
    pub refreshed_at_ms: u64,
}

impl Default for ImageRuntimeStatus {
    fn default() -> Self {
        Self {
            state: "stopped".to_string(),
            model_id: IMAGE_MODEL_ID.to_string(),
            device: None,
            cuda: None,
            last_error: None,
            refreshed_at_ms: timestamp_ms(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGenerationRequest {
    pub prompt: String,
    #[serde(default)]
    pub negative_prompt: Option<String>,
    #[serde(default)]
    pub aspect_ratio: Option<String>,
    #[serde(default)]
    pub steps: Option<u32>,
    #[serde(default)]
    pub seed: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGenerationResult {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub path: String,
    pub data_url: String,
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    pub device: Option<String>,
    pub cuda: Option<bool>,
    pub completed_at_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageEnvelope {
    #[serde(rename = "type")]
    kind: String,
    id: Option<String>,
    path: Option<String>,
    data_url: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    seed: Option<u64>,
    message: Option<String>,
    device: Option<String>,
    cuda: Option<bool>,
}

struct ImageProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    device: Option<String>,
    cuda: Option<bool>,
}

pub struct ImageRuntime {
    process: Mutex<Option<ImageProcess>>,
    status: Mutex<ImageRuntimeStatus>,
    request_counter: AtomicU64,
}

impl Default for ImageRuntime {
    fn default() -> Self {
        Self {
            process: Mutex::new(None),
            status: Mutex::new(ImageRuntimeStatus::default()),
            request_counter: AtomicU64::new(1),
        }
    }
}

impl ImageRuntime {
    pub fn status(&self) -> ImageRuntimeStatus {
        self.status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn generate(
        &self,
        app: &AppHandle,
        manager: &ModelManager,
        request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResult, String> {
        let prompt = request.prompt.trim().to_string();
        if prompt.is_empty() {
            return Err("Image prompt cannot be empty.".to_string());
        }
        if prompt.chars().count() > 1000 {
            return Err("Image prompt cannot exceed 1000 characters.".to_string());
        }

        let negative_prompt = request
            .negative_prompt
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        if negative_prompt
            .as_ref()
            .is_some_and(|value| value.chars().count() > 1000)
        {
            return Err("Negative prompt cannot exceed 1000 characters.".to_string());
        }

        let (width, height) = image_dimensions(request.aspect_ratio.as_deref())?;
        let steps = request.steps.unwrap_or(20);
        if !(4..=40).contains(&steps) {
            return Err("Inference steps must be between 4 and 40.".to_string());
        }

        let request_number = self.request_counter.fetch_add(1, Ordering::Relaxed);
        let seed = request
            .seed
            .unwrap_or_else(|| (timestamp_ms() ^ request_number) & u32::MAX as u64);

        let output_dir = app
            .path()
            .picture_dir()
            .or_else(|_| app.path().app_local_data_dir())
            .map_err(|error| error.to_string())?
            .join("AURA Create");
        fs::create_dir_all(&output_dir)
            .map_err(|error| format!("Could not create AURA Create output directory: {error}"))?;
        let output_path = output_dir.join(format!(
            "AURA-Create-{}-{}.png",
            timestamp_ms(),
            request_number
        ));

        let model_path = manager.installation_path(app, IMAGE_MODEL_ID)?;
        let mut process_guard = self
            .process
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if process_guard.is_none() {
            *process_guard = Some(self.start_process(app, &model_path)?);
        }

        let request_id = format!("create-{}-{}", timestamp_ms(), request_number);
        if let Some(running) = process_guard.as_ref() {
            self.set_status(ImageRuntimeStatus {
                state: "generating".to_string(),
                model_id: IMAGE_MODEL_ID.to_string(),
                device: running.device.clone(),
                cuda: running.cuda,
                last_error: None,
                refreshed_at_ms: timestamp_ms(),
            });
        }

        let payload = json!({
            "type": "generate",
            "id": request_id,
            "prompt": prompt,
            "negativePrompt": negative_prompt,
            "outputPath": output_path.to_string_lossy(),
            "width": width,
            "height": height,
            "steps": steps,
            "seed": seed,
        });

        {
            let running = process_guard
                .as_mut()
                .ok_or_else(|| "AURA Create runtime failed to start.".to_string())?;
            writeln!(running.stdin, "{payload}")
                .and_then(|_| running.stdin.flush())
                .map_err(|error| format!("Could not send request to AURA Create: {error}"))?;
        }

        loop {
            let envelope = {
                let running = process_guard
                    .as_mut()
                    .ok_or_else(|| "AURA Create runtime is no longer available.".to_string())?;
                match read_envelope(&mut running.stdout) {
                    Ok(envelope) => envelope,
                    Err(error) => {
                        if let Some(mut failed) = process_guard.take() {
                            let _ = failed.child.kill();
                            let _ = failed.child.wait();
                        }
                        self.set_status(ImageRuntimeStatus {
                            state: "error".to_string(),
                            model_id: IMAGE_MODEL_ID.to_string(),
                            last_error: Some(error.clone()),
                            refreshed_at_ms: timestamp_ms(),
                            ..ImageRuntimeStatus::default()
                        });
                        return Err(error);
                    }
                }
            };

            if envelope.id.as_deref() != Some(request_id.as_str()) {
                continue;
            }

            match envelope.kind.as_str() {
                "image" => {
                    let path = envelope
                        .path
                        .ok_or_else(|| "AURA Create returned no output path.".to_string())?;
                    let data_url = envelope
                        .data_url
                        .ok_or_else(|| "AURA Create returned no preview image.".to_string())?;
                    let result = ImageGenerationResult {
                        prompt: prompt.clone(),
                        negative_prompt: negative_prompt.clone(),
                        path,
                        data_url,
                        width: envelope.width.unwrap_or(width),
                        height: envelope.height.unwrap_or(height),
                        seed: envelope.seed.unwrap_or(seed),
                        device: envelope.device.clone(),
                        cuda: envelope.cuda,
                        completed_at_ms: timestamp_ms(),
                    };

                    self.set_status(ImageRuntimeStatus {
                        state: "ready".to_string(),
                        model_id: IMAGE_MODEL_ID.to_string(),
                        device: envelope.device.or_else(|| {
                            process_guard
                                .as_ref()
                                .and_then(|running| running.device.clone())
                        }),
                        cuda: envelope.cuda.or_else(|| {
                            process_guard.as_ref().and_then(|running| running.cuda)
                        }),
                        last_error: None,
                        refreshed_at_ms: timestamp_ms(),
                    });

                    return Ok(result);
                }
                "error" | "fatal" => {
                    let message = envelope
                        .message
                        .unwrap_or_else(|| "AURA Create failed locally.".to_string());
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

        self.set_status(ImageRuntimeStatus::default());
    }

    fn start_process(&self, app: &AppHandle, model_path: &Path) -> Result<ImageProcess, String> {
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
            .join("create-runtime");
        fs::create_dir_all(&runtime_dir)
            .map_err(|error| format!("Could not create AURA Create runtime directory: {error}"))?;
        let script_path = runtime_dir.join("image_runtime.py");
        fs::write(&script_path, IMAGE_RUNTIME_SCRIPT)
            .map_err(|error| format!("Could not write AURA Create runtime script: {error}"))?;

        self.set_status(ImageRuntimeStatus {
            state: "loading".to_string(),
            model_id: IMAGE_MODEL_ID.to_string(),
            refreshed_at_ms: timestamp_ms(),
            ..ImageRuntimeStatus::default()
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
            .map_err(|error| format!("Could not start AURA Create runtime: {error}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "AURA Create runtime stdin is unavailable.".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "AURA Create runtime stdout is unavailable.".to_string())?;

        let mut process = ImageProcess {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            device: None,
            cuda: None,
        };

        loop {
            let envelope = match read_envelope(&mut process.stdout) {
                Ok(envelope) => envelope,
                Err(error) => {
                    let _ = process.child.kill();
                    let _ = process.child.wait();
                    self.set_status(ImageRuntimeStatus {
                        state: "error".to_string(),
                        model_id: IMAGE_MODEL_ID.to_string(),
                        last_error: Some(error.clone()),
                        refreshed_at_ms: timestamp_ms(),
                        ..ImageRuntimeStatus::default()
                    });
                    return Err(error);
                }
            };
            match envelope.kind.as_str() {
                "ready" => {
                    process.device = envelope.device;
                    process.cuda = envelope.cuda;
                    self.set_status(ImageRuntimeStatus {
                        state: "ready".to_string(),
                        model_id: IMAGE_MODEL_ID.to_string(),
                        device: process.device.clone(),
                        cuda: process.cuda,
                        last_error: None,
                        refreshed_at_ms: timestamp_ms(),
                    });
                    return Ok(process);
                }
                "fatal" | "error" => {
                    let message = envelope
                        .message
                        .unwrap_or_else(|| "AURA Create runtime failed to initialize.".to_string());
                    let _ = process.child.kill();
                    let _ = process.child.wait();
                    self.set_status(ImageRuntimeStatus {
                        state: "error".to_string(),
                        model_id: IMAGE_MODEL_ID.to_string(),
                        last_error: Some(message.clone()),
                        refreshed_at_ms: timestamp_ms(),
                        ..ImageRuntimeStatus::default()
                    });
                    return Err(message);
                }
                _ => {}
            }
        }
    }

    fn set_error(&self, process: Option<&ImageProcess>, message: String) {
        self.set_status(ImageRuntimeStatus {
            state: "error".to_string(),
            model_id: IMAGE_MODEL_ID.to_string(),
            device: process.and_then(|running| running.device.clone()),
            cuda: process.and_then(|running| running.cuda),
            last_error: Some(message),
            refreshed_at_ms: timestamp_ms(),
        });
    }

    fn set_status(&self, status: ImageRuntimeStatus) {
        *self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = status;
    }
}

fn image_dimensions(aspect_ratio: Option<&str>) -> Result<(u32, u32), String> {
    match aspect_ratio.unwrap_or("square") {
        "square" => Ok((512, 512)),
        "landscape" => Ok((640, 384)),
        "portrait" => Ok((384, 640)),
        _ => Err("Unsupported image aspect ratio.".to_string()),
    }
}

fn read_envelope(reader: &mut BufReader<ChildStdout>) -> Result<ImageEnvelope, String> {
    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .map_err(|error| format!("Could not read AURA Create runtime: {error}"))?;

    if bytes == 0 {
        return Err("AURA Create runtime exited unexpectedly.".to_string());
    }

    serde_json::from_str(line.trim())
        .map_err(|error| format!("AURA Create runtime returned invalid data: {error}"))
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
    fn image_presets_are_bounded_and_divisible_by_eight() {
        for preset in ["square", "landscape", "portrait"] {
            let (width, height) = image_dimensions(Some(preset)).expect("known preset");
            assert!((256..=768).contains(&width));
            assert!((256..=768).contains(&height));
            assert_eq!(width % 8, 0);
            assert_eq!(height % 8, 0);
        }
    }

    #[test]
    fn unknown_aspect_ratio_is_rejected() {
        assert!(image_dimensions(Some("cinematic-ultrawide")).is_err());
    }
}
