use arboard::Clipboard;
use tracing::info;
#[cfg(target_os = "macos")]
use tracing::warn;
#[cfg(not(target_os = "linux"))]
use enigo::{Enigo, Keyboard, Settings as EnigoSettings, Key, Direction};
#[cfg(target_os = "linux")]
use std::process::Command;

use crate::error::DictationError;

/// Service for pasting transcribed text into active application
/// Uses clipboard + simulated Cmd+V (macOS), Ctrl+V (Windows/Linux)
pub struct PasteService;

impl PasteService {
    pub fn new() -> Self {
        Self
    }

    /// Paste text into currently active application
    /// Saves and restores previous clipboard contents
    pub fn paste(&self, text: &str) -> Result<(), DictationError> {
        if text.is_empty() {
            return Ok(());
        }

        let mut clipboard =
            Clipboard::new().map_err(|e| DictationError::PasteError(format!("Clipboard error: {e}")))?;

        // Save current clipboard text
        let saved_text = clipboard.get_text().ok();

        // Set new text
        clipboard
            .set_text(text)
            .map_err(|e| DictationError::PasteError(format!("Failed to set clipboard: {e}")))?;

        info!("Text copied to clipboard ({} chars)", text.len());

        // Check accessibility permission on macOS
        #[cfg(target_os = "macos")]
        {
            info!("Checking accessibility permission...");
            let trusted = crate::platform::macos::is_accessibility_trusted();
            info!("Accessibility trusted: {trusted}");
            if !trusted {
                warn!("Accessibility permission not granted — prompting user.");
                crate::platform::macos::request_accessibility_permission();
                return Err(DictationError::AccessibilityPermissionDenied);
            }
        }

        // Small delay to let previously-focused app regain focus
        info!("Waiting 50ms before paste simulation...");
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Simulate paste keystroke
        info!("Simulating paste keystroke...");
        simulate_paste()?;

        // Schedule clipboard restore
        let saved = saved_text;
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if let Some(text) = saved {
                if let Ok(mut cb) = Clipboard::new() {
                    let _ = cb.set_text(text);
                }
            }
        });

        Ok(())
    }
}

fn simulate_paste() -> Result<(), DictationError> {
    // Linux: Use xdotool directly instead of enigo (X11 key mapping issues)
    #[cfg(target_os = "linux")]
    {
        info!("Using xdotool for paste on Linux");
        // Try both Ctrl+V and Ctrl+Shift+V to handle different terminal paste shortcuts
        // Some terminals (GNOME Terminal, tilix, etc.) use Ctrl+Shift+V instead of Ctrl+V
        let _ = Command::new("xdotool")
            .arg("key")
            .arg("ctrl+v")
            .status()
            .map_err(|e| DictationError::PasteError(format!("xdotool Ctrl+V failed: {e}")))?;
        let _ = Command::new("xdotool")
            .arg("key")
            .arg("ctrl+shift+v")
            .status()
            .map_err(|e| DictationError::PasteError(format!("xdotool Ctrl+Shift+V failed: {e}")))?;
        info!("Paste keystrokes simulated (xdotool: Ctrl+V and Ctrl+Shift+V)");
        Ok(())
    }

    // macOS/Windows: Use enigo
    #[cfg(not(target_os = "linux"))]
    {
        let mut enigo = Enigo::new(&EnigoSettings::default())
            .map_err(|e| DictationError::PasteError(format!("Failed to create input simulator: {e}")))?;

        #[cfg(target_os = "macos")]
        let modifier = Key::Meta;

        #[cfg(target_os = "windows")]
        let modifier = Key::Control;

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let modifier = Key::Control;

        enigo
            .key(modifier, Direction::Press)
            .map_err(|e| DictationError::PasteError(format!("Key press failed: {e}")))?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| DictationError::PasteError(format!("Key click failed: {e}")))?;
        enigo
            .key(modifier, Direction::Release)
            .map_err(|e| DictationError::PasteError(format!("Key release failed: {e}")))?;

        info!("Paste keystroke simulated");
    }
}

