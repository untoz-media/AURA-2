use serde::Serialize;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const MAX_SEARCH_DEPTH: usize = 4;
const MAX_SCANNED_ENTRIES: usize = 8_000;
const MAX_RESULTS: usize = 20;
const MAX_QUERY_CHARS: usize = 120;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchItem {
    pub name: String,
    pub path: String,
    pub root_label: String,
    pub kind: String,
    pub extension: Option<String>,
    pub size_bytes: Option<u64>,
    pub modified_at_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSearchSnapshot {
    pub query: String,
    pub items: Vec<FileSearchItem>,
    pub roots: Vec<String>,
    pub scanned_entries: usize,
    pub scan_limit_reached: bool,
    pub max_depth: usize,
    pub refreshed_at_ms: u64,
}

#[derive(Clone)]
struct SearchRoot {
    label: &'static str,
    path: PathBuf,
}

struct SearchState {
    scanned_entries: usize,
    scan_limit_reached: bool,
    matches: Vec<(u16, FileSearchItem)>,
}

pub fn search_personal_files(
    app: &AppHandle,
    query: &str,
) -> Result<FileSearchSnapshot, String> {
    let query = sanitize_query(query)?;
    let normalized = query.to_lowercase();
    let tokens = normalized
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    let roots = personal_roots(app);
    if roots.is_empty() {
        return Err("AURA could not resolve any personal Windows folders to search.".to_string());
    }

    let mut state = SearchState {
        scanned_entries: 0,
        scan_limit_reached: false,
        matches: Vec::new(),
    };

    for root in &roots {
        scan_directory(
            &root.path,
            root,
            0,
            &normalized,
            &tokens,
            &mut state,
        );

        if state.scan_limit_reached {
            break;
        }
    }

    state.matches.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.modified_at_ms.cmp(&left.1.modified_at_ms))
            .then_with(|| left.1.name.to_lowercase().cmp(&right.1.name.to_lowercase()))
    });
    state.matches.truncate(MAX_RESULTS);

    Ok(FileSearchSnapshot {
        query,
        items: state.matches.into_iter().map(|(_, item)| item).collect(),
        roots: roots
            .iter()
            .map(|root| root.label.to_string())
            .collect(),
        scanned_entries: state.scanned_entries,
        scan_limit_reached: state.scan_limit_reached,
        max_depth: MAX_SEARCH_DEPTH,
        refreshed_at_ms: timestamp_ms(),
    })
}

pub fn summarize_file_search(snapshot: &FileSearchSnapshot) -> String {
    if snapshot.items.is_empty() {
        let suffix = if snapshot.scan_limit_reached {
            " The bounded scan reached its entry limit before every folder could be inspected."
        } else {
            ""
        };
        return format!(
            "No matching files or folders were found for “{}” in {}.{}",
            snapshot.query,
            snapshot.roots.join(", "),
            suffix
        );
    }

    let mut lines = vec![format!(
        "Found {} match{} for “{}”:",
        snapshot.items.len(),
        if snapshot.items.len() == 1 { "" } else { "es" },
        snapshot.query
    )];

    for (index, item) in snapshot.items.iter().take(10).enumerate() {
        lines.push(format!(
            "{}. {} · {} · {}",
            index + 1,
            item.name,
            item.root_label,
            item.path
        ));
    }

    if snapshot.items.len() > 10 {
        lines.push(format!(
            "…and {} more bounded result{}.",
            snapshot.items.len() - 10,
            if snapshot.items.len() - 10 == 1 { "" } else { "s" }
        ));
    }

    if snapshot.scan_limit_reached {
        lines.push(
            "The search reached AURA's 8,000-entry safety limit, so results may be incomplete."
                .to_string(),
        );
    }

    lines.push(
        "Tip: use “Reveal file <full path>” to show a result safely in File Explorer."
            .to_string(),
    );

    lines.join("\n")
}

pub fn reveal_personal_path(app: &AppHandle, value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("File path cannot be empty.".to_string());
    }

    let target = PathBuf::from(value);
    if !target.is_absolute() {
        return Err("Reveal file requires an absolute path from a File Intelligence result.".to_string());
    }

    let canonical_target = fs::canonicalize(&target)
        .map_err(|_| "That file or folder no longer exists.".to_string())?;

    let allowed = personal_roots(app)
        .into_iter()
        .filter_map(|root| fs::canonicalize(root.path).ok())
        .any(|root| canonical_target.starts_with(root));

    if !allowed {
        return Err(
            "AURA only reveals paths inside Desktop, Documents, Downloads, Pictures, Videos or Music."
                .to_string(),
        );
    }

    let argument = format!("/select,{}", canonical_target.to_string_lossy());
    Command::new("explorer.exe")
        .arg(argument)
        .spawn()
        .map_err(|error| format!("Could not open File Explorer: {error}"))?;

    Ok(format!(
        "Revealed {} in File Explorer.",
        canonical_target
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_else(|| canonical_target.to_string_lossy())
    ))
}

