//! Copy a checked, position-independent i386 x87 template into owned RX gates.
use super::{PAGE, unsupported};
std::arch::global_asm!(include_str!("ci_sqrt32.S"), options(att_syntax));
std::arch::global_asm!(include_str!("ci_atan32.S"), options(att_syntax));
unsafe extern "C" {
    static alkyhol_ci_sqrt32_start: u8;
    static alkyhol_ci_sqrt32_end: u8;
    static alkyhol_ci_sqrt32_failure: u8;
    static alkyhol_ci_atan32_start: u8;
    static alkyhol_ci_atan32_end: u8;
    static alkyhol_ci_atan32_failure: u8;
}
// API stubs occupy the first page; numeric bodies have their own second page.
pub(super) const GATE_SIZE: usize = 2 * PAGE;
pub(super) const OFFSET: usize = PAGE;
pub(super) const ATAN_OFFSET: usize = PAGE + 256;
pub(super) fn emit(template: &mut [u8], base: u32, failure: u32) -> crate::Result<()> {
    let start = std::ptr::addr_of!(alkyhol_ci_sqrt32_start) as usize;
    let end = std::ptr::addr_of!(alkyhol_ci_sqrt32_end) as usize;
    let patch = std::ptr::addr_of!(alkyhol_ci_sqrt32_failure) as usize - start;
    install(
        template,
        base,
        failure,
        [start, end, patch, OFFSET, ATAN_OFFSET],
    )
}
pub(super) fn emit_atan(template: &mut [u8], base: u32, failure: u32) -> crate::Result<()> {
    let start = std::ptr::addr_of!(alkyhol_ci_atan32_start) as usize;
    let end = std::ptr::addr_of!(alkyhol_ci_atan32_end) as usize;
    let patch = std::ptr::addr_of!(alkyhol_ci_atan32_failure) as usize - start;
    install(
        template,
        base,
        failure,
        [start, end, patch, ATAN_OFFSET, ATAN_OFFSET + 256],
    )
}
fn install(template: &mut [u8], base: u32, failure: u32, layout: [usize; 5]) -> crate::Result<()> {
    let [start, end, patch, offset, limit] = layout;
    let size = end
        .checked_sub(start)
        .filter(|&n| n > 0 && offset + n <= limit && limit <= GATE_SIZE)
        .ok_or("invalid x87 gateway template size")?;
    if patch + 4 > size
        || template.len() != GATE_SIZE
        || base.checked_add(GATE_SIZE as u32).is_none()
        || failure < base.saturating_add(256)
        || !(failure - base - 256).is_multiple_of(32)
        || failure
            .checked_add(26)
            .is_none_or(|v| v > base.saturating_add(OFFSET as u32))
    {
        return Err("invalid x87 gateway storage/patch".into());
    }
    template[offset..offset + size]
        .copy_from_slice(unsafe { std::slice::from_raw_parts(start as *const u8, size) });
    template[offset + patch..offset + patch + 4].copy_from_slice(&failure.to_le_bytes());
    Ok(())
}
pub(super) fn rejected(image: &super::LoadedImage32<'_>, snapshot: u32, caller: u32) -> ! {
    rejection(
        image,
        snapshot,
        caller,
        "_CIsqrt operand/control state outside the measured nonnegative finite x87 profile",
    )
}
pub(super) fn rejected_atan(image: &super::LoadedImage32<'_>, snapshot: u32, caller: u32) -> ! {
    rejection(
        image,
        snapshot,
        caller,
        "_CIatan operand/control state outside the measured finite x87 profile",
    )
}
fn rejection(image: &super::LoadedImage32<'_>, snapshot: u32, caller: u32, message: &str) -> ! {
    if !snapshot.is_multiple_of(16) || !image.permits(snapshot as usize, 512, false) {
        unsupported("invalid x87 rejection snapshot");
    }
    if image.cpu.diagnostic.is_some() {
        let b = unsafe { std::slice::from_raw_parts(snapshot as *const u8, 512) };
        eprintln!(
            "PE32 x87 rejection: caller=0x{caller:08x}; CW={:04x}; SW={:04x}; MXCSR={:08x}; ST0={:02x?}",
            u16::from_le_bytes(b[0..2].try_into().unwrap()),
            u16::from_le_bytes(b[2..4].try_into().unwrap()),
            u32::from_le_bytes(b[24..28].try_into().unwrap()),
            &b[32..42]
        );
    }
    unsupported(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn template_relocates_only_its_private_checked_failure_target() {
        let mut first = vec![0xa5; GATE_SIZE];
        let mut second = first.clone();
        emit(&mut first, 0x200000, 0x200c40).unwrap();
        emit(&mut second, 0x300000, 0x300c40).unwrap();
        assert_eq!(&first[..OFFSET], &vec![0xa5; OFFSET]);
        let start = std::ptr::addr_of!(alkyhol_ci_sqrt32_start) as usize;
        let patch = std::ptr::addr_of!(alkyhol_ci_sqrt32_failure) as usize - start + OFFSET;
        assert_eq!(&first[patch..patch + 4], &0x200c40u32.to_le_bytes());
        assert_eq!(&second[patch..patch + 4], &0x300c40u32.to_le_bytes());
        assert_eq!(&first[..patch], &second[..patch]);
        assert_eq!(&first[patch + 4..], &second[patch + 4..]);
    }
    #[test]
    fn incomplete_overflowing_or_unowned_gateway_patches_are_rejected() {
        assert!(emit(&mut vec![0; PAGE], 0x200000, 0x200c40).is_err());
        assert!(emit(&mut vec![0; GATE_SIZE - 1], 0x200000, 0x200c40).is_err());
        assert!(emit(&mut vec![0; GATE_SIZE], 0xfffff800, 0xfffffc00).is_err());
        for target in [0, 0x200000, 0x2000ff, 0x200101, 0x200df0, 0x201000] {
            assert!(emit(&mut vec![0; GATE_SIZE], 0x200000, target).is_err());
        }
    }
    #[test]
    fn both_numeric_templates_keep_separate_owned_code_ranges() {
        let mut code = vec![0xa5; GATE_SIZE];
        emit(&mut code, 0x200000, 0x200c40).unwrap();
        let before = code.clone();
        emit_atan(&mut code, 0x200000, 0x200ca0).unwrap();
        assert_eq!(&before[..ATAN_OFFSET], &code[..ATAN_OFFSET]);
        let start = std::ptr::addr_of!(alkyhol_ci_atan32_start) as usize;
        let patch = std::ptr::addr_of!(alkyhol_ci_atan32_failure) as usize - start + ATAN_OFFSET;
        assert_eq!(&code[patch..patch + 4], &0x200ca0u32.to_le_bytes());
        assert!(emit_atan(&mut vec![0; GATE_SIZE - 1], 0x200000, 0x200ca0).is_err());
        assert!(emit_atan(&mut vec![0; PAGE], 0x200000, 0x200ca0).is_err());
        assert!(emit_atan(&mut code, 0x200000, 0x200ca1).is_err());
    }
}
