use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_DROP_ITEMS: usize = 8;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DroppedFileItem {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub modified_at_ms: u64,
    pub can_use_vision: bool,
    pub can_reveal: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DropIntakeSnapshot {
    pub items: Vec<DroppedFileItem>,
    pub rejected_count: usize,
    pub truncated: bool,
    pub refreshed_at_ms: u64,
}

#[derive(Clone, Debug)]
struct DroppedFileRecord {
    path: PathBuf,
    item: DroppedFileItem,
}

pub struct DropIntakeState {
    records: Mutex<HashMap<String, DroppedFileRecord>>,
    counter: AtomicU64,
}

impl Default for DropIntakeState {
    fn default() -> Self {
        Self {
            records: Mutex::new(HashMap::new()),
            counter: AtomicU64::new(1),
        }
    }
}

impl DropIntakeState {
    pub fn ingest(&self, paths: Vec<String>) -> DropIntakeSnapshot {
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        records.clear();

        let truncated = paths.len() > MAX_DROP_ITEMS;
        let mut rejected_count = paths.len().saturating_sub(MAX_DROP_ITEMS);
        let mut seen = HashSet::new();

        for raw in paths.into_iter().take(MAX_DROP_ITEMS) {
            let path = PathBuf::from(raw);
            let canonical = match fs::canonicalize(&path) {
                Ok(path) => path,
                Err(_) => {
                    rejected_count += 1;
                    continue;
                }
            };

            if !canonical.is_file() {
                rejected_count += 1;
                continue;
            }

            let identity = canonical.to_string_lossy().to_lowercase();
            if !seen.insert(identity) {
                continue;
            }

            let metadata = match fs::metadata(&canonical) {
                Ok(metadata) => metadata,
                Err(_) => {
                    rejected_count += 1;
                    continue;
                }
            };

            let name = canonical
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| "Dropped file".to_string());
            let extension = canonical
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.to_ascii_lowercase());
            let kind = classify(extension.as_deref()).to_string();
            let can_use_vision = kind == "image";
            let id = format!(
                "drop-{}-{}",
                timestamp_ms(),
                self.counter.fetch_add(1, Ordering::Relaxed)
            );

            let item = DroppedFileItem {
                id: id.clone(),
                name,
                kind,
                extension,
                size_bytes: metadata.len(),
                modified_at_ms: metadata
                    .modified()
                    .ok()
                    .map(system_time_ms)
                    .unwrap_or(0),
                can_use_vision,
                can_reveal: true,
            };

            records.insert(
                id,
                DroppedFileRecord {
                    path: canonical,
                    item,
                },
            );
        }

        snapshot_from_records(&records, rejected_count, truncated)
    }

    pub fn snapshot(&self) -> DropIntakeSnapshot {
        let records = self
            .records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        snapshot_from_records(&records, 0, false)
    }

    pub fn clear(&self) -> DropIntakeSnapshot {
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        records.clear();
        snapshot_from_records(&records, 0, false)
    }

    pub fn path_for(&self, id: &str) -> Result<PathBuf, String> {
        let records = self
            .records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let record = records
            .get(id)
            .ok_or_else(|| "That dropped-file session item no longer exists.".to_string())?;

        if !record.path.exists() || !record.path.is_file() {
            return Err("The dropped file no longer exists at its original location.".to_string());
        }

        Ok(record.path.clone())
    }

    pub fn item_for(&self, id: &str) -> Result<DroppedFileItem, String> {
        let records = self
            .records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        records
            .get(id)
            .map(|record| record.item.clone())
            .ok_or_else(|| "That dropped-file session item no longer exists.".to_string())
    }
}

pub fn reveal_drop(state: &DropIntakeState, id: &str) -> Result<String, String> {
    let path = state.path_for(id)?;
    let item = state.item_for(id)?;
    let argument = format!("/select,{}", path.to_string_lossy());

    Command::new("explorer.exe")
        .arg(argument)
        .spawn()
        .map_err(|error| format!("Could not reveal the dropped file in Explorer: {error}"))?;

    Ok(format!("Revealed {} in File Explorer.", item.name))
}

pub fn is_supported_vision_image(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp"
            )
        })
        .unwrap_or(false)
}

fn snapshot_from_records(
    records: &HashMap<String, DroppedFileRecord>,
    rejected_count: usize,
    truncated: bool,
) -> DropIntakeSnapshot {
    let mut items = records
        .values()
        .map(|record| record.item.clone())
        .collect::<Vec<_>>();

    items.sort_by(|left, right| {
        right
            .modified_at_ms
            .cmp(&left.modified_at_ms)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    DropIntakeSnapshot {
        items,
        rejected_count,
        truncated,
        refreshed_at_ms: timestamp_ms(),
    }
}

fn classify(extension: Option<&str>) -> &'static str {
    let normalized = extension.unwrap_or_default().to_ascii_lowercase();
    match normalized.as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" => "image",
        "mp4" | "mov" | "mkv" | "avi" | "webm" | "m4v" | "wmv" | "mts" | "m2ts" => "video",
        "mp3" | "wav" | "flac" | "m4a" | "aac" | "ogg" | "opus" => "audio",
        "pdf" | "txt" | "md" | "rtf" | "doc" | "docx" | "ppt" | "pptx" | "xls"
        | "xlsx" | "csv" => "document",
        "zip" | "7z" | "rar" | "tar" | "gz" => "archive",
        _ => "other",
    }
}

fn system_time_ms(value: SystemTime) -> u64 {
    value
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn timestamp_ms() -> u64 {
    system_time_ms(SystemTime::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_common_drop_types() {
        assert_eq!(classify(Some("png")), "image");
        assert_eq!(classify(Some("MP4")), "video");
        assert_eq!(classify(Some("pdf")), "document");
        assert_eq!(classify(Some("zip")), "archive");
        assert_eq!(classify(None), "other");
    }

    #[test]
    fn vision_support_is_extension_bounded() {
        assert!(is_supported_vision_image(Path::new("test.JPG")));
        assert!(is_supported_vision_image(Path::new("test.webp")));
        assert!(!is_supported_vision_image(Path::new("test.svg")));
        assert!(!is_supported_vision_image(Path::new("test.exe")));
    }

    #[test]
    fn drop_limit_is_intentionally_small() {
        assert_eq!(MAX_DROP_ITEMS, 8);
    }
}
