//! Opt-in, integer-only UD2 SEH continuation. Native ABI contracts: docs/pe32.md.
//! No default-loader behavior changes and no nested/unwind/software exceptions.
use super::{LoadedImage32, PAGE, STACK, alkyhol_enter_pe32, mmap, mprotect, munmap};
use crate::{ProcessContext, Result};
use std::{
    ffi::c_void,
    marker::PhantomData,
    ptr,
    sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, AtomicU32, AtomicUsize, Ordering},
};

const SIGILL: i32 = 4;
const SIGNAL_STACK: usize = 131072;
const SIGNAL_MAPPING: usize = SIGNAL_STACK + 2 * PAGE;
const CONTEXT_FLAGS: u32 = 0x10007;
static OWNER: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static DIAGNOSTIC: AtomicBool = AtomicBool::new(false);
static HANDLING: AtomicBool = AtomicBool::new(false);
struct SavedAction {
    handler: AtomicUsize,
    mask: AtomicU32,
    flags: AtomicI32,
}
impl SavedAction {
    const fn new() -> Self {
        Self {
            handler: AtomicUsize::new(0),
            mask: AtomicU32::new(0),
            flags: AtomicI32::new(0),
        }
    }
    fn store(&self, a: Action) {
        self.handler.store(a.handler, Ordering::SeqCst);
        self.mask.store(a.mask, Ordering::SeqCst);
        self.flags.store(a.flags, Ordering::SeqCst);
    }
    fn load(&self) -> Action {
        Action {
            handler: self.handler.load(Ordering::SeqCst),
            mask: self.mask.load(Ordering::SeqCst),
            flags: self.flags.load(Ordering::SeqCst),
        }
    }
}
static PRIOR: [SavedAction; 2] = [SavedAction::new(), SavedAction::new()];
static ALT_FIRST: AtomicUsize = AtomicUsize::new(0);
static ALT_LAST: AtomicUsize = AtomicUsize::new(0);

// Interface layouts are independently asserted against the installed Darwin SDK.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Action {
    handler: usize,
    mask: u32,
    flags: i32,
}
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
struct AltStack {
    address: *mut c_void,
    size: usize,
    flags: i32,
}
#[repr(C)]
struct UserContext {
    onstack: i32,
    mask: u32,
    stack: AltStack,
    link: *mut c_void,
    machine_size: usize,
    machine: *mut MachinePrefix,
}
#[repr(C)]
struct MachinePrefix {
    exception: [u8; 16],
    registers: [u64; 21],
}
#[repr(C)]
struct SignalInfo {
    signal: i32,
    error: i32,
    code: i32,
}
#[repr(C, align(4))]
#[derive(Clone, Copy, PartialEq, Eq)]
struct Context32 {
    words: [u32; 179],
}
unsafe extern "C" {
    fn sigaction(signal: i32, action: *const Action, old: *mut Action) -> i32;
    fn sigaltstack(stack: *const AltStack, old: *mut AltStack) -> i32;
    fn pthread_sigmask(how: i32, set: *const u32, old: *mut u32) -> i32;
    fn raise(signal: i32) -> i32;
    fn write(fd: i32, buffer: *const c_void, len: usize) -> isize;
    fn _exit(status: i32) -> !;
}
pub(super) fn async_fail(message: &'static [u8]) -> ! {
    unsafe {
        write(2, message.as_ptr().cast(), message.len());
        _exit(78)
    }
}
pub(super) fn in_handler() -> bool {
    HANDLING.load(Ordering::Relaxed)
}

