use super::action_router::{route_command, ActionIntent, RouteResult, RoutedAction};
use crate::{
    computer::{
        app_launcher::AppTarget,
        audio::MediaAction,
        keyboard::{KeyCode, KeyboardShortcut, ModifierKey},
        mouse::{MouseAction, MouseButton, ScreenPoint},
        system::{SettingsPage, SystemAction},
    },
    permissions::{PermissionClass, PermissionDecision, PermissionPolicy},
};

fn assert_action(
    command: &str,
    policy: &PermissionPolicy,
    permission: PermissionClass,
    decision: PermissionDecision,
) -> ActionIntent {
    match route_command(command, policy) {
        RouteResult::Action(RoutedAction {
            intent,
            permission: actual_permission,
            decision: actual_decision,
        }) => {
            assert_eq!(actual_permission, permission, "permission mismatch for {command}");
            assert_eq!(actual_decision, decision, "decision mismatch for {command}");
            intent
        }
        other => panic!("expected routed action for {command:?}, got {other:?}"),
    }
}

#[test]
fn english_and_portuguese_app_launches_match() {
    let policy = PermissionPolicy::default();

    for command in ["Open OBS", "Abre o OBS"] {
        assert!(matches!(
            assert_action(
                command,
                &policy,
                PermissionClass::Act,
                PermissionDecision::Allow,
            ),
            ActionIntent::LaunchApp(AppTarget::ObsStudio)
        ));
    }

    for command in ["Launch Brave", "Abre o Brave"] {
        assert!(matches!(
            assert_action(
                command,
                &policy,
                PermissionClass::Act,
                PermissionDecision::Allow,
            ),
            ActionIntent::LaunchApp(AppTarget::Brave)
        ));
    }
}

#[test]
fn app_close_requires_confirmation_in_both_languages() {
    let policy = PermissionPolicy::default();

    for command in ["Close OBS", "Fecha o OBS"] {
        assert!(matches!(
            assert_action(
                command,
                &policy,
                PermissionClass::Modify,
                PermissionDecision::Ask,
            ),
            ActionIntent::CloseApp(AppTarget::ObsStudio)
        ));
    }
}

#[test]
fn window_switching_is_reversible_act() {
    let policy = PermissionPolicy::default();

    for command in ["Switch to Brave", "Vai para o Brave"] {
        assert!(matches!(
            assert_action(
                command,
                &policy,
                PermissionClass::Act,
                PermissionDecision::Allow,
            ),
            ActionIntent::SwitchToApp(AppTarget::Brave)
        ));
    }
}

#[test]
fn window_discovery_is_read_only() {
    let policy = PermissionPolicy::default();

    for command in ["List windows", "Que janelas estão abertas?"] {
        assert!(matches!(
            assert_action(
                command,
                &policy,
                PermissionClass::Read,
                PermissionDecision::Allow,
            ),
            ActionIntent::ListWindows
        ));
    }
}

