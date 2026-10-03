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
}