/// Explicit bounded hardware-exception profile for a trusted i386 EXE.
/// Supports owned-image cdecl frame handlers for UD2, ContinueSearch and integer
/// ContinueExecution only. Imported calls from a handler, floating/debug context,
/// nesting, unwinding, other faults and software RaiseException are unsupported.
/// Standard LoadedImage32 execution retains its prior raw-signal behavior.
///
/// ```compile_fail,E0505
/// # use alkyhol_pe::{ExceptionImage32,ProcessContext};
/// # fn example(bytes: &[u8])->alkyhol_pe::Result<()> {
/// let process=ProcessContext::install(&["exceptions32".into()],&[])?;
/// let image=ExceptionImage32::load(bytes,false,&process)?;
/// drop(process);
/// drop(image);
/// # Ok(()) }
/// ```
pub struct ExceptionImage32<'a> {
    image: LoadedImage32<'a>,
}
impl<'a> ExceptionImage32<'a> {
    /// Validate/map the EXE, retaining its checked SafeSEH table for actual use.
    /// No guest instruction runs until enter. NO_SEH images are rejected.
    pub fn load(bytes: &[u8], rebase: bool, process: &'a ProcessContext) -> Result<Self> {
        Ok(Self {
            image: LoadedImage32::load_kind(bytes, rebase, process, false, &[], true)?,
        })
    }
    pub fn relocated(&self) -> bool {
        self.image.relocated()
    }
    /// Execute under the bounded exception profile, restoring host signal state
    /// and the previous alternate stack before releasing any owned storage.
    ///
    /// # Safety
    /// The trusted EXE/handlers must obey this profile's stated restrictions,
    /// the ordinary native-code contract and retain no escaping guest pointers.
    /// Callers must serialize changes to the process's signal configuration
    /// throughout entry. A blocked/default-incompatible SIGILL state is rejected.
    pub unsafe fn enter(&self) -> Result<u32> {
        let _signals = SignalGuard::install(&self.image)?;
        unsafe { self.image.enter() }
    }
}

