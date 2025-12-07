use regex::Regex;
use std::fs;
use std::path::Path;

/// Parse a text file containing URLs (one per line)
/// Returns a vector of valid HTTP/HTTPS URLs
pub fn parse_url_file(file_path: &Path) -> Result<Vec<String>, String> {
    // Validate file path exists
    if !file_path.exists() {
        return Err(format!(
            "File does not exist: {}",
            file_path.display()
        ));
    }

    // Validate file is readable
    if !file_path.is_file() {
        return Err(format!(
            "Path is not a file: {}",
            file_path.display()
        ));
    }

    // Read file content
    let content = fs::read_to_string(file_path).map_err(|e| {
        format!(
            "Failed to read file '{}': {} (check file permissions)",
            file_path.display(),
            e
        )
    })?;

    // Count total lines for error reporting
    let total_lines = content.lines().count();

    // Parse URLs line by line
    let urls = parse_url_list(&content);

    if urls.is_empty() {
        return Err(format!(
            "No valid URLs found in file '{}' ({} lines read). \
            Expected HTTP/HTTPS URLs, one per line. \
            Lines starting with '#' are treated as comments.",
            file_path.display(),
            total_lines
        ));
    }

    Ok(urls)
}

/// Parse URL list from string content
/// Each line should contain one URL
/// Empty lines and lines starting with # are ignored
pub fn parse_url_list(content: &str) -> Vec<String> {
    let url_regex = Regex::new(
        r"^https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&/=]*)$"
    ).unwrap();

    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| url_regex.is_match(line))
        .map(|line| line.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url_list() {
        let content = r#"
https://example.com/file1.zip
http://example.com/file2.tar.gz

# This is a comment
https://www.example.com/download?id=123

not a valid url
ftp://example.com/file3.zip

https://example.org/file4.pdf
        "#;

        let urls = parse_url_list(content);

        assert_eq!(urls.len(), 4);
        assert_eq!(urls[0], "https://example.com/file1.zip");
        assert_eq!(urls[1], "http://example.com/file2.tar.gz");
        assert_eq!(urls[2], "https://www.example.com/download?id=123");
        assert_eq!(urls[3], "https://example.org/file4.pdf");
    }

    #[test]
    fn test_parse_url_list_empty() {
        let content = r#"
# All comments
# No URLs here

        "#;

        let urls = parse_url_list(content);
        assert_eq!(urls.len(), 0);
    }
}
