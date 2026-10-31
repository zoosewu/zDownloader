//! In-progress download file.
//!
//! Data is written in place at its final offset into `<name>.<id>.part`; a sidecar
//! `<...>.part.json` records which byte ranges are done. Resume is "download what the
//! sidecar says is missing", and completion is a single rename, so there are no segment
//! files to merge and no way for bookkeeping and data to disagree.
//!
//! Durability rule: the sidecar is only written after the data it describes has been
//! synced, so a crash can lose progress but never mark unwritten bytes as done.

use crate::store::write_atomic;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const FLUSH_INTERVAL: Duration = Duration::from_secs(1);
const FLUSH_BYTES: u64 = 8 * 1024 * 1024;

/// What the server said about the resource. Resume is only safe if all of it still matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    pub url: String,
    pub size: Option<u64>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Sidecar {
    validator: Validator,
    done: RangeSet,
}

struct State {
    done: RangeSet,
    last_flush: Instant,
    unflushed: u64,
}

pub struct PartFile {
    path: PathBuf,
    file: File,
    validator: Validator,
    state: Mutex<State>,
    auto_flush: bool,
}

impl PartFile {
    /// Start from scratch, discarding any previous part file at `path`.
    pub fn create(path: PathBuf, validator: Validator) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        if let Some(size) = validator.size {
            file.set_len(size)?;
        }
        let part = Self::with_state(path, file, validator, RangeSet::default());
        part.flush()?;
        Ok(part)
    }

    /// Reopen a previous attempt if it describes the same resource; `None` means start over.
    pub fn resume(path: PathBuf, validator: &Validator) -> io::Result<Option<Self>> {
        // Without a known size there is no way to verify what is missing.
        if validator.size.is_none() {
            return Ok(None);
        }
        let sidecar: Sidecar = match std::fs::read(sidecar_path(&path)) {
            Ok(bytes) => match serde_json::from_slice(&bytes) {
                Ok(s) => s,
                Err(_) => return Ok(None),
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        if sidecar.validator != *validator {
            return Ok(None);
        }
        let file = match OpenOptions::new().read(true).write(true).open(&path) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        if Some(file.metadata()?.len()) != validator.size {
            return Ok(None);
        }
        Ok(Some(Self::with_state(path, file, sidecar.validator, sidecar.done)))
    }

    fn with_state(path: PathBuf, file: File, validator: Validator, done: RangeSet) -> Self {
        Self {
            path,
            file,
            validator,
            state: Mutex::new(State { done, last_flush: Instant::now(), unflushed: 0 }),
            auto_flush: true,
        }
    }

    /// Only explicit `flush` calls persist progress; lets tests model "not flushed yet" without timing.
    #[cfg(test)]
    fn without_auto_flush(mut self) -> Self {
        self.auto_flush = false;
        self
    }

    pub fn size(&self) -> Option<u64> {
        self.validator.size
    }

    /// Write `buf` at `offset` and record it as done. Persists the record periodically.
    pub fn write_at(&self, offset: u64, buf: &[u8]) -> io::Result<()> {
        if let Some(size) = self.validator.size {
            if offset + buf.len() as u64 > size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("write past end: {}+{} > {size}", offset, buf.len()),
                ));
            }
        }
        write_all_at(&self.file, buf, offset)?;

        let due = {
            let mut st = self.state.lock().unwrap();
            st.done.insert(offset..offset + buf.len() as u64);
            st.unflushed += buf.len() as u64;
            self.auto_flush && (st.unflushed >= FLUSH_BYTES || st.last_flush.elapsed() >= FLUSH_INTERVAL)
        };
        if due {
            self.flush()?;
        }
        Ok(())
    }

    /// Sync data, then persist the done-ranges record.
    pub fn flush(&self) -> io::Result<()> {
        self.file.sync_data()?;
        let sidecar = {
            let mut st = self.state.lock().unwrap();
            st.last_flush = Instant::now();
            st.unflushed = 0;
            Sidecar { validator: self.validator.clone(), done: st.done.clone() }
        };
        let bytes = serde_json::to_vec(&sidecar).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        write_atomic(&sidecar_path(&self.path), &bytes)
    }

    pub fn downloaded(&self) -> u64 {
        self.state.lock().unwrap().done.covered()
    }

    /// Finished byte ranges `[start, end)`, sorted.
    pub fn done_ranges(&self) -> Vec<(u64, u64)> {
        self.state.lock().unwrap().done.0.clone()
    }

    /// Size and finished ranges recorded for the part file at `path`, without opening it.
    /// `None` when there is no (readable) record. Reflects the last flush, not unflushed writes.
    pub fn read_record(path: &Path) -> Option<(Option<u64>, Vec<(u64, u64)>)> {
        let bytes = std::fs::read(sidecar_path(path)).ok()?;
        let sidecar: Sidecar = serde_json::from_slice(&bytes).ok()?;
        Some((sidecar.validator.size, sidecar.done.0))
    }

    /// Byte ranges still to fetch. Empty for unknown-size files (they are never resumed).
    pub fn missing(&self) -> Vec<Range<u64>> {
        match self.validator.size {
            Some(size) => self.state.lock().unwrap().done.gaps(size),
            None => Vec::new(),
        }
    }

    pub fn is_complete(&self) -> bool {
        match self.validator.size {
            Some(size) => self.state.lock().unwrap().done.gaps(size).is_empty(),
            // Unknown size: complete when the stream ended, which only the caller knows.
            None => true,
        }
    }

    /// Move the finished file into place as `dir/name`, renaming on conflict. Returns the final path.
    pub fn finalize(self, dir: &Path, name: &str) -> io::Result<PathBuf> {
        if !self.is_complete() {
            return Err(io::Error::other(format!(
                "incomplete: {} bytes missing",
                self.missing().iter().map(|r| r.end - r.start).sum::<u64>()
            )));
        }
        if self.validator.size.is_none() {
            // Truncate to what was actually written; the file was never pre-sized.
            self.file.set_len(self.downloaded())?;
        }
        self.file.sync_all()?;
        let Self { path, file, .. } = self;
        drop(file);

        let target = crate::naming::available_path(dir, name);
        std::fs::rename(&path, &target)?;
        remove_if_exists(&sidecar_path(&path))?;
        Ok(target)
    }

    /// Delete the part file and its record.
    pub fn discard(path: &Path) -> io::Result<()> {
        remove_if_exists(path)?;
        remove_if_exists(&sidecar_path(path))
    }
}

