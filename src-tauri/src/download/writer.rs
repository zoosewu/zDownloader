// File writer with conflict handling
#![allow(dead_code)]

use std::path::{Path, PathBuf};

pub fn resolve_filename_conflict(filepath: &str) -> PathBuf {
    let path = Path::new(filepath);

    if !path.exists() {
        return path.to_path_buf();
    }

    let parent = path.parent();
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let mut counter = 1;
    loop {
        let new_filename = if ext.is_empty() {
            format!("{} ({})", stem, counter)
        } else {
            format!("{} ({}).{}", stem, counter, ext)
        };

        let new_path = if let Some(p) = parent {
            p.join(new_filename)
        } else {
            PathBuf::from(new_filename)
        };

        if !new_path.exists() {
            return new_path;
        }

        counter += 1;
        if counter > 1000 {
            // Safety limit
            return new_path;
        }
    }
}
