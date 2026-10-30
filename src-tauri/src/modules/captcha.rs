//! CAPTCHA challenge handling and event management
//!
//! This module provides the infrastructure for handling CAPTCHA challenges
//! in website modules. It manages the event flow between backend modules
//! and frontend UI for CAPTCHA verification.

#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// CAPTCHA challenge information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaChallenge {
    /// CAPTCHA type (e.g., "image", "recaptcha", "hcaptcha")
    pub captcha_type: String,
    /// CAPTCHA image URL or site key
    pub challenge_data: String,
    /// Additional parameters
    pub params: HashMap<String, String>,
}

/// CAPTCHA solution from user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaSolution {
    /// Solution text or token
    pub solution: String,
    /// Additional data
    pub metadata: HashMap<String, String>,
}

/// Result type for CAPTCHA operations
pub type CaptchaResult<T> = Result<T, CaptchaError>;

/// Errors that can occur during CAPTCHA handling
#[derive(Debug, Clone)]
pub enum CaptchaError {
    /// Challenge not found (expired or invalid ID)
    ChallengeNotFound(String),
    /// Challenge already solved
    AlreadySolved(String),
    /// Challenge expired
    Expired(String),
    /// Invalid solution
    InvalidSolution(String),
    /// Timeout waiting for user input
    Timeout(String),
    /// Internal error
    InternalError(String),
}

impl std::fmt::Display for CaptchaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptchaError::ChallengeNotFound(id) => {
                write!(f, "CAPTCHA challenge not found: {}", id)
            }
            CaptchaError::AlreadySolved(id) => write!(f, "CAPTCHA already solved: {}", id),
            CaptchaError::Expired(id) => write!(f, "CAPTCHA challenge expired: {}", id),
            CaptchaError::InvalidSolution(msg) => write!(f, "Invalid CAPTCHA solution: {}", msg),
            CaptchaError::Timeout(msg) => write!(f, "CAPTCHA timeout: {}", msg),
            CaptchaError::InternalError(msg) => write!(f, "CAPTCHA internal error: {}", msg),
        }
    }
}

impl std::error::Error for CaptchaError {}

/// Status of a CAPTCHA challenge
#[derive(Debug, Clone, PartialEq)]
pub enum ChallengeStatus {
    /// Waiting for user to solve
    Pending,
    /// User has submitted a solution
    Solved,
    /// Challenge has expired
    Expired,
    /// Challenge was cancelled
    Cancelled,
}

/// A CAPTCHA challenge with metadata
#[derive(Debug, Clone)]
pub struct ChallengeEntry {
    /// Unique challenge ID
    pub id: String,
    /// The challenge data
    pub challenge: CaptchaChallenge,
    /// Current status
    pub status: ChallengeStatus,
    /// Solution (if provided)
    pub solution: Option<CaptchaSolution>,
    /// When the challenge was created
    pub created_at: SystemTime,
    /// When the challenge expires
    pub expires_at: SystemTime,
    /// The download task ID associated with this challenge
    pub task_id: String,
    /// The module ID that created this challenge
    pub module_id: String,
}

/// Manager for CAPTCHA challenges
///
/// This manager handles the lifecycle of CAPTCHA challenges:
/// 1. Creating challenges when modules encounter CAPTCHAs
/// 2. Emitting events to notify the frontend
/// 3. Waiting for user solutions
/// 4. Validating and returning solutions to modules
pub struct CaptchaManager {
    /// Active challenges indexed by challenge ID
    challenges: Arc<RwLock<HashMap<String, ChallengeEntry>>>,
    /// Default timeout for CAPTCHA challenges (5 minutes)
    default_timeout: Duration,
}

