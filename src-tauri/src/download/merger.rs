// Segment merger - combines downloaded segments into a complete file
#![allow(dead_code)]

use crate::models::DownloadSegment;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Merge all downloaded segments into a single complete file
///
/// # Arguments
/// * `segments` - Vector of all download segments (must be sorted by segment_id)
/// * `final_path` - Path to the final merged file
/// * `temp_dir` - Directory containing temporary segment files
///
/// # Returns
/// * `Ok(())` - If merge was successful
/// * `Err(String)` - If merge failed with error message
pub async fn merge_segments(
    segments: &[DownloadSegment],
    final_path: &Path,
    temp_dir: &Path,
) -> Result<(), String> {
    // Validate all segments are completed
    for segment in segments {
        if segment.status != crate::models::SegmentStatus::Completed {
            return Err(format!(
                "Segment {} is not completed (status: {:?})",
                segment.segment_id, segment.status
            ));
        }
    }

    // Sort segments by segment_id to ensure correct order
    let mut sorted_segments = segments.to_vec();
    sorted_segments.sort_by_key(|s| s.segment_id);

    // Create final file
    if let Some(parent) = final_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }

    let mut final_file =
        File::create(final_path).map_err(|e| format!("Failed to create final file: {}", e))?;

    // Merge segments sequentially
    for segment in &sorted_segments {
        let temp_file_path = temp_dir.join(format!("segment_{}", segment.segment_id));

        if !temp_file_path.exists() {
            return Err(format!(
                "Segment file not found: {}",
                temp_file_path.display()
            ));
        }

        let mut temp_file = File::open(&temp_file_path)
            .map_err(|e| format!("Failed to open segment file {}: {}", segment.segment_id, e))?;

        // Verify segment size matches expected size
        let metadata = temp_file
            .metadata()
            .map_err(|e| format!("Failed to read segment metadata: {}", e))?;
        let expected_size = segment.end_byte - segment.start_byte + 1;

        if metadata.len() != expected_size {
            return Err(format!(
                "Segment {} size mismatch: expected {}, got {}",
                segment.segment_id,
                expected_size,
                metadata.len()
            ));
        }

        // Copy segment data to final file
        std::io::copy(&mut temp_file, &mut final_file)
            .map_err(|e| format!("Failed to copy segment {}: {}", segment.segment_id, e))?;
    }

    final_file
        .flush()
        .map_err(|e| format!("Failed to flush final file: {}", e))?;

    Ok(())
}

/// Merge segments in-place by writing each segment to its correct position
/// This is more memory-efficient for very large files
///
/// # Arguments
/// * `segments` - Vector of all download segments
/// * `final_path` - Path to the final file (will be created if doesn't exist)
/// * `temp_dir` - Directory containing temporary segment files
pub async fn merge_segments_in_place(
    segments: &[DownloadSegment],
    final_path: &Path,
    temp_dir: &Path,
) -> Result<(), String> {
    // Validate all segments are completed
    for segment in segments {
        if segment.status != crate::models::SegmentStatus::Completed {
            return Err(format!("Segment {} is not completed", segment.segment_id));
        }
    }

    // Create final file with correct size
    if let Some(parent) = final_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create parent directory: {}", e))?;
    }

    let mut final_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(final_path)
        .map_err(|e| format!("Failed to open final file: {}", e))?;

    // Write each segment to its correct position
    for segment in segments {
        let temp_file_path = temp_dir.join(format!("segment_{}", segment.segment_id));

        if !temp_file_path.exists() {
            return Err(format!(
                "Segment file not found: {}",
                temp_file_path.display()
            ));
        }

        let mut temp_file = File::open(&temp_file_path)
            .map_err(|e| format!("Failed to open segment file: {}", e))?;

        // Seek to the correct position in final file
        final_file
            .seek(SeekFrom::Start(segment.start_byte))
            .map_err(|e| format!("Failed to seek to position {}: {}", segment.start_byte, e))?;

        // Read segment data
        let mut buffer = Vec::new();
        temp_file
            .read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read segment data: {}", e))?;

        // Verify size
        let expected_size = (segment.end_byte - segment.start_byte + 1) as usize;
        if buffer.len() != expected_size {
            return Err(format!(
                "Segment {} size mismatch: expected {}, got {}",
                segment.segment_id,
                expected_size,
                buffer.len()
            ));
        }

        // Write to final file
        final_file
            .write_all(&buffer)
            .map_err(|e| format!("Failed to write segment data: {}", e))?;
    }

    final_file
        .flush()
        .map_err(|e| format!("Failed to flush final file: {}", e))?;

    Ok(())
}

/// Clean up temporary segment files after successful merge
pub fn cleanup_segment_files(segments: &[DownloadSegment], temp_dir: &Path) -> Result<(), String> {
    for segment in segments {
        let temp_file_path = temp_dir.join(format!("segment_{}", segment.segment_id));

        if temp_file_path.exists() {
            std::fs::remove_file(&temp_file_path).map_err(|e| {
                format!(
                    "Failed to remove segment file {}: {}",
                    segment.segment_id, e
                )
            })?;
        }
    }

    // Try to remove temp directory if empty
    if temp_dir.exists() {
        let _ = std::fs::remove_dir(temp_dir); // Ignore error if not empty
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DownloadSegment;
    use std::io::Write;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_merge_segments() {
        let temp_dir = std::env::temp_dir().join(format!("zoo_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Create test segments
        let task_id = Uuid::new_v4();
        let mut segments = vec![
            DownloadSegment::new(0, 0, 9).with_task_id(task_id),
            DownloadSegment::new(1, 10, 19).with_task_id(task_id),
            DownloadSegment::new(2, 20, 29).with_task_id(task_id),
        ];

        // Mark all segments as completed
        for segment in &mut segments {
            segment.status = crate::models::SegmentStatus::Completed;
        }

        // Create segment files with test data
        for (i, _segment) in segments.iter().enumerate() {
            let file_path = temp_dir.join(format!("segment_{}", i));
            let mut file = File::create(&file_path).unwrap();
            let data = format!("SEG{:02}TEST!", i); // Exactly 10 bytes each
            file.write_all(data.as_bytes()).unwrap();
        }

        // Merge segments
        let final_path = temp_dir.join("merged_file.txt");
        let result = merge_segments(&segments, &final_path, &temp_dir).await;

        if let Err(e) = &result {
            eprintln!("Merge failed with error: {}", e);
        }
        assert!(result.is_ok(), "Merge failed: {:?}", result);
        assert!(final_path.exists());

        // Verify merged content
        let content = std::fs::read_to_string(&final_path).unwrap();
        assert_eq!(content, "SEG00TEST!SEG01TEST!SEG02TEST!");

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}
