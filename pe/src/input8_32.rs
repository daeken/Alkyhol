//! Real low stdcall DirectInput8 interfaces over the Quartz event provider.
//! Virtual system devices are identified as Alkyhol; unsupported devices, effects,
//! acquisition modes and action maps return failures rather than synthetic input.
use super::{CallingConvention32, Cpu32, LoadedImage32, PAGE, d3d9_32, unsupported};
use crate::Result;
use std::collections::{BTreeMap, BTreeSet};
#[path = "input_host32.rs"]
mod host;
use host::{Binding, Kind};
pub(super) const BASE: u32 = 0xa000;
const INVALID: u32 = 0x80070057;
const NOTIMPL: u32 = 0x80004001;
const NOTACQUIRED: u32 = 0x8007000c;
const INPUTLOST: u32 = 0x8007001e;
const UNKNOWN: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0xc0, 0, 0, 0, 0, 0, 0, 0x46];
const INPUT_A: [u8; 16] = [
    0x30, 0x80, 0x79, 0xbf, 0x3a, 0x48, 0xa2, 0x4d, 0xaa, 0x99, 0x5d, 0x64, 0xed, 0x36, 0x97, 0,
];
const INPUT_W: [u8; 16] = [
    0x31, 0x80, 0x79, 0xbf, 0x3a, 0x48, 0xa2, 0x4d, 0xaa, 0x99, 0x5d, 0x64, 0xed, 0x36, 0x97, 0,
];
const DEVICE_A: [u8; 16] = [
    0x80, 0x10, 0xd4, 0x54, 0x15, 0xdc, 0x33, 0x48, 0xa4, 0x1b, 0x74, 0x8f, 0x73, 0xa3, 0x81, 0x79,
];
const DEVICE_W: [u8; 16] = [
    0x81, 0x10, 0xd4, 0x54, 0x15, 0xdc, 0x33, 0x48, 0xa4, 0x1b, 0x74, 0x8f, 0x73, 0xa3, 0x81, 0x79,
];
fn system(kind: Kind) -> [u8; 16] {
    let mut guid = [
        0x60, 0x2b, 0x1d, 0x6f, 0xa0, 0xd5, 0xcf, 0x11, 0xbf, 0xc7, 0x44, 0x45, 0x53, 0x54, 0, 0,
    ];
    if kind == Kind::Keyboard {
        guid[0] = 0x61;
    }
    guid
}
fn kind(guid: [u8; 16]) -> Option<Kind> {
    if guid == system(Kind::Keyboard) {
        Some(Kind::Keyboard)
    } else if guid == system(Kind::Mouse) {
        Some(Kind::Mouse)
    } else {
        None
    }
}
fn object_guid(component: u16) -> [u8; 16] {
    if component < 256 {
        [
            0x20, 0x82, 0x72, 0x55, 0x3c, 0xd3, 0xcf, 0x11, 0xbf, 0xc7, 0x44, 0x45, 0x53, 0x54, 0,
            0,
        ]
    } else {
        let mut v = [
            0xe0, 2, 0x6d, 0xa3, 0xf3, 0xc9, 0xcf, 0x11, 0xbf, 0xc7, 0x44, 0x45, 0x53, 0x54, 0, 0,
        ];
        v[0] = if component < 259 {
            0xe0 + (component - 256) as u8
        } else {
            0xf0
        };
        v
    }
}
struct Format {
    size: usize,
    bindings: Vec<Binding>,
}
struct Device {
    parent: u32,
    kind: Kind,
    format: Option<Format>,
    cooperative: Option<(u32, u32)>,
    buffer: usize,
    source: Option<host::Source>,
}
enum Body {
    Factory,
    Device(Device),
}
struct Object {
    refs: u32,
    wide: bool,
    body: Body,
}
#[derive(Default)]
pub(super) struct State {
    objects: BTreeMap<u32, Object>,
    tables: BTreeMap<u32, u32>,
}
fn checked(image: &LoadedImage32<'_>, p: u32, n: usize, write: bool) {
    if p == 0 || !image.permits(p as usize, n, write) {
        unsupported("DirectInput pointer outside owned guest storage");
    }
}
fn read<const N: usize>(image: &LoadedImage32<'_>, p: u32) -> [u32; N] {
    checked(image, p, N * 4, false);
    unsafe { (p as *const [u32; N]).read_unaligned() }
}
fn guid(image: &LoadedImage32<'_>, p: u32) -> [u8; 16] {
    checked(image, p, 16, false);
    unsafe { (p as *const [u8; 16]).read_unaligned() }
}
fn write(image: &LoadedImage32<'_>, p: u32, bytes: &[u8]) {
    checked(image, p, bytes.len(), true);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), p as *mut u8, bytes.len());
    }
}
fn words(image: &LoadedImage32<'_>, p: u32, v: &[u32]) {
    write(
        image,
        p,
        &v.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>(),
    );
}
fn addref(s: &mut State, p: u32) -> u32 {
    let o = s
        .objects
        .get_mut(&p)
        .unwrap_or_else(|| unsupported("DirectInput reference is dead"));
    o.refs = o
        .refs
        .checked_add(1)
        .unwrap_or_else(|| unsupported("DirectInput reference overflow"));
    o.refs
}
fn release(s: &mut State, p: u32) -> u32 {
    let o = s
        .objects
        .get_mut(&p)
        .unwrap_or_else(|| unsupported("DirectInput Release is dead"));
    o.refs -= 1;
    if o.refs != 0 {
        return o.refs;
    }
    let o = s.objects.remove(&p).unwrap();
    if let Body::Device(device) = o.body {
        let parent = device.parent;
        drop(device);
        release(s, parent);
    }
    0
}
fn allocate(cpu: &Cpu32, s: &mut State, wide: bool, body: Body) -> u32 {
    if s.objects.len() >= 128 {
        unsupported("DirectInput object budget exhausted");
    }
    let kind = if matches!(body, Body::Factory) { 0 } else { 1 };
    let table = table(cpu, s, kind).unwrap_or_else(|e| unsupported(&e));
    let p = d3d9_32::memory(cpu, &table.to_le_bytes(), false, false)
        .unwrap_or_else(|e| unsupported(&e));
    s.objects.insert(
        p,
        Object {
            refs: 1,
            wide,
            body,
        },
    );
    p
}
fn table(cpu: &Cpu32, s: &mut State, kind: u32) -> Result<u32> {
    if let Some(&p) = s.tables.get(&kind) {
        return Ok(p);
    }
    let count = if kind == 0 { 11 } else { 32 };
    let m = cpu.mapping(2 * PAGE, PAGE, 0)?;
    let base = m.base as u32;
    let mut bytes = vec![0; 2 * PAGE];
    let mut cs = 0u16;
    unsafe {
        std::arch::asm!("mov {0:x}, cs",out(reg)cs,options(nomem,nostack,preserves_flags));
    }
    for slot in 0..count {
        let id = BASE + kind * 256 + slot;
        let at = PAGE + slot as usize * 32;
        let address = base + at as u32;
        bytes[slot as usize * 4..slot as usize * 4 + 4].copy_from_slice(&address.to_le_bytes());
        bytes[at] = 0x68;
        bytes[at + 1..at + 5].copy_from_slice(&(address + 16).to_le_bytes());
        bytes[at + 5] = 0x68;
        bytes[at + 6..at + 10].copy_from_slice(&id.to_le_bytes());
        bytes[at + 10..at + 12].copy_from_slice(&[0xff, 0x2d]);
        bytes[at + 12..at + 16].copy_from_slice(&(address + 20).to_le_bytes());
        bytes[at + 16] = 0xc2;
        bytes[at + 17..at + 19].copy_from_slice(&signature(id).unwrap().1.to_le_bytes());
        bytes[at + 20..at + 24].copy_from_slice(&(cpu.service_gates.get() + 64).to_le_bytes());
        bytes[at + 24..at + 26].copy_from_slice(&cs.to_le_bytes());
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), m.base, bytes.len());
    }
    m.protect(0, PAGE, 1)?;
    m.protect(PAGE, PAGE, 5)?;
    cpu.com_storage.borrow_mut().push((m, false));
    s.tables.insert(kind, base);
    Ok(base)
}
pub(super) fn create(image: &LoadedImage32<'_>, a: &[u32; 16]) -> u32 {
    if a[3] == 0 || a[2] == 0 {
        return INVALID;
    }
    let iid = guid(image, a[2]);
    words(image, a[3], &[0]);
    if a[4] != 0 {
        return 0x80040110;
    }
    if a[1] != 0x800 || a[0] == 0 {
        return INVALID;
    }
    if iid != INPUT_A && iid != INPUT_W {
        return 0x80004002;
    }
    let pointer = allocate(
        &image.cpu,
        &mut image.cpu.input8.borrow_mut(),
        iid == INPUT_W,
        Body::Factory,
    );
    words(image, a[3], &[pointer]);
    0
}
fn text(bytes: &mut [u8], offset: usize, wide: bool, text: &str) {
    if wide {
        for (n, c) in text.encode_utf16().take(259).enumerate() {
            bytes[offset + 2 * n..offset + 2 * n + 2].copy_from_slice(&c.to_le_bytes());
        }
    } else {
        let text = text.as_bytes();
        let n = text.len().min(259);
        bytes[offset..offset + n].copy_from_slice(&text[..n]);
    }
}
fn device_info(kind: Kind, wide: bool) -> Vec<u8> {
    let size = if wide { 1100usize } else { 580 };
    let mut data = vec![0; size];
    data[..4].copy_from_slice(&(size as u32).to_le_bytes());
    data[4..20].copy_from_slice(&system(kind));
    data[20..36].copy_from_slice(&system(kind));
    data[36..40].copy_from_slice(
        &(if kind == Kind::Keyboard {
            0x13u32
        } else {
            0x12
        })
        .to_le_bytes(),
    );
    let name = if kind == Kind::Keyboard {
        "Alkyhol Quartz system keyboard"
    } else {
        "Alkyhol Quartz system mouse"
    };
    text(&mut data, 40, wide, name);
    text(&mut data, 40 + if wide { 520 } else { 260 }, wide, name);
    let tail = size - 4;
    data[tail..tail + 2].copy_from_slice(&1u16.to_le_bytes());
    data[tail + 2..]
        .copy_from_slice(&(if kind == Kind::Keyboard { 6u16 } else { 2 }).to_le_bytes());
    data
}
fn components(kind: Kind) -> Vec<u16> {
    if kind == Kind::Keyboard {
        (0..256).collect()
    } else {
        (256..259).chain(512..520).collect()
    }
}
fn object_type(component: u16) -> u32 {
    if component < 256 {
        12 | ((component as u32) << 8)
    } else if component < 259 {
        1 | (((component - 256) as u32) << 8)
    } else {
        12 | (((component - 512) as u32) << 8)
    }
}
fn object_info(component: u16, offset: u32, wide: bool) -> Vec<u8> {
    let size = if wide { 576usize } else { 316 };
    let mut v = vec![0; size];
    v[..4].copy_from_slice(&(size as u32).to_le_bytes());
    v[4..20].copy_from_slice(&object_guid(component));
    v[20..24].copy_from_slice(&offset.to_le_bytes());
    v[24..28].copy_from_slice(&object_type(component).to_le_bytes());
    text(
        &mut v,
        32,
        wide,
        &format!("Alkyhol input object {component}"),
    );
    v
}
fn data_format(
    image: &LoadedImage32<'_>,
    pointer: u32,
    kind: Kind,
) -> std::result::Result<Format, u32> {
    let a = read::<6>(image, pointer);
    if a[0] != 24
        || a[1] != 16
        || a[3] == 0
        || a[3] > 4096
        || a[4] > 512
        || a[5] == 0
        || ![1, 2].contains(&a[2])
    {
        return Err(INVALID);
    }
    if kind == Kind::Mouse && a[2] != 2 {
        return Err(NOTIMPL);
    }
    let mut bindings = Vec::new();
    let mut used = BTreeSet::new();
    let mut ranges = BTreeSet::new();
    for i in 0..a[4] {
        let p = a[5].checked_add(i * 16).ok_or(INVALID)?;
        let o = read::<4>(image, p);
        if o[3] != 0 || o[2] & !0x80ffff0f != 0 {
            return Err(NOTIMPL);
        }
        let instance = (o[2] >> 8) & 65535;
        let ty = o[2] & 255;
        let requested = if o[0] == 0 {
            None
        } else {
            Some(guid(image, o[0]))
        };
        let component = components(kind).into_iter().find(|&component| {
            let expected_type = object_type(component);
            let index = (expected_type >> 8) & 65535;
            !used.contains(&component)
                && (instance == 65535 || index == instance)
                && ((ty & 3 != 0 && (256..259).contains(&component))
                    || (ty & 12 != 0 && !(256..512).contains(&component)))
                && requested.is_none_or(|g| g == object_guid(component))
        });
        let Some(component) = component else {
            if o[2] & 0x80000000 != 0 {
                continue;
            }
            return Err(0x80070002);
        };
        let width = if (256..259).contains(&component) {
            4
        } else {
            1
        };
        let end = o[1].checked_add(width).ok_or(INVALID)?;
        if end > a[3] || (o[1]..end).any(|n| !ranges.insert(n)) {
            return Err(INVALID);
        }
        used.insert(component);
        bindings.push(Binding {
            offset: o[1] as usize,
            component,
            appdata: u32::MAX,
        });
    }
    Ok(Format {
        size: a[3] as usize,
        bindings,
    })
}
fn device_mut(s: &mut State, p: u32) -> &mut Device {
    let Body::Device(device) = &mut s.objects.get_mut(&p).unwrap().body else {
        unsupported("DirectInput device type mismatch")
    };
    device
}
fn callback(image: &LoadedImage32<'_>, procedure: u32, context: u32, data: &[u8]) -> u32 {
    let p = d3d9_32::memory(&image.cpu, data, false, false).unwrap_or_else(|e| unsupported(&e));
    unsafe { image.callback_frame(procedure, &[p, context], CallingConvention32::Stdcall) }
        .unwrap_or_else(|e| unsupported(&e))
        .unwrap_or(0)
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, a: &[u32; 16]) -> u32 {
    let class = (id - BASE) / 256;
    let slot = (id - BASE) % 256;
    let mut s = image.cpu.input8.borrow_mut();
    let o = s
        .objects
        .get(&a[0])
        .unwrap_or_else(|| unsupported("DirectInput this pointer is dead"));
    let actual = if matches!(o.body, Body::Factory) {
        0
    } else {
        1
    };
    if actual != class {
        unsupported("DirectInput interface type mismatch");
    }
    let wide = o.wide;
    eprintln!(
        "PE32 DirectInput COM: class={class} slot={slot} args={:x?}",
        &a[..signature(id).unwrap().1 as usize / 4]
    );
    match slot {
        0 => {
            if a[2] == 0 {
                return 0x80004003;
            }
            let iid = guid(image, a[1]);
            words(image, a[2], &[0]);
            let own = if class == 0 {
                if wide { INPUT_W } else { INPUT_A }
            } else if wide {
                DEVICE_W
            } else {
                DEVICE_A
            };
            if iid != UNKNOWN && iid != own {
                return 0x80004002;
            }
            addref(&mut s, a[0]);
            words(image, a[2], &[a[0]]);
            return 0;
        }
        1 => return addref(&mut s, a[0]),
        2 => return release(&mut s, a[0]),
        _ => (),
    }
    match (class, slot) {
        (0, 3) => {
            if a[2] == 0 {
                return INVALID;
            }
            let requested = guid(image, a[1]);
            words(image, a[2], &[0]);
            if a[3] != 0 {
                return 0x80040110;
            }
            let Some(kind) = kind(requested) else {
                return 0x80070002;
            };
            addref(&mut s, a[0]);
            let device = allocate(
                &image.cpu,
                &mut s,
                wide,
                Body::Device(Device {
                    parent: a[0],
                    kind,
                    format: None,
                    cooperative: None,
                    buffer: 0,
                    source: None,
                }),
            );
            words(image, a[2], &[device]);
            0
        }
        (0, 4) => {
            if a[2] == 0 || a[4] & !0x70101 != 0 {
                return INVALID;
            }
            if a[1] == 1 {
                return NOTIMPL;
            }
            let devices: Vec<_> = [Kind::Keyboard, Kind::Mouse]
                .into_iter()
                .filter(|kind| {
                    a[4] & 256 == 0
                        && (a[1] == 0
                            || a[1] == if *kind == Kind::Keyboard { 3 } else { 2 }
                            || a[1] == if *kind == Kind::Keyboard { 0x13 } else { 0x12 })
                })
                .collect();
            drop(s);
            for kind in devices {
                if callback(image, a[2], a[3], &device_info(kind, wide)) == 0 {
                    break;
                }
            }
            0
        }
        (0, 5) => {
            if kind(guid(image, a[1])).is_some() {
                0
            } else {
                0x80070002
            }
        }
        (0, 7) | (1, 17) => 0x800704df,
        (1, 3) => {
            let device = device_mut(&mut s, a[0]);
            if read::<1>(image, a[1])[0] != 44 {
                return INVALID;
            }
            let (axes, buttons) = if device.kind == Kind::Keyboard {
                (0, 256)
            } else {
                (3, 8)
            };
            words(
                image,
                a[1],
                &[
                    44,
                    5,
                    if device.kind == Kind::Keyboard {
                        0x13
                    } else {
                        0x12
                    },
                    axes,
                    buttons,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
            );
            0
        }
        (1, 4) => {
            if a[1] == 0 {
                return INVALID;
            }
            let device = device_mut(&mut s, a[0]);
            let list = components(device.kind);
            drop(s);
            for component in list {
                let ty = object_type(component);
                if a[3] == 0 || ty & a[3] & 255 != 0 {
                    let offset = if component < 256 {
                        component as u32
                    } else if component < 259 {
                        (component as u32 - 256) * 4
                    } else {
                        12 + component as u32 - 512
                    };
                    if callback(image, a[1], a[2], &object_info(component, offset, wide)) == 0 {
                        break;
                    }
                }
            }
            0
        }
        (1, 5 | 6) => {
            let device = device_mut(&mut s, a[0]);
            let header = read::<5>(image, a[2]);
            if header[0] != 20 || header[1] != 16 {
                return INVALID;
            }
            if slot == 6 && device.source.is_some() {
                return 0x800700aa;
            }
            match a[1] {
                1 | 2 => {
                    if header[2] != 0 || header[3] != 0 {
                        return INVALID;
                    }
                    if a[1] == 2 && device.kind != Kind::Mouse {
                        return NOTIMPL;
                    }
                    if slot == 6 {
                        if a[1] == 1 {
                            if header[4] > 4096 {
                                return NOTIMPL;
                            }
                            device.buffer = header[4] as usize;
                        } else if header[4] != 1 {
                            return NOTIMPL;
                        }
                    } else {
                        words(
                            image,
                            a[2] + 16,
                            &[if a[1] == 1 { device.buffer as u32 } else { 1 }],
                        );
                    }
                    0
                }
                22 => {
                    let Some(format) = &mut device.format else {
                        return 0x80070015;
                    };
                    let selected = format.bindings.iter_mut().find(|b| {
                        if header[3] == 1 {
                            b.offset == header[2] as usize
                        } else if header[3] == 2 {
                            object_type(b.component) == header[2]
                        } else {
                            false
                        }
                    });
                    let Some(binding) = selected else {
                        return 0x80070002;
                    };
                    if slot == 6 {
                        binding.appdata = header[4];
                    } else {
                        words(image, a[2] + 16, &[binding.appdata]);
                    }
                    0
                }
                _ => NOTIMPL,
            }
        }
        (1, 7) => {
            let device = device_mut(&mut s, a[0]);
            if device.source.as_ref().is_some_and(|s| !s.lost()) {
                return 1;
            }
            device.source = None;
            let Some(format) = &device.format else {
                return INVALID;
            };
            let Some((window, flags)) = device.cooperative else {
                return INVALID;
            };
            if flags & (1 | 8 | 16) != 0 {
                eprintln!("DirectInput acquisition mode is not implemented: {flags:#x}");
                return NOTIMPL;
            }
            if !image.cpu.ui.borrow().input_foreground(window) {
                return 0x80070005;
            }
            match host::Source::new(
                device.kind,
                format.size,
                format.bindings.clone(),
                device.buffer,
            ) {
                Ok(source) => {
                    device.source = Some(source);
                    0
                }
                Err(e) => {
                    eprintln!("DirectInput native acquisition failed: {e}");
                    0x80070005
                }
            }
        }
        (1, 8) => {
            let device = device_mut(&mut s, a[0]);
            if device.source.take().is_some() { 0 } else { 1 }
        }
        (1, 9) => {
            let device = device_mut(&mut s, a[0]);
            if device.source.is_none() {
                return NOTACQUIRED;
            }
            if let Some((window, _)) = device.cooperative
                && !image.cpu.ui.borrow().input_foreground(window)
            {
                device.source = None;
                return INPUTLOST;
            }
            let Some(source) = &device.source else {
                return NOTACQUIRED;
            };
            let Some(format) = &device.format else {
                return INVALID;
            };
            if a[1] as usize != format.size {
                return INVALID;
            }
            match source.state() {
                Ok(bytes) => {
                    write(image, a[2], &bytes);
                    0
                }
                Err(_) => {
                    device.source = None;
                    INPUTLOST
                }
            }
        }
        (1, 10) => {
            let device = device_mut(&mut s, a[0]);
            if device.source.is_some()
                && device
                    .cooperative
                    .is_some_and(|(window, _)| !image.cpu.ui.borrow().input_foreground(window))
            {
                device.source = None;
                return INPUTLOST;
            }
            if device.buffer == 0 {
                return 0x80040207;
            }
            let Some(source) = &device.source else {
                return NOTACQUIRED;
            };
            if ![16, 20].contains(&a[1]) || a[3] == 0 || a[4] & !1 != 0 {
                return INVALID;
            }
            let count = read::<1>(image, a[3])[0] as usize;
            let result = source.records(count, a[4] & 1 != 0);
            match result {
                Ok((records, overflow)) => {
                    if a[2] != 0 {
                        let bytes: Vec<_> = records
                            .iter()
                            .flat_map(|r| {
                                r[..(a[1] / 4) as usize]
                                    .iter()
                                    .flat_map(|n| n.to_le_bytes())
                            })
                            .collect();
                        write(image, a[2], &bytes);
                    }
                    words(image, a[3], &[records.len() as u32]);
                    overflow as u32
                }
                Err(_) => {
                    device.source = None;
                    INPUTLOST
                }
            }
        }
        (1, 11) => {
            let device = device_mut(&mut s, a[0]);
            if device.source.is_some() {
                return 0x800700aa;
            }
            match data_format(image, a[1], device.kind) {
                Ok(format) => {
                    device.format = Some(format);
                    0
                }
                Err(e) => e,
            }
        }
        (1, 12) => {
            if a[1] == 0 {
                0
            } else {
                NOTIMPL
            }
        }
        (1, 13) => {
            let device = device_mut(&mut s, a[0]);
            if device.source.is_some() {
                return 0x800700aa;
            }
            if a[2] & !31 != 0 || ![1, 2].contains(&(a[2] & 3)) || ![4, 8].contains(&(a[2] & 12)) {
                return INVALID;
            }
            if a[2] & 4 != 0 && !image.cpu.ui.borrow().input_window(a[1]) {
                return 0x80070006;
            }
            device.cooperative = Some((a[1], a[2]));
            0
        }
        (1, 14) => {
            let device = device_mut(&mut s, a[0]);
            let component = if a[3] == 1 {
                device
                    .format
                    .as_ref()
                    .and_then(|f| f.bindings.iter().find(|b| b.offset == a[2] as usize))
                    .map(|b| b.component)
            } else if a[3] == 2 {
                components(device.kind)
                    .into_iter()
                    .find(|&c| object_type(c) == a[2])
            } else {
                None
            };
            let Some(component) = component else {
                return 0x80070002;
            };
            let size = if wide { 576 } else { 316 };
            if read::<1>(image, a[1])[0] != size {
                return INVALID;
            }
            write(
                image,
                a[1],
                &object_info(component, if a[3] == 1 { a[2] } else { 0 }, wide),
            );
            0
        }
        (1, 15) => {
            let device = device_mut(&mut s, a[0]);
            let bytes = device_info(device.kind, wide);
            if read::<1>(image, a[1])[0] as usize != bytes.len() {
                return INVALID;
            }
            write(image, a[1], &bytes);
            0
        }
        (1, 25) => {
            let device = device_mut(&mut s, a[0]);
            match &device.source {
                Some(source) if source.lost() => INPUTLOST,
                Some(_) => 0,
                None => NOTACQUIRED,
            }
        }
        _ => {
            eprintln!("DirectInput operation unavailable: class={class} slot={slot}");
            NOTIMPL
        }
    }
}

// Documented interface order and i386 extents; no unknown method returns success.
const METHODS0: [(&str, u16); 11] = [
    ("QueryInterface", 12),
    ("AddRef", 4),
    ("Release", 4),
    ("CreateDevice", 16),
    ("EnumDevices", 20),
    ("GetDeviceStatus", 8),
    ("RunControlPanel", 12),
    ("Initialize", 12),
    ("FindDevice", 16),
    ("EnumDevicesBySemantics", 24),
    ("ConfigureDevices", 20),
];
const METHODS1: [(&str, u16); 32] = [
    ("QueryInterface", 12),
    ("AddRef", 4),
    ("Release", 4),
    ("GetCapabilities", 8),
    ("EnumObjects", 16),
    ("GetProperty", 12),
    ("SetProperty", 12),
    ("Acquire", 4),
    ("Unacquire", 4),
    ("GetDeviceState", 12),
    ("GetDeviceData", 20),
    ("SetDataFormat", 8),
    ("SetEventNotification", 8),
    ("SetCooperativeLevel", 12),
    ("GetObjectInfo", 16),
    ("GetDeviceInfo", 8),
    ("RunControlPanel", 12),
    ("Initialize", 16),
    ("CreateEffect", 20),
    ("EnumEffects", 16),
    ("GetEffectInfo", 12),
    ("GetForceFeedbackState", 8),
    ("SendForceFeedbackCommand", 8),
    ("EnumCreatedEffectObjects", 16),
    ("Escape", 8),
    ("Poll", 4),
    ("SendDeviceData", 20),
    ("EnumEffectsInFile", 20),
    ("WriteEffectToFile", 20),
    ("BuildActionMap", 16),
    ("SetActionMap", 16),
    ("GetImageInfo", 8),
];
pub(super) fn signature(id: u32) -> Option<(&'static str, u16)> {
    let n = id.checked_sub(BASE)?;
    let table: &[(&str, u16)] = match n / 256 {
        0 => &METHODS0,
        1 => &METHODS1,
        _ => return None,
    };
    table.get((n % 256) as usize).copied()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sdk_interface_extents_and_virtual_device_descriptors_are_bounded() {
        assert_eq!(signature(BASE + 3), Some(("CreateDevice", 16)));
        assert_eq!(signature(BASE + 256 + 10), Some(("GetDeviceData", 20)));
        assert!(signature(BASE + 11).is_none());
        assert!(signature(BASE + 256 + 32).is_none());
        assert_eq!(device_info(Kind::Keyboard, true).len(), 1100);
        assert_eq!(device_info(Kind::Mouse, false).len(), 580);
        assert_eq!(object_info(30, 0, true).len(), 576);
        assert_eq!(object_info(256, 0, false).len(), 316);
        assert_eq!(components(Kind::Keyboard).len(), 256);
        assert_eq!(components(Kind::Mouse).len(), 11);
    }
}
