use obws::{requests::{inputs::Volume, scene_items::SetEnabled}, Client};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

const DEFAULT_OBS_HOST: &str = "127.0.0.1";
const DEFAULT_OBS_PORT: u16 = 4455;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsConnectRequest {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub password: Option<String>,
}


#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsSceneSwitchRequest {
    pub scene_uuid: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsSceneSwitchResult {
    pub target: String,
    pub scene_name: String,
    pub scene_uuid: String,
    pub changed_at_ms: u64,
}


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsRecordingActionResult {
    pub action: String,
    pub recording: bool,
    pub paused: bool,
    pub output_path: Option<String>,
    pub changed_at_ms: u64,
}


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsStreamingActionResult {
    pub action: String,
    pub streaming: bool,
    pub changed_at_ms: u64,
}


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsStreamDuration {
    pub streaming: bool,
    pub duration_ms: u64,
    pub timecode: String,
    pub refreshed_at_ms: u64,
}


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsSourceItemSummary {
    pub scene_name: String,
    pub item_id: i64,
    pub index: u32,
    pub source_name: String,
    pub enabled: bool,
    pub input_kind: Option<String>,
    pub is_group: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsSourceItemList {
    pub scene_name: String,
    pub items: Vec<ObsSourceItemSummary>,
    pub refreshed_at_ms: u64,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsSourceVisibilityRequest {
    pub scene_name: String,
    pub item_id: i64,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsSourceVisibilityResult {
    pub scene_name: String,
    pub item_id: i64,
    pub source_name: String,
    pub enabled: bool,
    pub changed_at_ms: u64,
}


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsAudioInputSummary {
    pub input_name: String,
    pub input_uuid: String,
    pub input_kind: String,
    pub muted: bool,
    pub volume_percent: u8,
    pub volume_mul: f32,
    pub volume_db: f32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsAudioInputList {
    pub inputs: Vec<ObsAudioInputSummary>,
    pub refreshed_at_ms: u64,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsAudioMuteRequest {
    pub input_uuid: String,
    pub muted: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsAudioVolumeRequest {
    pub input_uuid: String,
    pub percent: u8,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsAudioControlResult {
    pub input_name: String,
    pub input_uuid: String,
    pub muted: bool,
    pub volume_percent: u8,
    pub volume_mul: f32,
    pub volume_db: f32,
    pub changed_at_ms: u64,
}


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsProductionHealth {
    pub status: String,
    pub summary: String,
    pub issues: Vec<String>,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub available_disk_space_mb: f64,
    pub active_fps: f64,
    pub average_frame_render_time_ms: f64,
    pub render_skipped_frames: u32,
    pub render_total_frames: u32,
    pub render_skipped_percent: f64,
    pub output_skipped_frames: u32,
    pub output_total_frames: u32,
    pub output_skipped_percent: f64,
    pub streaming: bool,
    pub stream_reconnecting: bool,
    pub stream_congestion_percent: Option<f64>,
    pub stream_bitrate_kbps: Option<f64>,
    pub stream_output_skipped_frames: Option<u64>,
    pub stream_output_total_frames: Option<u64>,
    pub checked_at_ms: u64,
}

#[derive(Clone, Copy, Debug)]
struct ObsHealthSample {
    output_bytes: u64,
    sampled_at_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsConnectionState {
    pub connected: bool,
    pub host: String,
    pub port: u16,
    pub obs_studio_version: Option<String>,
    pub obs_websocket_version: Option<String>,
    pub rpc_version: Option<u32>,
    pub connected_at_ms: Option<u64>,
    pub last_error: Option<String>,
}


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsRuntimeState {
    pub available: bool,
    pub streaming: bool,
    pub recording: bool,
    pub recording_paused: bool,
    pub studio_mode: bool,
    pub current_program_scene: Option<String>,
    pub current_preview_scene: Option<String>,
    pub stream_duration_ms: u64,
    pub stream_timecode: String,
    pub refreshed_at_ms: u64,
    pub last_error: Option<String>,
}

impl Default for ObsRuntimeState {
    fn default() -> Self {
        Self {
            available: false,
            streaming: false,
            recording: false,
            recording_paused: false,
            studio_mode: false,
            current_program_scene: None,
            current_preview_scene: None,
            stream_duration_ms: 0,
            stream_timecode: "00:00:00.000".to_string(),
            refreshed_at_ms: timestamp_ms(),
            last_error: None,
        }
    }
}


#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsSceneSummary {
    pub name: String,
    pub uuid: String,
    pub index: usize,
    pub is_program: bool,
    pub is_preview: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObsSceneList {
    pub scenes: Vec<ObsSceneSummary>,
    pub current_program_scene: Option<String>,
    pub current_preview_scene: Option<String>,
    pub refreshed_at_ms: u64,
    pub last_error: Option<String>,
}

impl Default for ObsSceneList {
    fn default() -> Self {
        Self {
            scenes: Vec::new(),
            current_program_scene: None,
            current_preview_scene: None,
            refreshed_at_ms: timestamp_ms(),
            last_error: None,
        }
    }
}

impl Default for ObsConnectionState {
    fn default() -> Self {
        Self {
            connected: false,
            host: DEFAULT_OBS_HOST.to_string(),
            port: DEFAULT_OBS_PORT,
            obs_studio_version: None,
            obs_websocket_version: None,
            rpc_version: None,
            connected_at_ms: None,
            last_error: None,
        }
    }
}

pub struct ObsController {
    client: Mutex<Option<Client>>,
    state: Mutex<ObsConnectionState>,
    health_sample: Mutex<Option<ObsHealthSample>>,
}

impl Default for ObsController {
    fn default() -> Self {
        Self {
            client: Mutex::new(None),
            state: Mutex::new(ObsConnectionState::default()),
            health_sample: Mutex::new(None),
        }
    }
}

impl ObsController {
    pub async fn snapshot(&self) -> ObsConnectionState {
        self.state.lock().await.clone()
    }


    pub async fn runtime_state(&self) -> ObsRuntimeState {
        let client_guard = self.client.lock().await;
        let Some(client) = client_guard.as_ref() else {
            return ObsRuntimeState {
                last_error: Some("OBS Studio is not connected.".to_string()),
                ..ObsRuntimeState::default()
            };
        };

        let studio_mode = match client.ui().studio_mode_enabled().await {
            Ok(enabled) => enabled,
            Err(error) => {
                return self
                    .runtime_failure(format!("Could not read OBS Studio Mode: {error}"))
                    .await;
            }
        };

        let stream_status = match client.streaming().status().await {
            Ok(status) => status,
            Err(error) => {
                return self
                    .runtime_failure(format!("Could not read OBS streaming state: {error}"))
                    .await;
            }
        };

        let record_status = match client.recording().status().await {
            Ok(status) => status,
            Err(error) => {
                return self
                    .runtime_failure(format!("Could not read OBS recording state: {error}"))
                    .await;
            }
        };

        let program_scene = match client.scenes().current_program_scene().await {
            Ok(scene) => Some(scene.id.name),
            Err(error) => {
                return self
                    .runtime_failure(format!("Could not read the current OBS program scene: {error}"))
                    .await;
            }
        };

        let preview_scene = if studio_mode {
            client
                .scenes()
                .current_preview_scene()
                .await
                .ok()
                .map(|scene| scene.id.name)
        } else {
            None
        };

        let stream_json = serde_json::to_value(stream_status).unwrap_or_default();
        let record_json = serde_json::to_value(record_status).unwrap_or_default();

        {
            let mut connection = self.state.lock().await;
            connection.connected = true;
            connection.last_error = None;
        }

        ObsRuntimeState {
            available: true,
            streaming: json_bool(&stream_json, "outputActive"),
            recording: json_bool(&record_json, "outputActive"),
            recording_paused: json_bool(&record_json, "outputPaused"),
            studio_mode,
            current_program_scene: program_scene,
            current_preview_scene: preview_scene,
            stream_duration_ms: json_u64(&stream_json, "outputDuration").unwrap_or(0),
            stream_timecode: json_string(&stream_json, "outputTimecode")
                .unwrap_or_else(|| "00:00:00.000".to_string()),
            refreshed_at_ms: timestamp_ms(),
            last_error: None,
        }
    }

    async fn runtime_failure(&self, message: String) -> ObsRuntimeState {
        {
            let mut connection = self.state.lock().await;
            connection.connected = false;
            connection.last_error = Some(message.clone());
        }

        ObsRuntimeState {
            last_error: Some(message),
            ..ObsRuntimeState::default()
        }
    }


    pub async fn scene_list(&self) -> ObsSceneList {
        let client_guard = self.client.lock().await;
        let Some(client) = client_guard.as_ref() else {
            return ObsSceneList {
                last_error: Some("OBS Studio is not connected.".to_string()),
                ..ObsSceneList::default()
            };
        };

        let response = match client.scenes().list().await {
            Ok(response) => response,
            Err(error) => {
                return ObsSceneList {
                    last_error: Some(format!("Could not list OBS scenes: {error}")),
                    ..ObsSceneList::default()
                };
            }
        };

        let program_name = response
            .current_program_scene
            .as_ref()
            .map(|scene| scene.name.clone());
        let preview_name = response
            .current_preview_scene
            .as_ref()
            .map(|scene| scene.name.clone());
        let program_uuid = response
            .current_program_scene
            .as_ref()
            .map(|scene| scene.uuid.to_string());
        let preview_uuid = response
            .current_preview_scene
            .as_ref()
            .map(|scene| scene.uuid.to_string());

        let mut scenes = response
            .scenes
            .into_iter()
            .map(|scene| {
                let uuid = scene.id.uuid.to_string();
                ObsSceneSummary {
                    name: scene.id.name,
                    is_program: program_uuid.as_deref() == Some(uuid.as_str()),
                    is_preview: preview_uuid.as_deref() == Some(uuid.as_str()),
                    uuid,
                    index: scene.index,
                }
            })
            .collect::<Vec<_>>();

        scenes.sort_by_key(|scene| scene.index);

        ObsSceneList {
            scenes,
            current_program_scene: program_name,
            current_preview_scene: preview_name,
            refreshed_at_ms: timestamp_ms(),
            last_error: None,
        }
    }


    pub async fn set_program_scene(
        &self,
        request: ObsSceneSwitchRequest,
    ) -> Result<ObsSceneSwitchResult, String> {
        let scene_uuid = validate_scene_uuid(&request.scene_uuid)?;
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let scene_list = client
            .scenes()
            .list()
            .await
            .map_err(|error| format!("Could not validate OBS scenes before switching: {error}"))?;

        let scene = scene_list
            .scenes
            .into_iter()
            .find(|scene| scene.id.uuid.to_string() == scene_uuid)
            .ok_or_else(|| "The requested OBS scene no longer exists.".to_string())?;

        let scene_id = scene.id.clone();
        let scene_name = scene_id.name.clone();
        let scene_uuid = scene_id.uuid.to_string();

        client
            .scenes()
            .set_current_program_scene(&scene_id)
            .await
            .map_err(|error| format!("Could not switch the OBS Program scene: {error}"))?;

        let current = client
            .scenes()
            .current_program_scene()
            .await
            .map_err(|error| format!("OBS switched scenes, but verification failed: {error}"))?;

        if current.id.uuid != scene_id.uuid {
            return Err("OBS did not confirm the requested Program scene.".to_string());
        }

        Ok(ObsSceneSwitchResult {
            target: "program".to_string(),
            scene_name,
            scene_uuid,
            changed_at_ms: timestamp_ms(),
        })
    }

    pub async fn set_preview_scene(
        &self,
        request: ObsSceneSwitchRequest,
    ) -> Result<ObsSceneSwitchResult, String> {
        let scene_uuid = validate_scene_uuid(&request.scene_uuid)?;
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let studio_mode = client
            .ui()
            .studio_mode_enabled()
            .await
            .map_err(|error| format!("Could not read OBS Studio Mode: {error}"))?;

        if !studio_mode {
            return Err(
                "Preview scenes are only available while OBS Studio Mode is enabled.".to_string(),
            );
        }

        let scene_list = client
            .scenes()
            .list()
            .await
            .map_err(|error| format!("Could not validate OBS scenes before switching: {error}"))?;

        let scene = scene_list
            .scenes
            .into_iter()
            .find(|scene| scene.id.uuid.to_string() == scene_uuid)
            .ok_or_else(|| "The requested OBS scene no longer exists.".to_string())?;

        let scene_id = scene.id.clone();
        let scene_name = scene_id.name.clone();
        let scene_uuid = scene_id.uuid.to_string();

        client
            .scenes()
            .set_current_preview_scene(&scene_id)
            .await
            .map_err(|error| format!("Could not switch the OBS Preview scene: {error}"))?;

        let current = client
            .scenes()
            .current_preview_scene()
            .await
            .map_err(|error| format!("OBS changed Preview, but verification failed: {error}"))?;

        if current.id.uuid != scene_id.uuid {
            return Err("OBS did not confirm the requested Preview scene.".to_string());
        }

        Ok(ObsSceneSwitchResult {
            target: "preview".to_string(),
            scene_name,
            scene_uuid,
            changed_at_ms: timestamp_ms(),
        })
    }


    pub async fn set_program_scene_by_name(
        &self,
        scene_name: &str,
    ) -> Result<ObsSceneSwitchResult, String> {
        let requested = validate_scene_name(scene_name)?;
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let scene_list = client
            .scenes()
            .list()
            .await
            .map_err(|error| format!("Could not read OBS scenes: {error}"))?;

        let scene = scene_list
            .scenes
            .into_iter()
            .find(|scene| scene.id.name.eq_ignore_ascii_case(&requested))
            .ok_or_else(|| format!("No OBS scene named “{requested}” exists."))?;

        let scene_id = scene.id.clone();
        let actual_name = scene_id.name.clone();
        let scene_uuid = scene_id.uuid.to_string();

        client
            .scenes()
            .set_current_program_scene(&scene_id)
            .await
            .map_err(|error| format!("Could not switch the OBS Program scene: {error}"))?;

        let current = client
            .scenes()
            .current_program_scene()
            .await
            .map_err(|error| format!("OBS switched scenes, but verification failed: {error}"))?;

        if current.id.uuid != scene_id.uuid {
            return Err("OBS did not confirm the requested Program scene.".to_string());
        }

        Ok(ObsSceneSwitchResult {
            target: "program".to_string(),
            scene_name: actual_name,
            scene_uuid,
            changed_at_ms: timestamp_ms(),
        })
    }

    pub async fn set_preview_scene_by_name(
        &self,
        scene_name: &str,
    ) -> Result<ObsSceneSwitchResult, String> {
        let requested = validate_scene_name(scene_name)?;
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let studio_mode = client
            .ui()
            .studio_mode_enabled()
            .await
            .map_err(|error| format!("Could not read OBS Studio Mode: {error}"))?;

        if !studio_mode {
            return Err(
                "Preview scenes are only available while OBS Studio Mode is enabled.".to_string(),
            );
        }

        let scene_list = client
            .scenes()
            .list()
            .await
            .map_err(|error| format!("Could not read OBS scenes: {error}"))?;

        let scene = scene_list
            .scenes
            .into_iter()
            .find(|scene| scene.id.name.eq_ignore_ascii_case(&requested))
            .ok_or_else(|| format!("No OBS scene named “{requested}” exists."))?;

        let scene_id = scene.id.clone();
        let actual_name = scene_id.name.clone();
        let scene_uuid = scene_id.uuid.to_string();

        client
            .scenes()
            .set_current_preview_scene(&scene_id)
            .await
            .map_err(|error| format!("Could not switch the OBS Preview scene: {error}"))?;

        let current = client
            .scenes()
            .current_preview_scene()
            .await
            .map_err(|error| format!("OBS changed Preview, but verification failed: {error}"))?;

        if current.id.uuid != scene_id.uuid {
            return Err("OBS did not confirm the requested Preview scene.".to_string());
        }

        Ok(ObsSceneSwitchResult {
            target: "preview".to_string(),
            scene_name: actual_name,
            scene_uuid,
            changed_at_ms: timestamp_ms(),
        })
    }

    pub async fn start_recording(&self) -> Result<ObsRecordingActionResult, String> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let (active, _) = read_recording_flags(client).await?;
        if active {
            return Err("OBS is already recording.".to_string());
        }

        client
            .recording()
            .start()
            .await
            .map_err(|error| format!("Could not start OBS recording: {error}"))?;

        let (recording, paused) = wait_for_recording_state(client, true, false).await?;

        Ok(ObsRecordingActionResult {
            action: "start".to_string(),
            recording,
            paused,
            output_path: None,
            changed_at_ms: timestamp_ms(),
        })
    }

    pub async fn stop_recording(&self) -> Result<ObsRecordingActionResult, String> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let (active, _) = read_recording_flags(client).await?;
        if !active {
            return Err("OBS is not currently recording.".to_string());
        }

        let output_path = client
            .recording()
            .stop()
            .await
            .map_err(|error| format!("Could not stop OBS recording: {error}"))?;

        let (recording, paused) = wait_for_recording_state(client, false, false).await?;

        Ok(ObsRecordingActionResult {
            action: "stop".to_string(),
            recording,
            paused,
            output_path: Some(output_path),
            changed_at_ms: timestamp_ms(),
        })
    }

    pub async fn pause_recording(&self) -> Result<ObsRecordingActionResult, String> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let (active, paused) = read_recording_flags(client).await?;
        if !active {
            return Err("OBS recording is not running.".to_string());
        }
        if paused {
            return Err("OBS recording is already paused.".to_string());
        }

        client
            .recording()
            .pause()
            .await
            .map_err(|error| format!("Could not pause OBS recording: {error}"))?;

        let (recording, paused) = wait_for_recording_state(client, true, true).await?;

        Ok(ObsRecordingActionResult {
            action: "pause".to_string(),
            recording,
            paused,
            output_path: None,
            changed_at_ms: timestamp_ms(),
        })
    }

    pub async fn resume_recording(&self) -> Result<ObsRecordingActionResult, String> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let (active, paused) = read_recording_flags(client).await?;
        if !active {
            return Err("OBS recording is not running.".to_string());
        }
        if !paused {
            return Err("OBS recording is not paused.".to_string());
        }

        client
            .recording()
            .resume()
            .await
            .map_err(|error| format!("Could not resume OBS recording: {error}"))?;

        let (recording, paused) = wait_for_recording_state(client, true, false).await?;

        Ok(ObsRecordingActionResult {
            action: "resume".to_string(),
            recording,
            paused,
            output_path: None,
            changed_at_ms: timestamp_ms(),
        })
    }

    pub async fn stream_duration(&self) -> Result<ObsStreamDuration, String> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let status = client
            .streaming()
            .status()
            .await
            .map_err(|error| format!("Could not read OBS stream duration: {error}"))?;
        let value = serde_json::to_value(status)
            .map_err(|error| format!("Could not decode OBS stream duration: {error}"))?;

        Ok(ObsStreamDuration {
            streaming: json_bool(&value, "outputActive"),
            duration_ms: json_u64(&value, "outputDuration").unwrap_or(0),
            timecode: json_string(&value, "outputTimecode")
                .unwrap_or_else(|| "00:00:00.000".to_string()),
            refreshed_at_ms: timestamp_ms(),
        })
    }

    pub async fn current_program_source_items(&self) -> Result<ObsSourceItemList, String> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let current = client
            .scenes()
            .current_program_scene()
            .await
            .map_err(|error| format!("Could not read the current OBS Program scene: {error}"))?;

        source_items_for_scene(client, current.id.name.as_str()).await
    }

    pub async fn set_source_visibility(
        &self,
        request: ObsSourceVisibilityRequest,
    ) -> Result<ObsSourceVisibilityResult, String> {
        let scene_name = validate_scene_name(&request.scene_name)?;
        if request.item_id < 0 {
            return Err("OBS scene item ID cannot be negative.".to_string());
        }

        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let current = client
            .scenes()
            .current_program_scene()
            .await
            .map_err(|error| format!("Could not validate the current OBS Program scene: {error}"))?;

        if !current.id.name.eq_ignore_ascii_case(&scene_name) {
            return Err(format!(
                "Program scene changed to “{}”. Refresh sources before changing visibility.",
                current.id.name
            ));
        }

        let items = client
            .scene_items()
            .list(scene_name.as_str().into())
            .await
            .map_err(|error| format!("Could not validate OBS sources in {scene_name}: {error}"))?;

        let item = items
            .into_iter()
            .find(|item| item.id == request.item_id)
            .ok_or_else(|| "The requested OBS source item no longer exists in that scene.".to_string())?;

        client
            .scene_items()
            .set_enabled(SetEnabled {
                scene: scene_name.as_str().into(),
                item_id: item.id,
                enabled: request.enabled,
            })
            .await
            .map_err(|error| format!("Could not change OBS source visibility: {error}"))?;

        let enabled = client
            .scene_items()
            .enabled(scene_name.as_str().into(), item.id)
            .await
            .map_err(|error| format!("OBS changed source visibility, but verification failed: {error}"))?;

        if enabled != request.enabled {
            return Err("OBS did not confirm the requested source visibility state.".to_string());
        }

        Ok(ObsSourceVisibilityResult {
            scene_name,
            item_id: item.id,
            source_name: item.source_name,
            enabled,
            changed_at_ms: timestamp_ms(),
        })
    }

    pub async fn set_source_visibility_by_name(
        &self,
        source_name: &str,
        enabled: bool,
    ) -> Result<ObsSourceVisibilityResult, String> {
        let requested = validate_source_name(source_name)?;
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let current = client
            .scenes()
            .current_program_scene()
            .await
            .map_err(|error| format!("Could not read the current OBS Program scene: {error}"))?;
        let scene_name = current.id.name;

        let items = client
            .scene_items()
            .list(scene_name.as_str().into())
            .await
            .map_err(|error| format!("Could not read OBS sources in {scene_name}: {error}"))?;

        let mut matches = items
            .into_iter()
            .filter(|item| item.source_name.eq_ignore_ascii_case(&requested));

        let item = matches
            .next()
            .ok_or_else(|| format!("No OBS source named “{requested}” exists in Program scene “{scene_name}”."))?;

        if matches.next().is_some() {
            return Err(format!(
                "More than one OBS scene item named “{requested}” exists in Program scene “{scene_name}”. Use the source controls in Settings."
            ));
        }

        client
            .scene_items()
            .set_enabled(SetEnabled {
                scene: scene_name.as_str().into(),
                item_id: item.id,
                enabled,
            })
            .await
            .map_err(|error| format!("Could not change OBS source visibility: {error}"))?;

        let confirmed = client
            .scene_items()
            .enabled(scene_name.as_str().into(), item.id)
            .await
            .map_err(|error| format!("OBS changed source visibility, but verification failed: {error}"))?;

        if confirmed != enabled {
            return Err("OBS did not confirm the requested source visibility state.".to_string());
        }

        Ok(ObsSourceVisibilityResult {
            scene_name,
            item_id: item.id,
            source_name: item.source_name,
            enabled: confirmed,
            changed_at_ms: timestamp_ms(),
        })
    }

    pub async fn audio_inputs(&self) -> Result<ObsAudioInputList, String> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        audio_inputs_snapshot(client).await
    }

    pub async fn set_audio_muted(
        &self,
        request: ObsAudioMuteRequest,
    ) -> Result<ObsAudioControlResult, String> {
        let input_uuid = validate_input_uuid(&request.input_uuid)?;
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let input = find_input_by_uuid(client, &input_uuid).await?;
        let input_id = (&input.id).into();

        client
            .inputs()
            .set_muted(input_id, request.muted)
            .await
            .map_err(|error| format!("Could not change OBS input mute state: {error}"))?;

        let confirmed = client
            .inputs()
            .muted((&input.id).into())
            .await
            .map_err(|error| format!("OBS changed mute state, but verification failed: {error}"))?;

        if confirmed != request.muted {
            return Err("OBS did not confirm the requested input mute state.".to_string());
        }

        audio_control_result(client, input, confirmed).await
    }

    pub async fn set_audio_volume(
        &self,
        request: ObsAudioVolumeRequest,
    ) -> Result<ObsAudioControlResult, String> {
        let input_uuid = validate_input_uuid(&request.input_uuid)?;
        let percent = validate_audio_percent(request.percent)?;
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let input = find_input_by_uuid(client, &input_uuid).await?;
        let requested_mul = f32::from(percent) / 100.0;

        client
            .inputs()
            .set_volume((&input.id).into(), Volume::Mul(requested_mul))
            .await
            .map_err(|error| format!("Could not change OBS input volume: {error}"))?;

        let volume = client
            .inputs()
            .volume((&input.id).into())
            .await
            .map_err(|error| format!("OBS changed input volume, but verification failed: {error}"))?;

        if (volume.mul - requested_mul).abs() > 0.02 {
            return Err("OBS did not confirm the requested input volume.".to_string());
        }

        let muted = client
            .inputs()
            .muted((&input.id).into())
            .await
            .map_err(|error| format!("Could not refresh OBS input mute state: {error}"))?;

        Ok(ObsAudioControlResult {
            input_name: input.id.name,
            input_uuid: input.id.uuid.to_string(),
            muted,
            volume_percent: volume_mul_to_percent(volume.mul),
            volume_mul: volume.mul,
            volume_db: volume.db,
            changed_at_ms: timestamp_ms(),
        })
    }

    pub async fn set_audio_muted_by_name(
        &self,
        input_name: &str,
        muted: bool,
    ) -> Result<ObsAudioControlResult, String> {
        let requested = validate_source_name(input_name)?;
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let input = find_input_by_name(client, &requested).await?;

        client
            .inputs()
            .set_muted((&input.id).into(), muted)
            .await
            .map_err(|error| format!("Could not change OBS input mute state: {error}"))?;

        let confirmed = client
            .inputs()
            .muted((&input.id).into())
            .await
            .map_err(|error| format!("OBS changed mute state, but verification failed: {error}"))?;

        if confirmed != muted {
            return Err("OBS did not confirm the requested input mute state.".to_string());
        }

        audio_control_result(client, input, confirmed).await
    }

    pub async fn set_audio_volume_by_name(
        &self,
        input_name: &str,
        percent: u8,
    ) -> Result<ObsAudioControlResult, String> {
        let requested = validate_source_name(input_name)?;
        let percent = validate_audio_percent(percent)?;
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let input = find_input_by_name(client, &requested).await?;
        let requested_mul = f32::from(percent) / 100.0;

        client
            .inputs()
            .set_volume((&input.id).into(), Volume::Mul(requested_mul))
            .await
            .map_err(|error| format!("Could not change OBS input volume: {error}"))?;

        let volume = client
            .inputs()
            .volume((&input.id).into())
            .await
            .map_err(|error| format!("OBS changed input volume, but verification failed: {error}"))?;

        if (volume.mul - requested_mul).abs() > 0.02 {
            return Err("OBS did not confirm the requested input volume.".to_string());
        }

        let muted = client
            .inputs()
            .muted((&input.id).into())
            .await
            .map_err(|error| format!("Could not refresh OBS input mute state: {error}"))?;

        Ok(ObsAudioControlResult {
            input_name: input.id.name,
            input_uuid: input.id.uuid.to_string(),
            muted,
            volume_percent: volume_mul_to_percent(volume.mul),
            volume_mul: volume.mul,
            volume_db: volume.db,
            changed_at_ms: timestamp_ms(),
        })
    }

    pub async fn start_streaming(&self) -> Result<ObsStreamingActionResult, String> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        if read_streaming_active(client).await? {
            return Err("OBS is already streaming.".to_string());
        }

        client
            .streaming()
            .start()
            .await
            .map_err(|error| format!("Could not start OBS streaming: {error}"))?;

        let streaming = wait_for_streaming_state(client, true).await?;

        Ok(ObsStreamingActionResult {
            action: "start".to_string(),
            streaming,
            changed_at_ms: timestamp_ms(),
        })
    }

    pub async fn stop_streaming(&self) -> Result<ObsStreamingActionResult, String> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        if !read_streaming_active(client).await? {
            return Err("OBS is not currently streaming.".to_string());
        }

        client
            .streaming()
            .stop()
            .await
            .map_err(|error| format!("Could not stop OBS streaming: {error}"))?;

        let streaming = wait_for_streaming_state(client, false).await?;

        Ok(ObsStreamingActionResult {
            action: "stop".to_string(),
            streaming,
            changed_at_ms: timestamp_ms(),
        })
    }

    pub async fn production_health(&self) -> Result<ObsProductionHealth, String> {
        let client_guard = self.client.lock().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| "OBS Studio is not connected.".to_string())?;

        let stats = client
            .general()
            .stats()
            .await
            .map_err(|error| format!("Could not read OBS production stats: {error}"))?;

        let stream_status = client
            .streaming()
            .status()
            .await
            .map_err(|error| format!("Could not read OBS stream health: {error}"))?;
        let stream_json = serde_json::to_value(stream_status)
            .map_err(|error| format!("Could not decode OBS stream health: {error}"))?;

        let streaming = json_bool(&stream_json, "outputActive");
        let stream_reconnecting = json_bool(&stream_json, "outputReconnecting");
        let stream_congestion = json_f64(&stream_json, "outputCongestion");
        let stream_output_bytes = json_u64(&stream_json, "outputBytes");
        let stream_output_skipped_frames = json_u64(&stream_json, "outputSkippedFrames");
        let stream_output_total_frames = json_u64(&stream_json, "outputTotalFrames");
        let checked_at_ms = timestamp_ms();

        let stream_bitrate_kbps = if streaming {
            match stream_output_bytes {
                Some(bytes) => {
                    let mut sample = self.health_sample.lock().await;
                    let bitrate = sample.and_then(|previous| {
                        let elapsed_ms = checked_at_ms.saturating_sub(previous.sampled_at_ms);
                        let byte_delta = bytes.checked_sub(previous.output_bytes)?;
                        (elapsed_ms > 0).then_some(
                            (byte_delta as f64 * 8.0) / elapsed_ms as f64
                        )
                    });
                    *sample = Some(ObsHealthSample {
                        output_bytes: bytes,
                        sampled_at_ms: checked_at_ms,
                    });
                    bitrate
                }
                None => None,
            }
        } else {
            *self.health_sample.lock().await = None;
            None
        };

        let render_skipped_percent =
            frame_loss_percent(stats.render_skipped_frames as u64, stats.render_total_frames as u64);
        let output_skipped_percent =
            frame_loss_percent(stats.output_skipped_frames as u64, stats.output_total_frames as u64);

        let (status, summary, issues) = evaluate_production_health(
            stats.cpu_usage,
            stats.available_disk_space,
            stats.average_frame_render_time,
            render_skipped_percent,
            output_skipped_percent,
            streaming,
            stream_reconnecting,
            stream_congestion,
        );

        Ok(ObsProductionHealth {
            status,
            summary,
            issues,
            cpu_usage_percent: stats.cpu_usage,
            memory_usage_mb: stats.memory_usage,
            available_disk_space_mb: stats.available_disk_space,
            active_fps: stats.active_fps,
            average_frame_render_time_ms: stats.average_frame_render_time,
            render_skipped_frames: stats.render_skipped_frames,
            render_total_frames: stats.render_total_frames,
            render_skipped_percent,
            output_skipped_frames: stats.output_skipped_frames,
            output_total_frames: stats.output_total_frames,
            output_skipped_percent,
            streaming,
            stream_reconnecting,
            stream_congestion_percent: stream_congestion.map(|value| value * 100.0),
            stream_bitrate_kbps,
            stream_output_skipped_frames,
            stream_output_total_frames,
            checked_at_ms,
        })
    }

    pub async fn connect(
        &self,
        request: ObsConnectRequest,
    ) -> Result<ObsConnectionState, String> {
        let host = request
            .host
            .unwrap_or_else(|| DEFAULT_OBS_HOST.to_string())
            .trim()
            .to_string();
        let port = request.port.unwrap_or(DEFAULT_OBS_PORT);

        validate_target(&host, port)?;

        let password = request
            .password
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        self.disconnect_client_only().await;
        *self.health_sample.lock().await = None;

        let connect_result = tokio::time::timeout(
            CONNECT_TIMEOUT,
            Client::connect(host.as_str(), port, password.as_deref()),
        )
        .await;

        let client = match connect_result {
            Ok(Ok(client)) => client,
            Ok(Err(error)) => {
                let message = format!("Could not connect to OBS WebSocket: {error}");
                self.record_failure(host, port, message.clone()).await;
                return Err(message);
            }
            Err(_) => {
                let message =
                    "Timed out while connecting to OBS WebSocket. Make sure OBS is running and the WebSocket server is enabled."
                        .to_string();
                self.record_failure(host, port, message.clone()).await;
                return Err(message);
            }
        };

        let version = match client.general().version().await {
            Ok(version) => version,
            Err(error) => {
                let message = format!("Connected to OBS, but version validation failed: {error}");
                self.record_failure(host, port, message.clone()).await;
                return Err(message);
            }
        };

        let state = ObsConnectionState {
            connected: true,
            host,
            port,
            obs_studio_version: Some(version.obs_studio_version.to_string()),
            obs_websocket_version: Some(version.obs_web_socket_version.to_string()),
            rpc_version: Some(version.rpc_version),
            connected_at_ms: Some(timestamp_ms()),
            last_error: None,
        };

        *self.client.lock().await = Some(client);
        *self.state.lock().await = state.clone();

        Ok(state)
    }

    pub async fn disconnect(&self) -> ObsConnectionState {
        self.disconnect_client_only().await;
        *self.health_sample.lock().await = None;

        let mut state = self.state.lock().await;
        state.connected = false;
        state.obs_studio_version = None;
        state.obs_websocket_version = None;
        state.rpc_version = None;
        state.connected_at_ms = None;
        state.last_error = None;

        state.clone()
    }

    async fn disconnect_client_only(&self) {
        let mut client = self.client.lock().await;
        if let Some(mut active) = client.take() {
            active.disconnect().await;
        }
    }

    async fn record_failure(&self, host: String, port: u16, message: String) {
        *self.client.lock().await = None;
        *self.health_sample.lock().await = None;

        let mut state = self.state.lock().await;
        state.connected = false;
        state.host = host;
        state.port = port;
        state.obs_studio_version = None;
        state.obs_websocket_version = None;
        state.rpc_version = None;
        state.connected_at_ms = None;
        state.last_error = Some(message);
    }
}

