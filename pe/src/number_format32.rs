//! Explicit custom en-US GetNumberFormatW, reached by original DAO loading.
use super::{LoadedImage32, startup32, unsupported};
#[path = "number_format.rs"]
mod decimal;

fn fail(error: u32) -> u32 {
    alkyhol::ffi::SetLastError(error);
    0
}
fn wide(image: &LoadedImage32<'_>, address: u32) -> Option<Vec<u16>> {
    if address != 0 && !address.is_multiple_of(2) {
        unsupported("number-format unaligned string pointer outside measured profile");
    }
    let mut value = startup32::read_units::<u16>(image, address, -1)?;
    value.pop();
    Some(value)
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if a[0] != 0x409 {
        unsupported("number-format locale outside explicit en-US profile");
    }
    if a[3] == 0 {
        unsupported("number-format defaults and user locale overrides are unsupported");
    }
    if a[1] != 0 {
        return fail(1004);
    }
    let capacity = a[5] as i32;
    if capacity < 0 || (capacity > 0 && a[4] == 0) {
        return fail(87);
    }
    if capacity > 32768 {
        unsupported("number-format output capacity outside bounded profile");
    }
    if !a[3].is_multiple_of(4) || !image.permits(a[3] as usize, 24, false) {
        unsupported("number-format structure outside checked readable guest storage");
    }
    let fields = unsafe { (a[3] as usize as *const [u32; 6]).read() };
    let Some(value) = wide(image, a[2]) else {
        return 0;
    };
    let Some(decimal) = wide(image, fields[3]) else {
        return 0;
    };
    let Some(thousands) = wide(image, fields[4]) else {
        return 0;
    };
    let f = decimal::Format {
        digits: fields[0],
        leading_zero: fields[1],
        grouping: fields[2],
        decimal: &decimal,
        thousands: &thousands,
        negative_order: fields[5],
    };
    let output = match decimal::format(&value, &f) {
        Ok(output) => output,
        Err(decimal::Error::Invalid) => return fail(87),
        Err(decimal::Error::Unsupported(reason)) => unsupported(reason),
    };
    if capacity == 0 {
        return output.len() as u32;
    }
    if !a[4].is_multiple_of(2) || !image.permits(a[4] as usize, capacity as usize * 2, true) {
        unsupported("number-format output outside checked writable guest storage");
    }
    let start = u64::from(a[4]);
    let end = start + capacity as u64 * 2;
    for (address, length) in [
        (a[2], (value.len() + 1) * 2),
        (a[3], 24),
        (fields[3], (decimal.len() + 1) * 2),
        (fields[4], (thousands.len() + 1) * 2),
    ] {
        if start < u64::from(address) + length as u64 && u64::from(address) < end {
            unsupported("overlapping number-format input/output storage is unmeasured");
        }
    }
    if (capacity as usize) < output.len() {
        return fail(122);
    }
    unsafe {
        std::ptr::copy_nonoverlapping(output.as_ptr(), a[4] as usize as *mut u16, output.len());
    }
    output.len() as u32
}