#[test]
fn keyboard_risk_levels_are_stable() {
    let policy = PermissionPolicy::default();

    assert!(matches!(
        assert_action(
            "Press F11",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::PressShortcut(KeyboardShortcut {
            key: KeyCode::Function(11),
            ..
        })
    ));

    assert!(matches!(
        assert_action(
            "Press Ctrl+S",
            &policy,
            PermissionClass::Modify,
            PermissionDecision::Ask,
        ),
        ActionIntent::PressShortcut(KeyboardShortcut {
            modifiers,
            key: KeyCode::Letter('s'),
        }) if modifiers == vec![ModifierKey::Ctrl]
    ));

    assert!(matches!(
        assert_action(
            "Press Delete",
            &policy,
            PermissionClass::Destructive,
            PermissionDecision::Ask,
        ),
        ActionIntent::PressShortcut(_)
    ));
}

#[test]
fn typed_unicode_text_preserves_content_and_requires_confirmation() {
    let policy = PermissionPolicy::default();

    assert!(matches!(
        assert_action(
            "Escreve \"Olá AURA — ção\"",
            &policy,
            PermissionClass::Modify,
            PermissionDecision::Ask,
        ),
        ActionIntent::TypeText(text) if text == "Olá AURA — ção"
    ));
}

#[test]
fn mouse_movement_and_scroll_are_act_but_clicks_are_modify() {
    let policy = PermissionPolicy::default();

    assert!(matches!(
        assert_action(
            "Move mouse to -120, 640",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::Mouse(MouseAction::MoveTo(ScreenPoint { x: -120, y: 640 }))
    ));

    assert!(matches!(
        assert_action(
            "Scroll down 3",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::Mouse(MouseAction::Scroll { notches: -3 })
    ));

    assert!(matches!(
        assert_action(
            "Right click at 500, 300",
            &policy,
            PermissionClass::Modify,
            PermissionDecision::Ask,
        ),
        ActionIntent::Mouse(MouseAction::ClickAt {
            point: ScreenPoint { x: 500, y: 300 },
            button: MouseButton::Right,
        })
    ));
}

#[test]
fn media_commands_keep_read_and_act_boundaries() {
    let policy = PermissionPolicy::default();

    assert!(matches!(
        assert_action(
            "Qual é o volume?",
            &policy,
            PermissionClass::Read,
            PermissionDecision::Allow,
        ),
        ActionIntent::Media(MediaAction::GetVolume)
    ));

    assert!(matches!(
        assert_action(
            "Set volume to 35%",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::Media(MediaAction::SetVolume(35))
    ));

    assert!(matches!(
        assert_action(
            "Pausa a música",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::Media(MediaAction::PlayPause)
    ));
}

#[test]
fn system_commands_preserve_safety_classes() {
    let policy = PermissionPolicy::default();

    assert!(matches!(
        assert_action(
            "System status",
            &policy,
            PermissionClass::Read,
            PermissionDecision::Allow,
        ),
        ActionIntent::System(SystemAction::GetStatus)
    ));

    assert!(matches!(
        assert_action(
            "Open display settings",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::System(SystemAction::OpenSettings(SettingsPage::Display))
    ));

    assert!(matches!(
        assert_action(
            "Lock PC",
            &policy,
            PermissionClass::Sensitive,
            PermissionDecision::Ask,
        ),
        ActionIntent::System(SystemAction::Lock)
    ));

    assert!(matches!(
        assert_action(
            "Shut down PC",
            &policy,
            PermissionClass::Destructive,
            PermissionDecision::Ask,
        ),
        ActionIntent::System(SystemAction::Shutdown)
    ));
}

#[test]
fn policy_overrides_are_applied_by_router() {
    let mut policy = PermissionPolicy::default();

    policy
        .set(PermissionClass::Act, PermissionDecision::Never)
        .unwrap();
    assert!(matches!(
        assert_action(
            "Open OBS",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Never,
        ),
        ActionIntent::LaunchApp(AppTarget::ObsStudio)
    ));

    policy
        .set(PermissionClass::Modify, PermissionDecision::Allow)
        .unwrap();
    assert!(matches!(
        assert_action(
            "Click",
            &policy,
            PermissionClass::Modify,
            PermissionDecision::Allow,
        ),
        ActionIntent::Mouse(MouseAction::Click {
            button: MouseButton::Left
        })
    ));
}

#[test]
fn obs_scene_commands_route_through_act_permission() {
    let policy = PermissionPolicy::default();

    assert!(matches!(
        assert_action(
            "Switch scene to Camera 2",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::ObsProgramScene(scene) if scene == "Camera 2"
    ));

    assert!(matches!(
        assert_action(
            "Set preview scene to Interview",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::ObsPreviewScene(scene) if scene == "Interview"
    ));

    assert!(matches!(
        assert_action(
            "Switch to Camera 2",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::ObsProgramScene(scene) if scene == "camera 2"
    ));
}

#[test]
fn obs_scene_commands_respect_act_policy_override() {
    let mut policy = PermissionPolicy::default();
    policy
        .set(PermissionClass::Act, PermissionDecision::Ask)
        .unwrap();

    assert!(matches!(
        assert_action(
            "Muda para a cena Câmara 1",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Ask,
        ),
        ActionIntent::ObsProgramScene(scene) if scene == "Câmara 1"
    ));
}

#[test]
fn sensitive_and_destructive_cannot_be_permanently_allowed() {
    let mut policy = PermissionPolicy::default();

    assert!(policy
        .set(PermissionClass::Sensitive, PermissionDecision::Allow)
        .is_err());
    assert!(policy
        .set(PermissionClass::Destructive, PermissionDecision::Allow)
        .is_err());

    assert_eq!(policy.sensitive, PermissionDecision::Ask);
    assert_eq!(policy.destructive, PermissionDecision::Ask);
}

#[test]
fn invalid_inputs_fail_closed() {
    let policy = PermissionPolicy::default();

    assert!(matches!(
        route_command("Set volume to 101%", &policy),
        RouteResult::InvalidMedia(_)
    ));

    assert!(matches!(
        route_command("Scroll down 999", &policy),
        RouteResult::InvalidMouse(_)
    ));

    assert!(matches!(
        route_command("Press Ctrl+Shift", &policy),
        RouteResult::InvalidKeyboard(_)
    ));

    assert!(matches!(
        route_command("Open totally-unknown-app", &policy),
        RouteResult::UnsupportedApp(_)
    ));

    assert!(matches!(
        route_command("do something magical", &policy),
        RouteResult::NoMatch
    ));
}

#[test]
fn common_punctuation_and_case_do_not_break_deterministic_commands() {
    let policy = PermissionPolicy::default();

    assert!(matches!(
        assert_action(
            "  OPEN OBS!!!  ",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::LaunchApp(AppTarget::ObsStudio)
    ));

    assert!(matches!(
        assert_action(
            "VOLUME UP",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::Media(MediaAction::VolumeUp)
    ));
}
