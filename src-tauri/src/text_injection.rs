use std::process::Command;
use std::time::Duration;
use arboard::Clipboard;
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    Wayland,
    X11,
    Unknown,
}

impl DisplayServer {
    pub fn detect() -> Self {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            return DisplayServer::Wayland;
        }

        if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
            if session.to_lowercase() == "wayland" {
                return DisplayServer::Wayland;
            } else if session.to_lowercase() == "x11" {
                return DisplayServer::X11;
            }
        }

        if std::env::var("DISPLAY").is_ok() {
            return DisplayServer::X11;
        }

        DisplayServer::Unknown
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DisplayServer::Wayland => "wayland",
            DisplayServer::X11 => "x11",
            DisplayServer::Unknown => "unknown",
        }
    }
}

pub struct TextInjector;

impl TextInjector {
    /// Inserts text into the currently focused external application window
    pub async fn inject_text(text: &str, mode: &str) -> Result<(), AppError> {
        let display = DisplayServer::detect();
        log::info!("Injecting text using display server: {:?}", display);

        if mode == "clipboard_only" {
            return Self::copy_to_clipboard(text);
        }

        // Try direct typing if mode is native_preferred
        if mode == "native_preferred" {
            let typed = match display {
                DisplayServer::X11 => Self::type_x11(text),
                DisplayServer::Wayland => Self::type_wayland(text),
                DisplayServer::Unknown => false,
            };

            if typed {
                return Ok(());
            }
        }

        // Standard / Auto method: Clipboard Paste Injection
        Self::paste_via_clipboard(text, display).await
    }

    /// Sets text to clipboard, simulates Ctrl+V paste event, then restores previous clipboard
    async fn paste_via_clipboard(text: &str, display: DisplayServer) -> Result<(), AppError> {
        // 1. Read existing clipboard content (so we can restore it)
        let previous_clipboard = {
            if let Ok(mut cb) = Clipboard::new() {
                cb.get_text().ok()
            } else {
                None
            }
        };

        // 2. Set new text to clipboard
        Self::copy_to_clipboard(text)?;

        // Short delay to allow clipboard daemon/X11 selection to settle
        tokio::time::sleep(Duration::from_millis(50)).await;

        // 3. Simulate Ctrl+V paste event
        let paste_success = match display {
            DisplayServer::X11 => Self::simulate_paste_x11(),
            DisplayServer::Wayland => Self::simulate_paste_wayland(),
            DisplayServer::Unknown => {
                Self::simulate_paste_x11() || Self::simulate_paste_wayland()
            }
        };

        // 4. Restore original clipboard asynchronously after 750ms
        if let Some(old_text) = previous_clipboard {
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(750)).await;
                if let Ok(mut cb) = Clipboard::new() {
                    let _ = cb.set_text(old_text);
                }
            });
        }

        if !paste_success {
            log::warn!(
                "Could not automatically simulate paste keystroke. Text remains in clipboard ready for manual Ctrl+V."
            );
        }

        Ok(())
    }

    pub fn copy_to_clipboard(text: &str) -> Result<(), AppError> {
        // Try arboard first
        if let Ok(mut clipboard) = Clipboard::new() {
            if clipboard.set_text(text).is_ok() {
                return Ok(());
            }
        }

        // Fallback to wl-copy on Wayland
        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(text.as_bytes());
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return Ok(());
                }
            }
        }

        // Fallback to xclip on X11
        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(text.as_bytes());
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return Ok(());
                }
            }
        }

        Err(AppError::TextInjection(
            "Failed to copy text to system clipboard".to_string(),
        ))
    }

    fn simulate_paste_x11() -> bool {
        // 1. Try xdotool
        if let Ok(status) = Command::new("xdotool")
            .args(["key", "--clearmodifiers", "ctrl+v"])
            .status()
        {
            if status.success() {
                return true;
            }
        }

        false
    }

    fn simulate_paste_wayland() -> bool {
        // 1. Try wtype
        if let Ok(status) = Command::new("wtype")
            .args(["-M", "ctrl", "-k", "v", "-m", "ctrl"])
            .status()
        {
            if status.success() {
                return true;
            }
        }

        // 2. Try ydotool
        if let Ok(status) = Command::new("ydotool")
            .args(["key", "29:1", "47:1", "47:0", "29:0"])
            .status()
        {
            if status.success() {
                return true;
            }
        }

        false
    }

    fn type_x11(text: &str) -> bool {
        if let Ok(status) = Command::new("xdotool")
            .args(["type", "--clearmodifiers", "--delay", "10", "--", text])
            .status()
        {
            return status.success();
        }
        false
    }

    fn type_wayland(text: &str) -> bool {
        if let Ok(status) = Command::new("wtype")
            .args(["-d", "10", "--", text])
            .status()
        {
            return status.success();
        }
        false
    }
}
