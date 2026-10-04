use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn write_json_atomic<T>(path: &Path, value: &T) -> Result<(), String>
where
    T: Serialize + ?Sized,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create storage directory: {error}"))?;
    }

    let content = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("Could not serialize local state: {error}"))?;
    let temporary = temporary_path(path);

    let write_result = (|| -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|error| format!("Could not create temporary state file: {error}"))?;

        file.write_all(&content)
            .map_err(|error| format!("Could not write temporary state file: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("Could not flush temporary state file: {error}"))?;
        drop(file);

        fs::rename(&temporary, path)
            .map_err(|error| format!("Could not atomically replace local state: {error}"))?;
        Ok(())
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }

    write_result
}

fn temporary_path(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("aura-state.json");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();

    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    parent.join(format!(
        ".{filename}.{}.{}.{}.tmp",
        process::id(),
        nonce,
        counter
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Sample {
        value: u32,
    }

    #[test]
    fn atomic_json_write_replaces_existing_document() {
        let root = std::env::temp_dir().join(format!(
            "aura-storage-test-{}-{}",
            process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|value| value.as_nanos())
                .unwrap_or_default()
        ));
        let path = root.join("state.json");

        write_json_atomic(&path, &Sample { value: 1 }).unwrap();
        write_json_atomic(&path, &Sample { value: 2 }).unwrap();

        let saved: Sample =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved, Sample { value: 2 });

        let leftovers = fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
            .count();
        assert_eq!(leftovers, 0);

        let _ = fs::remove_dir_all(root);
    }
}
