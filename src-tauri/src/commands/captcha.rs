//! CAPTCHA-related Tauri commands
//!
//! This module provides Tauri commands for frontend CAPTCHA handling:
//! - Getting pending CAPTCHA challenges
//! - Submitting CAPTCHA solutions
//! - Cancelling CAPTCHA challenges

use crate::modules::captcha::{CaptchaManager, ChallengeEntry};
use crate::modules::interface::CaptchaSolution;
use crate::utils::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Serializable challenge info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeInfo {
    /// Unique challenge ID
    pub id: String,
    /// CAPTCHA type (e.g., "image", "recaptcha")
    pub captcha_type: String,
    /// Challenge data (URL or site key)
    pub challenge_data: String,
    /// Additional parameters
    pub params: HashMap<String, String>,
    /// Download task ID
    pub task_id: String,
    /// Module ID
    pub module_id: String,
    /// When created (seconds since epoch)
    pub created_at: u64,
}

impl From<ChallengeEntry> for ChallengeInfo {
    fn from(entry: ChallengeEntry) -> Self {
        let created_at = entry
            .created_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: entry.id,
            captcha_type: entry.challenge.captcha_type,
            challenge_data: entry.challenge.challenge_data,
            params: entry.challenge.params,
            task_id: entry.task_id,
            module_id: entry.module_id,
            created_at,
        }
    }
}

/// Gets all pending CAPTCHA challenges
///
/// # Returns
/// List of pending challenges that need user input
#[tauri::command]
pub fn get_pending_captchas(
    captcha_manager: State<Arc<Mutex<CaptchaManager>>>,
) -> AppResult<Vec<ChallengeInfo>> {
    let manager = captcha_manager
        .lock()
        .map_err(|e| AppError::new(format!("Lock error: {}", e), "LOCK_ERROR"))?;

    let pending = manager.get_pending_challenges();
    let info_list: Vec<ChallengeInfo> = pending.into_iter().map(|e| e.into()).collect();

    Ok(info_list)
}

/// Submits a solution for a CAPTCHA challenge
///
/// # Arguments
/// * `challenge_id` - The challenge ID
/// * `solution_text` - The user-provided solution
/// * `metadata` - Optional additional metadata
///
/// # Returns
/// Ok(()) if solution was accepted
#[tauri::command]
pub fn submit_captcha_solution(
    captcha_manager: State<Arc<Mutex<CaptchaManager>>>,
    challenge_id: String,
    solution_text: String,
    metadata: Option<HashMap<String, String>>,
) -> AppResult<()> {
    let manager = captcha_manager
        .lock()
        .map_err(|e| AppError::new(format!("Lock error: {}", e), "LOCK_ERROR"))?;

    let solution = CaptchaSolution {
        solution: solution_text,
        metadata: metadata.unwrap_or_default(),
    };

    manager
        .submit_solution(&challenge_id, solution)
        .map_err(|e| AppError::new(e.to_string(), "CAPTCHA_ERROR"))?;

    Ok(())
}

/// Cancels a CAPTCHA challenge
///
/// # Arguments
/// * `challenge_id` - The challenge ID to cancel
///
/// # Returns
/// Ok(()) if challenge was cancelled
#[tauri::command]
pub fn cancel_captcha(
    captcha_manager: State<Arc<Mutex<CaptchaManager>>>,
    challenge_id: String,
) -> AppResult<()> {
    let manager = captcha_manager
        .lock()
        .map_err(|e| AppError::new(format!("Lock error: {}", e), "LOCK_ERROR"))?;

    manager
        .cancel_challenge(&challenge_id)
        .map_err(|e| AppError::new(e.to_string(), "CAPTCHA_ERROR"))?;

    Ok(())
}

/// Gets the count of active CAPTCHA challenges
///
/// # Returns
/// Number of active challenges
#[tauri::command]
pub fn get_active_captcha_count(
    captcha_manager: State<Arc<Mutex<CaptchaManager>>>,
) -> AppResult<usize> {
    let manager = captcha_manager
        .lock()
        .map_err(|e| AppError::new(format!("Lock error: {}", e), "LOCK_ERROR"))?;

    Ok(manager.active_count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::captcha::CaptchaManager;
    use crate::modules::interface::CaptchaChallenge;

    fn create_test_manager() -> Arc<Mutex<CaptchaManager>> {
        Arc::new(Mutex::new(CaptchaManager::new()))
    }

    fn create_test_challenge() -> CaptchaChallenge {
        CaptchaChallenge {
            captcha_type: "image".to_string(),
            challenge_data: "https://example.com/captcha.png".to_string(),
            params: HashMap::new(),
        }
    }

    #[test]
    fn test_challenge_info_conversion() {
        let manager = create_test_manager();
        let challenge = create_test_challenge();

        let challenge_id = {
            let mgr = manager.lock().unwrap();
            mgr.create_challenge("task1".to_string(), "module1".to_string(), challenge)
                .unwrap()
        };

        let entry = {
            let mgr = manager.lock().unwrap();
            mgr.get_challenge(&challenge_id).unwrap()
        };

        let info: ChallengeInfo = entry.into();
        assert_eq!(info.id, challenge_id);
        assert_eq!(info.captcha_type, "image");
        assert_eq!(info.task_id, "task1");
        assert_eq!(info.module_id, "module1");
    }

    #[test]
    fn test_manager_operations() {
        let manager = create_test_manager();
        let challenge = create_test_challenge();

        // Create challenge
        let challenge_id = {
            let mgr = manager.lock().unwrap();
            mgr.create_challenge("task1".to_string(), "module1".to_string(), challenge)
                .unwrap()
        };

        // Check pending count
        let count = {
            let mgr = manager.lock().unwrap();
            mgr.active_count()
        };
        assert_eq!(count, 1);

        // Submit solution
        {
            let mgr = manager.lock().unwrap();
            let solution = CaptchaSolution {
                solution: "12345".to_string(),
                metadata: HashMap::new(),
            };
            mgr.submit_solution(&challenge_id, solution).unwrap();
        }

        // Should no longer be pending
        let count = {
            let mgr = manager.lock().unwrap();
            mgr.active_count()
        };
        assert_eq!(count, 0);
    }
}
