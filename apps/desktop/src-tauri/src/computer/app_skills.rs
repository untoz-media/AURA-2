use std::{process::Command, thread, time::Duration};

use tauri::{AppHandle, Manager};

use super::{
    app_launcher::AppTarget,
    keyboard::{press_shortcut, KeyboardShortcut},
    window_manager::{current_app, switch_to_app},
};

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
