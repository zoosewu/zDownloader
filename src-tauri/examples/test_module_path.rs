// Test module path resolution

use std::path::PathBuf;

fn find_modules_directory() -> PathBuf {
    println!("🔧 Debug mode: searching for modules directory...");
    println!("Current dir: {:?}", std::env::current_dir().unwrap());

    let candidate_paths = vec![
        PathBuf::from("./modules"),           // Current directory
        PathBuf::from("../../modules"),       // From target/debug
        PathBuf::from("../modules"),          // From src-tauri
        PathBuf::from("../../../modules"),    // From deep nested directories
    ];

    // Try each candidate path
    for path in &candidate_paths {
        println!("Trying: {:?}", path);
        if path.exists() && path.is_dir() {
            let canonical = path.canonicalize().unwrap_or(path.clone());
            println!("✓ Found modules directory at: {:?}", canonical);

            // List contents
            if let Ok(entries) = std::fs::read_dir(&canonical) {
                println!("  Contents:");
                for entry in entries {
                    if let Ok(entry) = entry {
                        println!("    - {:?}", entry.file_name());
                    }
                }
            }

            return path.clone();
        } else {
            println!("  ✗ Not found or not a directory");
        }
    }

    println!("⚠️  Could not find existing modules directory");
    PathBuf::from("./modules")
}

fn main() {
    let modules_dir = find_modules_directory();
    println!("\nFinal modules_dir: {:?}", modules_dir);
    println!("Canonical path: {:?}", modules_dir.canonicalize());
}
