use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
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
pub struct DiagnosticCheck {
    pub id: String,
    pub label: String,
    pub status: String,
    pub detail: String,
}

impl DiagnosticCheck {
    pub fn passed(id: &str, label: &str, detail: impl Into<String>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            status: "passed".to_string(),
            detail: detail.into(),
        }
    }

    pub fn failed(id: &str, label: &str, detail: impl Into<String>) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            status: "failed".to_string(),
            detail: detail.into(),
        }
    }
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
    pub health_status: String,
    pub health_checks: Vec<DiagnosticCheck>,
    pub generated_at_ms: u64,
}

pub fn begin_session(
    app: &AppHandle,
    runtime: &BetaSessionRuntime,
) -> Result<bool, String> {
    let path = session_path(app)?;
    let previous_unclean = previous_session_was_unclean(&path);

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
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!("Could not read Beta session marker: {error}"));
        }
    };
    let mut marker = serde_json::from_str::<SessionMarker>(&content)
        .map_err(|error| format!("Beta session marker is invalid: {error}"))?;

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

pub fn local_health_checks(app: &AppHandle) -> Vec<DiagnosticCheck> {
    vec![
        writable_directory_check(
            "config-storage",
            "Configuration storage",
            app.path()
                .app_config_dir()
                .map_err(|error| error.to_string()),
        ),
        writable_directory_check(
            "local-data-storage",
            "Local data storage",
            app.path()
                .app_local_data_dir()
                .map_err(|error| error.to_string()),
        ),
        session_marker_health_check(app),
        beta_preferences_health_check(app),
    ]
}

fn writable_directory_check(
    id: &str,
    label: &str,
    directory: Result<PathBuf, String>,
) -> DiagnosticCheck {
    let directory = match directory {
        Ok(directory) => directory,
        Err(_) => {
            return DiagnosticCheck::failed(
                id,
                label,
                "AURA could not resolve this local storage directory.",
            );
        }
    };

    if fs::create_dir_all(&directory).is_err() {
        return DiagnosticCheck::failed(
            id,
            label,
            "AURA could not create or access this local storage directory.",
        );
    }

    let probe = directory.join(format!(
        ".aura-beta-health-{}-{}.tmp",
        std::process::id(),
        timestamp_ms()
    ));
    let result = (|| -> Result<(), ()> {
        let mut file = File::create(&probe).map_err(|_| ())?;
        file.write_all(b"AURA-2 beta health probe").map_err(|_| ())?;
        file.sync_all().map_err(|_| ())?;
        drop(file);
        fs::remove_file(&probe).map_err(|_| ())?;
        Ok(())
    })();

    if result.is_ok() {
        DiagnosticCheck::passed(
            id,
            label,
            "Local storage is writable and the health probe was cleaned up.",
        )
    } else {
        let _ = fs::remove_file(&probe);
        DiagnosticCheck::failed(
            id,
            label,
            "AURA could not complete a local read/write health probe.",
        )
    }
}

fn session_marker_health_check(app: &AppHandle) -> DiagnosticCheck {
    let path = match session_path(app) {
        Ok(path) => path,
        Err(_) => {
            return DiagnosticCheck::failed(
                "session-marker",
                "Session marker",
                "AURA could not resolve the Beta session marker.",
            );
        }
    };

    match fs::read_to_string(path) {
        Ok(content) if serde_json::from_str::<SessionMarker>(&content).is_ok() => {
            DiagnosticCheck::passed(
                "session-marker",
                "Session marker",
                "The active Beta session marker is readable and valid.",
            )
        }
        Ok(_) => DiagnosticCheck::failed(
            "session-marker",
            "Session marker",
            "The active Beta session marker is corrupt or invalid.",
        ),
        Err(_) => DiagnosticCheck::failed(
            "session-marker",
            "Session marker",
            "The active Beta session marker could not be read.",
        ),
    }
}