fn frame_loss_percent(skipped: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }

    skipped as f64 * 100.0 / total as f64
}

fn evaluate_production_health(
    cpu_usage_percent: f64,
    available_disk_space_mb: f64,
    average_frame_render_time_ms: f64,
    render_skipped_percent: f64,
    output_skipped_percent: f64,
    streaming: bool,
    stream_reconnecting: bool,
    stream_congestion: Option<f64>,
) -> (String, String, Vec<String>) {
    let mut critical = Vec::new();
    let mut warnings = Vec::new();

    if stream_reconnecting {
        critical.push("Stream output is reconnecting.".to_string());
    }

    if streaming {
        if let Some(congestion) = stream_congestion {
            if congestion >= 0.95 {
                critical.push(format!(
                    "Stream congestion is critically high ({:.0}%).",
                    congestion * 100.0
                ));
            } else if congestion >= 0.75 {
                warnings.push(format!(
                    "Stream congestion is elevated ({:.0}%).",
                    congestion * 100.0
                ));
            }
        }
    }

    if cpu_usage_percent >= 95.0 {
        critical.push(format!("OBS CPU usage is very high ({cpu_usage_percent:.1}%)."));
    } else if cpu_usage_percent >= 80.0 {
        warnings.push(format!("OBS CPU usage is high ({cpu_usage_percent:.1}%)."));
    }

    if available_disk_space_mb < 1024.0 {
        critical.push(format!(
            "Recording disk space is critically low ({:.1} GB free).",
            available_disk_space_mb / 1024.0
        ));
    } else if available_disk_space_mb < 5120.0 {
        warnings.push(format!(
            "Recording disk space is low ({:.1} GB free).",
            available_disk_space_mb / 1024.0
        ));
    }

    if average_frame_render_time_ms >= 20.0 {
        warnings.push(format!(
            "Average frame render time is high ({average_frame_render_time_ms:.1} ms)."
        ));
    }

    if render_skipped_percent >= 5.0 {
        critical.push(format!(
            "Renderer skipped {:.2}% of frames.",
            render_skipped_percent
        ));
    } else if render_skipped_percent >= 1.0 {
        warnings.push(format!(
            "Renderer skipped {:.2}% of frames.",
            render_skipped_percent
        ));
    }

    if output_skipped_percent >= 5.0 {
        critical.push(format!(
            "Output skipped {:.2}% of frames.",
            output_skipped_percent
        ));
    } else if output_skipped_percent >= 1.0 {
        warnings.push(format!(
            "Output skipped {:.2}% of frames.",
            output_skipped_percent
        ));
    }

    if !critical.is_empty() {
        critical.extend(warnings);
        (
            "critical".to_string(),
            "Production health needs immediate attention.".to_string(),
            critical,
        )
    } else if !warnings.is_empty() {
        (
            "warning".to_string(),
            "Production is running, but one or more metrics need attention.".to_string(),
            warnings,
        )
    } else {
        (
            "good".to_string(),
            "Production health is good.".to_string(),
            Vec::new(),
        )
    }
}

