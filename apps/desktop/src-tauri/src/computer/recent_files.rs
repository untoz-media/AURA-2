use serde::Serialize;
use std::{
    collections::HashSet,
    env,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const DEFAULT_RECENT_LIMIT: usize = 12;
const MAX_RECENT_LIMIT: usize = 50;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentFileItem {
    pub name: String,
    pub modified_at_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentFilesSnapshot {
    pub items: Vec<RecentFileItem>,
    pub source: String,
    pub refreshed_at_ms: u64,
}

#[derive(Debug)]
pub enum RecentFilesError {
    AppDataUnavailable,
    RecentFolderUnavailable(String),
    ReadFailed(String),
}

impl std::fmt::Display for RecentFilesError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AppDataUnavailable => write!(
                formatter,
                "Windows APPDATA is unavailable, so AURA cannot locate Recent Items."
            ),
            Self::RecentFolderUnavailable(path) => {
                write!(formatter, "Windows Recent Items is unavailable at {path}.")
            }
            Self::ReadFailed(message) => write!(formatter, "{message}"),
        }
    }
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn system_time_ms(value: SystemTime) -> u64 {
    value
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn recent_items_dir() -> Result<PathBuf, RecentFilesError> {
    let app_data = env::var_os("APPDATA").ok_or(RecentFilesError::AppDataUnavailable)?;
    Ok(PathBuf::from(app_data)
        .join("Microsoft")
        .join("Windows")
        .join("Recent"))
}

fn display_name(path: &Path) -> Option<String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    if !extension.eq_ignore_ascii_case("lnk") && !extension.eq_ignore_ascii_case("url") {
        return None;
    }

    let stem = path.file_stem()?.to_string_lossy().trim().to_string();
    (!stem.is_empty()).then_some(stem)
}

pub fn recent_files_snapshot(limit: usize) -> Result<RecentFilesSnapshot, RecentFilesError> {
    let directory = recent_items_dir()?;
    if !directory.is_dir() {
        return Err(RecentFilesError::RecentFolderUnavailable(
            directory.display().to_string(),
        ));
    }

    let entries = fs::read_dir(&directory).map_err(|error| {
        RecentFilesError::ReadFailed(format!(
            "AURA could not read Windows Recent Items: {error}"
        ))
    })?;

    let mut items = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = display_name(&path) else {
            continue;
        };

        let modified_at_ms = entry
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .map(system_time_ms)
            .unwrap_or_default();

        items.push(RecentFileItem {
            name,
            modified_at_ms,
        });
    }

    items.sort_by(|left, right| {
        right
            .modified_at_ms
            .cmp(&left.modified_at_ms)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    let mut seen = HashSet::new();
    items.retain(|item| seen.insert(item.name.to_lowercase()));

    let limit = limit.clamp(1, MAX_RECENT_LIMIT);
    items.truncate(limit);

    Ok(RecentFilesSnapshot {
        items,
        source: "windowsRecentItems".to_string(),
        refreshed_at_ms: timestamp_ms(),
    })
}

pub fn default_recent_files_snapshot() -> Result<RecentFilesSnapshot, RecentFilesError> {
    recent_files_snapshot(DEFAULT_RECENT_LIMIT)
}

pub fn summarize_recent_files(limit: usize) -> Result<String, RecentFilesError> {
    let snapshot = recent_files_snapshot(limit)?;

    if snapshot.items.is_empty() {
        return Ok("Windows Recent Items does not currently contain any recent files.".to_string());
    }

    let names = snapshot
        .items
        .iter()
        .map(|item| item.name.as_str())
        .collect::<Vec<_>>()
        .join(" · ");

    Ok(format!("Recent files: {names}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_windows_recent_shortcut_extensions() {
        assert_eq!(
            display_name(Path::new(r"C:\Recent\Artemis 2 Documentary.prproj.lnk")).as_deref(),
            Some("Artemis 2 Documentary.prproj")
        );
        assert_eq!(
            display_name(Path::new(r"C:\Recent\Untoz.url")).as_deref(),
            Some("Untoz")
        );
    }

    #[test]
    fn rejects_non_recent_payload_files() {
        assert!(display_name(Path::new(r"C:\Recent\desktop.ini")).is_none());
        assert!(display_name(Path::new(r"C:\Recent\notes.txt")).is_none());
    }
}
