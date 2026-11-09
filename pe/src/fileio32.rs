//! File-family state: real descriptors, process-local sharing and scoped writes.
use super::{LoadedImage32, bulk32, directory32, files32, path32, startup32, unsupported};
use std::{
    fs::OpenOptions,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
};
pub(super) struct Lease {
    pub identity: (u64, u64),
    pub access: u32,
    pub share: u32,
}
fn fail(error: u32) -> u32 {
    alkyhol::ffi::SetLastError(error);
    u32::MAX
}
pub(super) fn track(image: &LoadedImage32<'_>, h: u32, access: u32, share: u32) {
    let meta = alkyhol::kernel::kernel()
        .file_metadata(h as usize)
        .unwrap_or_else(|_| unsupported("file lease requires live descriptor metadata"));
    image.cpu.file_leases.borrow_mut().insert(
        h as usize,
        Lease {
            identity: (meta.dev(), meta.ino()),
            access,
            share,
        },
    );
}
pub(super) fn open(image: &LoadedImage32<'_>, a: &[u32; 8], ansi: bool) -> u32 {
    if a[3] != 0 || a[6] != 0 {
        unsupported("file security/template arguments are unsupported");
    }
    if image.cpu.files.borrow().len() >= 4096 {
        unsupported("owned file-handle budget exhausted");
    }
    let name = if ansi {
        bulk32::ascii(image, a[0])
    } else {
        let mut units = startup32::read_units::<u16>(image, a[0], -1)
            .unwrap_or_else(|| unsupported("file path outside checked UTF-16 storage"));
        units.pop();
        String::from_utf16(&units).unwrap_or_else(|_| unsupported("file path is invalid UTF-16"))
    };
    let cwd = directory32::current(image).unwrap_or_else(|e| unsupported(&e));
    let name = path32::resolve(&cwd, &name).unwrap_or_else(|e| unsupported(&e));
    if image.cpu.diagnostic.is_some() {
        eprintln!(
            "PE32 diagnostic file open: path={name:?}; access={:x}; share={:x}; disposition={}; flags={:x}",
            a[1], a[2], a[4], a[5]
        );
    }
    if a[5] & 0x02000000 != 0 {
        return super::watch32::open(image, &name, a);
    }
    if a[1] & !0xc0000000 != 0 || a[2] & !7 != 0 || a[5] & !(0x80 | 0x18000000) != 0 {
        unsupported("file access/sharing/flags are outside the synchronous ordinary profile");
    }
    let write = a[1] & 0x40000000 != 0;
    let (file, existed) = if write {
        let docs = image.cpu.documents.as_ref().unwrap_or_else(|| {
            unsupported("file creation/write requires explicit Documents backing")
        });
        match docs
            .open_file(&name, a[1], a[4])
            .unwrap_or_else(|e| unsupported(&e))
        {
            Ok(v) => v,
            Err(error) => return fail(error),
        }
    } else {
        if a[4] != 3 {
            unsupported("read-only file creation dispositions are unsupported");
        }
        let (path, meta) = match files32::existing(&name).unwrap_or_else(|e| unsupported(&e)) {
            Ok(v) => v,
            Err(error) => return fail(error),
        };
        if !meta.is_file() {
            unsupported("ordinary file open requires a regular file");
        }
        let file = match OpenOptions::new()
            .read(true)
            .custom_flags(0x100 | 0x1000000)
            .open(path)
        {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return fail(5),
            Err(e) => unsupported(&format!("file changed or failed to open: {e}")),
        };
        let opened = file
            .metadata()
            .unwrap_or_else(|e| unsupported(&e.to_string()));
        if (meta.dev(), meta.ino()) != (opened.dev(), opened.ino()) {
            unsupported("file identity changed during checked open");
        }
        (file, true)
    };
    let meta = file
        .metadata()
        .unwrap_or_else(|e| unsupported(&e.to_string()));
    let identity = (meta.dev(), meta.ino());
    if image.cpu.file_leases.borrow().values().any(|old| {
        old.identity == identity
            && ((a[1] & 0x80000000 != 0 && old.share & 1 == 0)
                || (a[1] & 0x40000000 != 0 && old.share & 2 == 0)
                || (old.access & 0x80000000 != 0 && a[2] & 1 == 0)
                || (old.access & 0x40000000 != 0 && a[2] & 2 == 0))
    }) {
        return fail(32);
    }
    if matches!(a[4], 2 | 5) {
        file.set_len(0)
            .unwrap_or_else(|e| unsupported(&format!("file truncation failed: {e}")));
    }
    let h = match alkyhol::kernel::kernel().adopt_file(file, a[1]) {
        Ok(h) => h,
        Err(e) => return fail(e.0),
    };
    if h > u32::MAX as usize {
        unsupported("file handle outside PE32 namespace");
    }
    image.cpu.files.borrow_mut().insert(h);
    image.cpu.file_leases.borrow_mut().insert(
        h,
        Lease {
            identity,
            access: a[1],
            share: a[2],
        },
    );
    alkyhol::ffi::SetLastError(if matches!(a[4], 2 | 4) && existed {
        183
    } else {
        0
    });
    h as u32
}
pub(super) fn seek(image: &LoadedImage32<'_>, a: &[u32; 8], wide: bool) -> u32 {
    if !image.cpu.files.borrow().contains(&(a[0] as usize)) {
        unsupported("seek requires an owned live file handle");
    }
    let (pointer, method, distance) = if wide {
        (a[3], a[4], ((a[2] as i32 as i64) << 32) | a[1] as i64)
    } else {
        if a[2] != 0 && (!a[2].is_multiple_of(4) || !image.permits(a[2] as usize, 4, true)) {
            unsupported("seek high-word pointer outside writable storage");
        }
        let distance = if a[2] == 0 {
            a[1] as i32 as i64
        } else {
            ((unsafe { (a[2] as *const i32).read() } as i64) << 32) | a[1] as i64
        };
        (a[2], a[3], distance)
    };
    if wide
        && pointer != 0
        && (!pointer.is_multiple_of(4) || !image.permits(pointer as usize, 8, true))
    {
        unsupported("seek position output outside writable storage");
    }
    match alkyhol::kernel::kernel().seek(a[0] as usize, distance, method) {
        Ok(position) => {
            if pointer != 0 {
                unsafe {
                    if wide {
                        (pointer as *mut u64).write_unaligned(position);
                    } else {
                        (pointer as *mut u32).write((position >> 32) as u32);
                    }
                }
            }
            if wide {
                1
            } else {
                let low = position as u32;
                if low == u32::MAX {
                    alkyhol::ffi::SetLastError(0);
                }
                low
            }
        }
        Err(error) => {
            alkyhol::ffi::SetLastError(error.0);
            if wide { 0 } else { u32::MAX }
        }
    }
}
pub(super) fn flush(image: &LoadedImage32<'_>, h: u32) -> u32 {
    if !image.cpu.files.borrow().contains(&(h as usize)) {
        unsupported("flush requires an owned live file handle");
    }
    match alkyhol::kernel::kernel().flush_file(h as usize) {
        Ok(()) => 1,
        Err(e) => {
            alkyhol::ffi::SetLastError(e.0);
            0
        }
    }
}

