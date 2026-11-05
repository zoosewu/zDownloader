//! `zoo-module new`: the template (crates/zoo-module/template) plus the SDK types in `types/`.

use std::path::Path;

/// Path in the new module → contents. The manifest is written separately (id and name filled in).
const FILES: &[(&str, &str)] = &[
    ("AGENTS.md", include_str!("../../template/AGENTS.md")),
    // Claude Code reads CLAUDE.md; this imports the shared instructions.
    ("CLAUDE.md", "@AGENTS.md\n"),
    (".gitignore", "node_modules/\n.credentials.json\n"),
    ("package.json", include_str!("../../template/package.json")),
    ("tsconfig.json", include_str!("../../template/tsconfig.json")),
    ("src/index.ts", include_str!("../../template/src/index.ts")),
    ("tests/cases.json", include_str!("../../template/tests/cases.json")),
    ("types/zoo-module.d.ts", include_str!("../../sdk/zoo-module.d.ts")),
    ("types/globals.d.ts", include_str!("../../sdk/globals.d.ts")),
    ("types/manifest.schema.json", include_str!("../../sdk/manifest.schema.json")),
];
const MANIFEST: &str = include_str!("../../template/manifest.json");

/// Returns the module id.
pub fn create(dir: &Path, id: Option<String>, name: Option<String>) -> Result<String, String> {
    if dir.read_dir().is_ok_and(|mut entries| entries.next().is_some()) {
        return Err(format!("{} is not empty", dir.display()));
    }
    let id = match id {
        Some(id) => id,
        None => default_id(dir)?,
    };
    let mut manifest: serde_json::Value = serde_json::from_str(MANIFEST).expect("template manifest is JSON");
    manifest["id"] = id.clone().into();
    manifest["name"] = name.unwrap_or_else(|| id.clone()).into();
    let manifest = serde_json::to_string_pretty(&manifest).expect("serializes") + "\n";
    zoo_module_client::manifest::Manifest::parse(&manifest)?;

    let write = |path: &str, contents: &str| -> Result<(), String> {
        let path = dir.join(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        std::fs::write(&path, contents).map_err(|e| format!("cannot write {}: {e}", path.display()))
    };
    write("manifest.json", &manifest)?;
    for (path, contents) in FILES {
        write(path, contents)?;
    }
    Ok(id)
}

/// The directory name, lowercased, with anything else than letters and digits turned into '-'.
fn default_id(dir: &Path) -> Result<String, String> {
    let name = std::path::absolute(dir)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()))
        .unwrap_or_default();
    let id: String = name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    let id = id.trim_matches('-').to_string();
    if id.is_empty() {
        return Err("cannot derive an id from the directory name; pass --id".into());
    }
    Ok(id)
}
