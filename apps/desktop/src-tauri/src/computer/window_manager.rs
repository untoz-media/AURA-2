use serde::Serialize;
use std::{path::Path, sync::Mutex, time::{SystemTime, UNIX_EPOCH}};

use windows_sys::Win32::{
    Foundation::{CloseHandle, HWND, LPARAM},
    System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::WindowsAndMessaging::{
        EnumWindows, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindow,
        SW_RESTORE,
    },
};

use super::app_launcher::AppTarget;

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub handle: isize,
    pub title: String,
    pub process_name: Option<String>,
}


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentAppInfo {
    pub app_name: String,
    pub process_name: String,
    pub process_id: u32,
    pub known_app: bool,
    pub context_source: String,
    pub captured_at_ms: u64,
}

#[derive(Default)]
pub struct CurrentAppAwareness {
    last_external: Mutex<Option<CurrentAppInfo>>,
}

impl CurrentAppAwareness {
    pub fn snapshot(&self) -> Result<CurrentAppInfo, WindowError> {
        let foreground = current_app()?;

        if foreground.process_id != std::process::id() {
            let mut last_external = self
                .last_external
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *last_external = Some(foreground.clone());
            return Ok(foreground);
        }

        let last_external = self
            .last_external
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(previous) = last_external.as_ref() {
            let mut context = previous.clone();
            context.context_source = "lastExternal".to_string();
            return Ok(context);
        }

        Ok(foreground)
    }
}

#[derive(Debug)]
pub enum WindowError {
    EnumerationFailed,
    TargetNotFound(&'static str),
    ForegroundDenied(&'static str),
    NoForegroundWindow,
    ProcessUnavailable,
}

impl std::fmt::Display for WindowError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnumerationFailed => write!(formatter, "Windows could not enumerate top-level windows."),
            Self::TargetNotFound(name) => write!(formatter, "No visible {name} window was found."),
            Self::ForegroundDenied(name) => write!(
                formatter,
                "Windows did not allow AURA to bring {name} to the foreground."
            ),
            Self::NoForegroundWindow => write!(formatter, "Windows does not currently report a foreground application."),
            Self::ProcessUnavailable => write!(formatter, "The foreground application's process information is unavailable."),
        }
    }
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> i32 {
    if IsWindowVisible(hwnd) == 0 {
        return 1;
    }

    let length = GetWindowTextLengthW(hwnd);
    if length <= 0 {
        return 1;
    }

    let mut buffer = vec![0u16; (length + 1) as usize];
    let copied = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);

    if copied <= 0 {
        return 1;
    }

    let title = String::from_utf16_lossy(&buffer[..copied as usize]).trim().to_string();
    if title.is_empty() {
        return 1;
    }

    let process_name = process_name_for_window(hwnd);

    let windows = &mut *(lparam as *mut Vec<WindowInfo>);
    windows.push(WindowInfo {
        handle: hwnd as isize,
        title,
        process_name,
    });

    1
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn friendly_process_name(process_name: &str) -> String {
    let stem = Path::new(process_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(process_name)
        .trim();

    if stem.is_empty() {
        return process_name.to_string();
    }

    stem.replace('-', " ").replace('_', " ")
}

pub fn current_app() -> Result<CurrentAppInfo, WindowError> {
    let hwnd = unsafe { GetForegroundWindow() };

    if hwnd.is_null() {
        return Err(WindowError::NoForegroundWindow);
    }

    let mut process_id = 0_u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut process_id);
    }

    if process_id == 0 {
        return Err(WindowError::ProcessUnavailable);
    }

    let process_name =
        unsafe { process_name_for_window(hwnd) }.ok_or(WindowError::ProcessUnavailable)?;
    let known = AppTarget::from_process_image(&process_name);

    Ok(CurrentAppInfo {
        app_name: known
            .map(AppTarget::display_name)
            .map(str::to_string)
            .unwrap_or_else(|| friendly_process_name(&process_name)),
        process_name,
        process_id,
        known_app: known.is_some(),
        context_source: "foreground".to_string(),
        captured_at_ms: timestamp_ms(),
    })
}

