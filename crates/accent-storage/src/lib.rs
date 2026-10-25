use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryJournalEntry {
    pub target_path: Option<PathBuf>,
    pub content: String,
    pub cursor_offset: usize,
    pub dirty: bool,
}

/// Atomically writes `content` to `path` via write-temp -> sync_all -> rename on the same filesystem.
pub fn atomic_save(path: &Path, content: &str) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("document.md");
    let tmp_path = parent.join(format!(".{file_name}.accent.tmp"));

    {
        let mut tmp_file = File::create(&tmp_path)?;
        tmp_file.write_all(content.as_bytes())?;
        tmp_file.sync_all()?;
    }

    fs::rename(&tmp_path, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_save_roundtrip() {
        let dir = std::env::temp_dir();
        let target = dir.join("accent_atomic_test.md");
        atomic_save(&target, "# Hello Accent\n").expect("atomic save failed");
        let read_back = fs::read_to_string(&target).expect("read back failed");
        assert_eq!(read_back, "# Hello Accent\n");
        let _ = fs::remove_file(&target);
    }
}
