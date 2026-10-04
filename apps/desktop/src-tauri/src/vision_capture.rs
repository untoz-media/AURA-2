use image::RgbaImage;
use serde::Serialize;
use std::{
    fs,
    mem::{size_of, zeroed},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};
use windows_sys::Win32::{
    Foundation::{HWND, POINT, RECT},
    Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
        GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT,
        DIB_RGB_COLORS, SRCCOPY,
    },
    UI::WindowsAndMessaging::{
        GetCursorPos, GetForegroundWindow, GetSystemMetrics, GetWindowRect, GetWindowTextLengthW,
        GetWindowTextW, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    },
};

const MAX_CAPTURE_PIXELS: i64 = 24_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionCapture {
    pub id: String,
    pub kind: String,
    pub path: String,
    pub rect: CaptureRect,
    pub window_title: Option<String>,
    pub created_at_ms: u64,
}

pub fn cursor_position() -> Result<(i32, i32), String> {
    let mut point: POINT = unsafe { zeroed() };
    let ok = unsafe { GetCursorPos(&mut point) };
    if ok == 0 {
        Err("Windows could not read the cursor position.".to_string())
    } else {
        Ok((point.x, point.y))
    }
}

pub fn virtual_desktop_rect() -> CaptureRect {
    unsafe {
        CaptureRect {
            x: GetSystemMetrics(SM_XVIRTUALSCREEN),
            y: GetSystemMetrics(SM_YVIRTUALSCREEN),
            width: GetSystemMetrics(SM_CXVIRTUALSCREEN),
            height: GetSystemMetrics(SM_CYVIRTUALSCREEN),
        }
    }
}

pub fn active_window_rect() -> Result<(CaptureRect, Option<String>), String> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        return Err("No active Windows window is available to capture.".to_string());
    }

    let mut rect: RECT = unsafe { zeroed() };
    if unsafe { GetWindowRect(hwnd, &mut rect) } == 0 {
        return Err("Windows could not read the active window bounds.".to_string());
    }

    let width = rect.right.saturating_sub(rect.left);
    let height = rect.bottom.saturating_sub(rect.top);
    let capture_rect = CaptureRect {
        x: rect.left,
        y: rect.top,
        width,
        height,
    };

    let title_length = unsafe { GetWindowTextLengthW(hwnd) };
    let title = if title_length > 0 {
        let mut buffer = vec![0_u16; title_length as usize + 1];
        let read = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
        if read > 0 {
            Some(String::from_utf16_lossy(&buffer[..read as usize]))
        } else {
            None
        }
    } else {
        None
    };

    Ok((validate_rect(capture_rect)?, title))
}

pub fn capture_full_screen(app: &AppHandle) -> Result<VisionCapture, String> {
    capture_rect(
        app,
        validate_rect(virtual_desktop_rect())?,
        "screen",
        None,
    )
}

pub fn capture_active_window(app: &AppHandle) -> Result<VisionCapture, String> {
    let (rect, title) = active_window_rect()?;
    capture_rect(app, rect, "activeWindow", title)
}

pub fn capture_window_handle(
    app: &AppHandle,
    handle: isize,
    title: Option<String>,
) -> Result<VisionCapture, String> {
    let hwnd = handle as HWND;
    if hwnd.is_null() {
        return Err("The requested Windows window handle is invalid.".to_string());
    }

    let mut rect: RECT = unsafe { zeroed() };
    if unsafe { GetWindowRect(hwnd, &mut rect) } == 0 {
        return Err("Windows could not read the requested window bounds.".to_string());
    }

    capture_rect(
        app,
        validate_rect(CaptureRect {
            x: rect.left,
            y: rect.top,
            width: rect.right.saturating_sub(rect.left),
            height: rect.bottom.saturating_sub(rect.top),
        })?,
        "activeWindow",
        title,
    )
}

