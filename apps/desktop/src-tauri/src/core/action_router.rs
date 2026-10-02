use crate::computer::{
    app_launcher::AppTarget,
    keyboard::KeyboardShortcut,
};
use crate::permissions::{PermissionClass, PermissionDecision, PermissionPolicy};

#[derive(Debug, Clone)]
pub enum ActionIntent {
    LaunchApp(AppTarget),
    CloseApp(AppTarget),
    SwitchToApp(AppTarget),
    ListWindows,
    PressShortcut(KeyboardShortcut),
    TypeText(String),
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
    NoMatch,
}

#[derive(Debug, Clone, Copy)]
enum AppOperation {
    Launch,
    Close,
    Switch,
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
    use crate::computer::keyboard::{KeyCode, ModifierKey};

    #[test]
    fn routes_english_obs_launch() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Open OBS", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::LaunchApp(AppTarget::ObsStudio),
                decision: PermissionDecision::Allow,
                ..
            })
        ));
    }

    #[test]
    fn routes_portuguese_close_as_modify() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Fecha o Terminal", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::CloseApp(AppTarget::WindowsTerminal),
                permission: PermissionClass::Modify,
                decision: PermissionDecision::Ask,
            })
        ));
    }

    #[test]
    fn routes_switch_to_obs_as_act() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Switch to OBS", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::SwitchToApp(AppTarget::ObsStudio),
                permission: PermissionClass::Act,
                decision: PermissionDecision::Allow,
            })
        ));
    }

    #[test]
    fn routes_safe_f11_as_act() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Press F11", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::PressShortcut(KeyboardShortcut {
                    key: KeyCode::Function(11),
                    ..
                }),
                permission: PermissionClass::Act,
                decision: PermissionDecision::Allow,
            })
        ));
    }

    #[test]
    fn routes_ctrl_s_as_modify() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Press Ctrl+S", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::PressShortcut(KeyboardShortcut {
                    modifiers,
                    key: KeyCode::Letter('s'),
                }),
                permission: PermissionClass::Modify,
                decision: PermissionDecision::Ask,
            }) if modifiers == vec![ModifierKey::Ctrl]
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
    fn routes_window_listing_as_read() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Que janelas estão abertas?", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::ListWindows,
                permission: PermissionClass::Read,
                decision: PermissionDecision::Allow,
            })
        ));
    }

    #[test]
    fn reports_unknown_apps() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("open something-unknown", &policy),
            RouteResult::UnsupportedApp(_)
        ));
    }
}
