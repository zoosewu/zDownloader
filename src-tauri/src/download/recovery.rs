// Download recovery - scan and restore unfinished downloads on app startup

use crate::download::load_checkpoint;
use crate::models::DownloadTask;
use std::fs;
use std::path::PathBuf;

/// Scan temp directory for unfinished downloads
///
/// Returns a list of tasks that can be resumed
pub fn scan_unfinished_downloads() -> Vec<DownloadTask> {
    let temp_base = get_temp_base_dir();

    if !temp_base.exists() {
        return Vec::new();
    }

    let mut unfinished_tasks = Vec::new();

    // Scan all subdirectories (each is a task ID)
    if let Ok(entries) = fs::read_dir(&temp_base) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    // Check if this directory has a checkpoint file
                    let checkpoint_path = entry.path().join("checkpoint.json");

                    if checkpoint_path.exists() {
                        match load_checkpoint(&checkpoint_path) {
                            Ok(task) => {
                                // Only include tasks that are not completed
                                if !task.status.is_completed() {
                                    unfinished_tasks.push(task);
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Failed to load checkpoint from {:?}: {}",
                                    checkpoint_path, e
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    unfinished_tasks
}

/// Get the base temp directory for all downloads
pub fn get_temp_base_dir() -> PathBuf {
    std::env::temp_dir().join("zoo-downloader")
}

// Add helper method to DownloadStatus
impl crate::models::DownloadStatus {
    pub fn is_completed(&self) -> bool {
        matches!(
            self,
            crate::models::DownloadStatus::Completed | crate::models::DownloadStatus::Cancelled
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_temp_base_dir() {
        let base_dir = get_temp_base_dir();
        assert!(base_dir.ends_with("zoo-downloader"));
    }

    #[test]
    fn test_scan_empty_directory() {
        // Should not panic on non-existent directory
        let _tasks = scan_unfinished_downloads();
        // We can't assert the count as it depends on actual temp directory state
        // Just verify the function completes without panicking
    }
}