impl CaptchaManager {
    /// Creates a new CAPTCHA manager
    pub fn new() -> Self {
        Self {
            challenges: Arc::new(RwLock::new(HashMap::new())),
            default_timeout: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Creates a new CAPTCHA challenge
    ///
    /// # Arguments
    /// * `task_id` - The download task ID
    /// * `module_id` - The module ID that encountered the CAPTCHA
    /// * `challenge` - The CAPTCHA challenge data
    ///
    /// # Returns
    /// The unique challenge ID
    pub fn create_challenge(
        &self,
        task_id: String,
        module_id: String,
        challenge: CaptchaChallenge,
    ) -> CaptchaResult<String> {
        let challenge_id = Uuid::new_v4().to_string();
        let now = SystemTime::now();

        let entry = ChallengeEntry {
            id: challenge_id.clone(),
            challenge,
            status: ChallengeStatus::Pending,
            solution: None,
            created_at: now,
            expires_at: now + self.default_timeout,
            task_id,
            module_id,
        };

        let mut challenges = self.challenges.write().unwrap();
        challenges.insert(challenge_id.clone(), entry);

        Ok(challenge_id)
    }

    /// Submits a solution for a CAPTCHA challenge
    ///
    /// # Arguments
    /// * `challenge_id` - The challenge ID
    /// * `solution` - The user-provided solution
    ///
    /// # Returns
    /// Ok(()) if solution was accepted
    pub fn submit_solution(
        &self,
        challenge_id: &str,
        solution: CaptchaSolution,
    ) -> CaptchaResult<()> {
        let mut challenges = self.challenges.write().unwrap();

        let entry = challenges
            .get_mut(challenge_id)
            .ok_or_else(|| CaptchaError::ChallengeNotFound(challenge_id.to_string()))?;

        // Check if expired
        if SystemTime::now() > entry.expires_at {
            entry.status = ChallengeStatus::Expired;
            return Err(CaptchaError::Expired(challenge_id.to_string()));
        }

        // Check if already solved
        if entry.status == ChallengeStatus::Solved {
            return Err(CaptchaError::AlreadySolved(challenge_id.to_string()));
        }

        // Update status and solution
        entry.status = ChallengeStatus::Solved;
        entry.solution = Some(solution);

        Ok(())
    }

    /// Retrieves a challenge by ID
    ///
    /// # Arguments
    /// * `challenge_id` - The challenge ID
    ///
    /// # Returns
    /// The challenge entry if found
    pub fn get_challenge(&self, challenge_id: &str) -> CaptchaResult<ChallengeEntry> {
        let challenges = self.challenges.read().unwrap();

        challenges
            .get(challenge_id)
            .cloned()
            .ok_or_else(|| CaptchaError::ChallengeNotFound(challenge_id.to_string()))
    }

    /// Waits for a challenge to be solved
    ///
    /// This function blocks until the user submits a solution or the challenge expires.
    ///
    /// # Arguments
    /// * `challenge_id` - The challenge ID to wait for
    /// * `timeout` - Optional custom timeout (uses default if None)
    ///
    /// # Returns
    /// The solution if provided, or an error if timeout/expired
    pub fn wait_for_solution(
        &self,
        challenge_id: &str,
        timeout: Option<Duration>,
    ) -> CaptchaResult<CaptchaSolution> {
        let timeout = timeout.unwrap_or(self.default_timeout);
        let deadline = SystemTime::now() + timeout;

        loop {
            // Check current status
            let entry = self.get_challenge(challenge_id)?;

            match entry.status {
                ChallengeStatus::Solved => {
                    return entry.solution.ok_or_else(|| {
                        CaptchaError::InternalError("Solution missing".to_string())
                    })
                }
                ChallengeStatus::Expired => {
                    return Err(CaptchaError::Expired(challenge_id.to_string()))
                }
                ChallengeStatus::Cancelled => {
                    return Err(CaptchaError::InvalidSolution("Cancelled".to_string()))
                }
                ChallengeStatus::Pending => {
                    // Check timeout
                    if SystemTime::now() > deadline {
                        // Mark as expired
                        let mut challenges = self.challenges.write().unwrap();
                        if let Some(entry) = challenges.get_mut(challenge_id) {
                            entry.status = ChallengeStatus::Expired;
                        }
                        return Err(CaptchaError::Timeout(challenge_id.to_string()));
                    }

                    // Wait a bit before checking again
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    /// Cancels a CAPTCHA challenge
    ///
    /// # Arguments
    /// * `challenge_id` - The challenge ID to cancel
    pub fn cancel_challenge(&self, challenge_id: &str) -> CaptchaResult<()> {
        let mut challenges = self.challenges.write().unwrap();

        let entry = challenges
            .get_mut(challenge_id)
            .ok_or_else(|| CaptchaError::ChallengeNotFound(challenge_id.to_string()))?;

        entry.status = ChallengeStatus::Cancelled;

        Ok(())
    }

    /// Cleans up expired challenges
    ///
    /// This should be called periodically to remove old challenges
    pub fn cleanup_expired(&self) -> usize {
        let mut challenges = self.challenges.write().unwrap();
        let now = SystemTime::now();

        let before_count = challenges.len();

        challenges.retain(|_, entry| {
            // Keep challenges that haven't expired yet or are solved
            now <= entry.expires_at || entry.status == ChallengeStatus::Solved
        });

        before_count - challenges.len()
    }

    /// Gets all pending challenges
    ///
    /// # Returns
    /// Vector of all pending challenge entries
    pub fn get_pending_challenges(&self) -> Vec<ChallengeEntry> {
        let challenges = self.challenges.read().unwrap();

        challenges
            .values()
            .filter(|entry| entry.status == ChallengeStatus::Pending)
            .cloned()
            .collect()
    }

    /// Gets the count of active challenges
    pub fn active_count(&self) -> usize {
        let challenges = self.challenges.read().unwrap();
        challenges
            .values()
            .filter(|entry| entry.status == ChallengeStatus::Pending)
            .count()
    }
}

impl Default for CaptchaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_challenge() -> CaptchaChallenge {
        CaptchaChallenge {
            captcha_type: "image".to_string(),
            challenge_data: "https://example.com/captcha.png".to_string(),
            params: HashMap::new(),
        }
    }

    fn create_test_solution() -> CaptchaSolution {
        CaptchaSolution {
            solution: "12345".to_string(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_create_challenge() {
        let manager = CaptchaManager::new();
        let challenge = create_test_challenge();

        let challenge_id = manager
            .create_challenge("task1".to_string(), "module1".to_string(), challenge)
            .unwrap();

        assert!(!challenge_id.is_empty());

        let entry = manager.get_challenge(&challenge_id).unwrap();
        assert_eq!(entry.status, ChallengeStatus::Pending);
        assert_eq!(entry.task_id, "task1");
        assert_eq!(entry.module_id, "module1");
    }

    #[test]
    fn test_submit_solution() {
        let manager = CaptchaManager::new();
        let challenge = create_test_challenge();

        let challenge_id = manager
            .create_challenge("task1".to_string(), "module1".to_string(), challenge)
            .unwrap();

        let solution = create_test_solution();
        manager.submit_solution(&challenge_id, solution).unwrap();

        let entry = manager.get_challenge(&challenge_id).unwrap();
        assert_eq!(entry.status, ChallengeStatus::Solved);
        assert!(entry.solution.is_some());
    }

    #[test]
    fn test_submit_solution_twice() {
        let manager = CaptchaManager::new();
        let challenge = create_test_challenge();

        let challenge_id = manager
            .create_challenge("task1".to_string(), "module1".to_string(), challenge)
            .unwrap();

        let solution = create_test_solution();
        manager
            .submit_solution(&challenge_id, solution.clone())
            .unwrap();

        // Trying to submit again should fail
        let result = manager.submit_solution(&challenge_id, solution);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CaptchaError::AlreadySolved(_)));
    }

    #[test]
    fn test_cancel_challenge() {
        let manager = CaptchaManager::new();
        let challenge = create_test_challenge();

        let challenge_id = manager
            .create_challenge("task1".to_string(), "module1".to_string(), challenge)
            .unwrap();

        manager.cancel_challenge(&challenge_id).unwrap();

        let entry = manager.get_challenge(&challenge_id).unwrap();
        assert_eq!(entry.status, ChallengeStatus::Cancelled);
    }

    #[test]
    fn test_get_pending_challenges() {
        let manager = CaptchaManager::new();

        // Create 3 challenges
        for i in 0..3 {
            let challenge = create_test_challenge();
            manager
                .create_challenge(
                    format!("task{}", i),
                    "module1".to_string(),
                    challenge,
                )
                .unwrap();
        }

        let pending = manager.get_pending_challenges();
        assert_eq!(pending.len(), 3);
    }

    #[test]
    fn test_active_count() {
        let manager = CaptchaManager::new();
        assert_eq!(manager.active_count(), 0);

        let challenge = create_test_challenge();
        let challenge_id = manager
            .create_challenge("task1".to_string(), "module1".to_string(), challenge)
            .unwrap();

        assert_eq!(manager.active_count(), 1);

        // Solve it
        let solution = create_test_solution();
        manager.submit_solution(&challenge_id, solution).unwrap();

        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_challenge_not_found() {
        let manager = CaptchaManager::new();

        let result = manager.get_challenge("nonexistent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CaptchaError::ChallengeNotFound(_)
        ));
    }
}
