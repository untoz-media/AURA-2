use super::obs::ObsController;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, time::{Duration, SystemTime, UNIX_EPOCH}};
use tauri::{AppHandle, Manager};

const PRESETS_FILENAME: &str = "director-presets.json";
const MAX_PRESETS: usize = 64;
const MAX_ACTIONS_PER_PRESET: usize = 32;
const MAX_ALIASES_PER_PRESET: usize = 12;
const MAX_WAIT_MS: u64 = 30_000;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DirectorRecordingAction {
    Start,
    Stop,
    Pause,
    Resume,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DirectorStreamingAction {
    Start,
    Stop,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum DirectorPresetAction {
    ProgramScene {
        scene_name: String,
    },
    PreviewScene {
        scene_name: String,
    },
    SourceVisibility {
        source_name: String,
        enabled: bool,
    },
    AudioMute {
        input_name: String,
        muted: bool,
    },
    AudioVolume {
        input_name: String,
        percent: u8,
    },
    Recording {
        action: DirectorRecordingAction,
    },
    Streaming {
        action: DirectorStreamingAction,
    },
    Wait {
        milliseconds: u64,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectorPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub actions: Vec<DirectorPresetAction>,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDirectorPresetRequest {
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub actions: Vec<DirectorPresetAction>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectorPresetStepResult {
    pub index: usize,
    pub status: String,
    pub label: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectorPresetRunResult {
    pub success: bool,
    pub preset_id: String,
    pub preset_name: String,
    pub completed_steps: usize,
    pub total_steps: usize,
    pub failed_step: Option<usize>,
    pub error: Option<String>,
    pub steps: Vec<DirectorPresetStepResult>,
    pub started_at_ms: u64,
    pub completed_at_ms: u64,
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn presets_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app.path().app_config_dir().map_err(|error| error.to_string())?;
    Ok(directory.join(PRESETS_FILENAME))
}

fn normalize_phrase(value: &str) -> String {
    value
        .trim()
        .trim_matches(|c: char| matches!(c, '.' | ',' | '!' | '?' | ';' | ':'))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn validate_text(value: &str, label: &str, max_chars: usize) -> Result<String, String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(format!("{label} cannot be empty."));
    }

    if trimmed.chars().count() > max_chars {
        return Err(format!("{label} is too long."));
    }

    Ok(trimmed.to_string())
}

fn validate_action(action: &DirectorPresetAction) -> Result<(), String> {
    match action {
        DirectorPresetAction::ProgramScene { scene_name }
        | DirectorPresetAction::PreviewScene { scene_name } => {
            validate_text(scene_name, "Scene name", 256)?;
        }
        DirectorPresetAction::SourceVisibility { source_name, .. } => {
            validate_text(source_name, "Source name", 256)?;
        }
        DirectorPresetAction::AudioMute { input_name, .. } => {
            validate_text(input_name, "Audio input name", 256)?;
        }
        DirectorPresetAction::AudioVolume {
            input_name,
            percent,
        } => {
            validate_text(input_name, "Audio input name", 256)?;
            if *percent > 100 {
                return Err("Audio volume must be between 0 and 100 percent.".to_string());
            }
        }
        DirectorPresetAction::Wait { milliseconds } => {
            if *milliseconds > MAX_WAIT_MS {
                return Err(format!(
                    "A Director Mode wait step cannot exceed {} seconds.",
                    MAX_WAIT_MS / 1_000
                ));
            }
        }
        DirectorPresetAction::Recording { .. } | DirectorPresetAction::Streaming { .. } => {}
    }

    Ok(())
}

fn validate_preset_request(
    request: &SaveDirectorPresetRequest,
    existing: &[DirectorPreset],
) -> Result<(String, String, Vec<String>), String> {
    let name = validate_text(&request.name, "Preset name", 64)?;
    let description = request.description.trim().to_string();
    if description.chars().count() > 240 {
        return Err("Preset description is too long.".to_string());
    }

    if request.actions.is_empty() {
        return Err("A Director Mode preset needs at least one action.".to_string());
    }

    if request.actions.len() > MAX_ACTIONS_PER_PRESET {
        return Err(format!(
            "A Director Mode preset can contain at most {MAX_ACTIONS_PER_PRESET} actions."
        ));
    }

    for action in &request.actions {
        validate_action(action)?;
    }

    let mut aliases = Vec::new();
    for alias in &request.aliases {
        let alias = validate_text(alias, "Preset alias", 64)?;
        let normalized = normalize_phrase(&alias);

        if normalize_phrase(&name) == normalized {
            continue;
        }

        if aliases
            .iter()
            .any(|existing: &String| normalize_phrase(existing) == normalized)
        {
            continue;
        }

        aliases.push(alias);
    }

    if aliases.len() > MAX_ALIASES_PER_PRESET {
        return Err(format!(
            "A Director Mode preset can contain at most {MAX_ALIASES_PER_PRESET} aliases."
        ));
    }

    let requested_id = request.id.as_deref();
    let mut keys = vec![normalize_phrase(&name)];
    keys.extend(aliases.iter().map(|alias| normalize_phrase(alias)));

    for preset in existing {
        if requested_id == Some(preset.id.as_str()) {
            continue;
        }

        let mut existing_keys = vec![normalize_phrase(&preset.name)];
        existing_keys.extend(preset.aliases.iter().map(|alias| normalize_phrase(alias)));

        if keys.iter().any(|key| existing_keys.contains(key)) {
            return Err(format!(
                "Preset name or alias conflicts with existing preset “{}”.",
                preset.name
            ));
        }
    }

    Ok((name, description, aliases))
}

fn write_presets(app: &AppHandle, presets: &[DirectorPreset]) -> Result<(), String> {
    let path = presets_path(app)?;

    crate::storage::write_json_atomic(&path, presets)
}

fn read_director_presets(app: &AppHandle) -> Result<Vec<DirectorPreset>, String> {
    let path = presets_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read Director presets: {error}"))?;
    let mut presets = serde_json::from_str::<Vec<DirectorPreset>>(&content)
        .map_err(|error| format!("Director presets file is invalid and was left unchanged: {error}"))?;
    presets.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    Ok(presets)
}

pub fn validate_director_store(app: &AppHandle) -> Result<usize, String> {
    read_director_presets(app).map(|presets| presets.len())
}

pub fn load_director_presets(app: &AppHandle) -> Vec<DirectorPreset> {
    read_director_presets(app).unwrap_or_default()
}

pub fn save_director_preset(
    app: &AppHandle,
    request: SaveDirectorPresetRequest,
) -> Result<DirectorPreset, String> {
    let mut presets = read_director_presets(app)?;

    if request.id.is_none() && presets.len() >= MAX_PRESETS {
        return Err(format!(
            "Director Mode supports at most {MAX_PRESETS} saved presets."
        ));
    }

    let (name, description, aliases) = validate_preset_request(&request, &presets)?;
    let now = timestamp_ms();

    let id = if let Some(id) = request.id.as_deref() {
        if !presets.iter().any(|preset| preset.id == id) {
            return Err("Director Mode preset no longer exists.".to_string());
        }
        id.to_string()
    } else {
        let slug = {
            let value = slugify(&name);
            if value.is_empty() {
                "preset".to_string()
            } else {
                value
            }
        };
        let base = format!("director-{slug}-{now}");
        let mut candidate = base.clone();
        let mut suffix = 2_u32;

        while presets.iter().any(|preset| preset.id == candidate) {
            candidate = format!("{base}-{suffix}");
            suffix += 1;
        }

        candidate
    };

    let preset = DirectorPreset {
        id: id.clone(),
        name,
        description,
        aliases,
        actions: request.actions,
        updated_at_ms: now,
    };

    if let Some(index) = presets.iter().position(|existing| existing.id == id) {
        presets[index] = preset.clone();
    } else {
        presets.push(preset.clone());
    }

    presets.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    write_presets(app, &presets)?;

    Ok(preset)
}

pub fn delete_director_preset(app: &AppHandle, preset_id: &str) -> Result<(), String> {
    let mut presets = read_director_presets(app)?;
    let original_len = presets.len();
    presets.retain(|preset| preset.id != preset_id);

    if presets.len() == original_len {
        return Err("Director Mode preset no longer exists.".to_string());
    }

    write_presets(app, &presets)
}

pub fn find_director_preset_by_id(app: &AppHandle, preset_id: &str) -> Option<DirectorPreset> {
    load_director_presets(app)
        .into_iter()
        .find(|preset| preset.id == preset_id)
}

pub fn resolve_director_preset_command_checked(
    app: &AppHandle,
    input: &str,
) -> Result<Option<DirectorPreset>, String> {
    Ok(resolve_director_preset_from_list(
        read_director_presets(app)?,
        input,
    ))
}

pub fn resolve_director_preset_command(app: &AppHandle, input: &str) -> Option<DirectorPreset> {
    resolve_director_preset_command_checked(app, input).ok().flatten()
}

fn resolve_director_preset_from_list(
    presets: Vec<DirectorPreset>,
    input: &str,
) -> Option<DirectorPreset> {
    let normalized = normalize_phrase(input);

    let target = [
        "run director preset ",
        "run preset ",
        "execute preset ",
        "executa o preset ",
        "executa preset ",
        "ativa o preset ",
        "ativa preset ",
    ]
    .iter()
    .find_map(|prefix| normalized.strip_prefix(prefix))
    .map(str::trim)
    .filter(|value| !value.is_empty());

    if let Some(target) = target {
        return presets.into_iter().find(|preset| preset_matches(preset, target));
    }

    presets
        .into_iter()
        .find(|preset| preset_matches(preset, &normalized))
}

pub fn preset_requires_sensitive_permission(preset: &DirectorPreset) -> bool {
    preset.actions.iter().any(|action| {
        matches!(
            action,
            DirectorPresetAction::Streaming {
                action: DirectorStreamingAction::Start
            }
        )
    })
}

pub async fn run_director_preset(
    obs: &ObsController,
    preset: &DirectorPreset,
) -> DirectorPresetRunResult {
    let started_at_ms = timestamp_ms();
    let total_steps = preset.actions.len();
    let mut steps = Vec::with_capacity(total_steps);
    let mut completed_steps = 0;

    for (index, action) in preset.actions.iter().enumerate() {
        let label = action_label(action);
        let outcome = execute_action(obs, action).await;

        match outcome {
            Ok((status, message)) => {
                steps.push(DirectorPresetStepResult {
                    index,
                    status,
                    label,
                    message,
                });
                completed_steps += 1;
            }
            Err(error) => {
                steps.push(DirectorPresetStepResult {
                    index,
                    status: "failed".to_string(),
                    label,
                    message: error.clone(),
                });

                return DirectorPresetRunResult {
                    success: false,
                    preset_id: preset.id.clone(),
                    preset_name: preset.name.clone(),
                    completed_steps,
                    total_steps,
                    failed_step: Some(index),
                    error: Some(error),
                    steps,
                    started_at_ms,
                    completed_at_ms: timestamp_ms(),
                };
            }
        }
    }

    DirectorPresetRunResult {
        success: true,
        preset_id: preset.id.clone(),
        preset_name: preset.name.clone(),
        completed_steps,
        total_steps,
        failed_step: None,
        error: None,
        steps,
        started_at_ms,
        completed_at_ms: timestamp_ms(),
    }
}

async fn execute_action(
    obs: &ObsController,
    action: &DirectorPresetAction,
) -> Result<(String, String), String> {
    match action {
        DirectorPresetAction::ProgramScene { scene_name } => {
            let result = obs.set_program_scene_by_name(scene_name).await?;
            Ok((
                "applied".to_string(),
                format!("Program switched to {}.", result.scene_name),
            ))
        }
        DirectorPresetAction::PreviewScene { scene_name } => {
            let result = obs.set_preview_scene_by_name(scene_name).await?;
            Ok((
                "applied".to_string(),
                format!("Preview switched to {}.", result.scene_name),
            ))
        }
        DirectorPresetAction::SourceVisibility {
            source_name,
            enabled,
        } => {
            let result = obs
                .set_source_visibility_by_name(source_name, *enabled)
                .await?;
            Ok((
                "applied".to_string(),
                format!(
                    "{} {} in Program scene {}.",
                    if result.enabled { "Showing" } else { "Hidden" },
                    result.source_name,
                    result.scene_name
                ),
            ))
        }
        DirectorPresetAction::AudioMute { input_name, muted } => {
            let result = obs.set_audio_muted_by_name(input_name, *muted).await?;
            Ok((
                "applied".to_string(),
                format!(
                    "{} is now {}.",
                    result.input_name,
                    if result.muted { "muted" } else { "unmuted" }
                ),
            ))
        }
        DirectorPresetAction::AudioVolume {
            input_name,
            percent,
        } => {
            let result = obs.set_audio_volume_by_name(input_name, *percent).await?;
            Ok((
                "applied".to_string(),
                format!(
                    "{} volume set to {}%.",
                    result.input_name, result.volume_percent
                ),
            ))
        }
        DirectorPresetAction::Recording { action } => {
            let state = obs.runtime_state().await;
            if !state.available {
                return Err(
                    state
                        .last_error
                        .unwrap_or_else(|| "OBS Studio is not connected.".to_string()),
                );
            }

            match action {
                DirectorRecordingAction::Start if state.recording => Ok((
                    "skipped".to_string(),
                    "Recording is already active.".to_string(),
                )),
                DirectorRecordingAction::Stop if !state.recording => Ok((
                    "skipped".to_string(),
                    "Recording is already stopped.".to_string(),
                )),
                DirectorRecordingAction::Pause if !state.recording => {
                    Err("Cannot pause because recording is not active.".to_string())
                }
                DirectorRecordingAction::Pause if state.recording_paused => Ok((
                    "skipped".to_string(),
                    "Recording is already paused.".to_string(),
                )),
                DirectorRecordingAction::Resume if !state.recording => {
                    Err("Cannot resume because recording is not active.".to_string())
                }
                DirectorRecordingAction::Resume if !state.recording_paused => Ok((
                    "skipped".to_string(),
                    "Recording is already running.".to_string(),
                )),
                DirectorRecordingAction::Start => {
                    obs.start_recording().await?;
                    Ok(("applied".to_string(), "Recording started.".to_string()))
                }
                DirectorRecordingAction::Stop => {
                    let result = obs.stop_recording().await?;
                    Ok((
                        "applied".to_string(),
                        result
                            .output_path
                            .map(|path| format!("Recording stopped. Saved to {path}."))
                            .unwrap_or_else(|| "Recording stopped.".to_string()),
                    ))
                }
                DirectorRecordingAction::Pause => {
                    obs.pause_recording().await?;
                    Ok(("applied".to_string(), "Recording paused.".to_string()))
                }
                DirectorRecordingAction::Resume => {
                    obs.resume_recording().await?;
                    Ok(("applied".to_string(), "Recording resumed.".to_string()))
                }
            }
        }
        DirectorPresetAction::Streaming { action } => {
            let state = obs.runtime_state().await;
            if !state.available {
                return Err(
                    state
                        .last_error
                        .unwrap_or_else(|| "OBS Studio is not connected.".to_string()),
                );
            }

            match action {
                DirectorStreamingAction::Start if state.streaming => Ok((
                    "skipped".to_string(),
                    "Stream is already live.".to_string(),
                )),
                DirectorStreamingAction::Stop if !state.streaming => Ok((
                    "skipped".to_string(),
                    "Stream is already offline.".to_string(),
                )),
                DirectorStreamingAction::Start => {
                    obs.start_streaming().await?;
                    Ok(("applied".to_string(), "Stream is live.".to_string()))
                }
                DirectorStreamingAction::Stop => {
                    obs.stop_streaming().await?;
                    Ok(("applied".to_string(), "Stream stopped.".to_string()))
                }
            }
        }
        DirectorPresetAction::Wait { milliseconds } => {
            tokio::time::sleep(Duration::from_millis(*milliseconds)).await;
            Ok((
                "applied".to_string(),
                format!("Waited {} ms.", milliseconds),
            ))
        }
    }
}

fn preset_matches(preset: &DirectorPreset, candidate: &str) -> bool {
    let candidate = normalize_phrase(candidate);

    normalize_phrase(&preset.name) == candidate
        || preset
            .aliases
            .iter()
            .any(|alias| normalize_phrase(alias) == candidate)
}

fn slugify(value: &str) -> String {
    let mut slug = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();

    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }

    slug.trim_matches('-').chars().take(40).collect()
}

fn action_label(action: &DirectorPresetAction) -> String {
    match action {
        DirectorPresetAction::ProgramScene { scene_name } => {
            format!("Program → {scene_name}")
        }
        DirectorPresetAction::PreviewScene { scene_name } => {
            format!("Preview → {scene_name}")
        }
        DirectorPresetAction::SourceVisibility {
            source_name,
            enabled,
        } => format!("{} {source_name}", if *enabled { "Show" } else { "Hide" }),
        DirectorPresetAction::AudioMute { input_name, muted } => {
            format!("{} {input_name}", if *muted { "Mute" } else { "Unmute" })
        }
        DirectorPresetAction::AudioVolume {
            input_name,
            percent,
        } => format!("{input_name} → {percent}%"),
        DirectorPresetAction::Recording { action } => {
            format!("Recording → {}", format!("{action:?}").to_lowercase())
        }
        DirectorPresetAction::Streaming { action } => {
            format!("Streaming → {}", format!("{action:?}").to_lowercase())
        }
        DirectorPresetAction::Wait { milliseconds } => {
            format!("Wait {} ms", milliseconds)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_sensitive_stream_start() {
        let preset = DirectorPreset {
            id: "test".to_string(),
            name: "Start Show".to_string(),
            description: String::new(),
            aliases: vec![],
            actions: vec![DirectorPresetAction::Streaming {
                action: DirectorStreamingAction::Start,
            }],
            updated_at_ms: 1,
        };

        assert!(preset_requires_sensitive_permission(&preset));
    }

    #[test]
    fn non_streaming_preset_is_not_sensitive() {
        let preset = DirectorPreset {
            id: "test".to_string(),
            name: "Break".to_string(),
            description: String::new(),
            aliases: vec![],
            actions: vec![DirectorPresetAction::ProgramScene {
                scene_name: "Break".to_string(),
            }],
            updated_at_ms: 1,
        };

        assert!(!preset_requires_sensitive_permission(&preset));
    }

    #[test]
    fn normalizes_names_and_aliases() {
        let preset = DirectorPreset {
            id: "test".to_string(),
            name: "Prepare Match".to_string(),
            description: String::new(),
            aliases: vec!["Ready the match".to_string()],
            actions: vec![],
            updated_at_ms: 1,
        };

        assert!(preset_matches(&preset, "prepare   match!"));
        assert!(preset_matches(&preset, "READY THE MATCH"));
    }

    #[test]
    fn resolves_bare_alias_and_explicit_preset_command() {
        let preset = DirectorPreset {
            id: "prepare".to_string(),
            name: "Prepare Match".to_string(),
            description: String::new(),
            aliases: vec!["ready the match".to_string()],
            actions: vec![DirectorPresetAction::ProgramScene {
                scene_name: "Match".to_string(),
            }],
            updated_at_ms: 1,
        };

        let by_alias =
            resolve_director_preset_from_list(vec![preset.clone()], "Ready the match!");
        assert_eq!(by_alias.as_ref().map(|item| item.id.as_str()), Some("prepare"));

        let by_command =
            resolve_director_preset_from_list(vec![preset], "Run preset Prepare Match");
        assert_eq!(
            by_command.as_ref().map(|item| item.id.as_str()),
            Some("prepare")
        );
    }

    #[test]
    fn validates_audio_volume_range() {
        assert!(validate_action(&DirectorPresetAction::AudioVolume {
            input_name: "Mic/Aux".to_string(),
            percent: 100,
        })
        .is_ok());

        assert!(validate_action(&DirectorPresetAction::AudioVolume {
            input_name: "Mic/Aux".to_string(),
            percent: 101,
        })
        .is_err());
    }

    #[test]
    fn rejects_overlong_description() {
        let request = SaveDirectorPresetRequest {
            id: None,
            name: "Test".to_string(),
            description: "x".repeat(241),
            aliases: vec![],
            actions: vec![DirectorPresetAction::Wait { milliseconds: 0 }],
        };

        assert!(validate_preset_request(&request, &[]).is_err());
    }

    #[test]
    fn validates_wait_limit() {
        assert!(validate_action(&DirectorPresetAction::Wait {
            milliseconds: MAX_WAIT_MS
        })
        .is_ok());
        assert!(validate_action(&DirectorPresetAction::Wait {
            milliseconds: MAX_WAIT_MS + 1
        })
        .is_err());
    }
}
