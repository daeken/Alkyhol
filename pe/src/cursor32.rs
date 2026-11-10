//! Read-only cursor queries in the host's Quartz logical desktop coordinates.
//! No synthetic cursor, event injection, DPI virtualization or window mapping.
//! Confinement requests fail explicitly until a real native backend exists.
use super::{LoadedImage32, unsupported};
use std::ffi::c_void;

#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventCreate(source: *const c_void) -> *const c_void;
    fn CGEventGetLocation(event: *const c_void) -> Point;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(value: *const c_void);
}

fn integer_point(point: Point) -> crate::Result<[i32; 2]> {
    let values = [point.x.floor(), point.y.floor()];
    if values
        .iter()
        .any(|&v| !v.is_finite() || v < i32::MIN as f64 || v > i32::MAX as f64)
    {
        return Err("host desktop cursor outside signed Win32 POINT coordinates".into());
    }
    Ok([values[0] as i32, values[1] as i32])
}

pub(super) fn position(image: &LoadedImage32<'_>, output: u32) -> u32 {
    if !output.is_multiple_of(4) || !image.permits(output as usize, 8, true) {
        unsupported("cursor POINT outside aligned writable guest storage");
    }
    let point = observed();
    unsafe {
        std::ptr::copy_nonoverlapping(point.as_ptr(), output as *mut i32, 2);
    }
    1 // Actual host query completed and both guest LONG fields were written.
}

pub(super) fn observed() -> [i32; 2] {
    let event = unsafe { CGEventCreate(std::ptr::null()) };
    if event.is_null() {
        unsupported("actual host desktop cursor query is unavailable");
    }
    let point = unsafe { CGEventGetLocation(event) };
    unsafe {
        CFRelease(event);
    }
    integer_point(point).unwrap_or_else(|e| unsupported(&e))
}

/// No native confinement is implemented yet. Preserve a genuine release of this
/// bridge's unconfined cursor and return an explicit Win32 failure for a range;
/// never claim that storing a rectangle has constrained physical input.
pub(super) fn clip(image: &LoadedImage32<'_>, rectangle: u32) -> u32 {
    if rectangle == 0 {
        return 1;
    }
    if !image.permits(rectangle as usize, 16, false) {
        unsupported("ClipCursor RECT outside checked guest storage");
    }
    let bounds = unsafe { (rectangle as *const [i32; 4]).read_unaligned() };
    let error = if bounds[0] > bounds[2] || bounds[1] > bounds[3] {
        87
    } else {
        120
    };
    eprintln!("PE32 ClipCursor: confinement unavailable; returning FALSE with Win32 error {error}");
    alkyhol::ffi::SetLastError(error);
    0
}

/// This bridge never reports successful physical confinement. Its current clip
/// rectangle is therefore the actual host virtual desktop, including negative
/// monitor origins, rather than an invented or remembered requested rectangle.
pub(super) fn clipping_rectangle(image: &LoadedImage32<'_>, output: u32) -> u32 {
    if !output.is_multiple_of(4) || !image.permits(output as usize, 16, true) {
        unsupported("cursor clipping RECT outside aligned writable guest storage");
    }
    let bounds = super::ui32::unconfined_cursor_bounds(image);
    unsafe {
        std::ptr::copy_nonoverlapping(bounds.as_ptr(), output as usize as *mut i32, 4);
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn logical_desktop_cells_preserve_negative_coordinates_and_long_boundaries() {
        assert_eq!(integer_point(Point { x: 1.9, y: -0.25 }).unwrap(), [1, -1]);
        assert_eq!(
            integer_point(Point {
                x: i32::MIN as f64,
                y: i32::MAX as f64 + 0.75
            })
            .unwrap(),
            [i32::MIN, i32::MAX]
        );
        for value in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            i32::MAX as f64 + 1.,
            i32::MIN as f64 - 0.25,
        ] {
            assert!(integer_point(Point { x: value, y: 0. }).is_err());
            assert!(integer_point(Point { x: 0., y: value }).is_err());
        }
    }
}
