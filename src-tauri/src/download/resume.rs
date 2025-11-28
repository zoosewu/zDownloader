// Resume download logic - handles partial file integrity check and resume capability
use crate::models::{DownloadSegment, DownloadTask, SegmentStatus};
use std::fs::metadata;
use std::path::Path;

/// Tolerance for file size matching (±1 byte)
const FILE_SIZE_TOLERANCE: u64 = 1;

/// Check if a partial download can be resumed
///
/// # Arguments
/// * `task` - The download task to check
/// * `temp_dir` - Directory containing partial segment files
///
/// # Returns
/// * `Ok(true)` - Resume is possible
/// * `Ok(false)` - Resume is not possible, need to restart
/// * `Err(String)` - Error occurred during check
pub fn can_resume_download(task: &DownloadTask, temp_dir: &Path) -> Result<bool, String> {
    // Check if task has segments
    if task.segments.is_empty() {
        return Ok(false);
    }

    // Check if temp directory exists
    if !temp_dir.exists() {
        return Ok(false);
    }

    // Verify each segment's partial file
    for segment in &task.segments {
        if segment.status == SegmentStatus::Completed {
            // Check if completed segment file exists and has correct size
            let segment_path = temp_dir.join(format!("segment_{}", segment.segment_id));

            if !segment_path.exists() {
                return Ok(false);
            }

            let file_meta = metadata(&segment_path)
                .map_err(|e| format!("Failed to read segment metadata: {}", e))?;

            let expected_size = segment.end_byte - segment.start_byte + 1;
            let actual_size = file_meta.len();

            // Verify size with tolerance
            if actual_size.abs_diff(expected_size) > FILE_SIZE_TOLERANCE {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Get resumable segments from a task
///
/// Returns a list of segments that need to be downloaded/resumed
pub fn get_resumable_segments(task: &DownloadTask, temp_dir: &Path) -> Vec<DownloadSegment> {
    let mut resumable = Vec::new();

    for mut segment in task.segments.clone() {
        match segment.status {
            SegmentStatus::Pending | SegmentStatus::Failed => {
                // Need to download from scratch
                resumable.push(segment);
            }
            SegmentStatus::Downloading => {
                // Check if partial file exists
                let segment_path = temp_dir.join(format!("segment_{}", segment.segment_id));

                if segment_path.exists() {
                    if let Ok(file_meta) = metadata(&segment_path) {
                        let downloaded = file_meta.len();

                        // Update downloaded bytes
                        segment.downloaded_bytes = downloaded;

                        // If some bytes downloaded but not complete, resume from this point
                        if downloaded > 0 && downloaded < segment.size() {
                            // Adjust start_byte to resume from where we left off
                            segment.start_byte += downloaded;
                            resumable.push(segment);
                        } else if downloaded == 0 {
                            // No data downloaded, start from beginning
                            resumable.push(segment);
                        }
                        // If downloaded == size, segment is actually complete, skip it
                    } else {
                        // Can't read metadata, start from scratch
                        segment.downloaded_bytes = 0;
                        resumable.push(segment);
                    }
                } else {
                    // No partial file, start from scratch
                    segment.downloaded_bytes = 0;
                    resumable.push(segment);
                }
            }
            SegmentStatus::Completed => {
                // Already completed, no need to download
            }
        }
    }

    resumable
}

/// Verify segment file integrity
///
/// Checks if the segment file has the correct size (with tolerance)
pub fn verify_segment_integrity(
    segment: &DownloadSegment,
    temp_dir: &Path,
) -> Result<bool, String> {
    let segment_path = temp_dir.join(format!("segment_{}", segment.segment_id));

    if !segment_path.exists() {
        return Ok(false);
    }

    let file_meta = metadata(&segment_path)
        .map_err(|e| format!("Failed to read segment file: {}", e))?;

    let expected_size = segment.end_byte - segment.start_byte + 1;
    let actual_size = file_meta.len();

    // Check with tolerance
    Ok(actual_size.abs_diff(expected_size) <= FILE_SIZE_TOLERANCE)
}

/// Calculate total downloaded bytes across all segments
pub fn calculate_total_downloaded(task: &DownloadTask, temp_dir: &Path) -> u64 {
    let mut total = 0u64;

    for segment in &task.segments {
        let segment_path = temp_dir.join(format!("segment_{}", segment.segment_id));

        if segment_path.exists() {
            if let Ok(file_meta) = metadata(&segment_path) {
                total += file_meta.len();
            }
        }
    }

    total
}

/// Create a checkpoint file to save download state
///
/// This allows recovery after application restart
pub fn save_checkpoint(task: &DownloadTask, checkpoint_path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(task)
        .map_err(|e| format!("Failed to serialize task: {}", e))?;

    std::fs::write(checkpoint_path, json)
        .map_err(|e| format!("Failed to write checkpoint: {}", e))?;

    Ok(())
}

/// Load a checkpoint file to resume download
pub fn load_checkpoint(checkpoint_path: &Path) -> Result<DownloadTask, String> {
    if !checkpoint_path.exists() {
        return Err("Checkpoint file not found".to_string());
    }

    let json = std::fs::read_to_string(checkpoint_path)
        .map_err(|e| format!("Failed to read checkpoint: {}", e))?;

    let task: DownloadTask = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse checkpoint: {}", e))?;

    Ok(task)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DownloadTask, DownloadStatus};
    use std::io::Write;
    use uuid::Uuid;

    #[test]
    fn test_can_resume_download_no_segments() {
        let task = DownloadTask {
            id: Uuid::new_v4(),
            url: "https://example.com/file.zip".to_string(),
            filename: "file.zip".to_string(),
            filepath: "/tmp/file.zip".to_string(),
            filesize: 1000,
            progress: 0.0,
            status: DownloadStatus::Paused,
            segments: vec![],
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
        };

        let temp_dir = Path::new("/tmp/nonexistent");
        let result = can_resume_download(&task, temp_dir);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_calculate_total_downloaded() {
        let temp_dir = std::env::temp_dir().join(format!("zoo_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Create test segment files
        let seg1_path = temp_dir.join("segment_0");
        let mut file1 = File::create(&seg1_path).unwrap();
        file1.write_all(b"0123456789").unwrap(); // 10 bytes

        let seg2_path = temp_dir.join("segment_1");
        let mut file2 = File::create(&seg2_path).unwrap();
        file2.write_all(b"abcdefghij").unwrap(); // 10 bytes

        let task = DownloadTask {
            id: Uuid::new_v4(),
            url: "https://example.com/file.zip".to_string(),
            filename: "file.zip".to_string(),
            filepath: "/tmp/file.zip".to_string(),
            filesize: 20,
            progress: 0.0,
            status: DownloadStatus::Paused,
            segments: vec![
                DownloadSegment::new(0, 0, 9),
                DownloadSegment::new(1, 10, 19),
            ],
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
        };

        let total = calculate_total_downloaded(&task, &temp_dir);
        assert_eq!(total, 20);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}
