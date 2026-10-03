use super::action_router::{route_command, ActionIntent, ObsRecordingAction, ObsStreamingAction, RouteResult, RoutedAction};
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
fn routine_intent_shape_remains_action_capable() {
    let intent = ActionIntent::UserRoutine("routine-start-editing".to_string());
    assert!(matches!(intent, ActionIntent::UserRoutine(id) if id == "routine-start-editing"));
}

#[test]
fn recent_files_context_is_read_only() {
    let policy = PermissionPolicy::default();

    for command in [
        "Recent files",
        "What files did I use recently?",
        "Ficheiros recentes",
    ] {
        assert!(matches!(
            assert_action(
                command,
                &policy,
                PermissionClass::Read,
                PermissionDecision::Allow,
            ),
            ActionIntent::RecentFiles
        ));
    }
}

#[test]
fn recent_files_context_respects_read_policy_override() {
    let mut policy = PermissionPolicy::default();
    policy
        .set(PermissionClass::Read, PermissionDecision::Ask)
        .unwrap();

    assert!(matches!(
        assert_action(
            "Show recent files",
            &policy,
            PermissionClass::Read,
            PermissionDecision::Ask,
        ),
        ActionIntent::RecentFiles
    ));
}

#[test]
fn active_window_context_is_read_only() {
    let policy = PermissionPolicy::default();

    for command in [
        "What window am I in?",
        "Active window",
        "Em que janela estou?",
    ] {
        assert!(matches!(
            assert_action(
                command,
                &policy,
                PermissionClass::Read,
                PermissionDecision::Allow,
            ),
            ActionIntent::ActiveWindow
        ));
    }
}

#[test]
fn active_window_context_respects_read_policy_override() {
    let mut policy = PermissionPolicy::default();
    policy
        .set(PermissionClass::Read, PermissionDecision::Ask)
        .unwrap();

    assert!(matches!(
        assert_action(
            "What window is active?",
            &policy,
            PermissionClass::Read,
            PermissionDecision::Ask,
        ),
        ActionIntent::ActiveWindow
    ));
}

#[test]
fn current_app_awareness_is_read_only() {
    let policy = PermissionPolicy::default();

    for command in [
        "What app am I using?",
        "Current app",
        "Que aplicação estou a usar?",
    ] {
        assert!(matches!(
            assert_action(
                command,
                &policy,
                PermissionClass::Read,
                PermissionDecision::Allow,
            ),
            ActionIntent::CurrentApp
        ));
    }
}

#[test]
fn current_app_awareness_respects_read_policy_override() {
    let mut policy = PermissionPolicy::default();
    policy
        .set(PermissionClass::Read, PermissionDecision::Ask)
        .unwrap();

    assert!(matches!(
        assert_action(
            "What application am I using?",
            &policy,
            PermissionClass::Read,
            PermissionDecision::Ask,
        ),
        ActionIntent::CurrentApp
    ));
}

#[test]
fn memory_commands_preserve_read_modify_and_destructive_boundaries() {
    let policy = PermissionPolicy::default();

    assert!(matches!(
        assert_action(
            "Remember that I prefer Brave",
            &policy,
            PermissionClass::Modify,
            PermissionDecision::Ask,
        ),
        ActionIntent::MemoryRemember(content) if content == "I prefer Brave"
    ));

    assert!(matches!(
        assert_action(
            "What do you remember?",
            &policy,
            PermissionClass::Read,
            PermissionDecision::Allow,
        ),
        ActionIntent::MemoryList
    ));

    assert!(matches!(
        assert_action(
            "Forget that I prefer Brave",
            &policy,
            PermissionClass::Destructive,
            PermissionDecision::Ask,
        ),
        ActionIntent::MemoryForget(content) if content == "I prefer Brave"
    ));
}

