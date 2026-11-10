//! Apple Silicon executable storage. Deliberately avoids Tier0's Linux RWX finalizer.
use crate::Result;
use sharpretro_jit::aarch64_enc::Aarch64Enc;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
unsafe extern "C" {
    fn pthread_jit_write_protect_np(enabled: i32);
    fn sys_icache_invalidate(start: *mut std::ffi::c_void, length: usize);
}
struct Page {
    base: *mut std::ffi::c_void,
    size: usize,
    used: Cell<usize>,
}
thread_local! { static PAGE: RefCell<Weak<Page>> = const { RefCell::new(Weak::new()) }; }
pub(crate) struct Code {
    page: Rc<Page>,
    offset: usize,
}
impl Code {
    pub fn new(enc: &Aarch64Enc) -> Result<Self> {
        let bytes = enc.bytes();
        if bytes.is_empty() || bytes.len() > 1024 * 1024 {
            return Err("invalid JIT code length".into());
        }
        let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        if page <= 0 {
            return Err("cannot query native page size".into());
        }
        let page = page as usize;
        let (page, offset) = PAGE.with(|current| -> Result<_> {
            if let Some(page) = current.borrow().upgrade() {
                let offset = page.used.get().div_ceil(16) * 16;
                if bytes.len() <= page.size - offset {
                    page.used.set(offset + bytes.len());
                    return Ok((page, offset));
                }
            }
            let size = bytes.len().max(64 * 1024).div_ceil(page) * page;
            let base = unsafe {
                libc::mmap(
                    std::ptr::null_mut(),
                    size,
                    libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
                    libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_JIT,
                    -1,
                    0,
                )
            };
            if base == libc::MAP_FAILED {
                return Err(format!("MAP_JIT: {}", std::io::Error::last_os_error()));
            }
            let page = Rc::new(Page {
                base,
                size,
                used: Cell::new(bytes.len()),
            });
            *current.borrow_mut() = Rc::downgrade(&page);
            Ok((page, 0))
        })?;
        let base = unsafe { page.base.cast::<u8>().add(offset) };
        unsafe {
            pthread_jit_write_protect_np(0);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), base.cast(), bytes.len());
            pthread_jit_write_protect_np(1);
            sys_icache_invalidate(base.cast(), bytes.len());
        }
        Ok(Self { page, offset })
    }
    /// The checked compiler owns all emitted instructions and bounds both arrays.
    pub unsafe fn enter(&self, state: &mut [u64; 91], spill: &mut [u64; 4096]) {
        let entry: unsafe extern "C" fn(*mut u64, *mut u64) =
            unsafe { std::mem::transmute(self.page.base.cast::<u8>().add(self.offset)) };
        unsafe {
            entry(state.as_mut_ptr(), spill.as_mut_ptr());
        }
    }
}
impl Drop for Page {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.base, self.size);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packed_blocks_are_disjoint_and_pages_live_until_the_last_block() {
        let mut blocks = Vec::new();
        for value in 0..64 {
            let mut enc = Aarch64Enc::new();
            enc.mov_imm64(9, value);
            enc.str_x(9, 0, 0);
            enc.ret();
            blocks.push(Code::new(&enc).unwrap());
        }
        assert!(blocks.iter().all(|b| Rc::ptr_eq(&b.page, &blocks[0].page)));
        let weak = Rc::downgrade(&blocks[0].page);
        for (i, b) in blocks.iter().enumerate() {
            assert_eq!(b.offset % 16, 0);
            let mut state = [0; 91];
            unsafe {
                b.enter(&mut state, &mut [0; 4096]);
            }
            assert_eq!(state[0], i as u64);
        }
        let survivor = blocks.remove(31);
        blocks.clear();
        assert!(weak.upgrade().is_some());
        let mut state = [0; 91];
        unsafe {
            survivor.enter(&mut state, &mut [0; 4096]);
        }
        assert_eq!(state[0], 31);
        drop(survivor);
        assert!(weak.upgrade().is_none());
        let mut large = Aarch64Enc::new();
        for _ in 0..20000 {
            large.put_raw(0xd503201f);
        }
        large.ret();
        let code = Code::new(&large).unwrap();
        assert!(code.page.size >= 80004);
        unsafe {
            code.enter(&mut state, &mut [0; 4096]);
        }
        assert_eq!(state[0], 31);
    }
    #[test]
    fn executes_generated_arm_and_releases_mapping() {
        let mut enc = Aarch64Enc::new();
        enc.mov_imm64(9, 0x123456789abcdef0);
        enc.str_x(9, 0, 0);
        enc.ret();
        let code = Code::new(&enc).unwrap();
        let mut state = [0; 91];
        unsafe {
            code.enter(&mut state, &mut [0; 4096]);
        }
        assert_eq!(state[0], 0x123456789abcdef0);
    }
}
