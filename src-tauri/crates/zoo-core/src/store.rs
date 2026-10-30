//! Durable JSON files. A reader never sees a half-written file: writes go to a sibling
//! temp file, are fsynced, then renamed over the target (atomic on the same volume).

use serde::{de::DeserializeOwned, Serialize};
use std::io::{self, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// Replace `path` with `bytes` atomically.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut tmp_name = path.file_name().unwrap_or_default().to_os_string();
    tmp_name.push(".tmp");
    let tmp = path.with_file_name(tmp_name);

    let mut file = std::fs::File::create(&tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    // std::fs::rename replaces an existing destination on both Unix and Windows.
    std::fs::rename(&tmp, path)
}

/// One JSON document on disk.
pub struct JsonStore<T> {
    path: PathBuf,
    _marker: PhantomData<fn() -> T>,
}

impl<T: Serialize + DeserializeOwned> JsonStore<T> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), _marker: PhantomData }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// `Ok(None)` when the file does not exist yet.
    pub fn load(&self) -> io::Result<Option<T>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn save(&self, value: &T) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(value)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        write_atomic(&self.path, &bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_missing_is_none_and_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let store = JsonStore::<Vec<u32>>::new(dir.path().join("nested/tasks.json"));
        assert_eq!(store.load().unwrap(), None);

        store.save(&vec![1, 2, 3]).unwrap();
        assert_eq!(store.load().unwrap(), Some(vec![1, 2, 3]));

        store.save(&vec![4]).unwrap();
        assert_eq!(store.load().unwrap(), Some(vec![4]));
        assert!(!dir.path().join("nested/tasks.json.tmp").exists());
    }

    #[test]
    fn corrupt_file_is_an_error_not_silently_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tasks.json");
        std::fs::write(&path, b"{not json").unwrap();
        let store = JsonStore::<Vec<u32>>::new(&path);
        assert_eq!(store.load().unwrap_err().kind(), io::ErrorKind::InvalidData);
    }
}
