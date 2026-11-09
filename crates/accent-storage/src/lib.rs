use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

/// Resolves the recovery journal directory (`ACCENT_RECOVERY_DIR` if set, otherwise XDG/Windows user state dir).
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
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        let trimmed = local_appdata.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("Accent").join("recovery");
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

/// Resolves the user settings configuration file path (`ACCENT_CONFIG_PATH` if set,
/// otherwise `$XDG_CONFIG_HOME/accent/settings.json`, `~/.config/accent/settings.json`,
/// or `%APPDATA%\Accent\settings.json` on Windows).
pub fn default_config_path() -> PathBuf {
    if let Ok(custom) = std::env::var("ACCENT_CONFIG_PATH") {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let trimmed = xdg.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("accent").join("settings.json");
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let trimmed = appdata.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("Accent").join("settings.json");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let trimmed = home.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed)
                .join(".config")
                .join("accent")
                .join("settings.json");
        }
    }
    std::env::temp_dir()
        .join("accent-config")
        .join("settings.json")
}

/// Returns the journal file path inside `recovery_dir` for the given `target_path`.
pub fn journal_file_path(recovery_dir: &Path, target_path: Option<&Path>) -> PathBuf {
    let sid = journal_session_id_for_path(target_path);
    recovery_dir.join(format!("journal-{sid}.json"))
}

/// Atomically writes `content` to `path` via `write-temp -> fsync (sync_all) -> atomic rename`.
/// Guarantees zero file corruption and zero orphan `.accent-tmp-*` files on success.
pub fn atomic_save(path: &Path, content: &str) -> io::Result<()> {
    atomic_save_bytes(path, content.as_bytes())
}

/// Atomically writes `content` to `path` via `write-temp -> flush -> atomic rename` without
/// blocking the GUI thread on a hardware disk controller `fsync` barrier.
/// Ideal for high-frequency internal state files (`settings.json` and 200ms debounced WAL journals).
pub fn atomic_save_fast(path: &Path, content: &str) -> io::Result<()> {
    atomic_save_bytes_inner(path, content.as_bytes(), false)
}

/// Atomically writes arbitrary `bytes` to `path` via `write-temp -> fsync (sync_all) -> atomic rename`.
pub fn atomic_save_bytes(path: &Path, bytes: &[u8]) -> io::Result<()> {
    atomic_save_bytes_inner(path, bytes, true)
}

fn atomic_save_bytes_inner(path: &Path, bytes: &[u8], sync_data: bool) -> io::Result<()> {
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
        tmp_file.write_all(bytes)?;
        if sync_data {
            tmp_file.sync_all()?;
        } else {
            tmp_file.flush()?;
        }
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
    atomic_save_fast(&jpath, &serialized)?;
    Ok(jpath)
}

