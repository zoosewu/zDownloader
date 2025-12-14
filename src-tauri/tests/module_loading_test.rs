// Integration test for module loading

use std::path::PathBuf;

// Mock find_modules_directory function
fn find_modules_directory() -> PathBuf {
    // When running tests, we're in target/debug/deps, so we need to go up several levels
    let candidate_paths = vec![
        PathBuf::from("./modules"),
        PathBuf::from("../../modules"),
        PathBuf::from("../modules"),
        PathBuf::from("../../../modules"),
        PathBuf::from("../../../../modules"),  // From target/debug/deps
    ];

    for path in &candidate_paths {
        if path.exists() && path.is_dir() {
            return path.clone();
        }
    }

    PathBuf::from("./modules")
}

#[test]
fn test_find_modules_directory() {
    let modules_dir = find_modules_directory();
    println!("Found modules directory: {:?}", modules_dir);

    // Canonicalize to get absolute path
    if let Ok(canonical) = modules_dir.canonicalize() {
        println!("Canonical path: {:?}", canonical);

        // List contents
        if let Ok(entries) = std::fs::read_dir(&canonical) {
            println!("Contents:");
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("  - {:?}", entry.file_name());
                }
            }
        }

        // Check for k2s module
        let k2s_dir = canonical.join("k2s");
        println!("\nk2s module check:");
        println!("  - k2s directory exists: {}", k2s_dir.exists());

        if k2s_dir.exists() {
            println!("  - k2s/manifest.json exists: {}", k2s_dir.join("manifest.json").exists());
            println!("  - k2s/dist exists: {}", k2s_dir.join("dist").exists());
            println!("  - k2s/dist/index.js exists: {}", k2s_dir.join("dist/index.js").exists());

            // Check file size
            if let Ok(metadata) = std::fs::metadata(k2s_dir.join("dist/index.js")) {
                println!("  - k2s/dist/index.js size: {} bytes", metadata.len());
            }
        }
    } else {
        println!("Failed to canonicalize path: {:?}", modules_dir);
    }

    assert!(modules_dir.exists(), "Modules directory should exist");
}
