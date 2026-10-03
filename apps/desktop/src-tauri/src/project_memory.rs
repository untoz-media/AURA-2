use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const PROJECTS_FILENAME: &str = "projects.json";
const MAX_PROJECTS: usize = 128;
const MAX_ALIASES: usize = 12;
const MAX_NOTES: usize = 64;
const MAX_ROUTINE_LINKS: usize = 32;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMemory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub notes: Vec<String>,
    pub routine_ids: Vec<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ProjectStore {
    pub active_project_id: Option<String>,
    pub projects: Vec<ProjectMemory>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMemorySnapshot {
    pub projects: Vec<ProjectMemory>,
    pub active_project_id: Option<String>,
    pub refreshed_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProjectRequest {
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub routine_ids: Vec<String>,
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
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

fn projects_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app.path().app_config_dir().map_err(|error| error.to_string())?;
    Ok(directory.join(PROJECTS_FILENAME))
}

fn read_store(app: &AppHandle) -> Result<ProjectStore, String> {
    let path = projects_path(app)?;
    if !path.exists() {
        return Ok(ProjectStore::default());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read project memory: {error}"))?;
    let mut store = serde_json::from_str::<ProjectStore>(&content)
        .map_err(|error| format!("Project memory is invalid and was left unchanged: {error}"))?;

    store.projects.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
    });

    if store
        .active_project_id
        .as_ref()
        .is_some_and(|id| !store.projects.iter().any(|project| &project.id == id))
    {
        store.active_project_id = None;
    }

    Ok(store)
}

fn write_store(app: &AppHandle, store: &ProjectStore) -> Result<(), String> {
    let path = projects_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let content = serde_json::to_string_pretty(store).map_err(|error| error.to_string())?;
    fs::write(path, content).map_err(|error| error.to_string())
}

fn slugify(value: &str) -> String {
    normalize(value)
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

fn clean_unique(values: Vec<String>, label: &str, limit: usize, max_chars: usize) -> Result<Vec<String>, String> {
    let mut output = Vec::new();

    for value in values {
        let value = validate_text(&value, label, max_chars)?;
        let normalized = normalize(&value);

        if !output
            .iter()
            .any(|existing: &String| normalize(existing) == normalized)
        {
            output.push(value);
        }
    }

    if output.len() > limit {
        return Err(format!("{label} limit exceeded ({limit})."));
    }

    Ok(output)
}

pub fn project_snapshot(app: &AppHandle) -> Result<ProjectMemorySnapshot, String> {
    let store = read_store(app)?;
    Ok(ProjectMemorySnapshot {
        projects: store.projects,
        active_project_id: store.active_project_id,
        refreshed_at_ms: timestamp_ms(),
    })
}

pub fn save_project(app: &AppHandle, request: SaveProjectRequest) -> Result<ProjectMemory, String> {
    let mut store = read_store(app)?;

    if request.id.is_none() && store.projects.len() >= MAX_PROJECTS {
        return Err(format!("AURA supports at most {MAX_PROJECTS} projects."));
    }

    let name = validate_text(&request.name, "Project name", 80)?;
    let description = request.description.trim().to_string();
    if description.chars().count() > 600 {
        return Err("Project description is too long.".to_string());
    }

    let aliases = clean_unique(request.aliases, "Project alias", MAX_ALIASES, 80)?
        .into_iter()
        .filter(|alias| normalize(alias) != normalize(&name))
        .collect::<Vec<_>>();
    let notes = clean_unique(request.notes, "Project note", MAX_NOTES, 500)?;
    let routine_ids = clean_unique(
        request.routine_ids,
        "Routine link",
        MAX_ROUTINE_LINKS,
        160,
    )?;

    let requested_id = request.id.as_deref();
    let mut requested_keys = vec![normalize(&name)];
    requested_keys.extend(aliases.iter().map(|alias| normalize(alias)));

    for project in &store.projects {
        if requested_id == Some(project.id.as_str()) {
            continue;
        }

        let mut existing_keys = vec![normalize(&project.name)];
        existing_keys.extend(project.aliases.iter().map(|alias| normalize(alias)));

        if requested_keys.iter().any(|key| existing_keys.contains(key)) {
            return Err(format!(
                "Project name or alias conflicts with existing project “{}”.",
                project.name
            ));
        }
    }

    let now = timestamp_ms();
    let (id, created_at_ms) = if let Some(id) = requested_id {
        let existing = store
            .projects
            .iter()
            .find(|project| project.id == id)
            .ok_or_else(|| "Project no longer exists.".to_string())?;
        (id.to_string(), existing.created_at_ms)
    } else {
        let slug = {
            let slug = slugify(&name);
            if slug.is_empty() { "project".to_string() } else { slug }
        };
        let base = format!("project-{slug}-{now}");
        let mut candidate = base.clone();
        let mut suffix = 2;
        while store.projects.iter().any(|project| project.id == candidate) {
            candidate = format!("{base}-{suffix}");
            suffix += 1;
        }
        (candidate, now)
    };

    let project = ProjectMemory {
        id: id.clone(),
        name,
        description,
        aliases,
        notes,
        routine_ids,
        created_at_ms,
        updated_at_ms: now,
    };

    if let Some(index) = store.projects.iter().position(|existing| existing.id == id) {
        store.projects[index] = project.clone();
    } else {
        store.projects.push(project.clone());
    }

    store.projects.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
    });

    write_store(app, &store)?;
    Ok(project)
}