/// Removes the recovery journal entry associated with `target_path` from `recovery_dir`.
pub fn remove_journal_for_target(
    recovery_dir: &Path,
    target_path: Option<&Path>,
) -> io::Result<()> {
    if !recovery_dir.exists() {
        return Ok(());
    }
    let direct_path = journal_file_path(recovery_dir, target_path);
    if direct_path.exists() {
        let _ = fs::remove_file(&direct_path);
        return Ok(());
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
                    if parsed.target_path.as_deref() == target_path
                        || parsed.target_path.as_deref().map(normalize_path) == norm_target
                    {
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

/// Cross-platform single-instance IPC message sent from a secondary `accent` launch
/// to the already-running primary Accent instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpcRequest {
    OpenFile(PathBuf),
    FocusWindow,
}

/// Result of attempting to acquire the single-instance lock on startup.
pub enum SingleInstanceOutcome {
    /// Current process is the primary instance and owns the active IPC listener.
    Primary(SingleInstanceGuard),
    /// An instance of Accent is already running and accepted our forwarded request.
    ForwardedToExisting,
}

/// Resolves the path to the single-instance endpoint lock file (`instance.lock`),
/// scoped to the user's state directory (or `ACCENT_INSTANCE_LOCK` if overridden).
pub fn default_instance_lock_path() -> PathBuf {
    if let Ok(custom) = std::env::var("ACCENT_INSTANCE_LOCK") {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    let rec_dir = default_recovery_dir();
    let state_dir = rec_dir.parent().unwrap_or(&rec_dir);
    state_dir.join("instance.lock")
}

/// Owns the localhost IPC listener for the running primary Accent instance.
/// When dropped on exit, stops the listener thread and removes `instance.lock`.
pub struct SingleInstanceGuard {
    lock_path: PathBuf,
    local_addr: SocketAddr,
    stop_flag: Arc<AtomicBool>,
    inbox: Arc<Mutex<Vec<IpcRequest>>>,
    waker: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>>,
}

impl SingleInstanceGuard {
    /// Attempts to connect to an already-running Accent instance recorded in `lock_path`.
    /// - If a running instance responds, forwards `initial_file` (or `FocusWindow`) and returns
    ///   `Ok(SingleInstanceOutcome::ForwardedToExisting)`.
    /// - Otherwise, binds a new localhost IPC listener, atomically records its address in `lock_path`,
    ///   and returns `Ok(SingleInstanceOutcome::Primary(guard))`.
    pub fn acquire_or_forward(
        lock_path: &Path,
        initial_file: Option<&Path>,
    ) -> io::Result<SingleInstanceOutcome> {
        let req = match initial_file {
            Some(p) => IpcRequest::OpenFile(normalize_path(p)),
            None => IpcRequest::FocusWindow,
        };

        if let Ok(raw) = fs::read_to_string(lock_path) {
            if let Ok(existing_addr) = raw.trim().parse::<SocketAddr>() {
                if let Ok(mut stream) =
                    TcpStream::connect_timeout(&existing_addr, Duration::from_millis(250))
                {
                    let _ = stream.set_write_timeout(Some(Duration::from_millis(250)));
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(250)));
                    if let Ok(mut payload) = serde_json::to_string(&req) {
                        payload.push('\n');
                        if stream.write_all(payload.as_bytes()).is_ok() && stream.flush().is_ok() {
                            let mut reader = BufReader::new(stream);
                            let mut ack = String::new();
                            if reader.read_line(&mut ack).is_ok() && ack.trim() == "OK" {
                                return Ok(SingleInstanceOutcome::ForwardedToExisting);
                            }
                        }
                    }
                }
            }
        }

        if let Some(parent) = lock_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let listener = TcpListener::bind("127.0.0.1:0")?;
        let local_addr = listener.local_addr()?;
        atomic_save(lock_path, &local_addr.to_string())?;

        let stop_flag = Arc::new(AtomicBool::new(false));
        let inbox = Arc::new(Mutex::new(Vec::new()));
        let waker: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>> = Arc::new(Mutex::new(None));

        let stop_clone = Arc::clone(&stop_flag);
        let inbox_clone = Arc::clone(&inbox);
        let waker_clone = Arc::clone(&waker);

        std::thread::Builder::new()
            .name("accent-single-instance-ipc".to_string())
            .spawn(move || {
                for incoming in listener.incoming() {
                    if stop_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    let Ok(mut stream) = incoming else {
                        continue;
                    };
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(250)));
                    let _ = stream.set_write_timeout(Some(Duration::from_millis(250)));
                    let mut line = String::new();
                    {
                        let mut reader = BufReader::new(&mut stream);
                        if reader.read_line(&mut line).is_err() {
                            continue;
                        }
                    }
                    if let Ok(msg) = serde_json::from_str::<IpcRequest>(line.trim()) {
                        let _ = stream.write_all(b"OK\n");
                        let _ = stream.flush();
                        if let Ok(mut queue) = inbox_clone.lock() {
                            queue.push(msg);
                        }
                        if let Ok(w_guard) = waker_clone.lock() {
                            if let Some(ref cb) = *w_guard {
                                cb();
                            }
                        }
                    }
                }
            })?;

        Ok(SingleInstanceOutcome::Primary(Self {
            lock_path: lock_path.to_path_buf(),
            local_addr,
            stop_flag,
            inbox,
            waker,
        }))
    }

    /// Registers a wakeup callback (e.g. `egui::Context::request_repaint`) invoked whenever
    /// a secondary instance forwards a command to this primary instance.
    pub fn set_waker<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if let Ok(mut guard) = self.waker.lock() {
            *guard = Some(Box::new(callback));
        }
    }

    /// Drains all pending `IpcRequest` messages received from secondary instances.
    pub fn drain_requests(&self) -> Vec<IpcRequest> {
        if let Ok(mut queue) = self.inbox.lock() {
            std::mem::take(&mut *queue)
        } else {
            Vec::new()
        }
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect_timeout(&self.local_addr, Duration::from_millis(50));
        if let Ok(raw) = fs::read_to_string(&self.lock_path) {
            if raw.trim() == self.local_addr.to_string() {
                let _ = fs::remove_file(&self.lock_path);
            }
        }
    }
}

