//! Owned file-backed Windows cursor images, decoded by public AppKit services.
//! Exact source dimensions/hotspots; unsupported scaling/resource forms fail.
use super::{
    LoadedImage32, bulk32, directory32, files32, startup32, unsupported, window_host32 as host,
};
use crate::Result;
use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Header {
    width: u32,
    height: u32,
    x: u32,
    y: u32,
}
fn header(bytes: &[u8]) -> Result<Header> {
    if bytes.len() < 22 || bytes[..6] != [0, 0, 2, 0, 1, 0] || bytes[9] != 0 {
        return Err("cursor requires one CUR image with a valid directory".into());
    }
    let width = if bytes[6] == 0 { 256 } else { bytes[6] as u32 };
    let height = if bytes[7] == 0 { 256 } else { bytes[7] as u32 };
    let x = u16::from_le_bytes(bytes[10..12].try_into().unwrap()) as u32;
    let y = u16::from_le_bytes(bytes[12..14].try_into().unwrap()) as u32;
    let len = u32::from_le_bytes(bytes[14..18].try_into().unwrap()) as usize;
    let offset = u32::from_le_bytes(bytes[18..22].try_into().unwrap()) as usize;
    if x >= width
        || y >= height
        || len == 0
        || offset < 22
        || offset.checked_add(len).is_none_or(|end| end > bytes.len())
    {
        return Err("cursor hotspot/data outside its directory image".into());
    }
    Ok(Header {
        width,
        height,
        x,
        y,
    })
}
struct Cursor {
    native: host::Cursor,
    _bytes: Option<Vec<u8>>,
    shared: bool,
}
#[derive(Default)]
pub(super) struct State {
    cursors: BTreeMap<u32, Cursor>,
}
fn fail(code: u32) -> u32 {
    alkyhol::ffi::SetLastError(code);
    0
}
pub(super) fn load(image: &LoadedImage32<'_>, a: &[u32; 8], ansi: bool) -> u32 {
    if a[0] != 0 || a[2] != 2 || a[5] & !0x50 != 0 || a[5] & 0x10 == 0 {
        return fail(50);
    }
    let text = if ansi {
        bulk32::ascii(image, a[1])
    } else {
        let mut u = startup32::read_units::<u16>(image, a[1], -1)
            .unwrap_or_else(|| unsupported("LoadImage path outside checked storage"));
        u.pop();
        String::from_utf16(&u).unwrap_or_else(|_| unsupported("LoadImage path is invalid UTF-16"))
    };
    let path = directory32::ordinary_path(image, &text).unwrap_or_else(|e| unsupported(&e));
    eprintln!(
        "PE32 original cursor load: path={path:?}; requested={}x{}; flags={:x}",
        a[3], a[4], a[5]
    );
    let (path, meta) = match files32::existing(&path).unwrap_or_else(|e| unsupported(&e)) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    if !meta.is_file() || meta.len() > 2 * 1024 * 1024 {
        return fail(13);
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(0x100 | 0x1000000)
        .open(path)
        .unwrap_or_else(|e| unsupported(&e.to_string()));
    let actual = file
        .metadata()
        .unwrap_or_else(|e| unsupported(&e.to_string()));
    if (meta.dev(), meta.ino()) != (actual.dev(), actual.ino()) {
        unsupported("cursor file changed identity during open");
    }
    let mut bytes = Vec::new();
    file.take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .unwrap_or_else(|e| unsupported(&e.to_string()));
    if bytes.len() > 2 * 1024 * 1024 {
        return fail(13);
    }
    let h = match header(&bytes) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("PE32 cursor decoder rejected: {e}");
            return fail(13);
        }
    };
    let width = if a[3] != 0 {
        a[3]
    } else if a[5] & 0x40 != 0 {
        32
    } else {
        h.width
    };
    let height = if a[4] != 0 {
        a[4]
    } else if a[5] & 0x40 != 0 {
        32
    } else {
        h.height
    };
    if width != h.width || height != h.height {
        return fail(50);
    }
    super::ui32::ensure_app(image);
    if image.cpu.images.borrow().cursors.len() >= 256 {
        return fail(8);
    }
    let native = match host::Cursor::new(&bytes, h.width, h.height, h.x, h.y) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("PE32 native cursor decoder: {e}");
            return fail(13);
        }
    };
    let token = alkyhol::kernel::reserve_handle_token()
        .unwrap_or_else(|_| unsupported("cursor handle namespace exhausted"));
    if token >= 0x10000000 {
        unsupported("cursor handle namespace exhausted");
    }
    let token = 0x40000000 | token as u32;
    if image.cpu.diagnostic.is_some() {
        eprintln!(
            "PE32 native cursor created: handle={token:x}; dimensions={}x{}; hotspot={},{}",
            h.width, h.height, h.x, h.y
        );
    }
    image.cpu.images.borrow_mut().cursors.insert(
        token,
        Cursor {
            native,
            _bytes: Some(bytes),
            shared: false,
        },
    );
    token
}
fn register_current(state: &mut State, native: Option<host::Cursor>) -> u32 {
    let Some(native) = native else {
        return 0;
    };
    if let Some((&handle, _)) = state.cursors.iter().find(|(_, c)| c.native.same(&native)) {
        return handle;
    }
    if state.cursors.len() >= 256 {
        unsupported("cursor namespace budget exhausted");
    }
    let token = alkyhol::kernel::reserve_handle_token()
        .unwrap_or_else(|_| unsupported("cursor handle allocation failed"));
    if token >= 0x10000000 {
        unsupported("cursor namespace exhausted");
    }
    let token = 0x40000000 | token as u32;
    state.cursors.insert(
        token,
        Cursor {
            native,
            _bytes: None,
            shared: true,
        },
    );
    token
}
pub(super) fn set(image: &LoadedImage32<'_>, handle: u32) -> u32 {
    if handle == 0 {
        unsupported("SetCursor(NULL) requires native transparent-cursor binding");
    }
    let mut state = image.cpu.images.borrow_mut();
    let Some(cursor) = state.cursors.get(&handle) else {
        return fail(1402);
    };
    let old = cursor.native.replace().unwrap_or_else(|e| unsupported(&e));
    register_current(&mut state, old)
}
pub(super) fn current(image: &LoadedImage32<'_>) -> u32 {
    super::ui32::ensure_app(image);
    let native = host::Cursor::retain_current().unwrap_or_else(|e| unsupported(&e));
    register_current(&mut image.cpu.images.borrow_mut(), native)
}
pub(super) fn destroy(image: &LoadedImage32<'_>, handle: u32) -> u32 {
    let mut state = image.cpu.images.borrow_mut();
    if let Some(cursor) = state.cursors.get(&handle) {
        if cursor.shared {
            return fail(50);
        }
        if cursor.native.is_current() {
            return fail(170);
        }
    }
    if state.cursors.remove(&handle).is_some() {
        1
    } else {
        fail(1402)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cursor_directory_hotspot_and_extents_are_checked() {
        let mut b = vec![0u8; 26];
        b[..6].copy_from_slice(&[0, 0, 2, 0, 1, 0]);
        b[6] = 32;
        b[7] = 16;
        b[10..12].copy_from_slice(&7u16.to_le_bytes());
        b[12..14].copy_from_slice(&9u16.to_le_bytes());
        b[14..18].copy_from_slice(&4u32.to_le_bytes());
        b[18..22].copy_from_slice(&22u32.to_le_bytes());
        assert_eq!(
            header(&b).unwrap(),
            Header {
                width: 32,
                height: 16,
                x: 7,
                y: 9
            }
        );
        let mut invalid = b.clone();
        invalid[10] = 32;
        assert!(header(&invalid).is_err());
        assert!(header(&b[..25]).is_err());
        b[4] = 2;
        assert!(header(&b).is_err());
    }
}
