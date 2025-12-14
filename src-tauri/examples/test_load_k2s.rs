// Test loading k2s module

use std::path::PathBuf;
use zoo_downloader_lib::modules::loader::ModuleLoader;
use zoo_downloader_lib::modules::registry::ModuleRegistry;

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

    let modules_dir = find_modules_directory();
    println!("📁 Modules directory: {:?}", modules_dir.canonicalize().unwrap());

    let loader = ModuleLoader::new(&modules_dir);
    let k2s_path = modules_dir.join("k2s");

    println!("\n🔍 Testing k2s module...");
    println!("📁 k2s path: {:?}", k2s_path);
    println!("📁 k2s exists: {}", k2s_path.exists());

    // Check manifest.json
    let manifest_path = k2s_path.join("manifest.json");
    println!("📄 manifest.json exists: {}", manifest_path.exists());
    if manifest_path.exists() {
        let content = std::fs::read_to_string(&manifest_path).unwrap();
        println!("📄 manifest.json content:\n{}", content);
    }

    // Check dist/index.js
    let dist_path = k2s_path.join("dist");
    let index_path = k2s_path.join("dist/index.js");
    println!("\n📁 dist directory exists: {}", dist_path.exists());
    println!("📄 dist/index.js exists: {}", index_path.exists());
    if index_path.exists() {
        let size = std::fs::metadata(&index_path).unwrap().len();
        println!("📄 dist/index.js size: {} bytes", size);
    }

    // Try loading the module
    println!("\n⏳ Loading k2s module...");
    match loader.load_module(&k2s_path) {
        Ok(module) => {
            println!("✅ Successfully loaded k2s module!");
            println!("   ID: {}", module.manifest().id);
            println!("   Name: {}", module.manifest().name);
            println!("   Version: {}", module.manifest().version);
        }
        Err(e) => {
            println!("❌ Failed to load k2s module: {}", e);
        }
    }

    // Try scanning all modules
    println!("\n⏳ Scanning all modules...");
    let mut registry = ModuleRegistry::with_path(&modules_dir);
    let loaded = registry.scan_and_register(&modules_dir);
    println!("✅ Loaded {} modules", loaded);

    for manifest in registry.get_all_js_module_manifests() {
        println!("   • {} v{} (ID: {})", manifest.name, manifest.version, manifest.id);
    }
}
