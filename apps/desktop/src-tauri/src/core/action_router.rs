use crate::computer::app_launcher::AppTarget;
use crate::permissions::{PermissionClass, PermissionDecision, PermissionPolicy};

#[derive(Debug, Clone)]
pub enum ActionIntent {
    LaunchApp(AppTarget),
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

fn normalize_command(input: &str) -> String {
    input
        .trim()
        .trim_matches(|c: char| matches!(c, '.' | ',' | '!' | '?' | ';' | ':'))
        .to_lowercase()
}

fn app_phrase(input: &str) -> Option<&str> {
    const PREFIXES: &[&str] = &[
        "open ",
        "launch ",
        "start ",
        "abre ",
        "abrir ",
        "inicia ",
        "iniciar ",
    ];

    PREFIXES
        .iter()
        .find_map(|prefix| input.strip_prefix(prefix))
        .map(str::trim)
        .map(|value| {
            value
                .strip_prefix("the ")
                .or_else(|| value.strip_prefix("o "))
                .or_else(|| value.strip_prefix("a "))
                .unwrap_or(value)
                .trim()
        })
}

pub fn route_command(input: &str, policy: &PermissionPolicy) -> RouteResult {
    let normalized = normalize_command(input);
    let Some(target_text) = app_phrase(&normalized) else {
        return RouteResult::NoMatch;
    };

    if target_text.is_empty() {
        return RouteResult::NoMatch;
    }

    match AppTarget::from_alias(target_text) {
        Some(target) => {
            let permission = PermissionClass::Act;
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::LaunchApp(target),
                permission,
                decision: policy.decision_for(permission),
            })
        }
        None => RouteResult::UnsupportedApp(target_text.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_english_obs_command() {
        let policy = PermissionPolicy::default();
        let result = route_command("Open OBS", &policy);

        assert!(matches!(
            result,
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::LaunchApp(AppTarget::ObsStudio),
                ..
            })
        ));
    }

    #[test]
    fn routes_portuguese_brave_command() {
        let policy = PermissionPolicy::default();
        let result = route_command("Abre o Brave", &policy);

        assert!(matches!(
            result,
            RouteResult::Action(RoutedAction {
                intent: ActionIntent::LaunchApp(AppTarget::Brave),
                ..
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
