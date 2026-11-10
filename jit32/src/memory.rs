//! Guest addresses are offsets into owned backing, never host pointers.
use crate::Result;

pub const READ: u8 = 1;
pub const WRITE: u8 = 2;
pub const EXEC: u8 = 4;
const EXTENT: usize = 1usize << 32;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    pub address: u64,
    pub length: usize,
    pub access: u8,
}
#[derive(Clone, Debug)]
struct Region {
    start: u64,
    end: u64,
    rights: u8,
}
#[cfg(feature = "shared-runtime")]
struct Foreign {
    context: *const (),
    validate: unsafe fn(*const (), u64, usize, u8) -> bool,
}
pub struct Memory {
    base: *mut u8,
    regions: Vec<Region>,
    pub(crate) fpu: std::rc::Rc<std::cell::RefCell<crate::fpu::State>>,
    #[cfg(feature = "shared-runtime")]
    foreign: Option<Foreign>,
    pub(crate) fault: Option<Fault>,
    pub(crate) cpu_fault: Option<&'static str>,
}
impl Memory {
    pub fn new() -> Result<Self> {
        // Sparse virtual backing: only mapped guest bytes are ever accessed.
        let base = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                EXTENT,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            )
        };
        if base == libc::MAP_FAILED {
            return Err(format!(
                "guest reservation: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(Self {
            base: base.cast(),
            regions: Vec::new(),
            fpu: Default::default(),
            #[cfg(feature = "shared-runtime")]
            foreign: None,
            fault: None,
            cpu_fault: None,
        })
    }
    /// Only used within a synchronous checked runtime invocation. The supplied
    /// context must outlive this view and validate real owned native mappings.
    #[cfg(feature = "shared-runtime")]
    pub(crate) unsafe fn foreign(
        context: *const (),
        base: u64,
        validate: unsafe fn(*const (), u64, usize, u8) -> bool,
    ) -> Self {
        Self {
            base: base as *mut u8,
            regions: vec![],
            fpu: Default::default(),
            fault: None,
            cpu_fault: None,
            foreign: Some(Foreign { context, validate }),
        }
    }
    fn pointer(&self, address: u64) -> *mut u8 {
        #[cfg(feature = "shared-runtime")]
        if self.foreign.is_some() {
            return (self.base as u64 + address) as *mut u8;
        }
        // Every caller validates the offset before translating it.
        unsafe { self.base.add(address as usize) }
    }
    pub(crate) fn base(&self) -> u64 {
        self.base as u64
    }
    pub fn check(&self, address: u64, length: usize, access: u8) -> std::result::Result<(), Fault> {
        let fail = || Fault {
            address,
            length,
            access,
        };
        let end = address
            .checked_add(length as u64)
            .filter(|e| *e <= EXTENT as u64)
            .ok_or_else(fail)?;
        if length == 0 || access == 0 || access & !(READ | WRITE | EXEC) != 0 {
            return Err(fail());
        }
        #[cfg(feature = "shared-runtime")]
        if let Some(foreign) = &self.foreign {
            return if unsafe { (foreign.validate)(foreign.context, address, length, access) } {
                Ok(())
            } else {
                Err(fail())
            };
        }
        // Walk adjacent regions too; instruction/data spans may cross sections.
        let mut at = address;
        for r in &self.regions {
            if r.end <= at {
                continue;
            }
            if r.start > at || r.rights & access != access {
                return Err(fail());
            }
            at = r.end.min(end);
            if at == end {
                return Ok(());
            }
        }
        Err(fail())
    }
    pub fn map(&mut self, address: u32, size: usize, rights: u8, initial: &[u8]) -> Result<()> {
        #[cfg(feature = "shared-runtime")]
        if self.foreign.is_some() {
            return Err("cannot map through a borrowed guest view".into());
        }
        let start = address as u64;
        let end = start
            .checked_add(size as u64)
            .filter(|e| *e <= EXTENT as u64)
            .ok_or("guest map overflow")?;
        if size == 0
            || initial.len() > size
            || rights == 0
            || rights & !(READ | WRITE | EXEC) != 0
            || rights & (WRITE | EXEC) == WRITE | EXEC
        {
            return Err("invalid or writable-executable guest map".into());
        }
        if self.regions.iter().any(|r| start < r.end && r.start < end) {
            return Err("overlapping guest maps".into());
        }
        // SAFETY: disjoint newly mapped logical range inside owned 4GiB backing.
        unsafe {
            std::ptr::copy_nonoverlapping(
                initial.as_ptr(),
                self.base.add(address as usize),
                initial.len(),
            );
        }
        self.regions.push(Region { start, end, rights });
        self.regions.sort_by_key(|r| r.start);
        Ok(())
    }
    pub(crate) fn shrink(&mut self, address: u32, old: usize, new: usize) -> Result<()> {
        if new == 0 || new > old {
            return Err("invalid guest region shrink".into());
        }
        let region = self
            .regions
            .iter_mut()
            .find(|r| r.start == address as u64 && r.end == address as u64 + old as u64)
            .ok_or("shrink requires exact owned region")?;
        region.end = region.start + new as u64;
        Ok(())
    }
    pub(crate) fn unmap(&mut self, address: u32, size: usize) -> Result<()> {
        let index = self
            .regions
            .iter()
            .position(|r| r.start == address as u64 && r.end == address as u64 + size as u64)
            .ok_or("unmap requires exact owned region")?;
        self.regions.remove(index);
        Ok(())
    }
    pub fn read(&self, address: u64, size: usize, access: u8) -> Result<&[u8]> {
        self.check(address, size, access)
            .map_err(|f| format!("guest fault: {f:?}"))?;
        Ok(unsafe { std::slice::from_raw_parts(self.pointer(address), size) })
    }
    pub fn write(&mut self, address: u64, bytes: &[u8]) -> Result<()> {
        self.check(address, bytes.len(), WRITE)
            .map_err(|f| format!("guest fault: {f:?}"))?;
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.pointer(address), bytes.len());
        }
        Ok(())
    }
    pub fn word(&self, address: u64) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.read(address, 4, READ)?.try_into().unwrap(),
        ))
    }
}
impl Drop for Memory {
    fn drop(&mut self) {
        #[cfg(feature = "shared-runtime")]
        if self.foreign.is_some() {
            return;
        }
        unsafe {
            if !self.base.is_null() {
                libc::munmap(self.base.cast(), EXTENT);
            }
        }
    }
}