#[test]
fn memory_commands_respect_policy_overrides() {
    let mut policy = PermissionPolicy::default();
    policy
        .set(PermissionClass::Modify, PermissionDecision::Allow)
        .unwrap();
    policy
        .set(PermissionClass::Read, PermissionDecision::Ask)
        .unwrap();

    assert!(matches!(
        assert_action(
            "Lembra-te que uso Brave",
            &policy,
            PermissionClass::Modify,
            PermissionDecision::Allow,
        ),
        ActionIntent::MemoryRemember(content) if content == "uso Brave"
    ));

    assert!(matches!(
        assert_action(
            "Mostra as memórias",
            &policy,
            PermissionClass::Read,
            PermissionDecision::Ask,
        ),
        ActionIntent::MemoryList
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
fn obs_audio_commands_route_through_act_permission() {
    let policy = PermissionPolicy::default();

    assert!(matches!(
        assert_action(
            "Mute Mic/Aux",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::ObsAudioMute {
            input_name,
            muted: true,
        } if input_name == "Mic/Aux"
    ));

    assert!(matches!(
        assert_action(
            "Unmute Desktop Audio",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::ObsAudioMute {
            input_name,
            muted: false,
        } if input_name == "Desktop Audio"
    ));

    assert!(matches!(
        assert_action(
            "Set Mic/Aux to 70%",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::ObsAudioVolume {
            input_name,
            percent: 70,
        } if input_name == "Mic/Aux"
    ));

    assert!(matches!(
        assert_action(
            "Define Desktop Audio para 45%",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::ObsAudioVolume {
            input_name,
            percent: 45,
        } if input_name == "Desktop Audio"
    ));
}

#[test]
fn obs_audio_commands_respect_act_policy_override() {
    let mut policy = PermissionPolicy::default();
    policy
        .set(PermissionClass::Act, PermissionDecision::Ask)
        .unwrap();

    assert!(matches!(
        assert_action(
            "Mute Commentary",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Ask,
        ),
        ActionIntent::ObsAudioMute {
            input_name,
            muted: true,
        } if input_name == "Commentary"
    ));
}

#[test]
fn plain_mute_still_routes_to_system_media() {
    let policy = PermissionPolicy::default();

    assert!(matches!(
        assert_action(
            "Mute",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::Media(MediaAction::Mute)
    ));
}

#[test]
fn obs_source_visibility_commands_route_through_act_permission() {
    let policy = PermissionPolicy::default();

    assert!(matches!(
        assert_action(
            "Hide Scoreboard",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::ObsSourceVisibility {
            source_name,
            enabled: false,
        } if source_name == "Scoreboard"
    ));

    assert!(matches!(
        assert_action(
            "Show Lower Third",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::ObsSourceVisibility {
            source_name,
            enabled: true,
        } if source_name == "Lower Third"
    ));

    assert!(matches!(
        assert_action(
            "Esconde Marcador",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::ObsSourceVisibility {
            source_name,
            enabled: false,
        } if source_name == "Marcador"
    ));
}

#[test]
fn obs_source_visibility_respects_act_policy_override() {
    let mut policy = PermissionPolicy::default();
    policy
        .set(PermissionClass::Act, PermissionDecision::Ask)
        .unwrap();

    assert!(matches!(
        assert_action(
            "Mostra Lower Third",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Ask,
        ),
        ActionIntent::ObsSourceVisibility {
            source_name,
            enabled: true,
        } if source_name == "Lower Third"
    ));
}

#[test]
fn obs_production_health_queries_are_read_only() {
    let policy = PermissionPolicy::default();

    for command in [
        "Check production health",
        "OBS health",
        "Verifica a saúde da produção",
    ] {
        assert!(matches!(
            assert_action(
                command,
                &policy,
                PermissionClass::Read,
                PermissionDecision::Allow,
            ),
            ActionIntent::ObsProductionHealth
        ));
    }
}

#[test]
fn obs_production_health_respects_read_policy_override() {
    let mut policy = PermissionPolicy::default();
    policy
        .set(PermissionClass::Read, PermissionDecision::Ask)
        .unwrap();

    assert!(matches!(
        assert_action(
            "Check OBS health",
            &policy,
            PermissionClass::Read,
            PermissionDecision::Ask,
        ),
        ActionIntent::ObsProductionHealth
    ));
}

#[test]
fn obs_stream_duration_queries_are_read_only() {
    let policy = PermissionPolicy::default();

    for command in [
        "How long have we been live?",
        "Stream duration",
        "Há quanto tempo estamos em direto?",
    ] {
        assert!(matches!(
            assert_action(
                command,
                &policy,
                PermissionClass::Read,
                PermissionDecision::Allow,
            ),
            ActionIntent::ObsStreamDuration
        ));
    }
}

#[test]
fn obs_stream_duration_respects_read_policy_override() {
    let mut policy = PermissionPolicy::default();
    policy
        .set(PermissionClass::Read, PermissionDecision::Ask)
        .unwrap();

    assert!(matches!(
        assert_action(
            "What is the stream duration?",
            &policy,
            PermissionClass::Read,
            PermissionDecision::Ask,
        ),
        ActionIntent::ObsStreamDuration
    ));
}

#[test]
fn obs_streaming_start_is_sensitive_and_stop_is_act() {
    let policy = PermissionPolicy::default();

    assert!(matches!(
        assert_action(
            "Go live",
            &policy,
            PermissionClass::Sensitive,
            PermissionDecision::Ask,
        ),
        ActionIntent::ObsStreaming(ObsStreamingAction::Start)
    ));

    assert!(matches!(
        assert_action(
            "Stop the stream",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Allow,
        ),
        ActionIntent::ObsStreaming(ObsStreamingAction::Stop)
    ));

    assert!(matches!(
        assert_action(
            "Entra em direto",
            &policy,
            PermissionClass::Sensitive,
            PermissionDecision::Ask,
        ),
        ActionIntent::ObsStreaming(ObsStreamingAction::Start)
    ));
}

#[test]
fn obs_streaming_start_cannot_be_permanently_allowed() {
    let mut policy = PermissionPolicy::default();

    assert!(policy
        .set(PermissionClass::Sensitive, PermissionDecision::Allow)
        .is_err());

    assert!(matches!(
        assert_action(
            "Start streaming",
            &policy,
            PermissionClass::Sensitive,
            PermissionDecision::Ask,
        ),
        ActionIntent::ObsStreaming(ObsStreamingAction::Start)
    ));
}

#[test]
fn obs_recording_commands_route_through_act_permission() {
    let policy = PermissionPolicy::default();

    for (command, expected) in [
        ("Start recording", ObsRecordingAction::Start),
        ("Stop recording", ObsRecordingAction::Stop),
        ("Pause recording", ObsRecordingAction::Pause),
        ("Resume recording", ObsRecordingAction::Resume),
        ("Começa a gravar", ObsRecordingAction::Start),
        ("Para a gravação", ObsRecordingAction::Stop),
    ] {
        assert!(matches!(
            assert_action(
                command,
                &policy,
                PermissionClass::Act,
                PermissionDecision::Allow,
            ),
            ActionIntent::ObsRecording(action) if action == expected
        ));
    }
}

#[test]
fn obs_recording_commands_respect_act_policy_override() {
    let mut policy = PermissionPolicy::default();
    policy
        .set(PermissionClass::Act, PermissionDecision::Ask)
        .unwrap();

    assert!(matches!(
        assert_action(
            "Pause recording",
            &policy,
            PermissionClass::Act,
            PermissionDecision::Ask,
        ),
        ActionIntent::ObsRecording(ObsRecordingAction::Pause)
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