/// OS-driven file modification watcher for the currently focused document tab.
///
/// - **Linux (`#[cfg(target_os = "linux")]`)**: Uses kernel `inotify` (`inotify_init1`,
///   `inotify_add_watch`, `poll`) with a self-pipe wakeup channel. Blocks indefinitely in
///   `poll(..., -1)` with 0% CPU and zero disk polling until the kernel reports an in-place
///   write (`IN_CLOSE_WRITE`, `IN_MODIFY`) or atomic rename (`IN_MOVED_TO`, `IN_CREATE`)
///   targeting the watched file.
/// - **Windows (`#[cfg(target_os = "windows")]`)**: Uses Win32 kernel change notifications
///   (`FindFirstChangeNotificationW`, `FindNextChangeNotification`, `WaitForMultipleObjects`)
///   plus a Win32 event object (`CreateEventW` / `SetEvent`) to block with 0% CPU and zero
///   continuous polling until NTFS/ReFS notifies a modification in the parent directory.
pub struct FocusedFileWatcher {
    stop_flag: Arc<AtomicBool>,
    changed_flag: Arc<AtomicBool>,
    suppress_until_utc_ms: Arc<std::sync::atomic::AtomicU64>,
    watched_path: Arc<Mutex<Option<PathBuf>>>,
    waker: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>>,
    #[cfg(target_os = "linux")]
    wake_write_fd: i32,
    #[cfg(target_os = "windows")]
    wake_event_handle: isize,
}

#[cfg(target_os = "linux")]
mod linux_inotify {
    use super::*;
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    const IN_NONBLOCK: i32 = 0o0004000;
    const IN_CLOEXEC: i32 = 0o2000000;
    const O_NONBLOCK: i32 = 0o0004000;
    const O_CLOEXEC: i32 = 0o2000000;

    const IN_MODIFY: u32 = 0x0000_0002;
    const IN_ATTRIB: u32 = 0x0000_0004;
    const IN_CLOSE_WRITE: u32 = 0x0000_0008;
    const IN_MOVED_TO: u32 = 0x0000_0080;
    const IN_CREATE: u32 = 0x0000_0100;
    const IN_DELETE_SELF: u32 = 0x0000_0400;
    const IN_MOVE_SELF: u32 = 0x0000_0800;

    const DIR_WATCH_MASK: u32 = IN_CLOSE_WRITE | IN_MOVED_TO | IN_CREATE | IN_MODIFY | IN_ATTRIB;
    const FILE_WATCH_MASK: u32 =
        IN_CLOSE_WRITE | IN_MODIFY | IN_ATTRIB | IN_DELETE_SELF | IN_MOVE_SELF;

    const POLLIN: i16 = 0x0001;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct PollFd {
        fd: i32,
        events: i16,
        revents: i16,
    }

    #[repr(C)]
    struct InotifyEventHeader {
        wd: i32,
        mask: u32,
        cookie: u32,
        len: u32,
    }

    extern "C" {
        fn inotify_init1(flags: i32) -> i32;
        fn inotify_add_watch(fd: i32, pathname: *const i8, mask: u32) -> i32;
        fn inotify_rm_watch(fd: i32, wd: i32) -> i32;
        fn pipe2(pipefd: *mut i32, flags: i32) -> i32;
        fn poll(fds: *mut PollFd, nfds: usize, timeout: i32) -> i32;
        fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
        fn write(fd: i32, buf: *const u8, count: usize) -> isize;
        fn close(fd: i32) -> i32;
    }

