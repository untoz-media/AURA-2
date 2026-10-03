use futures_util::StreamExt;
use reqwest::{header::RANGE, Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs as std_fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager};
use tokio::{
    fs as async_fs,
    io::AsyncWriteExt,
    time::sleep,
};

const MODEL_EVENT: &str = "aura:model-download";
const CANCELLED: &str = "__AURA_MODEL_DOWNLOAD_CANCELLED__";
const MODEL_CONFIG_FILENAME: &str = "model-manager.json";
const INSTALL_MARKER_FILENAME: &str = "install.json";
const MODEL_HEADROOM_BYTES: u64 = 1_000_000_000;

const AURA_1_FILES: &[&str] = &[
    "LICENSE",
    "README.md",
    "config.json",
    "generation_config.json",
    "merges.txt",
    "model-00001-of-00003.safetensors",
    "model-00002-of-00003.safetensors",
    "model-00003-of-00003.safetensors",
    "model.safetensors.index.json",
    "tokenizer.json",
    "tokenizer_config.json",
    "vocab.json",
];

#[derive(Clone)]
struct ModelDefinition {
    id: &'static str,
    name: &'static str,
    subtitle: &'static str,
    description: &'static str,
    generation: &'static str,
    source_repo: Option<&'static str>,
    source_revision: Option<&'static str>,
    license: Option<&'static str>,
    estimated_size_bytes: Option<u64>,
    files: &'static [&'static str],
    availability_message: Option<&'static str>,
}