async fn audio_inputs_snapshot(client: &Client) -> Result<ObsAudioInputList, String> {
    let inputs = client
        .inputs()
        .list(None)
        .await
        .map_err(|error| format!("Could not list OBS inputs: {error}"))?;

    let mut summaries = Vec::new();

    for input in inputs {
        let muted = match client.inputs().muted((&input.id).into()).await {
            Ok(value) => value,
            Err(_) => continue,
        };

        let volume = match client.inputs().volume((&input.id).into()).await {
            Ok(value) => value,
            Err(_) => continue,
        };

        summaries.push(ObsAudioInputSummary {
            input_name: input.id.name,
            input_uuid: input.id.uuid.to_string(),
            input_kind: input.kind,
            muted,
            volume_percent: volume_mul_to_percent(volume.mul),
            volume_mul: volume.mul,
            volume_db: volume.db,
        });
    }

    summaries.sort_by(|left, right| {
        left.input_name
            .to_lowercase()
            .cmp(&right.input_name.to_lowercase())
    });

    Ok(ObsAudioInputList {
        inputs: summaries,
        refreshed_at_ms: timestamp_ms(),
        last_error: None,
    })
}

async fn find_input_by_uuid(
    client: &Client,
    input_uuid: &str,
) -> Result<obws::responses::inputs::Input, String> {
    client
        .inputs()
        .list(None)
        .await
        .map_err(|error| format!("Could not list OBS inputs: {error}"))?
        .into_iter()
        .find(|input| input.id.uuid.to_string() == input_uuid)
        .ok_or_else(|| "The requested OBS input no longer exists.".to_string())
}

