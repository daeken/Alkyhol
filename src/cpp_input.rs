//! DirectInput public data layouts. Constants follow the pinned MinGW dinput.h
//! declarations and standard keyboard/mouse format tables (see cpp/upstream.json).
//! Device creation is explicitly unavailable; metadata is not input support.
#![allow(non_upper_case_globals)]
use crate::com::Guid;
const COMMON: [u8; 8] = [0xbf, 0xc7, 0x44, 0x45, 0x53, 0x54, 0, 0];
const fn guid(a: u32, b: u16) -> Guid {
    Guid {
        data1: a,
        data2: b,
        data3: 0x11cf,
        data4: COMMON,
    }
}
#[unsafe(no_mangle)]
pub static GUID_SysMouse: Guid = guid(0x6f1d2b60, 0xd5a0);
#[unsafe(no_mangle)]
pub static GUID_SysKeyboard: Guid = guid(0x6f1d2b61, 0xd5a0);
#[unsafe(no_mangle)]
pub static IID_IDirectInput8W: Guid = Guid {
    data1: 0xbf798031,
    data2: 0x483a,
    data3: 0x4da2,
    data4: [0xaa, 0x99, 0x5d, 0x64, 0xed, 0x36, 0x97, 0],
};
#[unsafe(no_mangle)]
pub static IID_ID3D12Device: Guid = Guid {
    data1: 0x189819f1,
    data2: 0x1db6,
    data3: 0x4b57,
    data4: [0xbe, 0x54, 0x18, 0x21, 0x33, 0x9b, 0x85, 0xf7],
};
#[unsafe(no_mangle)]
pub static GUID_Key: Guid = guid(0x55728220, 0xd33c);
#[unsafe(no_mangle)]
pub static GUID_XAxis: Guid = guid(0xa36d02e0, 0xc9f3);
#[unsafe(no_mangle)]
pub static GUID_YAxis: Guid = guid(0xa36d02e1, 0xc9f3);
#[unsafe(no_mangle)]
pub static GUID_ZAxis: Guid = guid(0xa36d02e2, 0xc9f3);
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ObjectFormat {
    pub guid: *const Guid,
    pub offset: u32,
    pub kind: u32,
    pub flags: u32,
}
// All pointers target immutable static GUIDs; neither table is mutated.
unsafe impl Sync for ObjectFormat {}
#[repr(C)]
pub struct DataFormat {
    pub size: u32,
    pub object_size: u32,
    pub flags: u32,
    pub data_size: u32,
    pub object_count: u32,
    pub objects: *const ObjectFormat,
}
// All pointers target immutable static object-format arrays.
unsafe impl Sync for DataFormat {}
const fn keyboard() -> [ObjectFormat; 256] {
    let mut out = [ObjectFormat {
        guid: &GUID_Key,
        offset: 0,
        kind: 0,
        flags: 0,
    }; 256];
    let mut i = 0;
    while i < 256 {
        out[i].offset = i as u32;
        out[i].kind = 0x8000000c | ((i as u32) << 8);
        i += 1;
    }
    out
}
static KEYBOARD: [ObjectFormat; 256] = keyboard();
const fn mouse() -> [ObjectFormat; 11] {
    let mut out = [ObjectFormat {
        guid: std::ptr::null(),
        offset: 0,
        kind: 0x00ffff0c,
        flags: 0,
    }; 11];
    out[0] = ObjectFormat {
        guid: &GUID_XAxis,
        offset: 0,
        kind: 0x00ffff03,
        flags: 0,
    };
    out[1] = ObjectFormat {
        guid: &GUID_YAxis,
        offset: 4,
        kind: 0x00ffff03,
        flags: 0,
    };
    out[2] = ObjectFormat {
        guid: &GUID_ZAxis,
        offset: 8,
        kind: 0x80ffff03,
        flags: 0,
    };
    let mut i = 3;
    while i < 11 {
        out[i].offset = 9 + i as u32;
        if i >= 5 {
            out[i].kind |= 0x80000000;
        }
        i += 1;
    }
    out
}
static MOUSE: [ObjectFormat; 11] = mouse();
#[unsafe(no_mangle)]
pub static c_dfDIKeyboard: DataFormat = DataFormat {
    size: std::mem::size_of::<DataFormat>() as u32,
    object_size: std::mem::size_of::<ObjectFormat>() as u32,
    flags: 2,
    data_size: 256,
    object_count: 256,
    objects: KEYBOARD.as_ptr(),
};
#[unsafe(no_mangle)]
pub static c_dfDIMouse2: DataFormat = DataFormat {
    size: std::mem::size_of::<DataFormat>() as u32,
    object_size: std::mem::size_of::<ObjectFormat>() as u32,
    flags: 2,
    data_size: 20,
    object_count: 11,
    objects: MOUSE.as_ptr(),
};
