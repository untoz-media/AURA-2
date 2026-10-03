use obws::Client;
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
}

impl Default for ObsController {
    fn default() -> Self {
        Self {
            client: Mutex::new(None),
            state: Mutex::new(ObsConnectionState::default()),
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
}
