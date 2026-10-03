use crate::computer::{
    app_launcher::AppTarget,
    audio::MediaAction,
    keyboard::KeyboardShortcut,
    mouse::{parse_point, validate_scroll_notches, MouseAction, MouseButton},
    system::{SettingsPage, SystemAction},
};
use crate::permissions::{PermissionClass, PermissionDecision, PermissionPolicy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObsRecordingAction {
    Start,
    Stop,
    Pause,
    Resume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObsStreamingAction {
    Start,
    Stop,
}

#[derive(Debug, Clone)]
pub enum ActionIntent {
    LaunchApp(AppTarget),
    CloseApp(AppTarget),
    SwitchToApp(AppTarget),
    ListWindows,
    PressShortcut(KeyboardShortcut),
    TypeText(String),
    Mouse(MouseAction),
    Media(MediaAction),
    System(SystemAction),
    ObsProgramScene(String),
    ObsPreviewScene(String),
    ObsRecording(ObsRecordingAction),
    ObsStreaming(ObsStreamingAction),
    ObsStreamDuration,
    ObsSourceVisibility { source_name: String, enabled: bool },
}

#[derive(Debug, Clone)]
pub struct RoutedAction {
    pub intent: ActionIntent,
    pub permission: PermissionClass,
    pub decision: PermissionDecision,
}

#[derive(Debug, Clone)]
pub enum RouteResult {
    Action(RoutedAction),
    UnsupportedApp(String),
    InvalidKeyboard(String),
    InvalidMouse(String),
    InvalidMedia(String),
    NoMatch,
}

#[derive(Debug, Clone, Copy)]
enum AppOperation {
    Launch,
    Close,
    Switch,
}

fn system_request(input: &str) -> Option<SystemAction> {
    let normalized = input.trim().to_lowercase();

    match normalized.as_str() {
        "system status" | "pc status" | "computer status"
        | "estado do sistema" | "estado do pc" => Some(SystemAction::GetStatus),

        "battery status" | "battery" | "estado da bateria"
        | "qual é a bateria" | "qual e a bateria" => Some(SystemAction::GetBattery),

        "show desktop" | "show the desktop" | "mostrar ambiente de trabalho"
        | "mostra o ambiente de trabalho" => Some(SystemAction::ShowDesktop),

        "open task manager" | "abre o gestor de tarefas" | "abrir gestor de tarefas"
        | "abre o task manager" => Some(SystemAction::OpenTaskManager),

        "open settings" | "open windows settings" | "abre as definições"
        | "abre as definicoes" | "abrir definições" | "abrir definicoes" => {
            Some(SystemAction::OpenSettings(SettingsPage::Home))
        }

        "open display settings" | "abre as definições de ecrã"
        | "abre as definicoes de ecra" | "abre as definições de display"
        | "abre as definicoes de display" => {
            Some(SystemAction::OpenSettings(SettingsPage::Display))
        }

        "open bluetooth settings" | "abre as definições de bluetooth"
        | "abre as definicoes de bluetooth" => {
            Some(SystemAction::OpenSettings(SettingsPage::Bluetooth))
        }

        "open network settings" | "abre as definições de rede"
        | "abre as definicoes de rede" => {
            Some(SystemAction::OpenSettings(SettingsPage::Network))
        }

        "open sound settings" | "open audio settings"
        | "abre as definições de som" | "abre as definicoes de som"
        | "abre as definições de áudio" | "abre as definicoes de audio" => {
            Some(SystemAction::OpenSettings(SettingsPage::Sound))
        }

        "lock pc" | "lock computer" | "lock windows"
        | "bloqueia o pc" | "bloquear o pc" => Some(SystemAction::Lock),

        "sleep pc" | "sleep computer" | "put pc to sleep"
        | "suspende o pc" | "suspender o pc" => Some(SystemAction::Sleep),

        "restart pc" | "restart computer" | "reinicia o pc"
        | "reiniciar o pc" => Some(SystemAction::Restart),

        "shutdown pc" | "shut down pc" | "turn off pc"
        | "desliga o pc" | "desligar o pc" => Some(SystemAction::Shutdown),

        _ => None,
    }
}

fn permission_for_system(action: SystemAction) -> PermissionClass {
    match action {
        SystemAction::GetStatus | SystemAction::GetBattery => PermissionClass::Read,

        SystemAction::ShowDesktop
        | SystemAction::OpenTaskManager
        | SystemAction::OpenSettings(_) => PermissionClass::Act,

        SystemAction::Lock | SystemAction::Sleep => PermissionClass::Sensitive,

        SystemAction::Restart | SystemAction::Shutdown => PermissionClass::Destructive,
    }
}

fn normalize_command(input: &str) -> String {
    input
        .trim()
        .trim_matches(|c: char| matches!(c, '.' | ',' | '!' | '?' | ';' | ':'))
        .to_lowercase()
}

fn strip_article(value: &str) -> &str {
    value
        .strip_prefix("the ")
        .or_else(|| value.strip_prefix("o "))
        .or_else(|| value.strip_prefix("a "))
        .unwrap_or(value)
        .trim()
}

fn value_after_prefix<'a>(input: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    let trimmed = input.trim();
    let lowered = trimmed.to_lowercase();

    for prefix in prefixes {
        if lowered.starts_with(prefix) {
            return trimmed.get(prefix.len()..).map(str::trim);
        }
    }

    None
}

fn unwrap_text_quotes(value: &str) -> &str {
    let trimmed = value.trim();

    if trimmed.len() >= 2 {
        let bytes = trimmed.as_bytes();
        let matching_quotes =
            (bytes[0] == b'"' && bytes[trimmed.len() - 1] == b'"')
                || (bytes[0] == b'\'' && bytes[trimmed.len() - 1] == b'\'');

        if matching_quotes {
            return &trimmed[1..trimmed.len() - 1];
        }
    }

    trimmed
}

fn parse_percent(value: &str) -> Result<u8, String> {
    let cleaned = value
        .trim()
        .trim_end_matches('%')
        .trim();

    let percent = cleaned
        .parse::<u16>()
        .map_err(|_| "Volume must be a whole number from 0 to 100.".to_string())?;

    if percent > 100 {
        return Err("Volume must be between 0 and 100 percent.".to_string());
    }

    Ok(percent as u8)
}

fn media_request(input: &str) -> Option<Result<ActionIntent, String>> {
    let normalized = input.trim().to_lowercase();

    if matches!(
        normalized.as_str(),
        "what is the volume"
            | "what's the volume"
            | "what is the volume?"
            | "what's the volume?"
            | "qual é o volume"
            | "qual e o volume"
            | "qual é o volume?"
            | "qual e o volume?"
    ) {
        return Some(Ok(ActionIntent::Media(MediaAction::GetVolume)));
    }

    let exact_action = match normalized.as_str() {
        "volume up" | "increase volume" | "aumenta o volume" | "aumentar o volume" => {
            Some(MediaAction::VolumeUp)
        }
        "volume down" | "decrease volume" | "baixa o volume" | "baixar o volume" => {
            Some(MediaAction::VolumeDown)
        }
        "mute" | "mute volume" | "silencia" | "silenciar" | "sem som" => {
            Some(MediaAction::Mute)
        }
        "unmute" | "tira o mute" | "tirar o mute" | "repõe o som" | "repoe o som" => {
            Some(MediaAction::Unmute)
        }
        "play pause" | "play/pause" | "pause music" | "resume music"
        | "pausa a música" | "pausa a musica" | "retoma a música" | "retoma a musica" => {
            Some(MediaAction::PlayPause)
        }
        "next track" | "next song" | "próxima música" | "proxima musica"
        | "próxima faixa" | "proxima faixa" => Some(MediaAction::NextTrack),
        "previous track" | "previous song" | "música anterior" | "musica anterior"
        | "faixa anterior" => Some(MediaAction::PreviousTrack),
        "stop media" | "stop music" | "para a música" | "para a musica"
        | "parar a música" | "parar a musica" => Some(MediaAction::Stop),
        _ => None,
    };

    if let Some(action) = exact_action {
        return Some(Ok(ActionIntent::Media(action)));
    }

    const SET_VOLUME_PREFIXES: &[&str] = &[
        "set volume to ",
        "set the volume to ",
        "volume ",
        "define o volume para ",
        "define volume para ",
        "mete o volume a ",
        "mete volume a ",
    ];

    value_after_prefix(input, SET_VOLUME_PREFIXES).map(|value| {
        parse_percent(value)
            .map(MediaAction::SetVolume)
            .map(ActionIntent::Media)
    })
}

fn permission_for_media(action: MediaAction) -> PermissionClass {
    match action {
        MediaAction::GetVolume => PermissionClass::Read,
        MediaAction::SetVolume(_)
        | MediaAction::VolumeUp
        | MediaAction::VolumeDown
        | MediaAction::Mute
        | MediaAction::Unmute
        | MediaAction::PlayPause
        | MediaAction::NextTrack
        | MediaAction::PreviousTrack
        | MediaAction::Stop => PermissionClass::Act,
    }
}

fn parse_scroll_amount(value: &str, direction: i32) -> Result<MouseAction, String> {
    let trimmed = value.trim();
    let amount = if trimmed.is_empty() {
        1
    } else {
        trimmed
            .parse::<i32>()
            .map_err(|_| "Scroll amount must be a whole number.".to_string())?
    };

    let signed = validate_scroll_notches(direction * amount)
        .map_err(|error| error.to_string())?;

    Ok(MouseAction::Scroll { notches: signed })
}

fn mouse_request(input: &str) -> Option<Result<ActionIntent, String>> {
    let normalized = input.trim().to_lowercase();

    const MOVE_PREFIXES: &[&str] = &[
        "move mouse to ",
        "move cursor to ",
        "move o rato para ",
        "mover o rato para ",
        "move cursor para ",
        "mover cursor para ",
    ];

    const RIGHT_CLICK_AT_PREFIXES: &[&str] = &[
        "right click at ",
        "clique direito em ",
        "clica com o botão direito em ",
        "clica com o botao direito em ",
    ];

    const CLICK_AT_PREFIXES: &[&str] = &[
        "click at ",
        "clica em ",
        "clique em ",
    ];

    if let Some(value) = value_after_prefix(input, MOVE_PREFIXES) {
        return Some(
            parse_point(value)
                .map(MouseAction::MoveTo)
                .map(ActionIntent::Mouse)
                .map_err(|error| error.to_string()),
        );
    }

    if let Some(value) = value_after_prefix(input, RIGHT_CLICK_AT_PREFIXES) {
        return Some(
            parse_point(value)
                .map(|point| MouseAction::ClickAt {
                    point,
                    button: MouseButton::Right,
                })
                .map(ActionIntent::Mouse)
                .map_err(|error| error.to_string()),
        );
    }

    if let Some(value) = value_after_prefix(input, CLICK_AT_PREFIXES) {
        return Some(
            parse_point(value)
                .map(|point| MouseAction::ClickAt {
                    point,
                    button: MouseButton::Left,
                })
                .map(ActionIntent::Mouse)
                .map_err(|error| error.to_string()),
        );
    }

    const SCROLL_DOWN_PREFIXES: &[&str] = &[
        "scroll down",
        "scroll para baixo",
        "rola para baixo",
        "rolar para baixo",
        "desce",
    ];

    for prefix in SCROLL_DOWN_PREFIXES {
        if normalized == *prefix {
            return Some(parse_scroll_amount("", -1).map(ActionIntent::Mouse));
        }

        if let Some(rest) = normalized.strip_prefix(&format!("{prefix} ")) {
            return Some(parse_scroll_amount(rest, -1).map(ActionIntent::Mouse));
        }
    }

    const SCROLL_UP_PREFIXES: &[&str] = &[
        "scroll up",
        "scroll para cima",
        "rola para cima",
        "rolar para cima",
        "sobe",
    ];

    for prefix in SCROLL_UP_PREFIXES {
        if normalized == *prefix {
            return Some(parse_scroll_amount("", 1).map(ActionIntent::Mouse));
        }

        if let Some(rest) = normalized.strip_prefix(&format!("{prefix} ")) {
            return Some(parse_scroll_amount(rest, 1).map(ActionIntent::Mouse));
        }
    }

    let exact = match normalized.as_str() {
        "click" | "left click" | "clica" | "clique" => Some(MouseAction::Click {
            button: MouseButton::Left,
        }),
        "right click" | "clique direito" | "clica com o botão direito"
        | "clica com o botao direito" => Some(MouseAction::Click {
            button: MouseButton::Right,
        }),
        "double click" | "double-click" | "duplo clique" | "duplo click" => {
            Some(MouseAction::DoubleClick)
        }
        _ => None,
    };

    exact.map(|action| Ok(ActionIntent::Mouse(action)))
}

fn permission_for_mouse(action: MouseAction) -> PermissionClass {
    match action {
        MouseAction::MoveTo(_) | MouseAction::Scroll { .. } => PermissionClass::Act,
        MouseAction::Click { .. }
        | MouseAction::DoubleClick
        | MouseAction::ClickAt { .. } => PermissionClass::Modify,
    }
}

fn keyboard_request(input: &str) -> Option<Result<ActionIntent, String>> {
    const PRESS_PREFIXES: &[&str] = &[
        "carrega em ",
        "pressiona ",
        "pressionar ",
        "press ",
        "carrega ",
    ];

    const TYPE_PREFIXES: &[&str] = &[
        "type ",
        "write ",
        "escreve ",
        "escrever ",
        "digita ",
        "digitar ",
    ];

    if let Some(value) = value_after_prefix(input, PRESS_PREFIXES) {
        return Some(
            KeyboardShortcut::parse(value)
                .map(ActionIntent::PressShortcut)
                .map_err(|error| error.to_string()),
        );
    }

    value_after_prefix(input, TYPE_PREFIXES).map(|value| {
        let text = unwrap_text_quotes(value);
        if text.is_empty() {
            Err("Text input cannot be empty.".to_string())
        } else {
            Ok(ActionIntent::TypeText(text.to_string()))
        }
    })
}

fn permission_for_keyboard(intent: &ActionIntent) -> Option<PermissionClass> {
    match intent {
        ActionIntent::PressShortcut(shortcut) if shortcut.is_delete_action() => {
            Some(PermissionClass::Destructive)
        }
        ActionIntent::PressShortcut(shortcut) if shortcut.is_navigation_only() => {
            Some(PermissionClass::Act)
        }
        ActionIntent::PressShortcut(_) | ActionIntent::TypeText(_) => {
            Some(PermissionClass::Modify)
        }
        _ => None,
    }
}

fn obs_source_visibility_request(input: &str) -> Option<ActionIntent> {
    const SHOW_PREFIXES: &[&str] = &[
        "show source ",
        "show ",
        "enable source ",
        "enable ",
        "turn on source ",
        "turn on ",
        "mostra a source ",
        "mostra ",
        "mostrar ",
        "ativa a source ",
        "ativa ",
        "ativar ",
    ];

    const HIDE_PREFIXES: &[&str] = &[
        "hide source ",
        "hide ",
        "disable source ",
        "disable ",
        "turn off source ",
        "turn off ",
        "esconde a source ",
        "esconde ",
        "esconder ",
        "oculta a source ",
        "oculta ",
        "ocultar ",
        "desativa a source ",
        "desativa ",
        "desativar ",
    ];

    if let Some(value) = value_after_prefix(input, SHOW_PREFIXES) {
        let source = unwrap_text_quotes(value).trim();
        return (!source.is_empty()).then(|| ActionIntent::ObsSourceVisibility {
            source_name: source.to_string(),
            enabled: true,
        });
    }

    value_after_prefix(input, HIDE_PREFIXES).and_then(|value| {
        let source = unwrap_text_quotes(value).trim();
        (!source.is_empty()).then(|| ActionIntent::ObsSourceVisibility {
            source_name: source.to_string(),
            enabled: false,
        })
    })
}

fn is_obs_stream_duration_request(input: &str) -> bool {
    let normalized = normalize_command(input);

    matches!(
        normalized.as_str(),
        "stream duration"
            | "live duration"
            | "how long have we been live"
            | "how long are we live"
            | "how long has the stream been live"
            | "how long has the stream been running"
            | "how long have we been streaming"
            | "what is the stream duration"
            | "what's the stream duration"
            | "há quanto tempo estamos em direto"
            | "ha quanto tempo estamos em direto"
            | "há quanto tempo estamos ao vivo"
            | "ha quanto tempo estamos ao vivo"
            | "há quanto tempo estamos a transmitir"
            | "ha quanto tempo estamos a transmitir"
            | "qual é a duração da transmissão"
            | "qual e a duracao da transmissao"
            | "qual é a duração do direto"
            | "qual e a duracao do direto"
    )
}

fn obs_streaming_request(input: &str) -> Option<ObsStreamingAction> {
    let normalized = normalize_command(input);

    match normalized.as_str() {
        "start streaming"
        | "start stream"
        | "start the stream"
        | "go live"
        | "go live on obs"
        | "inicia a transmissão"
        | "inicia a transmissao"
        | "iniciar transmissão"
        | "iniciar transmissao"
        | "começa a transmissão"
        | "comeca a transmissao"
        | "começar a transmissão"
        | "comecar a transmissao"
        | "entra em direto"
        | "começa o direto"
        | "comeca o direto" => Some(ObsStreamingAction::Start),

        "stop streaming"
        | "stop stream"
        | "stop the stream"
        | "end stream"
        | "end the stream"
        | "stop live"
        | "para a transmissão"
        | "para a transmissao"
        | "parar transmissão"
        | "parar transmissao"
        | "termina a transmissão"
        | "termina a transmissao"
        | "terminar transmissão"
        | "terminar transmissao"
        | "termina o direto"
        | "terminar o direto" => Some(ObsStreamingAction::Stop),

        _ => None,
    }
}

fn permission_for_obs_streaming(action: ObsStreamingAction) -> PermissionClass {
    match action {
        ObsStreamingAction::Start => PermissionClass::Sensitive,
        ObsStreamingAction::Stop => PermissionClass::Act,
    }
}

fn obs_recording_request(input: &str) -> Option<ObsRecordingAction> {
    let normalized = normalize_command(input);

    match normalized.as_str() {
        "start recording"
        | "start obs recording"
        | "record"
        | "begin recording"
        | "inicia a gravação"
        | "inicia a gravacao"
        | "iniciar gravação"
        | "iniciar gravacao"
        | "começa a gravar"
        | "comeca a gravar"
        | "começar a gravar"
        | "comecar a gravar" => Some(ObsRecordingAction::Start),

        "stop recording"
        | "stop obs recording"
        | "end recording"
        | "para a gravação"
        | "para a gravacao"
        | "parar gravação"
        | "parar gravacao"
        | "termina a gravação"
        | "termina a gravacao"
        | "terminar gravação"
        | "terminar gravacao" => Some(ObsRecordingAction::Stop),

        "pause recording"
        | "pause obs recording"
        | "pausa a gravação"
        | "pausa a gravacao"
        | "pausar gravação"
        | "pausar gravacao" => Some(ObsRecordingAction::Pause),

        "resume recording"
        | "resume obs recording"
        | "retoma a gravação"
        | "retoma a gravacao"
        | "retomar gravação"
        | "retomar gravacao"
        | "continua a gravação"
        | "continua a gravacao" => Some(ObsRecordingAction::Resume),

        _ => None,
    }
}

fn obs_scene_request(input: &str) -> Option<ActionIntent> {
    const PROGRAM_PREFIXES: &[&str] = &[
        "switch scene to ",
        "switch to scene ",
        "set program scene to ",
        "take scene ",
        "muda a cena para ",
        "muda para a cena ",
        "troca a cena para ",
        "troca para a cena ",
    ];

    const PREVIEW_PREFIXES: &[&str] = &[
        "preview scene ",
        "set preview scene to ",
        "set preview to ",
        "prepara a cena ",
        "preparar a cena ",
    ];

    if let Some(value) = value_after_prefix(input, PROGRAM_PREFIXES) {
        let scene = unwrap_text_quotes(value).trim();
        return (!scene.is_empty()).then(|| ActionIntent::ObsProgramScene(scene.to_string()));
    }

    value_after_prefix(input, PREVIEW_PREFIXES).and_then(|value| {
        let scene = unwrap_text_quotes(value).trim();
        (!scene.is_empty()).then(|| ActionIntent::ObsPreviewScene(scene.to_string()))
    })
}

fn is_list_windows_command(input: &str) -> bool {
    matches!(
        input,
        "list windows"
            | "show windows"
            | "what windows are open"
            | "what windows are open?"
            | "lista as janelas"
            | "listar janelas"
            | "que janelas estão abertas"
            | "que janelas estao abertas"
            | "que janelas estão abertas?"
            | "que janelas estao abertas?"
    )
}

fn app_request(input: &str) -> Option<(AppOperation, &str)> {
    const LAUNCH_PREFIXES: &[&str] = &[
        "open ",
        "launch ",
        "start ",
        "abre ",
        "abrir ",
        "inicia ",
        "iniciar ",
    ];

    const CLOSE_PREFIXES: &[&str] = &[
        "close ",
        "quit ",
        "exit ",
        "fecha ",
        "fechar ",
        "encerra ",
        "encerrar ",
    ];

    const SWITCH_PREFIXES: &[&str] = &[
        "switch to ",
        "focus ",
        "go to ",
        "vai para ",
        "muda para ",
        "troca para ",
        "foca ",
        "focar ",
    ];

    if let Some(value) = SWITCH_PREFIXES
        .iter()
        .find_map(|prefix| input.strip_prefix(prefix))
    {
        return Some((AppOperation::Switch, strip_article(value)));
    }

    if let Some(value) = LAUNCH_PREFIXES
        .iter()
        .find_map(|prefix| input.strip_prefix(prefix))
    {
        return Some((AppOperation::Launch, strip_article(value)));
    }

    CLOSE_PREFIXES
        .iter()
        .find_map(|prefix| input.strip_prefix(prefix))
        .map(|value| (AppOperation::Close, strip_article(value)))
}

pub fn route_command(input: &str, policy: &PermissionPolicy) -> RouteResult {
    if let Some(action) = system_request(input) {
        let permission = permission_for_system(action);
        return RouteResult::Action(RoutedAction {
            intent: ActionIntent::System(action),
            permission,
            decision: policy.decision_for(permission),
        });
    }

    if let Some(media) = media_request(input) {
        return match media {
            Ok(ActionIntent::Media(action)) => {
                let permission = permission_for_media(action);
                RouteResult::Action(RoutedAction {
                    intent: ActionIntent::Media(action),
                    permission,
                    decision: policy.decision_for(permission),
                })
            }
            Ok(_) => unreachable!("media_request should only produce media intents"),
            Err(message) => RouteResult::InvalidMedia(message),
        };
    }

    if let Some(mouse) = mouse_request(input) {
        return match mouse {
            Ok(ActionIntent::Mouse(action)) => {
                let permission = permission_for_mouse(action);
                RouteResult::Action(RoutedAction {
                    intent: ActionIntent::Mouse(action),
                    permission,
                    decision: policy.decision_for(permission),
                })
            }
            Ok(_) => unreachable!("mouse_request should only produce mouse intents"),
            Err(message) => RouteResult::InvalidMouse(message),
        };
    }

    if let Some(keyboard) = keyboard_request(input) {
        return match keyboard {
            Ok(intent) => {
                let permission = permission_for_keyboard(&intent)
                    .expect("keyboard intent should have a permission class");

                RouteResult::Action(RoutedAction {
                    intent,
                    permission,
                    decision: policy.decision_for(permission),
                })
            }
            Err(message) => RouteResult::InvalidKeyboard(message),
        };
    }

    if let Some(intent) = obs_source_visibility_request(input) {
        let permission = PermissionClass::Act;
        return RouteResult::Action(RoutedAction {
            intent,
            permission,
            decision: policy.decision_for(permission),
        });
    }

    if is_obs_stream_duration_request(input) {
        let permission = PermissionClass::Read;
        return RouteResult::Action(RoutedAction {
            intent: ActionIntent::ObsStreamDuration,
            permission,
            decision: policy.decision_for(permission),
        });
    }

    if let Some(action) = obs_streaming_request(input) {
        let permission = permission_for_obs_streaming(action);
        return RouteResult::Action(RoutedAction {
            intent: ActionIntent::ObsStreaming(action),
            permission,
            decision: policy.decision_for(permission),
        });
    }

    if let Some(action) = obs_recording_request(input) {
        let permission = PermissionClass::Act;
        return RouteResult::Action(RoutedAction {
            intent: ActionIntent::ObsRecording(action),
            permission,
            decision: policy.decision_for(permission),
        });
    }

    if let Some(intent) = obs_scene_request(input) {
        let permission = PermissionClass::Act;
        return RouteResult::Action(RoutedAction {
            intent,
            permission,
            decision: policy.decision_for(permission),
        });
    }

    let normalized = normalize_command(input);

    if is_list_windows_command(&normalized) {
        let permission = PermissionClass::Read;
        return RouteResult::Action(RoutedAction {
            intent: ActionIntent::ListWindows,
            permission,
            decision: policy.decision_for(permission),
        });
    }

    let Some((operation, target_text)) = app_request(&normalized) else {
        return RouteResult::NoMatch;
    };

    if target_text.is_empty() {
        return RouteResult::NoMatch;
    }

    let Some(target) = AppTarget::from_alias(target_text) else {
        if matches!(operation, AppOperation::Switch) {
            let permission = PermissionClass::Act;
            return RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsProgramScene(target_text.to_string()),
                permission,
                decision: policy.decision_for(permission),
            });
        }

        return RouteResult::UnsupportedApp(target_text.to_string());
    };

    let (intent, permission) = match operation {
        AppOperation::Launch => (
            ActionIntent::LaunchApp(target),
            PermissionClass::Act,
        ),
        AppOperation::Close => (
            ActionIntent::CloseApp(target),
            PermissionClass::Modify,
        ),
        AppOperation::Switch => (
            ActionIntent::SwitchToApp(target),
            PermissionClass::Act,
        ),
    };

    RouteResult::Action(RoutedAction {
        intent,
        permission,
        decision: policy.decision_for(permission),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::computer::{
        keyboard::{KeyCode, ModifierKey},
        mouse::{MouseAction, MouseButton, ScreenPoint},
    };

    #[test]
    fn routes_system_status_as_read() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("System status", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::System(SystemAction::GetStatus),
                permission: PermissionClass::Read,
                decision: PermissionDecision::Allow,
            })
        ));
    }

    #[test]
    fn routes_display_settings_as_act() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Open display settings", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::System(SystemAction::OpenSettings(SettingsPage::Display)),
                permission: PermissionClass::Act,
                decision: PermissionDecision::Allow,
            })
        ));
    }

    #[test]
    fn routes_lock_as_sensitive() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Lock PC", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::System(SystemAction::Lock),
                permission: PermissionClass::Sensitive,
                decision: PermissionDecision::Ask,
            })
        ));
    }

    #[test]
    fn routes_shutdown_as_destructive() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Shut down PC", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::System(SystemAction::Shutdown),
                permission: PermissionClass::Destructive,
                decision: PermissionDecision::Ask,
            })
        ));
    }

    #[test]
    fn routes_exact_volume_as_act() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Set volume to 30%", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::Media(MediaAction::SetVolume(30)),
                permission: PermissionClass::Act,
                decision: PermissionDecision::Allow,
            })
        ));
    }

    #[test]
    fn routes_volume_query_as_read() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Qual é o volume?", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::Media(MediaAction::GetVolume),
                permission: PermissionClass::Read,
                decision: PermissionDecision::Allow,
            })
        ));
    }

    #[test]
    fn rejects_invalid_volume() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Set volume to 140%", &policy),
            RouteResult::InvalidMedia(_)
        ));
    }

    #[test]
    fn routes_play_pause_as_act() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Pause music", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::Media(MediaAction::PlayPause),
                permission: PermissionClass::Act,
                ..
            })
        ));
    }

    #[test]
    fn routes_mouse_move_as_act() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Move mouse to 500, 300", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::Mouse(MouseAction::MoveTo(ScreenPoint { x: 500, y: 300 })),
                permission: PermissionClass::Act,
                ..
            })
        ));
    }

    #[test]
    fn routes_click_as_modify() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Click", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::Mouse(MouseAction::Click { button: MouseButton::Left }),
                permission: PermissionClass::Modify,
                decision: PermissionDecision::Ask,
            })
        ));
    }

    #[test]
    fn routes_delete_as_destructive() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Press Delete", &policy),
            RouteResult::Action(RoutedAction {
                permission: PermissionClass::Destructive,
                decision: PermissionDecision::Ask,
                ..
            })
        ));
    }

    #[test]
    fn preserves_typed_text_case() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Type \"Hello AURA\"", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::TypeText(text),
                permission: PermissionClass::Modify,
                ..
            }) if text == "Hello AURA"
        ));
    }


    #[test]
    fn routes_obs_source_visibility_as_act() {
        let policy = PermissionPolicy::default();

        assert!(matches!(
            route_command("Hide Scoreboard", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsSourceVisibility { source_name, enabled: false },
                permission: PermissionClass::Act,
                decision: PermissionDecision::Allow,
            }) if source_name == "Scoreboard"
        ));

        assert!(matches!(
            route_command("Show Lower Third", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsSourceVisibility { source_name, enabled: true },
                permission: PermissionClass::Act,
                ..
            }) if source_name == "Lower Third"
        ));

        assert!(matches!(
            route_command("Esconde Marcador", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsSourceVisibility { source_name, enabled: false },
                permission: PermissionClass::Act,
                ..
            }) if source_name == "Marcador"
        ));
    }

    #[test]
    fn routes_obs_stream_duration_as_read() {
        let policy = PermissionPolicy::default();

        for command in [
            "How long have we been live?",
            "Stream duration",
            "Há quanto tempo estamos em direto?",
        ] {
            assert!(matches!(
                route_command(command, &policy),
                RouteResult::Action(RoutedAction {
                    intent: ActionIntent::ObsStreamDuration,
                    permission: PermissionClass::Read,
                    decision: PermissionDecision::Allow,
                })
            ));
        }
    }

    #[test]
    fn routes_obs_streaming_controls_with_safe_permissions() {
        let policy = PermissionPolicy::default();

        assert!(matches!(
            route_command("Go live", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsStreaming(ObsStreamingAction::Start),
                permission: PermissionClass::Sensitive,
                decision: PermissionDecision::Ask,
            })
        ));

        assert!(matches!(
            route_command("Stop the stream", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsStreaming(ObsStreamingAction::Stop),
                permission: PermissionClass::Act,
                decision: PermissionDecision::Allow,
            })
        ));

        assert!(matches!(
            route_command("Entra em direto", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsStreaming(ObsStreamingAction::Start),
                permission: PermissionClass::Sensitive,
                ..
            })
        ));
    }

    #[test]
    fn routes_obs_recording_controls_as_act() {
        let policy = PermissionPolicy::default();

        for (command, expected) in [
            ("Start recording", ObsRecordingAction::Start),
            ("Stop recording", ObsRecordingAction::Stop),
            ("Pause recording", ObsRecordingAction::Pause),
            ("Resume recording", ObsRecordingAction::Resume),
            ("Inicia a gravação", ObsRecordingAction::Start),
            ("Para a gravação", ObsRecordingAction::Stop),
        ] {
            assert!(matches!(
                route_command(command, &policy),
                RouteResult::Action(RoutedAction {
                    intent: ActionIntent::ObsRecording(action),
                    permission: PermissionClass::Act,
                    decision: PermissionDecision::Allow,
                }) if action == expected
            ));
        }
    }

    #[test]
    fn routes_explicit_obs_program_scene_as_act() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Switch scene to Camera 2", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsProgramScene(scene),
                permission: PermissionClass::Act,
                decision: PermissionDecision::Allow,
            }) if scene == "Camera 2"
        ));
    }

    #[test]
    fn routes_obs_preview_scene_as_act() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Set preview scene to Interview", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsPreviewScene(scene),
                permission: PermissionClass::Act,
                ..
            }) if scene == "Interview"
        ));
    }

    #[test]
    fn unknown_switch_target_falls_back_to_obs_program_scene() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Switch to Camera 2", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsProgramScene(scene),
                permission: PermissionClass::Act,
                ..
            }) if scene == "camera 2"
        ));
    }
}