unsafe fn process_name_for_window(hwnd: HWND) -> Option<String> {
    let mut process_id = 0u32;
    GetWindowThreadProcessId(hwnd, &mut process_id);

    if process_id == 0 {
        return None;
    }

    let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
    if process.is_null() {
        return None;
    }

    let mut buffer = vec![0u16; 32768];
    let mut size = buffer.len() as u32;

    let result = QueryFullProcessImageNameW(
        process,
        0,
        buffer.as_mut_ptr(),
        &mut size,
    );

    let _ = CloseHandle(process);

    if result == 0 || size == 0 {
        return None;
    }

    let full_path = String::from_utf16_lossy(&buffer[..size as usize]);
    Path::new(&full_path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
}

pub fn list_windows() -> Result<Vec<WindowInfo>, WindowError> {
    let mut windows = Vec::<WindowInfo>::new();

    let result = unsafe {
        EnumWindows(
            Some(enum_windows_callback),
            &mut windows as *mut Vec<WindowInfo> as LPARAM,
        )
    };

    if result == 0 {
        return Err(WindowError::EnumerationFailed);
    }

    windows.sort_by(|left, right| {
        left.title
            .to_lowercase()
            .cmp(&right.title.to_lowercase())
    });

    Ok(windows)
}

fn window_matches_target(window: &WindowInfo, target: AppTarget) -> bool {
    if let Some(process_name) = &window.process_name {
        if target
            .process_images()
            .iter()
            .any(|candidate| process_name.eq_ignore_ascii_case(candidate))
        {
            return true;
        }
    }

    let title = window.title.to_lowercase();
    target
        .window_title_hints()
        .iter()
        .any(|hint| title.contains(hint))
}

pub fn switch_to_app(target: AppTarget) -> Result<WindowInfo, WindowError> {
    let windows = list_windows()?;

    let Some(window) = windows
        .into_iter()
        .find(|window| window_matches_target(window, target))
    else {
        return Err(WindowError::TargetNotFound(target.display_name()));
    };

    let hwnd = window.handle as HWND;

    unsafe {
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        }

        if SetForegroundWindow(hwnd) == 0 {
            return Err(WindowError::ForegroundDenied(target.display_name()));
        }
    }

    Ok(window)
}

pub fn summarize_windows(limit: usize) -> Result<String, WindowError> {
    let windows = list_windows()?;

    if windows.is_empty() {
        return Ok("No visible application windows were found.".to_string());
    }

    let total = windows.len();
    let shown = windows
        .iter()
        .take(limit)
        .map(|window| window.title.as_str())
        .collect::<Vec<_>>()
        .join(" · ");

    if total > limit {
        Ok(format!(
            "{total} visible windows. Showing the first {limit}: {shown}"
        ))
    } else {
        Ok(format!("{total} visible windows: {shown}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_matching_is_case_insensitive() {
        let window = WindowInfo {
            handle: 0,
            title: "OBS 32.0".to_string(),
            process_name: Some("OBS64.EXE".to_string()),
        };

        assert!(window_matches_target(&window, AppTarget::ObsStudio));
    }

    #[test]
    fn current_app_context_source_defaults_to_foreground_shape() {
        let info = CurrentAppInfo {
            app_name: "Brave".to_string(),
            process_name: "brave.exe".to_string(),
            process_id: 10,
            known_app: true,
            context_source: "foreground".to_string(),
            captured_at_ms: 1,
        };

        assert_eq!(info.context_source, "foreground");
    }

    #[test]
    fn friendly_process_name_removes_exe_and_separators() {
        assert_eq!(friendly_process_name("Adobe-Premiere_Pro.exe"), "Adobe Premiere Pro");
    }

    #[test]
    fn title_hint_can_match_when_process_is_unavailable() {
        let window = WindowInfo {
            handle: 0,
            title: "Settings - Brave".to_string(),
            process_name: None,
        };

        assert!(window_matches_target(&window, AppTarget::Brave));
    }
}
