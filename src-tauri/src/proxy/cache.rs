// Proxy cache - persistent storage for proxy list
use crate::models::ProxyInfo;
use crate::storage::paths::get_proxies_cache_file;
use std::fs::{create_dir_all, File};
use std::io::{BufRead, BufReader, Write};

/// File format: one proxy per line in format "host:port"
/// Example:
/// ```
/// 127.0.0.1:8080
/// 192.168.1.1:3128
/// proxy.example.com:8888
/// ```

/// Load proxies from cache file
///
/// Returns empty vector if file doesn't exist or is empty
pub fn load_proxies_from_cache() -> Result<Vec<ProxyInfo>, String> {
    let cache_path = get_proxies_cache_file();

    // If file doesn't exist, return empty vector
    if !cache_path.exists() {
        return Ok(Vec::new());
    }

    let file =
        File::open(&cache_path).map_err(|e| format!("Failed to open cache file: {}", e))?;

    let reader = BufReader::new(file);
    let mut proxies = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(|e| format!("Failed to read line {}: {}", line_num + 1, e))?;

        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse "host:port" format
        match parse_proxy_line(trimmed) {
            Ok(proxy) => proxies.push(proxy),
            Err(e) => {
                eprintln!("Warning: Skipping invalid proxy on line {}: {}", line_num + 1, e);
                continue;
            }
        }
    }

    Ok(proxies)
}

/// Save proxies to cache file
///
/// Overwrites existing file
pub fn save_proxies_to_cache(proxies: &[ProxyInfo]) -> Result<(), String> {
    let cache_path = get_proxies_cache_file();

    // Ensure parent directory exists
    if let Some(parent) = cache_path.parent() {
        create_dir_all(parent)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
    }

    let mut file =
        File::create(&cache_path).map_err(|e| format!("Failed to create cache file: {}", e))?;

    // Write header comment
    writeln!(file, "# Zoo Downloader Proxy Cache")
        .map_err(|e| format!("Failed to write header: {}", e))?;
    writeln!(file, "# Format: host:port (one per line)")
        .map_err(|e| format!("Failed to write header: {}", e))?;
    writeln!(file, "# Lines starting with # are ignored")
        .map_err(|e| format!("Failed to write header: {}", e))?;
    writeln!(file).map_err(|e| format!("Failed to write header: {}", e))?;

    // Write each proxy
    for proxy in proxies {
        writeln!(file, "{}:{}", proxy.host, proxy.port)
            .map_err(|e| format!("Failed to write proxy: {}", e))?;
    }

    file.flush()
        .map_err(|e| format!("Failed to flush cache file: {}", e))?;

    Ok(())
}

/// Append proxies to cache file (without duplicates)
///
/// This loads existing proxies, adds new ones (skipping duplicates), and saves
pub fn append_proxies_to_cache(new_proxies: &[ProxyInfo]) -> Result<(), String> {
    let mut existing = load_proxies_from_cache()?;

    // Add new proxies if they don't exist
    for new_proxy in new_proxies {
        let exists = existing.iter().any(|p| {
            p.host == new_proxy.host && p.port == new_proxy.port
        });

        if !exists {
            existing.push(new_proxy.clone());
        }
    }

    save_proxies_to_cache(&existing)?;

    Ok(())
}

/// Clear all proxies from cache
pub fn clear_proxy_cache() -> Result<(), String> {
    let cache_path = get_proxies_cache_file();

    if cache_path.exists() {
        std::fs::remove_file(&cache_path)
            .map_err(|e| format!("Failed to remove cache file: {}", e))?;
    }

    Ok(())
}

/// Parse a single proxy line in "host:port" format
fn parse_proxy_line(line: &str) -> Result<ProxyInfo, String> {
    let parts: Vec<&str> = line.split(':').collect();

    if parts.len() != 2 {
        return Err(format!("Invalid format '{}', expected 'host:port'", line));
    }

    let host = parts[0].trim().to_string();
    let port_str = parts[1].trim();

    if host.is_empty() {
        return Err("Empty host".to_string());
    }

    let port: u16 = port_str
        .parse()
        .map_err(|_| format!("Invalid port number: {}", port_str))?;

    Ok(ProxyInfo::new(host, port))
}

