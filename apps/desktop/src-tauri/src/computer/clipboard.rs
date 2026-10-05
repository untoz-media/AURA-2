use windows::Win32::{
        Foundation::{HANDLE, HGLOBAL},
        System::{
            DataExchange::{
                CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
            },
            Memory::{
                GlobalAlloc, GlobalFree, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
            },
            Ole::CF_UNICODETEXT,
        },
    },
};

const MAX_CLIPBOARD_WRITE_CHARS: usize = 32_768;
const MAX_CLIPBOARD_READ_CHARS: usize = 4_000;

struct ClipboardGuard;

impl ClipboardGuard {
    fn open() -> Result<Self, String> {
        unsafe { OpenClipboard(None) }
            .map_err(|error| format!("Windows could not open the clipboard: {error}"))?;
        Ok(Self)
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        let _ = unsafe { CloseClipboard() };
    }
}

struct GlobalMemoryGuard {
    handle: Option<HGLOBAL>,
}

impl GlobalMemoryGuard {
    fn allocate(bytes: usize) -> Result<Self, String> {
        let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes) }
            .map_err(|error| format!("Windows could not allocate clipboard memory: {error}"))?;
        Ok(Self {
            handle: Some(handle),
        })
    }

    fn handle(&self) -> Result<HGLOBAL, String> {
        self.handle
            .ok_or_else(|| "Clipboard memory handle is no longer available.".to_string())
    }

    fn transfer_to_windows(mut self) {
        self.handle.take();
    }
}

impl Drop for GlobalMemoryGuard {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            unsafe {
                let _ = GlobalFree(handle);
            }
        }
    }
}

struct LockedGlobal {
    global: HGLOBAL,
    ptr: *const u8,
    size: usize,
}

impl LockedGlobal {
    fn lock(global: HGLOBAL) -> Result<Self, String> {
        let size = unsafe { GlobalSize(global) };
        if size == 0 {
            return Err("Clipboard text is unavailable or empty.".to_string());
        }

        let ptr = unsafe { GlobalLock(global) };
        if ptr.is_null() {
            return Err("Windows could not lock clipboard memory.".to_string());
        }

        Ok(Self {
            global,
            ptr: ptr as *const u8,
            size,
        })
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.size) }
    }
}

impl Drop for LockedGlobal {
    fn drop(&mut self) {
        let _ = unsafe { GlobalUnlock(self.global) };
    }
}

pub fn read_text() -> Result<String, String> {
    let _guard = ClipboardGuard::open()?;
    let handle = unsafe { GetClipboardData(CF_UNICODETEXT.0 as u32) }
        .map_err(|_| "The clipboard does not currently contain readable text.".to_string())?;
    let locked = LockedGlobal::lock(HGLOBAL(handle.0))?;
    let bytes = locked.as_bytes();

    let words_len = bytes.len() / std::mem::size_of::<u16>();
    if words_len == 0 {
        return Ok(String::new());
    }

    let words =
        unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u16, words_len) };
    let actual_len = words.iter().position(|&value| value == 0).unwrap_or(words_len);

    Ok(String::from_utf16_lossy(&words[..actual_len]))
}

pub fn write_text(text: &str) -> Result<usize, String> {
    let character_count = text.chars().count();
    if character_count == 0 {
        return Err("Clipboard text cannot be empty.".to_string());
    }
    if character_count > MAX_CLIPBOARD_WRITE_CHARS {
        return Err(format!(
            "Clipboard writes are limited to {MAX_CLIPBOARD_WRITE_CHARS} characters per action."
        ));
    }

    let wide = text
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    let _guard = ClipboardGuard::open()?;
    unsafe { EmptyClipboard() }
        .map_err(|error| format!("Windows could not clear the clipboard before writing: {error}"))?;

    unsafe {
        let global = GlobalMemoryGuard::allocate(
            wide.len() * std::mem::size_of::<u16>(),
        )?;
        let handle = global.handle()?;

        let ptr = GlobalLock(handle);
        if ptr.is_null() {
            return Err("Windows could not lock clipboard memory for writing.".to_string());
        }

        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr as *mut u16, wide.len());
        let _ = GlobalUnlock(handle);

        SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(handle.0)))
            .map_err(|error| format!("Windows could not write clipboard text: {error}"))?;

        // Windows owns the HGLOBAL after a successful SetClipboardData call.
        global.transfer_to_windows();
    }

    Ok(character_count)
}

pub fn clear() -> Result<(), String> {
    let _guard = ClipboardGuard::open()?;
    unsafe { EmptyClipboard() }
        .map_err(|error| format!("Windows could not clear the clipboard: {error}"))
}

pub fn summarize_text(text: &str) -> String {
    let total = text.chars().count();
    if total == 0 {
        return "Clipboard text is empty.".to_string();
    }

    let visible = text.chars().take(MAX_CLIPBOARD_READ_CHARS).collect::<String>();
    if total > MAX_CLIPBOARD_READ_CHARS {
        format!(
            "Clipboard text ({total} characters; showing the first {MAX_CLIPBOARD_READ_CHARS}):\n\n{visible}"
        )
    } else {
        format!("Clipboard text ({total} characters):\n\n{visible}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_summary_does_not_truncate_short_text() {
        assert_eq!(
            summarize_text("Hello AURA"),
            "Clipboard text (10 characters):\n\nHello AURA"
        );
    }

    #[test]
    fn clipboard_summary_truncates_large_text() {
        let text = "x".repeat(MAX_CLIPBOARD_READ_CHARS + 25);
        let summary = summarize_text(&text);
        assert!(summary.contains("showing the first 4000"));
        assert_eq!(summary.matches('x').count(), MAX_CLIPBOARD_READ_CHARS);
    }
}
