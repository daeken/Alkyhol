//! Native CPU plug-in boundary. Services retain their existing implementations.
use super::*;
use std::sync::OnceLock;
#[path = "exec_ranges32.rs"]
mod exec_ranges;
pub struct ExecutionContext32<'a> {
    image: &'a LoadedImage32<'a>,
    executable_regions: exec_ranges::StaticExecutableRegions<'a>,
}
pub type Execute32 = for<'a> fn(&ExecutionContext32<'a>, u32, u32) -> Result<u64>;
thread_local! { static EXCEPTION_REGISTERS: std::cell::Cell<Option<[u32; 9]>> = const { std::cell::Cell::new(None) }; }
pub(super) fn exception_registers() -> [u32; 9] {
    EXCEPTION_REGISTERS
        .get()
        .unwrap_or_else(|| unsupported("native software exception requires actual CPU registers"))
}
struct ExceptionSnapshot;
impl Drop for ExceptionSnapshot {
    fn drop(&mut self) {
        EXCEPTION_REGISTERS.set(None);
    }
}
static EXECUTOR: OnceLock<Execute32> = OnceLock::new();
pub fn install_executor32(executor: Execute32) -> Result<()> {
    EXECUTOR
        .set(executor)
        .map_err(|_| "PE32 executor already installed".into())
}
pub(super) fn installed() -> bool {
    EXECUTOR.get().is_some()
}
impl ExecutionContext32<'_> {
    pub fn location(&self, address: u32) -> String {
        if let Some(label) = self
            .image
            .cpu
            .diagnostic
            .as_ref()
            .and_then(|t| t.at(host_address(address as usize) as u64))
        {
            return format!("unimplemented import {label}");
        }
        self.image
            .cpu
            .modules
            .borrow()
            .iter()
            .find(|m| address >= m.base && (address as u64) < m.base as u64 + m.size as u64)
            .map(|m| format!("{}+{:#x}", m.name, address - m.base))
            .unwrap_or_else(|| format!("owned/runtime {address:#x}"))
    }
    pub fn memory_base(&self) -> u64 {
        GUEST_BIAS as u64
    }
    pub fn teb(&self) -> u32 {
        self.image.cpu.teb().base as u32
    }
    pub fn thread_key(&self) -> (u64, u32, u32) {
        (
            self.image.cpu.native_generation,
            self.teb(),
            alkyhol::ffi::GetCurrentThreadId(),
        )
    }

    pub fn return_gate(&self) -> u32 {
        self.image.gates.base as u32
    }
    /// Guest addresses are usable as host addresses only after this ownership,
    /// permission and 32-bit extent check. This is not arbitrary pointer casting.
    pub fn permits(&self, address: u64, length: usize, rights: u8) -> bool {
        if rights == 0
            || rights & !7 != 0
            || length == 0
            || address >= 0x100000000
            || address
                .checked_add(length as u64)
                .is_none_or(|v| v > 0x100000000)
        {
            return false;
        }
        if rights.count_ones() > 1 {
            return [1, 2, 4]
                .into_iter()
                .filter(|b| rights & b != 0)
                .all(|b| self.permits(address, length, b));
        }
        if rights & 4 != 0 {
            let end = address + length as u64;
            let host = host_address(address as usize) as u64;
            let host_end = host + length as u64;
            return self
                .executable_regions
                .contains(host as usize, host_end as usize)
                || self
                    .image
                    .cpu
                    .native_code
                    .borrow()
                    .iter()
                    .any(|&(a, b)| address >= a as u64 && end <= b as u64);
        }
        let host = host_address(address as usize);
        let end = host + length;
        let teb = self.image.cpu.teb();
        if host >= teb.base as usize && end <= teb.base as usize + teb.len {
            return true;
        }
        for stack in [&self.image.cpu.stack, self.image.cpu.stack()] {
            let a = stack.base as usize + PAGE;
            if host >= a && end <= a + STACK {
                return true;
            }
        }
        let mask = (if rights & 1 != 0 { 0x40000000 } else { 0 })
            | (if rights & 2 != 0 { 0x80000000 } else { 0 });
        if self
            .image
            .regions
            .iter()
            .any(|&(a, b, r)| r & mask == mask && host >= a && end <= (b + PAGE - 1) & !(PAGE - 1))
        {
            return true;
        }
        if self
            .image
            .cpu
            .heap
            .borrow()
            .cpu_permits(address as usize, length)
        {
            return true;
        }
        self.image
            .permits(address as usize, length, rights & 2 != 0)
    }
    pub fn read(&self, address: u32, length: usize, executable: bool) -> Result<Vec<u8>> {
        if !self.permits(address as u64, length, if executable { 4 } else { 1 }) {
            return Err(format!(
                "guest {} outside admitted storage: {address:#x}+{length:#x}; {}",
                if executable { "fetch" } else { "read" },
                self.location(address)
            ));
        }
        Ok(unsafe { std::slice::from_raw_parts(address as *const u8, length) }.to_vec())
    }
    pub fn code_equals(&self, address: u32, bytes: &[u8]) -> bool {
        self.permits(address as u64, bytes.len(), 4)
            && unsafe { std::slice::from_raw_parts(address as *const u8, bytes.len()) } == bytes
    }
    pub fn write(&self, address: u32, bytes: &[u8]) -> Result<()> {
        if !self.permits(address as u64, bytes.len(), 2) {
            return Err(format!(
                "guest write outside admitted storage: {address:#x}+{:#x}",
                bytes.len()
            ));
        }
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), address as *mut u8, bytes.len());
        }
        Ok(())
    }
    pub fn gateway(&self, address: u32) -> Option<(u32, u16)> {
        self.image.cpu.native_gates.borrow().get(&address).copied()
    }
    /// Enter the existing software SEH dispatcher with the actual native CPU snapshot.
    /// Nested delivery and changed continuations retain the ordinary explicit failure.
    pub fn dispatch_exception(
        &self,
        address: u32,
        stack: u32,
        regs: [u32; 9],
    ) -> Result<(u32, u16, bool)> {
        if self.gateway(address).map(|v| v.0) != Some(177)
            || regs[3].checked_add(4) != Some(stack)
            || !self.permits(stack as u64, 20, 1)
        {
            return Err("invalid native software exception entry snapshot".into());
        }
        if EXCEPTION_REGISTERS.get().is_some() {
            return Err("nested native software exception entry is unsupported".into());
        }
        EXCEPTION_REGISTERS.set(Some(regs));
        let _snapshot = ExceptionSnapshot;
        self.dispatch(address, stack)
    }
    /// Return actual EAX, cleanup byte count and non-returning-exit indication.
    pub fn dispatch(&self, address: u32, stack: u32) -> Result<(u32, u16, bool)> {
        let (id, bytes) = self
            .gateway(address)
            .ok_or("not a registered service gate")?;
        let Some((_, argument_bytes)) = APIS
            .get(id as usize)
            .copied()
            .or_else(|| d3d9_32::signature(id))
            .or_else(|| input8_32::signature(id))
        else {
            return Err("unregistered service signature".into());
        };
        if !self.permits(stack as u64, 4 + argument_bytes as usize, 1) {
            return Err("service frame outside admitted guest storage".into());
        }
        let result = dispatch(id, (stack + 4) as *const u32);
        Ok((result as u32, bytes, result >> 32 != 0))
    }
}
pub(super) unsafe fn enter(entry: u32, stack: u32, _code: u32, _data: u32) -> u64 {
    ACTIVE.with(|slot| {
        let image = unsafe { slot.get().cast::<LoadedImage32<'_>>().as_ref() }
            .unwrap_or_else(|| unsupported("no active emulated PE32 image"));
        let execute = EXECUTOR
            .get()
            .unwrap_or_else(|| unsupported("no native PE32 CPU executor installed"));
        execute(
            &ExecutionContext32 {
                image,
                executable_regions: exec_ranges::StaticExecutableRegions::new(&image.regions),
            },
            entry,
            stack,
        )
        .unwrap_or_else(|e| unsupported(&e))
    })
}
