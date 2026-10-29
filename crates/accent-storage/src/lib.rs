use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Record persisted in the Write-Ahead Recovery Journal (`SPECS.md` Pillar 8 / Phase 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryJournalEntry {
    pub session_id: String,
    pub target_path: Option<PathBuf>,
    pub document_title: String,
    pub content: String,
    pub cursor_char_offset: usize,
    pub cursor_byte_offset: usize,
    pub timestamp_utc_ms: u64,
    pub original_file_hash: Option<String>,
    pub dirty: bool,
}

impl RecoveryJournalEntry {
    pub fn new(
        target_path: Option<PathBuf>,
        document_title: String,
        content: String,
        cursor_char_offset: usize,
        cursor_byte_offset: usize,
        original_file_hash: Option<String>,
        dirty: bool,
    ) -> Self {
        let session_id = journal_session_id_for_path(target_path.as_deref());
        Self {
            session_id,
            target_path,
            document_title,
            content,
            cursor_char_offset,
            cursor_byte_offset,
            timestamp_utc_ms: current_utc_ms(),
            original_file_hash,
            dirty,
        }
    }
}

/// Returns the current UTC timestamp in milliseconds since `UNIX_EPOCH`.
pub fn current_utc_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Computes a deterministic 128-bit (dual-64-bit FNV-1a + Sip-mix) hex digest of `content`.
pub fn compute_content_hash(content: &str) -> String {
    let mut h1: u64 = 0xcbf2_9ce4_8422_2325;
    let mut h2: u64 = 0x8422_2325_cbf2_9ce4;
    for &b in content.as_bytes() {
        h1 ^= b as u64;
        h1 = h1.wrapping_mul(0x0000_0100_0000_01b3);
        h2 = h2.wrapping_add((b as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        h2 = h2.rotate_left(13) ^ h1;
    }
    format!("{h1:016x}{h2:016x}")
}

/// Canonicalizes or normalizes a file path so journal keys match regardless of relative/absolute invocation.
pub fn normalize_path(path: &Path) -> PathBuf {
    if let Ok(canon) = fs::canonicalize(path) {
        return canon;
    }
    if path.is_relative() {
        if let Ok(cwd) = std::env::current_dir() {
            let joined = cwd.join(path);
            if let Ok(canon) = fs::canonicalize(&joined) {
                return canon;
            }
            return joined;
        }
    }
    path.to_path_buf()
}

/// Deterministic session identifier for a target file path (or `"untitled"` when `None`).
pub fn journal_session_id_for_path(target_path: Option<&Path>) -> String {
    match target_path {
        Some(p) => {
            let norm = normalize_path(p);
            let hash = compute_content_hash(&norm.to_string_lossy());
            hash[..16].to_string()
        }
        None => "untitled".to_string(),
    }
}

/// Resolves the recovery journal directory (`ACCENT_RECOVERY_DIR` if set, otherwise XDG/user state dir).
pub fn default_recovery_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("ACCENT_RECOVERY_DIR") {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if let Ok(xdg) = std::env::var("XDG_STATE_HOME") {
        let trimmed = xdg.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("accent").join("recovery");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let trimmed = home.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed)
                .join(".local")
                .join("state")
                .join("accent")
                .join("recovery");
        }
    }
    std::env::temp_dir().join("accent-recovery")
}

/// Returns the journal file path inside `recovery_dir` for the given `target_path`.
pub fn journal_file_path(recovery_dir: &Path, target_path: Option<&Path>) -> PathBuf {
    let sid = journal_session_id_for_path(target_path);
    recovery_dir.join(format!("journal-{sid}.json"))
}

/// Atomically writes `content` to `path` via `write-temp -> fsync (sync_all) -> atomic rename -> parent dir sync`.
/// Guarantees zero file corruption and zero orphan `.accent-tmp-*` files on success.
pub fn atomic_save(path: &Path, content: &str) -> io::Result<()> {
    let parent = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    if !parent.exists() {
        fs::create_dir_all(parent)?;
    }

    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("document.md");
    let pid = std::process::id();
    let tmp_path = parent.join(format!(".{file_name}.accent-tmp-{pid}"));

    let write_res = (|| -> io::Result<()> {
        let mut tmp_file = File::create(&tmp_path)?;
        tmp_file.write_all(content.as_bytes())?;
        tmp_file.sync_all()?;
        Ok(())
    })();

    if let Err(err) = write_res {
        let _ = fs::remove_file(&tmp_path);
        return Err(err);
    }

    if let Err(err) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(err);
    }

    #[cfg(unix)]
    {
        if let Ok(dir_file) = File::open(parent) {
            let _ = dir_file.sync_all();
        }
    }

    Ok(())
}

/// Atomically persists a `RecoveryJournalEntry` into `recovery_dir`.
pub fn write_journal_entry(
    recovery_dir: &Path,
    entry: &RecoveryJournalEntry,
) -> io::Result<PathBuf> {
    fs::create_dir_all(recovery_dir)?;
    let jpath = recovery_dir.join(format!("journal-{}.json", entry.session_id));
    let serialized = serde_json::to_string_pretty(entry)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    atomic_save(&jpath, &serialized)?;
    Ok(jpath)
}

