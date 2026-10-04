use crate::computer::{
    app_launcher::AppTarget,
    app_skills::{BrowserSkill, BrowserSkillAction},
    audio::MediaAction,
    file_intelligence::{FileCategory, PersonalRootFilter, RecentFileQuery},
    keyboard::KeyboardShortcut,
    mouse::{parse_point, validate_scroll_notches, MouseAction, MouseButton},
    system::{SettingsPage, SystemAction},
    window_manager::WindowDisplayAction,
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
    SetAppWindowState {
        target: AppTarget,
        action: WindowDisplayAction,
    },
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
    ObsAudioMute { input_name: String, muted: bool },
    ObsAudioVolume { input_name: String, percent: u8 },
    ObsProductionHealth,
    DirectorPreset(String),
    MemoryRemember(String),
    MemoryList,
    MemoryForget(String),
    CurrentApp,
    ActiveWindow,
    RecentFiles,
    ClipboardRead,
    ClipboardWrite(String),
    ClipboardClear,
    FindPersonalFiles(String),
    FindRecentPersonalFiles(RecentFileQuery),
    RevealPersonalPath(String),
    BrowserSkill(BrowserSkill),
    UserRoutine(String),
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
    InvalidAppSkill(String),
    InvalidMouse(String),
    InvalidMedia(String),
    NoMatch,
}

