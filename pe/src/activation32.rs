//! Fail-closed DLL-redirection misses for a validated empty embedded context.
//! Dependency binding, nonempty contexts and context stack operations are not
//! implemented. An unknown context must stop, never pretend the key is absent.
use super::{LoadedImage32, startup32, unsupported};

pub(super) fn find(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    let [flags, guid, section, key, output, ..] = *args;
    if flags > 1 || guid != 0 || section != 2 {
        unsupported("PE32 activation query flags/GUID/section are unsupported");
    }
    if !output.is_multiple_of(4) || !image.permits(output as usize, 4, false) {
        unsupported("PE32 activation keyed-data header outside checked guest storage");
    }
    let size = unsafe { (output as usize as *const u32).read() };
    if ![40, 64].contains(&size) || !image.permits(output as usize, size as usize, true) {
        unsupported("PE32 activation keyed-data size/output is unsupported");
    }
    // A miss still requires a real bounded, terminated guest string. No host
    // dereference of an unchecked pointer, and no lossy UTF-16 conversion.
    startup32::read_units::<u16>(image, key, -1)
        .unwrap_or_else(|| unsupported("PE32 activation key is unsupported"));
    if let Err(error) = &*image.cpu.activation.borrow() {
        unsupported(error);
    }
    // Native observations: a missing DLL key leaves every output byte untouched
    // for both legacy/current cbSize values and sets ERROR_SXS_KEY_NOT_FOUND.
    alkyhol::ffi::SetLastError(14007);
    0
}
