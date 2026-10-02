use std::mem::size_of;

use windows_sys::Win32::{
    UI::{
        Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEINPUT,
            MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTDOWN,
            MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL,
        },
        WindowsAndMessaging::{
            GetSystemMetrics, SetCursorPos, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
            SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
        },
    },
};

const WHEEL_DELTA: i32 = 120;
const MAX_SCROLL_NOTCHES: i32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    MoveTo(ScreenPoint),
    Scroll { notches: i32 },
    Click { button: MouseButton },
    DoubleClick,
    ClickAt { point: ScreenPoint, button: MouseButton },
}

#[derive(Debug)]
pub enum MouseError {
    InvalidCoordinates,
    OutsideVirtualDesktop {
        x: i32,
        y: i32,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    },
    InvalidScrollAmount,
    MoveFailed,
    InjectionFailed { expected: u32, sent: u32 },
}

impl std::fmt::Display for MouseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCoordinates => {
                write!(formatter, "Mouse coordinates must contain two whole numbers.")
            }
            Self::OutsideVirtualDesktop {
                x,
                y,
                left,
                top,
                right,
                bottom,
            } => write!(
                formatter,
                "Point ({x}, {y}) is outside the virtual desktop bounds ({left}, {top})–({right}, {bottom})."
            ),
            Self::InvalidScrollAmount => write!(
                formatter,
                "Scroll amount must be between 1 and {MAX_SCROLL_NOTCHES} notches."
            ),
            Self::MoveFailed => write!(formatter, "Windows could not move the mouse pointer."),
            Self::InjectionFailed { expected, sent } => write!(
                formatter,
                "Windows accepted {sent} of {expected} mouse input events."
            ),
        }
    }
}

pub fn parse_point(value: &str) -> Result<ScreenPoint, MouseError> {
    let cleaned = value
        .trim()
        .trim_matches(|c| matches!(c, '(' | ')' | '[' | ']'));

    let parts = cleaned
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.len() != 2 {
        return Err(MouseError::InvalidCoordinates);
    }

    let x = parts[0]
        .parse::<i32>()
        .map_err(|_| MouseError::InvalidCoordinates)?;
    let y = parts[1]
        .parse::<i32>()
        .map_err(|_| MouseError::InvalidCoordinates)?;

    Ok(ScreenPoint { x, y })
}

pub fn validate_scroll_notches(notches: i32) -> Result<i32, MouseError> {
    let magnitude = notches.abs();
    if magnitude == 0 || magnitude > MAX_SCROLL_NOTCHES {
        return Err(MouseError::InvalidScrollAmount);
    }

    Ok(notches)
}

fn virtual_desktop_bounds() -> (i32, i32, i32, i32) {
    unsafe {
        let left = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let top = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        (left, top, left + width - 1, top + height - 1)
    }
}

fn validate_point(point: ScreenPoint) -> Result<(), MouseError> {
    let (left, top, right, bottom) = virtual_desktop_bounds();

    if point.x < left || point.x > right || point.y < top || point.y > bottom {
        return Err(MouseError::OutsideVirtualDesktop {
            x: point.x,
            y: point.y,
            left,
            top,
            right,
            bottom,
        });
    }

    Ok(())
}

pub fn move_to(point: ScreenPoint) -> Result<(), MouseError> {
    validate_point(point)?;

    let moved = unsafe { SetCursorPos(point.x, point.y) };
    if moved == 0 {
        Err(MouseError::MoveFailed)
    } else {
        Ok(())
    }
}

fn mouse_input(flags: u32, mouse_data: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: mouse_data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send_inputs(inputs: &[INPUT]) -> Result<(), MouseError> {
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
        Err(MouseError::InjectionFailed { expected, sent })
    }
}

pub fn click(button: MouseButton) -> Result<(), MouseError> {
    let (down, up) = match button {
        MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
    };

    send_inputs(&[
        mouse_input(down, 0),
        mouse_input(up, 0),
    ])
}

pub fn double_click() -> Result<(), MouseError> {
    send_inputs(&[
        mouse_input(MOUSEEVENTF_LEFTDOWN, 0),
        mouse_input(MOUSEEVENTF_LEFTUP, 0),
        mouse_input(MOUSEEVENTF_LEFTDOWN, 0),
        mouse_input(MOUSEEVENTF_LEFTUP, 0),
    ])
}

pub fn click_at(point: ScreenPoint, button: MouseButton) -> Result<(), MouseError> {
    move_to(point)?;
    click(button)
}

pub fn scroll(notches: i32) -> Result<(), MouseError> {
    let notches = validate_scroll_notches(notches)?;
    let delta = notches
        .checked_mul(WHEEL_DELTA)
        .ok_or(MouseError::InvalidScrollAmount)?;

    send_inputs(&[mouse_input(MOUSEEVENTF_WHEEL, delta as u32)])
}

pub fn execute_mouse_action(action: MouseAction) -> Result<(), MouseError> {
    match action {
        MouseAction::MoveTo(point) => move_to(point),
        MouseAction::Scroll { notches } => scroll(notches),
        MouseAction::Click { button } => click(button),
        MouseAction::DoubleClick => double_click(),
        MouseAction::ClickAt { point, button } => click_at(point, button),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comma_coordinates() {
        assert_eq!(
            parse_point("500, 300").unwrap(),
            ScreenPoint { x: 500, y: 300 }
        );
    }

    #[test]
    fn parses_space_coordinates() {
        assert_eq!(
            parse_point("-120 640").unwrap(),
            ScreenPoint { x: -120, y: 640 }
        );
    }

    #[test]
    fn rejects_invalid_coordinates() {
        assert!(parse_point("500").is_err());
        assert!(parse_point("x, y").is_err());
    }

    #[test]
    fn limits_scroll_amount() {
        assert_eq!(validate_scroll_notches(-3).unwrap(), -3);
        assert!(validate_scroll_notches(0).is_err());
        assert!(validate_scroll_notches(21).is_err());
    }
}
