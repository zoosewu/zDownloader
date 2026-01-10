// Media verification module - ffmpeg integration for validating downloaded media files
//
// This module provides functionality to verify downloaded media files using ffmpeg.
// It can detect corrupted, truncated, or incomplete media files.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use tokio::process::Command as AsyncCommand;

/// Verification result status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VerificationStatus {
    /// File is valid and playable
    Valid,
    /// File has minor issues but is playable
    Warning,
    /// File is corrupted or unplayable
    Invalid,
    /// Verification was skipped (not a media file or ffmpeg unavailable)
    Skipped,
    /// Verification failed due to an error
    Error,
}

/// Detailed verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResult {
    pub status: VerificationStatus,
    /// Human-readable message describing the result
    pub message: String,
    /// Duration of the media file in seconds (if applicable)
    pub duration_secs: Option<f64>,
    /// Format name (e.g., "mp4", "mkv", "avi")
    pub format_name: Option<String>,
    /// Video codec (if present)
    pub video_codec: Option<String>,
    /// Audio codec (if present)
    pub audio_codec: Option<String>,
    /// File size in bytes
    pub file_size: u64,
    /// Any error messages from ffmpeg
    pub error_details: Option<String>,
}

/// Verification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationSettings {
    /// Enable automatic verification after download
    pub enabled: bool,
    /// Verify only media files (determined by extension)
    pub media_only: bool,
    /// Custom ffmpeg/ffprobe path (None = use system PATH)
    pub ffprobe_path: Option<String>,
    /// Timeout in seconds for verification
    pub timeout_secs: u64,
}

impl Default for VerificationSettings {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default
            media_only: true,
            ffprobe_path: None,
            timeout_secs: 60,
        }
    }
}

/// Media file extensions that should be verified
const MEDIA_EXTENSIONS: &[&str] = &[
    // Video formats
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpeg", "mpg", "3gp", "ts", "m2ts",
    // Audio formats
    "mp3", "flac", "wav", "aac", "ogg", "wma", "m4a", "opus",
];