fn beta_preferences_health_check(app: &AppHandle) -> DiagnosticCheck {
    let path = match preferences_path(app) {
        Ok(path) => path,
        Err(_) => {
            return DiagnosticCheck::failed(
                "beta-preferences",
                "Beta preferences",
                "AURA could not resolve the Beta preferences store.",
            );
        }
    };

    if !path.exists() {
        return DiagnosticCheck::passed(
            "beta-preferences",
            "Beta preferences",
            "No persisted Beta preferences exist yet; safe defaults are active.",
        );
    }

    match fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<BetaPreferences>(&content).ok())
    {
        Some(_) => DiagnosticCheck::passed(
            "beta-preferences",
            "Beta preferences",
            "Persisted Beta preferences are readable and valid.",
        ),
        None => DiagnosticCheck::failed(
            "beta-preferences",
            "Beta preferences",
            "Persisted Beta preferences are corrupt or unreadable.",
        ),
    }
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

fn previous_session_was_unclean(path: &Path) -> bool {
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str::<SessionMarker>(&content)
            .map(|marker| !marker.clean_exit)
            .unwrap_or(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(_) => true,
    }
}

fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let content = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("aura-state");
    let temporary = path.with_file_name(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        timestamp_ms()
    ));

    let write_result = (|| -> Result<(), String> {
        let mut file = File::create(&temporary)
            .map_err(|error| format!("Could not create temporary state file: {error}"))?;
        file.write_all(&content)
            .map_err(|error| format!("Could not write temporary state file: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("Could not flush temporary state file: {error}"))?;
        drop(file);

        match fs::rename(&temporary, path) {
            Ok(()) => Ok(()),
            Err(first_error) => {
                if path.exists() {
                    fs::remove_file(path).map_err(|remove_error| {
                        format!(
                            "Could not replace state file after rename failed ({first_error}): {remove_error}"
                        )
                    })?;
                    fs::rename(&temporary, path).map_err(|rename_error| {
                        format!("Could not finalize state file: {rename_error}")
                    })
                } else {
                    Err(format!("Could not finalize state file: {first_error}"))
                }
            }
        }
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }

    write_result
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
    fn beta_defaults_require_first_run_onboarding() {
        let preferences = BetaPreferences::default();
        assert!(!preferences.onboarding_complete);
    }

    #[test]
    fn missing_session_marker_is_not_recovery() {
        let path = std::env::temp_dir().join(format!(
            "aura-beta-missing-{}-{}.json",
            std::process::id(),
            timestamp_ms()
        ));
        let _ = fs::remove_file(&path);
        assert!(!previous_session_was_unclean(&path));
    }

    #[test]
    fn corrupt_session_marker_is_treated_as_unclean() {
        let path = std::env::temp_dir().join(format!(
            "aura-beta-corrupt-{}-{}.json",
            std::process::id(),
            timestamp_ms()
        ));
        fs::write(&path, b"{not-valid-json").expect("write corrupt marker");
        assert!(previous_session_was_unclean(&path));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn atomic_json_write_produces_parseable_state() {
        let path = std::env::temp_dir().join(format!(
            "aura-beta-atomic-{}-{}.json",
            std::process::id(),
            timestamp_ms()
        ));
        let preferences = BetaPreferences {
            onboarding_complete: true,
        };

        write_json(&path, &preferences).expect("atomic state write");
        let content = fs::read_to_string(&path).expect("read state");
        let restored: BetaPreferences = serde_json::from_str(&content).expect("parse state");

        assert!(restored.onboarding_complete);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn diagnostics_schema_is_explicitly_versioned() {
        let snapshot = DiagnosticsSnapshot {
            schema_version: 2,
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
            health_status: "healthy".to_string(),
            health_checks: vec![DiagnosticCheck::passed(
                "privacy",
                "Privacy invariants",
                "No telemetry is enabled.",
            )],
            generated_at_ms: 1,
        };

        assert_eq!(snapshot.schema_version, 2);
        assert!(!snapshot.telemetry_enabled);

        let serialized = serde_json::to_string(&snapshot).expect("serialize diagnostics");
        for forbidden in [
            "prompt",
            "response",
            "transcript",
            "screenshot",
            "password",
            "fileContent",
            "memoryContent",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "diagnostics unexpectedly contain sensitive field {forbidden}"
            );
        }
    }
}