async fn find_input_by_name(
    client: &Client,
    input_name: &str,
) -> Result<obws::responses::inputs::Input, String> {
    let mut matches = client
        .inputs()
        .list(None)
        .await
        .map_err(|error| format!("Could not list OBS inputs: {error}"))?
        .into_iter()
        .filter(|input| input.id.name.eq_ignore_ascii_case(input_name));

    let input = matches
        .next()
        .ok_or_else(|| format!("No OBS audio input named “{input_name}” exists."))?;

    if matches.next().is_some() {
        return Err(format!(
            "More than one OBS input named “{input_name}” exists. Use the audio controls in Settings."
        ));
    }

    client
        .inputs()
        .volume((&input.id).into())
        .await
        .map_err(|_| format!("OBS input “{}” does not expose audio volume control.", input.id.name))?;

    Ok(input)
}

async fn audio_control_result(
    client: &Client,
    input: obws::responses::inputs::Input,
    muted: bool,
) -> Result<ObsAudioControlResult, String> {
    let volume = client
        .inputs()
        .volume((&input.id).into())
        .await
        .map_err(|error| format!("Could not refresh OBS input volume: {error}"))?;

    Ok(ObsAudioControlResult {
        input_name: input.id.name,
        input_uuid: input.id.uuid.to_string(),
        muted,
        volume_percent: volume_mul_to_percent(volume.mul),
        volume_mul: volume.mul,
        volume_db: volume.db,
        changed_at_ms: timestamp_ms(),
    })
}