#[derive(Debug, Clone, Copy)]
enum AppOperation {
    Launch,
    Close,
    Switch,
    Minimize,
    Maximize,
    Restore,
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

fn is_active_window_request(input: &str) -> bool {
    let normalized = normalize_command(input);

    matches!(
        normalized.as_str(),
        "active window"
            | "current window"
            | "window context"
            | "active window context"
            | "what window am i in"
            | "which window am i in"
            | "what window is active"
            | "which window is active"
            | "what is the active window"
            | "what am i working on"
            | "current tab"
            | "active tab"
            | "what tab am i on"
            | "janela atual"
            | "janela ativa"
            | "contexto da janela"
            | "qual é a janela atual"
            | "qual e a janela atual"
            | "qual é a janela ativa"
            | "qual e a janela ativa"
            | "em que janela estou"
            | "em que separador estou"
            | "qual é o separador ativo"
            | "qual e o separador ativo"
    )
}

fn is_recent_files_request(input: &str) -> bool {
    let normalized = normalize_command(input);

    matches!(
        normalized.as_str(),
        "recent files"
            | "recent file"
            | "recent documents"
            | "recent items"
            | "show recent files"
            | "list recent files"
            | "what files did i use recently"
            | "what have i been working on"
            | "ficheiros recentes"
            | "ficheiro recente"
            | "documentos recentes"
            | "itens recentes"
            | "mostra os ficheiros recentes"
            | "lista os ficheiros recentes"
            | "que ficheiros usei recentemente"
            | "em que ficheiros estive a trabalhar"
    )
}

fn is_current_app_request(input: &str) -> bool {
    let normalized = normalize_command(input);

    matches!(
        normalized.as_str(),
        "current app"
            | "current application"
            | "active app"
            | "active application"
            | "what app am i using"
            | "what application am i using"
            | "which app am i using"
            | "which application is active"
            | "what app is active"
            | "qual é a app atual"
            | "qual e a app atual"
            | "qual é a aplicação atual"
            | "qual e a aplicacao atual"
            | "que app estou a usar"
            | "que aplicação estou a usar"
            | "que aplicacao estou a usar"
            | "em que app estou"
            | "em que aplicação estou"
            | "em que aplicacao estou"
    )
}

fn memory_request(input: &str) -> Option<ActionIntent> {
    let normalized = normalize_command(input);

    if matches!(
        normalized.as_str(),
        "what do you remember"
            | "what do you remember about me"
            | "list memories"
            | "show memories"
            | "show memory"
            | "memory"
            | "o que te lembras"
            | "o que te lembras de mim"
            | "lista as memórias"
            | "lista as memorias"
            | "mostra as memórias"
            | "mostra as memorias"
            | "memórias"
            | "memorias"
    ) {
        return Some(ActionIntent::MemoryList);
    }

    const REMEMBER_PREFIXES: &[&str] = &[
        "remember that ",
        "remember ",
        "remember this: ",
        "lembra-te que ",
        "lembra te que ",
        "lembra que ",
        "guarda na memória ",
        "guarda na memoria ",
        "memoriza ",
    ];

    if let Some(value) = value_after_prefix(input, REMEMBER_PREFIXES) {
        let content = unwrap_text_quotes(value).trim();
        if !content.is_empty() {
            return Some(ActionIntent::MemoryRemember(content.to_string()));
        }
    }

    const FORGET_PREFIXES: &[&str] = &[
        "forget that ",
        "forget ",
        "esquece que ",
        "esquece ",
        "apaga da memória ",
        "apaga da memoria ",
        "remove da memória ",
        "remove da memoria ",
    ];

    if let Some(value) = value_after_prefix(input, FORGET_PREFIXES) {
        let content = unwrap_text_quotes(value).trim();
        if !content.is_empty() {
            return Some(ActionIntent::MemoryForget(content.to_string()));
        }
    }

    None
}

fn browser_skill_request(input: &str) -> Option<Result<ActionIntent, String>> {
    const NEW_TAB_PREFIXES: &[&str] = &[
        "new tab in ",
        "open new tab in ",
        "new tab on ",
        "nova aba no ",
        "nova aba no navegador ",
        "novo separador no ",
        "abre novo separador no ",
        "abrir novo separador no ",
    ];
    const NEXT_TAB_PREFIXES: &[&str] = &[
        "next tab in ",
        "next tab on ",
        "próximo separador no ",
        "proximo separador no ",
        "separador seguinte no ",
    ];
    const PREVIOUS_TAB_PREFIXES: &[&str] = &[
        "previous tab in ",
        "previous tab on ",
        "prev tab in ",
        "separador anterior no ",
        "separador anterior em ",
    ];
    const FOCUS_ADDRESS_PREFIXES: &[&str] = &[
        "focus address bar in ",
        "focus url bar in ",
        "focus address bar on ",
        "foca a barra de endereços no ",
        "foca a barra de enderecos no ",
        "foca a barra de url no ",
    ];
    const REOPEN_PREFIXES: &[&str] = &[
        "reopen closed tab in ",
        "reopen last closed tab in ",
        "reabrir separador fechado no ",
        "reabre o separador fechado no ",
        "reabre separador fechado no ",
    ];

    fn build(target_text: &str, action: BrowserSkillAction) -> Result<ActionIntent, String> {
        let normalized_target = target_text.trim().to_lowercase();
        let target_text = strip_article(&normalized_target);
        let Some(target) = AppTarget::from_alias(target_text) else {
            return Err(format!(
                "Unknown browser target “{}”. Browser Skills V1 supports Brave and Chrome.",
                target_text
            ));
        };

        BrowserSkill::new(target, action)
            .map(ActionIntent::BrowserSkill)
    }

    for (prefixes, action) in [
        (NEW_TAB_PREFIXES, BrowserSkillAction::NewTab),
        (NEXT_TAB_PREFIXES, BrowserSkillAction::NextTab),
        (PREVIOUS_TAB_PREFIXES, BrowserSkillAction::PreviousTab),
        (FOCUS_ADDRESS_PREFIXES, BrowserSkillAction::FocusAddressBar),
        (REOPEN_PREFIXES, BrowserSkillAction::ReopenClosedTab),
    ] {
        if let Some(value) = value_after_prefix(input, prefixes) {
            return Some(build(value, action));
        }
    }

    let normalized = normalize_command(input);
    for (prefix, action) in [
        ("reload ", BrowserSkillAction::Reload),
        ("refresh ", BrowserSkillAction::Reload),
        ("recarrega ", BrowserSkillAction::Reload),
        ("recarregar ", BrowserSkillAction::Reload),
        ("atualiza ", BrowserSkillAction::Reload),
    ] {
        if let Some(value) = normalized.strip_prefix(prefix) {
            return Some(build(value, action));
        }
    }

    None
}

fn recent_file_request(input: &str) -> Option<ActionIntent> {
    let normalized = normalize_command(input);

    let request = match normalized.as_str() {
        "latest file" | "last file" | "most recent file" | "newest file"
        | "último ficheiro" | "ultimo ficheiro" | "ficheiro mais recente" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Any,
                PersonalRootFilter::All,
                1,
            ))
        }
        "recent files" | "latest files" | "newest files"
        | "ficheiros recentes" | "últimos ficheiros" | "ultimos ficheiros" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Any,
                PersonalRootFilter::All,
                10,
            ))
        }

        "latest video" | "last video" | "most recent video" | "newest video"
        | "latest video i exported" | "last video i exported"
        | "most recent video i exported"
        | "último vídeo" | "ultimo video" | "vídeo mais recente" | "video mais recente"
        | "último vídeo que exportei" | "ultimo video que exportei" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Video,
                PersonalRootFilter::All,
                1,
            ))
        }
        "recent videos" | "latest videos" | "newest videos"
        | "vídeos recentes" | "videos recentes" | "últimos vídeos" | "ultimos videos" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Video,
                PersonalRootFilter::All,
                10,
            ))
        }

        "latest image" | "last image" | "most recent image" | "newest image"
        | "latest picture" | "last picture"
        | "última imagem" | "ultima imagem" | "imagem mais recente"
        | "última fotografia" | "ultima fotografia" | "fotografia mais recente" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Image,
                PersonalRootFilter::All,
                1,
            ))
        }
        "recent images" | "latest images" | "newest images"
        | "recent pictures" | "latest pictures"
        | "imagens recentes" | "últimas imagens" | "ultimas imagens"
        | "fotografias recentes" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Image,
                PersonalRootFilter::All,
                10,
            ))
        }

        "latest audio" | "last audio" | "most recent audio"
        | "latest audio file" | "last audio file"
        | "último áudio" | "ultimo audio" | "áudio mais recente" | "audio mais recente" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Audio,
                PersonalRootFilter::All,
                1,
            ))
        }
        "recent audio" | "recent audio files" | "latest audio files"
        | "áudios recentes" | "audios recentes" | "ficheiros de áudio recentes"
        | "ficheiros de audio recentes" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Audio,
                PersonalRootFilter::All,
                10,
            ))
        }

        "latest document" | "last document" | "most recent document"
        | "último documento" | "ultimo documento" | "documento mais recente" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Document,
                PersonalRootFilter::All,
                1,
            ))
        }
        "recent documents" | "latest documents" | "newest documents"
        | "documentos recentes" | "últimos documentos" | "ultimos documentos" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Document,
                PersonalRootFilter::All,
                10,
            ))
        }

        "latest archive" | "last archive" | "most recent archive"
        | "último arquivo" | "ultimo arquivo" | "arquivo mais recente"
        | "último ficheiro comprimido" | "ultimo ficheiro comprimido" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Archive,
                PersonalRootFilter::All,
                1,
            ))
        }
        "recent archives" | "latest archives"
        | "arquivos recentes" | "ficheiros comprimidos recentes" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Archive,
                PersonalRootFilter::All,
                10,
            ))
        }

        "latest download" | "last download" | "most recent download" | "newest download"
        | "último download" | "ultimo download" | "download mais recente" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Any,
                PersonalRootFilter::Downloads,
                1,
            ))
        }
        "recent downloads" | "latest downloads" | "newest downloads"
        | "downloads recentes" | "últimos downloads" | "ultimos downloads" => {
            Some(RecentFileQuery::bounded(
                FileCategory::Any,
                PersonalRootFilter::Downloads,
                10,
            ))
        }

        _ => None,
    };

    request.map(ActionIntent::FindRecentPersonalFiles)
}

