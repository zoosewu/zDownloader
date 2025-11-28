use regex::Regex;
use serde::Serialize;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::time;

#[derive(Serialize, Clone)]
struct ClipboardDetectedPayload {
    url: String,
}

/// URL validation regex pattern
/// Matches HTTP/HTTPS URLs
fn is_valid_url(text: &str) -> bool {
    let url_regex = Regex::new(
        r"^https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&/=]*)$"
    ).unwrap();
    url_regex.is_match(text)
}

/// Clipboard monitor that polls clipboard every 200ms
/// Emits "clipboard-detected" event when a valid URL is detected
pub async fn start_clipboard_monitor(app_handle: AppHandle) {
    let mut last_clipboard_content = String::new();
    let mut interval = time::interval(Duration::from_millis(200));

    loop {
        interval.tick().await;

        // Try to read clipboard content using Tauri plugin
        if let Ok(content) = read_clipboard(&app_handle).await {
            // Skip if content hasn't changed
            if content == last_clipboard_content {
                continue;
            }

            // Validate if content is a URL
            let trimmed = content.trim();
            if is_valid_url(trimmed) {
                // Emit clipboard-detected event with proper payload structure
                let payload = ClipboardDetectedPayload {
                    url: trimmed.to_string(),
                };
                if let Err(e) = app_handle.emit("clipboard-detected", payload) {
                    eprintln!("Failed to emit clipboard-detected event: {}", e);
                }
            }

            last_clipboard_content = content;
        }
    }
}

/// Read clipboard text using tauri-plugin-clipboard-manager
async fn read_clipboard(app_handle: &AppHandle) -> Result<String, String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;

    // Use clipboard manager plugin
    match app_handle.clipboard().read_text() {
        Ok(text) => Ok(text),
        Err(e) => Err(format!("Failed to read clipboard: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_url() {
        // Valid URLs
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://example.com/path/to/file.zip"));
        assert!(is_valid_url("https://www.example.com/download?id=123"));

        // Invalid URLs
        assert!(!is_valid_url("not a url"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url(""));
        assert!(!is_valid_url("example.com")); // Missing protocol
    }
}