fn volume_mul_to_percent(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 100.0).round() as u8
}

fn validate_audio_percent(percent: u8) -> Result<u8, String> {
    if percent > 100 {
        return Err("OBS input volume must be between 0 and 100 percent.".to_string());
    }

    Ok(percent)
}

fn validate_input_uuid(value: &str) -> Result<String, String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("OBS input UUID cannot be empty.".to_string());
    }

    if trimmed.len() > 64 {
        return Err("OBS input UUID is invalid.".to_string());
    }

    Ok(trimmed.to_string())
}

async fn source_items_for_scene(
    client: &Client,
    scene_name: &str,
) -> Result<ObsSourceItemList, String> {
    let scene_name = validate_scene_name(scene_name)?;
    let mut items = client
        .scene_items()
        .list(scene_name.as_str().into())
        .await
        .map_err(|error| format!("Could not list OBS sources in {scene_name}: {error}"))?;

    items.sort_by_key(|item| item.index);

    let mut summaries = Vec::with_capacity(items.len());
    for item in items {
        let enabled = client
            .scene_items()
            .enabled(scene_name.as_str().into(), item.id)
            .await
            .map_err(|error| {
                format!(
                    "Could not read visibility for OBS source {}: {error}",
                    item.source_name
                )
            })?;

        summaries.push(ObsSourceItemSummary {
            scene_name: scene_name.clone(),
            item_id: item.id,
            index: item.index,
            source_name: item.source_name,
            enabled,
            input_kind: item.input_kind,
            is_group: item.is_group.unwrap_or(false),
        });
    }

    Ok(ObsSourceItemList {
        scene_name,
        items: summaries,
        refreshed_at_ms: timestamp_ms(),
        last_error: None,
    })
}