    pub fn spawn_linux_watcher(
        stop_flag: Arc<AtomicBool>,
        changed_flag: Arc<AtomicBool>,
        watched_path: Arc<Mutex<Option<PathBuf>>>,
        waker: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>>,
    ) -> io::Result<i32> {
        let inotify_fd = unsafe { inotify_init1(IN_NONBLOCK | IN_CLOEXEC) };
        if inotify_fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let mut pipe_fds = [-1i32; 2];
        if unsafe { pipe2(pipe_fds.as_mut_ptr(), O_NONBLOCK | O_CLOEXEC) } < 0 {
            let err = io::Error::last_os_error();
            unsafe {
                close(inotify_fd);
            }
            return Err(err);
        }
        let wake_read_fd = pipe_fds[0];
        let wake_write_fd = pipe_fds[1];

        std::thread::Builder::new()
            .name("accent-file-watcher".to_string())
            .spawn(move || {
                let mut dir_wd: i32 = -1;
                let mut file_wd: i32 = -1;
                let mut active_target: Option<PathBuf> = None;
                let mut active_file_name: Option<Vec<u8>> = None;

                let rearm_watches = |inotify_fd: i32,
                                     dir_wd: &mut i32,
                                     file_wd: &mut i32,
                                     target: &Option<PathBuf>,
                                     file_name_out: &mut Option<Vec<u8>>| {
                    if *dir_wd >= 0 {
                        unsafe {
                            inotify_rm_watch(inotify_fd, *dir_wd);
                        }
                        *dir_wd = -1;
                    }
                    if *file_wd >= 0 {
                        unsafe {
                            inotify_rm_watch(inotify_fd, *file_wd);
                        }
                        *file_wd = -1;
                    }
                    *file_name_out = None;
                    if let Some(ref p) = target {
                        let norm = normalize_path(p);
                        if let Some(fname) = norm.file_name() {
                            *file_name_out = Some(fname.as_bytes().to_vec());
                        }
                        if let Some(parent) = norm.parent() {
                            if let Ok(c_dir) = CString::new(parent.as_os_str().as_bytes()) {
                                *dir_wd = unsafe {
                                    inotify_add_watch(inotify_fd, c_dir.as_ptr(), DIR_WATCH_MASK)
                                };
                            }
                        }
                        if let Ok(c_file) = CString::new(norm.as_os_str().as_bytes()) {
                            *file_wd = unsafe {
                                inotify_add_watch(inotify_fd, c_file.as_ptr(), FILE_WATCH_MASK)
                            };
                        }
                    }
                };

                let drain_wake_pipe = |fd: i32| {
                    let mut sink = [0u8; 64];
                    while unsafe { read(fd, sink.as_mut_ptr(), sink.len()) } > 0 {}
                };

                // Initial arm if a path was already set
                if let Ok(g) = watched_path.lock() {
                    active_target = g.clone();
                }
                rearm_watches(
                    inotify_fd,
                    &mut dir_wd,
                    &mut file_wd,
                    &active_target,
                    &mut active_file_name,
                );

                let mut event_buf = [0u8; 4096];
                let mut coalesce_pending = false;

                loop {
                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }
                    let mut fds = [
                        PollFd {
                            fd: wake_read_fd,
                            events: POLLIN,
                            revents: 0,
                        },
                        PollFd {
                            fd: inotify_fd,
                            events: POLLIN,
                            revents: 0,
                        },
                    ];
                    let timeout_ms = if coalesce_pending { 35 } else { -1 };
                    let rc = unsafe { poll(fds.as_mut_ptr(), fds.len(), timeout_ms) };
                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }
                    if rc < 0 {
                        let err = io::Error::last_os_error();
                        if err.kind() == io::ErrorKind::Interrupted {
                            continue;
                        }
                        break;
                    }
                    if rc == 0 && coalesce_pending {
                        // Coalescing window elapsed after a relevant file modification event
                        coalesce_pending = false;
                        // Re-arm file watch in case an atomic rename replaced the inode
                        if let Some(ref p) = active_target {
                            let norm = normalize_path(p);
                            if let Ok(c_file) = CString::new(norm.as_os_str().as_bytes()) {
                                if file_wd >= 0 {
                                    unsafe {
                                        inotify_rm_watch(inotify_fd, file_wd);
                                    }
                                }
                                file_wd = unsafe {
                                    inotify_add_watch(inotify_fd, c_file.as_ptr(), FILE_WATCH_MASK)
                                };
                            }
                        }
                        changed_flag.store(true, Ordering::SeqCst);
                        if let Ok(w_guard) = waker.lock() {
                            if let Some(ref cb) = *w_guard {
                                cb();
                            }
                        }
                        continue;
                    }

                    // Check if the watched path changed via wake pipe
                    if (fds[0].revents & POLLIN) != 0 {
                        drain_wake_pipe(wake_read_fd);
                        if stop_flag.load(Ordering::Relaxed) {
                            break;
                        }
                        let new_target = watched_path.lock().ok().and_then(|g| g.clone());
                        if new_target != active_target {
                            active_target = new_target;
                            coalesce_pending = false;
                            rearm_watches(
                                inotify_fd,
                                &mut dir_wd,
                                &mut file_wd,
                                &active_target,
                                &mut active_file_name,
                            );
                        }
                    }

                    // Drain inotify events and check if any match our target file
                    if (fds[1].revents & POLLIN) != 0 {
                        loop {
                            let n =
                                unsafe { read(inotify_fd, event_buf.as_mut_ptr(), event_buf.len()) };
                            if n <= 0 {
                                break;
                            }
                            let n_usize = n as usize;
                            let hdr_size = std::mem::size_of::<InotifyEventHeader>();
                            let mut offset = 0usize;
                            while offset + hdr_size <= n_usize {
                                let hdr = unsafe {
                                    std::ptr::read_unaligned(
                                        event_buf.as_ptr().add(offset) as *const InotifyEventHeader
                                    )
                                };
                                let name_start = offset + hdr_size;
                                let name_end = (name_start + (hdr.len as usize)).min(n_usize);
                                offset = name_end;

                                if hdr.wd == file_wd && file_wd >= 0 {
                                    coalesce_pending = true;
                                } else if hdr.wd == dir_wd && dir_wd >= 0 && hdr.len > 0 {
                                    let raw_name = &event_buf[name_start..name_end];
                                    let trimmed_len = raw_name
                                        .iter()
                                        .position(|&b| b == 0)
                                        .unwrap_or(raw_name.len());
                                    let ev_name = &raw_name[..trimmed_len];
                                    if let Some(ref wanted_name) = active_file_name {
                                        if ev_name == wanted_name.as_slice() {
                                            coalesce_pending = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if dir_wd >= 0 {
                    unsafe {
                        inotify_rm_watch(inotify_fd, dir_wd);
                    }
                }
                if file_wd >= 0 {
                    unsafe {
                        inotify_rm_watch(inotify_fd, file_wd);
                    }
                }
                unsafe {
                    close(wake_read_fd);
                    close(inotify_fd);
                }
            })?;

        Ok(wake_write_fd)
    }

    pub fn signal_wake_pipe(wake_write_fd: i32) {
        if wake_write_fd >= 0 {
            let b = [1u8; 1];
            unsafe {
                let _ = write(wake_write_fd, b.as_ptr(), 1);
            }
        }
    }

    pub fn close_wake_pipe(wake_write_fd: i32) {
        if wake_write_fd >= 0 {
            unsafe {
                close(wake_write_fd);
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod windows_watcher {
    use super::*;
    use std::os::windows::ffi::OsStrExt;

    const FILE_NOTIFY_CHANGE_FILE_NAME: u32 = 0x0000_0001;
    const FILE_NOTIFY_CHANGE_SIZE: u32 = 0x0000_0008;
    const FILE_NOTIFY_CHANGE_LAST_WRITE: u32 = 0x0000_0010;
    const INVALID_HANDLE_VALUE: isize = -1;
    const WAIT_OBJECT_0: u32 = 0x0000_0000;
    const WAIT_TIMEOUT: u32 = 0x0000_0102;
    const INFINITE: u32 = 0xFFFF_FFFF;

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateEventW(
            lpEventAttributes: *const std::ffi::c_void,
            bManualReset: i32,
            bInitialState: i32,
            lpName: *const u16,
        ) -> isize;
        fn SetEvent(hEvent: isize) -> i32;
        fn ResetEvent(hEvent: isize) -> i32;
        fn CloseHandle(hObject: isize) -> i32;
        fn FindFirstChangeNotificationW(
            lpPathName: *const u16,
            bWatchSubtree: i32,
            dwNotifyFilter: u32,
        ) -> isize;
        fn FindNextChangeNotification(hChangeHandle: isize) -> i32;
        fn FindCloseChangeNotification(hChangeHandle: isize) -> i32;
        fn WaitForMultipleObjects(
            nCount: u32,
            lpHandles: *const isize,
            bWaitAll: i32,
            dwMilliseconds: u32,
        ) -> u32;
    }

    pub fn spawn_windows_watcher(
        stop_flag: Arc<AtomicBool>,
        changed_flag: Arc<AtomicBool>,
        watched_path: Arc<Mutex<Option<PathBuf>>>,
        waker: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>>,
    ) -> io::Result<isize> {
        let wake_event = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
        if wake_event == 0 || wake_event == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }

        std::thread::Builder::new()
            .name("accent-file-watcher-win32".to_string())
            .spawn(move || {
                let mut change_handle: isize = INVALID_HANDLE_VALUE;
                let mut active_target: Option<PathBuf> = None;

                let rearm_win32 = |change_handle: &mut isize, target: &Option<PathBuf>| {
                    if *change_handle != INVALID_HANDLE_VALUE && *change_handle != 0 {
                        unsafe {
                            FindCloseChangeNotification(*change_handle);
                        }
                        *change_handle = INVALID_HANDLE_VALUE;
                    }
                    if let Some(ref p) = target {
                        let norm = normalize_path(p);
                        if let Some(parent) = norm.parent() {
                            let wide: Vec<u16> = parent
                                .as_os_str()
                                .encode_wide()
                                .chain(std::iter::once(0))
                                .collect();
                            let h = unsafe {
                                FindFirstChangeNotificationW(
                                    wide.as_ptr(),
                                    0,
                                    FILE_NOTIFY_CHANGE_FILE_NAME
                                        | FILE_NOTIFY_CHANGE_SIZE
                                        | FILE_NOTIFY_CHANGE_LAST_WRITE,
                                )
                            };
                            if h != 0 && h != INVALID_HANDLE_VALUE {
                                *change_handle = h;
                            }
                        }
                    }
                };

                if let Ok(g) = watched_path.lock() {
                    active_target = g.clone();
                }
                rearm_win32(&mut change_handle, &active_target);
                let mut coalesce_pending = false;

                loop {
                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }
                    let timeout_ms = if coalesce_pending { 35 } else { INFINITE };
                    let wait_rc = if change_handle != INVALID_HANDLE_VALUE && change_handle != 0 {
                        let handles = [wake_event, change_handle];
                        unsafe { WaitForMultipleObjects(2, handles.as_ptr(), 0, timeout_ms) }
                    } else {
                        let handles = [wake_event];
                        unsafe { WaitForMultipleObjects(1, handles.as_ptr(), 0, timeout_ms) }
                    };

                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }
                    if wait_rc == WAIT_TIMEOUT && coalesce_pending {
                        coalesce_pending = false;
                        changed_flag.store(true, Ordering::SeqCst);
                        if let Ok(w_guard) = waker.lock() {
                            if let Some(ref cb) = *w_guard {
                                cb();
                            }
                        }
                        continue;
                    }
                    if wait_rc == WAIT_OBJECT_0 {
                        unsafe {
                            ResetEvent(wake_event);
                        }
                        if stop_flag.load(Ordering::Relaxed) {
                            break;
                        }
                        let new_target = watched_path.lock().ok().and_then(|g| g.clone());
                        if new_target != active_target {
                            active_target = new_target;
                            coalesce_pending = false;
                            rearm_win32(&mut change_handle, &active_target);
                        }
                    } else if wait_rc == WAIT_OBJECT_0 + 1 {
                        if change_handle != INVALID_HANDLE_VALUE && change_handle != 0 {
                            unsafe {
                                FindNextChangeNotification(change_handle);
                            }
                        }
                        coalesce_pending = true;
                    } else {
                        break;
                    }
                }

                if change_handle != INVALID_HANDLE_VALUE && change_handle != 0 {
                    unsafe {
                        FindCloseChangeNotification(change_handle);
                    }
                }
                unsafe {
                    CloseHandle(wake_event);
                }
            })?;

        Ok(wake_event)
    }

    pub fn signal_wake_event(wake_event: isize) {
        if wake_event != 0 && wake_event != INVALID_HANDLE_VALUE {
            unsafe {
                SetEvent(wake_event);
            }
        }
    }
}

impl FocusedFileWatcher {
    /// Creates and starts an OS-driven file watcher for the active document tab.
    pub fn new() -> io::Result<Self> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let changed_flag = Arc::new(AtomicBool::new(false));
        let suppress_until_utc_ms = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let watched_path = Arc::new(Mutex::new(None));
        let waker: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>> = Arc::new(Mutex::new(None));

        #[cfg(target_os = "linux")]
        let wake_write_fd = linux_inotify::spawn_linux_watcher(
            Arc::clone(&stop_flag),
            Arc::clone(&changed_flag),
            Arc::clone(&watched_path),
            Arc::clone(&waker),
        )?;

        #[cfg(target_os = "windows")]
        let wake_event_handle = windows_watcher::spawn_windows_watcher(
            Arc::clone(&stop_flag),
            Arc::clone(&changed_flag),
            Arc::clone(&watched_path),
            Arc::clone(&waker),
        )?;

        Ok(Self {
            stop_flag,
            changed_flag,
            suppress_until_utc_ms,
            watched_path,
            waker,
            #[cfg(target_os = "linux")]
            wake_write_fd,
            #[cfg(target_os = "windows")]
            wake_event_handle,
        })
    }

    /// Suppresses self-triggered file watcher notifications for `duration_ms` milliseconds
    /// (called right when Accent saves the currently watched file to disk).
    pub fn suppress_notifications_for(&self, duration_ms: u64) {
        let until = current_utc_ms().saturating_add(duration_ms);
        self.suppress_until_utc_ms.store(until, Ordering::SeqCst);
        self.changed_flag.store(false, Ordering::SeqCst);
    }

    /// Registers a wakeup callback (e.g. `egui::Context::request_repaint`) invoked whenever
    /// the operating system notifies that the watched file has been modified on disk.
    pub fn set_waker<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if let Ok(mut guard) = self.waker.lock() {
            *guard = Some(Box::new(callback));
        }
    }

    /// Updates the file path currently being watched (called when the user switches the focused tab,
    /// opens a document, or saves a new file). Immediately wakes the kernel watcher thread to re-arm.
    pub fn set_watched_file(&self, path: Option<PathBuf>) {
        let normalized = path.map(|p| normalize_path(&p));
        if let Ok(mut guard) = self.watched_path.lock() {
            if *guard == normalized {
                return;
            }
            *guard = normalized;
        }
        self.changed_flag.store(false, Ordering::SeqCst);
        #[cfg(target_os = "linux")]
        linux_inotify::signal_wake_pipe(self.wake_write_fd);
        #[cfg(target_os = "windows")]
        windows_watcher::signal_wake_event(self.wake_event_handle);
    }

    /// Returns `true` (and clears the notification flag) if the operating system reported a
    /// modification event on the watched file since the last check.
    pub fn take_change_notification(&self) -> bool {
        let had = self.changed_flag.swap(false, Ordering::SeqCst);
        if !had {
            return false;
        }
        if current_utc_ms() < self.suppress_until_utc_ms.load(Ordering::SeqCst) {
            return false;
        }
        true
    }
}

impl Drop for FocusedFileWatcher {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        #[cfg(target_os = "linux")]
        {
            linux_inotify::signal_wake_pipe(self.wake_write_fd);
            linux_inotify::close_wake_pipe(self.wake_write_fd);
        }
        #[cfg(target_os = "windows")]
        {
            windows_watcher::signal_wake_event(self.wake_event_handle);
        }
    }
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

    #[test]
    fn test_single_instance_lock_and_ipc_file_forwarding() {
        let test_dir = std::env::temp_dir().join(format!(
            "accent_single_instance_test_{}_{}",
            std::process::id(),
            current_utc_ms()
        ));
        fs::create_dir_all(&test_dir).expect("create test_dir");
        let lock_path = test_dir.join("instance.lock");
        let doc_a = test_dir.join("secondo_file.md");
        atomic_save(&doc_a, "# Secondo File\n").expect("create doc_a");

        // 1. Write a stale unlistening port to lock_path to simulate a crashed previous instance
        atomic_save(&lock_path, "127.0.0.1:9").expect("write stale lock");

        // 2. First live instance must recover from stale lock and acquire Primary
        let primary = match SingleInstanceGuard::acquire_or_forward(&lock_path, None)
            .expect("acquire primary")
        {
            SingleInstanceOutcome::Primary(guard) => guard,
            SingleInstanceOutcome::ForwardedToExisting => {
                panic!("Expected first live instance to become Primary")
            }
        };

        let woke_up = Arc::new(AtomicBool::new(false));
        let woke_clone = Arc::clone(&woke_up);
        primary.set_waker(move || {
            woke_clone.store(true, Ordering::SeqCst);
        });

        // 3. Second instance launched with `doc_a` must forward to primary and return `ForwardedToExisting`
        let second = SingleInstanceGuard::acquire_or_forward(&lock_path, Some(&doc_a))
            .expect("second launch");
        assert!(matches!(second, SingleInstanceOutcome::ForwardedToExisting));

        // 4. Third instance launched without arguments must forward `FocusWindow` and return `ForwardedToExisting`
        let third =
            SingleInstanceGuard::acquire_or_forward(&lock_path, None).expect("third launch");
        assert!(matches!(third, SingleInstanceOutcome::ForwardedToExisting));

        // Wait briefly for the background IPC thread to process both messages
        for _ in 0..20 {
            if woke_up.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(woke_up.load(Ordering::SeqCst));

        let requests = primary.drain_requests();
        assert_eq!(
            requests,
            vec![
                IpcRequest::OpenFile(normalize_path(&doc_a)),
                IpcRequest::FocusWindow,
            ]
        );

        // 5. Dropping the primary instance cleans up `instance.lock` and allows a new primary instance
        drop(primary);
        assert!(!lock_path.exists());

        let new_primary =
            SingleInstanceGuard::acquire_or_forward(&lock_path, None).expect("re-acquire primary");
        assert!(matches!(new_primary, SingleInstanceOutcome::Primary(_)));

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_focused_file_watcher_os_notification_detects_in_place_and_atomic_rename_writes() {
        let test_dir = std::env::temp_dir().join(format!(
            "accent_file_watcher_test_{}_{}",
            std::process::id(),
            current_utc_ms()
        ));
        fs::create_dir_all(&test_dir).expect("create test_dir");
        let file_a = test_dir.join("watched_a.md");
        let file_b = test_dir.join("watched_b.md");
        let unrelated = test_dir.join("other_file.md");
        atomic_save(&file_a, "# Versione 1\n").expect("save file_a");
        atomic_save(&file_b, "# File B v1\n").expect("save file_b");

        let watcher = FocusedFileWatcher::new().expect("create FocusedFileWatcher");
        let woke_count = Arc::new( std::sync::atomic::AtomicUsize::new(0));
        let woke_clone = Arc::clone(&woke_count);
        watcher.set_waker(move || {
            woke_clone.fetch_add(1, Ordering::SeqCst);
        });

        watcher.set_watched_file(Some(file_a.clone()));
        // Give the watcher thread a few ms to arm the kernel inotify/Win32 watch
        std::thread::sleep(Duration::from_millis(25));
        assert!(!watcher.take_change_notification());

        // 1. Modifying an unrelated file in the same directory must NOT trigger a notification on Linux
        #[cfg(target_os = "linux")]
        {
            fs::write(&unrelated, "unrelated modification").expect("write unrelated");
            std::thread::sleep(Duration::from_millis(60));
            assert!(
                !watcher.take_change_notification(),
                "Unrelated file write in parent directory must not trigger watched file notification"
            );
        }

        // 2. Direct in-place write (`fs::write`) on `file_a` must trigger OS notification
        fs::write(&file_a, "# Versione 2 (in-place write)\n").expect("in-place write");
        let mut notified_in_place = false;
        for _ in 0..40 {
            if watcher.take_change_notification() {
                notified_in_place = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            notified_in_place,
            "OS file watcher must detect in-place modification without polling"
        );
        assert!(woke_count.load(Ordering::SeqCst) >= 1);

        // 3. Atomic save (`write-temp -> rename`) on `file_a` must also trigger OS notification
        atomic_save(&file_a, "# Versione 3 (atomic rename)\n").expect("atomic save");
        let mut notified_atomic = false;
        for _ in 0..40 {
            if watcher.take_change_notification() {
                notified_atomic = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            notified_atomic,
            "OS file watcher must detect atomic rename modification"
        );

        // 4. Switching watched file to `file_b` (simulating tab switch) must watch `file_b`
        watcher.set_watched_file(Some(file_b.clone()));
        std::thread::sleep(Duration::from_millis(25));
        atomic_save(&file_b, "# File B v2 (modified externally)\n").expect("modify file_b");
        let mut notified_b = false;
        for _ in 0..40 {
            if watcher.take_change_notification() {
                notified_b = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            notified_b,
            "OS file watcher must detect modification after switching watched tab file"
        );

        drop(watcher);
        let _ = fs::remove_dir_all(&test_dir);
    }
}
