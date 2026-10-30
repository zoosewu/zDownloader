//! File naming: everything that turns untrusted input into a path under the download directory.
//!
//! `PathBuf::join` replaces the base on absolute paths and happily follows `..`, so every
//! name must pass through `sanitize` before it touches the filesystem.

use std::path::{Path, PathBuf};

const FALLBACK: &str = "download";

/// Pick the requested name (or the URL's last path segment, percent-decoded) and make it
/// a single safe path component.
pub fn filename_for(requested: Option<&str>, url: &str) -> String {
    let raw = requested
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
        .or_else(|| last_path_segment(url))
        .unwrap_or_default();
    sanitize(&raw)
}

fn last_path_segment(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    let segment = parsed.path_segments()?.next_back().filter(|s| !s.is_empty())?;
    Some(
        percent_encoding::percent_decode_str(segment)
            .decode_utf8_lossy()
            .into_owned(),
    )
}

/// Reduce `raw` to one filename component that cannot escape its directory.
pub fn sanitize(raw: &str) -> String {
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
        return format!("_{cleaned}");
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

/// In-progress file for a task. The task id keeps two downloads of the same name apart.
pub fn part_path(dir: &Path, name: &str, task_id: &str) -> PathBuf {
    let short: String = task_id.chars().filter(|c| c.is_ascii_alphanumeric()).take(8).collect();
    dir.join(format!("{name}.{short}.part"))
}

/// First of `name`, `stem (1).ext`, `stem (2).ext`, ... that does not exist in `dir`.
pub fn available_path(dir: &Path, name: &str) -> PathBuf {
    let first = dir.join(name);
    if !first.exists() {
        return first;
    }
    let (stem, ext) = split_extension(name);
    (1..)
        .map(|n| dir.join(format!("{stem} ({n}){ext}")))
        .find(|p| !p.exists())
        .expect("unbounded search always finds a free name")
}

/// "a.tar.gz" -> ("a.tar", ".gz"); ".hidden" and "noext" have no extension.
fn split_extension(name: &str) -> (&str, &str) {
    match name.rfind('.') {
        Some(i) if i > 0 => name.split_at(i),
        _ => (name, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Component;

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
            let name = sanitize(raw);
            assert_single_component(&name);
            assert_ne!(name, "..");
        }
        assert_eq!(sanitize(r"..\..\Startup\x.bat"), "x.bat");
        assert_eq!(sanitize(r"C:\Windows\evil.dll"), "evil.dll");
        assert_eq!(sanitize(".."), "download");
    }

    #[test]
    fn removes_illegal_characters_and_trailing_dots() {
        assert_eq!(sanitize("a<b>c:d\"e|f?g*h.txt"), "abcdefgh.txt");
        assert_eq!(sanitize("file.zip. . "), "file.zip");
        assert_eq!(sanitize("tab\there.bin"), "tabhere.bin");
    }

    #[test]
    fn prefixes_windows_reserved_names() {
        assert_eq!(sanitize("CON"), "_CON");
        assert_eq!(sanitize("nul.txt"), "_nul.txt");
        assert_eq!(sanitize("COM1.log"), "_COM1.log");
        assert_eq!(sanitize("COM0.log"), "COM0.log");
        assert_eq!(sanitize("CONSOLE.txt"), "CONSOLE.txt");
    }

    #[test]
    fn filename_prefers_request_then_decoded_url_path() {
        assert_eq!(filename_for(Some("a.zip"), "https://x.com/b.zip"), "a.zip");
        assert_eq!(filename_for(Some("  "), "https://x.com/dir/b.zip?token=1#f"), "b.zip");
        assert_eq!(filename_for(None, "https://x.com/"), "download");
        assert_eq!(filename_for(None, "https://x.com/%E5%BD%B1%E7%89%87.mp4"), "影片.mp4");
        // Decoding must not reopen traversal: "%2e%2e%5c" is "..\".
        assert_eq!(filename_for(None, "https://x.com/%2e%2e%5cevil.bat"), "evil.bat");
    }

    #[test]
    fn part_path_is_unique_per_task() {
        let dir = Path::new("/d");
        assert_ne!(part_path(dir, "a.zip", "11111111-aaaa"), part_path(dir, "a.zip", "22222222-bbbb"));
        assert_eq!(part_path(dir, "a.zip", "1234abcd-ef"), dir.join("a.zip.1234abcd.part"));
    }

    #[test]
    fn available_path_never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(available_path(dir.path(), "a.tar.gz"), dir.path().join("a.tar.gz"));
        std::fs::write(dir.path().join("a.tar.gz"), b"x").unwrap();
        assert_eq!(available_path(dir.path(), "a.tar.gz"), dir.path().join("a.tar (1).gz"));
        std::fs::write(dir.path().join("a.tar (1).gz"), b"x").unwrap();
        assert_eq!(available_path(dir.path(), "a.tar.gz"), dir.path().join("a.tar (2).gz"));
        std::fs::write(dir.path().join(".hidden"), b"x").unwrap();
        assert_eq!(available_path(dir.path(), ".hidden"), dir.path().join(".hidden (1)"));
    }
}