async fn read_streaming_active(client: &Client) -> Result<bool, String> {
    let status = client
        .streaming()
        .status()
        .await
        .map_err(|error| format!("Could not read OBS streaming state: {error}"))?;
    let value = serde_json::to_value(status)
        .map_err(|error| format!("Could not decode OBS streaming state: {error}"))?;

    Ok(json_bool(&value, "outputActive"))
}

async fn wait_for_streaming_state(
    client: &Client,
    expected_active: bool,
) -> Result<bool, String> {
    for _ in 0..8 {
        let active = read_streaming_active(client).await?;
        if active == expected_active {
            return Ok(active);
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Err(format!(
        "OBS did not confirm streaming state active={}.",
        expected_active
    ))
}

async fn wait_for_recording_state(
    client: &Client,
    expected_active: bool,
    expected_paused: bool,
) -> Result<(bool, bool), String> {
    for _ in 0..6 {
        let state = read_recording_flags(client).await?;
        if state == (expected_active, expected_paused) {
            return Ok(state);
        }

        tokio::time::sleep(Duration::from_millis(60)).await;
    }

    Err(format!(
        "OBS did not confirm recording state active={} paused={}.",
        expected_active, expected_paused
    ))
}

async fn read_recording_flags(client: &Client) -> Result<(bool, bool), String> {
    let status = client
        .recording()
        .status()
        .await
        .map_err(|error| format!("Could not read OBS recording state: {error}"))?;

    Ok((status.active, status.paused))
}

fn json_bool(value: &serde_json::Value, key: &str) -> bool {
    value.get(key).and_then(serde_json::Value::as_bool).unwrap_or(false)
}


fn json_u64(value: &serde_json::Value, key: &str) -> Option<u64> {
    value.get(key).and_then(serde_json::Value::as_u64)
}

fn json_f64(value: &serde_json::Value, key: &str) -> Option<f64> {
    value.get(key).and_then(serde_json::Value::as_f64)
}

fn json_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn validate_scene_uuid(value: &str) -> Result<String, String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("OBS scene UUID cannot be empty.".to_string());
    }

    if trimmed.len() > 64 {
        return Err("OBS scene UUID is invalid.".to_string());
    }

    Ok(trimmed.to_string())
}