/// Check if a file extension indicates a media file
pub fn is_media_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| MEDIA_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Check if ffprobe is available on the system
pub fn is_ffprobe_available(custom_path: Option<&str>) -> bool {
    let ffprobe = custom_path.unwrap_or("ffprobe");
    Command::new(ffprobe)
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get the ffprobe command path
fn get_ffprobe_path(settings: &VerificationSettings) -> &str {
    settings.ffprobe_path.as_deref().unwrap_or("ffprobe")
}

/// Verify a media file using ffprobe
///
/// This function runs ffprobe to analyze the file and check for errors.
/// It returns detailed information about the file's validity.
pub async fn verify_media_file(
    file_path: &Path,
    settings: &VerificationSettings,
) -> VerificationResult {
    // Check if file exists
    if !file_path.exists() {
        return VerificationResult {
            status: VerificationStatus::Error,
            message: "File does not exist".to_string(),
            duration_secs: None,
            format_name: None,
            video_codec: None,
            audio_codec: None,
            file_size: 0,
            error_details: Some(format!("Path: {:?}", file_path)),
        };
    }

    // Get file size
    let file_size = std::fs::metadata(file_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // Check if this is a media file (if media_only is enabled)
    if settings.media_only && !is_media_file(file_path) {
        return VerificationResult {
            status: VerificationStatus::Skipped,
            message: "Not a media file".to_string(),
            duration_secs: None,
            format_name: None,
            video_codec: None,
            audio_codec: None,
            file_size,
            error_details: None,
        };
    }

    // Check if ffprobe is available
    let ffprobe = get_ffprobe_path(settings);
    if !is_ffprobe_available(Some(ffprobe)) {
        return VerificationResult {
            status: VerificationStatus::Skipped,
            message: "ffprobe not available".to_string(),
            duration_secs: None,
            format_name: None,
            video_codec: None,
            audio_codec: None,
            file_size,
            error_details: Some("Install ffmpeg to enable media verification".to_string()),
        };
    }

    // Run ffprobe to get file information
    let probe_result = run_ffprobe(file_path, ffprobe, settings.timeout_secs).await;

    match probe_result {
        Ok(info) => {
            // Check for errors in the probe result
            if info.has_errors {
                VerificationResult {
                    status: VerificationStatus::Invalid,
                    message: "File has errors".to_string(),
                    duration_secs: info.duration,
                    format_name: info.format_name,
                    video_codec: info.video_codec,
                    audio_codec: info.audio_codec,
                    file_size,
                    error_details: info.error_message,
                }
            } else if info.has_warnings {
                VerificationResult {
                    status: VerificationStatus::Warning,
                    message: "File has minor issues but is playable".to_string(),
                    duration_secs: info.duration,
                    format_name: info.format_name,
                    video_codec: info.video_codec,
                    audio_codec: info.audio_codec,
                    file_size,
                    error_details: info.error_message,
                }
            } else {
                VerificationResult {
                    status: VerificationStatus::Valid,
                    message: "File is valid".to_string(),
                    duration_secs: info.duration,
                    format_name: info.format_name,
                    video_codec: info.video_codec,
                    audio_codec: info.audio_codec,
                    file_size,
                    error_details: None,
                }
            }
        }
        Err(e) => VerificationResult {
            status: VerificationStatus::Error,
            message: "Verification failed".to_string(),
            duration_secs: None,
            format_name: None,
            video_codec: None,
            audio_codec: None,
            file_size,
            error_details: Some(e),
        },
    }
}

/// Internal struct for ffprobe results
struct FfprobeInfo {
    duration: Option<f64>,
    format_name: Option<String>,
    video_codec: Option<String>,
    audio_codec: Option<String>,
    has_errors: bool,
    has_warnings: bool,
    error_message: Option<String>,
}

/// Run ffprobe and parse the output
async fn run_ffprobe(
    file_path: &Path,
    ffprobe_path: &str,
    timeout_secs: u64,
) -> Result<FfprobeInfo, String> {
    // Run ffprobe with JSON output
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        AsyncCommand::new(ffprobe_path)
            .args([
                "-v", "error",           // Only show errors
                "-show_format",          // Show format info
                "-show_streams",         // Show stream info
                "-print_format", "json", // Output as JSON
            ])
            .arg(file_path)
            .output(),
    )
    .await
    .map_err(|_| "Verification timed out".to_string())?
    .map_err(|e| format!("Failed to run ffprobe: {}", e))?;

    // Check stderr for errors
    let stderr = String::from_utf8_lossy(&output.stderr);
    let has_errors = !stderr.is_empty() && stderr.contains("error");
    let has_warnings = !stderr.is_empty() && !has_errors;

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.is_empty() && !output.status.success() {
        return Err(format!(
            "ffprobe failed: {}",
            if stderr.is_empty() {
                "Unknown error"
            } else {
                stderr.trim()
            }
        ));
    }

    // Parse JSON
    let json: serde_json::Value =
        serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    // Extract format info
    let format = json.get("format");
    let duration = format
        .and_then(|f| f.get("duration"))
        .and_then(|d| d.as_str())
        .and_then(|s| s.parse::<f64>().ok());

    let format_name = format
        .and_then(|f| f.get("format_name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string());

    // Extract stream info
    let streams = json.get("streams").and_then(|s| s.as_array());

    let mut video_codec = None;
    let mut audio_codec = None;

    if let Some(streams) = streams {
        for stream in streams {
            let codec_type = stream.get("codec_type").and_then(|t| t.as_str());
            let codec_name = stream
                .get("codec_name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());

            match codec_type {
                Some("video") if video_codec.is_none() => video_codec = codec_name,
                Some("audio") if audio_codec.is_none() => audio_codec = codec_name,
                _ => {}
            }
        }
    }

    Ok(FfprobeInfo {
        duration,
        format_name,
        video_codec,
        audio_codec,
        has_errors,
        has_warnings,
        error_message: if stderr.is_empty() {
            None
        } else {
            Some(stderr.trim().to_string())
        },
    })
}

/// Quick integrity check using ffmpeg's error detection
///
/// This performs a faster check by reading the file headers and checking
/// for obvious corruption. It's less thorough than full verification.
pub async fn quick_verify(
    file_path: &Path,
    settings: &VerificationSettings,
) -> VerificationResult {
    // Get file size
    let file_size = std::fs::metadata(file_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // Check if this is a media file
    if settings.media_only && !is_media_file(file_path) {
        return VerificationResult {
            status: VerificationStatus::Skipped,
            message: "Not a media file".to_string(),
            duration_secs: None,
            format_name: None,
            video_codec: None,
            audio_codec: None,
            file_size,
            error_details: None,
        };
    }

    let ffprobe = get_ffprobe_path(settings);
    if !is_ffprobe_available(Some(ffprobe)) {
        return VerificationResult {
            status: VerificationStatus::Skipped,
            message: "ffprobe not available".to_string(),
            duration_secs: None,
            format_name: None,
            video_codec: None,
            audio_codec: None,
            file_size,
            error_details: None,
        };
    }

    // Quick check: just try to read format info
    let output = AsyncCommand::new(ffprobe)
        .args([
            "-v", "error",
            "-show_entries", "format=duration,format_name",
            "-print_format", "json",
        ])
        .arg(file_path)
        .output()
        .await;

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

            let duration = json
                .get("format")
                .and_then(|f| f.get("duration"))
                .and_then(|d| d.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let format_name = json
                .get("format")
                .and_then(|f| f.get("format_name"))
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());

            VerificationResult {
                status: VerificationStatus::Valid,
                message: "Quick check passed".to_string(),
                duration_secs: duration,
                format_name,
                video_codec: None,
                audio_codec: None,
                file_size,
                error_details: None,
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            VerificationResult {
                status: VerificationStatus::Invalid,
                message: "Quick check failed".to_string(),
                duration_secs: None,
                format_name: None,
                video_codec: None,
                audio_codec: None,
                file_size,
                error_details: Some(stderr.trim().to_string()),
            }
        }
        Err(e) => VerificationResult {
            status: VerificationStatus::Error,
            message: "Failed to run verification".to_string(),
            duration_secs: None,
            format_name: None,
            video_codec: None,
            audio_codec: None,
            file_size,
            error_details: Some(e.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_media_file() {
        assert!(is_media_file(Path::new("video.mp4")));
        assert!(is_media_file(Path::new("video.MKV")));
        assert!(is_media_file(Path::new("audio.mp3")));
        assert!(is_media_file(Path::new("audio.FLAC")));
        assert!(!is_media_file(Path::new("document.pdf")));
        assert!(!is_media_file(Path::new("image.jpg")));
        assert!(!is_media_file(Path::new("archive.zip")));
    }

    #[test]
    fn test_verification_settings_default() {
        let settings = VerificationSettings::default();
        assert!(!settings.enabled);
        assert!(settings.media_only);
        assert!(settings.ffprobe_path.is_none());
        assert_eq!(settings.timeout_secs, 60);
    }

    #[test]
    fn test_verification_result_serialization() {
        let result = VerificationResult {
            status: VerificationStatus::Valid,
            message: "File is valid".to_string(),
            duration_secs: Some(120.5),
            format_name: Some("mp4".to_string()),
            video_codec: Some("h264".to_string()),
            audio_codec: Some("aac".to_string()),
            file_size: 1024000,
            error_details: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        // Verify camelCase serialization
        assert!(json.contains("\"durationSecs\""));
        assert!(json.contains("\"formatName\""));
        assert!(json.contains("\"videoCodec\""));
        assert!(json.contains("\"audioCodec\""));
        assert!(json.contains("\"fileSize\""));
        assert!(json.contains("\"errorDetails\""));
    }
}