fn personal_roots(app: &AppHandle) -> Vec<SearchRoot> {
    let resolver = app.path();
    let candidates = [
        ("Desktop", resolver.desktop_dir()),
        ("Documents", resolver.document_dir()),
        ("Downloads", resolver.download_dir()),
        ("Pictures", resolver.picture_dir()),
        ("Videos", resolver.video_dir()),
        ("Music", resolver.audio_dir()),
    ];

    let mut seen = HashSet::new();
    let mut roots = Vec::new();

    for (label, result) in candidates {
        let Ok(path) = result else {
            continue;
        };
        if !path.exists() || !path.is_dir() {
            continue;
        }

        let identity = path.to_string_lossy().to_lowercase();
        if seen.insert(identity) {
            roots.push(SearchRoot { label, path });
        }
    }

    roots
}

fn scan_directory(
    directory: &Path,
    root: &SearchRoot,
    depth: usize,
    normalized_query: &str,
    tokens: &[&str],
    state: &mut SearchState,
) {
    if depth > MAX_SEARCH_DEPTH || state.scan_limit_reached {
        return;
    }

    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        if state.scanned_entries >= MAX_SCANNED_ENTRIES {
            state.scan_limit_reached = true;
            return;
        }
        state.scanned_entries += 1;

        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if name.is_empty() || name.starts_with('.') {
            continue;
        }

        let path = entry.path();
        let normalized_name = name.to_lowercase();

        if let Some(score) = match_score(&normalized_name, normalized_query, tokens) {
            let metadata = entry.metadata().ok();
            let modified_at_ms = metadata
                .as_ref()
                .and_then(|metadata| metadata.modified().ok())
                .map(system_time_ms)
                .unwrap_or(0);

            state.matches.push((
                score,
                FileSearchItem {
                    name: name.clone(),
                    path: path.to_string_lossy().to_string(),
                    root_label: root.label.to_string(),
                    kind: if file_type.is_dir() {
                        "directory".to_string()
                    } else {
                        "file".to_string()
                    },
                    extension: if file_type.is_file() {
                        path.extension()
                            .map(|value| value.to_string_lossy().to_string())
                            .filter(|value| !value.is_empty())
                    } else {
                        None
                    },
                    size_bytes: metadata
                        .as_ref()
                        .filter(|_| file_type.is_file())
                        .map(|metadata| metadata.len()),
                    modified_at_ms,
                },
            ));
        }

        if file_type.is_dir() && depth < MAX_SEARCH_DEPTH {
            scan_directory(
                &path,
                root,
                depth + 1,
                normalized_query,
                tokens,
                state,
            );

            if state.scan_limit_reached {
                return;
            }
        }
    }
}

fn match_score(name: &str, query: &str, tokens: &[&str]) -> Option<u16> {
    if name == query {
        return Some(100);
    }
    if name.starts_with(query) {
        return Some(85);
    }
    if name.contains(query) {
        return Some(70);
    }
    if !tokens.is_empty() && tokens.iter().all(|token| name.contains(token)) {
        return Some(55);
    }
    None
}

fn sanitize_query(query: &str) -> Result<String, String> {
    let query = query.trim();
    let count = query.chars().count();
    if count < 2 {
        return Err("File search needs at least 2 characters.".to_string());
    }
    if count > MAX_QUERY_CHARS {
        return Err(format!(
            "File search is limited to {MAX_QUERY_CHARS} characters."
        ));
    }
    if query.chars().any(char::is_control) {
        return Err("File search cannot contain control characters.".to_string());
    }

    Ok(query.to_string())
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
    fn exact_name_scores_above_partial_match() {
        assert!(match_score("report.pdf", "report.pdf", &["report.pdf"])
            > match_score("report.pdf", "report", &["report"]));
    }

    #[test]
    fn token_matching_supports_multi_word_queries() {
        assert_eq!(
            match_score(
                "world united final poster.png",
                "world poster",
                &["world", "poster"],
            ),
            Some(55)
        );
    }

    #[test]
    fn unrelated_names_do_not_match() {
        assert_eq!(
            match_score("invoice.pdf", "world united", &["world", "united"]),
            None
        );
    }

    #[test]
    fn search_summary_includes_safe_reveal_hint() {
        let snapshot = FileSearchSnapshot {
            query: "report".to_string(),
            items: vec![FileSearchItem {
                name: "report.pdf".to_string(),
                path: "C:\\Users\\Test\\Documents\\report.pdf".to_string(),
                root_label: "Documents".to_string(),
                kind: "file".to_string(),
                extension: Some("pdf".to_string()),
                size_bytes: Some(10),
                modified_at_ms: 1,
            }],
            roots: vec!["Documents".to_string()],
            scanned_entries: 1,
            scan_limit_reached: false,
            max_depth: MAX_SEARCH_DEPTH,
            refreshed_at_ms: 1,
        };

        assert!(summarize_file_search(&snapshot).contains("Reveal file <full path>"));
    }

    #[test]
    fn file_search_query_is_bounded() {
        assert!(sanitize_query("a").is_err());
        assert!(sanitize_query("report").is_ok());
        assert!(sanitize_query(&"x".repeat(MAX_QUERY_CHARS + 1)).is_err());
    }
}
