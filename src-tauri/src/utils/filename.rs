/// Filename safety for anything that becomes a path under the download directory.
///
/// Every download ingress (UI, batch import, Chrome extension HTTP endpoint) must
/// pass its filename through `safe_filename` before joining it onto a directory:
/// `PathBuf::join` replaces the base on absolute paths and happily follows `..`.
use crate::utils::url_validator::extract_filename;

const FALLBACK: &str = "download";

/// Pick the requested name (or the URL's last path segment) and make it a single safe component.
pub fn safe_filename(requested: Option<String>, url: &str) -> String {
    let raw = requested
        .filter(|s| !s.trim().is_empty())
        .or_else(|| extract_filename(url))
        .unwrap_or_default();
    sanitize_filename(&raw)
}

/// Reduce `raw` to one filename component that cannot escape its directory.
pub fn sanitize_filename(raw: &str) -> String {
    // Keep only the last component under either separator style.
    let base = raw.rsplit(['/', '\\']).next().unwrap_or("");

    let cleaned: String = base
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
        .collect();
    // Windows silently drops trailing dots and spaces, which can turn "x.." into "x".
    let cleaned = cleaned.trim().trim_end_matches(['.', ' ']).to_string();

    if cleaned.is_empty() || cleaned.chars().all(|c| c == '.') {
        return FALLBACK.to_string();
    }
    if is_windows_reserved(&cleaned) {
        return format!("_{}", cleaned);
    }
    cleaned
}

fn is_windows_reserved(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem.as_bytes()[3].is_ascii_digit()
            && stem.as_bytes()[3] != b'0')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Component, Path};

    fn assert_single_component(name: &str) {
        let comps: Vec<_> = Path::new(name).components().collect();
        assert_eq!(comps.len(), 1, "{name:?} is not a single component");
        assert!(matches!(comps[0], Component::Normal(_)), "{name:?} is not a normal component");
    }

    #[test]
    fn strips_traversal_and_absolute_paths() {
        for raw in [
            r"..\..\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup\x.bat",
            "../../etc/passwd",
            r"C:\Windows\System32\evil.dll",
            "/etc/cron.d/job",
            r"\\server\share\file.exe",
            "..",
            ".",
            "",
        ] {
            let name = sanitize_filename(raw);
            assert_single_component(&name);
            assert_ne!(name, "..");
        }
        assert_eq!(sanitize_filename(r"..\..\Startup\x.bat"), "x.bat");
        assert_eq!(sanitize_filename(r"C:\Windows\evil.dll"), "evil.dll");
        assert_eq!(sanitize_filename(".."), "download");
    }

    #[test]
    fn removes_illegal_characters_and_trailing_dots() {
        assert_eq!(sanitize_filename("a<b>c:d\"e|f?g*h.txt"), "abcdefgh.txt");
        assert_eq!(sanitize_filename("file.zip. . "), "file.zip");
        assert_eq!(sanitize_filename("tab\there.bin"), "tabhere.bin");
    }

    #[test]
    fn prefixes_windows_reserved_names() {
        assert_eq!(sanitize_filename("CON"), "_CON");
        assert_eq!(sanitize_filename("nul.txt"), "_nul.txt");
        assert_eq!(sanitize_filename("COM1.log"), "_COM1.log");
        assert_eq!(sanitize_filename("COM0.log"), "COM0.log");
        assert_eq!(sanitize_filename("CONSOLE.txt"), "CONSOLE.txt");
    }

    #[test]
    fn keeps_normal_names() {
        assert_eq!(sanitize_filename("影片 01.mp4"), "影片 01.mp4");
        assert_eq!(sanitize_filename(".hidden"), ".hidden");
    }

    #[test]
    fn safe_filename_prefers_request_then_url_path() {
        assert_eq!(safe_filename(Some("a.zip".into()), "https://x.com/b.zip"), "a.zip");
        assert_eq!(safe_filename(Some("  ".into()), "https://x.com/dir/b.zip?token=1#f"), "b.zip");
        assert_eq!(safe_filename(None, "https://x.com/"), "download");
        assert_eq!(safe_filename(None, "https://x.com/%2e%2e%5cevil.bat"), "%2e%2e%5cevil.bat");
    }
}
