use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const MANAGED_RUNTIME_EVENT: &str = "aura:managed-runtime";
const PYTHON_VERSION: &str = "3.12.10";
const PYTHON_INSTALLER_NAME: &str = "python-3.12.10-amd64.exe";
const PYTHON_INSTALLER_URL: &str =
    "https://www.python.org/ftp/python/3.12.10/python-3.12.10-amd64.exe";
const MIN_PYTHON_INSTALLER_BYTES: u64 = 20_000_000;
const MIN_RUNTIME_FREE_SPACE_BYTES: u64 = 10_000_000_000;
const RUNTIME_MARKER: &str = "managed-runtime.json";

const PYTORCH_CUDA_INDEX: &str = "https://download.pytorch.org/whl/cu128";
const PYTORCH_CPU_INDEX: &str = "https://download.pytorch.org/whl/cpu";

const RUNTIME_PACKAGES: &[&str] = &[
    "transformers>=5.0",
    "accelerate>=1.0",
    "bitsandbytes>=0.45",
    "safetensors>=0.4",
    "Pillow>=11.0",
];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedRuntimeStatus {
    pub state: String,
    pub progress_percent: f64,
    pub message: String,
    pub python_path: Option<String>,
    pub python_version: Option<String>,
    pub torch_version: Option<String>,
    pub transformers_version: Option<String>,
    pub accelerate_version: Option<String>,
    pub bitsandbytes_version: Option<String>,
    pub cuda_available: Option<bool>,
    pub cuda_device_name: Option<String>,
    pub last_error: Option<String>,
    pub updated_at_ms: u64,
}

