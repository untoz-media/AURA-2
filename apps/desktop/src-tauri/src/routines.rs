use crate::{
    computer::{
        app_launcher::{launch_app, AppTarget},
        window_manager::switch_to_app,
    },
    integrations::{
        director::{
            preset_requires_sensitive_permission, resolve_director_preset_command,
            run_director_preset,
        },
        obs::ObsController,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const ROUTINES_FILENAME: &str = "routines.json";
const MAX_ROUTINES: usize = 64;
const MAX_STEPS: usize = 24;
const MAX_ALIASES: usize = 12;
const MAX_WAIT_MS: u64 = 30_000;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum RoutineStep {
    LaunchApp { app: String },
    SwitchToApp { app: String },
    DirectorPreset { preset: String },
    Wait { milliseconds: u64 },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRoutine {
    pub id: String,
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub steps: Vec<RoutineStep>,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveRoutineRequest {
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub steps: Vec<RoutineStep>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineStepResult {
    pub index: usize,
    pub status: String,
    pub label: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineRunResult {
    pub success: bool,
    pub routine_id: String,
    pub routine_name: String,
    pub completed_steps: usize,
    pub total_steps: usize,
    pub failed_step: Option<usize>,
    pub error: Option<String>,
    pub steps: Vec<RoutineStepResult>,
    pub started_at_ms: u64,
    pub completed_at_ms: u64,
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
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

fn routines_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app.path().app_config_dir().map_err(|error| error.to_string())?;
    Ok(directory.join(ROUTINES_FILENAME))
}

fn read_routines(app: &AppHandle) -> Result<Vec<UserRoutine>, String> {
    let path = routines_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read routines file: {error}"))?;

    let mut routines = serde_json::from_str::<Vec<UserRoutine>>(&content)
        .map_err(|error| format!("Routines file is invalid and was left unchanged: {error}"))?;

    routines.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    Ok(routines)
}

fn write_routines(app: &AppHandle, routines: &[UserRoutine]) -> Result<(), String> {
    let path = routines_path(app)?;
    crate::storage::write_json_atomic(&path, routines)
}

fn validate_text(value: &str, label: &str, max_chars: usize) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label} cannot be empty."));
    }
    if value.chars().count() > max_chars {
        return Err(format!("{label} is too long."));
    }
    Ok(value.to_string())
}

fn validate_step(step: &RoutineStep) -> Result<(), String> {
    match step {
        RoutineStep::LaunchApp { app } | RoutineStep::SwitchToApp { app } => {
            let app = normalize_phrase(app);
            if AppTarget::from_alias(&app).is_none() {
                return Err(format!("Unsupported routine application: {app}."));
            }
        }
        RoutineStep::DirectorPreset { preset } => {
            validate_text(preset, "Director preset", 96)?;
        }
        RoutineStep::Wait { milliseconds } => {
            if *milliseconds > MAX_WAIT_MS {
                return Err(format!(
                    "A routine wait step cannot exceed {} seconds.",
                    MAX_WAIT_MS / 1000
                ));
            }
        }
    }
    Ok(())
}

fn slugify(value: &str) -> String {
    normalize_phrase(value)
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn list_routines(app: &AppHandle) -> Result<Vec<UserRoutine>, String> {
    read_routines(app)
}

pub fn save_routine(app: &AppHandle, request: SaveRoutineRequest) -> Result<UserRoutine, String> {
    let mut routines = read_routines(app)?;

    if request.id.is_none() && routines.len() >= MAX_ROUTINES {
        return Err(format!("AURA supports at most {MAX_ROUTINES} routines."));
    }

    let name = validate_text(&request.name, "Routine name", 64)?;
    let description = request.description.trim().to_string();
    if description.chars().count() > 240 {
        return Err("Routine description is too long.".to_string());
    }
    if request.steps.is_empty() {
        return Err("A routine needs at least one step.".to_string());
    }
    if request.steps.len() > MAX_STEPS {
        return Err(format!("A routine can contain at most {MAX_STEPS} steps."));
    }

    for step in &request.steps {
        validate_step(step)?;
    }

    let mut aliases = Vec::new();
    for alias in &request.aliases {
        let alias = validate_text(alias, "Routine alias", 64)?;
        if normalize_phrase(&alias) == normalize_phrase(&name) {
            continue;
        }
        if aliases
            .iter()
            .any(|existing: &String| normalize_phrase(existing) == normalize_phrase(&alias))
        {
            continue;
        }
        aliases.push(alias);
    }
    if aliases.len() > MAX_ALIASES {
        return Err(format!("A routine can contain at most {MAX_ALIASES} aliases."));
    }

    let requested_id = request.id.as_deref();
    let mut requested_keys = vec![normalize_phrase(&name)];
    requested_keys.extend(aliases.iter().map(|alias| normalize_phrase(alias)));

    for routine in &routines {
        if requested_id == Some(routine.id.as_str()) {
            continue;
        }

        let mut existing_keys = vec![normalize_phrase(&routine.name)];
        existing_keys.extend(routine.aliases.iter().map(|alias| normalize_phrase(alias)));

        if requested_keys.iter().any(|key| existing_keys.contains(key)) {
            return Err(format!(
                "Routine name or alias conflicts with existing routine “{}”.",
                routine.name
            ));
        }
    }

    let now = timestamp_ms();
    let id = if let Some(id) = requested_id {
        if !routines.iter().any(|routine| routine.id == id) {
            return Err("Routine no longer exists.".to_string());
        }
        id.to_string()
    } else {
        let slug = {
            let slug = slugify(&name);
            if slug.is_empty() { "routine".to_string() } else { slug }
        };
        let base = format!("routine-{slug}-{now}");
        let mut candidate = base.clone();
        let mut suffix = 2;
        while routines.iter().any(|routine| routine.id == candidate) {
            candidate = format!("{base}-{suffix}");
            suffix += 1;
        }
        candidate
    };

    let routine = UserRoutine {
        id: id.clone(),
        name,
        description,
        aliases,
        steps: request.steps,
        updated_at_ms: now,
    };

    if let Some(index) = routines.iter().position(|existing| existing.id == id) {
        routines[index] = routine.clone();
    } else {
        routines.push(routine.clone());
    }

    routines.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    write_routines(app, &routines)?;
    Ok(routine)
}

pub fn delete_routine(app: &AppHandle, routine_id: &str) -> Result<(), String> {
    let mut routines = read_routines(app)?;
    let original_len = routines.len();
    routines.retain(|routine| routine.id != routine_id);

    if routines.len() == original_len {
        return Err("Routine no longer exists.".to_string());
    }

    write_routines(app, &routines)
}

pub fn find_routine_by_id(app: &AppHandle, routine_id: &str) -> Option<UserRoutine> {
    read_routines(app)
        .ok()?
        .into_iter()
        .find(|routine| routine.id == routine_id)
}

pub fn resolve_routine_command(app: &AppHandle, input: &str) -> Option<UserRoutine> {
    let routines = read_routines(app).ok()?;
    let normalized = normalize_phrase(input);

    let target = [
        "run routine ",
        "execute routine ",
        "start routine ",
        "executa a rotina ",
        "executa rotina ",
        "inicia a rotina ",
        "inicia rotina ",
    ]
    .iter()
    .find_map(|prefix| normalized.strip_prefix(prefix))
    .map(str::trim)
    .filter(|value| !value.is_empty());

    let candidate = target.unwrap_or(normalized.as_str());

    routines.into_iter().find(|routine| {
        normalize_phrase(&routine.name) == candidate
            || routine
                .aliases
                .iter()
                .any(|alias| normalize_phrase(alias) == candidate)
    })
}

pub fn routine_requires_sensitive_permission(app: &AppHandle, routine: &UserRoutine) -> bool {
    routine.steps.iter().any(|step| match step {
        RoutineStep::DirectorPreset { preset } => resolve_director_preset_command(app, preset)
            .is_some_and(|resolved| preset_requires_sensitive_permission(&resolved)),
        _ => false,
    })
}

fn step_label(step: &RoutineStep) -> String {
    match step {
        RoutineStep::LaunchApp { app } => format!("Launch {app}"),
        RoutineStep::SwitchToApp { app } => format!("Switch to {app}"),
        RoutineStep::DirectorPreset { preset } => format!("Director preset {preset}"),
        RoutineStep::Wait { milliseconds } => format!("Wait {milliseconds} ms"),
    }
}

pub async fn run_routine(
    app: &AppHandle,
    obs: &ObsController,
    routine: &UserRoutine,
) -> RoutineRunResult {
    let started_at_ms = timestamp_ms();
    let total_steps = routine.steps.len();
    let mut results = Vec::with_capacity(total_steps);
    let mut completed_steps = 0;

    for (index, step) in routine.steps.iter().enumerate() {
        let label = step_label(step);
        let outcome: Result<String, String> = match step {
            RoutineStep::LaunchApp { app: target } => {
                let normalized = normalize_phrase(target);
                let target = AppTarget::from_alias(&normalized)
                    .ok_or_else(|| format!("Unsupported application: {normalized}."));
                target.and_then(|target| {
                    launch_app(target)
                        .map(|_| format!("Opened {}.", target.display_name()))
                        .map_err(|error| error.to_string())
                })
            }
            RoutineStep::SwitchToApp { app: target } => {
                let normalized = normalize_phrase(target);
                let target = AppTarget::from_alias(&normalized)
                    .ok_or_else(|| format!("Unsupported application: {normalized}."));
                target.and_then(|target| {
                    switch_to_app(target)
                        .map(|window| format!("Focused {}: {}.", target.display_name(), window.title))
                        .map_err(|error| error.to_string())
                })
            }
            RoutineStep::DirectorPreset { preset } => {
                match resolve_director_preset_command(app, preset) {
                    Some(resolved) => {
                        let run = run_director_preset(obs, &resolved).await;
                        if run.success {
                            Ok(format!(
                                "Director preset {} completed ({} steps).",
                                run.preset_name, run.completed_steps
                            ))
                        } else {
                            Err(run
                                .error
                                .unwrap_or_else(|| "Director preset failed.".to_string()))
                        }
                    }
                    None => Err(format!("Director preset “{preset}” was not found.")),
                }
            }
            RoutineStep::Wait { milliseconds } => {
                tokio::time::sleep(Duration::from_millis(*milliseconds)).await;
                Ok(format!("Waited {milliseconds} ms."))
            }
        };

        match outcome {
            Ok(message) => {
                results.push(RoutineStepResult {
                    index,
                    status: "applied".to_string(),
                    label,
                    message,
                });
                completed_steps += 1;
            }
            Err(error) => {
                results.push(RoutineStepResult {
                    index,
                    status: "failed".to_string(),
                    label,
                    message: error.clone(),
                });

                return RoutineRunResult {
                    success: false,
                    routine_id: routine.id.clone(),
                    routine_name: routine.name.clone(),
                    completed_steps,
                    total_steps,
                    failed_step: Some(index),
                    error: Some(error),
                    steps: results,
                    started_at_ms,
                    completed_at_ms: timestamp_ms(),
                };
            }
        }
    }

    RoutineRunResult {
        success: true,
        routine_id: routine.id.clone(),
        routine_name: routine.name.clone(),
        completed_steps,
        total_steps,
        failed_step: None,
        error: None,
        steps: results,
        started_at_ms,
        completed_at_ms: timestamp_ms(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_routine_phrases() {
        assert_eq!(normalize_phrase("  Start Editing! "), "start editing");
    }

    #[test]
    fn rejects_unknown_app_targets() {
        assert!(validate_step(&RoutineStep::LaunchApp {
            app: "unknown app".to_string()
        })
        .is_err());
    }

    #[test]
    fn caps_wait_steps() {
        assert!(validate_step(&RoutineStep::Wait {
            milliseconds: MAX_WAIT_MS + 1
        })
        .is_err());
    }
}
