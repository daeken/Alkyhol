//! Real directory descriptors and kqueue-backed overlapped notifications.
//! Pending buffers follow Win32's exclusive-lifetime contract until completion.
use super::{LoadedImage32, files32, unsupported};
use alkyhol::{
    ffi,
    kernel::{EventNotifier, kernel},
};
use std::{
    collections::BTreeMap,
    fs::{File, Metadata, OpenOptions},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};
#[repr(C)]
struct Kevent {
    ident: usize,
    filter: i16,
    flags: u16,
    fflags: u32,
    data: isize,
    udata: usize,
}
#[repr(C)]
struct Timespec {
    seconds: i64,
    nanoseconds: i64,
}
unsafe extern "C" {
    fn kqueue() -> i32;
    fn kevent(
        fd: i32,
        changes: *const Kevent,
        nchanges: i32,
        events: *mut Kevent,
        nevents: i32,
        timeout: *const Timespec,
    ) -> i32;
}
#[derive(Clone, PartialEq, Eq)]
struct Node {
    id: (u64, u64),
    directory: bool,
    size: u64,
    mode: u32,
    uid: u32,
    gid: u32,
    write: (i64, i64),
    access: (i64, i64),
    created: Option<std::time::SystemTime>,
}
impl Node {
    fn new(m: &Metadata) -> Self {
        Self {
            id: (m.dev(), m.ino()),
            directory: m.is_dir(),
            size: m.len(),
            mode: m.mode(),
            uid: m.uid(),
            gid: m.gid(),
            write: (m.mtime(), m.mtime_nsec()),
            access: (m.atime(), m.atime_nsec()),
            created: m.created().ok(),
        }
    }
}
type Snapshot = BTreeMap<String, Node>;
struct Monitor {
    queue: File,
    _files: Vec<File>,
    snapshot: Snapshot,
}
fn scan(root: &Path, tree: bool) -> std::io::Result<Snapshot> {
    fn visit(root: &Path, relative: &str, tree: bool, out: &mut Snapshot) -> std::io::Result<()> {
        if out.len() > 32768 {
            return Err(std::io::Error::other(
                "directory notification snapshot budget exceeded",
            ));
        }
        for entry in std::fs::read_dir(root.join(relative.replace('\\', "/")))? {
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| std::io::Error::other("non-Unicode watched name"))?;
            if !name.is_ascii() || name.contains(['\\', ':']) {
                return Err(std::io::Error::other(
                    "watched names require the bounded ASCII namespace",
                ));
            }
            let name = if relative.is_empty() {
                name
            } else {
                format!("{relative}\\{name}")
            };
            let meta = std::fs::symlink_metadata(entry.path())?;
            if meta.is_symlink() || !(meta.is_file() || meta.is_dir()) {
                return Err(std::io::Error::other(
                    "watched aliases/special files are unsupported",
                ));
            }
            out.insert(name.clone(), Node::new(&meta));
            if tree && meta.is_dir() {
                visit(root, &name, tree, out)?;
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    visit(root, "", tree, &mut out)?;
    Ok(out)
}
impl Monitor {
    fn arm(root: &Path, tree: bool, filter: u32) -> std::io::Result<Self> {
        let snapshot = scan(root, tree)?;
        let fd = unsafe { kqueue() };
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        let queue = unsafe { File::from_raw_fd(fd) };
        let mut files = Vec::new();
        let mut paths = vec![root.to_path_buf()];
        for (name, node) in &snapshot {
            if node.directory || filter & !3 != 0 {
                paths.push(root.join(name.replace('\\', "/")));
            }
        }
        if paths.len() > 8192 {
            return Err(std::io::Error::other(
                "native watch descriptor budget exceeded",
            ));
        }
        let mut changes = Vec::new();
        for path in paths {
            let file = OpenOptions::new()
                .read(true)
                .custom_flags(0x100 | 0x1000000)
                .open(path)?;
            changes.push(Kevent {
                ident: file.as_raw_fd() as usize,
                filter: -4,
                flags: 0x25,
                fflags: 0x7f,
                data: 0,
                udata: 0,
            });
            files.push(file);
        }
        let count = unsafe {
            kevent(
                queue.as_raw_fd(),
                changes.as_ptr(),
                changes.len() as i32,
                std::ptr::null_mut(),
                0,
                std::ptr::null(),
            )
        };
        if count < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self {
            queue,
            _files: files,
            snapshot,
        })
    }
    fn changed(&self) -> std::io::Result<bool> {
        let mut event = Kevent {
            ident: 0,
            filter: 0,
            flags: 0,
            fflags: 0,
            data: 0,
            udata: 0,
        };
        let timeout = Timespec {
            seconds: 0,
            nanoseconds: 20_000_000,
        };
        let count = unsafe {
            kevent(
                self.queue.as_raw_fd(),
                std::ptr::null(),
                0,
                &mut event,
                1,
                &timeout,
            )
        };
        if count < 0 {
            return Err(std::io::Error::last_os_error());
        }
        if count > 0 && event.flags & 0x4000 != 0 {
            return Err(std::io::Error::from_raw_os_error(event.data as i32));
        }
        Ok(count > 0)
    }
}
fn changes(before: &Snapshot, after: &Snapshot, filter: u32) -> Vec<(u32, String)> {
    let mut removed: Vec<_> = before
        .iter()
        .filter(|(n, _)| !after.contains_key(*n))
        .collect();
    let mut added: Vec<_> = after
        .iter()
        .filter(|(n, _)| !before.contains_key(*n))
        .collect();
    let mut out = Vec::new();
    for (old, node) in removed.drain(..) {
        let wanted = if node.directory { 2 } else { 1 };
        if filter & wanted == 0 {
            continue;
        }
        if let Some(i) = added
            .iter()
            .position(|(_, new)| new.id == node.id && !node.directory)
        {
            let (new, _) = added.remove(i);
            out.push((4, old.clone()));
            out.push((5, new.clone()));
        } else {
            out.push((2, old.clone()));
        }
    }
    for (name, node) in added {
        if filter & if node.directory { 2 } else { 1 } != 0 {
            out.push((1, name.clone()));
        }
    }
    for (name, old) in before {
        if let Some(new) = after.get(name) {
            let modified = (filter & 4 != 0 && old.mode != new.mode)
                || (filter & 8 != 0 && old.size != new.size)
                || (filter & 16 != 0 && old.write != new.write)
                || (filter & 32 != 0 && old.access != new.access)
                || (filter & 64 != 0 && old.created != new.created)
                || (filter & 256 != 0
                    && (old.mode, old.uid, old.gid) != (new.mode, new.uid, new.gid));
            if modified {
                out.push((3, name.clone()));
            }
        }
    }
    out
}
fn encode(records: &[(u32, String)], capacity: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (i, (action, name)) in records.iter().enumerate() {
        let name: Vec<_> = name.encode_utf16().collect();
        let length = 12 + name.len() * 2;
        let next = if i + 1 == records.len() {
            0
        } else {
            length.div_ceil(4) * 4
        };
        let size = if next == 0 { length } else { next };
        if bytes.len() + size > capacity {
            return Vec::new();
        }
        bytes.extend_from_slice(&(next as u32).to_le_bytes());
        bytes.extend_from_slice(&action.to_le_bytes());
        bytes.extend_from_slice(&((name.len() * 2) as u32).to_le_bytes());
        for unit in name {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.resize(bytes.len() + size - length, 0);
    }
    bytes
}
struct Request {
    buffer: usize,
    capacity: usize,
    overlap: usize,
    event: u32,
    owner: u32,
    cancel: AtomicBool,
    result: Mutex<Option<std::result::Result<u32, u32>>>,
    notify: EventNotifier,
    directory_notify: EventNotifier,
}
impl Request {
    fn complete(&self, result: std::result::Result<Vec<u8>, u32>) {
        let (bytes, error) = match result {
            Ok(v) => (v, 0),
            Err(e) => (Vec::new(), e),
        };
        let status = match error {
            0 => 0,
            995 => 0xc0000120,
            5 => 0xc0000022,
            _ => 0xc0000001,
        };
        // SAFETY: ReadDirectoryChangesW requires its buffer/OVERLAPPED to remain
        // exclusively live until completion/cancellation. All monitors are
        // joined before process heaps/mappings can be released. No Rust slice
        // into these regions survives the issuing API's return.
        unsafe {
            if !bytes.is_empty() {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.buffer as *mut u8, bytes.len());
            }
            std::ptr::write_volatile((self.overlap + 4) as *mut u32, bytes.len() as u32);
            std::sync::atomic::fence(Ordering::Release);
            std::ptr::write_volatile(self.overlap as *mut u32, status);
        }
        let mut published = self.result.lock().unwrap();
        *published = Some(if error == 0 {
            Ok(bytes.len() as u32)
        } else {
            Err(error)
        });
        self.notify.signal();
        self.directory_notify.signal();
    }
}
struct WatchControl {
    request: Option<Arc<Request>>,
    backlog: Vec<(u32, String)>,
    overflow: bool,
    stopped: bool,
    failed: Option<u32>,
}
struct Watcher {
    tree: bool,
    filter: u32,
    control: Arc<Mutex<WatchControl>>,
    join: JoinHandle<()>,
}
impl Watcher {
    fn stop(self) {
        self.control.lock().unwrap().stopped = true;
        if self.join.join().is_err() {
            unsupported("directory monitor panicked");
        }
    }
}
struct Directory {
    file: File,
    path: PathBuf,
    watcher: Option<Watcher>,
    last_request: Option<Arc<Request>>,
}
#[derive(Default)]
pub(super) struct State {
    directories: BTreeMap<u32, Directory>,
}
impl State {
    pub(super) fn contains(&self, h: u32) -> bool {
        self.directories.contains_key(&h)
    }
    pub(super) fn completion_event(&self, h: u32) -> bool {
        self.directories
            .values()
            .any(|d| d.last_request.as_ref().is_some_and(|p| p.event == h))
    }
    pub(super) fn clear(&mut self) {
        for (h, mut row) in std::mem::take(&mut self.directories) {
            if let Some(p) = row.watcher.take() {
                p.stop();
            }
            ffi::CloseHandle(h as usize);
        }
    }
    pub(super) fn close(&mut self, h: u32) -> bool {
        let Some(mut row) = self.directories.remove(&h) else {
            return false;
        };
        if let Some(p) = row.watcher.take() {
            p.stop();
        }
        if ffi::CloseHandle(h as usize) == 0 {
            unsupported("directory completion handle disappeared");
        }
        true
    }
}
fn fail(error: u32) -> u32 {
    ffi::SetLastError(error);
    0
}
pub(super) fn open(image: &LoadedImage32<'_>, name: &str, a: &[u32; 8]) -> u32 {
    if a[1] != 1 || a[2] != 7 || a[4] != 3 || a[5] != 0x42000000 {
        unsupported(
            "directory watch open requires FILE_LIST_DIRECTORY, full sharing and overlapped backup semantics",
        );
    }
    let (path, meta) =
        match files32::existing(name.trim_end_matches('\\')).unwrap_or_else(|e| unsupported(&e)) {
            Ok(v) => v,
            Err(e) => {
                fail(e);
                return u32::MAX;
            }
        };
    if !meta.is_dir() {
        fail(267);
        return u32::MAX;
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(0x100 | 0x1000000 | 0x100000)
        .open(&path)
        .unwrap_or_else(|e| unsupported(&format!("directory backing open failed: {e}")));
    let opened = file
        .metadata()
        .unwrap_or_else(|e| unsupported(&e.to_string()));
    if (meta.dev(), meta.ino()) != (opened.dev(), opened.ino()) {
        unsupported("directory identity changed during open");
    }
    if image.cpu.watches.borrow().directories.len() >= 64 {
        fail(8);
        return u32::MAX;
    }
    let h = unsafe { ffi::CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
    if h == 0 || h > u32::MAX as usize {
        fail(8);
        return u32::MAX;
    }
    image.cpu.watches.borrow_mut().directories.insert(
        h as u32,
        Directory {
            file,
            path,
            watcher: None,
            last_request: None,
        },
    );
    h as u32
}
pub(super) fn read(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if a[7] != 0 {
        unsupported("directory completion routines require APC delivery");
    }
    if a[4] == 0 || a[4] & !0x17f != 0 || a[2] == 0 || a[2] > 65536 {
        return fail(87);
    }
    if !a[1].is_multiple_of(4)
        || !image.permits(a[1] as usize, a[2] as usize, true)
        || !a[6].is_multiple_of(4)
        || !image.permits(a[6] as usize, 20, true)
    {
        unsupported("directory notification buffers outside checked writable storage");
    }
    if a[5] != 0 && (!a[5].is_multiple_of(4) || !image.permits(a[5] as usize, 4, true)) {
        unsupported("directory byte-count output outside writable storage");
    }
    let end = a[1] as u64 + a[2] as u64;
    if (a[1] as u64) < a[6] as u64 + 20 && (a[6] as u64) < end {
        unsupported("directory notification buffer aliases its OVERLAPPED record");
    }
    let overlap = unsafe { (a[6] as *const [u32; 5]).read() };
    let event = if overlap[4] == 0 { a[0] } else { overlap[4] };
    if event != a[0] && !image.cpu.events.read().unwrap().contains(event) {
        unsupported("directory completion event must be an owned event");
    }
    let notify = kernel()
        .event_notifier(event as usize)
        .unwrap_or_else(|_| unsupported("directory completion requires an event object"));
    let directory_notify = kernel()
        .event_notifier(a[0] as usize)
        .unwrap_or_else(|_| unsupported("directory completion handle is not owned"));
    let mut state = image.cpu.watches.borrow_mut();
    let Some(row) = state.directories.get_mut(&a[0]) else {
        return fail(6);
    };
    if row
        .last_request
        .as_ref()
        .is_some_and(|r| r.result.lock().unwrap().is_none())
    {
        return fail(87);
    }
    let root = row.path.clone();
    let observed = row
        .file
        .metadata()
        .unwrap_or_else(|e| unsupported(&e.to_string()));
    let pathmeta = std::fs::metadata(&root).unwrap_or_else(|e| unsupported(&e.to_string()));
    if (observed.dev(), observed.ino()) != (pathmeta.dev(), pathmeta.ino()) {
        unsupported("watched directory identity changed");
    }
    let tree = a[3] != 0;
    let filter = a[4];
    if row
        .watcher
        .as_ref()
        .is_some_and(|w| w.tree != tree || w.filter != filter)
    {
        unsupported("changing an established directory watch filter requires a new handle");
    }
    let request = Arc::new(Request {
        buffer: a[1] as usize,
        capacity: a[2] as usize,
        overlap: a[6] as usize,
        event,
        owner: ffi::GetCurrentThreadId(),
        cancel: AtomicBool::new(false),
        result: Mutex::new(None),
        notify,
        directory_notify,
    });
    ffi::ResetEvent(event as usize);
    ffi::ResetEvent(a[0] as usize);
    unsafe {
        (a[6] as *mut u32).write_volatile(0x103);
        ((a[6] + 4) as *mut u32).write_volatile(0);
        if a[5] != 0 {
            (a[5] as *mut u32).write(0);
        }
    }
    if let Some(watcher) = &row.watcher {
        watcher.control.lock().unwrap().request = Some(request.clone());
    } else {
        let monitor = match Monitor::arm(&root, tree, filter) {
            Ok(m) => m,
            Err(e) => {
                return fail(if e.kind() == std::io::ErrorKind::PermissionDenied {
                    5
                } else {
                    50
                });
            }
        };
        let capacity = request.capacity;
        let control = Arc::new(Mutex::new(WatchControl {
            request: Some(request.clone()),
            backlog: Vec::new(),
            overflow: false,
            stopped: false,
            failed: None,
        }));
        let shared = control.clone();
        let join = std::thread::Builder::new()
            .name("Alkyhol directory monitor".into())
            .spawn(move || monitor_loop(monitor, root, tree, filter, capacity, shared))
            .unwrap_or_else(|e| unsupported(&format!("directory monitor spawn failed: {e}")));
        row.watcher = Some(Watcher {
            tree,
            filter,
            control,
            join,
        });
    }
    row.last_request = Some(request);
    1
}
pub(super) fn cancel(image: &LoadedImage32<'_>, h: u32) -> u32 {
    let state = image.cpu.watches.borrow();
    let Some(row) = state.directories.get(&h) else {
        return fail(6);
    };
    if let Some(p) = &row.last_request
        && p.owner == ffi::GetCurrentThreadId()
    {
        p.cancel.store(true, Ordering::Release);
    }
    1
}
pub(super) fn result(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if !a[2].is_multiple_of(4) || !image.permits(a[2] as usize, 4, true) {
        unsupported("overlapped transfer count outside writable guest storage");
    }
    let request = {
        let state = image.cpu.watches.borrow();
        let Some(row) = state.directories.get(&a[0]) else {
            return fail(6);
        };
        let Some(p) = &row.last_request else {
            return fail(87);
        };
        if p.overlap != a[1] as usize {
            return fail(87);
        }
        p.clone()
    };
    if a[3] != 0 && request.result.lock().unwrap().is_none() {
        super::thread32::wait_handle(image, request.event, u32::MAX);
    }
    let done = *request.result.lock().unwrap();
    match done {
        None => fail(996),
        Some(Err(e)) => fail(e),
        Some(Ok(bytes)) => {
            unsafe {
                (a[2] as *mut u32).write(bytes);
            }
            1
        }
    }
}

fn monitor_loop(
    mut monitor: Monitor,
    root: PathBuf,
    tree: bool,
    filter: u32,
    capacity: usize,
    shared: Arc<Mutex<WatchControl>>,
) {
    loop {
        let (stopped, completion) = {
            let mut state = shared.lock().unwrap();
            let canceled = state
                .request
                .as_ref()
                .is_some_and(|r| r.cancel.load(Ordering::Acquire));
            let deliver = state.request.is_some()
                && (state.stopped
                    || canceled
                    || state.failed.is_some()
                    || state.overflow
                    || !state.backlog.is_empty());
            let completion = if deliver {
                let request = state.request.take().unwrap();
                let result = if state.stopped || canceled {
                    Err(995)
                } else if let Some(error) = state.failed {
                    Err(error)
                } else if state.overflow {
                    state.overflow = false;
                    state.backlog.clear();
                    Ok(Vec::new()) // Documented zero-byte completion requires full enumeration.
                } else {
                    Ok(encode(
                        &std::mem::take(&mut state.backlog),
                        request.capacity,
                    ))
                };
                Some((request, result))
            } else {
                None
            };
            (state.stopped, completion)
        };
        if let Some((request, result)) = completion {
            request.complete(result);
        }
        if stopped {
            return;
        }
        match monitor.changed() {
            Ok(false) => continue,
            Err(_) => {
                shared.lock().unwrap().failed = Some(1022);
                std::thread::sleep(std::time::Duration::from_millis(20));
                continue;
            }
            Ok(true) => {}
        }
        let after = match scan(&root, tree) {
            Ok(v) => v,
            Err(_) => {
                shared.lock().unwrap().failed = Some(1022);
                continue;
            }
        };
        let records = changes(&monitor.snapshot, &after, filter);
        // Keep the old native queue alive until its replacement is armed. A
        // coalesced/unrepresentable change requests enumeration, never a made-up name.
        let replacement = match Monitor::arm(&root, tree, filter) {
            Ok(v) => v,
            Err(_) => {
                shared.lock().unwrap().failed = Some(1022);
                continue;
            }
        };
        let replaced_identity = after
            .iter()
            .any(|(name, n)| monitor.snapshot.get(name).is_some_and(|old| old.id != n.id));
        let renamed_directory = monitor.snapshot.iter().any(|(name, old)| {
            old.directory
                && !after.contains_key(name)
                && after
                    .iter()
                    .any(|(new_name, new)| new_name != name && new.id == old.id)
        });
        let uncertain = after == monitor.snapshot
            || replacement.snapshot != after
            || replaced_identity
            || renamed_directory;
        monitor = replacement;
        let mut state = shared.lock().unwrap();
        if uncertain {
            state.overflow = true;
        }
        state.backlog.extend(records);
        if state.backlog.len() > 32768
            || (!state.backlog.is_empty() && encode(&state.backlog, capacity).is_empty())
        {
            state.overflow = true;
        }
        if state.overflow {
            state.backlog.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notification_records_align_links_count_utf16_and_discard_overflow() {
        let records = vec![(4, "old".into()), (5, "new.txt".into())];
        let bytes = encode(&records, 64);
        assert_eq!(bytes.len(), 46);
        assert_eq!(&bytes[..12], &[20, 0, 0, 0, 4, 0, 0, 0, 6, 0, 0, 0]);
        assert_eq!(&bytes[18..20], &[0, 0]);
        assert_eq!(&bytes[20..32], &[0, 0, 0, 0, 5, 0, 0, 0, 14, 0, 0, 0]);
        assert!(encode(&records, 45).is_empty());
        assert_eq!(encode(&[(1, "\u{1f338}".into())], 16).len(), 16);
        assert_eq!(std::mem::size_of::<Kevent>(), 32);
        assert_eq!(std::mem::offset_of!(Kevent, udata), 24);
    }
    #[test]
    fn real_snapshots_distinguish_rename_content_and_filtered_changes() {
        let root = std::env::temp_dir().join(format!("alkyhol-watch-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("first"), b"one").unwrap();
        let before = scan(&root, false).unwrap();
        std::fs::rename(root.join("first"), root.join("second")).unwrap();
        let renamed = scan(&root, false).unwrap();
        assert_eq!(
            changes(&before, &renamed, 1),
            vec![(4, "first".into()), (5, "second".into())]
        );
        assert!(changes(&before, &renamed, 8).is_empty());
        std::fs::write(root.join("second"), b"longer").unwrap();
        let modified = scan(&root, false).unwrap();
        assert_eq!(changes(&renamed, &modified, 8), vec![(3, "second".into())]);
        assert!(changes(&renamed, &modified, 1).is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }
}