impl Default for ManagedRuntimeStatus {
    fn default() -> Self {
        Self {
            state: "notInstalled".to_string(),
            progress_percent: 0.0,
            message: "Managed AI runtime is not installed.".to_string(),
            python_path: None,
            python_version: None,
            torch_version: None,
            transformers_version: None,
            accelerate_version: None,
            bitsandbytes_version: None,
            cuda_available: None,
            cuda_device_name: None,
            last_error: None,
            updated_at_ms: timestamp_ms(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeMarker {
    runtime_version: u32,
    python_version: String,
    torch_version: String,
    transformers_version: String,
    accelerate_version: String,
    bitsandbytes_version: String,
    cuda_available: bool,
    cuda_device_name: Option<String>,
    installed_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct VerificationPayload {
    python: String,
    torch: String,
    transformers: String,
    accelerate: String,
    bitsandbytes: String,
    cuda: bool,
    device: Option<String>,
}

#[derive(Clone, Default)]
pub struct ManagedRuntimeSetup {
    status: Arc<Mutex<ManagedRuntimeStatus>>,
}

impl ManagedRuntimeSetup {
    pub fn status(&self, app: &AppHandle) -> ManagedRuntimeStatus {
        let current = self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();

        if is_busy_state(&current.state) {
            return current;
        }

        let inspected = inspect_runtime(app).unwrap_or_else(|error| ManagedRuntimeStatus {
            state: "error".to_string(),
            progress_percent: 0.0,
            message: "Managed runtime needs attention.".to_string(),
            last_error: Some(error),
            ..ManagedRuntimeStatus::default()
        });

        self.set_status(app, inspected.clone());
        inspected
    }

    pub fn start_install(
        &self,
        app: AppHandle,
        repair: bool,
    ) -> Result<ManagedRuntimeStatus, String> {
        let current = self.status(&app);
        if is_busy_state(&current.state) {
            return Err("Managed runtime setup is already running.".to_string());
        }

        self.set_status(
            &app,
            ManagedRuntimeStatus {
                state: "preparing".to_string(),
                progress_percent: 2.0,
                message: if repair {
                    "Preparing a clean managed runtime repair…".to_string()
                } else {
                    "Preparing the managed AI runtime…".to_string()
                },
                last_error: None,
                ..current
            },
        );

        let setup = self.clone();
        let app_for_worker = app.clone();
        thread::spawn(move || {
            if let Err(error) = setup.run_setup(&app_for_worker, repair) {
                setup.set_status(
                    &app_for_worker,
                    ManagedRuntimeStatus {
                        state: "error".to_string(),
                        progress_percent: 0.0,
                        message: "Managed runtime setup failed.".to_string(),
                        python_path: runtime_python_path(&app_for_worker)
                            .ok()
                            .filter(|path| path.exists())
                            .map(|path| path.to_string_lossy().to_string()),
                        last_error: Some(error),
                        ..ManagedRuntimeStatus::default()
                    },
                );
            }
        });

        Ok(self.status(&app))
    }

    pub fn remove(&self, app: &AppHandle) -> Result<ManagedRuntimeStatus, String> {
        let current = self.status(app);
        if is_busy_state(&current.state) {
            return Err("Managed runtime setup is still running.".to_string());
        }

        let root = runtime_root(app)?;
        if root.exists() {
            fs::remove_dir_all(&root)
                .map_err(|error| format!("Could not remove managed runtime: {error}"))?;
        }

        let cache = runtime_cache_dir(app)?;
        if cache.exists() {
            let _ = fs::remove_dir_all(cache);
        }

        let status = ManagedRuntimeStatus::default();
        self.set_status(app, status.clone());
        Ok(status)
    }

    fn run_setup(&self, app: &AppHandle, repair: bool) -> Result<(), String> {
        #[cfg(not(windows))]
        {
            let _ = repair;
            return Err("Managed runtime setup currently supports Windows only.".to_string());
        }

        #[cfg(windows)]
        {
            let root = runtime_root(app)?;
            let python_dir = runtime_python_dir(app)?;

            if let Some(parent) = root.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("Could not create runtime parent directory: {error}"))?;
                let free = fs2::available_space(parent)
                    .map_err(|error| format!("Could not read available runtime disk space: {error}"))?;
                let reclaimable = if repair && root.exists() {
                    directory_size(&root).unwrap_or(0)
                } else {
                    0
                };
                let effective_free = free.saturating_add(reclaimable);

                if effective_free < MIN_RUNTIME_FREE_SPACE_BYTES {
                    return Err(format!(
                        "AURA needs at least {:.0} GB of effective free space to prepare the managed AI runtime safely.",
                        MIN_RUNTIME_FREE_SPACE_BYTES as f64 / 1_000_000_000.0
                    ));
                }
            }

            if repair && root.exists() {
                fs::remove_dir_all(&root)
                    .map_err(|error| format!("Could not reset managed runtime: {error}"))?;
            }

            fs::create_dir_all(&root)
                .map_err(|error| format!("Could not create managed runtime directory: {error}"))?;

            let python_exe = runtime_python_path(app)?;

            if !python_exe.exists() {
                self.update_phase(
                    app,
                    "downloadingPython",
                    8.0,
                    "Downloading Python 3.12 for AURA…",
                );

                let installer = self.download_python_installer(app)?;

                self.update_phase(
                    app,
                    "verifyingInstaller",
                    24.0,
                    "Verifying the official Python installer…",
                );
                verify_authenticode(&installer)?;

                self.update_phase(
                    app,
                    "installingPython",
                    30.0,
                    "Installing AURA's private Python runtime…",
                );
                install_python(&installer, &python_dir)?;

                if !python_exe.exists() {
                    return Err(
                        "Python installer completed but AURA could not find python.exe.".to_string(),
                    );
                }
            }

            self.update_phase(
                app,
                "preparingPackages",
                42.0,
                "Updating pip inside the managed runtime…",
            );
            run_python(
                &python_exe,
                &[
                    "-m",
                    "pip",
                    "install",
                    "--disable-pip-version-check",
                    "--no-input",
                    "--upgrade",
                    "pip",
                ],
            )?;

            let nvidia = nvidia_gpu_available();

            self.update_phase(
                app,
                "installingPackages",
                50.0,
                if nvidia {
                    "Installing CUDA-enabled PyTorch for the detected NVIDIA GPU…"
                } else {
                    "Installing CPU PyTorch runtime…"
                },
            );

            let torch_index = if nvidia {
                PYTORCH_CUDA_INDEX
            } else {
                PYTORCH_CPU_INDEX
            };

            run_python(
                &python_exe,
                &[
                    "-m",
                    "pip",
                    "install",
                    "--disable-pip-version-check",
                    "--no-input",
                    "--upgrade",
                    "--index-url",
                    torch_index,
                    "torch>=2.7",
                ],
            )?;

            self.update_phase(
                app,
                "installingPackages",
                76.0,
                "Installing Transformers, Accelerate and quantization dependencies…",
            );

            let mut args = vec![
                "-m",
                "pip",
                "install",
                "--disable-pip-version-check",
                "--no-input",
                "--upgrade",
            ];
            args.extend(RUNTIME_PACKAGES.iter().copied());
            run_python(&python_exe, &args)?;

            self.update_phase(
                app,
                "verifying",
                94.0,
                "Verifying the managed AI runtime…",
            );

            let verification = verify_python_runtime(&python_exe)?;
            let marker = RuntimeMarker {
                runtime_version: 1,
                python_version: verification.python.clone(),
                torch_version: verification.torch.clone(),
                transformers_version: verification.transformers.clone(),
                accelerate_version: verification.accelerate.clone(),
                bitsandbytes_version: verification.bitsandbytes.clone(),
                cuda_available: verification.cuda,
                cuda_device_name: verification.device.clone(),
                installed_at_ms: timestamp_ms(),
            };

            let marker_content =
                serde_json::to_string_pretty(&marker).map_err(|error| error.to_string())?;
            fs::write(root.join(RUNTIME_MARKER), marker_content)
                .map_err(|error| format!("Could not write managed runtime marker: {error}"))?;

            let status = status_from_marker(&python_exe, marker);
            self.set_status(app, status);
            Ok(())
        }
    }

    #[cfg(windows)]
    fn download_python_installer(&self, app: &AppHandle) -> Result<PathBuf, String> {
        let cache_dir = runtime_cache_dir(app)?;
        fs::create_dir_all(&cache_dir)
            .map_err(|error| format!("Could not create runtime download cache: {error}"))?;
        let installer = cache_dir.join(PYTHON_INSTALLER_NAME);

        if installer.exists() {
            let size = fs::metadata(&installer)
                .map_err(|error| error.to_string())?
                .len();
            if size >= MIN_PYTHON_INSTALLER_BYTES && verify_authenticode(&installer).is_ok() {
                return Ok(installer);
            }
            let _ = fs::remove_file(&installer);
        }

        let client = Client::builder()
            .user_agent(format!("AURA-2/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|error| format!("Could not initialize runtime downloader: {error}"))?;

        let mut response = client
            .get(PYTHON_INSTALLER_URL)
            .send()
            .map_err(|error| format!("Could not download Python runtime: {error}"))?
            .error_for_status()
            .map_err(|error| format!("Could not download Python runtime: {error}"))?;

        let total = response.content_length().unwrap_or(0);
        let mut output = fs::File::create(&installer)
            .map_err(|error| format!("Could not create Python installer cache: {error}"))?;
        let mut buffer = [0_u8; 64 * 1024];
        let mut downloaded = 0_u64;
        let mut last_bucket = 0_u64;

        loop {
            let read = response
                .read(&mut buffer)
                .map_err(|error| format!("Python runtime download failed: {error}"))?;
            if read == 0 {
                break;
            }

            output
                .write_all(&buffer[..read])
                .map_err(|error| format!("Could not save Python installer: {error}"))?;
            downloaded = downloaded.saturating_add(read as u64);

            let bucket = downloaded / (512 * 1024);
            if bucket != last_bucket {
                last_bucket = bucket;
                let transfer = if total > 0 {
                    (downloaded as f64 / total as f64).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                self.update_phase(
                    app,
                    "downloadingPython",
                    8.0 + transfer * 15.0,
                    &if total > 0 {
                        format!(
                            "Downloading Python 3.12… {:.1} MB / {:.1} MB",
                            downloaded as f64 / 1_000_000.0,
                            total as f64 / 1_000_000.0
                        )
                    } else {
                        format!(
                            "Downloading Python 3.12… {:.1} MB",
                            downloaded as f64 / 1_000_000.0
                        )
                    },
                );
            }
        }

        output
            .flush()
            .map_err(|error| format!("Could not finalize Python installer: {error}"))?;

        let actual = fs::metadata(&installer)
            .map_err(|error| error.to_string())?
            .len();

        if actual < MIN_PYTHON_INSTALLER_BYTES {
            let _ = fs::remove_file(&installer);
            return Err(format!(
                "Python installer download is unexpectedly small ({actual} bytes)."
            ));
        }

        if total > 0 && actual != total {
            let _ = fs::remove_file(&installer);
            return Err(format!(
                "Python installer download is incomplete. Expected {total} bytes, got {actual}."
            ));
        }

        Ok(installer)
    }

    fn update_phase(
        &self,
        app: &AppHandle,
        state: &str,
        progress_percent: f64,
        message: &str,
    ) {
        let current = self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();

        self.set_status(
            app,
            ManagedRuntimeStatus {
                state: state.to_string(),
                progress_percent,
                message: message.to_string(),
                last_error: None,
                updated_at_ms: timestamp_ms(),
                ..current
            },
        );
    }

    fn set_status(&self, app: &AppHandle, status: ManagedRuntimeStatus) {
        {
            let mut current = self
                .status
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *current = status.clone();
        }

        let _ = app.emit(MANAGED_RUNTIME_EVENT, status);
    }
}

fn inspect_runtime(app: &AppHandle) -> Result<ManagedRuntimeStatus, String> {
    let python = runtime_python_path(app)?;
    let marker_path = runtime_root(app)?.join(RUNTIME_MARKER);

    if !python.exists() && !marker_path.exists() {
        return Ok(ManagedRuntimeStatus::default());
    }

    if !python.exists() {
        return Err("Managed runtime marker exists but python.exe is missing.".to_string());
    }

    if !marker_path.exists() {
        return Ok(ManagedRuntimeStatus {
            state: "needsRepair".to_string(),
            progress_percent: 0.0,
            message: "Managed Python exists, but the AI dependency verification is incomplete."
                .to_string(),
            python_path: Some(python.to_string_lossy().to_string()),
            last_error: Some(
                "Use Repair runtime to verify or reinstall the managed environment.".to_string(),
            ),
            ..ManagedRuntimeStatus::default()
        });
    }

    let content = fs::read_to_string(marker_path)
        .map_err(|error| format!("Could not read managed runtime marker: {error}"))?;
    let marker: RuntimeMarker = serde_json::from_str(&content)
        .map_err(|error| format!("Managed runtime marker is invalid: {error}"))?;

    if marker.runtime_version != 1 {
        return Err("Managed runtime version is no longer supported.".to_string());
    }

    Ok(status_from_marker(&python, marker))
}

fn status_from_marker(python: &Path, marker: RuntimeMarker) -> ManagedRuntimeStatus {
    ManagedRuntimeStatus {
        state: "ready".to_string(),
        progress_percent: 100.0,
        message: if marker.cuda_available {
            "Managed AI runtime is ready with CUDA acceleration.".to_string()
        } else {
            "Managed AI runtime is ready. CUDA is not currently available.".to_string()
        },
        python_path: Some(python.to_string_lossy().to_string()),
        python_version: Some(marker.python_version),
        torch_version: Some(marker.torch_version),
        transformers_version: Some(marker.transformers_version),
        accelerate_version: Some(marker.accelerate_version),
        bitsandbytes_version: Some(marker.bitsandbytes_version),
        cuda_available: Some(marker.cuda_available),
        cuda_device_name: marker.cuda_device_name,
        last_error: None,
        updated_at_ms: timestamp_ms(),
    }
}

#[cfg(windows)]
fn verify_authenticode(installer: &Path) -> Result<(), String> {
    let path = installer
        .to_string_lossy()
        .replace('\'', "''");
    let script = format!(
        "$sig = Get-AuthenticodeSignature -LiteralPath '{}'; if ($sig.Status -ne 'Valid') {{ Write-Error $sig.StatusMessage; exit 2 }}",
        path
    );

    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW);

    let output = command
        .output()
        .map_err(|error| format!("Could not verify Python installer signature: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    Err(format!(
        "The Python installer did not pass Windows Authenticode verification: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}

#[cfg(windows)]
fn install_python(installer: &Path, target: &Path) -> Result<(), String> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create runtime parent directory: {error}"))?;
    }

    let target_arg = format!("TargetDir={}", target.to_string_lossy());

    let mut command = Command::new(installer);
    command
        .arg("/quiet")
        .arg("InstallAllUsers=0")
        .arg(&target_arg)
        .arg("Include_launcher=0")
        .arg("InstallLauncherAllUsers=0")
        .arg("AssociateFiles=0")
        .arg("Shortcuts=0")
        .arg("PrependPath=0")
        .arg("AppendPath=0")
        .arg("Include_pip=1")
        .arg("Include_test=0")
        .arg("Include_doc=0")
        .arg("Include_tcltk=0")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW);

    let status = command
        .status()
        .map_err(|error| format!("Could not launch Python installer: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Python installer exited with status {}.",
            status.code().unwrap_or(-1)
        ))
    }
}

#[cfg(windows)]
fn nvidia_gpu_available() -> bool {
    let mut command = Command::new("nvidia-smi");
    command
        .arg("-L")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW);

    command
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(windows)]
fn run_python(python: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new(python)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|error| format!("Could not run managed Python: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        return Err(format!("Managed Python command failed: {detail}"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(windows)]
fn verify_python_runtime(python: &Path) -> Result<VerificationPayload, String> {
    let script = r#"import json, sys, torch, transformers, accelerate, bitsandbytes
payload = {
    "python": sys.version.split()[0],
    "torch": torch.__version__,
    "transformers": transformers.__version__,
    "accelerate": accelerate.__version__,
    "bitsandbytes": bitsandbytes.__version__,
    "cuda": bool(torch.cuda.is_available()),
    "device": torch.cuda.get_device_name(0) if torch.cuda.is_available() else None,
}
print(json.dumps(payload))"#;

    let output = run_python(python, &["-c", script])?;
    serde_json::from_str(&output)
        .map_err(|error| format!("Managed runtime verification returned invalid data: {error}"))
}

fn directory_size(path: &Path) -> Result<u64, String> {
    if !path.exists() {
        return Ok(0);
    }

    let mut total = 0_u64;
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let metadata = entry.metadata().map_err(|error| error.to_string())?;

        if metadata.is_dir() {
            total = total.saturating_add(directory_size(&entry.path())?);
        } else {
            total = total.saturating_add(metadata.len());
        }
    }

    Ok(total)
}

fn runtime_root(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_local_data_dir()
        .map_err(|error| error.to_string())?
        .join("runtime"))
}

fn runtime_python_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(runtime_root(app)?.join("python"))
}

pub(crate) fn managed_python_path(app: &AppHandle) -> Result<PathBuf, String> {
    runtime_python_path(app)
}

pub(crate) fn python_module_available(
    app: &AppHandle,
    module: &str,
) -> Result<bool, String> {
    let python = runtime_python_path(app)?;
    if !python.exists() {
        return Ok(false);
    }

    #[cfg(windows)]
    {
        let script = format!(
            "import importlib.util, sys; sys.exit(0 if importlib.util.find_spec({:?}) else 1)",
            module
        );
        let mut command = Command::new(&python);
        command
            .args(["-c", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW);

        return command
            .status()
            .map(|status| status.success())
            .map_err(|error| format!("Could not inspect managed Python module {module}: {error}"));
    }

    #[cfg(not(windows))]
    {
        let _ = module;
        Ok(false)
    }
}

pub(crate) fn install_managed_python_package(
    app: &AppHandle,
    package: &str,
) -> Result<(), String> {
    let python = runtime_python_path(app)?;
    if !python.exists() {
        return Err("AURA Managed Runtime is not installed. Install it from Models first.".to_string());
    }

    #[cfg(windows)]
    {
        run_python(
            &python,
            &[
                "-m",
                "pip",
                "install",
                "--disable-pip-version-check",
                "--no-input",
                "--upgrade",
                package,
            ],
        )?;
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = package;
        Err("Managed package installation currently supports Windows only.".to_string())
    }
}

fn runtime_python_path(app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        Ok(runtime_python_dir(app)?.join("python.exe"))
    }

    #[cfg(not(windows))]
    {
        Ok(runtime_python_dir(app)?.join("bin").join("python3"))
    }
}

fn runtime_cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("managed-runtime"))
}