/// Removes the recovery journal entry associated with `target_path` from `recovery_dir`.
pub fn remove_journal_for_target(
    recovery_dir: &Path,
    target_path: Option<&Path>,
) -> io::Result<()> {
    let direct_path = journal_file_path(recovery_dir, target_path);
    if direct_path.exists() {
        let _ = fs::remove_file(&direct_path);
    }
    let norm_target = target_path.map(normalize_path);
    if let Ok(entries) = fs::read_dir(recovery_dir) {
        for dir_entry in entries.flatten() {
            let p = dir_entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(raw) = fs::read_to_string(&p) {
                if let Ok(parsed) = serde_json::from_str::<RecoveryJournalEntry>(&raw) {
                    let parsed_norm = parsed.target_path.as_deref().map(normalize_path);
                    if parsed_norm == norm_target {
                        let _ = fs::remove_file(&p);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Clears all recovery journal files inside `recovery_dir`.
pub fn clear_all_journals(recovery_dir: &Path) -> io::Result<()> {
    if !recovery_dir.exists() {
        return Ok(());
    }
    for dir_entry in fs::read_dir(recovery_dir)?.flatten() {
        let p = dir_entry.path();
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if (name.starts_with("journal-") && name.ends_with(".json"))
            || name.contains(".accent-tmp-")
        {
            let _ = fs::remove_file(&p);
        }
    }
    Ok(())
}

/// Scans `recovery_dir` and returns all dirty `RecoveryJournalEntry` records sorted newest-first.
pub fn scan_recovery_journals(recovery_dir: &Path) -> Vec<RecoveryJournalEntry> {
    let mut found = Vec::new();
    let Ok(read_dir) = fs::read_dir(recovery_dir) else {
        return found;
    };
    for dir_entry in read_dir.flatten() {
        let path = dir_entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.starts_with("journal-") || !name.ends_with(".json") {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(entry) = serde_json::from_str::<RecoveryJournalEntry>(&raw) {
                if entry.dirty {
                    found.push(entry);
                }
            }
        }
    }
    found.sort_by(|a, b| b.timestamp_utc_ms.cmp(&a.timestamp_utc_ms));
    found
}

/// Looks up a dirty recovery journal entry matching `target_path` inside `recovery_dir`.
pub fn find_recovery_journal_for_path(
    recovery_dir: &Path,
    target_path: Option<&Path>,
) -> Option<RecoveryJournalEntry> {
    let direct_path = journal_file_path(recovery_dir, target_path);
    if let Ok(raw) = fs::read_to_string(&direct_path) {
        if let Ok(entry) = serde_json::from_str::<RecoveryJournalEntry>(&raw) {
            if entry.dirty {
                return Some(entry);
            }
        }
    }
    let norm_target = target_path.map(normalize_path);
    scan_recovery_journals(recovery_dir)
        .into_iter()
        .find(|entry| entry.target_path.as_deref().map(normalize_path) == norm_target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_save_roundtrip_and_no_orphan_tmp_files() {
        let test_dir = std::env::temp_dir().join(format!(
            "accent_atomic_test_{}_{}",
            std::process::id(),
            current_utc_ms()
        ));
        fs::create_dir_all(&test_dir).expect("create test_dir");
        let target = test_dir.join("documento_atomico.md");

        let payload = "# Salvataggio Atomico\n\nContenuto UTF-8: Città perché — **parola**\n";
        atomic_save(&target, payload).expect("atomic save failed");

        let read_back = fs::read_to_string(&target).expect("read back failed");
        assert_eq!(read_back, payload);

        // Verify no orphan .accent-tmp-* files remain in the directory
        let orphans: Vec<PathBuf> = fs::read_dir(&test_dir)
            .expect("read_dir")
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.contains(".accent-tmp-"))
                    .unwrap_or(false)
            })
            .collect();
        assert!(
            orphans.is_empty(),
            "Found unexpected orphan temp files: {:?}",
            orphans
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_recovery_journal_write_scan_restore_and_cleanup_lifecycle() {
        let test_dir = std::env::temp_dir().join(format!(
            "accent_wal_test_{}_{}",
            std::process::id(),
            current_utc_ms()
        ));
        let recovery_dir = test_dir.join("recovery");
        fs::create_dir_all(&recovery_dir).expect("create recovery_dir");

        let doc_path = test_dir.join("note_crash.md");
        let original_text = "# Titolo Originale\n";
        atomic_save(&doc_path, original_text).expect("save initial file");
        let orig_hash = compute_content_hash(original_text);

        // Write a dirty WAL journal entry simulating unsaved edits before a crash
        let unsaved_text = "# Titolo Originale\n\nFrase scritta prima del crash improvviso";
        let entry = RecoveryJournalEntry::new(
            Some(doc_path.clone()),
            "note_crash.md".to_string(),
            unsaved_text.to_string(),
            unsaved_text.chars().count(),
            unsaved_text.len(),
            Some(orig_hash.clone()),
            true,
        );
        let jpath = write_journal_entry(&recovery_dir, &entry).expect("write journal");
        assert!(jpath.exists());

        // Original disk file must remain untouched before explicit save
        assert_eq!(
            fs::read_to_string(&doc_path).expect("read original"),
            original_text
        );

        // Simulate startup scan & path lookup
        let scanned = scan_recovery_journals(&recovery_dir);
        assert_eq!(scanned.len(), 1);
        let found = find_recovery_journal_for_path(&recovery_dir, Some(&doc_path))
            .expect("must find journal for doc_path");
        assert_eq!(found.content, unsaved_text);
        assert_eq!(found.cursor_char_offset, unsaved_text.chars().count());
        assert_eq!(found.cursor_byte_offset, unsaved_text.len());
        assert_eq!(found.original_file_hash.as_deref(), Some(orig_hash.as_str()));
        assert!(found.dirty);

        // Simulate user saving the recovered document -> journal is cleaned up
        atomic_save(&doc_path, &found.content).expect("atomic save recovered content");
        remove_journal_for_target(&recovery_dir, Some(&doc_path)).expect("remove journal");
        assert!(find_recovery_journal_for_path(&recovery_dir, Some(&doc_path)).is_none());
        assert!(scan_recovery_journals(&recovery_dir).is_empty());

        let _ = fs::remove_dir_all(&test_dir);
    }
}
