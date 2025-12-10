// Example program to test module loading with ES6 syntax transformation
// Run with: cargo run --example test_module_loading

use std::path::PathBuf;

fn main() {
    println!("=== Testing Module Loading ===\n");

    // Find modules directory using the same logic as find_modules_directory()
    let modules_dir = find_modules_directory();

    println!("📁 Using modules directory: {:?}\n", modules_dir.canonicalize().unwrap_or(modules_dir.clone()));

    // Check if directory exists
    if !modules_dir.exists() {
        eprintln!("❌ Modules directory not found!");
        return;
    }

    // List files in directory
    println!("📋 Directory contents:");
    if let Ok(entries) = std::fs::read_dir(&modules_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.path().is_dir();
            println!("  {} {}", if is_dir { "📁" } else { "📄" }, name);
        }
    }
    println!();

    // Test module loading by creating a temporary registry
    println!("🔄 Loading modules...\n");

    // We can't directly test ModuleRegistry here due to module visibility,
    // but we can verify the files exist

    // Check k2s module
    let k2s_dir = modules_dir.join("k2s");
    if k2s_dir.exists() {
        println!("✓ Found k2s module directory");

        let manifest = k2s_dir.join("manifest.json");
        let index = k2s_dir.join("index.js");

        if manifest.exists() {
            println!("  ✓ manifest.json exists");
        } else {
            println!("  ❌ manifest.json missing");
        }

        if index.exists() {
            println!("  ✓ index.js exists");

            // Read and check for export syntax
            if let Ok(code) = std::fs::read_to_string(&index) {
                if code.contains("export class") {
                    println!("  ⚠️  Contains 'export class' - will be transformed to 'class'");
                } else if code.contains("class") {
                    println!("  ✓ Contains 'class' (no export)");
                }
            }
        } else {
            println!("  ❌ index.js missing");
        }
        println!();
    }

    // Check example-simple module
    let example_dir = modules_dir.join("example-simple");
    if example_dir.exists() {
        println!("✓ Found example-simple module directory");

        let manifest = example_dir.join("manifest.json");
        let index = example_dir.join("index.js");

        if manifest.exists() {
            println!("  ✓ manifest.json exists");
        } else {
            println!("  ❌ manifest.json missing");
        }

        if index.exists() {
            println!("  ✓ index.js exists");

            // Read and check for export syntax
            if let Ok(code) = std::fs::read_to_string(&index) {
                if code.contains("export class") {
                    println!("  ⚠️  Contains 'export class' - will be transformed to 'class'");
                } else if code.contains("class") {
                    println!("  ✓ Contains 'class' (no export)");
                }
            }
        } else {
            println!("  ❌ index.js missing");
        }
        println!();
    }

    println!("✅ Module file validation complete!");
    println!("\n💡 To test actual loading, run the application and click 'Scan Modules'");
}

fn find_modules_directory() -> PathBuf {
    // Debug/Dev mode: smart search for developer convenience
    #[cfg(debug_assertions)]
    {
        println!("🔧 Debug mode: searching for modules directory...");

        let candidate_paths = vec![
            PathBuf::from("./modules"),
            PathBuf::from("../modules"),
            PathBuf::from("../../modules"),
            PathBuf::from("../../../modules"),
        ];

        for path in &candidate_paths {
            if path.exists() && path.is_dir() {
                println!("✓ Found modules directory at: {:?}\n", path.canonicalize().unwrap_or(path.clone()));
                return path.clone();
            }
        }

        println!("⚠️  Could not find existing modules directory, using fallback: ./modules\n");
        PathBuf::from("./modules")
    }

    // Release mode: only use ./modules
    #[cfg(not(debug_assertions))]
    {
        println!("🚀 Release mode: using ./modules\n");
        PathBuf::from("./modules")
    }
}
