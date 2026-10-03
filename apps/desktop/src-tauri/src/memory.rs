use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const MEMORY_FILENAME: &str = "memory.json";
const MAX_MEMORIES: usize = 1_000;
const MAX_MEMORY_CHARS: usize = 2_000;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRecord {
    pub id: String,
    pub content: String,
    pub source: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySnapshot {
    pub records: Vec<MemoryRecord>,
    pub refreshed_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMemoryRequest {
    pub content: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryCreateResult {
    pub record: MemoryRecord,
    pub created: bool,
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn memory_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app.path().app_config_dir().map_err(|error| error.to_string())?;
    Ok(directory.join(MEMORY_FILENAME))
}

fn normalize_content(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn validate_content(value: &str) -> Result<String, String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("Memory content cannot be empty.".to_string());
    }

    if trimmed.chars().count() > MAX_MEMORY_CHARS {
        return Err(format!(
            "A memory can contain at most {MAX_MEMORY_CHARS} characters."
        ));
    }

    Ok(trimmed.to_string())
}

fn write_records(app: &AppHandle, records: &[MemoryRecord]) -> Result<(), String> {
    let path = memory_path(app)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let content = serde_json::to_string_pretty(records).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())
}

fn read_records(app: &AppHandle) -> Result<Vec<MemoryRecord>, String> {
    let path = memory_path(app)?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read local memory file: {error}"))?;

    let mut records = serde_json::from_str::<Vec<MemoryRecord>>(&content)
        .map_err(|error| {
            format!(
                "Local memory file is invalid and was left unchanged: {error}"
            )
        })?;

    records.sort_by(|left, right| right.updated_at_ms.cmp(&left.updated_at_ms));
    Ok(records)
}

pub fn memory_snapshot(app: &AppHandle) -> Result<MemorySnapshot, String> {
    Ok(MemorySnapshot {
        records: read_records(app)?,
        refreshed_at_ms: timestamp_ms(),
    })
}

pub fn create_memory(
    app: &AppHandle,
    request: CreateMemoryRequest,
    source: &str,
) -> Result<MemoryCreateResult, String> {
    let content = validate_content(&request.content)?;
    let normalized = normalize_content(&content);
    let mut records = read_records(app)?;

    if let Some(existing) = records
        .iter_mut()
        .find(|record| normalize_content(&record.content) == normalized)
    {
        existing.updated_at_ms = timestamp_ms();
        let record = existing.clone();
        write_records(app, &records)?;

        return Ok(MemoryCreateResult {
            record,
            created: false,
        });
    }

    if records.len() >= MAX_MEMORIES {
        return Err(format!(
            "Local memory is full. Delete an existing memory before adding another (limit: {MAX_MEMORIES})."
        ));
    }

    let now = timestamp_ms();
    let id = unique_memory_id(&records, now);
    let record = MemoryRecord {
        id,
        content,
        source: source.trim().to_string(),
        created_at_ms: now,
        updated_at_ms: now,
    };

    records.push(record.clone());
    records.sort_by(|left, right| right.updated_at_ms.cmp(&left.updated_at_ms));
    write_records(app, &records)?;

    Ok(MemoryCreateResult {
        record,
        created: true,
    })
}

pub fn delete_memory(app: &AppHandle, memory_id: &str) -> Result<MemoryRecord, String> {
    let mut records = read_records(app)?;
    let index = records
        .iter()
        .position(|record| record.id == memory_id)
        .ok_or_else(|| "Memory no longer exists.".to_string())?;

    let removed = records.remove(index);
    write_records(app, &records)?;
    Ok(removed)
}

pub fn delete_memory_by_content(
    app: &AppHandle,
    content: &str,
) -> Result<MemoryRecord, String> {
    let requested = validate_content(content)?;
    let normalized = normalize_content(&requested);
    let records = read_records(app)?;

    let matches = records
        .iter()
        .filter(|record| normalize_content(&record.content) == normalized)
        .collect::<Vec<_>>();

    if matches.is_empty() {
        return Err(format!("No saved memory exactly matches “{requested}”."));
    }

    if matches.len() > 1 {
        return Err(
            "More than one saved memory matches that text. Delete it from the Memory screen instead."
                .to_string(),
        );
    }

    delete_memory(app, &matches[0].id)
}

pub fn summarize_memories(app: &AppHandle, limit: usize) -> Result<String, String> {
    let records = read_records(app)?;

    if records.is_empty() {
        return Ok("I do not have any explicit local memories yet.".to_string());
    }

    let visible = records.iter().take(limit).collect::<Vec<_>>();
    let mut summary = visible
        .iter()
        .enumerate()
        .map(|(index, record)| format!("{}. {}", index + 1, record.content))
        .collect::<Vec<_>>()
        .join(" ");

    if records.len() > visible.len() {
        summary.push_str(&format!(
            " And {} more saved memories.",
            records.len() - visible.len()
        ));
    }

    Ok(summary)
}

fn unique_memory_id(records: &[MemoryRecord], now: u64) -> String {
    let base = format!("memory-{now}");
    let mut candidate = base.clone();
    let mut suffix = 2_u32;

    while records.iter().any(|record| record.id == candidate) {
        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }

    candidate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_memory_content() {
        assert_eq!(
            normalize_content("  My   favourite browser is Brave "),
            "my favourite browser is brave"
        );
    }

    #[test]
    fn validates_memory_content() {
        assert!(validate_content("").is_err());
        assert!(validate_content("Remember this").is_ok());
        assert!(validate_content(&"x".repeat(MAX_MEMORY_CHARS + 1)).is_err());
    }

    #[test]
    fn creates_unique_ids() {
        let records = vec![MemoryRecord {
            id: "memory-10".to_string(),
            content: "One".to_string(),
            source: "user".to_string(),
            created_at_ms: 10,
            updated_at_ms: 10,
        }];

        assert_eq!(unique_memory_id(&records, 10), "memory-10-2");
    }
}