pub(super) fn information(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if !image.cpu.files.borrow().contains(&(a[0] as usize)) {
        unsupported("file information requires an owned live file handle");
    }
    if !a[1].is_multiple_of(4) || !image.permits(a[1] as usize, 52, true) {
        unsupported("file information output outside writable storage");
    }
    let m = match alkyhol::kernel::kernel().file_metadata(a[0] as usize) {
        Ok(m) => m,
        Err(e) => {
            alkyhol::ffi::SetLastError(e.0);
            return 0;
        }
    };
    let mut out = [0u32; 13];
    out[0] = if m.permissions().readonly() {
        0x21
    } else {
        0x20
    };
    for (i, time) in [m.created(), m.accessed(), m.modified()]
        .into_iter()
        .enumerate()
    {
        let words = crate::process::file_time(
            time.unwrap_or_else(|_| unsupported("file timestamp unavailable")),
        )
        .unwrap_or_else(|_| unsupported("file timestamp outside FILETIME"));
        out[1 + i * 2..3 + i * 2].copy_from_slice(&words);
    }
    out[7] = m.dev() as u32;
    out[8] = (m.len() >> 32) as u32;
    out[9] = m.len() as u32;
    out[10] =
        u32::try_from(m.nlink()).unwrap_or_else(|_| unsupported("file link count outside DWORD"));
    out[11] = (m.ino() >> 32) as u32;
    out[12] = m.ino() as u32;
    unsafe {
        std::ptr::copy_nonoverlapping(out.as_ptr(), a[1] as *mut u32, 13);
    }
    1
}
pub(super) fn end(image: &LoadedImage32<'_>, h: u32) -> u32 {
    if !image.cpu.files.borrow().contains(&(h as usize)) {
        unsupported("file truncation requires an owned live handle");
    }
    match alkyhol::kernel::kernel().set_end_of_file(h as usize) {
        Ok(()) => 1,
        Err(e) => {
            alkyhol::ffi::SetLastError(e.0);
            0
        }
    }
}
