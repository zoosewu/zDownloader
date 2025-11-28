// Segmentation strategy for multi-threaded downloads
use crate::models::DownloadSegment;

/// Default segment size: 10MB
pub const DEFAULT_SEGMENT_SIZE: u64 = 10 * 1024 * 1024;

/// Minimum file size to enable segmented download: 100MB
pub const MIN_FILE_SIZE_FOR_SEGMENTS: u64 = 100 * 1024 * 1024;

/// Configuration for segmented downloads
#[derive(Debug, Clone)]
pub struct SegmentConfig {
    pub segment_size: u64,
    pub max_concurrent_segments: usize,
}

impl Default for SegmentConfig {
    fn default() -> Self {
        Self {
            segment_size: DEFAULT_SEGMENT_SIZE,
            max_concurrent_segments: 8, // Default: 8 concurrent segments
        }
    }
}

impl SegmentConfig {
    pub fn new(segment_size: u64, max_concurrent_segments: usize) -> Self {
        Self {
            segment_size,
            max_concurrent_segments,
        }
    }
}

/// Calculate segments for a file
/// Returns a vector of DownloadSegment representing the file split
pub fn calculate_segments(
    file_size: u64,
    config: &SegmentConfig,
) -> Vec<DownloadSegment> {
    let mut segments = Vec::new();

    // If file is too small, use single segment
    if file_size < MIN_FILE_SIZE_FOR_SEGMENTS {
        segments.push(DownloadSegment::new(0, 0, file_size - 1));
        return segments;
    }

    let segment_size = config.segment_size;
    let num_segments = (file_size + segment_size - 1) / segment_size; // Ceiling division

    for i in 0..num_segments {
        let start = i * segment_size;
        let end = std::cmp::min(start + segment_size - 1, file_size - 1);

        segments.push(DownloadSegment::new(i as usize, start, end));
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_segments_small_file() {
        let config = SegmentConfig::default();
        let segments = calculate_segments(50 * 1024 * 1024, &config); // 50MB

        // Should use single segment for files < 100MB
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start_byte, 0);
        assert_eq!(segments[0].end_byte, 50 * 1024 * 1024 - 1);
    }

    #[test]
    fn test_calculate_segments_large_file() {
        let config = SegmentConfig::default();
        let file_size = 250 * 1024 * 1024; // 250MB
        let segments = calculate_segments(file_size, &config);

        // 250MB / 10MB = 25 segments
        assert_eq!(segments.len(), 25);

        // Check first segment
        assert_eq!(segments[0].segment_id, 0);
        assert_eq!(segments[0].start_byte, 0);
        assert_eq!(segments[0].end_byte, 10 * 1024 * 1024 - 1);

        // Check last segment
        let last = segments.last().unwrap();
        assert_eq!(last.segment_id, 24);
        assert_eq!(last.end_byte, file_size - 1);
    }

    #[test]
    fn test_calculate_segments_exact_multiple() {
        let config = SegmentConfig::default();
        let file_size = 100 * 1024 * 1024; // Exactly 100MB
        let segments = calculate_segments(file_size, &config);

        // 100MB / 10MB = 10 segments
        assert_eq!(segments.len(), 10);

        // Check all segments are exactly 10MB except possibly the last
        for (i, segment) in segments.iter().enumerate() {
            let expected_size = if i == segments.len() - 1 {
                10 * 1024 * 1024 // Last segment is also exactly 10MB in this case
            } else {
                10 * 1024 * 1024
            };
            assert_eq!(segment.end_byte - segment.start_byte + 1, expected_size);
        }
    }
}
