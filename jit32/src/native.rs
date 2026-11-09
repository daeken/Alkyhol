//! Apple Silicon executable storage. Deliberately avoids Tier0's Linux RWX finalizer.
use crate::Result;
use sharpretro_jit::aarch64_enc::Aarch64Enc;
unsafe extern "C" {
    fn pthread_jit_write_protect_np(enabled: i32);
    fn sys_icache_invalidate(start: *mut std::ffi::c_void, length: usize);
}
pub(crate) struct Code {
    base: *mut std::ffi::c_void,
    size: usize,
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
        let size = bytes.len().div_ceil(page) * page;
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
        unsafe {
            pthread_jit_write_protect_np(0);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), base.cast(), bytes.len());
            pthread_jit_write_protect_np(1);
            sys_icache_invalidate(base, bytes.len());
        }
        Ok(Self { base, size })
    }
    /// The checked compiler owns all emitted instructions and bounds both arrays.
    pub unsafe fn enter(&self, state: &mut [u64; 91], spill: &mut [u64; 4096]) {
        let entry: unsafe extern "C" fn(*mut u64, *mut u64) =
            unsafe { std::mem::transmute(self.base) };
        unsafe {
            entry(state.as_mut_ptr(), spill.as_mut_ptr());
        }
    }
}
impl Drop for Code {
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
