use std::{
    process::Command,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::permissions::PermissionClass;

use super::{
    app_launcher::AppTarget,
    keyboard::{press_shortcut, KeyboardShortcut},
    window_manager::{current_app, switch_to_app},
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSkillDescriptor {
    pub id: String,
    pub group: String,
    pub app_name: String,
    pub name: String,
    pub description: String,
    pub command: String,
    pub permission: PermissionClass,
    pub available: bool,
    pub contextual: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSkillCatalog {
    pub skills: Vec<AppSkillDescriptor>,
    pub context_app_name: Option<String>,
    pub refreshed_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalFolderSkill {
    Desktop,
    Documents,
    Downloads,
    Pictures,
    Videos,
    Music,
}

impl PersonalFolderSkill {
    pub fn id(self) -> &'static str {
        match self {
            Self::Desktop => "explorer.desktop",
            Self::Documents => "explorer.documents",
            Self::Downloads => "explorer.downloads",
            Self::Pictures => "explorer.pictures",
            Self::Videos => "explorer.videos",
            Self::Music => "explorer.music",
        }
    }

    pub fn command(self) -> String {
        format!("Open {}", self.display_name())
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Desktop => "Desktop",
            Self::Documents => "Documents",
            Self::Downloads => "Downloads",
            Self::Pictures => "Pictures",
            Self::Videos => "Videos",
            Self::Music => "Music",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalSkillAction {
    NewTab,
    NextTab,
    PreviousTab,
    CommandPalette,
    Find,
    OpenTabDropdown,
}

impl TerminalSkillAction {
    pub fn id(self) -> &'static str {
        match self {
            Self::NewTab => "new-tab",
            Self::NextTab => "next-tab",
            Self::PreviousTab => "previous-tab",
            Self::CommandPalette => "command-palette",
            Self::Find => "find",
            Self::OpenTabDropdown => "tab-dropdown",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::NewTab => "New tab",
            Self::NextTab => "Next tab",
            Self::PreviousTab => "Previous tab",
            Self::CommandPalette => "Command palette",
            Self::Find => "Find",
            Self::OpenTabDropdown => "Tab dropdown",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::NewTab => "Open a new Windows Terminal tab using its configured default profile.",
            Self::NextTab => "Move to the next Windows Terminal tab.",
            Self::PreviousTab => "Move to the previous Windows Terminal tab.",
            Self::CommandPalette => "Open Windows Terminal's built-in command palette.",
            Self::Find => "Open Windows Terminal's scrollback search.",
            Self::OpenTabDropdown => "Open Windows Terminal's new-tab/profile dropdown.",
        }
    }

    pub fn command(self) -> &'static str {
        match self {
            Self::NewTab => "New tab in Windows Terminal",
            Self::NextTab => "Next tab in Windows Terminal",
            Self::PreviousTab => "Previous tab in Windows Terminal",
            Self::CommandPalette => "Command palette in Windows Terminal",
            Self::Find => "Find in Windows Terminal",
            Self::OpenTabDropdown => "Open tab dropdown in Windows Terminal",
        }
    }

    fn shortcut(self) -> &'static str {
        match self {
            Self::NewTab => "Ctrl+Shift+T",
            Self::NextTab => "Ctrl+Tab",
            Self::PreviousTab => "Ctrl+Shift+Tab",
            Self::CommandPalette => "Ctrl+Shift+P",
            Self::Find => "Ctrl+Shift+F",
            Self::OpenTabDropdown => "Ctrl+Shift+Space",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSkill {
    pub action: TerminalSkillAction,
}

impl TerminalSkill {
    pub fn summary(self) -> String {
        format!("{} in Windows Terminal", self.action.display_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalculatorSkillAction {
    StandardMode,
    ScientificMode,
    ProgrammerMode,
    DateCalculationMode,
    GraphingMode,
}

impl CalculatorSkillAction {
    pub fn id(self) -> &'static str {
        match self {
            Self::StandardMode => "standard-mode",
            Self::ScientificMode => "scientific-mode",
            Self::ProgrammerMode => "programmer-mode",
            Self::DateCalculationMode => "date-calculation-mode",
            Self::GraphingMode => "graphing-mode",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::StandardMode => "Standard mode",
            Self::ScientificMode => "Scientific mode",
            Self::ProgrammerMode => "Programmer mode",
            Self::DateCalculationMode => "Date calculation",
            Self::GraphingMode => "Graphing mode",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::StandardMode => "Switch Windows Calculator to Standard mode.",
            Self::ScientificMode => "Switch Windows Calculator to Scientific mode.",
            Self::ProgrammerMode => "Switch Windows Calculator to Programmer mode.",
            Self::DateCalculationMode => "Switch Windows Calculator to Date Calculation mode.",
            Self::GraphingMode => "Switch Windows Calculator to Graphing mode.",
        }
    }

    pub fn command(self) -> &'static str {
        match self {
            Self::StandardMode => "Standard mode in Calculator",
            Self::ScientificMode => "Scientific mode in Calculator",
            Self::ProgrammerMode => "Programmer mode in Calculator",
            Self::DateCalculationMode => "Date calculation in Calculator",
            Self::GraphingMode => "Graphing mode in Calculator",
        }
    }

    fn shortcut(self) -> &'static str {
        match self {
            Self::StandardMode => "Alt+1",
            Self::ScientificMode => "Alt+2",
            Self::ProgrammerMode => "Alt+3",
            Self::DateCalculationMode => "Alt+4",
            Self::GraphingMode => "Alt+5",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalculatorSkill {
    pub action: CalculatorSkillAction,
}

impl CalculatorSkill {
    pub fn summary(self) -> String {
        format!("{} in Calculator", self.action.display_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotepadSkillAction {
    NewNote,
    Find,
    SelectAll,
    Undo,
    Redo,
}

impl NotepadSkillAction {
    pub fn id(self) -> &'static str {
        match self {
            Self::NewNote => "new-note",
            Self::Find => "find",
            Self::SelectAll => "select-all",
            Self::Undo => "undo",
            Self::Redo => "redo",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::NewNote => "New note",
            Self::Find => "Find",
            Self::SelectAll => "Select all",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::NewNote => "Create a new Notepad document.",
            Self::Find => "Open Notepad's Find interface.",
            Self::SelectAll => "Select all text in the active Notepad document.",
            Self::Undo => "Undo the most recent Notepad edit.",
            Self::Redo => "Redo the most recently undone Notepad edit.",
        }
    }

    pub fn command(self) -> &'static str {
        match self {
            Self::NewNote => "New note in Notepad",
            Self::Find => "Find in Notepad",
            Self::SelectAll => "Select all in Notepad",
            Self::Undo => "Undo in Notepad",
            Self::Redo => "Redo in Notepad",
        }
    }

    pub fn permission(self) -> PermissionClass {
        match self {
            Self::Undo | Self::Redo => PermissionClass::Modify,
            Self::NewNote | Self::Find | Self::SelectAll => PermissionClass::Act,
        }
    }

    fn shortcut(self) -> &'static str {
        match self {
            Self::NewNote => "Ctrl+N",
            Self::Find => "Ctrl+F",
            Self::SelectAll => "Ctrl+A",
            Self::Undo => "Ctrl+Z",
            Self::Redo => "Ctrl+Y",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotepadSkill {
    pub action: NotepadSkillAction,
}

impl NotepadSkill {
    pub fn summary(self) -> String {
        format!("{} in Notepad", self.action.display_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserSkillAction {
    NewTab,
    NextTab,
    PreviousTab,
    Reload,
    FocusAddressBar,
    ReopenClosedTab,
}

impl BrowserSkillAction {
    pub fn id(self) -> &'static str {
        match self {
            Self::NewTab => "new-tab",
            Self::NextTab => "next-tab",
            Self::PreviousTab => "previous-tab",
            Self::Reload => "reload",
            Self::FocusAddressBar => "focus-address-bar",
            Self::ReopenClosedTab => "reopen-closed-tab",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::NewTab => "Open a new browser tab.",
            Self::NextTab => "Move to the next browser tab.",
            Self::PreviousTab => "Move to the previous browser tab.",
            Self::Reload => "Reload the current browser tab.",
            Self::FocusAddressBar => "Focus the browser address bar.",
            Self::ReopenClosedTab => "Reopen the most recently closed browser tab.",
        }
    }

    pub fn command(self, target: AppTarget) -> String {
        match self {
            Self::NewTab => format!("New tab in {}", target.display_name()),
            Self::NextTab => format!("Next tab in {}", target.display_name()),
            Self::PreviousTab => format!("Previous tab in {}", target.display_name()),
            Self::Reload => format!("Reload {}", target.display_name()),
            Self::FocusAddressBar => {
                format!("Focus address bar in {}", target.display_name())
            }
            Self::ReopenClosedTab => {
                format!("Reopen closed tab in {}", target.display_name())
            }
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::NewTab => "New tab",
            Self::NextTab => "Next tab",
            Self::PreviousTab => "Previous tab",
            Self::Reload => "Reload tab",
            Self::FocusAddressBar => "Focus address bar",
            Self::ReopenClosedTab => "Reopen closed tab",
        }
    }

    fn shortcut(self) -> &'static str {
        match self {
            Self::NewTab => "Ctrl+T",
            Self::NextTab => "Ctrl+Tab",
            Self::PreviousTab => "Ctrl+Shift+Tab",
            Self::Reload => "Ctrl+R",
            Self::FocusAddressBar => "Ctrl+L",
            Self::ReopenClosedTab => "Ctrl+Shift+T",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowserSkill {
    pub target: AppTarget,
    pub action: BrowserSkillAction,
}

impl BrowserSkill {
    pub fn new(target: AppTarget, action: BrowserSkillAction) -> Result<Self, String> {
        if !matches!(target, AppTarget::Brave | AppTarget::Chrome) {
            return Err(format!(
                "{} does not expose Browser Skills V1.",
                target.display_name()
            ));
        }

        Ok(Self { target, action })
    }

    pub fn summary(self) -> String {
        format!(
            "{} in {}",
            self.action.display_name(),
            self.target.display_name()
        )
    }
}

pub fn app_skill_catalog(context: Option<AppTarget>) -> AppSkillCatalog {
    let mut skills = Vec::new();

    for folder in [
        PersonalFolderSkill::Desktop,
        PersonalFolderSkill::Documents,
        PersonalFolderSkill::Downloads,
        PersonalFolderSkill::Pictures,
        PersonalFolderSkill::Videos,
        PersonalFolderSkill::Music,
    ] {
        skills.push(AppSkillDescriptor {
            id: folder.id().to_string(),
            group: "File Explorer".to_string(),
            app_name: "File Explorer".to_string(),
            name: folder.display_name().to_string(),
            description: format!(
                "Open the resolved Windows {} folder in File Explorer.",
                folder.display_name()
            ),
            command: folder.command(),
            permission: PermissionClass::Act,
            available: true,
            contextual: false,
        });
    }

    for target in [AppTarget::Brave, AppTarget::Chrome] {
        let available = context == Some(target);

        let target_id = match target {
            AppTarget::Brave => "brave",
            AppTarget::Chrome => "chrome",
            _ => continue,
        };

        for action in [
            BrowserSkillAction::NewTab,
            BrowserSkillAction::NextTab,
            BrowserSkillAction::PreviousTab,
            BrowserSkillAction::Reload,
            BrowserSkillAction::FocusAddressBar,
            BrowserSkillAction::ReopenClosedTab,
        ] {
            skills.push(AppSkillDescriptor {
                id: format!(
                    "browser.{}.{}",
                    target_id,
                    action.id()
                ),
                group: "Browser".to_string(),
                app_name: target.display_name().to_string(),
                name: action.display_name().to_string(),
                description: action.description().to_string(),
                command: action.command(target),
                permission: PermissionClass::Act,
                available,
                contextual: true,
            });
        }
    }

    for action in [
        NotepadSkillAction::NewNote,
        NotepadSkillAction::Find,
        NotepadSkillAction::SelectAll,
        NotepadSkillAction::Undo,
        NotepadSkillAction::Redo,
    ] {
        skills.push(AppSkillDescriptor {
            id: format!("notepad.{}", action.id()),
            group: "Notepad".to_string(),
            app_name: "Notepad".to_string(),
            name: action.display_name().to_string(),
            description: action.description().to_string(),
            command: action.command().to_string(),
            permission: action.permission(),
            available: context == Some(AppTarget::Notepad),
            contextual: true,
        });
    }

    for action in [
        TerminalSkillAction::NewTab,
        TerminalSkillAction::NextTab,
        TerminalSkillAction::PreviousTab,
        TerminalSkillAction::CommandPalette,
        TerminalSkillAction::Find,
        TerminalSkillAction::OpenTabDropdown,
    ] {
        skills.push(AppSkillDescriptor {
            id: format!("terminal.{}", action.id()),
            group: "Windows Terminal".to_string(),
            app_name: "Windows Terminal".to_string(),
            name: action.display_name().to_string(),
            description: action.description().to_string(),
            command: action.command().to_string(),
            permission: PermissionClass::Act,
            available: context == Some(AppTarget::WindowsTerminal),
            contextual: true,
        });
    }

    for action in [
        CalculatorSkillAction::StandardMode,
        CalculatorSkillAction::ScientificMode,
        CalculatorSkillAction::ProgrammerMode,
        CalculatorSkillAction::DateCalculationMode,
        CalculatorSkillAction::GraphingMode,
    ] {
        skills.push(AppSkillDescriptor {
            id: format!("calculator.{}", action.id()),
            group: "Calculator".to_string(),
            app_name: "Calculator".to_string(),
            name: action.display_name().to_string(),
            description: action.description().to_string(),
            command: action.command().to_string(),
            permission: PermissionClass::Act,
            available: context == Some(AppTarget::Calculator),
            contextual: true,
        });
    }

    AppSkillCatalog {
        skills,
        context_app_name: context.map(AppTarget::display_name).map(str::to_string),
        refreshed_at_ms: timestamp_ms(),
    }
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn execute_focus_verified_shortcut(
    target: AppTarget,
    label: &str,
    shortcut_value: &str,
) -> Result<String, String> {
    let window = switch_to_app(target).map_err(|error| error.to_string())?;

    // Give Windows a short moment to complete the explicit foreground transition
    // before injecting the bounded application shortcut.
    thread::sleep(Duration::from_millis(80));

    let foreground = current_app().map_err(|error| {
        format!(
            "Could not verify {} focus before sending the shortcut: {error}",
            target.display_name()
        )
    })?;

    let target_is_foreground = target
        .process_images()
        .iter()
        .any(|image| foreground.process_name.eq_ignore_ascii_case(image));

    if !target_is_foreground {
        return Err(format!(
            "{} did not remain in the foreground. The App Skill shortcut was not sent.",
            target.display_name()
        ));
    }

    let shortcut = KeyboardShortcut::parse(shortcut_value)
        .map_err(|error| error.to_string())?;
    press_shortcut(&shortcut).map_err(|error| error.to_string())?;

    Ok(format!(
        "{} executed in {} — {}.",
        label,
        target.display_name(),
        window.title
    ))
}

pub fn execute_browser_skill(skill: BrowserSkill) -> Result<String, String> {
    execute_focus_verified_shortcut(
        skill.target,
        skill.action.display_name(),
        skill.action.shortcut(),
    )
}

pub fn execute_notepad_skill(skill: NotepadSkill) -> Result<String, String> {
    execute_focus_verified_shortcut(
        AppTarget::Notepad,
        skill.action.display_name(),
        skill.action.shortcut(),
    )
}

pub fn execute_terminal_skill(skill: TerminalSkill) -> Result<String, String> {
    execute_focus_verified_shortcut(
        AppTarget::WindowsTerminal,
        skill.action.display_name(),
        skill.action.shortcut(),
    )
}

pub fn execute_calculator_skill(skill: CalculatorSkill) -> Result<String, String> {
    execute_focus_verified_shortcut(
        AppTarget::Calculator,
        skill.action.display_name(),
        skill.action.shortcut(),
    )
}

pub fn execute_personal_folder_skill(
    app: &AppHandle,
    skill: PersonalFolderSkill,
) -> Result<String, String> {
    let resolver = app.path();
    let path = match skill {
        PersonalFolderSkill::Desktop => resolver.desktop_dir(),
        PersonalFolderSkill::Documents => resolver.document_dir(),
        PersonalFolderSkill::Downloads => resolver.download_dir(),
        PersonalFolderSkill::Pictures => resolver.picture_dir(),
        PersonalFolderSkill::Videos => resolver.video_dir(),
        PersonalFolderSkill::Music => resolver.audio_dir(),
    }
    .map_err(|error| {
        format!(
            "Windows did not expose the {} folder: {error}",
            skill.display_name()
        )
    })?;

    if !path.exists() || !path.is_dir() {
        return Err(format!(
            "The resolved {} folder does not exist or is unavailable.",
            skill.display_name()
        ));
    }

    Command::new("explorer.exe")
        .arg(&path)
        .spawn()
        .map_err(|error| {
            format!(
                "Could not open {} in File Explorer: {error}",
                skill.display_name()
            )
        })?;

    Ok(format!("Opened {} in File Explorer.", skill.display_name()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_catalog_has_stable_browser_and_explorer_entries() {
        let catalog = app_skill_catalog(Some(AppTarget::Brave));
        assert_eq!(catalog.skills.len(), 34);
        assert_eq!(catalog.context_app_name.as_deref(), Some("Brave"));

        let brave = catalog
            .skills
            .iter()
            .filter(|skill| skill.app_name == "Brave")
            .collect::<Vec<_>>();
        assert_eq!(brave.len(), 6);
        assert!(brave.iter().all(|skill| skill.available));

        let chrome = catalog
            .skills
            .iter()
            .filter(|skill| skill.app_name == "Google Chrome")
            .collect::<Vec<_>>();
        assert_eq!(chrome.len(), 6);
        assert!(chrome.iter().all(|skill| !skill.available));

        let explorer = catalog
            .skills
            .iter()
            .filter(|skill| skill.app_name == "File Explorer")
            .collect::<Vec<_>>();
        assert_eq!(explorer.len(), 6);
        assert!(explorer.iter().all(|skill| skill.available && !skill.contextual));

        let notepad = catalog
            .skills
            .iter()
            .filter(|skill| skill.app_name == "Notepad")
            .collect::<Vec<_>>();
        assert_eq!(notepad.len(), 5);
        assert!(notepad.iter().all(|skill| !skill.available));
        assert!(notepad
            .iter()
            .any(|skill| skill.name == "Undo" && skill.permission == PermissionClass::Modify));

        let terminal = catalog
            .skills
            .iter()
            .filter(|skill| skill.app_name == "Windows Terminal")
            .collect::<Vec<_>>();
        assert_eq!(terminal.len(), 6);
        assert!(terminal.iter().all(|skill| !skill.available));

        let calculator = catalog
            .skills
            .iter()
            .filter(|skill| skill.app_name == "Calculator")
            .collect::<Vec<_>>();
        assert_eq!(calculator.len(), 5);
        assert!(calculator.iter().all(|skill| !skill.available));
    }

    #[test]
    fn skill_registry_commands_match_router_language() {
        let catalog = app_skill_catalog(Some(AppTarget::Chrome));
        assert!(catalog
            .skills
            .iter()
            .any(|skill| skill.command == "New tab in Google Chrome"));
        assert!(catalog
            .skills
            .iter()
            .any(|skill| skill.command == "Open Downloads"));
    }

    #[test]
    fn personal_folder_skill_labels_are_stable() {
        assert_eq!(PersonalFolderSkill::Desktop.display_name(), "Desktop");
        assert_eq!(PersonalFolderSkill::Downloads.display_name(), "Downloads");
        assert_eq!(PersonalFolderSkill::Music.display_name(), "Music");
    }

    #[test]
    fn terminal_skill_shortcuts_match_windows_terminal_defaults() {
        assert_eq!(TerminalSkillAction::NewTab.shortcut(), "Ctrl+Shift+T");
        assert_eq!(TerminalSkillAction::CommandPalette.shortcut(), "Ctrl+Shift+P");
        assert_eq!(TerminalSkillAction::Find.shortcut(), "Ctrl+Shift+F");
        assert_eq!(TerminalSkillAction::OpenTabDropdown.shortcut(), "Ctrl+Shift+Space");
    }

    #[test]
    fn calculator_mode_shortcuts_match_official_hotkeys() {
        assert_eq!(CalculatorSkillAction::StandardMode.shortcut(), "Alt+1");
        assert_eq!(CalculatorSkillAction::ScientificMode.shortcut(), "Alt+2");
        assert_eq!(CalculatorSkillAction::ProgrammerMode.shortcut(), "Alt+3");
        assert_eq!(CalculatorSkillAction::DateCalculationMode.shortcut(), "Alt+4");
        assert_eq!(CalculatorSkillAction::GraphingMode.shortcut(), "Alt+5");
    }

    #[test]
    fn notepad_skill_permissions_keep_edits_guarded() {
        assert_eq!(NotepadSkillAction::NewNote.permission(), PermissionClass::Act);
        assert_eq!(NotepadSkillAction::Find.permission(), PermissionClass::Act);
        assert_eq!(NotepadSkillAction::Undo.permission(), PermissionClass::Modify);
        assert_eq!(NotepadSkillAction::Redo.permission(), PermissionClass::Modify);
    }

    #[test]
    fn notepad_shortcuts_are_stable() {
        assert_eq!(NotepadSkillAction::NewNote.shortcut(), "Ctrl+N");
        assert_eq!(NotepadSkillAction::Find.shortcut(), "Ctrl+F");
        assert_eq!(NotepadSkillAction::Undo.shortcut(), "Ctrl+Z");
    }

    #[test]
    fn browser_skill_rejects_non_browser_targets() {
        assert!(BrowserSkill::new(
            AppTarget::Notepad,
            BrowserSkillAction::NewTab
        )
        .is_err());
    }

    #[test]
    fn browser_skill_accepts_brave_and_chrome() {
        assert!(BrowserSkill::new(
            AppTarget::Brave,
            BrowserSkillAction::Reload
        )
        .is_ok());
        assert!(BrowserSkill::new(
            AppTarget::Chrome,
            BrowserSkillAction::FocusAddressBar
        )
        .is_ok());
    }

    #[test]
    fn browser_targets_have_process_images_for_focus_verification() {
        assert!(AppTarget::Brave
            .process_images()
            .iter()
            .any(|image| image.eq_ignore_ascii_case("brave.exe")));
        assert!(AppTarget::Chrome
            .process_images()
            .iter()
            .any(|image| image.eq_ignore_ascii_case("chrome.exe")));
    }

    #[test]
    fn skill_shortcuts_are_stable() {
        assert_eq!(BrowserSkillAction::NewTab.shortcut(), "Ctrl+T");
        assert_eq!(BrowserSkillAction::PreviousTab.shortcut(), "Ctrl+Shift+Tab");
        assert_eq!(BrowserSkillAction::FocusAddressBar.shortcut(), "Ctrl+L");
    }
}
