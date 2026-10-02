use serde::{Deserialize, Serialize};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};

static COMMAND_COUNTER: AtomicU64 = AtomicU64::new(1);

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

fn emit_core_event(app: &tauri::AppHandle, event: CoreEvent) {
    let _ = app.emit("aura:core-event", event);
}

fn emit_core_error(app: &tauri::AppHandle, error: CoreError) {
    let _ = app.emit("aura:core-error", error);
}

#[tauri::command]
fn get_app_status() -> AppStatus {
    AppStatus {
        name: "AURA-2",
        version: env!("CARGO_PKG_VERSION"),
        stage: "M002.4 Core ↔ Desktop",
        local_first: true,
    }
}

#[tauri::command]
fn process_user_command(
    app: tauri::AppHandle,
    request: CommandRequest,
) -> Result<CommandAck, String> {
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

fn toggle_main_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_main_window(app);
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            process_user_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running AURA-2");
}
