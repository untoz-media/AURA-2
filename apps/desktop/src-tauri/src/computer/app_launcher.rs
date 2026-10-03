use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTarget {
    ObsStudio,
    Brave,
    Chrome,
    FileExplorer,
    WindowsTerminal,
    Notepad,
    Calculator,
}

impl AppTarget {
    pub fn from_process_image(value: &str) -> Option<Self> {
        let normalized = value.trim().to_lowercase();

        [
            Self::ObsStudio,
            Self::Brave,
            Self::Chrome,
            Self::FileExplorer,
            Self::WindowsTerminal,
            Self::Notepad,
            Self::Calculator,
        ]
        .into_iter()
        .find(|target| {
            target
                .process_images()
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(&normalized))
        })
    }

    pub fn from_alias(value: &str) -> Option<Self> {
        match value.trim() {
            "obs" | "obs studio" | "obsstudio" => Some(Self::ObsStudio),
            "brave" | "brave browser" => Some(Self::Brave),
            "chrome" | "google chrome" => Some(Self::Chrome),
            "explorer" | "file explorer" | "windows explorer" | "explorador"
            | "explorador de ficheiros" | "ficheiros" => Some(Self::FileExplorer),
            "terminal" | "windows terminal" | "wt" => Some(Self::WindowsTerminal),
            "notepad" | "bloco de notas" => Some(Self::Notepad),
            "calculator" | "calc" | "calculadora" => Some(Self::Calculator),
            _ => None,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::ObsStudio => "OBS Studio",
            Self::Brave => "Brave",
            Self::Chrome => "Google Chrome",
            Self::FileExplorer => "File Explorer",
            Self::WindowsTerminal => "Windows Terminal",
            Self::Notepad => "Notepad",
            Self::Calculator => "Calculator",
        }
    }

    pub fn process_images(self) -> &'static [&'static str] {
        match self {
            Self::ObsStudio => &["obs64.exe"],
            Self::Brave => &["brave.exe"],
            Self::Chrome => &["chrome.exe"],
            Self::FileExplorer => &["explorer.exe"],
            Self::WindowsTerminal => &["WindowsTerminal.exe"],
            Self::Notepad => &["notepad.exe"],
            Self::Calculator => &["CalculatorApp.exe", "Calculator.exe"],
        }
    }

    pub fn window_title_hints(self) -> &'static [&'static str] {
        match self {
            Self::ObsStudio => &["obs"],
            Self::Brave => &["brave"],
            Self::Chrome => &["chrome"],
            Self::FileExplorer => &["file explorer", "explorer"],
            Self::WindowsTerminal => &["terminal"],
            Self::Notepad => &["notepad", "bloco de notas"],
            Self::Calculator => &["calculator", "calculadora"],
        }
    }

    pub fn is_close_protected(self) -> bool {
        matches!(self, Self::FileExplorer)
    }
}

#[derive(Debug)]
pub enum LaunchError {
    NotInstalled(&'static str),
    SpawnFailed(String),
}

impl std::fmt::Display for LaunchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInstalled(name) => {
                write!(formatter, "{name} was not found in a supported Windows location.")
            }
            Self::SpawnFailed(message) => write!(formatter, "{message}"),
        }
    }
}

fn env_path(variable: &str, suffix: &[&str]) -> Option<PathBuf> {
    let root = env::var_os(variable)?;
    let mut path = PathBuf::from(root);
    for part in suffix {
        path.push(part);
    }
    Some(path)
}

fn existing_path(candidates: impl IntoIterator<Item = Option<PathBuf>>) -> Option<PathBuf> {
    candidates
        .into_iter()
        .flatten()
        .find(|candidate| candidate.is_file())
}

fn spawn_path(path: &Path, current_dir: Option<&Path>) -> Result<(), LaunchError> {
    let mut command = Command::new(path);

    if let Some(directory) = current_dir {
        command.current_dir(directory);
    }

    command
        .spawn()
        .map(|_| ())
        .map_err(|error| LaunchError::SpawnFailed(error.to_string()))
}

fn spawn_system(program: &str) -> Result<(), LaunchError> {
    Command::new(program)
        .spawn()
        .map(|_| ())
        .map_err(|error| LaunchError::SpawnFailed(error.to_string()))
}

fn launch_obs() -> Result<(), LaunchError> {
    let path = existing_path([
        env_path("PROGRAMFILES", &["obs-studio", "bin", "64bit", "obs64.exe"]),
        env_path("PROGRAMFILES(X86)", &["obs-studio", "bin", "64bit", "obs64.exe"]),
    ])
    .ok_or(LaunchError::NotInstalled("OBS Studio"))?;

    let directory = path.parent();
    spawn_path(&path, directory)
}

fn launch_brave() -> Result<(), LaunchError> {
    let path = existing_path([
        env_path(
            "PROGRAMFILES",
            &["BraveSoftware", "Brave-Browser", "Application", "brave.exe"],
        ),
        env_path(
            "PROGRAMFILES(X86)",
            &["BraveSoftware", "Brave-Browser", "Application", "brave.exe"],
        ),
        env_path(
            "LOCALAPPDATA",
            &["BraveSoftware", "Brave-Browser", "Application", "brave.exe"],
        ),
    ])
    .ok_or(LaunchError::NotInstalled("Brave"))?;

    spawn_path(&path, None)
}

fn launch_chrome() -> Result<(), LaunchError> {
    let path = existing_path([
        env_path(
            "PROGRAMFILES",
            &["Google", "Chrome", "Application", "chrome.exe"],
        ),
        env_path(
            "PROGRAMFILES(X86)",
            &["Google", "Chrome", "Application", "chrome.exe"],
        ),
        env_path(
            "LOCALAPPDATA",
            &["Google", "Chrome", "Application", "chrome.exe"],
        ),
    ])
    .ok_or(LaunchError::NotInstalled("Google Chrome"))?;

    spawn_path(&path, None)
}

pub fn launch_app(target: AppTarget) -> Result<(), LaunchError> {
    match target {
        AppTarget::ObsStudio => launch_obs(),
        AppTarget::Brave => launch_brave(),
        AppTarget::Chrome => launch_chrome(),
        AppTarget::FileExplorer => spawn_system("explorer.exe"),
        AppTarget::WindowsTerminal => spawn_system("wt.exe"),
        AppTarget::Notepad => spawn_system("notepad.exe"),
        AppTarget::Calculator => spawn_system("calc.exe"),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_image_matching_is_case_insensitive() {
        assert_eq!(
            AppTarget::from_process_image("BRAVE.EXE"),
            Some(AppTarget::Brave)
        );
        assert_eq!(
            AppTarget::from_process_image("obs64.exe"),
            Some(AppTarget::ObsStudio)
        );
        assert_eq!(AppTarget::from_process_image("unknown.exe"), None);
    }
}