fn file_reveal_request(input: &str) -> Option<ActionIntent> {
    const PREFIXES: &[&str] = &[
        "reveal file ",
        "reveal folder ",
        "show file in explorer ",
        "show folder in explorer ",
        "show in explorer ",
        "mostrar ficheiro no explorador ",
        "mostrar pasta no explorador ",
        "mostra ficheiro no explorador ",
        "mostra pasta no explorador ",
        "revela ficheiro ",
        "revela pasta ",
    ];

    value_after_prefix(input, PREFIXES).and_then(|value| {
        let path = unwrap_text_quotes(value).trim();
        (!path.is_empty()).then(|| ActionIntent::RevealPersonalPath(path.to_string()))
    })
}

fn file_search_request(input: &str) -> Option<ActionIntent> {
    const PREFIXES: &[&str] = &[
        "find file ",
        "find files ",
        "search file ",
        "search files ",
        "look for file ",
        "look for files ",
        "encontra ficheiro ",
        "encontrar ficheiro ",
        "procura ficheiro ",
        "procurar ficheiro ",
        "pesquisa ficheiro ",
        "pesquisar ficheiro ",
        "procura ficheiros ",
        "pesquisa ficheiros ",
    ];

    value_after_prefix(input, PREFIXES).and_then(|value| {
        let query = unwrap_text_quotes(value).trim();
        (!query.is_empty()).then(|| ActionIntent::FindPersonalFiles(query.to_string()))
    })
}