fn validate_source_name(value: &str) -> Result<String, String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("OBS source name cannot be empty.".to_string());
    }

    if trimmed.chars().count() > 256 {
        return Err("OBS source name is too long.".to_string());
    }

    Ok(trimmed.to_string())
}

fn validate_scene_name(value: &str) -> Result<String, String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("OBS scene name cannot be empty.".to_string());
    }

    if trimmed.chars().count() > 256 {
        return Err("OBS scene name is too long.".to_string());
    }

    Ok(trimmed.to_string())
}

fn validate_target(host: &str, port: u16) -> Result<(), String> {
    if host.is_empty() {
        return Err("OBS WebSocket host cannot be empty.".to_string());
    }

    if host.contains("://") {
        return Err(
            "Use only a hostname or IP address for OBS WebSocket, without ws:// or wss://."
                .to_string(),
        );
    }

    if port == 0 {
        return Err("OBS WebSocket port must be between 1 and 65535.".to_string());
    }

    Ok(())
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
    fn default_state_uses_obs_websocket_v5_port() {
        let state = ObsConnectionState::default();
        assert_eq!(state.host, "127.0.0.1");
        assert_eq!(state.port, 4455);
        assert!(!state.connected);
    }

    #[test]
    fn validates_obs_target() {
        assert!(validate_target("127.0.0.1", 4455).is_ok());
        assert!(validate_target("", 4455).is_err());
        assert!(validate_target("ws://127.0.0.1", 4455).is_err());
        assert!(validate_target("127.0.0.1", 0).is_err());
    }


    #[test]
    fn runtime_state_defaults_to_unavailable() {
        let state = ObsRuntimeState::default();
        assert!(!state.available);
        assert!(!state.streaming);
        assert!(!state.recording);
        assert!(!state.recording_paused);
        assert!(!state.studio_mode);
        assert!(state.current_program_scene.is_none());
        assert!(state.current_preview_scene.is_none());
        assert_eq!(state.stream_duration_ms, 0);
        assert_eq!(state.stream_timecode, "00:00:00.000");
    }

    #[test]
    fn reads_obs_boolean_response_fields_safely() {
        let value = serde_json::json!({
            "outputActive": true,
            "outputPaused": false
        });

        assert!(json_bool(&value, "outputActive"));
        assert!(!json_bool(&value, "outputPaused"));
        assert!(!json_bool(&value, "missing"));
    }


    #[test]
    fn reads_stream_duration_fields_safely() {
        let value = serde_json::json!({
            "outputActive": true,
            "outputDuration": 3723456,
            "outputTimecode": "01:02:03.456"
        });

        assert_eq!(json_u64(&value, "outputDuration"), Some(3_723_456));
        assert_eq!(
            json_string(&value, "outputTimecode").as_deref(),
            Some("01:02:03.456")
        );
        assert_eq!(json_u64(&value, "missing"), None);
    }


    #[test]
    fn scene_list_defaults_to_empty() {
        let scenes = ObsSceneList::default();
        assert!(scenes.scenes.is_empty());
        assert!(scenes.current_program_scene.is_none());
        assert!(scenes.current_preview_scene.is_none());
        assert!(scenes.last_error.is_none());
    }


    #[test]
    fn validates_scene_uuid_input() {
        assert_eq!(
            validate_scene_uuid(" 123e4567-e89b-12d3-a456-426614174000 ").unwrap(),
            "123e4567-e89b-12d3-a456-426614174000"
        );
        assert!(validate_scene_uuid("").is_err());
        assert!(validate_scene_uuid(&"x".repeat(65)).is_err());
    }


    #[test]
    fn validates_scene_name_input() {
        assert_eq!(validate_scene_name("  Camera 2  ").unwrap(), "Camera 2");
        assert!(validate_scene_name("").is_err());
        assert!(validate_scene_name(&"x".repeat(257)).is_err());
    }


    #[test]
    fn validates_source_name_input() {
        assert_eq!(validate_source_name("  Scoreboard  ").unwrap(), "Scoreboard");
        assert!(validate_source_name("").is_err());
        assert!(validate_source_name(&"x".repeat(257)).is_err());
    }

    #[test]
    fn source_visibility_result_serializes_expected_state() {
        let result = ObsSourceVisibilityResult {
            scene_name: "Program".to_string(),
            item_id: 7,
            source_name: "Lower Third".to_string(),
            enabled: true,
            changed_at_ms: 42,
        };

        let value = serde_json::to_value(result).unwrap();
        assert_eq!(value["sceneName"], "Program");
        assert_eq!(value["itemId"], 7);
        assert_eq!(value["sourceName"], "Lower Third");
        assert_eq!(value["enabled"], true);
    }


    #[test]
    fn converts_audio_multiplier_to_percent() {
        assert_eq!(volume_mul_to_percent(0.0), 0);
        assert_eq!(volume_mul_to_percent(0.5), 50);
        assert_eq!(volume_mul_to_percent(1.0), 100);
        assert_eq!(volume_mul_to_percent(1.5), 100);
    }

    #[test]
    fn calculates_frame_loss_percent() {
        assert_eq!(frame_loss_percent(0, 0), 0.0);
        assert_eq!(frame_loss_percent(1, 100), 1.0);
        assert_eq!(frame_loss_percent(5, 100), 5.0);
    }

    #[test]
    fn production_health_grades_good_warning_and_critical() {
        let good = evaluate_production_health(
            20.0, 50_000.0, 4.0, 0.0, 0.0, true, false, Some(0.0),
        );
        assert_eq!(good.0, "good");
        assert!(good.2.is_empty());

        let warning = evaluate_production_health(
            85.0, 50_000.0, 4.0, 0.0, 0.0, true, false, Some(0.0),
        );
        assert_eq!(warning.0, "warning");
        assert!(!warning.2.is_empty());

        let critical = evaluate_production_health(
            20.0, 500.0, 4.0, 0.0, 0.0, true, false, Some(0.0),
        );
        assert_eq!(critical.0, "critical");
        assert!(!critical.2.is_empty());
    }

    #[test]
    fn validates_audio_percent() {
        assert_eq!(validate_audio_percent(0).unwrap(), 0);
        assert_eq!(validate_audio_percent(100).unwrap(), 100);
        assert!(validate_audio_percent(101).is_err());
    }

    #[test]
    fn audio_control_result_serializes_expected_state() {
        let result = ObsAudioControlResult {
            input_name: "Mic/Aux".to_string(),
            input_uuid: "123".to_string(),
            muted: false,
            volume_percent: 70,
            volume_mul: 0.7,
            volume_db: -3.1,
            changed_at_ms: 42,
        };

        let value = serde_json::to_value(result).unwrap();
        assert_eq!(value["inputName"], "Mic/Aux");
        assert_eq!(value["volumePercent"], 70);
        assert_eq!(value["muted"], false);
    }


    #[test]
    fn recording_action_result_serializes_expected_state() {
        let result = ObsRecordingActionResult {
            action: "stop".to_string(),
            recording: false,
            paused: false,
            output_path: Some("C:\\Videos\\capture.mkv".to_string()),
            changed_at_ms: 42,
        };

        let value = serde_json::to_value(result).unwrap();
        assert_eq!(value["action"], "stop");
        assert_eq!(value["recording"], false);
        assert_eq!(value["paused"], false);
        assert_eq!(value["outputPath"], "C:\\Videos\\capture.mkv");
    }


    #[test]
    fn streaming_action_result_serializes_expected_state() {
        let result = ObsStreamingActionResult {
            action: "start".to_string(),
            streaming: true,
            changed_at_ms: 42,
        };

        let value = serde_json::to_value(result).unwrap();
        assert_eq!(value["action"], "start");
        assert_eq!(value["streaming"], true);
        assert_eq!(value["changedAtMs"], 42);
    }
}