/// Called by emitted ARM before each memory operation. No Rust references to the
/// backing are live across entry to generated code. A failed check exits the
/// instruction before the native load/store and preserves the first fault.
pub(crate) unsafe extern "C" fn guard(
    context: *mut Memory,
    address: u64,
    size: u64,
    write: u64,
) -> u64 {
    let mem = unsafe { &mut *context };
    match mem.check(
        address,
        size as usize,
        if write == 0 { READ } else { WRITE },
    ) {
        Ok(()) => 1,
        Err(fault) => {
            if mem.fault.is_none() {
                mem.fault = Some(fault);
            }
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boundaries_permissions_and_real_bytes() {
        let mut m = Memory::new().unwrap();
        m.map(0x1000, 4, READ | WRITE, &[1, 2, 3, 4]).unwrap();
        m.map(0x1004, 4, READ, &[5, 6, 7, 8]).unwrap();
        assert_eq!(m.read(0x1002, 4, READ).unwrap(), [3, 4, 5, 6]);
        assert!(m.write(0x1002, &[9; 4]).is_err());
        assert!(m.check(0xfff, 2, READ).is_err());
        assert!(m.check(u64::MAX, 2, READ).is_err());
        assert!(m.check(0xffff_ffff, 2, READ).is_err());
        assert!(m.read(0, 1, READ).is_err());
        assert!(m.check(0x1000, 1, EXEC).is_err());
        assert!(m.map(0x1001, 1, READ, &[]).is_err());
        assert!(m.map(0x2000, 1, WRITE | EXEC, &[]).is_err());
        m.write(0x1000, &[9, 8, 7, 6]).unwrap();
        assert_eq!(m.word(0x1000).unwrap(), 0x06070809);
        unsafe {
            assert_eq!(guard(&mut m, 0, 4, 0), 0);
            assert_eq!(guard(&mut m, 0x1004, 4, 1), 0);
        }
        assert_eq!(m.fault.as_ref().unwrap().address, 0);
    }
}

/// x86 DIV/IDIV raise #DE for zero divisors and unrepresentable quotients.
/// AArch64 division does neither, so guard before touching the guest result.
pub(crate) unsafe extern "C" fn divide_guard(
    context: *mut Memory,
    numerator: u64,
    denominator: u64,
    signed: u64,
) -> u64 {
    let valid = if signed != 0 {
        let d = denominator as i64 as i128;
        d != 0 && {
            let q = (numerator as i64 as i128) / d;
            (i32::MIN as i128..=i32::MAX as i128).contains(&q)
        }
    } else {
        denominator != 0 && numerator / denominator <= u32::MAX as u64
    };
    if valid {
        1
    } else {
        unsafe {
            (*context)
                .cpu_fault
                .get_or_insert("x86 #DE: zero divisor or quotient overflow");
        }
        0
    }
}

pub(crate) unsafe extern "C" fn atomic_alignment(
    memory: *mut Memory,
    address: u64,
    size: u64,
) -> u64 {
    if address.is_multiple_of(size) {
        1
    } else {
        unsafe {
            (*memory)
                .cpu_fault
                .get_or_insert("unaligned x86 atomic requires a split-lock implementation");
        }
        0
    }
}

pub(crate) unsafe extern "C" fn pop_flags(memory: *mut Memory, flags: u64) -> u64 {
    // TF delivery, nested-task state and alignment-check faults require CPU
    // exception support. Do not silently drop a guest request to enable them.
    if flags & ((1 << 8) | (1 << 14) | (1 << 18)) != 0 {
        unsafe {
            (*memory)
                .cpu_fault
                .get_or_insert("POPF requests unsupported TF/NT/AC exception state");
        }
        0
    } else {
        1
    }
}

pub(crate) unsafe extern "C" fn vector_alignment(memory: *mut Memory, address: u64) -> u64 {
    if address.is_multiple_of(16) {
        1
    } else {
        unsafe {
            (*memory)
                .cpu_fault
                .get_or_insert("x86 #GP: aligned vector access is misaligned");
        }
        0
    }
}
