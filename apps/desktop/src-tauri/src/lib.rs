mod computer;
mod core;
mod permissions;

use computer::app_launcher::launch_app;
use computer::app_lifecycle::close_app;
use computer::audio::{execute_media_action, MediaAction};
use computer::keyboard::{press_shortcut, type_text};
use computer::mouse::{execute_mouse_action, MouseAction};
use computer::system::{execute_system_action, summarize_system, SystemAction};
use computer::window_manager::{summarize_windows, switch_to_app};
use core::action_router::{route_command, ActionIntent, RouteResult};
use permissions::{PermissionDecision, PermissionPolicy};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WindowEvent,
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
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            paused: Mutex::new(false),
            background_enabled: Mutex::new(true),
            autostart_enabled: Mutex::new(false),
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandRequest {
    text: String,
    source: String,
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

fn next_command_id() -> String {
    let counter = COMMAND_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("cmd-{}-{}", unix_timestamp_ms(), counter)
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

fn emit_core_event(app: &tauri::AppHandle, event: CoreEvent) {
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
        stage: "M003.7 System Commands",
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
    let id = next_command_id();

    emit_core_event(
        &app,
        CoreEvent {
            id: id.clone(),
            kind: "command.accepted",
            status: AuraRuntimeStatus::Thinking,
            message: format!("Understanding: “{}”", text),
            command: Some(text.clone()),
            timestamp_ms: unix_timestamp_ms(),
        },
    );

    let policy = PermissionPolicy::default();

    match route_command(&text, &policy) {
        RouteResult::Action(action) => match action.decision {
            PermissionDecision::Allow => {
                let worker_app = app.clone();
                let worker_id = id.clone();
                let worker_text = text.clone();
                let worker_source = source.clone();

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
                        "Closing {} requires confirmation because unsaved work could be lost. Confirmation controls arrive in M003.9.",
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
            PermissionDecision::Block => {
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
        },
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
            emit_core_event(
                &app,
                CoreEvent {
                    id: id.clone(),
                    kind: "command.unhandled",
                    status: AuraRuntimeStatus::Idle,
                    message:
                        "No deterministic computer action matched yet. M003 currently supports app/window control, keyboard, mouse, audio/media and Windows system commands."
                            .to_string(),
                    command: Some(text.clone()),
                    timestamp_ms: unix_timestamp_ms(),
                },
            );
        }
    }

    Ok(CommandAck {
        id,
        accepted: true,
        status: AuraRuntimeStatus::Thinking,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(RuntimeState::default())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_overlay(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let preferences = load_preferences(app.handle());
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
            }

            let launched_in_background = std::env::args().any(|arg| arg == "--background");

            let shortcut = Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::SHIFT),
                Code::Space,
            );
            app.global_shortcut().register(shortcut)?;

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
                    "quit" => app.exit(0),
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
            set_runtime_paused,
            set_background_enabled,
            set_autostart_enabled,
            process_user_command,
            open_main_window,
            hide_overlay
        ])
        .run(tauri::generate_context!())
        .expect("error while running AURA-2");
}
