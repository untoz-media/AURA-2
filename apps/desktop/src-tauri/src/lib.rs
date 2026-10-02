use serde::{Deserialize, Serialize};
use std::{
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
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};

static COMMAND_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Default)]
struct RuntimeState {
    paused: Mutex<bool>,
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

fn runtime_snapshot(state: &RuntimeState) -> RuntimeSnapshot {
    RuntimeSnapshot {
        paused: *state.paused.lock().unwrap_or_else(|poisoned| poisoned.into_inner()),
    }
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

    let snapshot = RuntimeSnapshot { paused };
    let _ = app.emit("aura:runtime-state", snapshot.clone());
    snapshot
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
        stage: "M002.7 AURA Overlay",
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

    let worker_app = app.clone();
    let worker_id = id.clone();
    let worker_text = text.clone();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(140));

        emit_core_event(
            &worker_app,
            CoreEvent {
                id: worker_id.clone(),
                kind: "command.processing",
                status: AuraRuntimeStatus::Working,
                message: "Routing command through AURA Core…".to_string(),
                command: Some(worker_text.clone()),
                timestamp_ms: unix_timestamp_ms(),
            },
        );

        thread::sleep(Duration::from_millis(420));

        emit_core_event(
            &worker_app,
            CoreEvent {
                id: worker_id,
                kind: "command.completed",
                status: AuraRuntimeStatus::Idle,
                message:
                    "AURA Core received the command successfully. Action execution arrives in M003."
                        .to_string(),
                command: Some(worker_text),
                timestamp_ms: unix_timestamp_ms(),
            },
        );
    });

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
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_for_close.hide();
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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            get_runtime_state,
            set_runtime_paused,
            process_user_command,
            open_main_window,
            hide_overlay
        ])
        .run(tauri::generate_context!())
        .expect("error while running AURA-2");
}