pub fn delete_project(app: &AppHandle, project_id: &str) -> Result<(), String> {
    let mut store = read_store(app)?;
    let original_len = store.projects.len();
    store.projects.retain(|project| project.id != project_id);

    if store.projects.len() == original_len {
        return Err("Project no longer exists.".to_string());
    }

    if store.active_project_id.as_deref() == Some(project_id) {
        store.active_project_id = None;
    }

    write_store(app, &store)
}

pub fn set_active_project(
    app: &AppHandle,
    project_id: Option<&str>,
) -> Result<ProjectMemorySnapshot, String> {
    let mut store = read_store(app)?;

    if let Some(project_id) = project_id {
        if !store.projects.iter().any(|project| project.id == project_id) {
            return Err("Project no longer exists.".to_string());
        }
        store.active_project_id = Some(project_id.to_string());
    } else {
        store.active_project_id = None;
    }

    write_store(app, &store)?;
    project_snapshot(app)
}

pub fn find_project_by_name_or_alias(app: &AppHandle, value: &str) -> Option<ProjectMemory> {
    let candidate = normalize(value);
    read_store(app)
        .ok()?
        .projects
        .into_iter()
        .find(|project| {
            normalize(&project.name) == candidate
                || project
                    .aliases
                    .iter()
                    .any(|alias| normalize(alias) == candidate)
        })
}

pub fn active_project(app: &AppHandle) -> Result<Option<ProjectMemory>, String> {
    let store = read_store(app)?;
    let Some(active_id) = store.active_project_id else {
        return Ok(None);
    };

    Ok(store
        .projects
        .into_iter()
        .find(|project| project.id == active_id))
}

pub fn summarize_active_project(app: &AppHandle) -> Result<Option<String>, String> {
    let Some(project) = active_project(app)? else {
        return Ok(None);
    };

    let mut lines = vec![format!("Active project: {}", project.name)];

    if !project.description.is_empty() {
        lines.push(format!("Project description: {}", project.description));
    }

    if !project.notes.is_empty() {
        lines.push(format!("Project notes: {}", project.notes.join(" · ")));
    }

    if !project.routine_ids.is_empty() {
        lines.push(format!(
            "Linked routine IDs: {}",
            project.routine_ids.join(" · ")
        ));
    }

    Ok(Some(lines.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_project_names() {
        assert_eq!(normalize("  Artemis   Documentary  "), "artemis documentary");
    }

    #[test]
    fn cleans_duplicate_values() {
        let values = clean_unique(
            vec!["One".to_string(), " one ".to_string(), "Two".to_string()],
            "Value",
            4,
            20,
        )
        .unwrap();

        assert_eq!(values, vec!["One".to_string(), "Two".to_string()]);
    }
}
