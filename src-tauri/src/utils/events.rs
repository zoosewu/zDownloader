//! Event emission utilities for Tauri
//!
//! This module provides helpers for emitting events to the frontend

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

/// Event names used in the application
#[allow(dead_code)]
pub mod event_names {
    pub const PROGRESS_UPDATED: &str = "progress-updated";
    pub const STATUS_CHANGED: &str = "status-changed";
    pub const CLIPBOARD_DETECTED: &str = "clipboard-detected";
    pub const UNFINISHED_DOWNLOADS_DETECTED: &str = "unfinished-downloads-detected";
    pub const CAPTCHA_REQUIRED: &str = "captcha-required";
    pub const CAPTCHA_SOLVED: &str = "captcha-solved";
    pub const CAPTCHA_CANCELLED: &str = "captcha-cancelled";
}

/// CAPTCHA required event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaRequiredEvent {
    /// Unique challenge ID
    pub challenge_id: String,
    /// CAPTCHA type (e.g., "image", "recaptcha")
    pub captcha_type: String,
    /// Challenge data (URL or site key)
    pub challenge_data: String,
    /// Additional parameters
    pub params: HashMap<String, String>,
    /// Download task ID that requires CAPTCHA
    pub task_id: String,
    /// Module ID that encountered CAPTCHA
    pub module_id: String,
}

/// CAPTCHA solved event payload
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaSolvedEvent {
    /// Challenge ID that was solved
    pub challenge_id: String,
    /// Download task ID
    pub task_id: String,
}

/// CAPTCHA cancelled event payload
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaCancelledEvent {
    /// Challenge ID that was cancelled
    pub challenge_id: String,
    /// Download task ID
    pub task_id: String,
}

/// Emits a CAPTCHA required event to the frontend
///
/// # Arguments
/// * `app_handle` - The Tauri app handle
/// * `event` - The CAPTCHA required event data
#[allow(dead_code)]
pub fn emit_captcha_required(
    app_handle: &AppHandle,
    event: CaptchaRequiredEvent,
) -> Result<(), String> {
    app_handle
        .emit(event_names::CAPTCHA_REQUIRED, event)
        .map_err(|e| format!("Failed to emit captcha-required event: {}", e))
}

/// Emits a CAPTCHA solved event to the frontend
///
/// # Arguments
/// * `app_handle` - The Tauri app handle
/// * `event` - The CAPTCHA solved event data
#[allow(dead_code)]
pub fn emit_captcha_solved(
    app_handle: &AppHandle,
    event: CaptchaSolvedEvent,
) -> Result<(), String> {
    app_handle
        .emit(event_names::CAPTCHA_SOLVED, event)
        .map_err(|e| format!("Failed to emit captcha-solved event: {}", e))
}

/// Emits a CAPTCHA cancelled event to the frontend
///
/// # Arguments
/// * `app_handle` - The Tauri app handle
/// * `event` - The CAPTCHA cancelled event data
#[allow(dead_code)]
pub fn emit_captcha_cancelled(
    app_handle: &AppHandle,
    event: CaptchaCancelledEvent,
) -> Result<(), String> {
    app_handle
        .emit(event_names::CAPTCHA_CANCELLED, event)
        .map_err(|e| format!("Failed to emit captcha-cancelled event: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_captcha_required_event_serialization() {
        let event = CaptchaRequiredEvent {
            challenge_id: "challenge123".to_string(),
            captcha_type: "image".to_string(),
            challenge_data: "https://example.com/captcha.png".to_string(),
            params: HashMap::new(),
            task_id: "task1".to_string(),
            module_id: "module1".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: CaptchaRequiredEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.challenge_id, deserialized.challenge_id);
        assert_eq!(event.captcha_type, deserialized.captcha_type);
        assert_eq!(event.task_id, deserialized.task_id);
    }

    #[test]
    fn test_captcha_solved_event_serialization() {
        let event = CaptchaSolvedEvent {
            challenge_id: "challenge123".to_string(),
            task_id: "task1".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: CaptchaSolvedEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.challenge_id, deserialized.challenge_id);
        assert_eq!(event.task_id, deserialized.task_id);
    }

    #[test]
    fn test_captcha_cancelled_event_serialization() {
        let event = CaptchaCancelledEvent {
            challenge_id: "challenge123".to_string(),
            task_id: "task1".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: CaptchaCancelledEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.challenge_id, deserialized.challenge_id);
        assert_eq!(event.task_id, deserialized.task_id);
    }
}
