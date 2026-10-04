use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_DROP_ITEMS: usize = 8;
const MAX_TEXT_PREVIEW_BYTES: u64 = 64 * 1024;
const MAX_TEXT_PREVIEW_CHARS: usize = 12_000;

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
    pub can_inspect: bool,
    pub can_preview_text: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DropIntakeSnapshot {
    pub items: Vec<DroppedFileItem>,
    pub rejected_count: usize,
    pub truncated: bool,
    pub refreshed_at_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DroppedFileInspection {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub modified_at_ms: u64,
    pub content_mode: String,
    pub summary: String,
    pub text_preview: Option<String>,
    pub preview_truncated: bool,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub note: Option<String>,
    pub inspected_at_ms: u64,
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
            let can_preview_text = is_text_preview_extension(extension.as_deref());
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
                can_inspect: true,
                can_preview_text,
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

pub fn inspect_drop(
    state: &DropIntakeState,
    id: &str,
) -> Result<DroppedFileInspection, String> {
    let path = state.path_for(id)?;
    let item = state.item_for(id)?;

    let mut text_preview = None;
    let mut preview_truncated = false;
    let mut image_width = None;
    let mut image_height = None;
    let mut note = None;
    let content_mode;

    if item.can_preview_text {
        let (preview, truncated) = read_text_preview(&path)?;
        text_preview = preview;
        preview_truncated = truncated;
        content_mode = if text_preview.is_some() {
            "textPreview"
        } else {
            "metadataOnly"
        }
        .to_string();

        if text_preview.is_none() {
            note = Some(
                "This text-like file was not valid UTF-8, so AURA kept inspection metadata-only."
                    .to_string(),
            );
        }
    } else if item.kind == "image" {
        match image::image_dimensions(&path) {
            Ok((width, height)) => {
                image_width = Some(width);
                image_height = Some(height);
                content_mode = "imageMetadata".to_string();
            }
            Err(_) => {
                content_mode = "metadataOnly".to_string();
                note = Some(
                    "AURA could not read the image dimensions without decoding more content."
                        .to_string(),
                );
            }
        }
    } else {
        content_mode = "metadataOnly".to_string();
        note = inspection_note(&item);
    }

    let summary = inspection_summary(
        &item,
        text_preview.is_some(),
        image_width,
        image_height,
    );

    Ok(DroppedFileInspection {
        id: item.id,
        name: item.name,
        kind: item.kind,
        extension: item.extension,
        size_bytes: item.size_bytes,
        modified_at_ms: item.modified_at_ms,
        content_mode,
        summary,
        text_preview,
        preview_truncated,
        image_width,
        image_height,
        note,
        inspected_at_ms: timestamp_ms(),
    })
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

fn read_text_preview(path: &Path) -> Result<(Option<String>, bool), String> {
    let file = File::open(path)
        .map_err(|error| format!("Could not open the dropped file for inspection: {error}"))?;
    let mut bytes = Vec::new();
    file.take(MAX_TEXT_PREVIEW_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Could not read the dropped file preview: {error}"))?;

    let mut truncated = bytes.len() as u64 > MAX_TEXT_PREVIEW_BYTES;
    if truncated {
        bytes.truncate(MAX_TEXT_PREVIEW_BYTES as usize);
    }

    if bytes.contains(&0) {
        return Ok((None, truncated));
    }

    let value = match String::from_utf8(bytes) {
        Ok(value) => value,
        Err(_) => return Ok((None, truncated)),
    };

    if value.chars().count() <= MAX_TEXT_PREVIEW_CHARS {
        return Ok((Some(value), truncated));
    }

    let preview = value.chars().take(MAX_TEXT_PREVIEW_CHARS).collect::<String>();
    truncated = true;
    Ok((Some(preview), truncated))
}

fn inspection_note(item: &DroppedFileItem) -> Option<String> {
    match item.kind.as_str() {
        "video" => Some(
            "Video inspection is metadata-only in this Beta step; duration, codecs and frame analysis are not read yet."
                .to_string(),
        ),
        "audio" => Some(
            "Audio inspection is metadata-only in this Beta step; duration, codecs and transcription are not read yet."
                .to_string(),
        ),
        "document" if item.extension.as_deref() == Some("pdf") => Some(
            "PDF inspection is metadata-only in this Beta step; page text is not extracted automatically."
                .to_string(),
        ),
        "document" => Some(
            "This document format is metadata-only unless it is on AURA's bounded plain-text preview allowlist."
                .to_string(),
        ),
        "archive" => Some(
            "Archives are never extracted by Drag & Drop inspection."
                .to_string(),
        ),
        _ => Some(
            "AURA kept this item metadata-only because its content type is not on the preview allowlist."
                .to_string(),
        ),
    }
}

fn inspection_summary(
    item: &DroppedFileItem,
    has_text_preview: bool,
    image_width: Option<u32>,
    image_height: Option<u32>,
) -> String {
    if let (Some(width), Some(height)) = (image_width, image_height) {
        return format!("Image metadata ready: {width}×{height}.");
    }

    if has_text_preview {
        return "Bounded local text preview ready for explicit use as context.".to_string();
    }

    format!(
        "{} metadata ready; file contents were not loaded.",
        title_case_kind(&item.kind)
    )
}

fn title_case_kind(kind: &str) -> String {
    let mut chars = kind.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => "File".to_string(),
    }
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
        | "xlsx" | "csv" | "json" | "log" | "yaml" | "yml" | "toml" | "xml" => "document",
        "zip" | "7z" | "rar" | "tar" | "gz" => "archive",
        _ => "other",
    }
}

fn is_text_preview_extension(extension: Option<&str>) -> bool {
    matches!(
        extension.unwrap_or_default().to_ascii_lowercase().as_str(),
        "txt"
            | "md"
            | "csv"
            | "json"
            | "log"
            | "yaml"
            | "yml"
            | "toml"
            | "xml"
            | "html"
            | "htm"
            | "css"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "py"
            | "rs"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "java"
            | "kt"
            | "go"
            | "sql"
            | "ini"
            | "conf"
    )
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
    use std::io::Write;

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
    fn text_preview_allowlist_is_explicit() {
        assert!(is_text_preview_extension(Some("txt")));
        assert!(is_text_preview_extension(Some("TSX")));
        assert!(!is_text_preview_extension(Some("pdf")));
        assert!(!is_text_preview_extension(Some("exe")));
    }

    #[test]
    fn text_preview_is_bounded() {
        let path = std::env::temp_dir().join(format!("aura-drop-preview-{}.txt", timestamp_ms()));
        let mut file = File::create(&path).expect("create preview fixture");
        let data = "a".repeat((MAX_TEXT_PREVIEW_BYTES + 512) as usize);
        file.write_all(data.as_bytes()).expect("write preview fixture");
        drop(file);

        let (preview, truncated) = read_text_preview(&path).expect("read preview");
        let preview = preview.expect("utf8 preview");
        assert!(truncated);
        assert!(preview.chars().count() <= MAX_TEXT_PREVIEW_CHARS);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn drop_limit_is_intentionally_small() {
        assert_eq!(MAX_DROP_ITEMS, 8);
    }
}