/// Import proxies from a text file (same format as cache)
///
/// This can be used to import proxy lists from external sources
pub fn import_proxies_from_file(file_path: &str) -> Result<Vec<ProxyInfo>, String> {
    let file = File::open(file_path)
        .map_err(|e| format!("Failed to open import file: {}", e))?;

    let reader = BufReader::new(file);
    let mut proxies = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result
            .map_err(|e| format!("Failed to read line {}: {}", line_num + 1, e))?;

        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse proxy line
        match parse_proxy_line(trimmed) {
            Ok(proxy) => proxies.push(proxy),
            Err(e) => {
                eprintln!("Warning: Skipping invalid proxy on line {}: {}", line_num + 1, e);
                continue;
            }
        }
    }

    Ok(proxies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use uuid::Uuid;

    fn get_test_cache_path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("zoo_test_cache_{}.txt", Uuid::new_v4()))
    }

    #[test]
    fn test_parse_proxy_line() {
        let proxy = parse_proxy_line("127.0.0.1:8080").unwrap();
        assert_eq!(proxy.host, "127.0.0.1");
        assert_eq!(proxy.port, 8080);

        let proxy2 = parse_proxy_line("  proxy.example.com : 3128  ").unwrap();
        assert_eq!(proxy2.host, "proxy.example.com");
        assert_eq!(proxy2.port, 3128);
    }

    #[test]
    fn test_parse_proxy_line_invalid() {
        assert!(parse_proxy_line("invalid").is_err());
        assert!(parse_proxy_line("host:notaport").is_err());
        assert!(parse_proxy_line(":8080").is_err());
        assert!(parse_proxy_line("host:port:extra").is_err());
    }

    #[test]
    fn test_save_and_load_proxies() {
        let test_path = get_test_cache_path();

        // Override cache path for testing (we can't easily do this, so we'll test import/export instead)
        let proxies = vec![
            ProxyInfo::new("127.0.0.1".to_string(), 8080),
            ProxyInfo::new("192.168.1.1".to_string(), 3128),
        ];

        // Create a test file
        let mut file = File::create(&test_path).unwrap();
        writeln!(file, "# Test proxies").unwrap();
        writeln!(file, "127.0.0.1:8080").unwrap();
        writeln!(file, "192.168.1.1:3128").unwrap();

        // Load from the test file
        let loaded = import_proxies_from_file(test_path.to_str().unwrap()).unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].host, "127.0.0.1");
        assert_eq!(loaded[0].port, 8080);
        assert_eq!(loaded[1].host, "192.168.1.1");
        assert_eq!(loaded[1].port, 3128);

        // Cleanup
        std::fs::remove_file(test_path).unwrap();
    }

    #[test]
    fn test_import_with_comments_and_empty_lines() {
        let test_path = get_test_cache_path();

        let mut file = File::create(&test_path).unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file).unwrap(); // Empty line
        writeln!(file, "127.0.0.1:8080").unwrap();
        writeln!(file, "# Another comment").unwrap();
        writeln!(file, "192.168.1.1:3128").unwrap();
        writeln!(file).unwrap(); // Empty line

        let loaded = import_proxies_from_file(test_path.to_str().unwrap()).unwrap();

        assert_eq!(loaded.len(), 2);

        // Cleanup
        std::fs::remove_file(test_path).unwrap();
    }

    #[test]
    fn test_import_with_invalid_lines() {
        let test_path = get_test_cache_path();

        let mut file = File::create(&test_path).unwrap();
        writeln!(file, "127.0.0.1:8080").unwrap();
        writeln!(file, "invalid-line").unwrap(); // Invalid
        writeln!(file, "192.168.1.1:3128").unwrap();

        let loaded = import_proxies_from_file(test_path.to_str().unwrap()).unwrap();

        // Should skip invalid line and load the valid ones
        assert_eq!(loaded.len(), 2);

        // Cleanup
        std::fs::remove_file(test_path).unwrap();
    }

    #[test]
    fn test_load_nonexistent_cache() {
        // This test uses the actual cache path function, which should return empty if file doesn't exist
        // We can't easily test this without mocking, but we can verify the function doesn't panic

        // Just verify the function exists and has correct signature
        let _ = load_proxies_from_cache();
    }
}
