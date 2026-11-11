//! Real first-pass x86 software SEH delivery; unchanged-context continuation only.
use super::{CallingConvention32, LoadedImage32, PAGE, STACK, unsupported};
use crate::Result;
use std::cell::Cell;
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
struct Delivery(u32);
impl Drop for Delivery {
    fn drop(&mut self) {
        ACTIVE.set(false);
        alkyhol::ffi::SetLastError(self.0);
    }
}
#[cfg(target_arch = "x86_64")]
std::arch::global_asm!(include_str!("raise32.S"), options(att_syntax));
#[cfg(target_arch = "x86_64")]
unsafe extern "C" {
    static alkyhol_raise32_start: u8;
    static alkyhol_raise32_end: u8;
    static alkyhol_raise32_service: u8;
}
pub(super) const OFFSET: usize = 2 * PAGE;
pub(super) fn emit(bytes: &mut [u8], base: u32, service: u32) -> Result<()> {
    #[cfg(target_arch = "x86_64")]
    let (first, last, slot) = {
        let first = std::ptr::addr_of!(alkyhol_raise32_start) as usize;
        let last = std::ptr::addr_of!(alkyhol_raise32_end) as usize;
        let slot = std::ptr::addr_of!(alkyhol_raise32_service) as usize - first;
        (first, last, slot)
    };
    #[cfg(target_arch = "aarch64")]
    let (first, last, slot) = {
        let bytes = super::guest_templates::RAISE;
        (
            bytes.as_ptr() as usize,
            bytes.as_ptr() as usize + bytes.len(),
            super::guest_templates::RAISE_PATCH,
        )
    };
    let size = last
        .checked_sub(first)
        .filter(|&v| v <= 256)
        .ok_or("invalid software SEH template")?;
    if slot + 4 > size || OFFSET + 288 > bytes.len() {
        return Err("software SEH template exceeds backing".into());
    }
    bytes[OFFSET..OFFSET + size]
        .copy_from_slice(unsafe { std::slice::from_raw_parts(first as *const u8, size) });
    bytes[OFFSET + slot..OFFSET + slot + 4].copy_from_slice(&service.to_le_bytes());
    if base.checked_add(bytes.len() as u32).is_none() {
        return Err("software SEH gate address overflow".into());
    }
    Ok(())
}
fn load<const N: usize>(image: &LoadedImage32<'_>, at: u32) -> [u32; N] {
    if !at.is_multiple_of(4) || !image.permits(at as usize, N * 4, false) {
        unsupported("software SEH input outside checked guest storage");
    }
    unsafe { (at as *const [u32; N]).read() }
}
struct Storage<'a> {
    image: &'a LoadedImage32<'a>,
    heap: u32,
    at: u32,
}
impl Drop for Storage<'_> {
    fn drop(&mut self) {
        let error = alkyhol::ffi::GetLastError();
        let _ = self.image.cpu.heap.borrow_mut().free(self.heap, 0, self.at);
        alkyhol::ffi::SetLastError(error);
    }
}
pub(super) fn deliver(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if a[2] != 0x53454832
        || a[3] != 0
        || !a[1].is_multiple_of(16)
        || !image.permits(a[1] as usize, 512, true)
    {
        unsupported("software SEH requires its checked CPU entry snapshot");
    }
    deliver_registers(image, load::<9>(image, a[0]))
}
/// Real pushad/flags snapshot, supplied either by the x86 wrapper or native CPU.
pub(super) fn deliver_registers(image: &LoadedImage32<'_>, regs: [u32; 9]) -> u32 {
    if ACTIVE.replace(true) {
        unsupported("nested software exception dispatch is unsupported");
    }
    let _delivery = Delivery(alkyhol::ffi::GetLastError());
    let call_at = regs[3]
        .checked_add(4)
        .unwrap_or_else(|| unsupported("software exception stack overflow"));
    let input = load::<5>(image, call_at);
    let [caller, code, flags, count, parameters] = input;
    if flags != 0 || count > 15 {
        unsupported("noncontinuable or oversized software exception is unsupported");
    }
    let mut info = [0u32; 15];
    if count != 0 {
        if !parameters.is_multiple_of(4)
            || !image.permits(parameters as usize, count as usize * 4, false)
        {
            unsupported("software exception parameters outside checked storage");
        }
        unsafe {
            std::ptr::copy_nonoverlapping(
                parameters as *const u32,
                info.as_mut_ptr(),
                count as usize,
            );
        }
    }
    if image.cpu.diagnostic.is_some() {
        eprintln!(
            "PE32 software exception: code={code:08x}; flags={flags:x}; count={count}; parameters={:x?}",
            &info[..count as usize]
        );
    }
    let mut context = [0u32; 179];
    context[0] = 0x10007;
    context[35..39].copy_from_slice(&[
        0,
        u32::from(super::thread32::guest_fs()),
        u32::from(image.data),
        u32::from(image.data),
    ]);
    context[39..47].copy_from_slice(&[
        regs[0], regs[1], regs[4], regs[5], regs[6], regs[7], regs[2], caller,
    ]);
    context[47] = u32::from(image.code);
    context[48] = regs[8];
    context[49] = call_at
        .checked_add(20)
        .unwrap_or_else(|| unsupported("software continuation stack overflow"));
    context[50] = u32::from(image.data);
    let mut record = [0u32; 20];
    record[0] = code;
    record[1] = flags;
    record[3] = caller;
    record[4] = count;
    record[5..].copy_from_slice(&info);
    let heap = image
        .cpu
        .heap
        .borrow_mut()
        .handle()
        .unwrap_or_else(|e| unsupported(&e));
    let at = image
        .cpu
        .heap
        .borrow_mut()
        .allocate(heap, 0, 80 + 716)
        .unwrap_or_else(|e| unsupported(&e));
    if at == 0 {
        unsupported("software SEH scratch allocation failed");
    }
    let _storage = Storage { image, heap, at };
    let context_at = at + 80;
    unsafe {
        std::ptr::copy_nonoverlapping(record.as_ptr(), at as *mut u32, 20);
        std::ptr::copy_nonoverlapping(context.as_ptr(), context_at as *mut u32, 179);
    }
    let head = unsafe { image.cpu.teb().base.cast::<u32>().read_unaligned() };
    let mut frame = head;
    let first = image.cpu.stack().base as u32 as usize + PAGE;
    let last = first + STACK;
    for _ in 0..128 {
        if frame == u32::MAX {
            unsupported("unhandled software exception (no fabricated continuation)");
        }
        if !frame.is_multiple_of(4)
            || (frame as usize) < first
            || (frame as usize) + 8 > last
            || frame < call_at
        {
            unsupported("invalid software SEH registration chain");
        }
        let [next, handler] = load::<2>(image, frame);
        if next != u32::MAX
            && (next <= frame || !next.is_multiple_of(4) || next as usize + 8 > last)
        {
            unsupported("cyclic or invalid software SEH chain");
        }
        let disposition = unsafe {
            image.callback_frame(
                handler,
                &[at, frame, context_at, 0],
                CallingConvention32::Cdecl,
            )
        }
        .unwrap_or_else(|e| unsupported(&e))
        .unwrap_or_else(|| {
            unsupported("non-returning software SEH handler requires unwind support")
        });
        if load::<179>(image, context_at) != context
            || load::<20>(image, at) != record
            || load::<2>(image, frame) != [next, handler]
            || unsafe { image.cpu.teb().base.cast::<u32>().read_unaligned() } != head
        {
            unsupported(
                "software SEH mutated context/record/chain requires further dispatch support",
            );
        }
        match disposition {
            0 => return 0,
            1 => frame = next,
            _ => unsupported("nested or collided software exception dispatch is unsupported"),
        }
    }
    unsupported("software SEH traversal budget exhausted")
}