fn is_busy_state(state: &str) -> bool {
    matches!(
        state,
        "preparing"
            | "downloadingPython"
            | "verifyingInstaller"
            | "installingPython"
            | "preparingPackages"
            | "installingPackages"
            | "verifying"
    )
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
    fn pinned_python_installer_matches_expected_release() {
        assert_eq!(PYTHON_VERSION, "3.12.10");
        assert!(PYTHON_INSTALLER_URL.ends_with("python-3.12.10-amd64.exe"));
        assert!(MIN_PYTHON_INSTALLER_BYTES >= 20_000_000);
        assert!(MIN_RUNTIME_FREE_SPACE_BYTES >= 10_000_000_000);
    }

    #[test]
    fn managed_runtime_packages_cover_aura_one_runtime() {
        assert!(PYTORCH_CUDA_INDEX.ends_with("/cu128"));
        assert!(PYTORCH_CPU_INDEX.ends_with("/cpu"));
        assert!(RUNTIME_PACKAGES
            .iter()
            .any(|value| value.starts_with("transformers")));
        assert!(RUNTIME_PACKAGES
            .iter()
            .any(|value| value.starts_with("accelerate")));
        assert!(RUNTIME_PACKAGES
            .iter()
            .any(|value| value.starts_with("bitsandbytes")));
    }

    #[test]
    fn busy_states_are_explicit() {
        assert!(is_busy_state("downloadingPython"));
        assert!(is_busy_state("installingPackages"));
        assert!(!is_busy_state("ready"));
        assert!(!is_busy_state("error"));
    }
}
