//! Explicit UTC guest clock profile. No inference of macOS timezone/DST rules.
use super::{LoadedImage32, unsupported};
unsafe extern "C" {
    fn mach_continuous_time() -> u64;
    fn mach_timebase_info(info: *mut [u32; 2]) -> i32;
}
pub(super) fn tick_count() -> u32 {
    let mut scale = [0u32; 2];
    if unsafe { mach_timebase_info(&mut scale) } != 0 || scale[0] == 0 || scale[1] == 0 {
        unsupported("host continuous clock scale is unavailable");
    }
    // Apple's continuous boot clock includes system sleep. Widen before scaling,
    // then deliberately retain the low DWORD of milliseconds (49.7-day wrap).
    tick_milliseconds(unsafe { mach_continuous_time() }, scale)
}
fn tick_milliseconds(raw: u64, scale: [u32; 2]) -> u32 {
    (u128::from(raw) * u128::from(scale[0]) / u128::from(scale[1]) / 1_000_000) as u32
}
fn check(image: &LoadedImage32<'_>, address: u32, size: usize, write: bool) {
    if !address.is_multiple_of(4) || !image.permits(address as usize, size, write) {
        unsupported("PE32 time buffer outside checked guest storage");
    }
}
fn zone() -> [u8; 172] {
    let mut bytes = [0; 172];
    // An owned UTC timezone: zero bias, no DST transitions, explicit profile
    // names. Windows' localized display strings are not asserted as identical.
    for offset in [4, 88] {
        for (i, unit) in "UTC".encode_utf16().enumerate() {
            bytes[offset + i * 2..offset + i * 2 + 2].copy_from_slice(&unit.to_le_bytes());
        }
    }
    bytes
}
fn calendar(ticks: u64) -> Option<[u16; 8]> {
    if ticks >> 63 != 0 {
        return None;
    }
    let seconds = ticks / 10_000_000;
    let days = seconds / 86400;
    // Gregorian leap rules repeat every 400 years. Start at 1601 and inspect at
    // most 399 years, keeping every operation in a bounded integer domain.
    let mut year = 1601 + (days / 146097) * 400;
    let mut remaining = days % 146097;
    let leap = |y: u64| y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400));
    loop {
        let length = if leap(year) { 366 } else { 365 };
        if remaining < length {
            break;
        }
        remaining -= length;
        year += 1;
    }
    let months = [
        31,
        if leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 0;
    while remaining >= months[month] {
        remaining -= months[month];
        month += 1;
    }
    Some([
        year as u16,
        month as u16 + 1,
        ((days + 1) % 7) as u16,
        remaining as u16 + 1,
        ((seconds / 3600) % 24) as u16,
        ((seconds / 60) % 60) as u16,
        (seconds % 60) as u16,
        ((ticks % 10_000_000) / 10_000) as u16,
    ])
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, args: &[u32; 8]) -> u32 {
    if id == 70 {
        check(image, args[0], 8, false);
        if !args[1].is_multiple_of(2) || !image.permits(args[1] as usize, 16, true) {
            unsupported("PE32 SYSTEMTIME output outside checked writable guest storage");
        }
        let (a, b) = (args[0] as usize, args[1] as usize);
        if a < b + 16 && b < a + 8 {
            unsupported("overlapping PE32 FILETIME/SYSTEMTIME buffers are unsupported");
        }
        let value = unsafe { (a as *const [u32; 2]).read() };
        let ticks = u64::from(value[0]) | (u64::from(value[1]) << 32);
        let Some(output) = calendar(ticks) else {
            // Frozen native observations: fail without touching SYSTEMTIME.
            alkyhol::ffi::SetLastError(87);
            return 0;
        };
        unsafe {
            (b as *mut [u16; 8]).write(output);
        }
        return 1;
    }
    if !image.cpu.utc_timezone {
        unsupported(
            "PE32 local-time APIs require an explicitly configured supported guest timezone",
        );
    }
    if id == 69 {
        check(image, args[0], 172, true);
        let bytes = zone();
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), args[0] as usize as *mut u8, bytes.len());
        }
        return 0; // TIME_ZONE_ID_UNKNOWN: this UTC profile has no DST regime.
    }
    check(image, args[0], 8, false);
    check(image, args[1], 8, true);
    let (a, b) = (args[0] as usize, args[1] as usize);
    if a != b && a < b + 8 && b < a + 8 {
        unsupported("partially overlapping PE32 FILETIME buffers are unsupported");
    }
    // Native UTC observations admit every u64 pattern, including the high bit.
    // Stage both DWORDs before output so exact in-place conversion is supported.
    let value = unsafe { (a as *const [u32; 2]).read() };
    unsafe {
        (b as *mut [u32; 2]).write(value);
    }
    1
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn utc_profile_has_real_names_zero_bias_and_no_dst_transitions() {
        let bytes = zone();
        assert_eq!(bytes.len(), 172);
        for at in [0, 84, 168] {
            assert_eq!(&bytes[at..at + 4], &[0; 4]);
        }
        for at in [68, 152] {
            assert_eq!(&bytes[at..at + 16], &[0; 16]);
        }
        for at in [4, 88] {
            assert_eq!(&bytes[at..at + 8], b"U\0T\0C\0\0\0");
        }
    }
    #[test]
    fn calendar_epoch_submillisecond_and_gregorian_centuries_are_exact() {
        assert_eq!(calendar(0), Some([1601, 1, 1, 1, 0, 0, 0, 0]));
        assert_eq!(calendar(9999), calendar(0));
        assert_eq!(calendar(10000), Some([1601, 1, 1, 1, 0, 0, 0, 1]));
        assert_eq!(
            calendar(116444736000000000),
            Some([1970, 1, 4, 1, 0, 0, 0, 0])
        );
        assert_eq!(
            calendar(94405824000000000),
            Some([1900, 3, 4, 1, 0, 0, 0, 0])
        );
        assert_eq!(
            calendar(125962560000000000),
            Some([2000, 2, 2, 29, 0, 0, 0, 0])
        );
        assert_eq!(
            calendar(133537247999990000),
            Some([2024, 2, 4, 29, 23, 59, 59, 999])
        );
        assert_eq!(
            calendar(i64::MAX as u64),
            Some([30828, 9, 4, 14, 2, 48, 5, 477])
        );
        assert!(calendar(1 << 63).is_none());
        assert!(calendar(u64::MAX).is_none());
    }
    #[test]
    fn boot_ticks_scale_before_truncation_and_wrap_at_one_dword() {
        assert_eq!(tick_milliseconds(999999, [1, 1]), 0);
        assert_eq!(tick_milliseconds(3, [1_000_000, 3]), 1);
        assert_eq!(tick_milliseconds((1u64 << 32) * 1_000_000, [1, 1]), 0);
        assert_eq!(
            tick_milliseconds(((1u64 << 32) + 37) * 1_000_000, [1, 1]),
            37
        );
        // The multiplication must widen before division: this overflows u64.
        assert_eq!(tick_milliseconds(u64::MAX, [1_000_000, 1]), u32::MAX);
    }
}