fn model_definitions() -> Vec<ModelDefinition> {
    vec![
        ModelDefinition {
            id: "aura-1",
            name: "AURA-1",
            subtitle: "Fast · Lightweight · Local",
            description: "The original AURA runtime profile, backed by Qwen3-4B-Instruct-2507.",
            generation: "1st generation",
            source_repo: Some("Qwen/Qwen3-4B-Instruct-2507"),
            source_revision: Some("main"),
            license: Some("Apache-2.0"),
            estimated_size_bytes: Some(8_060_000_000),
            files: AURA_1_FILES,
            availability_message: None,
        },
        ModelDefinition {
            id: "aura-2",
            name: "AURA-2",
            subtitle: "Personal Computer Assistant",
            description: "The next AURA model line for context, tool use and deeper computer assistance.",
            generation: "2nd generation",
            source_repo: None,
            source_revision: None,
            license: None,
            estimated_size_bytes: None,
            files: &[],
            availability_message: Some(
                "The AURA-2 checkpoint has not been defined yet. The Model Manager is ready for it.",
            ),
        },
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelInstallState {
    Unavailable,
    NotInstalled,
    Downloading,
    Paused,
    Installed,
    Failed,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub description: String,
    pub generation: String,
    pub state: ModelInstallState,
    pub download_available: bool,
    pub installed: bool,
    pub active: bool,
    pub source_repo: Option<String>,
    pub source_revision: Option<String>,
    pub license: Option<String>,
    pub estimated_size_bytes: Option<u64>,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub progress_percent: f64,
    pub bytes_per_second: Option<u64>,
    pub current_file: Option<String>,
    pub error: Option<String>,
    pub availability_message: Option<String>,
    pub install_path: Option<String>,
    pub installed_at_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalog {
    pub models: Vec<ModelStatus>,
    pub active_model_id: Option<String>,
    pub models_root: String,
    pub refreshed_at_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadProgress {
    pub model_id: String,
    pub state: ModelInstallState,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub progress_percent: f64,
    pub bytes_per_second: Option<u64>,
    pub current_file: Option<String>,
    pub error: Option<String>,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ModelManagerConfig {
    active_model_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstalledFile {
    path: String,
    size_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallMarker {
    model_id: String,
    source_repo: String,
    source_revision: String,
    installed_at_ms: u64,
    total_bytes: u64,
    files: Vec<InstalledFile>,
}

#[derive(Clone)]
struct DownloadControl {
    paused: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
}

#[derive(Clone, Default)]
pub struct ModelManager {
    controls: Arc<Mutex<HashMap<String, DownloadControl>>>,
    progress: Arc<Mutex<HashMap<String, ModelDownloadProgress>>>,
}

impl ModelManager {
    pub fn catalog(&self, app: &AppHandle) -> Result<ModelCatalog, String> {
        let config = read_config(app)?;
        let root = models_root(app)?;
        std_fs::create_dir_all(&root).map_err(|error| error.to_string())?;

        let mut models = model_definitions()
            .into_iter()
            .map(|definition| self.model_status(app, &definition, &config))
            .collect::<Result<Vec<_>, _>>()?;

        let active_is_valid = config.active_model_id.as_deref().is_some_and(|active_id| {
            models.iter().any(|model| model.id == active_id && model.installed)
        });

        let active_model_id = if active_is_valid {
            config.active_model_id.clone()
        } else {
            if config.active_model_id.is_some() {
                let mut repaired = config.clone();
                repaired.active_model_id = None;
                let _ = write_config(app, &repaired);
            }
            for model in &mut models {
                model.active = false;
            }
            None
        };

        Ok(ModelCatalog {
            models,
            active_model_id,
            models_root: root.to_string_lossy().to_string(),
            refreshed_at_ms: timestamp_ms(),
        })
    }

    pub fn start_download(&self, app: AppHandle, model_id: &str) -> Result<ModelCatalog, String> {
        let definition = definition_for(model_id)
            .ok_or_else(|| format!("Unknown model: {model_id}."))?;

        if definition.source_repo.is_none() || definition.files.is_empty() {
            return Err(
                definition
                    .availability_message
                    .unwrap_or("This model is not available for download yet.")
                    .to_string(),
            );
        }

        {
            let controls = self
                .controls
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            if controls.contains_key(model_id) {
                return Err(format!("{} already has an active download.", definition.name));
            }
        }

        let root = models_root(&app)?;
        std_fs::create_dir_all(&root).map_err(|error| error.to_string())?;

        if let Some(estimated) = definition.estimated_size_bytes {
            let available = fs2::available_space(&root)
                .map_err(|error| format!("Could not read available disk space: {error}"))?;
            let staging = staging_dir(&app, model_id)?;
            let existing = directory_size(&staging).unwrap_or(0);
            let remaining = estimated.saturating_sub(existing);

            if available < remaining.saturating_add(MODEL_HEADROOM_BYTES) {
                return Err(format!(
                    "Not enough free disk space for {}. AURA needs about {:.1} GB plus 1 GB of headroom.",
                    definition.name,
                    estimated as f64 / 1_000_000_000.0
                ));
            }
        }

        let control = DownloadControl {
            paused: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
        };

        {
            let mut controls = self
                .controls
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            controls.insert(model_id.to_string(), control.clone());
        }

        let initial_progress = ModelDownloadProgress {
            model_id: model_id.to_string(),
            state: ModelInstallState::Downloading,
            bytes_downloaded: directory_size(&staging_dir(&app, model_id)?).unwrap_or(0),
            total_bytes: definition.estimated_size_bytes,
            progress_percent: 0.0,
            bytes_per_second: None,
            current_file: None,
            error: None,
            updated_at_ms: timestamp_ms(),
        };
        self.set_progress(&app, initial_progress);

        let manager = self.clone();
        let app_for_task = app.clone();
        tauri::async_runtime::spawn(async move {
            let result = download_model(
                &app_for_task,
                &manager,
                &definition,
                &control,
            )
            .await;

            manager.finish_download(&app_for_task, &definition, result).await;
        });

        self.catalog(&app)
    }

    pub fn pause_download(&self, app: &AppHandle, model_id: &str) -> Result<ModelCatalog, String> {
        let controls = self
            .controls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let control = controls
            .get(model_id)
            .ok_or_else(|| "No active download exists for this model.".to_string())?;
        control.paused.store(true, Ordering::SeqCst);
        drop(controls);

        self.update_progress_state(app, model_id, ModelInstallState::Paused, None);
        self.catalog(app)
    }

    pub fn resume_download(&self, app: &AppHandle, model_id: &str) -> Result<ModelCatalog, String> {
        let controls = self
            .controls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let control = controls
            .get(model_id)
            .ok_or_else(|| "No paused download exists for this model.".to_string())?;
        control.paused.store(false, Ordering::SeqCst);
        drop(controls);

        self.update_progress_state(app, model_id, ModelInstallState::Downloading, None);
        self.catalog(app)
    }

    pub fn cancel_download(&self, app: &AppHandle, model_id: &str) -> Result<ModelCatalog, String> {
        let controls = self
            .controls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let control = controls
            .get(model_id)
            .ok_or_else(|| "No active download exists for this model.".to_string())?;
        control.cancelled.store(true, Ordering::SeqCst);
        control.paused.store(false, Ordering::SeqCst);
        drop(controls);

        self.catalog(app)
    }

    pub fn set_active(&self, app: &AppHandle, model_id: &str) -> Result<ModelCatalog, String> {
        let definition = definition_for(model_id)
            .ok_or_else(|| format!("Unknown model: {model_id}."))?;

        match inspect_installation(app, &definition) {
            Ok(Some(_)) => {}
            Ok(None) => {
                return Err(format!(
                    "{} must be installed before it can be selected.",
                    definition.name
                ))
            }
            Err(error) => return Err(error),
        }

        let mut config = read_config(app)?;
        config.active_model_id = Some(model_id.to_string());
        write_config(app, &config)?;
        self.catalog(app)
    }

    pub fn remove_model(&self, app: &AppHandle, model_id: &str) -> Result<ModelCatalog, String> {
        {
            let controls = self
                .controls
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if controls.contains_key(model_id) {
                return Err("Cancel the model download before removing it.".to_string());
            }
        }

        let final_dir = final_model_dir(app, model_id)?;
        let staging = staging_dir(app, model_id)?;

        if final_dir.exists() {
            std_fs::remove_dir_all(&final_dir)
                .map_err(|error| format!("Could not remove model files: {error}"))?;
        }
        if staging.exists() {
            std_fs::remove_dir_all(&staging)
                .map_err(|error| format!("Could not remove partial model files: {error}"))?;
        }

        {
            let mut progress = self
                .progress
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            progress.remove(model_id);
        }

        let mut config = read_config(app)?;
        if config.active_model_id.as_deref() == Some(model_id) {
            config.active_model_id = None;
            write_config(app, &config)?;
        }

        self.catalog(app)
    }

    fn model_status(
        &self,
        app: &AppHandle,
        definition: &ModelDefinition,
        config: &ModelManagerConfig,
    ) -> Result<ModelStatus, String> {
        let current_progress = self
            .progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(definition.id)
            .cloned();

        let install = inspect_installation(app, definition);
        let (installed, marker, install_error) = match install {
            Ok(Some(marker)) => (true, Some(marker), None),
            Ok(None) => (false, None, None),
            Err(error) => (false, None, Some(error)),
        };

        let active =
            installed && config.active_model_id.as_deref() == Some(definition.id);

        let state = if let Some(progress) = &current_progress {
            progress.state
        } else if definition.source_repo.is_none() {
            ModelInstallState::Unavailable
        } else if installed {
            ModelInstallState::Installed
        } else if install_error.is_some() {
            ModelInstallState::Failed
        } else {
            ModelInstallState::NotInstalled
        };

        let bytes_downloaded = current_progress
            .as_ref()
            .map(|progress| progress.bytes_downloaded)
            .or_else(|| marker.as_ref().map(|marker| marker.total_bytes))
            .unwrap_or_else(|| {
                staging_dir(app, definition.id)
                    .ok()
                    .and_then(|path| directory_size(&path).ok())
                    .unwrap_or(0)
            });

        let total_bytes = current_progress
            .as_ref()
            .and_then(|progress| progress.total_bytes)
            .or_else(|| marker.as_ref().map(|marker| marker.total_bytes))
            .or(definition.estimated_size_bytes);

        Ok(ModelStatus {
            id: definition.id.to_string(),
            name: definition.name.to_string(),
            subtitle: definition.subtitle.to_string(),
            description: definition.description.to_string(),
            generation: definition.generation.to_string(),
            state,
            download_available: definition.source_repo.is_some(),
            installed,
            active,
            source_repo: definition.source_repo.map(str::to_string),
            source_revision: definition.source_revision.map(str::to_string),
            license: definition.license.map(str::to_string),
            estimated_size_bytes: definition.estimated_size_bytes,
            bytes_downloaded,
            total_bytes,
            progress_percent: current_progress
                .as_ref()
                .map(|progress| progress.progress_percent)
                .or_else(|| {
                    if installed {
                        Some(100.0)
                    } else {
                        total_bytes.map(|total| percent(bytes_downloaded, total))
                    }
                })
                .unwrap_or(0.0),
            bytes_per_second: current_progress
                .as_ref()
                .and_then(|progress| progress.bytes_per_second),
            current_file: current_progress
                .as_ref()
                .and_then(|progress| progress.current_file.clone()),
            error: current_progress
                .as_ref()
                .and_then(|progress| progress.error.clone())
                .or(install_error),
            availability_message: definition.availability_message.map(str::to_string),
            install_path: if installed {
                Some(final_model_dir(app, definition.id)?.to_string_lossy().to_string())
            } else {
                None
            },
            installed_at_ms: marker.as_ref().map(|marker| marker.installed_at_ms),
        })
    }

    fn set_progress(&self, app: &AppHandle, mut progress: ModelDownloadProgress) {
        progress.updated_at_ms = timestamp_ms();

        {
            let mut map = self
                .progress
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            map.insert(progress.model_id.clone(), progress.clone());
        }

        let _ = app.emit(MODEL_EVENT, progress);
    }

    fn update_progress_state(
        &self,
        app: &AppHandle,
        model_id: &str,
        state: ModelInstallState,
        error: Option<String>,
    ) {
        let existing = {
            let map = self
                .progress
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            map.get(model_id).cloned()
        };

        if let Some(mut progress) = existing {
            progress.state = state;
            progress.error = error;
            self.set_progress(app, progress);
        }
    }

    async fn finish_download(
        &self,
        app: &AppHandle,
        definition: &ModelDefinition,
        result: Result<InstallMarker, String>,
    ) {
        {
            let mut controls = self
                .controls
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            controls.remove(definition.id);
        }

        match result {
            Ok(marker) => {
                let mut config = read_config(app).unwrap_or_default();
                if config.active_model_id.is_none() {
                    config.active_model_id = Some(definition.id.to_string());
                    let _ = write_config(app, &config);
                }

                let progress = ModelDownloadProgress {
                    model_id: definition.id.to_string(),
                    state: ModelInstallState::Installed,
                    bytes_downloaded: marker.total_bytes,
                    total_bytes: Some(marker.total_bytes),
                    progress_percent: 100.0,
                    bytes_per_second: None,
                    current_file: None,
                    error: None,
                    updated_at_ms: timestamp_ms(),
                };
                self.set_progress(app, progress);
            }
            Err(error) if error == CANCELLED => {
                if let Ok(staging) = staging_dir(app, definition.id) {
                    let _ = async_fs::remove_dir_all(staging).await;
                }

                {
                    let mut progress = self
                        .progress
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    progress.remove(definition.id);
                }

                let _ = app.emit(
                    MODEL_EVENT,
                    ModelDownloadProgress {
                        model_id: definition.id.to_string(),
                        state: ModelInstallState::NotInstalled,
                        bytes_downloaded: 0,
                        total_bytes: definition.estimated_size_bytes,
                        progress_percent: 0.0,
                        bytes_per_second: None,
                        current_file: None,
                        error: None,
                        updated_at_ms: timestamp_ms(),
                    },
                );
            }
            Err(error) => {
                self.update_progress_state(
                    app,
                    definition.id,
                    ModelInstallState::Failed,
                    Some(error),
                );
            }
        }
    }
}

fn definition_for(model_id: &str) -> Option<ModelDefinition> {
    model_definitions()
        .into_iter()
        .find(|definition| definition.id == model_id)
}

async fn download_model(
    app: &AppHandle,
    manager: &ModelManager,
    definition: &ModelDefinition,
    control: &DownloadControl,
) -> Result<InstallMarker, String> {
    let repo = definition
        .source_repo
        .ok_or_else(|| "Model source is not configured.".to_string())?;
    let revision = definition.source_revision.unwrap_or("main");
    let client = Client::builder()
        .user_agent(format!("AURA-2/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| format!("Could not initialize model downloader: {error}"))?;

    let staging = staging_dir(app, definition.id)?;
    async_fs::create_dir_all(&staging)
        .await
        .map_err(|error| format!("Could not create model staging directory: {error}"))?;

    let mut remote_sizes = HashMap::new();
    let mut exact_total = 0_u64;
    let mut all_sizes_known = true;

    for file in definition.files {
        wait_if_paused(app, manager, definition.id, control).await?;
        if control.cancelled.load(Ordering::SeqCst) {
            return Err(CANCELLED.to_string());
        }

        let url = huggingface_url(repo, revision, file);
        match remote_file_size(&client, &url).await? {
            Some(size) => {
                remote_sizes.insert((*file).to_string(), size);
                exact_total = exact_total.saturating_add(size);
            }
            None => {
                all_sizes_known = false;
            }
        }
    }

    let total_bytes = if all_sizes_known && exact_total > 0 {
        Some(exact_total)
    } else {
        definition.estimated_size_bytes
    };

    let mut downloaded = valid_existing_bytes(&staging, definition.files, &remote_sizes)?;
    let session_start_bytes = downloaded;
    let session_started = Instant::now();
    let mut installed_files = Vec::with_capacity(definition.files.len());

    manager.set_progress(
        app,
        ModelDownloadProgress {
            model_id: definition.id.to_string(),
            state: ModelInstallState::Downloading,
            bytes_downloaded: downloaded,
            total_bytes,
            progress_percent: total_bytes
                .map(|total| percent(downloaded, total))
                .unwrap_or(0.0),
            bytes_per_second: None,
            current_file: None,
            error: None,
            updated_at_ms: timestamp_ms(),
        },
    );

    for file in definition.files {
        wait_if_paused(app, manager, definition.id, control).await?;
        if control.cancelled.load(Ordering::SeqCst) {
            return Err(CANCELLED.to_string());
        }

        let expected_size = remote_sizes.get(*file).copied();
        let url = huggingface_url(repo, revision, file);
        let destination = staging.join(file);

        if let Some(parent) = destination.parent() {
            async_fs::create_dir_all(parent)
                .await
                .map_err(|error| format!("Could not create model directory: {error}"))?;
        }

        let size = download_file(
            app,
            manager,
            definition.id,
            &client,
            &url,
            file,
            &destination,
            expected_size,
            control,
            &mut downloaded,
            total_bytes,
            session_start_bytes,
            session_started,
        )
        .await?;

        installed_files.push(InstalledFile {
            path: (*file).to_string(),
            size_bytes: size,
        });
    }

    let verified_total = verify_staging(&staging, &installed_files)?;

    let marker = InstallMarker {
        model_id: definition.id.to_string(),
        source_repo: repo.to_string(),
        source_revision: revision.to_string(),
        installed_at_ms: timestamp_ms(),
        total_bytes: verified_total,
        files: installed_files,
    };

    let marker_content =
        serde_json::to_string_pretty(&marker).map_err(|error| error.to_string())?;
    async_fs::write(staging.join(INSTALL_MARKER_FILENAME), marker_content)
        .await
        .map_err(|error| format!("Could not write model installation marker: {error}"))?;

    let final_dir = final_model_dir(app, definition.id)?;
    if final_dir.exists() {
        async_fs::remove_dir_all(&final_dir)
            .await
            .map_err(|error| format!("Could not replace previous model installation: {error}"))?;
    }

    async_fs::rename(&staging, &final_dir)
        .await
        .map_err(|error| format!("Could not finalize model installation: {error}"))?;

    Ok(marker)
}

#[allow(clippy::too_many_arguments)]
async fn download_file(
    app: &AppHandle,
    manager: &ModelManager,
    model_id: &str,
    client: &Client,
    url: &str,
    display_name: &str,
    destination: &Path,
    expected_size: Option<u64>,
    control: &DownloadControl,
    downloaded: &mut u64,
    total_bytes: Option<u64>,
    session_start_bytes: u64,
    session_started: Instant,
) -> Result<u64, String> {
    let mut existing = async_fs::metadata(destination)
        .await
        .map(|metadata| metadata.len())
        .unwrap_or(0);

    if let Some(expected) = expected_size {
        if existing == expected {
            return Ok(existing);
        }

        if existing > expected {
            async_fs::remove_file(destination)
                .await
                .map_err(|error| format!("Could not reset invalid partial file: {error}"))?;
            *downloaded = downloaded.saturating_sub(existing);
            existing = 0;
        }
    }

    let mut request = client.get(url);
    if existing > 0 {
        request = request.header(RANGE, format!("bytes={existing}-"));
    }

    let response = request
        .send()
        .await
        .map_err(|error| format!("Could not download {display_name}: {error}"))?;

    if response.status() == StatusCode::RANGE_NOT_SATISFIABLE {
        if let Some(expected) = expected_size {
            if existing == expected {
                return Ok(existing);
            }
        }
        return Err(format!(
            "The model server rejected the resume position for {display_name}."
        ));
    }

    let status = response.status();
    let response = response
        .error_for_status()
        .map_err(|error| format!("Could not download {display_name}: {error}"))?;

    let append = existing > 0 && status == StatusCode::PARTIAL_CONTENT;
    if existing > 0 && !append {
        *downloaded = downloaded.saturating_sub(existing);
        existing = 0;
    }

    let mut options = async_fs::OpenOptions::new();
    options.create(true).write(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
    }

    let mut output = options
        .open(destination)
        .await
        .map_err(|error| format!("Could not write {display_name}: {error}"))?;

    let mut stream = response.bytes_stream();
    let mut file_size = existing;
    let mut last_emit = Instant::now()
        .checked_sub(Duration::from_secs(1))
        .unwrap_or_else(Instant::now);

    while let Some(chunk) = stream.next().await {
        wait_if_paused(app, manager, model_id, control).await?;
        if control.cancelled.load(Ordering::SeqCst) {
            return Err(CANCELLED.to_string());
        }

        let chunk =
            chunk.map_err(|error| format!("Download interrupted for {display_name}: {error}"))?;
        output
            .write_all(&chunk)
            .await
            .map_err(|error| format!("Could not write {display_name}: {error}"))?;

        let chunk_len = chunk.len() as u64;
        file_size = file_size.saturating_add(chunk_len);
        *downloaded = downloaded.saturating_add(chunk_len);

        if last_emit.elapsed() >= Duration::from_millis(250) {
            emit_download_progress(
                app,
                manager,
                model_id,
                ModelInstallState::Downloading,
                *downloaded,
                total_bytes,
                Some(display_name.to_string()),
                None,
                session_start_bytes,
                session_started,
            );
            last_emit = Instant::now();
        }
    }

    output
        .flush()
        .await
        .map_err(|error| format!("Could not finalize {display_name}: {error}"))?;

    if let Some(expected) = expected_size {
        if file_size != expected {
            return Err(format!(
                "Verification failed for {display_name}: expected {expected} bytes, got {file_size}."
            ));
        }
    } else if file_size == 0 {
        return Err(format!("Verification failed for {display_name}: file is empty."));
    }

    emit_download_progress(
        app,
        manager,
        model_id,
        ModelInstallState::Downloading,
        *downloaded,
        total_bytes,
        Some(display_name.to_string()),
        None,
        session_start_bytes,
        session_started,
    );

    Ok(file_size)
}

async fn wait_if_paused(
    app: &AppHandle,
    manager: &ModelManager,
    model_id: &str,
    control: &DownloadControl,
) -> Result<(), String> {
    let mut emitted_pause = false;

    while control.paused.load(Ordering::SeqCst) {
        if control.cancelled.load(Ordering::SeqCst) {
            return Err(CANCELLED.to_string());
        }

        if !emitted_pause {
            manager.update_progress_state(app, model_id, ModelInstallState::Paused, None);
            emitted_pause = true;
        }

        sleep(Duration::from_millis(200)).await;
    }

    if emitted_pause {
        manager.update_progress_state(app, model_id, ModelInstallState::Downloading, None);
    }

    Ok(())
}

fn emit_download_progress(
    app: &AppHandle,
    manager: &ModelManager,
    model_id: &str,
    state: ModelInstallState,
    bytes_downloaded: u64,
    total_bytes: Option<u64>,
    current_file: Option<String>,
    error: Option<String>,
    session_start_bytes: u64,
    session_started: Instant,
) {
    let elapsed = session_started.elapsed().as_secs_f64();
    let session_bytes = bytes_downloaded.saturating_sub(session_start_bytes);
    let bytes_per_second = if elapsed >= 0.25 {
        Some((session_bytes as f64 / elapsed) as u64)
    } else {
        None
    };

    manager.set_progress(
        app,
        ModelDownloadProgress {
            model_id: model_id.to_string(),
            state,
            bytes_downloaded,
            total_bytes,
            progress_percent: total_bytes
                .map(|total| percent(bytes_downloaded, total))
                .unwrap_or(0.0),
            bytes_per_second,
            current_file,
            error,
            updated_at_ms: timestamp_ms(),
        },
    );
}

async fn remote_file_size(client: &Client, url: &str) -> Result<Option<u64>, String> {
    let response = client
        .head(url)
        .send()
        .await
        .map_err(|error| format!("Could not inspect model file: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Could not inspect model file: {error}"))?;

    Ok(response.content_length())
}

fn inspect_installation(
    app: &AppHandle,
    definition: &ModelDefinition,
) -> Result<Option<InstallMarker>, String> {
    let directory = final_model_dir(app, definition.id)?;
    if !directory.exists() {
        return Ok(None);
    }

    let marker_path = directory.join(INSTALL_MARKER_FILENAME);
    let content = std_fs::read_to_string(&marker_path).map_err(|error| {
        format!(
            "{} installation exists but its verification marker is missing or unreadable: {error}",
            definition.name
        )
    })?;

    let marker: InstallMarker = serde_json::from_str(&content).map_err(|error| {
        format!(
            "{} installation marker is invalid: {error}",
            definition.name
        )
    })?;

    if marker.model_id != definition.id {
        return Err(format!(
            "{} installation marker belongs to another model.",
            definition.name
        ));
    }

    if definition.source_repo != Some(marker.source_repo.as_str()) {
        return Err(format!(
            "{} installation source does not match the current model definition.",
            definition.name
        ));
    }

    if definition.source_revision.unwrap_or("main") != marker.source_revision {
        return Err(format!(
            "{} installation revision does not match the current model definition.",
            definition.name
        ));
    }

    for file in &marker.files {
        let path = directory.join(&file.path);
        let actual = std_fs::metadata(&path)
            .map_err(|_| format!("{} installation is missing {}.", definition.name, file.path))?
            .len();

        if actual != file.size_bytes {
            return Err(format!(
                "{} installation failed verification for {}.",
                definition.name, file.path
            ));
        }
    }

    Ok(Some(marker))
}

fn verify_staging(directory: &Path, files: &[InstalledFile]) -> Result<u64, String> {
    let mut total = 0_u64;

    for file in files {
        let path = directory.join(&file.path);
        let size = std_fs::metadata(&path)
            .map_err(|error| format!("Could not verify {}: {error}", file.path))?
            .len();

        if size == 0 || size != file.size_bytes {
            return Err(format!("Verification failed for {}.", file.path));
        }

        total = total.saturating_add(size);
    }

    Ok(total)
}

fn valid_existing_bytes(
    staging: &Path,
    files: &[&str],
    remote_sizes: &HashMap<String, u64>,
) -> Result<u64, String> {
    let mut total = 0_u64;

    for file in files {
        let path = staging.join(file);
        let Ok(metadata) = std_fs::metadata(&path) else {
            continue;
        };
        let size = metadata.len();

        if let Some(expected) = remote_sizes.get(*file) {
            if size > *expected {
                std_fs::remove_file(&path)
                    .map_err(|error| format!("Could not reset invalid partial file: {error}"))?;
                continue;
            }
        }

        total = total.saturating_add(size);
    }

    Ok(total)
}

fn read_config(app: &AppHandle) -> Result<ModelManagerConfig, String> {
    let path = model_config_path(app)?;

    if !path.exists() {
        return Ok(ModelManagerConfig::default());
    }

    let content =
        std_fs::read_to_string(path).map_err(|error| format!("Could not read Model Manager config: {error}"))?;

    serde_json::from_str(&content)
        .map_err(|error| format!("Model Manager config is invalid: {error}"))
}

fn write_config(app: &AppHandle, config: &ModelManagerConfig) -> Result<(), String> {
    let path = model_config_path(app)?;

    if let Some(parent) = path.parent() {
        std_fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let content = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
    std_fs::write(path, content).map_err(|error| format!("Could not save Model Manager config: {error}"))
}

fn model_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join(MODEL_CONFIG_FILENAME))
}

fn models_root(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_local_data_dir()
        .map_err(|error| error.to_string())?
        .join("models"))
}

fn final_model_dir(app: &AppHandle, model_id: &str) -> Result<PathBuf, String> {
    Ok(models_root(app)?.join(model_id))
}

fn staging_dir(app: &AppHandle, model_id: &str) -> Result<PathBuf, String> {
    Ok(models_root(app)?.join(format!("{model_id}.partial")))
}

fn huggingface_url(repo: &str, revision: &str, file: &str) -> String {
    format!("https://huggingface.co/{repo}/resolve/{revision}/{file}")
}

fn directory_size(path: &Path) -> Result<u64, String> {
    if !path.exists() {
        return Ok(0);
    }

    let mut total = 0_u64;
    for entry in std_fs::read_dir(path).map_err(|error| error.to_string())? {
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

fn percent(downloaded: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }

    ((downloaded as f64 / total as f64) * 100.0).clamp(0.0, 100.0)
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
    fn catalog_keeps_aura_2_unavailable_until_checkpoint_exists() {
        let aura_2 = definition_for("aura-2").unwrap();
        assert!(aura_2.source_repo.is_none());
        assert!(aura_2.files.is_empty());
    }

    #[test]
    fn aura_1_uses_existing_upstream_runtime_model() {
        let aura_1 = definition_for("aura-1").unwrap();
        assert_eq!(
            aura_1.source_repo,
            Some("Qwen/Qwen3-4B-Instruct-2507")
        );
        assert_eq!(aura_1.license, Some("Apache-2.0"));
        assert_eq!(aura_1.files.len(), 12);
    }

    #[test]
    fn progress_percentage_is_clamped() {
        assert_eq!(percent(0, 100), 0.0);
        assert_eq!(percent(50, 100), 50.0);
        assert_eq!(percent(150, 100), 100.0);
    }

    #[test]
    fn huggingface_urls_are_deterministic() {
        assert_eq!(
            huggingface_url("Qwen/Test", "main", "config.json"),
            "https://huggingface.co/Qwen/Test/resolve/main/config.json"
        );
    }
}
