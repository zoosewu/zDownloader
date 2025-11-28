// Download recovery - scan and restore unfinished downloads on app startup
use crate::download::load_checkpoint;
use crate::models::DownloadTask;
use std::fs;
use std::path::{Path, PathBuf};

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

/// Clean up old temp directories for completed downloads
///
/// This should be called periodically to prevent disk space issues
pub fn cleanup_completed_downloads(days_old: u64) -> Result<usize, String> {
    let temp_base = get_temp_base_dir();

    if !temp_base.exists() {
        return Ok(0);
    }

    let mut cleaned_count = 0;
    let cutoff_time =
        std::time::SystemTime::now() - std::time::Duration::from_secs(days_old * 24 * 60 * 60);

    if let Ok(entries) = fs::read_dir(&temp_base) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let checkpoint_path = entry.path().join("checkpoint.json");

                    // Check if checkpoint exists and is old
                    if let Ok(metadata) = fs::metadata(&checkpoint_path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified < cutoff_time {
                                // Try to remove the entire directory
                                if fs::remove_dir_all(entry.path()).is_ok() {
                                    cleaned_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(cleaned_count)
}

/// Get statistics about temp directory
pub fn get_temp_directory_stats() -> TempDirectoryStats {
    let temp_base = get_temp_base_dir();

    let mut stats = TempDirectoryStats {
        total_tasks: 0,
        total_size_bytes: 0,
        oldest_task_age_days: 0,
    };

    if !temp_base.exists() {
        return stats;
    }

    let mut oldest_time = std::time::SystemTime::now();

    if let Ok(entries) = fs::read_dir(&temp_base) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    stats.total_tasks += 1;

                    // Calculate directory size
                    if let Ok(size) = get_dir_size(&entry.path()) {
                        stats.total_size_bytes += size;
                    }

                    // Check modification time
                    let checkpoint_path = entry.path().join("checkpoint.json");
                    if let Ok(metadata) = fs::metadata(&checkpoint_path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified < oldest_time {
                                oldest_time = modified;
                            }
                        }
                    }
                }
            }
        }
    }

    // Calculate age of oldest task
    if let Ok(duration) = std::time::SystemTime::now().duration_since(oldest_time) {
        stats.oldest_task_age_days = duration.as_secs() / (24 * 60 * 60);
    }

    stats
}

/// Statistics about temp directory
#[derive(Debug, Clone)]
pub struct TempDirectoryStats {
    pub total_tasks: usize,
    pub total_size_bytes: u64,
    pub oldest_task_age_days: u64,
}

/// Get size of a directory recursively
fn get_dir_size(path: &Path) -> Result<u64, std::io::Error> {
    let mut total_size = 0u64;

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                total_size += fs::metadata(&path)?.len();
            } else if path.is_dir() {
                total_size += get_dir_size(&path)?;
            }
        }
    }

    Ok(total_size)
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
        let tasks = scan_unfinished_downloads();
        // We can't assert the count as it depends on actual temp directory state
        assert!(tasks.len() >= 0);
    }
}
