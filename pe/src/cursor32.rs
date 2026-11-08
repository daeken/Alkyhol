//! Read-only cursor queries in the host's Quartz logical desktop coordinates.
//! No synthetic cursor, event injection, DPI virtualization or window mapping.
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
    let event = unsafe { CGEventCreate(std::ptr::null()) };
    if event.is_null() {
        unsupported("actual host desktop cursor query is unavailable");
    }
    let point = unsafe { CGEventGetLocation(event) };
    unsafe {
        CFRelease(event);
    }
    let point = integer_point(point).unwrap_or_else(|e| unsupported(&e));
    unsafe {
        std::ptr::copy_nonoverlapping(point.as_ptr(), output as *mut i32, 2);
    }
    1 // Actual host query completed and both guest LONG fields were written.
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
