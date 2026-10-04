mod audio_input;
mod computer;
mod core;
mod integrations;
mod managed_runtime;
mod memory;
mod model_manager;
mod model_runtime;
mod permissions;
mod project_memory;
mod routines;
mod speech_runtime;
mod tts_runtime;

use audio_input::{AudioInputManager, AudioInputSnapshot, CapturedAudio};
use computer::app_launcher::launch_app;
use computer::app_lifecycle::close_app;
use computer::audio::{execute_media_action, MediaAction};
use computer::keyboard::{press_shortcut, type_text};
use computer::mouse::{execute_mouse_action, MouseAction};
use computer::recent_files::{
    default_recent_files_snapshot, recent_files_snapshot, summarize_recent_files, RecentFilesSnapshot,
};
use computer::system::{execute_system_action, summarize_system, SystemAction};
use computer::window_manager::{summarize_windows, switch_to_app, CurrentAppAwareness, CurrentAppInfo};
use core::{
    action_router::{route_command, ActionIntent, ObsRecordingAction, ObsStreamingAction, RouteResult, RoutedAction},
    confirmation::{
        validate_pending_confirmation, PendingConfirmation, CONFIRMATION_TTL_MS,
    },
};
use integrations::director::{
    delete_director_preset, find_director_preset_by_id, load_director_presets,
    preset_requires_sensitive_permission, resolve_director_preset_command, run_director_preset,
    save_director_preset, DirectorPreset, DirectorPresetRunResult, SaveDirectorPresetRequest,
};
use integrations::obs::{ObsAudioControlResult, ObsAudioInputList, ObsAudioMuteRequest, ObsAudioVolumeRequest, ObsConnectRequest, ObsConnectionState, ObsController, ObsProductionHealth, ObsRecordingActionResult, ObsRuntimeState, ObsSceneList, ObsSceneSwitchRequest, ObsSceneSwitchResult, ObsSourceItemList, ObsSourceVisibilityRequest, ObsSourceVisibilityResult, ObsStreamDuration, ObsStreamingActionResult};
use memory::{
    create_memory, delete_memory, delete_memory_by_content, memory_snapshot, summarize_memories,
    CreateMemoryRequest, MemoryCreateResult, MemoryRecord, MemorySnapshot,
};
use managed_runtime::{ManagedRuntimeSetup, ManagedRuntimeStatus};
use model_manager::{ModelCatalog, ModelManager};
use model_runtime::{ModelRuntime, ModelRuntimeStatus};
use permissions::{PermissionClass, PermissionDecision, PermissionPolicy};
use project_memory::{
    active_project, delete_project, project_snapshot, save_project, set_active_project,
    summarize_active_project, ProjectMemory, ProjectMemorySnapshot, SaveProjectRequest,
};
use routines::{
    delete_routine, find_routine_by_id, list_routines, resolve_routine_command,
    routine_requires_sensitive_permission, run_routine, save_routine, RoutineRunResult,
    SaveRoutineRequest, UserRoutine,
};
use speech_runtime::{SpeechRuntime, SpeechRuntimeStatus};
use tts_runtime::{TtsRuntime, TtsRuntimeStatus};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};

static COMMAND_COUNTER: AtomicU64 = AtomicU64::new(1);

struct RuntimeState {
    paused: Mutex<bool>,
    background_enabled: Mutex<bool>,
    autostart_enabled: Mutex<bool>,
    permission_policy: Mutex<PermissionPolicy>,
    pending_confirmations: Mutex<HashMap<String, PendingConfirmation>>,
    voice_command_ids: Mutex<HashSet<String>>,
    voice_preferences: Mutex<VoicePreferences>,
    wake_monitor_generation: AtomicU64,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            paused: Mutex::new(false),
            background_enabled: Mutex::new(true),
            autostart_enabled: Mutex::new(false),
            permission_policy: Mutex::new(PermissionPolicy::default()),
            pending_confirmations: Mutex::new(HashMap::new()),
            voice_command_ids: Mutex::new(HashSet::new()),
            voice_preferences: Mutex::new(VoicePreferences::default()),
            wake_monitor_generation: AtomicU64::new(1),
        }
    }
}

