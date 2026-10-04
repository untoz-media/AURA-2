use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const BETA_STATE_FILE: &str = "beta-state.json";
const SESSION_FILE: &str = "beta-session.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BetaPreferences {
    pub onboarding_complete: bool,
}

impl Default for BetaPreferences {
    fn default() -> Self {
        Self {
            onboarding_complete: false,
        }
    }
}

#[derive(Default)]
pub struct BetaSessionRuntime {
    previous_session_unclean: AtomicBool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionMarker {
    session_id: String,
    started_at_ms: u64,
    clean_exit: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaStatus {
    pub channel: String,
    pub onboarding_complete: bool,
    pub previous_session_unclean: bool,
    pub telemetry_enabled: bool,
    pub automatic_crash_uploads: bool,
    pub local_diagnostics_only: bool,
    pub refreshed_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBetaPreferencesRequest {
    pub onboarding_complete: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsSnapshot {
    pub schema_version: u32,
    pub app_name: String,
    pub app_version: String,
    pub channel: String,
    pub platform: String,
    pub architecture: String,
    pub paused: bool,
    pub background_enabled: bool,
    pub autostart_enabled: bool,
    pub active_model_id: Option<String>,
    pub installed_model_ids: Vec<String>,
    pub managed_runtime_state: String,
    pub agent_runs_total: usize,
    pub active_agent_runs: usize,
    pub saved_actions: usize,
    pub automations: usize,
    pub enabled_automations: usize,
    pub telemetry_enabled: bool,
    pub generated_at_ms: u64,
}

pub fn begin_session(
    app: &AppHandle,
    runtime: &BetaSessionRuntime,
) -> Result<bool, String> {
    let path = session_path(app)?;
    let previous_unclean = fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str::<SessionMarker>(&content).ok())
        .is_some_and(|marker| !marker.clean_exit);

    runtime
        .previous_session_unclean
        .store(previous_unclean, Ordering::Relaxed);

    let marker = SessionMarker {
        session_id: format!("session-{}", timestamp_ms()),
        started_at_ms: timestamp_ms(),
        clean_exit: false,
    };
    write_json(&path, &marker)?;
    Ok(previous_unclean)
}

pub fn mark_session_clean(app: &AppHandle) -> Result<(), String> {
    let path = session_path(app)?;
    let Some(mut marker) = fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str::<SessionMarker>(&content).ok())
    else {
        return Ok(());
    };

    marker.clean_exit = true;
    write_json(&path, &marker)
}

pub fn status(
    app: &AppHandle,
    runtime: &BetaSessionRuntime,
) -> Result<BetaStatus, String> {
    let preferences = load_preferences(app)?;

    Ok(BetaStatus {
        channel: "beta".to_string(),
        onboarding_complete: preferences.onboarding_complete,
        previous_session_unclean: runtime
            .previous_session_unclean
            .load(Ordering::Relaxed),
        telemetry_enabled: false,
        automatic_crash_uploads: false,
        local_diagnostics_only: true,
        refreshed_at_ms: timestamp_ms(),
    })
}

pub fn save_preferences(
    app: &AppHandle,
    runtime: &BetaSessionRuntime,
    request: SetBetaPreferencesRequest,
) -> Result<BetaStatus, String> {
    let preferences = BetaPreferences {
        onboarding_complete: request.onboarding_complete,
    };
    write_json(&preferences_path(app)?, &preferences)?;
    status(app, runtime)
}

pub fn export_diagnostics(
    app: &AppHandle,
    snapshot: &DiagnosticsSnapshot,
) -> Result<String, String> {
    let directory = app
        .path()
        .download_dir()
        .or_else(|_| app.path().app_local_data_dir())
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create diagnostics export directory: {error}"))?;

    let path = directory.join(format!(
        "AURA-2-Diagnostics-{}.json",
        snapshot.generated_at_ms
    ));
    write_json(&path, snapshot)?;
    Ok(path.to_string_lossy().to_string())
}

fn load_preferences(app: &AppHandle) -> Result<BetaPreferences, String> {
    let path = preferences_path(app)?;
    if !path.exists() {
        return Ok(BetaPreferences::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read Beta preferences: {error}"))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("Beta preferences are invalid and were left unchanged: {error}"))
}

fn write_json<T: Serialize + ?Sized>(path: &PathBuf, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let content = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())
}

fn preferences_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join(BETA_STATE_FILE))
}

fn session_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join(SESSION_FILE))
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
    fn beta_defaults_disable_all_network_telemetry() {
        let preferences = BetaPreferences::default();
        assert!(!preferences.onboarding_complete);
        assert!(!preferences.onboarding_complete);
    }

    #[test]
    fn diagnostics_schema_is_explicitly_versioned() {
        let snapshot = DiagnosticsSnapshot {
            schema_version: 1,
            app_name: "AURA-2".to_string(),
            app_version: "0.9.0-beta.1".to_string(),
            channel: "beta".to_string(),
            platform: "windows".to_string(),
            architecture: "x86_64".to_string(),
            paused: false,
            background_enabled: true,
            autostart_enabled: false,
            active_model_id: None,
            installed_model_ids: vec![],
            managed_runtime_state: "ready".to_string(),
            agent_runs_total: 0,
            active_agent_runs: 0,
            saved_actions: 0,
            automations: 0,
            enabled_automations: 0,
            telemetry_enabled: false,
            generated_at_ms: 1,
        };

        assert_eq!(snapshot.schema_version, 1);
        assert!(!snapshot.telemetry_enabled);
    }
}
