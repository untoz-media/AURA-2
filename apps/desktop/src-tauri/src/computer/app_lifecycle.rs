use std::process::Command;

use super::app_launcher::AppTarget;

#[derive(Debug)]
pub enum CloseError {
    ProtectedTarget(&'static str),
    NotRunning(&'static str),
    CommandFailed(String),
}

impl std::fmt::Display for CloseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProtectedTarget(name) => write!(
                formatter,
                "{name} is protected from process termination by AURA."
            ),
            Self::NotRunning(name) => write!(formatter, "{name} is not currently running."),
            Self::CommandFailed(message) => write!(formatter, "{message}"),
        }
    }
}

fn process_images(target: AppTarget) -> &'static [&'static str] {
    match target {
        AppTarget::ObsStudio => &["obs64.exe"],
        AppTarget::Brave => &["brave.exe"],
        AppTarget::Chrome => &["chrome.exe"],
        AppTarget::WindowsTerminal => &["WindowsTerminal.exe"],
        AppTarget::Notepad => &["notepad.exe"],
        AppTarget::Calculator => &["CalculatorApp.exe", "Calculator.exe"],
        AppTarget::FileExplorer => &[],
    }
}

fn image_is_running(image_name: &str) -> Result<bool, CloseError> {
    let filter = format!("IMAGENAME eq {image_name}");
    let output = Command::new("tasklist.exe")
        .args(["/FI", &filter, "/NH"])
        .output()
        .map_err(|error| CloseError::CommandFailed(error.to_string()))?;

    if !output.status.success() {
        return Err(CloseError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
    Ok(stdout.contains(&image_name.to_lowercase()))
}

fn end_image(image_name: &str) -> Result<(), CloseError> {
    let output = Command::new("taskkill.exe")
        .args(["/IM", image_name])
        .output()
        .map_err(|error| CloseError::CommandFailed(error.to_string()))?;

    if output.status.success() {
        return Ok(());
    }

    let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(CloseError::CommandFailed(if message.is_empty() {
        format!("Windows could not close {image_name}.")
    } else {
        message
    }))
}

pub fn close_app(target: AppTarget) -> Result<(), CloseError> {
    if target == AppTarget::FileExplorer {
        return Err(CloseError::ProtectedTarget("File Explorer"));
    }

    let images = process_images(target);
    let mut found = false;
    let mut last_error: Option<CloseError> = None;

    for image in images {
        match image_is_running(image) {
            Ok(true) => {
                found = true;
                if let Err(error) = end_image(image) {
                    last_error = Some(error);
                }
            }
            Ok(false) => {}
            Err(error) => last_error = Some(error),
        }
    }

    if let Some(error) = last_error {
        return Err(error);
    }

    if !found {
        return Err(CloseError::NotRunning(target.display_name()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explorer_is_explicitly_protected() {
        assert!(matches!(
            close_app(AppTarget::FileExplorer),
            Err(CloseError::ProtectedTarget("File Explorer"))
        ));
    }

    #[test]
    fn known_targets_have_fixed_process_images() {
        assert_eq!(process_images(AppTarget::ObsStudio), &["obs64.exe"]);
        assert_eq!(process_images(AppTarget::Brave), &["brave.exe"]);
    }
}
