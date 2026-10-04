use crate::vision_capture::{remove_capture, VisionCapture};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const HISTORY_FILE: &str = "vision-history.json";
const PREFERENCES_FILE: &str = "vision-preferences.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct VisionPreferences {
    pub history_enabled: bool,
    pub retain_images: bool,
    pub max_history: usize,
}

impl Default for VisionPreferences {
    fn default() -> Self {
        Self {
            history_enabled: false,
            retain_images: false,
            max_history: 20,
        }
    }
}

impl VisionPreferences {
    pub fn sanitized(mut self) -> Self {
        self.max_history = self.max_history.clamp(1, 50);
        if !self.history_enabled {
            self.retain_images = false;
        }
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionHistoryItem {
    pub id: String,
    pub capture_kind: String,
    pub prompt: String,
    pub analysis: String,
    pub window_title: Option<String>,
    pub image_path: Option<String>,
    pub created_at_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionHistorySnapshot {
    pub preferences: VisionPreferences,
    pub items: Vec<VisionHistoryItem>,
    pub refreshed_at_ms: u64,
}

pub fn snapshot(app: &AppHandle) -> Result<VisionHistorySnapshot, String> {
    Ok(VisionHistorySnapshot {
        preferences: load_preferences(app),
        items: load_history(app)?,
        refreshed_at_ms: timestamp_ms(),
    })
}

pub fn load_preferences(app: &AppHandle) -> VisionPreferences {
    let Ok(path) = preferences_path(app) else {
        return VisionPreferences::default();
    };

    let Ok(content) = fs::read_to_string(path) else {
        return VisionPreferences::default();
    };

    serde_json::from_str::<VisionPreferences>(&content)
        .unwrap_or_default()
        .sanitized()
}

pub fn save_preferences(
    app: &AppHandle,
    preferences: VisionPreferences,
) -> Result<VisionHistorySnapshot, String> {
    let preferences = preferences.sanitized();
    let path = preferences_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let content =
        serde_json::to_string_pretty(&preferences).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())?;

    if !preferences.retain_images {
        remove_retained_images(app)?;
        let mut history = load_history(app)?;
        for item in &mut history {
            item.image_path = None;
        }
        write_history(app, &history)?;
    }

    snapshot(app)
}

pub fn record_analysis(
    app: &AppHandle,
    capture: &VisionCapture,
    prompt: &str,
    analysis: &str,
) -> Result<VisionHistorySnapshot, String> {
    let preferences = load_preferences(app);

    if !preferences.history_enabled {
        remove_capture(&capture.path);
        return snapshot(app);
    }

    let mut history = load_history(app)?;
    let retained_path = if preferences.retain_images {
        Some(retain_capture(app, capture)?)
    } else {
        remove_capture(&capture.path);
        None
    };

    history.insert(
        0,
        VisionHistoryItem {
            id: capture.id.clone(),
            capture_kind: capture.kind.clone(),
            prompt: prompt.to_string(),
            analysis: analysis.to_string(),
            window_title: capture.window_title.clone(),
            image_path: retained_path,
            created_at_ms: timestamp_ms(),
        },
    );

    while history.len() > preferences.max_history {
        if let Some(removed) = history.pop() {
            if let Some(path) = removed.image_path {
                let _ = fs::remove_file(path);
            }
        }
    }

    write_history(app, &history)?;
    snapshot(app)
}

pub fn clear_history(app: &AppHandle) -> Result<VisionHistorySnapshot, String> {
    remove_retained_images(app)?;
    let path = history_path(app)?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    snapshot(app)
}

fn retain_capture(app: &AppHandle, capture: &VisionCapture) -> Result<String, String> {
    let directory = history_images_dir(app)?;
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let destination = directory.join(format!("{}.png", capture.id));

    fs::copy(&capture.path, &destination)
        .map(|_| ())
        .map_err(|error| format!("Could not retain Vision screenshot: {error}"))?;
    remove_capture(&capture.path);

    Ok(destination.to_string_lossy().to_string())
}

fn remove_retained_images(app: &AppHandle) -> Result<(), String> {
    let directory = history_images_dir(app)?;
    if directory.exists() {
        fs::remove_dir_all(&directory).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn load_history(app: &AppHandle) -> Result<Vec<VisionHistoryItem>, String> {
    let path = history_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content =
        fs::read_to_string(path).map_err(|error| format!("Could not read Vision history: {error}"))?;

    serde_json::from_str(&content)
        .map_err(|error| format!("Vision history is invalid: {error}"))
}

fn write_history(app: &AppHandle, items: &[VisionHistoryItem]) -> Result<(), String> {
    let path = history_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let content = serde_json::to_string_pretty(items).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())
}

fn preferences_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join(PREFERENCES_FILE))
}

fn history_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?
        .join(HISTORY_FILE))
}

fn history_images_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_local_data_dir()
        .map_err(|error| error.to_string())?
        .join("vision-history")
        .join("images"))
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_disable_image_retention_when_history_is_off() {
        let preferences = VisionPreferences {
            history_enabled: false,
            retain_images: true,
            max_history: 999,
        }
        .sanitized();

        assert!(!preferences.retain_images);
        assert_eq!(preferences.max_history, 50);
    }
}
