use crate::computer::app_launcher::AppTarget;
use crate::permissions::{PermissionClass, PermissionDecision, PermissionPolicy};

#[derive(Debug, Clone)]
pub enum ActionIntent {
    LaunchApp(AppTarget),
    CloseApp(AppTarget),
    SwitchToApp(AppTarget),
    ListWindows,
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
    fn routes_portuguese_switch() {
        let policy = PermissionPolicy::default();
        assert!(matches!(
            route_command("Vai para o Brave", &policy),
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::SwitchToApp(AppTarget::Brave),
                ..
            })
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
