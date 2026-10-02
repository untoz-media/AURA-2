use std::{mem::size_of, thread, time::Duration};

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END,
    VK_ESCAPE, VK_F1, VK_HOME, VK_LWIN, VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN,
    VK_RIGHT, VK_SHIFT, VK_SPACE, VK_TAB, VK_UP, VK_LEFT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierKey {
    Ctrl,
    Shift,
    Alt,
    Win,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Letter(char),
    Digit(char),
    Enter,
    Escape,
    Tab,
    Space,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Function(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardShortcut {
    pub modifiers: Vec<ModifierKey>,
    pub key: KeyCode,
}

#[derive(Debug)]
pub enum KeyboardError {
    InvalidShortcut(String),
    TextTooLong,
    UnsupportedControlCharacter,
    InjectionFailed { expected: u32, sent: u32 },
}

impl std::fmt::Display for KeyboardError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidShortcut(value) => write!(formatter, "Unsupported keyboard shortcut: {value}"),
            Self::TextTooLong => write!(formatter, "Text input is limited to 500 characters per action."),
            Self::UnsupportedControlCharacter => write!(
                formatter,
                "Typed text cannot contain control characters such as Enter or Tab."
            ),
            Self::InjectionFailed { expected, sent } => write!(
                formatter,
                "Windows accepted {sent} of {expected} keyboard input events."
            ),
        }
    }
}

impl KeyboardShortcut {
    pub fn parse(value: &str) -> Result<Self, KeyboardError> {
        let cleaned = value.trim();
        if cleaned.is_empty() {
            return Err(KeyboardError::InvalidShortcut(value.to_string()));
        }

        let parts = cleaned
            .split('+')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();

        if parts.is_empty() || parts.len() > 5 {
            return Err(KeyboardError::InvalidShortcut(value.to_string()));
        }

        let mut modifiers = Vec::new();
        let mut key = None;

        for part in parts {
            let normalized = part.to_lowercase();
            let modifier = match normalized.as_str() {
                "ctrl" | "control" => Some(ModifierKey::Ctrl),
                "shift" => Some(ModifierKey::Shift),
                "alt" => Some(ModifierKey::Alt),
                "win" | "windows" | "super" => Some(ModifierKey::Win),
                _ => None,
            };

            if let Some(modifier) = modifier {
                if key.is_some() || modifiers.contains(&modifier) {
                    return Err(KeyboardError::InvalidShortcut(value.to_string()));
                }
                modifiers.push(modifier);
                continue;
            }

            if key.is_some() {
                return Err(KeyboardError::InvalidShortcut(value.to_string()));
            }

            key = Some(parse_key(&normalized).ok_or_else(|| {
                KeyboardError::InvalidShortcut(value.to_string())
            })?);
        }

        let Some(key) = key else {
            return Err(KeyboardError::InvalidShortcut(value.to_string()));
        };

        Ok(Self { modifiers, key })
    }

    pub fn display_name(&self) -> String {
        let mut parts = self
            .modifiers
            .iter()
            .map(|modifier| match modifier {
                ModifierKey::Ctrl => "Ctrl".to_string(),
                ModifierKey::Shift => "Shift".to_string(),
                ModifierKey::Alt => "Alt".to_string(),
                ModifierKey::Win => "Win".to_string(),
            })
            .collect::<Vec<_>>();

        parts.push(match self.key {
            KeyCode::Letter(value) => value.to_ascii_uppercase().to_string(),
            KeyCode::Digit(value) => value.to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Escape => "Esc".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Space => "Space".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            KeyCode::Function(number) => format!("F{number}"),
        });

        parts.join("+")
    }

    pub fn is_navigation_only(&self) -> bool {
        if !self.modifiers.is_empty() {
            return false;
        }

        matches!(
            self.key,
            KeyCode::Escape
                | KeyCode::Tab
                | KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::Home
                | KeyCode::End
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Function(_)
        )
    }

    pub fn is_delete_action(&self) -> bool {
        matches!(self.key, KeyCode::Delete)
    }
}

fn parse_key(value: &str) -> Option<KeyCode> {
    if value.len() == 1 {
        let character = value.chars().next()?;
        if character.is_ascii_alphabetic() {
            return Some(KeyCode::Letter(character));
        }
        if character.is_ascii_digit() {
            return Some(KeyCode::Digit(character));
        }
    }

    match value {
        "enter" | "return" => Some(KeyCode::Enter),
        "esc" | "escape" => Some(KeyCode::Escape),
        "tab" => Some(KeyCode::Tab),
        "space" | "spacebar" => Some(KeyCode::Space),
        "backspace" => Some(KeyCode::Backspace),
        "delete" | "del" => Some(KeyCode::Delete),
        "up" | "arrowup" => Some(KeyCode::Up),
        "down" | "arrowdown" => Some(KeyCode::Down),
        "left" | "arrowleft" => Some(KeyCode::Left),
        "right" | "arrowright" => Some(KeyCode::Right),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" | "page up" => Some(KeyCode::PageUp),
        "pagedown" | "page down" => Some(KeyCode::PageDown),
        _ => parse_function_key(value),
    }
}

