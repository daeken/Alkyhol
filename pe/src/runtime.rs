use crate::{Image, Result, abi};
use std::{ffi::c_void, ptr};

unsafe extern "C" {
    fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64,
    ) -> *mut c_void;
    fn mprotect(addr: *mut c_void, len: usize, prot: i32) -> i32;
    fn munmap(addr: *mut c_void, len: usize) -> i32;
    fn getpagesize() -> i32;
}
const READ: i32 = 1;
const WRITE: i32 = 2;
const EXEC: i32 = 4;
const PRIVATE_ANON: i32 = 2 | 0x1000; // Darwin MAP_PRIVATE | MAP_ANON; never MAP_FIXED.

/// Owned executable mapping. Loading performs all metadata/import checks before
/// allocation and never enters guest code. Dropping unmaps it; code may not retain
/// pointers or start threads beyond the mapping's lifetime.
pub struct LoadedImage {
    base: *mut u8,
    size: usize,
    entry: usize,
    relocated: bool,
}
impl LoadedImage {
    /// Map checked AMD64 PE data, apply DIR64 relocations and bind an explicit
    /// kernel32 whitelist. `rebase` deliberately avoids the preferred address.
    pub fn load(bytes: &[u8], rebase: bool) -> Result<Self> {
        let image = Image::parse(bytes)?;
        image.check_startup()?;
        let imports = image.imports()?;
        let relocations = image.relocations()?;
        let mut bindings = Vec::new();
        for import in &imports {
            let address = abi::resolve(&import.module, &import.symbol)
                .ok_or_else(|| format!("unsupported import {}!{}", import.module, import.symbol))?;
            if relocations
                .iter()
                .any(|&r| r < import.iat + 8 && import.iat < r + 8)
            {
                return Err("IAT overlaps relocation fixup".into());
            }
            bindings.push((import.iat, address));
        }
        // This contract uses independent 4K section pages. Do not accidentally
        // grant neighbouring sections RWX on a different host page geometry.
        if unsafe { getpagesize() } != 4096 {
            return Err("PE runtime requires 4K x86 process pages".into());
        }
        let hint = if rebase {
            ptr::null_mut()
        } else {
            image.base as *mut c_void
        };
        // SAFETY: anonymous private mapping, not a fixed-address replacement.
        let base = unsafe { mmap(hint, image.size, READ | WRITE, PRIVATE_ANON, -1, 0) };
        if base as isize == -1 {
            return Err(format!(
                "image mmap failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        let mapped = Self {
            base: base.cast(),
            size: image.size,
            entry: image.entry,
            relocated: base as u64 != image.base,
        };
        if rebase && !mapped.relocated {
            return Err("forced rebasing did not obtain a different address".into());
        }
        if mapped.relocated && relocations.is_empty() {
            return Err("preferred image base unavailable and no DIR64 relocation targets".into());
        }
        // SAFETY: checked sections fit, don't overlap and all writes precede W^X protections.
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), mapped.base, image.headers);
            for s in &image.sections {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr().add(s.raw),
                    mapped.base.add(s.rva),
                    s.raw_size,
                );
            }
            if mapped.relocated {
                let delta = (mapped.base as u64).wrapping_sub(image.base);
                for rva in relocations {
                    let p = mapped.base.add(rva).cast::<u64>();
                    ptr::write_unaligned(p, ptr::read_unaligned(p).wrapping_add(delta));
                }
            }
            for (iat, address) in bindings {
                ptr::write_unaligned(mapped.base.add(iat).cast::<usize>(), address);
            }
        }
        mapped.protect(0, mapped.size, 0)?;
        mapped.protect(0, image.headers, READ)?;
        for s in &image.sections {
            let mut rights = 0;
            if s.rights & 0x40000000 != 0 {
                rights |= READ;
            }
            if s.rights & 0x80000000 != 0 {
                rights |= WRITE;
            }
            if s.rights & 0x20000000 != 0 {
                rights |= EXEC;
            }
            mapped.protect(s.rva, s.size, rights)?;
        }
        Ok(mapped)
    }
    fn protect(&self, rva: usize, len: usize, rights: i32) -> Result<()> {
        // SAFETY: parser ensures aligned, disjoint, in-mapping ranges. mprotect
        // rounds the ending page; the header/section page extents never overlap.
        if unsafe { mprotect(self.base.add(rva).cast(), len, rights) } != 0 {
            return Err(format!(
                "image mprotect failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }
    pub fn relocated(&self) -> bool {
        self.relocated
    }
    /// Enter trusted Windows machine code on the current thread using Win64 ABI.
    ///
    /// # Safety
    /// Caller must trust all executable bytes to obey this runtime's bounded
    /// contract (no direct syscalls, GS/TEB accesses, SEH, guest callbacks/threads,
    /// arbitrary host memory access or pointers escaping this mapping). This is
    /// native execution, not validation of code safety or a security sandbox.
    pub unsafe fn enter(&self) -> u32 {
        let entry: unsafe extern "win64" fn() -> u32 =
            unsafe { std::mem::transmute(self.base.add(self.entry)) };
        unsafe { entry() }
    }
}
impl Drop for LoadedImage {
    fn drop(&mut self) {
        unsafe {
            munmap(self.base.cast(), self.size);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;

    // Small conventional PE: RX code returns 42, RO data, RW zero-filled data.
    fn permission_fixture() -> Vec<u8> {
        let mut b = vec![0; 0xa00];
        b[..2].copy_from_slice(b"MZ");
        let p16 = |b: &mut [u8], o, n: u16| b[o..o + 2].copy_from_slice(&n.to_le_bytes());
        let p32 = |b: &mut [u8], o, n: u32| b[o..o + 4].copy_from_slice(&n.to_le_bytes());
        p32(&mut b, 0x3c, 0x80);
        b[0x80..0x84].copy_from_slice(b"PE\0\0");
        p16(&mut b, 0x84, 0x8664);
        p16(&mut b, 0x86, 3);
        p16(&mut b, 0x94, 240);
        p16(&mut b, 0x96, 0x22);
        let o = 0x98;
        p16(&mut b, o, 0x20b);
        p32(&mut b, o + 16, 0x1000);
        b[o + 24..o + 32].copy_from_slice(&0x140000000u64.to_le_bytes());
        p32(&mut b, o + 32, 4096);
        p32(&mut b, o + 36, 512);
        p32(&mut b, o + 56, 0x4000);
        p32(&mut b, o + 60, 0x400);
        p16(&mut b, o + 68, 3);
        p32(&mut b, o + 108, 16);
        for (i, rva, raw, rights) in [
            (0, 0x1000, 0x400, 0x60000020),
            (1, 0x2000, 0x600, 0x40000040),
            (2, 0x3000, 0x800, 0xc0000040),
        ] {
            let s = o + 240 + i * 40;
            p32(&mut b, s + 8, 512);
            p32(&mut b, s + 12, rva);
            p32(&mut b, s + 16, 512);
            p32(&mut b, s + 20, raw);
            p32(&mut b, s + 36, rights);
        }
        b[0x400..0x406].copy_from_slice(&[0xb8, 42, 0, 0, 0, 0xc3]);
        // One DIR64 entry in RO data, fixing a pointer in RW data.
        p32(&mut b, o + 112 + 5 * 8, 0x2000);
        p32(&mut b, o + 116 + 5 * 8, 12);
        p32(&mut b, 0x600, 0x3000);
        p32(&mut b, 0x604, 12);
        p16(&mut b, 0x608, 0xa000);
        b[0x800..0x808].copy_from_slice(&0x140001000u64.to_le_bytes());
        b
    }

    #[test]
    fn image_permissions_are_enforced_by_the_host() {
        const CHILD: &str = "ALKYHOL_PE_PERMISSION_CHILD";
        if let Ok(mode) = std::env::var(CHILD) {
            let bytes = permission_fixture();
            let mapped = LoadedImage::load(&bytes, true).unwrap();
            unsafe {
                assert_eq!(
                    mapped.base.add(0x3000).cast::<usize>().read(),
                    mapped.base.add(0x1000) as usize
                );
                match mode.as_str() {
                    "header-write" => mapped.base.write_volatile(0),
                    "code-write" => mapped.base.add(0x1000).write_volatile(0),
                    "rodata-write" => mapped.base.add(0x2000).write_volatile(0),
                    "data-execute" => {
                        mapped.base.add(0x3010).write_volatile(0xc3);
                        let f: unsafe extern "win64" fn() =
                            std::mem::transmute(mapped.base.add(0x3010));
                        f();
                    }
                    "allowed" => {
                        mapped.base.add(0x3010).write_volatile(0x55);
                        assert_eq!(mapped.base.add(0x3010).read_volatile(), 0x55);
                        assert_eq!(mapped.enter(), 42);
                        return;
                    }
                    _ => panic!("unknown permission case"),
                }
            }
            panic!("forbidden mapping access unexpectedly succeeded");
        }
        let current = std::env::current_exe().unwrap();
        for mode in [
            "allowed",
            "header-write",
            "code-write",
            "rodata-write",
            "data-execute",
        ] {
            let child = std::process::Command::new(&current)
                .args([
                    "--exact",
                    "runtime::tests::image_permissions_are_enforced_by_the_host",
                    "--nocapture",
                ])
                .env(CHILD, mode)
                .output()
                .unwrap();
            if mode == "allowed" {
                assert!(child.status.success(), "{child:?}");
            } else {
                assert!(
                    matches!(child.status.signal(), Some(10 | 11)),
                    "{mode}: {child:?}"
                );
            }
        }
    }
}
