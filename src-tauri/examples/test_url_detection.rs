// Test URL detection with k2s module

use std::path::PathBuf;

fn find_modules_directory() -> PathBuf {
    let candidate_paths = vec![
        PathBuf::from("./modules"),
        PathBuf::from("../../modules"),
        PathBuf::from("../modules"),
    ];

    for path in &candidate_paths {
        if path.exists() && path.is_dir() {
            return path.clone();
        }
    }

    PathBuf::from("./modules")
}

fn main() {
    println!("=== Testing URL Detection with k2s Module ===\n");

    // Find modules directory
    let modules_dir = find_modules_directory();
    println!("📁 Modules directory: {:?}\n", modules_dir.canonicalize().unwrap());

    // Create module registry and scan for modules
    let mut registry = zoo_downloader_lib::modules::registry::ModuleRegistry::with_path(&modules_dir);
    let loaded_count = registry.scan_and_register(&modules_dir);
    println!("📦 Loaded {} module(s)\n", loaded_count);

    // List loaded modules
    let manifests = registry.get_all_js_module_manifests();
    println!("Loaded modules:");
    for manifest in &manifests {
        println!("  • {} (ID: {})", manifest.name, manifest.id);
        println!("    URL patterns: {:?}", manifest.url_patterns);
    }
    println!();

    // Test URLs
    let test_urls = vec![
        "https://k2s.cc/file/1234567890abcdef",
        "https://keep2share.cc/file/test123",
        "https://example.com/file.zip",
    ];

    println!("Testing URL detection:");
    for url in test_urls {
        println!("\n  URL: {}", url);

        let result = registry.find_js_module_for_url(url);
        match result {
            Some(module_id) => {
                let manifest = manifests.iter().find(|m| m.id == module_id).unwrap();
                println!("    ✓ Module: {} ({})", manifest.name, module_id);
                println!("    ✓ Requires auth: {}", manifest.permissions.credentials.is_some());
            }
            None => {
                println!("    ✗ No module found");
            }
        }
    }

    println!("\n=== Test completed ===");
}