#[derive(Clone, Serialize)]
enum AuraRuntimeStatus {
    Idle,
    Listening,
    Thinking,
    Working,
    Waiting,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppStatus {
    name: &'static str,
    version: &'static str,
    stage: &'static str,
    local_first: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeSnapshot {
    paused: bool,
    background_enabled: bool,
    autostart_enabled: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct VoicePreferences {
    auto_speak: bool,
    tts_speed: f32,
    conversation_mode: bool,
    conversation_timeout_seconds: u64,
    wake_word_enabled: bool,
    wake_phrase: String,
    tts_voice_id: String,
}

impl Default for VoicePreferences {
    fn default() -> Self {
        Self {
            auto_speak: true,
            tts_speed: 1.0,
            conversation_mode: false,
            conversation_timeout_seconds: 8,
            wake_word_enabled: false,
            wake_phrase: "AURA".to_string(),
            tts_voice_id: "voice-piper-ptpt".to_string(),
        }
    }
}

impl VoicePreferences {
    fn sanitized(mut self) -> Self {
        self.tts_speed = self.tts_speed.clamp(0.6, 1.5);
        self.conversation_timeout_seconds =
            self.conversation_timeout_seconds.clamp(3, 20);
        self.wake_phrase = self.wake_phrase.trim().to_string();
        if self.wake_phrase.is_empty() {
            self.wake_phrase = "AURA".to_string();
        }
        if !matches!(
            self.tts_voice_id.as_str(),
            "voice-piper-ptpt" | "voice-piper-engb-alan"
        ) {
            self.tts_voice_id = "voice-piper-ptpt".to_string();
        }
        if self.wake_phrase.chars().count() > 32 {
            self.wake_phrase = self.wake_phrase.chars().take(32).collect();
        }
        self
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopPreferences {
    background_enabled: bool,
}

impl Default for DesktopPreferences {
    fn default() -> Self {
        Self {
            background_enabled: true,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LifecycleEvent {
    kind: &'static str,
    message: String,
    timestamp_ms: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VoiceCaptureEvent {
    phase: &'static str,
    shortcut: &'static str,
    sample_count: usize,
    duration_ms: u64,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    message: String,
    text: Option<String>,
    timestamp_ms: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandRequest {
    text: String,
    source: String,
    #[serde(default)]
    approval_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandAck {
    id: String,
    accepted: bool,
    status: AuraRuntimeStatus,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CoreEvent {
    id: String,
    kind: &'static str,
    status: AuraRuntimeStatus,
    message: String,
    command: Option<String>,
    timestamp_ms: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CoreError {
    id: Option<String>,
    code: &'static str,
    message: String,
}

fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}


fn format_duration_ms(duration_ms: u64) -> String {
    let total_seconds = duration_ms / 1_000;
    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;

    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

fn should_prefer_director_preset(text: &str, base_route: &RouteResult) -> bool {
    let normalized = text
        .trim()
        .trim_matches(|character: char| {
            matches!(character, '.' | ',' | '!' | '?' | ';' | ':')
        })
        .to_lowercase();

    let explicit_preset = [
        "run director preset ",
        "run preset ",
        "execute preset ",
        "executa o preset ",
        "executa preset ",
        "ativa o preset ",
        "ativa preset ",
    ]
    .iter()
    .any(|prefix| normalized.starts_with(prefix));

    if explicit_preset {
        return true;
    }

    match base_route {
        RouteResult::NoMatch | RouteResult::UnsupportedApp(_) => true,
        RouteResult::Action(RoutedAction {
            intent: ActionIntent::ObsProgramScene(_),
            ..
        }) => [
            "switch to ",
            "focus ",
            "go to ",
            "vai para ",
            "muda para ",
            "troca para ",
            "foca ",
            "focar ",
        ]
        .iter()
        .any(|prefix| normalized.starts_with(prefix)),
        _ => false,
    }
}

fn normalize_voice_command(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let lowered = trimmed.to_lowercase();
    let prefixes = [
        "hey aura",
        "ok aura",
        "okay aura",
        "olá aura",
        "ola aura",
        "aura",
    ];

    for prefix in prefixes {
        if lowered == prefix {
            return String::new();
        }

        if lowered.starts_with(prefix) {
            let boundary = trimmed
                .chars()
                .nth(prefix.chars().count())
                .is_some_and(|character| {
                    character.is_whitespace()
                        || matches!(character, ',' | ':' | ';' | '-' | '—')
                });

            if boundary {
                return trimmed[prefix.len()..]
                    .trim_start_matches(|character: char| {
                        character.is_whitespace()
                            || matches!(character, ',' | ':' | ';' | '-' | '—')
                    })
                    .trim()
                    .to_string();
            }
        }
    }

    trimmed.to_string()
}

fn should_prefer_user_routine(text: &str, base_route: &RouteResult) -> bool {
    let normalized = text
        .trim()
        .trim_matches(|character: char| {
            matches!(character, '.' | ',' | '!' | '?' | ';' | ':')
        })
        .to_lowercase();

    let explicit_routine = [
        "run routine ",
        "execute routine ",
        "start routine ",
        "executa a rotina ",
        "executa rotina ",
        "inicia a rotina ",
        "inicia rotina ",
    ]
    .iter()
    .any(|prefix| normalized.starts_with(prefix));

    explicit_routine
        || matches!(
            base_route,
            RouteResult::NoMatch | RouteResult::UnsupportedApp(_)
        )
}

fn route_user_routine(
    app: &AppHandle,
    text: &str,
    policy: &PermissionPolicy,
) -> Option<RouteResult> {
    let routine = resolve_routine_command(app, text)?;
    let permission = if routine_requires_sensitive_permission(app, &routine) {
        PermissionClass::Sensitive
    } else {
        PermissionClass::Act
    };

    Some(RouteResult::Action(RoutedAction {
        intent: ActionIntent::UserRoutine(routine.id),
        permission,
        decision: policy.decision_for(permission),
    }))
}

fn route_director_preset(
    app: &AppHandle,
    text: &str,
    policy: &PermissionPolicy,
) -> Option<RouteResult> {
    let preset = resolve_director_preset_command(app, text)?;
    let permission = if preset_requires_sensitive_permission(&preset) {
        PermissionClass::Sensitive
    } else {
        PermissionClass::Act
    };

    Some(RouteResult::Action(RoutedAction {
        intent: ActionIntent::DirectorPreset(preset.id),
        permission,
        decision: policy.decision_for(permission),
    }))
}

#[cfg(test)]
mod director_route_tests {
    use super::*;

    fn routed(intent: ActionIntent) -> RouteResult {
        RouteResult::Action(RoutedAction {
            intent,
            permission: PermissionClass::Act,
            decision: PermissionDecision::Allow,
        })
    }

    #[test]
    fn director_presets_override_only_generic_fallbacks() {
        assert!(should_prefer_director_preset(
            "run preset Prepare Match",
            &RouteResult::NoMatch,
        ));
        assert!(should_prefer_director_preset(
            "start show",
            &RouteResult::UnsupportedApp("show".to_string()),
        ));
        assert!(should_prefer_director_preset(
            "go to break",
            &routed(ActionIntent::ObsProgramScene("break".to_string())),
        ));

        assert!(!should_prefer_director_preset(
            "Mute",
            &routed(ActionIntent::Media(MediaAction::Mute)),
        ));
        assert!(!should_prefer_director_preset(
            "switch scene to Camera 2",
            &routed(ActionIntent::ObsProgramScene("Camera 2".to_string())),
        ));
    }
}

#[cfg(test)]
mod voice_command_tests {
    use super::*;

    #[test]
    fn removes_aura_wake_prefix_without_changing_command() {
        assert_eq!(normalize_voice_command("AURA, abre o Brave"), "abre o Brave");
        assert_eq!(normalize_voice_command("Hey AURA: open Brave"), "open Brave");
        assert_eq!(normalize_voice_command("Olá AURA — abre o OBS"), "abre o OBS");
    }

    #[test]
    fn preserves_commands_without_wake_prefix() {
        assert_eq!(normalize_voice_command("abre o Brave"), "abre o Brave");
    }

    #[test]
    fn wake_word_alone_is_not_a_command() {
        assert_eq!(normalize_voice_command("AURA"), "");
    }

    #[test]
    fn extracts_configured_wake_phrase_and_followup() {
        assert_eq!(
            extract_wake_command("AURA, abre o Brave", "AURA"),
            Some("abre o Brave".to_string())
        );
        assert_eq!(
            extract_wake_command("AURA", "AURA"),
            Some(String::new())
        );
        assert_eq!(
            extract_wake_command("abre o Brave", "AURA"),
            None
        );
    }

    #[test]
    fn voice_preferences_are_sanitized() {
        let preferences = VoicePreferences {
            auto_speak: true,
            tts_speed: 9.0,
            conversation_mode: true,
            conversation_timeout_seconds: 99,
            wake_word_enabled: true,
            wake_phrase: "   ".to_string(),
            tts_voice_id: "unknown".to_string(),
        }
        .sanitized();

        assert_eq!(preferences.tts_speed, 1.5);
        assert_eq!(preferences.conversation_timeout_seconds, 20);
        assert_eq!(preferences.wake_phrase, "AURA");
        assert_eq!(preferences.tts_voice_id, "voice-piper-ptpt");
    }
}

fn next_command_id() -> String {
    let counter = COMMAND_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("cmd-{}-{}", unix_timestamp_ms(), counter)
}

fn permission_policy_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directory = app.path().app_config_dir().map_err(|error| error.to_string())?;
    Ok(directory.join("permission-policy.json"))
}

fn load_permission_policy(app: &tauri::AppHandle) -> PermissionPolicy {
    let Ok(path) = permission_policy_path(app) else {
        return PermissionPolicy::default();
    };

    let Ok(content) = fs::read_to_string(path) else {
        return PermissionPolicy::default();
    };

    serde_json::from_str::<PermissionPolicy>(&content)
        .unwrap_or_default()
        .sanitized()
}

fn save_permission_policy(
    app: &tauri::AppHandle,
    policy: &PermissionPolicy,
) -> Result<(), String> {
    let path = permission_policy_path(app)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let content = serde_json::to_string_pretty(policy).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())
}

fn voice_preferences_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directory = app.path().app_config_dir().map_err(|error| error.to_string())?;
    Ok(directory.join("voice-preferences.json"))
}

fn load_voice_preferences(app: &tauri::AppHandle) -> VoicePreferences {
    let Ok(path) = voice_preferences_path(app) else {
        return VoicePreferences::default();
    };
    let Ok(content) = fs::read_to_string(path) else {
        return VoicePreferences::default();
    };
    serde_json::from_str::<VoicePreferences>(&content)
        .unwrap_or_default()
        .sanitized()
}

fn save_voice_preferences(
    app: &tauri::AppHandle,
    preferences: &VoicePreferences,
) -> Result<(), String> {
    let path = voice_preferences_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let content =
        serde_json::to_string_pretty(preferences).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_voice_preferences(state: State<'_, RuntimeState>) -> VoicePreferences {
    state
        .voice_preferences
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

#[tauri::command]
fn set_voice_preferences(
    app: AppHandle,
    state: State<'_, RuntimeState>,
    preferences: VoicePreferences,
) -> Result<VoicePreferences, String> {
    let preferences = preferences.sanitized();
    let previous_voice = state
        .voice_preferences
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .tts_voice_id
        .clone();
    save_voice_preferences(&app, &preferences)?;

    if previous_voice != preferences.tts_voice_id {
        app.state::<TtsRuntime>().stop();
    }

    *state
        .voice_preferences
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = preferences.clone();
    let generation = state
        .wake_monitor_generation
        .fetch_add(1, Ordering::Relaxed)
        .saturating_add(1);

    if preferences.wake_word_enabled {
        spawn_wake_monitor(app.clone(), generation);
    }

    Ok(preferences)
}

fn preferences_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directory = app.path().app_config_dir().map_err(|error| error.to_string())?;
    Ok(directory.join("desktop-preferences.json"))
}

fn load_preferences(app: &tauri::AppHandle) -> DesktopPreferences {
    let Ok(path) = preferences_path(app) else {
        return DesktopPreferences::default();
    };

    let Ok(content) = fs::read_to_string(path) else {
        return DesktopPreferences::default();
    };

    serde_json::from_str(&content).unwrap_or_default()
}

fn save_preferences(
    app: &tauri::AppHandle,
    preferences: &DesktopPreferences,
) -> Result<(), String> {
    let path = preferences_path(app)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let content =
        serde_json::to_string_pretty(preferences).map_err(|error| error.to_string())?;

    fs::write(path, content).map_err(|error| error.to_string())
}

fn runtime_snapshot(state: &RuntimeState) -> RuntimeSnapshot {
    RuntimeSnapshot {
        paused: *state
            .paused
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        background_enabled: *state
            .background_enabled
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        autostart_enabled: *state
            .autostart_enabled
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
    }
}

fn emit_runtime_state(app: &tauri::AppHandle) -> RuntimeSnapshot {
    let state = app.state::<RuntimeState>();
    let snapshot = runtime_snapshot(&state);
    let _ = app.emit("aura:runtime-state", snapshot.clone());
    snapshot
}

fn emit_lifecycle_event(app: &tauri::AppHandle, kind: &'static str, message: &str) {
    let _ = app.emit(
        "aura:lifecycle-event",
        LifecycleEvent {
            kind,
            message: message.to_string(),
            timestamp_ms: unix_timestamp_ms(),
        },
    );
}

fn set_paused_state(app: &tauri::AppHandle, paused: bool) -> RuntimeSnapshot {
    let state = app.state::<RuntimeState>();
    {
        let mut current = state
            .paused
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = paused;
    }

    emit_runtime_state(app)
}

fn set_background_state(
    app: &tauri::AppHandle,
    background_enabled: bool,
) -> Result<RuntimeSnapshot, String> {
    save_preferences(
        app,
        &DesktopPreferences {
            background_enabled,
        },
    )?;

    let state = app.state::<RuntimeState>();
    {
        let mut current = state
            .background_enabled
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = background_enabled;
    }

    let snapshot = emit_runtime_state(app);

    if background_enabled {
        emit_lifecycle_event(
            app,
            "background.enabled",
            "Background mode enabled. AURA will remain available after the main window closes.",
        );
    } else {
        emit_lifecycle_event(
            app,
            "background.disabled",
            "Background mode disabled. Closing the main window will quit AURA.",
        );
    }

    Ok(snapshot)
}

fn set_autostart_state(
    app: &tauri::AppHandle,
    autostart_enabled: bool,
) -> Result<RuntimeSnapshot, String> {
    let manager = app.autolaunch();

    if autostart_enabled {
        manager.enable().map_err(|error| error.to_string())?;
    } else {
        manager.disable().map_err(|error| error.to_string())?;
    }

    let actual = manager.is_enabled().map_err(|error| error.to_string())?;

    let state = app.state::<RuntimeState>();
    {
        let mut current = state
            .autostart_enabled
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = actual;
    }

    let snapshot = emit_runtime_state(app);

    emit_lifecycle_event(
        app,
        if actual { "autostart.enabled" } else { "autostart.disabled" },
        if actual {
            "AURA will start silently with Windows."
        } else {
            "AURA will no longer start automatically with Windows."
        },
    );

    Ok(snapshot)
}

fn emit_voice_capture_event(app: &tauri::AppHandle, event: VoiceCaptureEvent) {
    let _ = app.emit("aura:voice-capture", event);
}

fn process_voice_capture(
    app: tauri::AppHandle,
    capture: CapturedAudio,
    source_label: &'static str,
) {
    thread::spawn(move || {
        let sample_count = capture.samples.len();
        let duration_ms = capture.completed_at_ms.saturating_sub(capture.started_at_ms);

        emit_voice_capture_event(
            &app,
            VoiceCaptureEvent {
                phase: "transcribing",
                shortcut: source_label,
                sample_count,
                duration_ms,
                sample_rate: Some(capture.sample_rate),
                channels: Some(capture.channels),
                message: "Transcribing locally with AURA Voice STT…".to_string(),
                text: None,
                timestamp_ms: unix_timestamp_ms(),
            },
        );

        let manager = app.state::<ModelManager>();
        let speech = app.state::<SpeechRuntime>();

        match speech.transcribe(&app, &manager, capture) {
            Ok(result) => {
                let transcript = result.text.trim().to_string();

                if result.duration_ms < 250 || result.input_rms < 0.003 {
                    emit_voice_capture_event(
                        &app,
                        VoiceCaptureEvent {
                            phase: "error",
                            shortcut: source_label,
                            sample_count: result.input_samples_16khz,
                            duration_ms: result.duration_ms,
                            sample_rate: Some(16_000),
                            channels: Some(1),
                            message: "Voice capture was too short or too quiet to execute safely.".to_string(),
                            text: Some(transcript),
                            timestamp_ms: unix_timestamp_ms(),
                        },
                    );
                    return;
                }

                emit_voice_capture_event(
                    &app,
                    VoiceCaptureEvent {
                        phase: "transcribed",
                        shortcut: source_label,
                        sample_count: result.input_samples_16khz,
                        duration_ms: result.duration_ms,
                        sample_rate: Some(16_000),
                        channels: Some(1),
                        message: "Local transcription complete.".to_string(),
                        text: Some(transcript.clone()),
                        timestamp_ms: unix_timestamp_ms(),
                    },
                );

                let command_text = normalize_voice_command(&transcript);
                if command_text.is_empty() {
                    emit_voice_capture_event(
                        &app,
                        VoiceCaptureEvent {
                            phase: "error",
                            shortcut: source_label,
                            sample_count: result.input_samples_16khz,
                            duration_ms: result.duration_ms,
                            sample_rate: Some(16_000),
                            channels: Some(1),
                            message: "AURA heard the wake name but no command followed it.".to_string(),
                            text: Some(transcript),
                            timestamp_ms: unix_timestamp_ms(),
                        },
                    );
                    return;
                }

                let state = app.state::<RuntimeState>();
                match process_user_command(
                    app.clone(),
                    state,
                    CommandRequest {
                        text: command_text.clone(),
                        source: "voice".to_string(),
                        approval_id: None,
                    },
                ) {
                    Ok(_) => emit_voice_capture_event(
                        &app,
                        VoiceCaptureEvent {
                            phase: "submitted",
                            shortcut: source_label,
                            sample_count: result.input_samples_16khz,
                            duration_ms: result.duration_ms,
                            sample_rate: Some(16_000),
                            channels: Some(1),
                            message: "Voice command sent to AURA Core.".to_string(),
                            text: Some(command_text),
                            timestamp_ms: unix_timestamp_ms(),
                        },
                    ),
                    Err(error) => emit_voice_capture_event(
                        &app,
                        VoiceCaptureEvent {
                            phase: "error",
                            shortcut: source_label,
                            sample_count: result.input_samples_16khz,
                            duration_ms: result.duration_ms,
                            sample_rate: Some(16_000),
                            channels: Some(1),
                            message: error,
                            text: Some(command_text),
                            timestamp_ms: unix_timestamp_ms(),
                        },
                    ),
                }
            }
            Err(error) => emit_voice_capture_event(
                &app,
                VoiceCaptureEvent {
                    phase: "error",
                    shortcut: source_label,
                    sample_count: 0,
                    duration_ms: 0,
                    sample_rate: None,
                    channels: None,
                    message: error,
                    text: None,
                    timestamp_ms: unix_timestamp_ms(),
                },
            ),
        }
    });
}

fn start_conversation_follow_up(app: tauri::AppHandle, force: bool) {
    thread::spawn(move || {
        let preferences = app
            .state::<RuntimeState>()
            .voice_preferences
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();

        if !force && !preferences.conversation_mode {
            return;
        }

        let audio = app.state::<AudioInputManager>();
        if audio.capture_active() {
            return;
        }

        if let Err(error) = audio.start_push_to_talk() {
            emit_voice_capture_event(
                &app,
                VoiceCaptureEvent {
                    phase: "error",
                    shortcut: "conversation",
                    sample_count: 0,
                    duration_ms: 0,
                    sample_rate: None,
                    channels: None,
                    message: error,
                    text: None,
                    timestamp_ms: unix_timestamp_ms(),
                },
            );
            return;
        }

        emit_voice_capture_event(
            &app,
            VoiceCaptureEvent {
                phase: "conversationListening",
                shortcut: "conversation",
                sample_count: 0,
                duration_ms: 0,
                sample_rate: None,
                channels: None,
                message: "Conversation Mode is listening for a follow-up…".to_string(),
                text: None,
                timestamp_ms: unix_timestamp_ms(),
            },
        );

        let started = Instant::now();
        let mut heard_speech = false;
        let mut last_voice = Instant::now();
        let timeout = Duration::from_secs(preferences.conversation_timeout_seconds);

        while started.elapsed() < timeout {
            let level = audio.current_level();

            if level >= 0.015 {
                heard_speech = true;
                last_voice = Instant::now();
            }

            if heard_speech && level < 0.008 && last_voice.elapsed() >= Duration::from_millis(900) {
                break;
            }

            thread::sleep(Duration::from_millis(100));
        }

        let _ = audio.stop_push_to_talk();

        if !heard_speech {
            let _ = audio.take_last_capture();
            emit_voice_capture_event(
                &app,
                VoiceCaptureEvent {
                    phase: "conversationTimeout",
                    shortcut: "conversation",
                    sample_count: 0,
                    duration_ms: started.elapsed().as_millis() as u64,
                    sample_rate: None,
                    channels: None,
                    message: "Conversation Mode timed out with no follow-up speech.".to_string(),
                    text: None,
                    timestamp_ms: unix_timestamp_ms(),
                },
            );
            return;
        }

        if let Some(capture) = audio.take_last_capture() {
            process_voice_capture(app.clone(), capture, "conversation");
        }
    });
}

fn extract_wake_command(transcript: &str, wake_phrase: &str) -> Option<String> {
    let transcript = transcript.trim();
    let wake_phrase = wake_phrase.trim();
    if transcript.is_empty() || wake_phrase.is_empty() {
        return None;
    }

    let transcript_lower = transcript.to_lowercase();
    let wake_lower = wake_phrase.to_lowercase();

    if !transcript_lower.starts_with(&wake_lower) {
        return None;
    }

    let remainder = transcript[wake_phrase.len()..]
        .trim_start_matches(|character: char| {
            character.is_whitespace()
                || matches!(character, ',' | ':' | ';' | '-' | '—' | '.' | '!')
        })
        .trim();

    Some(remainder.to_string())
}

fn spawn_wake_monitor(app: tauri::AppHandle, generation: u64) {
    thread::spawn(move || {
        loop {
            let state = app.state::<RuntimeState>();
            if state.wake_monitor_generation.load(Ordering::Relaxed) != generation {
                break;
            }

            let preferences = state
                .voice_preferences
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone();

            let voice_busy = !state
                .voice_command_ids
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .is_empty();

            if voice_busy {
                thread::sleep(Duration::from_millis(500));
                continue;
            }

            if !preferences.wake_word_enabled || runtime_snapshot(&state).paused {
                thread::sleep(Duration::from_millis(750));
                continue;
            }

            if app.state::<TtsRuntime>().is_speaking() {
                thread::sleep(Duration::from_millis(500));
                continue;
            }

            let audio = app.state::<AudioInputManager>();
            if audio.capture_active() {
                thread::sleep(Duration::from_millis(500));
                continue;
            }

            if audio.start_push_to_talk().is_err() {
                thread::sleep(Duration::from_secs(2));
                continue;
            }

            thread::sleep(Duration::from_millis(2200));

            if app
                .state::<RuntimeState>()
                .wake_monitor_generation
                .load(Ordering::Relaxed)
                != generation
            {
                break;
            }

            let _ = audio.stop_push_to_talk();

            let Some(capture) = audio.take_last_capture() else {
                thread::sleep(Duration::from_millis(600));
                continue;
            };

            if capture.rms() < 0.004 {
                thread::sleep(Duration::from_millis(600));
                continue;
            }

            let manager = app.state::<ModelManager>();
            let speech = app.state::<SpeechRuntime>();
            let Ok(result) = speech.transcribe(&app, &manager, capture) else {
                thread::sleep(Duration::from_secs(3));
                continue;
            };

            let Some(command) = extract_wake_command(&result.text, &preferences.wake_phrase) else {
                thread::sleep(Duration::from_millis(600));
                continue;
            };

            emit_voice_capture_event(
                &app,
                VoiceCaptureEvent {
                    phase: "wakeDetected",
                    shortcut: "wakePhrase",
                    sample_count: result.input_samples_16khz,
                    duration_ms: result.duration_ms,
                    sample_rate: Some(16_000),
                    channels: Some(1),
                    message: format!("Wake phrase “{}” detected.", preferences.wake_phrase),
                    text: Some(result.text.clone()),
                    timestamp_ms: unix_timestamp_ms(),
                },
            );

            if command.is_empty() {
                start_conversation_follow_up(app.clone(), true);
            } else {
                let state = app.state::<RuntimeState>();
                let _ = process_user_command(
                    app.clone(),
                    state,
                    CommandRequest {
                        text: command,
                        source: "voice".to_string(),
                        approval_id: None,
                    },
                );
            }

            thread::sleep(Duration::from_secs(1));
        }
    });
}

fn emit_core_event(app: &tauri::AppHandle, event: CoreEvent) {
    let should_speak = {
        let state = app.state::<RuntimeState>();
        let mut voice_ids = state
            .voice_command_ids
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let is_voice = voice_ids.contains(&event.id);
        if matches!(
            event.kind,
            "command.completed" | "command.failed" | "command.cancelled"
        ) {
            voice_ids.remove(&event.id);
        }
        is_voice && matches!(event.kind, "command.completed" | "command.failed")
    };

    if should_speak {
        let app_for_tts = app.clone();
        let text = event.message.clone();
        thread::spawn(move || {
            let manager = app_for_tts.state::<ModelManager>();
            let tts = app_for_tts.state::<TtsRuntime>();
            let preferences = app_for_tts
                .state::<RuntimeState>()
                .voice_preferences
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone();
            if preferences.auto_speak {
                let _ = tts.speak(
                    &app_for_tts,
                    &manager,
                    &text,
                    preferences.tts_speed,
                    &preferences.tts_voice_id,
                );
            }

            if preferences.conversation_mode {
                start_conversation_follow_up(app_for_tts.clone(), false);
            }
        });
    }

    let _ = app.emit("aura:core-event", event);
}

fn emit_core_error(app: &tauri::AppHandle, error: CoreError) {
    let _ = app.emit("aura:core-error", error);
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.hide();
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        emit_lifecycle_event(app, "foreground.entered", "AURA returned to the foreground.");
    }
}

fn hide_overlay_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.hide();
    }
}

fn toggle_overlay(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("overlay") else {
        return;
    };

    match window.is_visible() {
        Ok(true) => {
            let _ = window.hide();
        }
        Ok(false) => {
            let _ = window.show();
            let _ = window.set_focus();
        }
        Err(_) => {}
    }
}

fn aura_tray_icon() -> Image<'static> {
    const SIZE: u32 = 32;
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);

    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 - 15.5;
            let dy = y as f32 - 15.5;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= 12.5 {
                let t = x as f32 / (SIZE - 1) as f32;
                let r = (32.0 + (155.0 - 32.0) * t) as u8;
                let g = (199.0 + (92.0 - 199.0) * t) as u8;
                let b = 255u8;

                let inner = distance <= 4.0;
                if inner {
                    rgba.extend_from_slice(&[247, 249, 255, 255]);
                } else {
                    rgba.extend_from_slice(&[r, g, b, 255]);
                }
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    Image::new_owned(rgba, SIZE, SIZE)
}

#[tauri::command]
fn get_app_status() -> AppStatus {
    AppStatus {
        name: "AURA-2",
        version: env!("CARGO_PKG_VERSION"),
        stage: "M006 Complete · Voice · 0.6.0-alpha.1",
        local_first: true,
    }
}

#[tauri::command]
fn open_main_window(app: tauri::AppHandle) {
    show_main_window(&app);
}

#[tauri::command]
fn hide_overlay(app: tauri::AppHandle) {
    hide_overlay_window(&app);
}

#[tauri::command]
fn get_runtime_state(state: State<'_, RuntimeState>) -> RuntimeSnapshot {
    runtime_snapshot(&state)
}

#[tauri::command]
fn set_runtime_paused(
    app: tauri::AppHandle,
    paused: bool,
) -> RuntimeSnapshot {
    set_paused_state(&app, paused)
}

#[tauri::command]
fn set_background_enabled(
    app: tauri::AppHandle,
    background_enabled: bool,
) -> Result<RuntimeSnapshot, String> {
    set_background_state(&app, background_enabled)
}

#[tauri::command]
fn set_autostart_enabled(
    app: tauri::AppHandle,
    autostart_enabled: bool,
) -> Result<RuntimeSnapshot, String> {
    set_autostart_state(&app, autostart_enabled)
}

#[tauri::command]
fn get_permission_policy(state: State<'_, RuntimeState>) -> PermissionPolicy {
    state
        .permission_policy
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

#[tauri::command]
fn set_permission_decision(
    app: tauri::AppHandle,
    state: State<'_, RuntimeState>,
    class: PermissionClass,
    decision: PermissionDecision,
) -> Result<PermissionPolicy, String> {
    let mut next = state
        .permission_policy
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();

    next.set(class, decision).map_err(str::to_string)?;
    save_permission_policy(&app, &next)?;

    {
        let mut current = state
            .permission_policy
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = next.clone();
    }

    emit_lifecycle_event(
        &app,
        "permissions.updated",
        "AURA permission policy updated.",
    );

    Ok(next)
}

#[tauri::command]
fn reset_permission_policy(
    app: tauri::AppHandle,
    state: State<'_, RuntimeState>,
) -> Result<PermissionPolicy, String> {
    let next = PermissionPolicy::default();
    save_permission_policy(&app, &next)?;

    {
        let mut current = state
            .permission_policy
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = next.clone();
    }

    emit_lifecycle_event(
        &app,
        "permissions.reset",
        "AURA permission policy restored to safe defaults.",
    );

    Ok(next)
}

#[tauri::command]
fn resolve_confirmation(
    app: tauri::AppHandle,
    state: State<'_, RuntimeState>,
    id: String,
    approved: bool,
) -> Result<Option<CommandAck>, String> {
    let pending = {
        let confirmations = state
            .pending_confirmations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        confirmations.get(&id).cloned()
    };

    let Some(pending) = pending else {
        emit_core_event(
            &app,
            CoreEvent {
                id: id.clone(),
                kind: "command.failed",
                status: AuraRuntimeStatus::Idle,
                message: "Confirmation is no longer available or has expired.".to_string(),
                command: None,
                timestamp_ms: unix_timestamp_ms(),
            },
        );
        return Err("Confirmation is no longer available or has expired.".to_string());
    };

    if pending.expires_at_ms <= unix_timestamp_ms() {
        {
            let mut confirmations = state
                .pending_confirmations
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            confirmations.remove(&id);
        }

        emit_core_event(
            &app,
            CoreEvent {
                id: id.clone(),
                kind: "command.failed",
                status: AuraRuntimeStatus::Idle,
                message: "Confirmation expired. Submit the command again.".to_string(),
                command: Some(pending.command.clone()),
                timestamp_ms: unix_timestamp_ms(),
            },
        );

        return Err("Confirmation expired. Submit the command again.".to_string());
    }

    if !approved {
        {
            let mut confirmations = state
                .pending_confirmations
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            confirmations.remove(&id);
        }

        emit_core_event(
            &app,
            CoreEvent {
                id,
                kind: "command.cancelled",
                status: AuraRuntimeStatus::Idle,
                message: "Action cancelled.".to_string(),
                command: Some(pending.command),
                timestamp_ms: unix_timestamp_ms(),
            },
        );

        return Ok(None);
    }

    let ack = process_user_command(
        app,
        state,
        CommandRequest {
            text: pending.command,
            source: pending.source,
            approval_id: Some(id),
        },
    )?;

    Ok(Some(ack))
}

#[tauri::command]
fn process_user_command(
    app: tauri::AppHandle,
    state: State<'_, RuntimeState>,
    request: CommandRequest,
) -> Result<CommandAck, String> {
    if runtime_snapshot(&state).paused {
        let error = CoreError {
            id: None,
            code: "runtime.paused",
            message: "AURA is paused. Resume it from the system tray before sending commands."
                .to_string(),
        };
        emit_core_error(&app, error);
        return Err("AURA is paused.".to_string());
    }

    let text = request.text.trim().to_string();

    if text.is_empty() {
        let error = CoreError {
            id: None,
            code: "command.empty",
            message: "AURA received an empty command.".to_string(),
        };
        emit_core_error(&app, error);
        return Err("Command cannot be empty.".to_string());
    }

    if text.chars().count() > 4000 {
        let error = CoreError {
            id: None,
            code: "command.too_long",
            message: "That command is too long for the desktop command bridge.".to_string(),
        };
        emit_core_error(&app, error);
        return Err("Command exceeds the 4000-character limit.".to_string());
    }

    if !matches!(request.source.as_str(), "desktop" | "overlay" | "voice") {
        let error = CoreError {
            id: None,
            code: "command.invalid_source",
            message: "AURA received a command from an unknown source.".to_string(),
        };
        emit_core_error(&app, error);
        return Err("Unknown command source.".to_string());
    }

    let source = request.source.clone();
    let approval_id = request.approval_id.clone();
    let id = approval_id.clone().unwrap_or_else(next_command_id);

    if source == "voice" {
        state
            .voice_command_ids
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(id.clone());
    }

    let approved_permission = if let Some(approval_id) = &approval_id {
        let pending = {
            let mut confirmations = state
                .pending_confirmations
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            confirmations.remove(approval_id)
        };

        let Some(pending) = pending else {
            return Err("Confirmation is no longer available or has expired.".to_string());
        };

        Some(
            validate_pending_confirmation(
                &pending,
                &text,
                &source,
                unix_timestamp_ms(),
            )
            .map_err(str::to_string)?,
        )
    } else {
        None
    };

    emit_core_event(
        &app,
        CoreEvent {
            id: id.clone(),
            kind: if approval_id.is_some() {
                "command.confirmed"
            } else {
                "command.accepted"
            },
            status: AuraRuntimeStatus::Thinking,
            message: if approval_id.is_some() {
                "Confirmation accepted. Executing action…".to_string()
            } else {
                format!("Understanding: “{}”", text)
            },
            command: Some(text.clone()),
            timestamp_ms: unix_timestamp_ms(),
        },
    );

    let policy = state
        .permission_policy
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();

    let base_route = route_command(&text, &policy);
    let routine_route = route_user_routine(&app, &text, &policy);
    let director_route = route_director_preset(&app, &text, &policy);

    let routed = match routine_route {
        Some(routine_route) if should_prefer_user_routine(&text, &base_route) => routine_route,
        _ => match director_route {
            Some(preset_route) if should_prefer_director_preset(&text, &base_route) => preset_route,
            _ => base_route,
        },
    };

    match routed {
        RouteResult::Action(action) => {
            let decision = if let Some(permission) = approved_permission {
                if permission != action.permission {
                    emit_core_event(
                        &app,
                        CoreEvent {
                            id: id.clone(),
                            kind: "command.failed",
                            status: AuraRuntimeStatus::Idle,
                            message: "Confirmation no longer matches the routed action.".to_string(),
                            command: Some(text.clone()),
                            timestamp_ms: unix_timestamp_ms(),
                        },
                    );
                    return Err("Confirmation no longer matches the routed action.".to_string());
                }

                match action.decision {
                    PermissionDecision::Never => PermissionDecision::Never,
                    PermissionDecision::Allow | PermissionDecision::Ask => {
                        PermissionDecision::Allow
                    }
                }
            } else {
                action.decision
            };

            match decision {
            PermissionDecision::Allow => {
                let worker_app = app.clone();
                let worker_id = id.clone();
                let worker_text = text.clone();
                let worker_source = source.clone();
                let worker_permission = action.permission;

                thread::spawn(move || {
                    match action.intent {
                        ActionIntent::LaunchApp(target) => {
                            let display_name = target.display_name();

                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: format!("Opening {}…", display_name),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            match launch_app(target) {
                                Ok(()) => {
                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: format!("Opened {}.", display_name),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message = format!(
                                        "Could not open {}: {}",
                                        display_name, error
                                    );

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "computer.app_launch_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::SwitchToApp(target) => {
                            let display_name = target.display_name();

                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: format!("Switching to {}…", display_name),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            match switch_to_app(target) {
                                Ok(window) => {
                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: format!("Switched to {} — {}.", display_name, window.title),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message = format!("Could not switch to {}: {}", display_name, error);

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "computer.window_switch_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::ListWindows => {
                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: "Reading visible desktop windows…".to_string(),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            match summarize_windows(8) {
                                Ok(summary) => {
                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: summary,
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message = format!("Could not read visible windows: {}", error);

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "computer.window_discovery_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::PressShortcut(shortcut) => {
                            let display_name = shortcut.display_name();

                            if worker_source != "overlay" {
                                let message = format!(
                                    "Keyboard action {} was not sent. Use the AURA Overlay so input returns to the app you were using.",
                                    display_name
                                );

                                emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id.clone(),
                                        kind: "command.failed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: message.clone(),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                );

                                emit_core_error(
                                    &worker_app,
                                    CoreError {
                                        id: Some(worker_id),
                                        code: "computer.keyboard_requires_overlay",
                                        message,
                                    },
                                );
                                return;
                            }

                            hide_overlay_window(&worker_app);
                            thread::sleep(Duration::from_millis(90));

                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: format!("Pressing {}…", display_name),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            match press_shortcut(&shortcut) {
                                Ok(()) => emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id,
                                        kind: "command.completed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: format!("Pressed {}.", display_name),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                ),
                                Err(error) => {
                                    let message = format!(
                                        "Could not press {}: {}",
                                        display_name, error
                                    );

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "computer.keyboard_input_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::TypeText(value) => {
                            if worker_source != "overlay" {
                                let message =
                                    "Text input was not sent. Use the AURA Overlay so typing returns to the app you were using."
                                        .to_string();

                                emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id.clone(),
                                        kind: "command.failed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: message.clone(),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                );

                                emit_core_error(
                                    &worker_app,
                                    CoreError {
                                        id: Some(worker_id),
                                        code: "computer.keyboard_requires_overlay",
                                        message,
                                    },
                                );
                                return;
                            }

                            hide_overlay_window(&worker_app);
                            thread::sleep(Duration::from_millis(90));

                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: "Typing approved text…".to_string(),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            match type_text(&value) {
                                Ok(()) => emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id,
                                        kind: "command.completed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: format!(
                                            "Typed {} characters.",
                                            value.chars().count()
                                        ),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                ),
                                Err(error) => {
                                    let message = format!("Could not type text: {}", error);

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "computer.keyboard_input_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::Mouse(action) => {
                            let needs_overlay = matches!(
                                action,
                                MouseAction::Scroll { .. }
                                    | MouseAction::Click { .. }
                                    | MouseAction::DoubleClick
                                    | MouseAction::ClickAt { .. }
                            );

                            if needs_overlay && worker_source != "overlay" {
                                let message =
                                    "This mouse action was not sent. Use the AURA Overlay so the action targets the app you were using."
                                        .to_string();

                                emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id.clone(),
                                        kind: "command.failed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: message.clone(),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                );

                                emit_core_error(
                                    &worker_app,
                                    CoreError {
                                        id: Some(worker_id),
                                        code: "computer.mouse_requires_overlay",
                                        message,
                                    },
                                );
                                return;
                            }

                            if worker_source == "overlay" {
                                hide_overlay_window(&worker_app);
                                thread::sleep(Duration::from_millis(90));
                            }

                            let action_label = match action {
                                MouseAction::MoveTo(point) => {
                                    format!("Moving pointer to ({}, {})…", point.x, point.y)
                                }
                                MouseAction::Scroll { notches } if notches < 0 => {
                                    format!("Scrolling down {}…", notches.abs())
                                }
                                MouseAction::Scroll { notches } => {
                                    format!("Scrolling up {}…", notches)
                                }
                                MouseAction::Click { .. } => "Clicking…".to_string(),
                                MouseAction::DoubleClick => "Double-clicking…".to_string(),
                                MouseAction::ClickAt { point, .. } => {
                                    format!("Clicking at ({}, {})…", point.x, point.y)
                                }
                            };

                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: action_label,
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            match execute_mouse_action(action) {
                                Ok(()) => emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id,
                                        kind: "command.completed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: "Mouse action completed.".to_string(),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                ),
                                Err(error) => {
                                    let message = format!("Could not perform mouse action: {}", error);

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "computer.mouse_input_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::Media(action) => {
                            let message = match action {
                                MediaAction::GetVolume => "Reading Windows audio state…".to_string(),
                                MediaAction::SetVolume(percent) => {
                                    format!("Setting volume to {}%…", percent)
                                }
                                MediaAction::VolumeUp => "Increasing volume…".to_string(),
                                MediaAction::VolumeDown => "Decreasing volume…".to_string(),
                                MediaAction::Mute => "Muting audio…".to_string(),
                                MediaAction::Unmute => "Unmuting audio…".to_string(),
                                MediaAction::PlayPause => "Toggling media playback…".to_string(),
                                MediaAction::NextTrack => "Skipping to next track…".to_string(),
                                MediaAction::PreviousTrack => "Going to previous track…".to_string(),
                                MediaAction::Stop => "Stopping media playback…".to_string(),
                            };

                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message,
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            match execute_media_action(action) {
                                Ok(Some(state)) => {
                                    let mute_text = if state.muted { " · muted" } else { "" };
                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: format!(
                                                "Windows volume: {}%{}.",
                                                state.volume_percent,
                                                mute_text
                                            ),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Ok(None) => emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id,
                                        kind: "command.completed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: "Media control sent.".to_string(),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                ),
                                Err(error) => {
                                    let message = format!("Could not control Windows audio: {}", error);

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "computer.audio_control_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::System(action) => {
                            let activity = match action {
                                SystemAction::GetStatus => "Reading Windows system status…",
                                SystemAction::GetBattery => "Reading battery status…",
                                SystemAction::ShowDesktop => "Showing desktop…",
                                SystemAction::OpenTaskManager => "Opening Task Manager…",
                                SystemAction::OpenSettings(_) => "Opening Windows Settings…",
                                SystemAction::Lock => "Locking Windows…",
                                SystemAction::Sleep => "Putting Windows to sleep…",
                                SystemAction::Restart => "Restarting Windows…",
                                SystemAction::Shutdown => "Shutting down Windows…",
                            };

                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: activity.to_string(),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            match execute_system_action(action) {
                                Ok(Some(snapshot)) => {
                                    let battery_only = matches!(action, SystemAction::GetBattery);
                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: summarize_system(&snapshot, battery_only),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Ok(None) => {
                                    let completed = match action {
                                        SystemAction::ShowDesktop => "Desktop shown.",
                                        SystemAction::OpenTaskManager => "Task Manager opened.",
                                        SystemAction::OpenSettings(_) => "Windows Settings opened.",
                                        SystemAction::Lock => "Windows lock requested.",
                                        SystemAction::Sleep => "Windows sleep requested.",
                                        SystemAction::Restart => "Windows restart requested.",
                                        SystemAction::Shutdown => "Windows shutdown requested.",
                                        SystemAction::GetStatus | SystemAction::GetBattery => {
                                            "System command completed."
                                        }
                                    };

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: completed.to_string(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message = format!("Could not run Windows system command: {}", error);

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "computer.system_command_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::ObsAudioMute {
                            input_name,
                            muted,
                        } => {
                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: format!(
                                        "{} OBS input {}…",
                                        if muted { "Muting" } else { "Unmuting" },
                                        input_name
                                    ),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            let result = tauri::async_runtime::block_on(async {
                                let obs = worker_app.state::<ObsController>();
                                obs.set_audio_muted_by_name(&input_name, muted).await
                            });

                            match result {
                                Ok(changed) => emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id,
                                        kind: "command.completed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: format!(
                                            "OBS input {} is now {}.",
                                            changed.input_name,
                                            if changed.muted { "muted" } else { "unmuted" }
                                        ),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                ),
                                Err(error) => {
                                    let message = format!(
                                        "Could not change OBS input mute state: {}",
                                        error
                                    );

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "obs.audio_mute_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::ObsAudioVolume {
                            input_name,
                            percent,
                        } => {
                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: format!(
                                        "Setting OBS input {} to {}%…",
                                        input_name, percent
                                    ),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            let result = tauri::async_runtime::block_on(async {
                                let obs = worker_app.state::<ObsController>();
                                obs.set_audio_volume_by_name(&input_name, percent).await
                            });

                            match result {
                                Ok(changed) => emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id,
                                        kind: "command.completed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: format!(
                                            "OBS input {} volume set to {}%.",
                                            changed.input_name,
                                            changed.volume_percent
                                        ),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                ),
                                Err(error) => {
                                    let message = format!(
                                        "Could not change OBS input volume: {}",
                                        error
                                    );

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "obs.audio_volume_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::ObsSourceVisibility {
                            source_name,
                            enabled,
                        } => {
                            let verb = if enabled { "Showing" } else { "Hiding" };

                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: format!("{verb} OBS source {source_name}…"),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            let result = tauri::async_runtime::block_on(async {
                                let obs = worker_app.state::<ObsController>();
                                obs.set_source_visibility_by_name(&source_name, enabled).await
                            });

                            match result {
                                Ok(changed) => emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id,
                                        kind: "command.completed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: format!(
                                            "{} {} in OBS Program scene {}.",
                                            if changed.enabled { "Showing" } else { "Hidden" },
                                            changed.source_name,
                                            changed.scene_name
                                        ),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                ),
                                Err(error) => {
                                    let message = format!(
                                        "Could not change OBS source visibility: {}",
                                        error
                                    );

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "obs.source_visibility_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::CurrentApp => {
                            let result = {
                                let awareness = worker_app.state::<CurrentAppAwareness>();
                                awareness.snapshot()
                            };

                            match result {
                                Ok(context) => {
                                    let qualifier = if context.context_source == "lastExternal" {
                                        " (last external app before AURA took focus)"
                                    } else {
                                        ""
                                    };

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: format!(
                                                "Current app: {} · {}{}.",
                                                context.app_name,
                                                context.process_name,
                                                qualifier
                                            ),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message =
                                        format!("Could not detect the current app: {error}");

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "context.current_app_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::ActiveWindow => {
                            let result = {
                                let awareness = worker_app.state::<CurrentAppAwareness>();
                                awareness.snapshot()
                            };

                            match result {
                                Ok(context) => {
                                    let qualifier = if context.context_source == "lastExternal" {
                                        " (last external window before AURA took focus)"
                                    } else {
                                        ""
                                    };

                                    let message = match context.window_title.as_deref() {
                                        Some(title) => format!(
                                            "Active window: {} · {}{}.",
                                            title, context.app_name, qualifier
                                        ),
                                        None => format!(
                                            "Current app: {}. Windows did not expose a title for its active window{}.",
                                            context.app_name, qualifier
                                        ),
                                    };

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message,
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message =
                                        format!("Could not detect the active window: {error}");

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "context.active_window_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::RecentFiles => {
                            match summarize_recent_files(8) {
                                Ok(summary) => {
                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: summary,
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message =
                                        format!("Could not read recent files: {error}");

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "context.recent_files_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::UserRoutine(routine_id) => {
                            let routine = find_routine_by_id(&worker_app, &routine_id);
                            match routine {
                                Some(routine) => {
                                    let became_sensitive =
                                        routine_requires_sensitive_permission(&worker_app, &routine);
                                    if became_sensitive
                                        && worker_permission != PermissionClass::Sensitive
                                    {
                                        let message =
                                            "Routine changed and now contains a Sensitive action. Submit it again so AURA can request confirmation."
                                                .to_string();

                                        emit_core_event(
                                            &worker_app,
                                            CoreEvent {
                                                id: worker_id.clone(),
                                                kind: "command.failed",
                                                status: AuraRuntimeStatus::Idle,
                                                message: message.clone(),
                                                command: Some(worker_text),
                                                timestamp_ms: unix_timestamp_ms(),
                                            },
                                        );

                                        emit_core_error(
                                            &worker_app,
                                            CoreError {
                                                id: Some(worker_id),
                                                code: "routine.permission_changed",
                                                message,
                                            },
                                        );
                                        return;
                                    }

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.processing",
                                            status: AuraRuntimeStatus::Working,
                                            message: format!(
                                                "Running routine {}…",
                                                routine.name
                                            ),
                                            command: Some(worker_text.clone()),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    let result = tauri::async_runtime::block_on(async {
                                        let obs = worker_app.state::<ObsController>();
                                        run_routine(&worker_app, &obs, &routine).await
                                    });

                                    let kind = if result.success {
                                        "command.completed"
                                    } else {
                                        "command.failed"
                                    };
                                    let message = if result.success {
                                        format!(
                                            "Routine {} completed ({} steps).",
                                            result.routine_name, result.completed_steps
                                        )
                                    } else {
                                        format!(
                                            "Routine {} failed at step {}: {}",
                                            result.routine_name,
                                            result.failed_step.unwrap_or(0) + 1,
                                            result
                                                .error
                                                .clone()
                                                .unwrap_or_else(|| "Unknown error.".to_string())
                                        )
                                    };

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind,
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    if !result.success {
                                        emit_core_error(
                                            &worker_app,
                                            CoreError {
                                                id: Some(worker_id),
                                                code: "routine.execution_failed",
                                                message,
                                            },
                                        );
                                    }
                                }
                                None => {
                                    let message = "Routine no longer exists.".to_string();
                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "routine.not_found",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::MemoryRemember(content) => {
                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: "Saving explicit local memory…".to_string(),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            match create_memory(
                                &worker_app,
                                CreateMemoryRequest {
                                    content: content.clone(),
                                },
                                "command",
                            ) {
                                Ok(result) => {
                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: if result.created {
                                                format!("Remembered: {}.", result.record.content)
                                            } else {
                                                format!(
                                                    "I already remembered that. Refreshed: {}.",
                                                    result.record.content
                                                )
                                            },
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message = format!("Could not save memory: {error}");
                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "memory.create_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::MemoryList => {
                            match summarize_memories(&worker_app, 8) {
                                Ok(summary) => emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id,
                                        kind: "command.completed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: summary,
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                ),
                                Err(error) => {
                                    let message = format!("Could not read local memory: {error}");
                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "memory.read_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::MemoryForget(content) => {
                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: "Removing explicit local memory…".to_string(),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            match delete_memory_by_content(&worker_app, &content) {
                                Ok(record) => {
                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: format!("Forgot: {}.", record.content),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message = format!("Could not forget memory: {error}");
                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "memory.delete_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                    ActionIntent::DirectorPreset(preset_id) => {
                            let Some(preset) = find_director_preset_by_id(&worker_app, &preset_id) else {
                                let message = "Director Mode preset no longer exists.".to_string();

                                emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id.clone(),
                                        kind: "command.failed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: message.clone(),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                );

                                emit_core_error(
                                    &worker_app,
                                    CoreError {
                                        id: Some(worker_id),
                                        code: "director.preset_missing",
                                        message,
                                    },
                                );
                                return;
                            };

                            if preset_requires_sensitive_permission(&preset)
                                && worker_permission != PermissionClass::Sensitive
                            {
                                let message = "Preset changed to require Sensitive permission. Run the command again so AURA can request confirmation.".to_string();

                                emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id.clone(),
                                        kind: "command.failed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: message.clone(),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                );

                                emit_core_error(
                                    &worker_app,
                                    CoreError {
                                        id: Some(worker_id),
                                        code: "director.permission_changed",
                                        message,
                                    },
                                );
                                return;
                            }

                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: format!(
                                        "Running Director Mode preset {}…",
                                        preset.name
                                    ),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            let result = tauri::async_runtime::block_on(async {
                                let obs = worker_app.state::<ObsController>();
                                run_director_preset(&obs, &preset).await
                            });

                            if result.success {
                                emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id,
                                        kind: "command.completed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: format!(
                                            "Director Mode preset {} completed: {}/{} steps.",
                                            result.preset_name,
                                            result.completed_steps,
                                            result.total_steps
                                        ),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                );
                            } else {
                                let message = format!(
                                    "Director Mode preset {} stopped at step {}: {}",
                                    result.preset_name,
                                    result.failed_step.map(|step| step + 1).unwrap_or(0),
                                    result.error.as_deref().unwrap_or("Unknown preset error.")
                                );

                                emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id.clone(),
                                        kind: "command.failed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: message.clone(),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                );

                                emit_core_error(
                                    &worker_app,
                                    CoreError {
                                        id: Some(worker_id),
                                        code: "director.preset_failed",
                                        message,
                                    },
                                );
                            }
                        }
                    ActionIntent::ObsProductionHealth => {
                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: "Checking OBS production health…".to_string(),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            let result = tauri::async_runtime::block_on(async {
                                let obs = worker_app.state::<ObsController>();
                                obs.production_health().await
                            });

                            match result {
                                Ok(health) => {
                                    let message = if health.issues.is_empty() {
                                        format!(
                                            "Production health: {}. {:.1} FPS, OBS CPU {:.1}%, no frame-loss warnings detected.",
                                            health.status.to_uppercase(),
                                            health.active_fps,
                                            health.cpu_usage_percent
                                        )
                                    } else {
                                        format!(
                                            "Production health: {}. {}",
                                            health.status.to_uppercase(),
                                            health.issues.join(" ")
                                        )
                                    };

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message,
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message =
                                        format!("Could not check OBS production health: {error}");

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "obs.production_health_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::ObsStreamDuration => {
                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: "Reading OBS stream duration…".to_string(),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            let result = tauri::async_runtime::block_on(async {
                                let obs = worker_app.state::<ObsController>();
                                obs.stream_duration().await
                            });

                            match result {
                                Ok(duration) => {
                                    let message = if duration.streaming {
                                        format!(
                                            "We have been live for {}.",
                                            format_duration_ms(duration.duration_ms)
                                        )
                                    } else {
                                        "OBS is not currently live.".to_string()
                                    };

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message,
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message = format!(
                                        "Could not read OBS stream duration: {}",
                                        error
                                    );

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "obs.stream_duration_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::ObsStreaming(action) => {
                            let action_label = match action {
                                ObsStreamingAction::Start => "Starting OBS stream…",
                                ObsStreamingAction::Stop => "Stopping OBS stream…",
                            };

                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: action_label.to_string(),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            let result = tauri::async_runtime::block_on(async {
                                let obs = worker_app.state::<ObsController>();
                                match action {
                                    ObsStreamingAction::Start => obs.start_streaming().await,
                                    ObsStreamingAction::Stop => obs.stop_streaming().await,
                                }
                            });

                            match result {
                                Ok(_) => {
                                    let message = match action {
                                        ObsStreamingAction::Start => {
                                            "OBS stream is live.".to_string()
                                        }
                                        ObsStreamingAction::Stop => {
                                            "OBS stream stopped.".to_string()
                                        }
                                    };

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message,
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message = format!(
                                        "Could not control OBS streaming: {}",
                                        error
                                    );

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "obs.streaming_control_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::ObsRecording(action) => {
                            let action_label = match action {
                                ObsRecordingAction::Start => "Starting OBS recording…",
                                ObsRecordingAction::Stop => "Stopping OBS recording…",
                                ObsRecordingAction::Pause => "Pausing OBS recording…",
                                ObsRecordingAction::Resume => "Resuming OBS recording…",
                            };

                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: action_label.to_string(),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            let result = tauri::async_runtime::block_on(async {
                                let obs = worker_app.state::<ObsController>();
                                match action {
                                    ObsRecordingAction::Start => obs.start_recording().await,
                                    ObsRecordingAction::Stop => obs.stop_recording().await,
                                    ObsRecordingAction::Pause => obs.pause_recording().await,
                                    ObsRecordingAction::Resume => obs.resume_recording().await,
                                }
                            });

                            match result {
                                Ok(recording) => {
                                    let message = match action {
                                        ObsRecordingAction::Start => {
                                            "OBS recording started.".to_string()
                                        }
                                        ObsRecordingAction::Stop => recording
                                            .output_path
                                            .as_deref()
                                            .map(|path| {
                                                format!("OBS recording stopped. Saved to {}.", path)
                                            })
                                            .unwrap_or_else(|| "OBS recording stopped.".to_string()),
                                        ObsRecordingAction::Pause => {
                                            "OBS recording paused.".to_string()
                                        }
                                        ObsRecordingAction::Resume => {
                                            "OBS recording resumed.".to_string()
                                        }
                                    };

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message,
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message = format!(
                                        "Could not control OBS recording: {}",
                                        error
                                    );

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "obs.recording_control_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::ObsProgramScene(scene_name) => {
                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: format!("Switching OBS Program to {}…", scene_name),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            let result = tauri::async_runtime::block_on(async {
                                let obs = worker_app.state::<ObsController>();
                                obs.set_program_scene_by_name(&scene_name).await
                            });

                            match result {
                                Ok(switched) => emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id,
                                        kind: "command.completed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: format!(
                                            "OBS Program switched to {}.",
                                            switched.scene_name
                                        ),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                ),
                                Err(error) => {
                                    let message = format!(
                                        "Could not switch OBS Program to {}: {}",
                                        scene_name, error
                                    );

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "obs.scene_switch_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::ObsPreviewScene(scene_name) => {
                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: format!("Setting OBS Preview to {}…", scene_name),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            let result = tauri::async_runtime::block_on(async {
                                let obs = worker_app.state::<ObsController>();
                                obs.set_preview_scene_by_name(&scene_name).await
                            });

                            match result {
                                Ok(switched) => emit_core_event(
                                    &worker_app,
                                    CoreEvent {
                                        id: worker_id,
                                        kind: "command.completed",
                                        status: AuraRuntimeStatus::Idle,
                                        message: format!(
                                            "OBS Preview switched to {}.",
                                            switched.scene_name
                                        ),
                                        command: Some(worker_text),
                                        timestamp_ms: unix_timestamp_ms(),
                                    },
                                ),
                                Err(error) => {
                                    let message = format!(
                                        "Could not switch OBS Preview to {}: {}",
                                        scene_name, error
                                    );

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "obs.preview_switch_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                        ActionIntent::CloseApp(target) => {
                            let display_name = target.display_name();

                            emit_core_event(
                                &worker_app,
                                CoreEvent {
                                    id: worker_id.clone(),
                                    kind: "command.processing",
                                    status: AuraRuntimeStatus::Working,
                                    message: format!("Closing {}…", display_name),
                                    command: Some(worker_text.clone()),
                                    timestamp_ms: unix_timestamp_ms(),
                                },
                            );

                            match close_app(target) {
                                Ok(()) => {
                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id,
                                            kind: "command.completed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: format!("Closed {}.", display_name),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );
                                }
                                Err(error) => {
                                    let message = format!(
                                        "Could not close {}: {}",
                                        display_name, error
                                    );

                                    emit_core_event(
                                        &worker_app,
                                        CoreEvent {
                                            id: worker_id.clone(),
                                            kind: "command.failed",
                                            status: AuraRuntimeStatus::Idle,
                                            message: message.clone(),
                                            command: Some(worker_text),
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    );

                                    emit_core_error(
                                        &worker_app,
                                        CoreError {
                                            id: Some(worker_id),
                                            code: "computer.app_close_failed",
                                            message,
                                        },
                                    );
                                }
                            }
                        }
                    }
                });
            }
            PermissionDecision::Ask => {
                let message = match &action.intent {
                    ActionIntent::CloseApp(target) => format!(
                        "Closing {} requires confirmation because unsaved work could be lost.",
                        target.display_name()
                    ),
                    ActionIntent::LaunchApp(target) => format!(
                        "Opening {} requires confirmation under the current permission policy.",
                        target.display_name()
                    ),
                    ActionIntent::SwitchToApp(target) => format!(
                        "Switching to {} requires confirmation under the current permission policy.",
                        target.display_name()
                    ),
                    ActionIntent::ListWindows => {
                        "Reading visible windows requires confirmation under the current permission policy."
                            .to_string()
                    }
                    ActionIntent::ObsAudioMute {
                        input_name,
                        muted,
                    } => format!(
                        "{} OBS input {} requires confirmation under the current permission policy.",
                        if *muted { "Muting" } else { "Unmuting" },
                        input_name
                    ),
                    ActionIntent::ObsAudioVolume {
                        input_name,
                        percent,
                    } => format!(
                        "Setting OBS input {} to {}% requires confirmation under the current permission policy.",
                        input_name,
                        percent
                    ),
                    ActionIntent::ObsSourceVisibility {
                        source_name,
                        enabled,
                    } => format!(
                        "{} OBS source {} requires confirmation under the current permission policy.",
                        if *enabled { "Showing" } else { "Hiding" },
                        source_name
                    ),
                    ActionIntent::CurrentApp => {
                        "Reading the current application requires confirmation under the current permission policy."
                            .to_string()
                    }
                    ActionIntent::ActiveWindow => {
                        "Reading the active window title requires confirmation under the current permission policy."
                            .to_string()
                    }
                    ActionIntent::RecentFiles => {
                        "Reading Windows Recent Items requires confirmation under the current permission policy."
                            .to_string()
                    }
                    ActionIntent::UserRoutine(routine_id) => format!(
                        "Running routine {} requires confirmation under the current permission policy.",
                        routine_id
                    ),
                    ActionIntent::MemoryRemember(content) => format!(
                        "Saving “{}” to AURA's local memory requires confirmation under the current permission policy.",
                        content
                    ),
                    ActionIntent::MemoryList => {
                        "Reading AURA's local memory requires confirmation under the current permission policy."
                            .to_string()
                    }
                    ActionIntent::MemoryForget(content) => format!(
                        "Deleting the saved memory “{}” is destructive and requires confirmation.",
                        content
                    ),
                    ActionIntent::DirectorPreset(preset_id) => {
                        let preset_name = find_director_preset_by_id(&app, preset_id)
                            .map(|preset| preset.name)
                            .unwrap_or_else(|| preset_id.clone());
                        format!(
                            "Running Director Mode preset {} requires confirmation under the current permission policy.",
                            preset_name
                        )
                    }
                    ActionIntent::ObsProductionHealth => {
                        "Reading OBS production health requires confirmation under the current permission policy."
                            .to_string()
                    }
                    ActionIntent::ObsStreamDuration => {
                        "Reading OBS stream duration requires confirmation under the current permission policy."
                            .to_string()
                    }
                    ActionIntent::ObsStreaming(action) => match action {
                        ObsStreamingAction::Start => {
                            "Going live can publish audio/video externally. Confirm before starting the OBS stream."
                                .to_string()
                        }
                        ObsStreamingAction::Stop => {
                            "Stopping the OBS stream requires confirmation under the current permission policy."
                                .to_string()
                        }
                    },
                    ActionIntent::ObsRecording(action) => match action {
                        ObsRecordingAction::Start => {
                            "Starting OBS recording requires confirmation under the current permission policy."
                                .to_string()
                        }
                        ObsRecordingAction::Stop => {
                            "Stopping OBS recording requires confirmation under the current permission policy."
                                .to_string()
                        }
                        ObsRecordingAction::Pause => {
                            "Pausing OBS recording requires confirmation under the current permission policy."
                                .to_string()
                        }
                        ObsRecordingAction::Resume => {
                            "Resuming OBS recording requires confirmation under the current permission policy."
                                .to_string()
                        }
                    },
                    ActionIntent::ObsProgramScene(scene) => format!(
                        "Switching OBS Program to {} requires confirmation under the current permission policy.",
                        scene
                    ),
                    ActionIntent::ObsPreviewScene(scene) => format!(
                        "Setting OBS Preview to {} requires confirmation under the current permission policy.",
                        scene
                    ),
                    ActionIntent::PressShortcut(shortcut) => format!(
                        "Pressing {} requires confirmation because keyboard input can change application state.",
                        shortcut.display_name()
                    ),
                    ActionIntent::TypeText(value) => format!(
                        "Typing {} characters requires confirmation before AURA sends text to another application.",
                        value.chars().count()
                    ),
                    ActionIntent::Mouse(MouseAction::Click { .. }) => {
                        "Clicking requires confirmation because it can activate controls or submit actions."
                            .to_string()
                    }
                    ActionIntent::Mouse(MouseAction::DoubleClick) => {
                        "Double-clicking requires confirmation because it can open or activate content."
                            .to_string()
                    }
                    ActionIntent::Mouse(MouseAction::ClickAt { point, .. }) => format!(
                        "Clicking at ({}, {}) requires confirmation because it can activate a UI control.",
                        point.x, point.y
                    ),
                    ActionIntent::Mouse(MouseAction::MoveTo(_))
                    | ActionIntent::Mouse(MouseAction::Scroll { .. }) => {
                        "This mouse action requires confirmation under the current permission policy."
                            .to_string()
                    }
                    ActionIntent::Media(MediaAction::GetVolume) => {
                        "Reading audio state requires confirmation under the current permission policy."
                            .to_string()
                    }
                    ActionIntent::Media(_) => {
                        "This audio/media action requires confirmation under the current permission policy."
                            .to_string()
                    }
                    ActionIntent::System(SystemAction::Lock) => {
                        "Locking Windows requires confirmation because it immediately ends access to the current interactive session."
                            .to_string()
                    }
                    ActionIntent::System(SystemAction::Sleep) => {
                        "Putting the PC to sleep requires confirmation."
                            .to_string()
                    }
                    ActionIntent::System(SystemAction::Restart) => {
                        "Restarting the PC requires confirmation because open work may be lost."
                            .to_string()
                    }
                    ActionIntent::System(SystemAction::Shutdown) => {
                        "Shutting down the PC requires confirmation because open work may be lost."
                            .to_string()
                    }
                    ActionIntent::System(_) => {
                        "This Windows system action requires confirmation under the current permission policy."
                            .to_string()
                    }
                };

                let expires_at_ms = unix_timestamp_ms() + CONFIRMATION_TTL_MS;

                {
                    let mut confirmations = state
                        .pending_confirmations
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());

                    confirmations.retain(|_, value| {
                        value.expires_at_ms > unix_timestamp_ms()
                            && value.source != source
                    });

                    confirmations.insert(
                        id.clone(),
                        PendingConfirmation {
                            command: text.clone(),
                            source: source.clone(),
                            permission: action.permission,
                            expires_at_ms,
                        },
                    );
                }

                emit_core_event(
                    &app,
                    CoreEvent {
                        id: id.clone(),
                        kind: "command.awaiting_confirmation",
                        status: AuraRuntimeStatus::Waiting,
                        message,
                        command: Some(text.clone()),
                        timestamp_ms: unix_timestamp_ms(),
                    },
                );
            }
            PermissionDecision::Never => {
                emit_core_event(
                    &app,
                    CoreEvent {
                        id: id.clone(),
                        kind: "command.failed",
                        status: AuraRuntimeStatus::Idle,
                        message: "This action is blocked by AURA's permission policy.".to_string(),
                        command: Some(text.clone()),
                        timestamp_ms: unix_timestamp_ms(),
                    },
                );
            }
            }
        }
        RouteResult::InvalidKeyboard(message) => {
            emit_core_event(
                &app,
                CoreEvent {
                    id: id.clone(),
                    kind: "command.failed",
                    status: AuraRuntimeStatus::Idle,
                    message: format!("Keyboard command rejected: {}", message),
                    command: Some(text.clone()),
                    timestamp_ms: unix_timestamp_ms(),
                },
            );
        }
        RouteResult::InvalidMouse(message) => {
            emit_core_event(
                &app,
                CoreEvent {
                    id: id.clone(),
                    kind: "command.failed",
                    status: AuraRuntimeStatus::Idle,
                    message: format!("Mouse command rejected: {}", message),
                    command: Some(text.clone()),
                    timestamp_ms: unix_timestamp_ms(),
                },
            );
        }
        RouteResult::InvalidMedia(message) => {
            emit_core_event(
                &app,
                CoreEvent {
                    id: id.clone(),
                    kind: "command.failed",
                    status: AuraRuntimeStatus::Idle,
                    message: format!("Media command rejected: {}", message),
                    command: Some(text.clone()),
                    timestamp_ms: unix_timestamp_ms(),
                },
            );
        }
        RouteResult::UnsupportedApp(target) => {
            emit_core_event(
                &app,
                CoreEvent {
                    id: id.clone(),
                    kind: "command.failed",
                    status: AuraRuntimeStatus::Idle,
                    message: format!(
                        "I do not have a safe application target for “{}” yet. Try OBS, Brave, Chrome, File Explorer, Windows Terminal, Notepad or Calculator.",
                        target
                    ),
                    command: Some(text.clone()),
                    timestamp_ms: unix_timestamp_ms(),
                },
            );
        }
        RouteResult::NoMatch => {
            let worker_app = app.clone();
            let worker_id = id.clone();
            let worker_text = text.clone();

            thread::spawn(move || {
                emit_core_event(
                    &worker_app,
                    CoreEvent {
                        id: worker_id.clone(),
                        kind: "command.processing",
                        status: AuraRuntimeStatus::Working,
                        message: "Thinking with the selected local model…".to_string(),
                        command: Some(worker_text.clone()),
                        timestamp_ms: unix_timestamp_ms(),
                    },
                );

                let manager = worker_app.state::<ModelManager>();
                let runtime = worker_app.state::<ModelRuntime>();
                let desktop_context = {
                    let awareness = worker_app.state::<CurrentAppAwareness>();
                    let foreground = awareness.snapshot().ok();
                    let recent = recent_files_snapshot(5).ok();
                    let project = summarize_active_project(&worker_app).ok().flatten();

                    if foreground.is_none() && recent.is_none() && project.is_none() {
                        None
                    } else {
                        let mut summary = String::new();

                        if let Some(context) = foreground {
                            summary.push_str(&format!(
                                "Current app: {}\nProcess: {}",
                                context.app_name, context.process_name
                            ));

                            if let Some(title) = context.window_title.as_deref() {
                                summary.push_str(&format!("\nActive window title: {title}"));
                            }

                            if context.context_source == "lastExternal" {
                                summary.push_str(
                                    "\nContext source: last external window before AURA took focus",
                                );
                            } else {
                                summary.push_str("\nContext source: foreground");
                            }
                        }

                        if let Some(snapshot) = recent {
                            if !snapshot.items.is_empty() {
                                if !summary.is_empty() {
                                    summary.push_str("\n");
                                }
                                let names = snapshot
                                    .items
                                    .iter()
                                    .map(|item| item.name.as_str())
                                    .collect::<Vec<_>>()
                                    .join(" · ");
                                summary.push_str(&format!("Recent files: {names}"));
                            }
                        }

                        if let Some(project) = project {
                            if !summary.is_empty() {
                                summary.push_str("\n");
                            }
                            summary.push_str(&project);
                        }

                        (!summary.is_empty()).then_some(summary)
                    }
                };

                match runtime.generate(
                    &worker_app,
                    &manager,
                    &worker_text,
                    desktop_context.as_deref(),
                ) {
                    Ok(response) => {
                        emit_core_event(
                            &worker_app,
                            CoreEvent {
                                id: worker_id,
                                kind: "command.completed",
                                status: AuraRuntimeStatus::Idle,
                                message: response,
                                command: Some(worker_text),
                                timestamp_ms: unix_timestamp_ms(),
                            },
                        );
                    }
                    Err(error) => {
                        let message = format!("Local model unavailable: {error}");

                        emit_core_event(
                            &worker_app,
                            CoreEvent {
                                id: worker_id.clone(),
                                kind: "command.failed",
                                status: AuraRuntimeStatus::Idle,
                                message: message.clone(),
                                command: Some(worker_text),
                                timestamp_ms: unix_timestamp_ms(),
                            },
                        );

                        emit_core_error(
                            &worker_app,
                            CoreError {
                                id: Some(worker_id),
                                code: "model.runtime_failed",
                                message,
                            },
                        );
                    }
                }
            });
        }
    }

    Ok(CommandAck {
        id,
        accepted: true,
        status: AuraRuntimeStatus::Thinking,
    })
}

#[tauri::command]
fn get_managed_runtime_status(
    app: AppHandle,
    setup: State<'_, ManagedRuntimeSetup>,
) -> ManagedRuntimeStatus {
    setup.status(&app)
}

#[tauri::command]
fn install_managed_runtime(
    app: AppHandle,
    runtime: State<'_, ModelRuntime>,
    setup: State<'_, ManagedRuntimeSetup>,
) -> Result<ManagedRuntimeStatus, String> {
    runtime.stop();
    setup.start_install(app, false)
}

#[tauri::command]
fn repair_managed_runtime(
    app: AppHandle,
    runtime: State<'_, ModelRuntime>,
    setup: State<'_, ManagedRuntimeSetup>,
) -> Result<ManagedRuntimeStatus, String> {
    runtime.stop();
    setup.start_install(app, true)
}

#[tauri::command]
fn remove_managed_runtime(
    app: AppHandle,
    runtime: State<'_, ModelRuntime>,
    setup: State<'_, ManagedRuntimeSetup>,
) -> Result<ManagedRuntimeStatus, String> {
    runtime.stop();
    setup.remove(&app)
}

#[tauri::command]
fn get_model_runtime_status(
    runtime: State<'_, ModelRuntime>,
) -> ModelRuntimeStatus {
    runtime.status()
}

#[tauri::command]
fn clear_model_conversation(
    runtime: State<'_, ModelRuntime>,
) -> ModelRuntimeStatus {
    runtime.clear_conversation();
    runtime.status()
}

#[tauri::command]
fn get_model_catalog(
    app: AppHandle,
    manager: State<'_, ModelManager>,
) -> Result<ModelCatalog, String> {
    manager.catalog(&app)
}

#[tauri::command]
fn start_model_download(
    app: AppHandle,
    model_id: String,
    manager: State<'_, ModelManager>,
) -> Result<ModelCatalog, String> {
    manager.start_download(app, &model_id)
}

#[tauri::command]
fn pause_model_download(
    app: AppHandle,
    model_id: String,
    manager: State<'_, ModelManager>,
) -> Result<ModelCatalog, String> {
    manager.pause_download(&app, &model_id)
}

#[tauri::command]
fn resume_model_download(
    app: AppHandle,
    model_id: String,
    manager: State<'_, ModelManager>,
) -> Result<ModelCatalog, String> {
    manager.resume_download(&app, &model_id)
}

#[tauri::command]
fn cancel_model_download(
    app: AppHandle,
    model_id: String,
    manager: State<'_, ModelManager>,
) -> Result<ModelCatalog, String> {
    manager.cancel_download(&app, &model_id)
}

#[tauri::command]
fn set_active_model(
    app: AppHandle,
    model_id: String,
    manager: State<'_, ModelManager>,
    runtime: State<'_, ModelRuntime>,
) -> Result<ModelCatalog, String> {
    let catalog = manager.set_active(&app, &model_id)?;
    runtime.stop();
    Ok(catalog)
}

#[tauri::command]
fn remove_model(
    app: AppHandle,
    model_id: String,
    manager: State<'_, ModelManager>,
    runtime: State<'_, ModelRuntime>,
    speech: State<'_, SpeechRuntime>,
    tts: State<'_, TtsRuntime>,
) -> Result<ModelCatalog, String> {
    if model_id == "voice-whisper-base" {
        speech.stop();
    }
    if matches!(
        model_id.as_str(),
        "voice-piper-ptpt" | "voice-piper-engb-alan"
    ) {
        tts.stop();
    }
    let catalog = manager.remove_model(&app, &model_id)?;
    runtime.stop();
    Ok(catalog)
}

#[tauri::command]
fn get_speech_runtime_status(
    runtime: State<'_, SpeechRuntime>,
) -> SpeechRuntimeStatus {
    runtime.status()
}

#[tauri::command]
fn get_tts_runtime_status(
    app: AppHandle,
    manager: State<'_, ModelManager>,
    runtime: State<'_, TtsRuntime>,
) -> TtsRuntimeStatus {
    runtime.status(&app, &manager)
}

#[tauri::command]
fn prepare_tts_runtime(
    app: AppHandle,
    manager: State<'_, ModelManager>,
    runtime: State<'_, TtsRuntime>,
) -> Result<TtsRuntimeStatus, String> {
    runtime.prepare_dependency(&app, &manager)
}

#[tauri::command]
fn stop_tts_speaking(
    runtime: State<'_, TtsRuntime>,
) -> Result<TtsRuntimeStatus, String> {
    runtime.interrupt()
}

#[tauri::command]
fn test_tts_voice(
    app: AppHandle,
    manager: State<'_, ModelManager>,
    runtime: State<'_, TtsRuntime>,
    text: Option<String>,
) -> Result<TtsRuntimeStatus, String> {
    let phrase = text
        .as_deref()
        .unwrap_or("Olá. Eu sou a AURA, o teu assistente pessoal.")
        .trim();
    let speed = app
        .state::<RuntimeState>()
        .voice_preferences
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .tts_speed;
    let voice_id = app
        .state::<RuntimeState>()
        .voice_preferences
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .tts_voice_id
        .clone();
    runtime.speak(&app, &manager, phrase, speed, &voice_id)
}

#[tauri::command]
fn get_audio_input_state(
    audio: State<'_, AudioInputManager>,
) -> Result<AudioInputSnapshot, String> {
    audio.snapshot()
}

#[tauri::command]
fn select_audio_input_device(
    audio: State<'_, AudioInputManager>,
    device_name: Option<String>,
) -> Result<AudioInputSnapshot, String> {
    audio.select_device(device_name)
}

#[tauri::command]
fn start_audio_input_test(
    audio: State<'_, AudioInputManager>,
) -> Result<AudioInputSnapshot, String> {
    audio.start_test()
}

#[tauri::command]
fn stop_audio_input_test(
    audio: State<'_, AudioInputManager>,
) -> AudioInputSnapshot {
    audio.stop_test()
}

#[tauri::command]
fn get_current_app_context(
    awareness: State<'_, CurrentAppAwareness>,
) -> Result<CurrentAppInfo, String> {
    awareness.snapshot().map_err(|error| error.to_string())
}

#[tauri::command]
fn get_recent_files_context() -> Result<RecentFilesSnapshot, String> {
    default_recent_files_snapshot().map_err(|error| error.to_string())
}

#[tauri::command]
fn get_project_memory(app: AppHandle) -> Result<ProjectMemorySnapshot, String> {
    project_snapshot(&app)
}

#[tauri::command]
fn save_project_memory(
    app: AppHandle,
    request: SaveProjectRequest,
) -> Result<ProjectMemory, String> {
    save_project(&app, request)
}

#[tauri::command]
fn delete_project_memory(app: AppHandle, project_id: String) -> Result<(), String> {
    delete_project(&app, &project_id)
}

#[tauri::command]
fn set_active_project_memory(
    app: AppHandle,
    project_id: Option<String>,
) -> Result<ProjectMemorySnapshot, String> {
    set_active_project(&app, project_id.as_deref())
}

#[tauri::command]
fn get_user_routines(app: AppHandle) -> Result<Vec<UserRoutine>, String> {
    list_routines(&app)
}

#[tauri::command]
fn save_user_routine(
    app: AppHandle,
    request: SaveRoutineRequest,
) -> Result<UserRoutine, String> {
    save_routine(&app, request)
}

#[tauri::command]
fn delete_user_routine(app: AppHandle, routine_id: String) -> Result<(), String> {
    delete_routine(&app, &routine_id)
}

#[tauri::command]
fn run_user_routine(
    app: AppHandle,
    routine_id: String,
) -> Result<RoutineRunResult, String> {
    let routine = find_routine_by_id(&app, &routine_id)
        .ok_or_else(|| "Routine no longer exists.".to_string())?;

    let permission = if routine_requires_sensitive_permission(&app, &routine) {
        PermissionClass::Sensitive
    } else {
        PermissionClass::Act
    };

    if permission == PermissionClass::Sensitive {
        return Err(
            "This routine contains a Sensitive action. Run it from Chat so AURA can request confirmation."
                .to_string(),
        );
    }

    let obs = app.state::<ObsController>();
    Ok(tauri::async_runtime::block_on(run_routine(&app, &obs, &routine)))
}

#[tauri::command]
fn get_memories(app: AppHandle) -> Result<MemorySnapshot, String> {
    memory_snapshot(&app)
}

#[tauri::command]
fn create_memory_command(
    app: AppHandle,
    request: CreateMemoryRequest,
) -> Result<MemoryCreateResult, String> {
    create_memory(&app, request, "ui")
}

#[tauri::command]
fn delete_memory_command(
    app: AppHandle,
    memory_id: String,
) -> Result<MemoryRecord, String> {
    delete_memory(&app, &memory_id)
}

#[tauri::command]
fn get_director_presets(app: AppHandle) -> Vec<DirectorPreset> {
    load_director_presets(&app)
}

#[tauri::command]
fn save_director_preset_command(
    app: AppHandle,
    request: SaveDirectorPresetRequest,
) -> Result<DirectorPreset, String> {
    save_director_preset(&app, request)
}

#[tauri::command]
fn delete_director_preset_command(
    app: AppHandle,
    preset_id: String,
) -> Result<(), String> {
    delete_director_preset(&app, &preset_id)
}

#[tauri::command]
async fn run_director_preset_command(
    app: AppHandle,
    preset_id: String,
    obs: State<'_, ObsController>,
) -> Result<DirectorPresetRunResult, String> {
    let preset = find_director_preset_by_id(&app, &preset_id)
        .ok_or_else(|| "Director Mode preset no longer exists.".to_string())?;

    Ok(run_director_preset(&obs, &preset).await)
}

#[tauri::command]
async fn get_obs_connection_state(obs: State<'_, ObsController>) -> ObsConnectionState {
    obs.snapshot().await
}

#[tauri::command]
async fn connect_obs(
    request: ObsConnectRequest,
    obs: State<'_, ObsController>,
) -> Result<ObsConnectionState, String> {
    obs.connect(request).await
}

#[tauri::command]
async fn disconnect_obs(obs: State<'_, ObsController>) -> ObsConnectionState {
    obs.disconnect().await
}

#[tauri::command]
async fn get_obs_runtime_state(obs: State<'_, ObsController>) -> ObsRuntimeState {
    obs.runtime_state().await
}

#[tauri::command]
async fn get_obs_scenes(obs: State<'_, ObsController>) -> ObsSceneList {
    obs.scene_list().await
}

#[tauri::command]
async fn get_obs_stream_duration(
    obs: State<'_, ObsController>,
) -> Result<ObsStreamDuration, String> {
    obs.stream_duration().await
}

#[tauri::command]
async fn get_obs_production_health(
    obs: State<'_, ObsController>,
) -> Result<ObsProductionHealth, String> {
    obs.production_health().await
}

#[tauri::command]
async fn get_obs_source_items(
    obs: State<'_, ObsController>,
) -> Result<ObsSourceItemList, String> {
    obs.current_program_source_items().await
}

#[tauri::command]
async fn set_obs_source_visibility(
    request: ObsSourceVisibilityRequest,
    obs: State<'_, ObsController>,
) -> Result<ObsSourceVisibilityResult, String> {
    obs.set_source_visibility(request).await
}

#[tauri::command]
async fn get_obs_audio_inputs(
    obs: State<'_, ObsController>,
) -> Result<ObsAudioInputList, String> {
    obs.audio_inputs().await
}

#[tauri::command]
async fn set_obs_audio_muted(
    request: ObsAudioMuteRequest,
    obs: State<'_, ObsController>,
) -> Result<ObsAudioControlResult, String> {
    obs.set_audio_muted(request).await
}

#[tauri::command]
async fn set_obs_audio_volume(
    request: ObsAudioVolumeRequest,
    obs: State<'_, ObsController>,
) -> Result<ObsAudioControlResult, String> {
    obs.set_audio_volume(request).await
}

#[tauri::command]
async fn set_obs_program_scene(
    request: ObsSceneSwitchRequest,
    obs: State<'_, ObsController>,
) -> Result<ObsSceneSwitchResult, String> {
    obs.set_program_scene(request).await
}

#[tauri::command]
async fn set_obs_preview_scene(
    request: ObsSceneSwitchRequest,
    obs: State<'_, ObsController>,
) -> Result<ObsSceneSwitchResult, String> {
    obs.set_preview_scene(request).await
}

#[tauri::command]
async fn start_obs_recording(
    obs: State<'_, ObsController>,
) -> Result<ObsRecordingActionResult, String> {
    obs.start_recording().await
}

#[tauri::command]
async fn stop_obs_recording(
    obs: State<'_, ObsController>,
) -> Result<ObsRecordingActionResult, String> {
    obs.stop_recording().await
}

#[tauri::command]
async fn pause_obs_recording(
    obs: State<'_, ObsController>,
) -> Result<ObsRecordingActionResult, String> {
    obs.pause_recording().await
}

#[tauri::command]
async fn resume_obs_recording(
    obs: State<'_, ObsController>,
) -> Result<ObsRecordingActionResult, String> {
    obs.resume_recording().await
}

#[tauri::command]
async fn start_obs_streaming(
    obs: State<'_, ObsController>,
) -> Result<ObsStreamingActionResult, String> {
    obs.start_streaming().await
}

#[tauri::command]
async fn stop_obs_streaming(
    obs: State<'_, ObsController>,
) -> Result<ObsStreamingActionResult, String> {
    obs.stop_streaming().await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(RuntimeState::default())
        .manage(AudioInputManager::default())
        .manage(CurrentAppAwareness::default())
        .manage(ManagedRuntimeSetup::default())
        .manage(ModelManager::default())
        .manage(ModelRuntime::default())
        .manage(SpeechRuntime::default())
        .manage(TtsRuntime::default())
        .manage(ObsController::default())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if shortcut.matches(
                        Modifiers::CONTROL | Modifiers::SHIFT,
                        Code::Space,
                    ) {
                        if event.state() == ShortcutState::Pressed {
                            toggle_overlay(app);
                        }
                        return;
                    }

                    if shortcut.matches(
                        Modifiers::CONTROL | Modifiers::SHIFT,
                        Code::F8,
                    ) {
                        let runtime = app.state::<RuntimeState>();
                        let paused = *runtime
                            .paused
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());

                        if paused {
                            return;
                        }

                        let audio = app.state::<AudioInputManager>();

                        match event.state() {
                            ShortcutState::Pressed => {
                                if app.state::<TtsRuntime>().is_speaking() {
                                    let _ = app.state::<TtsRuntime>().interrupt();
                                }

                                runtime
                                    .wake_monitor_generation
                                    .fetch_add(1, Ordering::Relaxed);

                                if audio.capture_active() {
                                    let _ = audio.stop_push_to_talk();
                                    let _ = audio.take_last_capture();
                                }

                                match audio.start_push_to_talk() {
                                    Ok(snapshot) => emit_voice_capture_event(
                                        app,
                                        VoiceCaptureEvent {
                                            phase: "listening",
                                            shortcut: "Ctrl+Shift+F8",
                                            sample_count: 0,
                                            duration_ms: 0,
                                            sample_rate: snapshot.sample_rate,
                                            channels: snapshot.channels,
                                            message: "Push-to-talk listening…".to_string(),
                                            text: None,
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    ),
                                    Err(error) => emit_voice_capture_event(
                                        app,
                                        VoiceCaptureEvent {
                                            phase: "error",
                                            shortcut: "Ctrl+Shift+F8",
                                            sample_count: 0,
                                            duration_ms: 0,
                                            sample_rate: None,
                                            channels: None,
                                            message: error,
                                            text: None,
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    ),
                                }
                            }
                            ShortcutState::Released => {
                                match audio.stop_push_to_talk() {
                                    Ok(_) => {
                                        let info = audio.last_capture_info();
                                        emit_voice_capture_event(
                                            app,
                                            VoiceCaptureEvent {
                                                phase: "captured",
                                                shortcut: "Ctrl+Shift+F8",
                                                sample_count: info.map(|value| value.0).unwrap_or(0),
                                                duration_ms: info.map(|value| value.3).unwrap_or(0),
                                                sample_rate: info.map(|value| value.1),
                                                channels: info.map(|value| value.2),
                                                message: "Voice capture ready for local speech-to-text.".to_string(),
                                                text: None,
                                                timestamp_ms: unix_timestamp_ms(),
                                            },
                                        );

                                        if let Some(capture) = audio.take_last_capture() {
                                            process_voice_capture(
                                                app.clone(),
                                                capture,
                                                "Ctrl+Shift+F8",
                                            );
                                        }
                                    }
                                    Err(error) => emit_voice_capture_event(
                                        app,
                                        VoiceCaptureEvent {
                                            phase: "error",
                                            shortcut: "Ctrl+Shift+F8",
                                            sample_count: 0,
                                            duration_ms: 0,
                                            sample_rate: None,
                                            channels: None,
                                            message: error,
                                            text: None,
                                            timestamp_ms: unix_timestamp_ms(),
                                        },
                                    ),
                                }

                                let preferences = runtime
                                    .voice_preferences
                                    .lock()
                                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                                    .clone();
                                if preferences.wake_word_enabled {
                                    let generation = runtime
                                        .wake_monitor_generation
                                        .load(Ordering::Relaxed);
                                    spawn_wake_monitor(app.clone(), generation);
                                }
                            }
                        }
                    }
                })
                .build(),
        )
        .setup(|app| {
            let preferences = load_preferences(app.handle());
            let voice_preferences = load_voice_preferences(app.handle());
            let permission_policy = load_permission_policy(app.handle());
            {
                let runtime = app.state::<RuntimeState>();
                let mut background_enabled = runtime
                    .background_enabled
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *background_enabled = preferences.background_enabled;

                let registered = app.autolaunch().is_enabled().unwrap_or(false);
                let mut autostart_enabled = runtime
                    .autostart_enabled
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *autostart_enabled = registered;

                let mut current_policy = runtime
                    .permission_policy
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *current_policy = permission_policy;

                *runtime
                    .voice_preferences
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                    voice_preferences.clone();
            }

            if voice_preferences.wake_word_enabled {
                let generation = app
                    .state::<RuntimeState>()
                    .wake_monitor_generation
                    .load(Ordering::Relaxed);
                spawn_wake_monitor(app.handle().clone(), generation);
            }

            let launched_in_background = std::env::args().any(|arg| arg == "--background");

            let shortcut = Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::SHIFT),
                Code::Space,
            );
            app.global_shortcut().register(shortcut)?;

            let push_to_talk_shortcut = Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::SHIFT),
                Code::F8,
            );
            app.global_shortcut().register(push_to_talk_shortcut)?;

            let open_item =
                MenuItem::with_id(app, "open", "Open AURA", true, None::<&str>)?;
            let pause_item =
                CheckMenuItem::with_id(app, "pause", "Pause AURA", true, false, None::<&str>)?;
            let settings_item =
                MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_item =
                MenuItem::with_id(app, "quit", "Quit AURA", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[&open_item, &pause_item, &settings_item, &quit_item],
            )?;

            let pause_item_for_menu = pause_item.clone();

            TrayIconBuilder::with_id("aura-main")
                .icon(aura_tray_icon())
                .tooltip("AURA-2")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "open" => show_main_window(app),
                    "pause" => {
                        let current = app.state::<RuntimeState>();
                        let next = !runtime_snapshot(&current).paused;
                        let snapshot = set_paused_state(app, next);
                        let _ = pause_item_for_menu.set_checked(snapshot.paused);
                    }
                    "settings" => {
                        show_main_window(app);
                        let _ = app.emit("aura:open-settings", ());
                    }
                    "quit" => {
                        app.state::<ModelRuntime>().stop();
                        app.state::<SpeechRuntime>().stop();
                        app.state::<TtsRuntime>().stop();
                        app.exit(0);
                    },
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let window_for_close = window.clone();
                let app_for_close = app.handle().clone();

                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        let runtime = app_for_close.state::<RuntimeState>();
                        let background_enabled = runtime_snapshot(&runtime).background_enabled;

                        if background_enabled {
                            api.prevent_close();
                            let _ = window_for_close.hide();
                            emit_lifecycle_event(
                                &app_for_close,
                                "background.entered",
                                "AURA is running in the background. Use the tray or shortcut to return.",
                            );
                        } else {
                            app_for_close.state::<ModelRuntime>().stop();
                            app_for_close.state::<SpeechRuntime>().stop();
                            app_for_close.state::<TtsRuntime>().stop();
                            app_for_close.exit(0);
                        }
                    }
                });
            }

            if let Some(overlay) = app.get_webview_window("overlay") {
                let overlay_for_events = overlay.clone();
                overlay.on_window_event(move |event| {
                    if let WindowEvent::Focused(false) = event {
                        let _ = overlay_for_events.hide();
                    }
                });
            }

            if launched_in_background {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
                emit_lifecycle_event(
                    app.handle(),
                    "startup.background",
                    "AURA started with Windows and is running in the background.",
                );
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            get_runtime_state,
            get_voice_preferences,
            set_voice_preferences,
            set_runtime_paused,
            set_background_enabled,
            set_autostart_enabled,
            get_permission_policy,
            set_permission_decision,
            reset_permission_policy,
            resolve_confirmation,
            process_user_command,
            get_managed_runtime_status,
            install_managed_runtime,
            repair_managed_runtime,
            remove_managed_runtime,
            get_model_runtime_status,
            clear_model_conversation,
            get_model_catalog,
            start_model_download,
            pause_model_download,
            resume_model_download,
            cancel_model_download,
            set_active_model,
            remove_model,
            get_speech_runtime_status,
            get_tts_runtime_status,
            prepare_tts_runtime,
            test_tts_voice,
            stop_tts_speaking,
            get_audio_input_state,
            select_audio_input_device,
            start_audio_input_test,
            stop_audio_input_test,
            get_current_app_context,
            get_recent_files_context,
            get_project_memory,
            save_project_memory,
            delete_project_memory,
            set_active_project_memory,
            get_user_routines,
            save_user_routine,
            delete_user_routine,
            run_user_routine,
            get_memories,
            create_memory_command,
            delete_memory_command,
            get_director_presets,
            save_director_preset_command,
            delete_director_preset_command,
            run_director_preset_command,
            open_main_window,
            hide_overlay,
            get_obs_connection_state,
            connect_obs,
            disconnect_obs,
            get_obs_runtime_state,
            get_obs_scenes,
            get_obs_stream_duration,
            get_obs_production_health,
            get_obs_source_items,
            set_obs_source_visibility,
            get_obs_audio_inputs,
            set_obs_audio_muted,
            set_obs_audio_volume,
            set_obs_program_scene,
            set_obs_preview_scene,
            start_obs_recording,
            stop_obs_recording,
            pause_obs_recording,
            resume_obs_recording,
            start_obs_streaming,
            stop_obs_streaming
        ])
        .run(tauri::generate_context!())
        .expect("error while running AURA-2");
}