pub(super) struct SignalGuard<'a> {
    memory: *mut c_void,
    old_action: Action,
    extra_actions: [Action; 2],
    extra_set: usize,
    old_stack: AltStack,
    claimed: bool,
    action_set: bool,
    stack_set: bool,
    _borrow: PhantomData<&'a LoadedImage32<'a>>,
}
impl<'a> SignalGuard<'a> {
    fn install(image: &'a LoadedImage32<'_>) -> Result<Self> {
        Self::install_mode(image, false)
    }
    pub(super) fn install_diagnostic(image: &'a LoadedImage32<'_>) -> Result<Self> {
        Self::install_mode(image, true)
    }
    #[cfg(target_arch="aarch64")]
    fn install_mode(_image:&'a LoadedImage32<'_>,_diagnostic:bool)->Result<Self>{
        Err("native x86 signal continuation is unavailable with the ARM executor".into())
    }
    #[cfg(target_arch="x86_64")]
    fn install_mode(image: &'a LoadedImage32<'_>, diagnostic: bool) -> Result<Self> {
        let mut mask = 0u32;
        if unsafe { pthread_sigmask(3, ptr::null(), &mut mask) } != 0
            || mask & (1 << (SIGILL - 1)) != 0
        {
            return Err("PE32 exception profile requires unblocked host SIGILL".into());
        }
        let mut old_action = Action::default();
        let mut old_stack = AltStack {
            address: ptr::null_mut(),
            size: 0,
            flags: 0,
        };
        if unsafe { sigaction(SIGILL, ptr::null(), &mut old_action) } != 0
            || old_action.handler != 0
        {
            return Err("PE32 exception profile requires the default host SIGILL action".into());
        }
        if unsafe { sigaltstack(ptr::null(), &mut old_stack) } != 0 || old_stack.flags & 1 != 0 {
            return Err("cannot inspect/change the host alternate stack".into());
        }
        let mut extra_actions = [Action::default(); 2];
        if diagnostic {
            for (index, number) in [10, 11].into_iter().enumerate() {
                if mask & (1 << (number - 1)) != 0
                    || unsafe { sigaction(number, ptr::null(), &mut extra_actions[index]) } != 0
                    || extra_actions[index].handler == 1
                    || extra_actions[index].flags & 4 != 0
                {
                    return Err(
                        "diagnostic startup requires unblocked, non-ignored SIGBUS/SIGSEGV".into(),
                    );
                }
            }
        }
        let memory = unsafe {
            mmap(
                0x200000000usize as *mut c_void,
                SIGNAL_MAPPING,
                3,
                0x1002,
                -1,
                0,
            )
        };
        if memory as isize == -1 {
            return Err("cannot allocate native exception stack".into());
        }
        if memory as usize <= u32::MAX as usize {
            unsafe {
                munmap(memory, SIGNAL_MAPPING);
            }
            return Err("native exception stack must reside above 4 GiB".into());
        }
        let mut guard = Self {
            memory,
            old_action,
            extra_actions,
            extra_set: 0,
            old_stack,
            claimed: false,
            action_set: false,
            stack_set: false,
            _borrow: PhantomData,
        };
        if unsafe { mprotect(memory, PAGE, 0) } != 0
            || unsafe { mprotect(memory.byte_add(PAGE + SIGNAL_STACK), PAGE, 0) } != 0
        {
            return Err("cannot guard the native exception stack".into());
        }
        OWNER
            .compare_exchange(
                ptr::null_mut(),
                ptr::from_ref(image).cast_mut().cast(),
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .map_err(|_| "another PE32 exception entry is active")?;
        guard.claimed = true;
        if diagnostic {
            for (slot, action) in PRIOR.iter().zip(guard.extra_actions) {
                slot.store(action);
            }
        }
        DIAGNOSTIC.store(diagnostic, Ordering::SeqCst);
        ALT_FIRST.store(memory as usize + PAGE, Ordering::SeqCst);
        ALT_LAST.store(memory as usize + PAGE + SIGNAL_STACK, Ordering::SeqCst);
        let stack = AltStack {
            address: unsafe { memory.byte_add(PAGE) },
            size: SIGNAL_STACK,
            flags: 0,
        };
        if unsafe { sigaltstack(&stack, ptr::null_mut()) } != 0 {
            return Err("cannot install native exception stack".into());
        }
        guard.stack_set = true;
        let action = Action {
            handler: signal as *const () as usize,
            mask: 0,
            flags: 0x51, // SA_ONSTACK | SA_SIGINFO | SA_NODEFER.
        };
        if unsafe { sigaction(SIGILL, &action, ptr::null_mut()) } != 0 {
            return Err("cannot install PE32 SIGILL bridge".into());
        }
        guard.action_set = true;
        if diagnostic {
            for (index, number) in [10, 11].into_iter().enumerate() {
                let prior = guard.extra_actions[index];
                let action = Action {
                    handler: signal as *const () as usize,
                    mask: prior.mask,
                    flags: 0x41 | (prior.flags & 0x12),
                };
                if unsafe { sigaction(number, &action, ptr::null_mut()) } != 0 {
                    return Err("cannot install diagnostic fault guard".into());
                }
                guard.extra_set += 1;
            }
        }
        Ok(guard)
    }
}
impl Drop for SignalGuard<'_> {
    fn drop(&mut self) {
        for index in (0..self.extra_set).rev() {
            let number = [10, 11][index];
            let expected = &self.extra_actions[index];
            let mut actual = Action::default();
            if unsafe { sigaction(number, expected, ptr::null_mut()) } != 0
                || unsafe { sigaction(number, ptr::null(), &mut actual) } != 0
                || actual != *expected
            {
                async_fail(
                    b"Alkyhol PE32 unsupported: cannot restore diagnostic host signal action\n",
                );
            }
        }
        if self.action_set && unsafe { sigaction(SIGILL, &self.old_action, ptr::null_mut()) } != 0 {
            async_fail(b"Alkyhol PE32 unsupported: cannot restore host SIGILL action\n");
        }
        if self.stack_set && unsafe { sigaltstack(&self.old_stack, ptr::null_mut()) } != 0 {
            async_fail(b"Alkyhol PE32 unsupported: cannot restore host alternate stack\n");
        }
        if self.action_set {
            let mut actual = Action::default();
            if unsafe { sigaction(SIGILL, ptr::null(), &mut actual) } != 0
                || actual != self.old_action
            {
                async_fail(b"Alkyhol PE32 unsupported: restored SIGILL action differs\n");
            }
        }
        if self.stack_set {
            let mut actual = self.old_stack;
            if unsafe { sigaltstack(ptr::null(), &mut actual) } != 0 || actual != self.old_stack {
                async_fail(b"Alkyhol PE32 unsupported: restored alternate stack differs\n");
            }
        }
        if unsafe { munmap(self.memory, SIGNAL_MAPPING) } != 0 {
            async_fail(b"Alkyhol PE32 unsupported: cannot release native exception stack\n");
        }
        if self.claimed {
            ALT_FIRST.store(0, Ordering::SeqCst);
            ALT_LAST.store(0, Ordering::SeqCst);
            DIAGNOSTIC.store(false, Ordering::SeqCst);
            OWNER.store(ptr::null_mut(), Ordering::SeqCst);
        }
    }
}
extern "C" fn signal(number: i32, info: *const SignalInfo, context: *mut UserContext) {
    let mut fs = 0u16;
    #[cfg(target_arch="x86_64")]
    unsafe {
        std::arch::asm!("mov {0:x}, fs",out(reg) fs,options(nomem,nostack,preserves_flags));
    }
    if fs != 0 {
        async_fail(b"Alkyhol PE32 unsupported: native signal entry has active FS\n");
    }
    let owner = OWNER.load(Ordering::SeqCst);
    let Some(image) = (unsafe { owner.cast::<LoadedImage32<'_>>().as_ref() }) else {
        forward_prior(number, info, context);
        return;
    };
    let Some(uc) = (unsafe { context.as_mut() }) else {
        forward_prior(number, info, context);
        return;
    };
    let diagnostic = DIAGNOSTIC.load(Ordering::SeqCst);
    if (!diagnostic && number != SIGILL)
        || (diagnostic && ![4, 10, 11].contains(&number))
        || info.is_null()
        || (!diagnostic && unsafe { (*info).code } != 1)
        || uc.machine_size < std::mem::size_of::<MachinePrefix>()
        || uc.machine.is_null()
    {
        forward_prior(number, info, context);
        return;
    }
    let machine = unsafe { &mut *uc.machine };
    if machine.registers[18] != u64::from(image.code) {
        forward_prior(number, info, context);
        return;
    }
    // The process action may briefly coexist with another thread's ordinary
    // entry before this profile acquires the exclusive FS lease. Never route
    // that thread's fault through this image's handler table or memory view.
    if super::ACTIVE.with(|slot| slot.get()) != owner.cast_const() {
        forward_prior(number, info, context);
        return;
    }
    if diagnostic {
        super::diagnostic32::fault(image, &machine.exception, &machine.registers);
    }

    if HANDLING.swap(true, Ordering::SeqCst) {
        async_fail(b"Alkyhol PE32 unsupported: nested hardware exception\n");
    }
    #[cfg(target_arch="aarch64")]
    let native_sp=0usize;
    #[cfg(target_arch="x86_64")]
    let native_sp: usize;
    #[cfg(target_arch="x86_64")]
    unsafe {
        std::arch::asm!("mov {}, rsp",out(reg) native_sp,options(nomem,nostack,preserves_flags));
    }
    if native_sp < ALT_FIRST.load(Ordering::SeqCst) || native_sp >= ALT_LAST.load(Ordering::SeqCst)
    {
        async_fail(b"Alkyhol PE32 unsupported: signal did not enter the owned high native stack\n");
    }
    if let Err(message) = unsafe { dispatch_exception(image, machine) } {
        async_fail(message);
    }
    HANDLING.store(false, Ordering::SeqCst);
}
fn forward_prior(number: i32, info: *const SignalInfo, context: *mut UserContext) {
    if DIAGNOSTIC.load(Ordering::SeqCst) && [10, 11].contains(&number) {
        let action = PRIOR[if number == 10 { 0 } else { 1 }].load();
        if action.handler > 1 {
            if action.flags & 0x40 != 0 {
                let call: extern "C" fn(i32, *const SignalInfo, *mut UserContext) =
                    unsafe { std::mem::transmute(action.handler) };
                call(number, info, context);
            } else {
                let call: extern "C" fn(i32) = unsafe { std::mem::transmute(action.handler) };
                call(number);
            }
            return;
        }
    }
    forward_default(number);
}
fn forward_default(number: i32) {
    if unsafe { sigaction(number, &Action::default(), ptr::null_mut()) } != 0
        || unsafe { raise(number) } != 0
    {
        async_fail(b"Alkyhol PE32 unsupported: cannot forward unrelated host signal\n");
    }
}
type FaultResult<T> = std::result::Result<T, &'static [u8]>;
fn code(image: &LoadedImage32<'_>, address: u32, len: usize) -> bool {
    let start = address as usize;
    image.code_regions.iter().any(|&(first, last)| {
        start >= first && start.checked_add(len).is_some_and(|end| end <= last)
    })
}
fn stack(image: &LoadedImage32<'_>, address: u32, len: usize) -> bool {
    let first = image.cpu.stack().base as usize + PAGE;
    let start = address as usize;
    start >= first
        && start
            .checked_add(len)
            .is_some_and(|end| end <= first + STACK)
}
fn context32(image: &LoadedImage32<'_>, regs: &[u64; 21]) -> Context32 {
    let mut words = [0u32; 179];
    words[0] = CONTEXT_FLAGS;
    words[35..39].copy_from_slice(&[
        regs[20] as u32,
        regs[19] as u32,
        u32::from(image.data),
        u32::from(image.data),
    ]);
    for (index, register) in [
        (39, 4),
        (40, 5),
        (41, 1),
        (42, 3),
        (43, 2),
        (44, 0),
        (45, 6),
        (46, 16),
        (47, 18),
        (48, 17),
        (49, 7),
    ] {
        words[index] = regs[register] as u32;
    }
    words[50] = u32::from(image.data);
    Context32 { words }
}
fn validate_context(image: &LoadedImage32<'_>, original: &Context32, changed: &Context32) -> bool {
    let a = &original.words;
    let b = &changed.words;
    b[0] == CONTEXT_FLAGS
        && b[1..39] == a[1..39]
        && b[47] == a[47]
        && b[50..] == a[50..]
        && (b[48] ^ a[48]) & !0xcd5 == 0
        && code(image, b[46], 1)
        && b[49].is_multiple_of(4)
        && stack(image, b[49], 4)
}
fn registration_allowed(
    image: &LoadedImage32<'_>,
    sp: u32,
    frame: u32,
    next: u32,
    handler: u32,
) -> bool {
    frame >= sp
        && frame.is_multiple_of(4)
        && stack(image, frame, 8)
        && (next == u32::MAX || (next > frame && next.is_multiple_of(4) && stack(image, next, 8)))
        && code(image, handler, 1)
        && (image.seh_handlers.is_empty() || image.seh_handlers.binary_search(&handler).is_ok())
}
unsafe fn dispatch_exception(
    image: &LoadedImage32<'_>,
    machine: &mut MachinePrefix,
) -> FaultResult<()> {
    let pc = u32::try_from(machine.registers[16])
        .map_err(|_| &b"Alkyhol PE32 unsupported: exception PC width\n"[..])?;
    let sp = u32::try_from(machine.registers[7])
        .map_err(|_| &b"Alkyhol PE32 unsupported: exception SP width\n"[..])?;
    if !code(image, pc, 2)
        || !image.permits(pc as usize, 2, false)
        || !stack(image, sp, 4)
        || machine.registers[19] != 0x1017
        || unsafe { (pc as usize as *const u16).read_unaligned() } != 0x0b0f
    {
        return Err(b"Alkyhol PE32 unsupported: hardware exception form outside UD2 profile\n");
    }
    let context_at = sp
        .checked_sub(716)
        .ok_or(&b"Alkyhol PE32 unsupported: exception stack overflow\n"[..])?
        & !15;
    let record_at = context_at
        .checked_sub(80)
        .ok_or(&b"Alkyhol PE32 unsupported: exception stack overflow\n"[..])?;
    let frame_at = record_at
        .checked_sub(20)
        .ok_or(&b"Alkyhol PE32 unsupported: exception stack overflow\n"[..])?;
    if !stack(
        image,
        frame_at.saturating_sub(4096),
        sp as usize - frame_at as usize + 4096,
    ) {
        return Err(b"Alkyhol PE32 unsupported: insufficient exception callback stack\n");
    }
    let original = context32(image, &machine.registers);
    let context = context_at as usize as *mut Context32;
    let mut record = [0u32; 20];
    record[0] = 0xc000001d;
    record[3] = pc;
    unsafe {
        context.write(original);
        ptr::copy_nonoverlapping(record.as_ptr(), record_at as usize as *mut u32, 20);
    }
    let original_head = unsafe { image.cpu.teb().base.cast::<u32>().read_unaligned() };
    let mut registration = original_head;
    for _ in 0..128 {
        if registration == u32::MAX {
            return Err(b"Alkyhol PE32 unsupported: unhandled UD2 structured exception\n");
        }
        if registration < sp || !registration.is_multiple_of(4) || !stack(image, registration, 8) {
            return Err(b"Alkyhol PE32 unsupported: invalid SEH registration frame\n");
        }
        let p = registration as usize as *const u32;
        let next = unsafe { p.read_unaligned() };
        let handler = unsafe { p.add(1).read_unaligned() };
        if !registration_allowed(image, sp, registration, next, handler) {
            return Err(
                b"Alkyhol PE32 unsupported: invalid SEH chain or unregistered SafeSEH handler\n",
            );
        }
        let frame = frame_at as usize as *mut u32;
        for (index, value) in [
            image.gates.base as u32,
            record_at,
            registration,
            context_at,
            0,
        ]
        .into_iter()
        .enumerate()
        {
            unsafe {
                frame.add(index).write_unaligned(value);
            }
        }
        let result = unsafe {
            alkyhol_enter_pe32(
                handler,
                frame_at,
                u32::from(image.code),
                u32::from(image.data),
            )
        };
        if (result >> 32) as u32 != frame_at + 4 {
            return Err(b"Alkyhol PE32 unsupported: exception handler stack cleanup\n");
        }
        let changed = unsafe { context.read() };
        if !validate_context(image, &original, &changed)
            || unsafe { std::slice::from_raw_parts(record_at as usize as *const u32, 20) } != record
            || unsafe { p.read_unaligned() } != next
            || unsafe { p.add(1).read_unaligned() } != handler
            || unsafe { image.cpu.teb().base.cast::<u32>().read_unaligned() } != original_head
        {
            return Err(b"Alkyhol PE32 unsupported: exception handler changed unsupported context or registration state\n");
        }
        match result as u32 {
            0 => {
                for (index, register) in [
                    (39, 4),
                    (40, 5),
                    (41, 1),
                    (42, 3),
                    (43, 2),
                    (44, 0),
                    (45, 6),
                    (46, 16),
                    (48, 17),
                    (49, 7),
                ] {
                    machine.registers[register] = u64::from(changed.words[index]);
                }
                return Ok(());
            }
            1 => registration = next,
            _ => {
                return Err(
                    b"Alkyhol PE32 unsupported: exception disposition requires nesting or unwind\n",
                );
            }
        }
    }
    Err(b"Alkyhol PE32 unsupported: SEH chain traversal budget exhausted\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signal_abi_and_windows_integer_context_offsets_match_independent_sdk_assertions() {
        assert_eq!(std::mem::size_of::<Action>(), 16);
        assert_eq!(std::mem::offset_of!(Action, mask), 8);
        assert_eq!(std::mem::offset_of!(Action, flags), 12);
        assert_eq!(std::mem::size_of::<AltStack>(), 24);
        assert_eq!(std::mem::offset_of!(UserContext, machine_size), 40);
        assert_eq!(std::mem::offset_of!(UserContext, machine), 48);
        assert_eq!(std::mem::offset_of!(MachinePrefix, registers), 16);
        assert_eq!(std::mem::size_of::<MachinePrefix>(), 184);
        assert_eq!(std::mem::size_of::<Context32>(), 716);
    }
    #[test]
    fn context_mapping_preserves_only_selected_integer_control_and_segment_state() {
        let mut image = super::super::tests::layout();
        image.code = 0x1007;
        image.data = 0x100f;
        let mut registers = [0u64; 21];
        registers[0] = 0x12345678;
        registers[1] = 0x23456789;
        registers[16] = 0x100040;
        registers[7] = 0x402000;
        registers[17] = 0x202;
        registers[18] = 0x1007;
        registers[19] = 0x1017;
        let original = context32(&image, &registers);
        assert_eq!(original.words[44], 0x12345678);
        assert_eq!(original.words[41], 0x23456789);
        assert_eq!(original.words[46], 0x100040);
        assert_eq!(original.words[49], 0x402000);
        assert_eq!(original.words[36], 0x1017);
        assert_eq!(original.words[50], 0x100f);
        assert!(validate_context(&image, &original, &original));
        let mut changed = original;
        changed.words[44] = 0xaabbccdd;
        changed.words[46] += 2;
        changed.words[48] |= 1;
        assert!(validate_context(&image, &original, &changed));
        for (index, value) in [
            (0, 0x1003f),
            (1, 1),
            (7, 1),
            (36, 0),
            (47, 0),
            (48, 0x302),
            (49, 0x400000),
            (49, 0x402001),
            (46, 0x100300),
            (46, 0x200000),
            (51, 1),
            (178, 1),
        ] {
            let mut bad = original;
            bad.words[index] = value;
            assert!(!validate_context(&image, &original, &bad), "{index}");
        }
    }
    #[test]
    fn registration_validation_bounds_frames_rejects_cycles_and_enforces_safeseh_membership() {
        let mut image = super::super::tests::layout();
        assert!(registration_allowed(
            &image,
            0x402000,
            0x402010,
            u32::MAX,
            0x100040
        ));
        assert!(registration_allowed(
            &image, 0x402000, 0x402010, 0x402020, 0x100040
        ));
        for (frame, next, handler) in [
            (0x401ffc, u32::MAX, 0x100040),
            (0x402011, u32::MAX, 0x100040),
            (0x402010, 0x402010, 0x100040),
            (0x402010, 0x402008, 0x100040),
            (0x402010, 0xfffffff0, 0x100040),
            (0x402010, 0x402021, 0x100040),
            (0x402010, u32::MAX, 0x200000),
            (0x402010, u32::MAX, 0x100300),
        ] {
            assert!(!registration_allowed(
                &image, 0x402000, frame, next, handler
            ));
        }
        image.seh_handlers = vec![0x100020, 0x100040];
        assert!(registration_allowed(
            &image,
            0x402000,
            0x402010,
            u32::MAX,
            0x100040
        ));
        assert!(!registration_allowed(
            &image,
            0x402000,
            0x402010,
            u32::MAX,
            0x100030
        ));
    }
    #[test]
    fn signal_guard_exclusively_owns_a_high_guarded_stack_and_restores_actual_host_state() {
        let _serial = super::super::FS_SLOT.lock().unwrap();
        let image = super::super::tests::layout();
        let blocked = 1u32 << (SIGILL - 1);
        let mut old_mask = 0u32;
        assert_eq!(unsafe { pthread_sigmask(1, &blocked, &mut old_mask) }, 0);
        let refused = SignalGuard::install(&image).is_err();
        assert_eq!(unsafe { pthread_sigmask(3, &old_mask, ptr::null_mut()) }, 0);
        assert!(refused);
        let mut before = Action::default();
        let mut stack_before = AltStack {
            address: ptr::null_mut(),
            size: 0,
            flags: 0,
        };
        assert_eq!(unsafe { sigaction(SIGILL, ptr::null(), &mut before) }, 0);
        assert_eq!(unsafe { sigaltstack(ptr::null(), &mut stack_before) }, 0);
        {
            let guard = SignalGuard::install(&image).unwrap();
            assert!(guard.memory as usize > u32::MAX as usize);
            assert!(SignalGuard::install(&image).is_err());
            let mut current = Action::default();
            let mut current_stack = stack_before;
            assert_eq!(unsafe { sigaction(SIGILL, ptr::null(), &mut current) }, 0);
            assert_eq!(unsafe { sigaltstack(ptr::null(), &mut current_stack) }, 0);
            assert_eq!(current.handler, signal as *const () as usize);
            assert_eq!(current.flags & 0x51, 0x51);
            assert_eq!(current_stack.address as usize, guard.memory as usize + PAGE);
            assert_eq!(current_stack.size, SIGNAL_STACK);
        }
        assert!(OWNER.load(Ordering::SeqCst).is_null());
        let mut after = Action::default();
        let mut stack_after = stack_before;
        assert_eq!(unsafe { sigaction(SIGILL, ptr::null(), &mut after) }, 0);
        assert_eq!(unsafe { sigaltstack(ptr::null(), &mut stack_after) }, 0);
        assert!(before == after && stack_before == stack_after);
    }
}
