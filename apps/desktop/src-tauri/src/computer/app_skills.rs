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
                    match target {
                        AppTarget::Brave => "brave",
                        AppTarget::Chrome => "chrome",
                        _ => unreachable!("browser registry uses browser targets only"),
                    },
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

pub fn execute_browser_skill(skill: BrowserSkill) -> Result<String, String> {
    let window = switch_to_app(skill.target).map_err(|error| error.to_string())?;

    // Give Windows a short moment to complete the explicit foreground transition
    // before injecting the bounded browser shortcut.
    thread::sleep(Duration::from_millis(80));

    let foreground = current_app().map_err(|error| {
        format!(
            "Could not verify {} focus before sending the shortcut: {error}",
            skill.target.display_name()
        )
    })?;

    let target_is_foreground = skill
        .target
        .process_images()
        .iter()
        .any(|image| foreground.process_name.eq_ignore_ascii_case(image));

    if !target_is_foreground {
        return Err(format!(
            "{} did not remain in the foreground. The browser shortcut was not sent.",
            skill.target.display_name()
        ));
    }

    let shortcut = KeyboardShortcut::parse(skill.action.shortcut())
        .map_err(|error| error.to_string())?;
    press_shortcut(&shortcut).map_err(|error| error.to_string())?;

    Ok(format!(
        "{} executed in {} — {}.",
        skill.action.display_name(),
        skill.target.display_name(),
        window.title
    ))
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
        assert_eq!(catalog.skills.len(), 18);
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