fn sidecar_path(part: &Path) -> PathBuf {
    let mut name = part.file_name().unwrap_or_default().to_os_string();
    name.push(".json");
    part.with_file_name(name)
}

fn remove_if_exists(path: &Path) -> io::Result<()> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

#[cfg(unix)]
fn write_all_at(file: &File, buf: &[u8], offset: u64) -> io::Result<()> {
    std::os::unix::fs::FileExt::write_all_at(file, buf, offset)
}

#[cfg(windows)]
fn write_all_at(file: &File, mut buf: &[u8], mut offset: u64) -> io::Result<()> {
    use std::os::windows::fs::FileExt;
    while !buf.is_empty() {
        match file.seek_write(buf, offset) {
            Ok(0) => return Err(io::Error::new(io::ErrorKind::WriteZero, "failed to write whole buffer")),
            Ok(n) => {
                buf = &buf[n..];
                offset += n as u64;
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Sorted, non-overlapping, non-adjacent half-open byte ranges.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct RangeSet(Vec<(u64, u64)>);

impl RangeSet {
    fn insert(&mut self, r: Range<u64>) {
        if r.is_empty() {
            return;
        }
        let (mut start, mut end) = (r.start, r.end);
        // Absorb every existing range that overlaps or touches [start, end).
        self.0.retain(|&(s, e)| {
            if e < start || s > end {
                true
            } else {
                start = start.min(s);
                end = end.max(e);
                false
            }
        });
        let at = self.0.partition_point(|&(s, _)| s < start);
        self.0.insert(at, (start, end));
    }

    fn covered(&self) -> u64 {
        self.0.iter().map(|(s, e)| e - s).sum()
    }

    fn gaps(&self, total: u64) -> Vec<Range<u64>> {
        let mut out = Vec::new();
        let mut cursor = 0;
        for &(s, e) in &self.0 {
            if s > cursor {
                out.push(cursor..s.min(total));
            }
            cursor = cursor.max(e);
        }
        if cursor < total {
            out.push(cursor..total);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validator(size: Option<u64>) -> Validator {
        Validator { url: "https://x/f".into(), size, etag: Some("\"v1\"".into()), last_modified: None }
    }

    #[test]
    fn range_set_merges_and_finds_gaps() {
        let mut set = RangeSet::default();
        set.insert(10..20);
        set.insert(30..40);
        set.insert(20..25); // touches 10..20
        set.insert(35..50); // overlaps 30..40
        assert_eq!(set.0, vec![(10, 25), (30, 50)]);
        assert_eq!(set.covered(), 35);
        assert_eq!(set.gaps(60), vec![0..10, 25..30, 50..60]);
        set.insert(0..60);
        assert_eq!(set.0, vec![(0, 60)]);
        assert!(set.gaps(60).is_empty());
    }

    #[test]
    fn out_of_order_writes_complete_and_finalize() {
        let dir = tempfile::tempdir().unwrap();
        let part = PartFile::create(dir.path().join("f.part"), validator(Some(10))).unwrap();
        part.write_at(5, b"56789").unwrap();
        assert!(!part.is_complete());
        assert_eq!(part.missing(), vec![0..5]);
        part.write_at(0, b"01234").unwrap();
        assert!(part.is_complete());

        let out = part.finalize(dir.path(), "f.bin").unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), b"0123456789");
        assert!(!dir.path().join("f.part").exists());
        assert!(!dir.path().join("f.part.json").exists());
    }

    #[test]
    fn finalize_refuses_incomplete_and_renames_on_conflict() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("f.bin"), b"existing").unwrap();

        let part = PartFile::create(dir.path().join("f.part"), validator(Some(4))).unwrap().without_auto_flush();
        part.write_at(0, b"ab").unwrap();
        let part = match part.finalize(dir.path(), "f.bin") {
            Err(e) => {
                assert!(e.to_string().contains("2 bytes missing"));
                PartFile::resume(dir.path().join("f.part"), &validator(Some(4))).unwrap().unwrap()
            }
            Ok(_) => panic!("finalized an incomplete file"),
        };
        // "ab" was never flushed, so it is not claimed as done.
        assert_eq!(part.missing(), vec![0..4]);
        part.write_at(0, b"abcd").unwrap();
        let out = part.finalize(dir.path(), "f.bin").unwrap();
        assert_eq!(out, dir.path().join("f (1).bin"));
        assert_eq!(std::fs::read(dir.path().join("f.bin")).unwrap(), b"existing");
        assert_eq!(std::fs::read(out).unwrap(), b"abcd");
    }

    #[test]
    fn resume_keeps_flushed_progress_only_for_same_resource() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f.part");
        {
            let part = PartFile::create(path.clone(), validator(Some(8))).unwrap();
            part.write_at(0, b"abcd").unwrap();
            part.flush().unwrap();
        } // dropped: simulates the app closing

        let resumed = PartFile::resume(path.clone(), &validator(Some(8))).unwrap().unwrap();
        assert_eq!(resumed.downloaded(), 4);
        assert_eq!(resumed.missing(), vec![4..8]);
        drop(resumed);

        let mut changed = validator(Some(8));
        changed.etag = Some("\"v2\"".into());
        assert!(PartFile::resume(path.clone(), &changed).unwrap().is_none());
        assert!(PartFile::resume(path.clone(), &validator(Some(9))).unwrap().is_none());
        assert!(PartFile::resume(dir.path().join("missing.part"), &validator(Some(8))).unwrap().is_none());
    }

    #[test]
    fn unflushed_progress_is_not_claimed_after_crash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f.part");
        {
            let part = PartFile::create(path.clone(), validator(Some(8))).unwrap().without_auto_flush();
            part.write_at(0, b"abcd").unwrap(); // written but never flushed
        }
        let resumed = PartFile::resume(path, &validator(Some(8))).unwrap().unwrap();
        assert_eq!(resumed.downloaded(), 0);
    }

    #[test]
    fn unknown_size_streams_and_truncates_on_finalize() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f.part");
        let part = PartFile::create(path.clone(), validator(None)).unwrap();
        part.write_at(0, b"hello").unwrap();
        part.write_at(5, b" world").unwrap();
        assert!(PartFile::resume(path, &validator(None)).unwrap().is_none());
        let out = part.finalize(dir.path(), "h.txt").unwrap();
        assert_eq!(std::fs::read(out).unwrap(), b"hello world");
    }

    #[test]
    fn rejects_writes_past_known_size() {
        let dir = tempfile::tempdir().unwrap();
        let part = PartFile::create(dir.path().join("f.part"), validator(Some(4))).unwrap();
        assert_eq!(part.write_at(2, b"abc").unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn discard_removes_both_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f.part");
        PartFile::create(path.clone(), validator(Some(4))).unwrap();
        assert!(sidecar_path(&path).exists());
        PartFile::discard(&path).unwrap();
        assert!(!path.exists() && !sidecar_path(&path).exists());
        PartFile::discard(&path).unwrap(); // idempotent
    }
}