fn parse_function_key(value: &str) -> Option<KeyCode> {
    let number = value.strip_prefix('f')?.parse::<u8>().ok()?;
    (1..=12)
        .contains(&number)
        .then_some(KeyCode::Function(number))
}

fn virtual_key(key: KeyCode) -> u16 {
    match key {
        KeyCode::Letter(value) => value.to_ascii_uppercase() as u16,
        KeyCode::Digit(value) => value as u16,
        KeyCode::Enter => VK_RETURN,
        KeyCode::Escape => VK_ESCAPE,
        KeyCode::Tab => VK_TAB,
        KeyCode::Space => VK_SPACE,
        KeyCode::Backspace => VK_BACK,
        KeyCode::Delete => VK_DELETE,
        KeyCode::Up => VK_UP,
        KeyCode::Down => VK_DOWN,
        KeyCode::Left => VK_LEFT,
        KeyCode::Right => VK_RIGHT,
        KeyCode::Home => VK_HOME,
        KeyCode::End => VK_END,
        KeyCode::PageUp => VK_PRIOR,
        KeyCode::PageDown => VK_NEXT,
        KeyCode::Function(number) => VK_F1 + (number - 1) as u16,
    }
}

fn modifier_virtual_key(modifier: ModifierKey) -> u16 {
    match modifier {
        ModifierKey::Ctrl => VK_CONTROL,
        ModifierKey::Shift => VK_SHIFT,
        ModifierKey::Alt => VK_MENU,
        ModifierKey::Win => VK_LWIN,
    }
}

fn key_input(vk: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn unicode_input(unit: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: unit,
                dwFlags: KEYEVENTF_UNICODE | flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send_inputs(inputs: &[INPUT]) -> Result<(), KeyboardError> {
    let expected = inputs.len() as u32;
    let sent = unsafe {
        SendInput(
            expected,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    };

    if sent == expected {
        Ok(())
    } else {
        Err(KeyboardError::InjectionFailed { expected, sent })
    }
}

pub fn press_shortcut(shortcut: &KeyboardShortcut) -> Result<(), KeyboardError> {
    let mut inputs = Vec::with_capacity(shortcut.modifiers.len() * 2 + 2);

    for modifier in &shortcut.modifiers {
        inputs.push(key_input(modifier_virtual_key(*modifier), 0));
    }

    inputs.push(key_input(virtual_key(shortcut.key), 0));
    inputs.push(key_input(virtual_key(shortcut.key), KEYEVENTF_KEYUP));

    for modifier in shortcut.modifiers.iter().rev() {
        inputs.push(key_input(
            modifier_virtual_key(*modifier),
            KEYEVENTF_KEYUP,
        ));
    }

    send_inputs(&inputs)
}

pub fn type_text(text: &str) -> Result<(), KeyboardError> {
    if text.chars().count() > 500 {
        return Err(KeyboardError::TextTooLong);
    }

    if text.chars().any(char::is_control) {
        return Err(KeyboardError::UnsupportedControlCharacter);
    }

    let mut inputs = Vec::new();

    for unit in text.encode_utf16() {
        inputs.push(unicode_input(unit, 0));
        inputs.push(unicode_input(unit, KEYEVENTF_KEYUP));
    }

    // Keep large text injections cooperative with the active application.
    for chunk in inputs.chunks(64) {
        send_inputs(chunk)?;
        if inputs.len() > 64 {
            thread::sleep(Duration::from_millis(4));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ctrl_s() {
        let shortcut = KeyboardShortcut::parse("Ctrl+S").unwrap();
        assert_eq!(shortcut.modifiers, vec![ModifierKey::Ctrl]);
        assert_eq!(shortcut.key, KeyCode::Letter('s'));
        assert_eq!(shortcut.display_name(), "Ctrl+S");
    }

    #[test]
    fn parses_function_key() {
        let shortcut = KeyboardShortcut::parse("F11").unwrap();
        assert!(shortcut.is_navigation_only());
        assert_eq!(shortcut.key, KeyCode::Function(11));
    }

    #[test]
    fn rejects_modifier_without_key() {
        assert!(KeyboardShortcut::parse("Ctrl+Shift").is_err());
    }

    #[test]
    fn delete_is_marked_separately() {
        let shortcut = KeyboardShortcut::parse("Delete").unwrap();
        assert!(shortcut.is_delete_action());
    }
}