pub fn capture_region(
    app: &AppHandle,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<VisionCapture, String> {
    capture_rect(
        app,
        validate_rect(CaptureRect {
            x,
            y,
            width,
            height,
        })?,
        "region",
        None,
    )
}

pub fn region_from_points(
    first: (i32, i32),
    second: (i32, i32),
) -> Result<CaptureRect, String> {
    let left = first.0.min(second.0);
    let top = first.1.min(second.1);
    let right = first.0.max(second.0);
    let bottom = first.1.max(second.1);
    let rect = CaptureRect {
        x: left,
        y: top,
        width: right.saturating_sub(left),
        height: bottom.saturating_sub(top),
    };

    validate_dimensions(rect)
}

pub fn remove_capture(path: &str) {
    let _ = fs::remove_file(path);
}

fn capture_rect(
    app: &AppHandle,
    rect: CaptureRect,
    kind: &str,
    window_title: Option<String>,
) -> Result<VisionCapture, String> {
    let rect = validate_rect(rect)?;
    let output = capture_path(app)?;

    let screen_dc = unsafe { GetDC(std::ptr::null_mut()) };
    if screen_dc.is_null() {
        return Err("Windows could not open the desktop device context.".to_string());
    }

    let memory_dc = unsafe { CreateCompatibleDC(screen_dc) };
    if memory_dc.is_null() {
        unsafe {
            ReleaseDC(std::ptr::null_mut(), screen_dc);
        }
        return Err("Windows could not create a compatible capture context.".to_string());
    }

    let bitmap = unsafe { CreateCompatibleBitmap(screen_dc, rect.width, rect.height) };
    if bitmap.is_null() {
        unsafe {
            DeleteDC(memory_dc);
            ReleaseDC(std::ptr::null_mut(), screen_dc);
        }
        return Err("Windows could not allocate the screenshot bitmap.".to_string());
    }

    let old_object = unsafe { SelectObject(memory_dc, bitmap) };
    let copied = unsafe {
        BitBlt(
            memory_dc,
            0,
            0,
            rect.width,
            rect.height,
            screen_dc,
            rect.x,
            rect.y,
            SRCCOPY | CAPTUREBLT,
        )
    };

    if copied == 0 {
        unsafe {
            SelectObject(memory_dc, old_object);
            DeleteObject(bitmap);
            DeleteDC(memory_dc);
            ReleaseDC(std::ptr::null_mut(), screen_dc);
        }
        return Err("Windows could not copy pixels from the selected screen region.".to_string());
    }

    let mut info: BITMAPINFO = unsafe { zeroed() };
    info.bmiHeader = BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: rect.width,
        biHeight: -rect.height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        ..unsafe { zeroed() }
    };

    let byte_len = rect.width as usize * rect.height as usize * 4;
    let mut pixels = vec![0_u8; byte_len];
    let scan_lines = unsafe {
        GetDIBits(
            memory_dc,
            bitmap,
            0,
            rect.height as u32,
            pixels.as_mut_ptr().cast(),
            &mut info,
            DIB_RGB_COLORS,
        )
    };

    unsafe {
        SelectObject(memory_dc, old_object);
        DeleteObject(bitmap);
        DeleteDC(memory_dc);
        ReleaseDC(std::ptr::null_mut(), screen_dc);
    }

    if scan_lines == 0 {
        return Err("Windows could not read screenshot pixels.".to_string());
    }

    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
        pixel[3] = 255;
    }

    let image = RgbaImage::from_raw(rect.width as u32, rect.height as u32, pixels)
        .ok_or_else(|| "Screenshot pixel buffer is invalid.".to_string())?;

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create Vision capture cache: {error}"))?;
    }

    image
        .save(&output)
        .map_err(|error| format!("Could not encode screenshot PNG: {error}"))?;

    let created_at_ms = timestamp_ms();
    let id = output
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("vision-capture")
        .to_string();

    Ok(VisionCapture {
        id,
        kind: kind.to_string(),
        path: output.to_string_lossy().to_string(),
        rect,
        window_title,
        created_at_ms,
    })
}

fn validate_dimensions(rect: CaptureRect) -> Result<CaptureRect, String> {
    if rect.width < 8 || rect.height < 8 {
        return Err("Vision capture region must be at least 8×8 pixels.".to_string());
    }

    let pixels = i64::from(rect.width) * i64::from(rect.height);
    if pixels <= 0 || pixels > MAX_CAPTURE_PIXELS {
        return Err("Vision capture region is too large.".to_string());
    }

    Ok(rect)
}

fn validate_rect(rect: CaptureRect) -> Result<CaptureRect, String> {
    let rect = validate_dimensions(rect)?;
    let desktop = virtual_desktop_rect();
    let right = rect.x.saturating_add(rect.width);
    let bottom = rect.y.saturating_add(rect.height);
    let desktop_right = desktop.x.saturating_add(desktop.width);
    let desktop_bottom = desktop.y.saturating_add(desktop.height);

    if rect.x < desktop.x
        || rect.y < desktop.y
        || right > desktop_right
        || bottom > desktop_bottom
    {
        return Err("Vision capture region is outside the Windows virtual desktop.".to_string());
    }

    Ok(rect)
}

fn capture_path(app: &AppHandle) -> Result<PathBuf, String> {
    let root = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("vision-captures");
    Ok(root.join(format!("vision-{}.png", timestamp_ms())))
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
    fn region_from_points_normalizes_drag_direction() {
        assert_eq!(
            region_from_points((800, 600), (100, 200)).unwrap(),
            CaptureRect {
                x: 100,
                y: 200,
                width: 700,
                height: 400,
            }
        );
    }
}
