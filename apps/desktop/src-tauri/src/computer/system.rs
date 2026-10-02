use std::{mem::size_of, process::Command};

use windows_sys::Win32::{
    System::{
        Power::{GetSystemPowerStatus, SetSuspendState, SYSTEM_POWER_STATUS},
        Shutdown::LockWorkStation,
        SystemInformation::{GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX},
    },
};

use super::keyboard::{press_shortcut, KeyboardShortcut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPage {
    Home,
    Display,
    Bluetooth,
    Network,
    Sound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemAction {
    GetStatus,
    GetBattery,
    ShowDesktop,
    OpenTaskManager,
    OpenSettings(SettingsPage),
    Lock,
    Sleep,
    Restart,
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub memory_load_percent: u32,
    pub memory_used_gb: f64,
    pub memory_total_gb: f64,
    pub uptime_minutes: u64,
    pub ac_connected: Option<bool>,
    pub battery_percent: Option<u8>,
    pub charging: Option<bool>,
}

#[derive(Debug)]
pub enum SystemCommandError {
    WindowsApi(&'static str),
    SpawnFailed(String),
    Keyboard(String),
}

impl std::fmt::Display for SystemCommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WindowsApi(name) => write!(formatter, "Windows API call failed: {name}."),
            Self::SpawnFailed(message) => write!(formatter, "{message}"),
            Self::Keyboard(message) => write!(formatter, "{message}"),
        }
    }
}

fn open_program(program: &str, args: &[&str]) -> Result<(), SystemCommandError> {
    Command::new(program)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|error| SystemCommandError::SpawnFailed(error.to_string()))
}

fn settings_uri(page: SettingsPage) -> &'static str {
    match page {
        SettingsPage::Home => "ms-settings:",
        SettingsPage::Display => "ms-settings:display",
        SettingsPage::Bluetooth => "ms-settings:bluetooth",
        SettingsPage::Network => "ms-settings:network",
        SettingsPage::Sound => "ms-settings:sound",
    }
}

pub fn read_system_snapshot() -> Result<SystemSnapshot, SystemCommandError> {
    let mut memory = MEMORYSTATUSEX {
        dwLength: size_of::<MEMORYSTATUSEX>() as u32,
        dwMemoryLoad: 0,
        ullTotalPhys: 0,
        ullAvailPhys: 0,
        ullTotalPageFile: 0,
        ullAvailPageFile: 0,
        ullTotalVirtual: 0,
        ullAvailVirtual: 0,
        ullAvailExtendedVirtual: 0,
    };

    if unsafe { GlobalMemoryStatusEx(&mut memory) } == 0 {
        return Err(SystemCommandError::WindowsApi("GlobalMemoryStatusEx"));
    }

    let mut power = SYSTEM_POWER_STATUS {
        ACLineStatus: 255,
        BatteryFlag: 255,
        BatteryLifePercent: 255,
        SystemStatusFlag: 0,
        BatteryLifeTime: u32::MAX,
        BatteryFullLifeTime: u32::MAX,
    };

    let power_ok = unsafe { GetSystemPowerStatus(&mut power) } != 0;

    let total_gb = memory.ullTotalPhys as f64 / 1_073_741_824.0;
    let available_gb = memory.ullAvailPhys as f64 / 1_073_741_824.0;
    let used_gb = (total_gb - available_gb).max(0.0);

    let (ac_connected, battery_percent, charging) = if power_ok {
        (
            match power.ACLineStatus {
                0 => Some(false),
                1 => Some(true),
                _ => None,
            },
            if power.BatteryLifePercent == 255 {
                None
            } else {
                Some(power.BatteryLifePercent)
            },
            if power.BatteryFlag == 255 {
                None
            } else {
                Some((power.BatteryFlag & 8) != 0)
            },
        )
    } else {
        (None, None, None)
    };

    Ok(SystemSnapshot {
        memory_load_percent: memory.dwMemoryLoad,
        memory_used_gb: used_gb,
        memory_total_gb: total_gb,
        uptime_minutes: unsafe { GetTickCount64() } / 60_000,
        ac_connected,
        battery_percent,
        charging,
    })
}

pub fn summarize_system(snapshot: &SystemSnapshot, battery_only: bool) -> String {
    let power = match snapshot.battery_percent {
        Some(percent) => {
            let charging = if snapshot.charging == Some(true) {
                " · charging"
            } else {
                ""
            };
            format!("Battery: {percent}%{charging}")
        }
        None => match snapshot.ac_connected {
            Some(true) => "Power: AC connected · no battery reported".to_string(),
            Some(false) => "Power: battery state unavailable".to_string(),
            None => "Power: status unavailable".to_string(),
        },
    };

    if battery_only {
        return power;
    }

    let uptime_hours = snapshot.uptime_minutes / 60;
    let uptime_minutes = snapshot.uptime_minutes % 60;

    format!(
        "{} · RAM: {:.1}/{:.1} GB used ({}%) · Uptime: {}h {}m",
        power,
        snapshot.memory_used_gb,
        snapshot.memory_total_gb,
        snapshot.memory_load_percent,
        uptime_hours,
        uptime_minutes,
    )
}

pub fn execute_system_action(
    action: SystemAction,
) -> Result<Option<SystemSnapshot>, SystemCommandError> {
    match action {
        SystemAction::GetStatus | SystemAction::GetBattery => {
            read_system_snapshot().map(Some)
        }
        SystemAction::ShowDesktop => {
            let shortcut = KeyboardShortcut::parse("Win+D")
                .map_err(|error| SystemCommandError::Keyboard(error.to_string()))?;
            press_shortcut(&shortcut)
                .map_err(|error| SystemCommandError::Keyboard(error.to_string()))?;
            Ok(None)
        }
        SystemAction::OpenTaskManager => {
            open_program("taskmgr.exe", &[])?;
            Ok(None)
        }
        SystemAction::OpenSettings(page) => {
            open_program("explorer.exe", &[settings_uri(page)])?;
            Ok(None)
        }
        SystemAction::Lock => {
            if unsafe { LockWorkStation() } == 0 {
                return Err(SystemCommandError::WindowsApi("LockWorkStation"));
            }
            Ok(None)
        }
        SystemAction::Sleep => {
            if !unsafe { SetSuspendState(false, false, false) } {
                return Err(SystemCommandError::WindowsApi("SetSuspendState"));
            }
            Ok(None)
        }
        SystemAction::Restart => {
            open_program("shutdown.exe", &["/r", "/t", "0"])?;
            Ok(None)
        }
        SystemAction::Shutdown => {
            open_program("shutdown.exe", &["/s", "/t", "0"])?;
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_pages_use_fixed_uris() {
        assert_eq!(settings_uri(SettingsPage::Display), "ms-settings:display");
        assert_eq!(settings_uri(SettingsPage::Bluetooth), "ms-settings:bluetooth");
    }

    #[test]
    fn snapshot_summary_handles_desktop_without_battery() {
        let snapshot = SystemSnapshot {
            memory_load_percent: 42,
            memory_used_gb: 6.0,
            memory_total_gb: 16.0,
            uptime_minutes: 125,
            ac_connected: Some(true),
            battery_percent: None,
            charging: None,
        };

        let summary = summarize_system(&snapshot, false);
        assert!(summary.contains("no battery reported"));
        assert!(summary.contains("16.0 GB"));
        assert!(summary.contains("2h 5m"));
    }
}