fn clipboard_request(input: &str) -> Option<ActionIntent> {
    let normalized = normalize_command(input);

    if matches!(
        normalized.as_str(),
        "read clipboard"
            | "show clipboard"
            | "show my clipboard"
            | "what is in my clipboard"
            | "what's in my clipboard"
            | "what is on my clipboard"
            | "clipboard contents"
            | "lê o clipboard"
            | "le o clipboard"
            | "mostra o clipboard"
            | "o que está no clipboard"
            | "o que esta no clipboard"
            | "lê a área de transferência"
            | "le a area de transferencia"
            | "mostra a área de transferência"
            | "mostra a area de transferencia"
            | "o que está na área de transferência"
            | "o que esta na area de transferencia"
    ) {
        return Some(ActionIntent::ClipboardRead);
    }

    if matches!(
        normalized.as_str(),
        "clear clipboard"
            | "empty clipboard"
            | "clear my clipboard"
            | "limpa o clipboard"
            | "limpar o clipboard"
            | "esvazia o clipboard"
            | "limpa a área de transferência"
            | "limpa a area de transferencia"
            | "esvazia a área de transferência"
            | "esvazia a area de transferencia"
    ) {
        return Some(ActionIntent::ClipboardClear);
    }

    const WRITE_PREFIXES: &[&str] = &[
        "copy to clipboard ",
        "set clipboard to ",
        "put on clipboard ",
        "put in clipboard ",
        "copia para o clipboard ",
        "copiar para o clipboard ",
        "coloca no clipboard ",
        "colocar no clipboard ",
        "guarda no clipboard ",
        "guardar no clipboard ",
        "copia para a área de transferência ",
        "copia para a area de transferencia ",
        "coloca na área de transferência ",
        "coloca na area de transferencia ",
    ];

    value_after_prefix(input, WRITE_PREFIXES).and_then(|value| {
        let text = unwrap_text_quotes(value).trim();
        (!text.is_empty()).then(|| ActionIntent::ClipboardWrite(text.to_string()))
    })
}

