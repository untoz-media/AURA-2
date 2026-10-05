use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const BETA_STATE_FILE: &str = "beta-state.json";
const SESSION_FILE: &str = "beta-session.json";
const TEST_SESSION_FILE: &str = "beta-test-session.json";
const DIAGNOSTICS_MAX_BYTES: usize = 64 * 1024;
const TEST_REPORT_MAX_BYTES: usize = 96 * 1024;
const CRASH_LOOP_THRESHOLD: u32 = 2;
const DIAGNOSTICS_TOP_LEVEL_FIELDS: &[&str] = &[
    "schemaVersion",
    "appName",
    "appVersion",
    "channel",
    "buildCommit",
    "buildSource",
    "buildLabel",
    "platform",
    "architecture",
    "paused",
    "backgroundEnabled",
    "autostartEnabled",
    "activeModelId",
    "installedModelIds",
    "managedRuntimeState",
    "createImageRuntimeState",
    "agentRunsTotal",
    "activeAgentRuns",
    "savedActions",
    "automations",
    "enabledAutomations",
    "telemetryEnabled",
    "healthStatus",
    "healthChecks",
    "generatedAtMs",
];
const DIAGNOSTIC_CHECK_FIELDS: &[&str] = &["id", "label", "status", "detail"];

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
    consecutive_unclean_sessions: AtomicU32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionMarker {
    session_id: String,
    started_at_ms: u64,
    clean_exit: bool,
    #[serde(default)]
    unclean_streak: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaStatus {
    pub channel: String,
    pub onboarding_complete: bool,
    pub previous_session_unclean: bool,
    pub consecutive_unclean_sessions: u32,
    pub crash_loop_guard_active: bool,
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


const BETA_TEST_AREAS: &[(&str, &str)] = &[
    ("install", "Install & first run"),
    ("lifecycle", "Desktop lifecycle & tray"),
    ("recovery", "Recovery & Crash Loop Guard"),
    ("permissions", "Permissions & safety"),
    ("computer", "Computer Control"),
    ("models", "Managed Runtime & models"),
    ("attachments", "Drag & Drop / file context"),
    ("create", "AURA Create"),
    ("voice", "Voice"),
    ("vision", "Vision"),
    ("memory", "Memory & context"),
    ("agents", "Agents & Automations"),
    ("obs", "Director Mode / OBS"),
    ("diagnostics", "Beta diagnostics"),
    ("installer", "Installer lifecycle"),
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BetaTestCheck {
    pub id: String,
    pub label: String,
    pub completed: bool,
    pub completed_at_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BetaTestSession {
    pub schema_version: u32,
    pub active: bool,
    pub started_at_ms: Option<u64>,
    pub updated_at_ms: u64,
    pub checks: Vec<BetaTestCheck>,
}

impl Default for BetaTestSession {
    fn default() -> Self {
        Self {
            schema_version: 1,
            active: false,
            started_at_ms: None,
            updated_at_ms: timestamp_ms(),
            checks: default_beta_test_checks(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BetaTestReport {
    schema_version: u32,
    session: BetaTestSession,
    diagnostics: DiagnosticsSnapshot,
    generated_at_ms: u64,
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
    pub build_commit: String,
    pub build_source: String,
    pub build_label: String,
    pub platform: String,
    pub architecture: String,
    pub paused: bool,
    pub background_enabled: bool,
    pub autostart_enabled: bool,
    pub active_model_id: Option<String>,
    pub installed_model_ids: Vec<String>,
    pub managed_runtime_state: String,
    pub create_image_runtime_state: String,
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
    let (previous_unclean, consecutive_unclean_sessions) = session_recovery_state(&path);

    runtime
        .previous_session_unclean
        .store(previous_unclean, Ordering::Relaxed);
    runtime
        .consecutive_unclean_sessions
        .store(consecutive_unclean_sessions, Ordering::Relaxed);

    let marker = SessionMarker {
        session_id: format!("session-{}", timestamp_ms()),
        started_at_ms: timestamp_ms(),
        clean_exit: false,
        unclean_streak: consecutive_unclean_sessions,
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


pub fn repair_recovery_state(
    app: &AppHandle,
    runtime: &BetaSessionRuntime,
) -> Result<BetaStatus, String> {
    let path = session_path(app)?;
    let now = timestamp_ms();
    let backup = if path.exists() {
        let backup = path.with_file_name(format!(
            "beta-session.recovery-backup-{now}.json"
        ));
        fs::rename(&path, &backup)
            .map_err(|error| format!("Could not preserve the previous Beta session marker: {error}"))?;
        Some(backup)
    } else {
        None
    };

    let marker = SessionMarker {
        session_id: format!("session-repaired-{now}"),
        started_at_ms: now,
        clean_exit: false,
        unclean_streak: 0,
    };

    if let Err(error) = write_json(&path, &marker) {
        if let Some(backup) = backup.as_ref() {
            if !path.exists() {
                let _ = fs::rename(backup, &path);
            }
        }
        return Err(format!("Could not create a repaired Beta session marker: {error}"));
    }

    runtime
        .previous_session_unclean
        .store(false, Ordering::Relaxed);
    runtime
        .consecutive_unclean_sessions
        .store(0, Ordering::Relaxed);

    status(app, runtime)
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
        consecutive_unclean_sessions: runtime
            .consecutive_unclean_sessions
            .load(Ordering::Relaxed),
        crash_loop_guard_active: runtime
            .consecutive_unclean_sessions
            .load(Ordering::Relaxed)
            >= CRASH_LOOP_THRESHOLD,
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

pub fn test_session(app: &AppHandle) -> Result<BetaTestSession, String> {
    load_test_session(app)
}

pub fn start_test_session(app: &AppHandle) -> Result<BetaTestSession, String> {
    let now = timestamp_ms();
    let session = BetaTestSession {
        schema_version: 1,
        active: true,
        started_at_ms: Some(now),
        updated_at_ms: now,
        checks: default_beta_test_checks(),
    };
    write_json(&test_session_path(app)?, &session)?;
    Ok(session)
}

pub fn set_test_check(
    app: &AppHandle,
    check_id: &str,
    completed: bool,
) -> Result<BetaTestSession, String> {
    if !BETA_TEST_AREAS.iter().any(|(id, _)| *id == check_id) {
        return Err(format!("Unknown Beta test area '{check_id}'."));
    }

    let mut session = load_test_session(app)?;
    if !session.active {
        return Err("Start a Beta test session before updating checklist items.".to_string());
    }

    let now = timestamp_ms();
    let Some(check) = session.checks.iter_mut().find(|check| check.id == check_id) else {
        return Err(format!("Beta test area '{check_id}' is unavailable."));
    };

    check.completed = completed;
    check.completed_at_ms = if completed { Some(now) } else { None };
    session.updated_at_ms = now;
    write_json(&test_session_path(app)?, &session)?;
    Ok(session)
}

pub fn reset_test_session(app: &AppHandle) -> Result<BetaTestSession, String> {
    let session = BetaTestSession::default();
    write_json(&test_session_path(app)?, &session)?;
    Ok(session)
}

pub fn export_test_report(
    app: &AppHandle,
    session: &BetaTestSession,
    diagnostics: &DiagnosticsSnapshot,
) -> Result<String, String> {
    validate_diagnostics_payload(diagnostics)?;

    let report = BetaTestReport {
        schema_version: 1,
        session: normalize_test_session(session.clone()),
        diagnostics: diagnostics.clone(),
        generated_at_ms: timestamp_ms(),
    };

    let serialized = serde_json::to_vec(&report)
        .map_err(|error| format!("Could not serialize Beta test report: {error}"))?;
    if serialized.len() > TEST_REPORT_MAX_BYTES {
        return Err("Beta test report exceeded the local privacy size limit.".to_string());
    }

    let directory = app
        .path()
        .download_dir()
        .or_else(|_| app.path().app_local_data_dir())
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create Beta test report directory: {error}"))?;

    let path = directory.join(format!(
        "AURA-2-Beta-Test-Report-{}.json",
        report.generated_at_ms
    ));
    write_json(&path, &report)?;
    Ok(path.to_string_lossy().to_string())
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
        beta_test_session_health_check(app),
        recovery_streak_health_check(app),
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


fn beta_test_session_health_check(app: &AppHandle) -> DiagnosticCheck {
    let path = match test_session_path(app) {
        Ok(path) => path,
        Err(_) => {
            return DiagnosticCheck::failed(
                "beta-test-session",
                "Beta Test Session",
                "AURA could not resolve the local Beta Test Session store.",
            );
        }
    };

    if !path.exists() {
        return DiagnosticCheck::passed(
            "beta-test-session",
            "Beta Test Session",
            "No Beta Test Session has been started yet.",
        );
    }

    match fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<BetaTestSession>(&content).ok())
        .map(normalize_test_session)
    {
        Some(session) if session.checks.len() == BETA_TEST_AREAS.len() => {
            DiagnosticCheck::passed(
                "beta-test-session",
                "Beta Test Session",
                "The local testing checklist is readable and uses only fixed privacy-safe areas.",
            )
        }
        _ => DiagnosticCheck::failed(
            "beta-test-session",
            "Beta Test Session",
            "The local testing checklist is corrupt or invalid.",
        ),
    }
}

fn recovery_streak_health_check(app: &AppHandle) -> DiagnosticCheck {
    let path = match session_path(app) {
        Ok(path) => path,
        Err(_) => {
            return DiagnosticCheck::failed(
                "recovery-streak",
                "Recovery streak",
                "AURA could not resolve the Beta session marker.",
            );
        }
    };

    let streak = fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<SessionMarker>(&content).ok())
        .map(|marker| marker.unclean_streak)
        .unwrap_or(0);

    if streak >= CRASH_LOOP_THRESHOLD {
        DiagnosticCheck::failed(
            "recovery-streak",
            "Crash loop guard",
            format!(
                "AURA detected {streak} consecutive unclean sessions. Background startup stays visible and AURA remains paused until the recovery state is reviewed."
            ),
        )
    } else {
        DiagnosticCheck::passed(
            "recovery-streak",
            "Recovery streak",
            if streak == 0 {
                "No repeated unclean-session pattern is active.".to_string()
            } else {
                "One unclean session was detected; crash-loop protection is not active.".to_string()
            },
        )
    }
}

pub fn export_diagnostics(
    app: &AppHandle,
    snapshot: &DiagnosticsSnapshot,
) -> Result<String, String> {
    validate_diagnostics_payload(snapshot)?;

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

fn validate_diagnostics_payload(snapshot: &DiagnosticsSnapshot) -> Result<(), String> {
    let serialized = serde_json::to_vec(snapshot)
        .map_err(|error| format!("Could not serialize diagnostics for privacy validation: {error}"))?;
    if serialized.len() > DIAGNOSTICS_MAX_BYTES {
        return Err("Diagnostics export exceeded the local privacy size limit.".to_string());
    }

    let value: serde_json::Value = serde_json::from_slice(&serialized)
        .map_err(|error| format!("Could not validate diagnostics JSON: {error}"))?;
    let Some(object) = value.as_object() else {
        return Err("Diagnostics export must be a JSON object.".to_string());
    };

    for key in object.keys() {
        if !DIAGNOSTICS_TOP_LEVEL_FIELDS.contains(&key.as_str()) {
            return Err(format!(
                "Diagnostics export blocked by privacy guard: unexpected field '{key}'."
            ));
        }
    }

    if let Some(checks) = object.get("healthChecks").and_then(|value| value.as_array()) {
        for check in checks {
            let Some(check_object) = check.as_object() else {
                return Err(
                    "Diagnostics export blocked by privacy guard: invalid health check.".to_string(),
                );
            };
            for key in check_object.keys() {
                if !DIAGNOSTIC_CHECK_FIELDS.contains(&key.as_str()) {
                    return Err(format!(
                        "Diagnostics export blocked by privacy guard: unexpected health-check field '{key}'."
                    ));
                }
            }
        }
    }

    Ok(())
}

fn default_beta_test_checks() -> Vec<BetaTestCheck> {
    BETA_TEST_AREAS
        .iter()
        .map(|(id, label)| BetaTestCheck {
            id: (*id).to_string(),
            label: (*label).to_string(),
            completed: false,
            completed_at_ms: None,
        })
        .collect()
}

fn normalize_test_session(session: BetaTestSession) -> BetaTestSession {
    let checks = BETA_TEST_AREAS
        .iter()
        .map(|(id, label)| {
            session
                .checks
                .iter()
                .find(|check| check.id == *id)
                .map(|check| BetaTestCheck {
                    id: (*id).to_string(),
                    label: (*label).to_string(),
                    completed: check.completed,
                    completed_at_ms: check.completed_at_ms,
                })
                .unwrap_or_else(|| BetaTestCheck {
                    id: (*id).to_string(),
                    label: (*label).to_string(),
                    completed: false,
                    completed_at_ms: None,
                })
        })
        .collect();

    BetaTestSession {
        schema_version: 1,
        active: session.active,
        started_at_ms: session.started_at_ms,
        updated_at_ms: session.updated_at_ms,
        checks,
    }
}

fn load_test_session(app: &AppHandle) -> Result<BetaTestSession, String> {
    let path = test_session_path(app)?;
    if !path.exists() {
        return Ok(BetaTestSession::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read Beta test session: {error}"))?;
    let session = serde_json::from_str::<BetaTestSession>(&content)
        .map_err(|error| format!("Beta test session is invalid: {error}"))?;
    Ok(normalize_test_session(session))
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

fn session_recovery_state(path: &Path) -> (bool, u32) {
    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<SessionMarker>(&content) {
            Ok(marker) if marker.clean_exit => (false, 0),
            Ok(marker) => (true, marker.unclean_streak.saturating_add(1)),
            Err(_) => (true, 1),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (false, 0),
        Err(_) => (true, 1),
    }
}

fn previous_session_was_unclean(path: &Path) -> bool {
    session_recovery_state(path).0
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

fn test_session_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join(TEST_SESSION_FILE))
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
    fn beta_test_session_has_fixed_privacy_safe_areas() {
        let session = BetaTestSession::default();
        assert_eq!(session.schema_version, 1);
        assert!(!session.active);
        assert_eq!(session.checks.len(), BETA_TEST_AREAS.len());
        assert!(session
            .checks
            .iter()
            .all(|check| !check.completed && check.completed_at_ms.is_none()));
    }

    #[test]
    fn beta_test_session_normalization_drops_unknown_areas() {
        let mut session = BetaTestSession::default();
        session.checks.push(BetaTestCheck {
            id: "unexpected-free-text-area".to_string(),
            label: "Unexpected".to_string(),
            completed: true,
            completed_at_ms: Some(1),
        });

        let normalized = normalize_test_session(session);
        assert_eq!(normalized.checks.len(), BETA_TEST_AREAS.len());
        assert!(!normalized
            .checks
            .iter()
            .any(|check| check.id == "unexpected-free-text-area"));
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
    fn repaired_session_marker_is_valid_and_resets_streak() {
        let marker = SessionMarker {
            session_id: "session-repaired-test".to_string(),
            started_at_ms: 1,
            clean_exit: false,
            unclean_streak: 0,
        };
        let serialized = serde_json::to_string(&marker).expect("serialize repaired marker");
        let restored: SessionMarker =
            serde_json::from_str(&serialized).expect("parse repaired marker");

        assert!(!restored.clean_exit);
        assert_eq!(restored.unclean_streak, 0);
    }

    #[test]
    fn repeated_unclean_sessions_increment_recovery_streak() {
        let path = std::env::temp_dir().join(format!(
            "aura-beta-streak-{}-{}.json",
            std::process::id(),
            timestamp_ms()
        ));
        let marker = SessionMarker {
            session_id: "test".to_string(),
            started_at_ms: 1,
            clean_exit: false,
            unclean_streak: 1,
        };
        write_json(&path, &marker).expect("write marker");
        assert_eq!(session_recovery_state(&path), (true, 2));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn clean_session_resets_recovery_streak() {
        let path = std::env::temp_dir().join(format!(
            "aura-beta-clean-streak-{}-{}.json",
            std::process::id(),
            timestamp_ms()
        ));
        let marker = SessionMarker {
            session_id: "test".to_string(),
            started_at_ms: 1,
            clean_exit: true,
            unclean_streak: 4,
        };
        write_json(&path, &marker).expect("write marker");
        assert_eq!(session_recovery_state(&path), (false, 0));
        let _ = fs::remove_file(path);
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
            schema_version: 3,
            app_name: "AURA-2".to_string(),
            app_version: "0.9.0-beta.1".to_string(),
            channel: "beta".to_string(),
            build_commit: "test-commit".to_string(),
            build_source: "test".to_string(),
            build_label: "diagnostics-test".to_string(),
            platform: "windows".to_string(),
            architecture: "x86_64".to_string(),
            paused: false,
            background_enabled: true,
            autostart_enabled: false,
            active_model_id: None,
            installed_model_ids: vec![],
            managed_runtime_state: "ready".to_string(),
            create_image_runtime_state: "stopped".to_string(),
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

        assert_eq!(snapshot.schema_version, 3);
        assert!(!snapshot.telemetry_enabled);
        validate_diagnostics_payload(&snapshot).expect("diagnostics privacy guard");

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