fn permission_for_clipboard(intent: &ActionIntent) -> Option<PermissionClass> {
    match intent {
        ActionIntent::ClipboardRead => Some(PermissionClass::Sensitive),
        ActionIntent::ClipboardWrite(_) => Some(PermissionClass::Modify),
        ActionIntent::ClipboardClear => Some(PermissionClass::Destructive),
        _ => None,
    }
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

fn obs_audio_request(input: &str) -> Option<Result<ActionIntent, String>> {
    const MUTE_PREFIXES: &[&str] = &[
        "mute input ",
        "mute ",
        "silencia o input ",
        "silencia input ",
        "silencia ",
        "silenciar ",
    ];

    const UNMUTE_PREFIXES: &[&str] = &[
        "unmute input ",
        "unmute ",
        "tira o mute de ",
        "tirar o mute de ",
        "reativa o som de ",
        "reativa som de ",
        "ativa o som de ",
    ];

    if let Some(value) = value_after_prefix(input, MUTE_PREFIXES) {
        let input_name = unwrap_text_quotes(value).trim();
        if input_name.is_empty() {
            return Some(Err("OBS audio input name cannot be empty.".to_string()));
        }

        return Some(Ok(ActionIntent::ObsAudioMute {
            input_name: input_name.to_string(),
            muted: true,
        }));
    }

    if let Some(value) = value_after_prefix(input, UNMUTE_PREFIXES) {
        let input_name = unwrap_text_quotes(value).trim();
        if input_name.is_empty() {
            return Some(Err("OBS audio input name cannot be empty.".to_string()));
        }

        return Some(Ok(ActionIntent::ObsAudioMute {
            input_name: input_name.to_string(),
            muted: false,
        }));
    }

    fn parse_named_volume(
        input: &str,
        prefix: &str,
        separator: &str,
    ) -> Option<Result<ActionIntent, String>> {
        let rest = input.strip_prefix(prefix)?;
        let (name, value) = rest.rsplit_once(separator)?;
        let input_name = unwrap_text_quotes(name).trim();

        if input_name.is_empty() {
            return Some(Err("OBS audio input name cannot be empty.".to_string()));
        }

        Some(parse_percent(value).map(|percent| ActionIntent::ObsAudioVolume {
            input_name: input_name.to_string(),
            percent,
        }))
    }

    let normalized = input.trim().to_lowercase();

    for (prefix, separator) in [
        ("set volume of ", " to "),
        ("set ", " volume to "),
        ("set obs input ", " to "),
        ("define o volume de ", " para "),
        ("define volume de ", " para "),
        ("define ", " para "),
        ("mete ", " a "),
    ] {
        if let Some(result) = parse_named_volume(&normalized, prefix, separator) {
            let original_name = match &result {
                Ok(ActionIntent::ObsAudioVolume { input_name, .. }) => input_name.clone(),
                _ => String::new(),
            };

            if original_name.is_empty() {
                return Some(result);
            }

            let prefix_len = prefix.len();
            let original_rest = input.trim().get(prefix_len..)?;
            let (original_name, _) = original_rest.rsplit_once(separator)?;
            return Some(result.map(|intent| match intent {
                ActionIntent::ObsAudioVolume { percent, .. } => ActionIntent::ObsAudioVolume {
                    input_name: unwrap_text_quotes(original_name).trim().to_string(),
                    percent,
                },
                other => other,
            }));
        }
    }

    if normalized.starts_with("set ") {
        let rest = input.trim().get(4..)?;
        if let Some((name, value)) = rest.rsplit_once(" to ") {
            if value.trim().ends_with('%') {
                let input_name = unwrap_text_quotes(name).trim();
                if input_name.is_empty() {
                    return Some(Err("OBS audio input name cannot be empty.".to_string()));
                }

                return Some(parse_percent(value).map(|percent| ActionIntent::ObsAudioVolume {
                    input_name: input_name.to_string(),
                    percent,
                }));
            }
        }
    }

    None
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

fn is_obs_production_health_request(input: &str) -> bool {
    let normalized = normalize_command(input);

    matches!(
        normalized.as_str(),
        "check production health"
            | "production health"
            | "check obs health"
            | "obs health"
            | "how is the production"
            | "how is production health"
            | "is the production healthy"
            | "verifica a saúde da produção"
            | "verifica a saude da producao"
            | "saúde da produção"
            | "saude da producao"
            | "estado da produção"
            | "estado da producao"
            | "como está a produção"
            | "como esta a producao"
            | "verifica o obs"
    )
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
    const MINIMIZE_PREFIXES: &[&str] = &[
        "minimize ",
        "minimise ",
        "minimiza ",
        "minimizar ",
    ];

    const MAXIMIZE_PREFIXES: &[&str] = &[
        "maximize ",
        "maximise ",
        "maximiza ",
        "maximizar ",
    ];

    const RESTORE_PREFIXES: &[&str] = &[
        "restore ",
        "restaura ",
        "restaurar ",
    ];

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

    for (operation, prefixes) in [
        (AppOperation::Minimize, MINIMIZE_PREFIXES),
        (AppOperation::Maximize, MAXIMIZE_PREFIXES),
        (AppOperation::Restore, RESTORE_PREFIXES),
        (AppOperation::Switch, SWITCH_PREFIXES),
        (AppOperation::Launch, LAUNCH_PREFIXES),
        (AppOperation::Close, CLOSE_PREFIXES),
    ] {
        if let Some(value) = prefixes
            .iter()
            .find_map(|prefix| input.strip_prefix(prefix))
        {
            return Some((operation, strip_article(value)));
        }
    }

    None
}

pub fn route_command(input: &str, policy: &PermissionPolicy) -> RouteResult {
    if is_recent_files_request(input) {
        let permission = PermissionClass::Read;
        return RouteResult::Action(RoutedAction {
            intent: ActionIntent::RecentFiles,
            permission,
            decision: policy.decision_for(permission),
        });
    }

    if is_active_window_request(input) {
        let permission = PermissionClass::Read;
        return RouteResult::Action(RoutedAction {
            intent: ActionIntent::ActiveWindow,
            permission,
            decision: policy.decision_for(permission),
        });
    }

    if is_current_app_request(input) {
        let permission = PermissionClass::Read;
        return RouteResult::Action(RoutedAction {
            intent: ActionIntent::CurrentApp,
            permission,
            decision: policy.decision_for(permission),
        });
    }

    if let Some(intent) = memory_request(input) {
        let permission = match &intent {
            ActionIntent::MemoryList => PermissionClass::Read,
            ActionIntent::MemoryRemember(_) => PermissionClass::Modify,
            ActionIntent::MemoryForget(_) => PermissionClass::Destructive,
            _ => unreachable!(),
        };

        return RouteResult::Action(RoutedAction {
            intent,
            permission,
            decision: policy.decision_for(permission),
        });
    }

    if let Some(skill) = browser_skill_request(input) {
        return match skill {
            Ok(intent) => {
                let permission = PermissionClass::Act;
                RouteResult::Action(RoutedAction {
                    intent,
                    permission,
                    decision: policy.decision_for(permission),
                })
            }
            Err(message) => RouteResult::InvalidAppSkill(message),
        };
    }

    if let Some(intent) = recent_file_request(input) {
        let permission = PermissionClass::Read;
        return RouteResult::Action(RoutedAction {
            intent,
            permission,
            decision: policy.decision_for(permission),
        });
    }

    if let Some(intent) = file_reveal_request(input) {
        let permission = PermissionClass::Act;
        return RouteResult::Action(RoutedAction {
            intent,
            permission,
            decision: policy.decision_for(permission),
        });
    }

    if let Some(intent) = file_search_request(input) {
        let permission = PermissionClass::Read;
        return RouteResult::Action(RoutedAction {
            intent,
            permission,
            decision: policy.decision_for(permission),
        });
    }

    if let Some(intent) = clipboard_request(input) {
        let permission = permission_for_clipboard(&intent)
            .expect("clipboard intent should have a permission class");
        return RouteResult::Action(RoutedAction {
            intent,
            permission,
            decision: policy.decision_for(permission),
        });
    }

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

    if is_obs_production_health_request(input) {
        let permission = PermissionClass::Read;
        return RouteResult::Action(RoutedAction {
            intent: ActionIntent::ObsProductionHealth,
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

    if let Some(audio) = obs_audio_request(input) {
        return match audio {
            Ok(intent) => {
                let permission = PermissionClass::Act;
                RouteResult::Action(RoutedAction {
                    intent,
                    permission,
                    decision: policy.decision_for(permission),
                })
            }
            Err(message) => RouteResult::InvalidMedia(message),
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
        AppOperation::Minimize => (
            ActionIntent::SetAppWindowState {
                target,
                action: WindowDisplayAction::Minimize,
            },
            PermissionClass::Act,
        ),
        AppOperation::Maximize => (
            ActionIntent::SetAppWindowState {
                target,
                action: WindowDisplayAction::Maximize,
            },
            PermissionClass::Act,
        ),
        AppOperation::Restore => (
            ActionIntent::SetAppWindowState {
                target,
                action: WindowDisplayAction::Restore,
            },
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
    fn routes_named_window_state_controls_as_act() {
        let policy = PermissionPolicy::default();

        assert!(matches!(
            route_command("Minimize Brave", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::SetAppWindowState {
                    target: AppTarget::Brave,
                    action: WindowDisplayAction::Minimize,
                },
                permission: PermissionClass::Act,
                decision: PermissionDecision::Allow,
            })
        ));

        assert!(matches!(
            route_command("Maximiza o OBS", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::SetAppWindowState {
                    target: AppTarget::ObsStudio,
                    action: WindowDisplayAction::Maximize,
                },
                permission: PermissionClass::Act,
                decision: PermissionDecision::Allow,
            })
        ));

        assert!(matches!(
            route_command("Restaura o Brave", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::SetAppWindowState {
                    target: AppTarget::Brave,
                    action: WindowDisplayAction::Restore,
                },
                permission: PermissionClass::Act,
                decision: PermissionDecision::Allow,
            })
        ));
    }

    #[test]
    fn unknown_window_state_target_is_not_guessed() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Minimize Photoshop", &policy),
            RouteResult::UnsupportedApp(name) if name == "photoshop"
        ));
    }

    #[test]
    fn browser_skill_targets_are_case_insensitive() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("New tab in GOOGLE CHROME", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::BrowserSkill(BrowserSkill {
                    target: AppTarget::Chrome,
                    action: BrowserSkillAction::NewTab,
                }),
                ..
            })
        ));
    }

    #[test]
    fn routes_brave_new_tab_as_app_skill() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("New tab in Brave", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::BrowserSkill(BrowserSkill {
                    target: AppTarget::Brave,
                    action: BrowserSkillAction::NewTab,
                }),
                permission: PermissionClass::Act,
                decision: PermissionDecision::Allow,
            })
        ));
    }

    #[test]
    fn routes_portuguese_chrome_address_bar_skill() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Foca a barra de endereços no Chrome", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::BrowserSkill(BrowserSkill {
                    target: AppTarget::Chrome,
                    action: BrowserSkillAction::FocusAddressBar,
                }),
                permission: PermissionClass::Act,
                ..
            })
        ));
    }

    #[test]
    fn rejects_browser_skill_for_non_browser_target() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("New tab in Notepad", &policy),
            RouteResult::InvalidAppSkill(_)
        ));
    }

    #[test]
    fn routes_latest_video_as_bounded_read_query() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Latest video I exported", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::FindRecentPersonalFiles(RecentFileQuery {
                    category: FileCategory::Video,
                    root: PersonalRootFilter::All,
                    limit: 1,
                }),
                permission: PermissionClass::Read,
                decision: PermissionDecision::Allow,
            })
        ));
    }

    #[test]
    fn routes_portuguese_recent_images() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Imagens recentes", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::FindRecentPersonalFiles(RecentFileQuery {
                    category: FileCategory::Image,
                    root: PersonalRootFilter::All,
                    limit: 10,
                }),
                permission: PermissionClass::Read,
                ..
            })
        ));
    }

    #[test]
    fn latest_download_is_scoped_to_downloads() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Último download", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::FindRecentPersonalFiles(RecentFileQuery {
                    category: FileCategory::Any,
                    root: PersonalRootFilter::Downloads,
                    limit: 1,
                }),
                permission: PermissionClass::Read,
                ..
            })
        ));
    }

    #[test]
    fn reveal_file_is_reversible_act_action() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command(
                "Reveal file \"C:\\Users\\Test\\Documents\\report.pdf\"",
                &policy
            ),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::RevealPersonalPath(path),
                permission: PermissionClass::Act,
                decision: PermissionDecision::Allow,
            }) if path.ends_with("Documents\\report.pdf")
        ));
    }

    #[test]
    fn file_search_is_read_only_and_preserves_query_case() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Find file \"WorldUnited Final.psd\"", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::FindPersonalFiles(query),
                permission: PermissionClass::Read,
                decision: PermissionDecision::Allow,
            }) if query == "WorldUnited Final.psd"
        ));
    }

    #[test]
    fn portuguese_file_search_routes_to_personal_search() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Procura ficheiro Artemis", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::FindPersonalFiles(query),
                permission: PermissionClass::Read,
                ..
            }) if query == "Artemis"
        ));
    }

    #[test]
    fn clipboard_read_is_sensitive_and_requires_confirmation() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("What's in my clipboard?", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ClipboardRead,
                permission: PermissionClass::Sensitive,
                decision: PermissionDecision::Ask,
            })
        ));
    }

    #[test]
    fn clipboard_write_preserves_case_and_is_modify() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Copy to clipboard \"Hello AURA 2\"", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ClipboardWrite(text),
                permission: PermissionClass::Modify,
                decision: PermissionDecision::Ask,
            }) if text == "Hello AURA 2"
        ));
    }

    #[test]
    fn clipboard_clear_is_destructive() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Limpa o clipboard", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ClipboardClear,
                permission: PermissionClass::Destructive,
                decision: PermissionDecision::Ask,
            })
        ));
    }

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
    fn source_visibility_does_not_override_window_discovery() {
        let policy = PermissionPolicy::default();

        assert!(matches!(
            route_command("Show windows", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ListWindows,
                permission: PermissionClass::Read,
                ..
            })
        ));
    }

    #[test]
    fn routes_obs_audio_controls_as_act() {
        let policy = PermissionPolicy::default();

        assert!(matches!(
            route_command("Mute Mic/Aux", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsAudioMute { input_name, muted: true },
                permission: PermissionClass::Act,
                decision: PermissionDecision::Allow,
            }) if input_name == "Mic/Aux"
        ));

        assert!(matches!(
            route_command("Unmute Desktop Audio", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsAudioMute { input_name, muted: false },
                permission: PermissionClass::Act,
                ..
            }) if input_name == "Desktop Audio"
        ));

        assert!(matches!(
            route_command("Set Mic/Aux to 70%", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsAudioVolume { input_name, percent: 70 },
                permission: PermissionClass::Act,
                ..
            }) if input_name == "Mic/Aux"
        ));

        assert!(matches!(
            route_command("Define Desktop Audio para 45%", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ObsAudioVolume { input_name, percent: 45 },
                permission: PermissionClass::Act,
                ..
            }) if input_name == "Desktop Audio"
        ));
    }

    #[test]
    fn obs_audio_controls_do_not_override_system_mute() {
        let policy = PermissionPolicy::default();

        assert!(matches!(
            route_command("Mute", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::Media(MediaAction::Mute),
                ..
            })
        ));
    }

    #[test]
    fn routes_recent_files_context_as_read() {
        let policy = PermissionPolicy::default();

        for command in [
            "Recent files",
            "What files did I use recently?",
            "Ficheiros recentes",
            "Mostra os ficheiros recentes",
        ] {
            assert!(matches!(
                route_command(command, &policy),
                RouteResult::Action(RoutedAction {
                    intent: ActionIntent::RecentFiles,
                    permission: PermissionClass::Read,
                    decision: PermissionDecision::Allow,
                })
            ));
        }
    }

    #[test]
    fn routes_active_window_context_as_read() {
        let policy = PermissionPolicy::default();

        for command in [
            "What window am I in?",
            "Active window",
            "Em que janela estou?",
            "Qual é o separador ativo?",
        ] {
            assert!(matches!(
                route_command(command, &policy),
                RouteResult::Action(RoutedAction {
                    intent: ActionIntent::ActiveWindow,
                    permission: PermissionClass::Read,
                    decision: PermissionDecision::Allow,
                })
            ));
        }
    }

    #[test]
    fn routes_current_app_awareness_as_read() {
        let policy = PermissionPolicy::default();

        for command in [
            "What app am I using?",
            "Current app",
            "Que aplicação estou a usar?",
        ] {
            assert!(matches!(
                route_command(command, &policy),
                RouteResult::Action(RoutedAction {
                    intent: ActionIntent::CurrentApp,
                    permission: PermissionClass::Read,
                    decision: PermissionDecision::Allow,
                })
            ));
        }
    }

    #[test]
    fn routes_explicit_memory_commands() {
        let policy = PermissionPolicy::default();

        assert!(matches!(
            route_command("Remember that I prefer Brave", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::MemoryRemember(content),
                permission: PermissionClass::Modify,
                decision: PermissionDecision::Ask,
            }) if content == "I prefer Brave"
        ));

        assert!(matches!(
            route_command("What do you remember?", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::MemoryList,
                permission: PermissionClass::Read,
                decision: PermissionDecision::Allow,
            })
        ));

        assert!(matches!(
            route_command("Forget that I prefer Brave", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::MemoryForget(content),
                permission: PermissionClass::Destructive,
                decision: PermissionDecision::Ask,
            }) if content == "I prefer Brave"
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
    fn routes_obs_production_health_as_read() {
        let policy = PermissionPolicy::default();

        for command in [
            "Check production health",
            "OBS health",
            "Verifica a saúde da produção",
        ] {
            assert!(matches!(
                route_command(command, &policy),
                RouteResult::Action(RoutedAction {
                    intent: ActionIntent::ObsProductionHealth,
                    permission: PermissionClass::Read,
                    decision: PermissionDecision::Allow,
                })
            ));
        }
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
