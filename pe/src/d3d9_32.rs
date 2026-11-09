//! PE32 stdcall COM over owned low vtables. No host pointer is published.
//! Unsupported virtual methods stop before returning; no successful COM stubs.
use super::{Cpu32, LoadedImage32, PAGE, unsupported};
use crate::Result;
use alkyhol_d3d9::{Device as Gpu, Target};
use std::{collections::BTreeMap, rc::Rc};
pub(super) const BASE: u32 = 0x9000;
const INVALID: u32 = 0x8876086c;
const UNAVAILABLE: u32 = 0x8876086a;
const NO_INTERFACE: u32 = 0x80004002;
const POINTER: u32 = 0x80004003;
const IID_UNKNOWN: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0xc0, 0, 0, 0, 0, 0, 0, 0x46];
const IID_RESOURCE: [u8; 16] = [
    0x5d, 0xc0, 0xee, 0x05, 0x7d, 0x8f, 0x62, 0x43, 0xb9, 0x99, 0xd1, 0xba, 0xf3, 0x57, 0xc7, 0x04,
];
const IIDS: [[u8; 16]; 4] = [
    [
        0xca, 0xcb, 0xbd, 0x81, 0xd4, 0x64, 0x6d, 0x42, 0xae, 0x8d, 0xad, 1, 0x47, 0xf4, 0x27, 0x5c,
    ],
    [
        0x96, 0x3b, 0x22, 0xd0, 0x7a, 0xbf, 0xfd, 0x43, 0x92, 0xbd, 0xa4, 0x3b, 0x0d, 0x82, 0xb9,
        0xeb,
    ],
    [
        0x3a, 0xaf, 0xfb, 0x0c, 0xf6, 0x9f, 0x9a, 0x42, 0x99, 0xb3, 0xa2, 0x79, 0x6a, 0xf8, 0xb8,
        0x9b,
    ],
    [
        0xb5, 0xb1, 0x4b, 0xb6, 0x70, 0xfd, 0xf6, 0x4d, 0xbf, 0x91, 0x19, 0xd0, 0xa1, 0x24, 0x55,
        0xe3,
    ],
];
struct Object {
    refs: u32,
    body: Body,
}
enum Body {
    Factory(Rc<Gpu>),
    Device {
        parent: u32,
        gpu: Rc<Gpu>,
        target: Target,
        hwnd: u32,
        scene: bool,
        parameters: [u32; 14],
        back: u32,
        fvf: u32,
        stream: Option<(u32, u32, u32)>,
    },
    Surface {
        parent: u32,
        back: bool,
        data: u32,
        width: u32,
        height: u32,
        format: u32,
        initialized: bool,
        locked: bool,
    },
    VertexBuffer {
        parent: u32,
        data: u32,
        length: u32,
        fvf: u32,
        lock: Option<(u32, u32)>,
        initialized: Vec<(u32, u32)>,
        bindings: u32,
    },
}
impl Body {
    fn kind(&self) -> u32 {
        match self {
            Self::Factory(_) => 0,
            Self::Device { .. } => 1,
            Self::Surface { .. } => 2,
            Self::VertexBuffer { .. } => 3,
        }
    }
}
#[derive(Default)]
pub(super) struct State {
    tables: BTreeMap<u32, u32>,
    objects: BTreeMap<u32, Object>,
}
// Exact stack sizes for implemented slots. Unknown slots never return to the
// guest, so their unsupported gate only snapshots this and terminates explicitly.
pub(super) fn signature(id: u32) -> Option<(&'static str, u16)> {
    let n = id.checked_sub(BASE)?;
    let kind = n / 256;
    let slot = n % 256;
    if slot
        >= match kind {
            0 => 17,
            1 => 119,
            2 => 17,
            3 => 14,
            _ => return None,
        }
    {
        return None;
    }
    if slot == 0 {
        return Some(("D3D9 QueryInterface", 12));
    }
    if slot == 1 {
        return Some(("D3D9 AddRef", 4));
    }
    if slot == 2 {
        return Some(("D3D9 Release", 4));
    }
    Some(match (kind, slot) {
        (0, 3) => ("IDirect3D9::RegisterSoftwareDevice", 8),
        (0, 4) => ("IDirect3D9::GetAdapterCount", 4),
        (0, 6) => ("IDirect3D9::GetAdapterModeCount", 12),
        (0, 7) => ("IDirect3D9::EnumAdapterModes", 20),
        (0, 8) => ("IDirect3D9::GetAdapterDisplayMode", 12),
        (0, 9) => ("IDirect3D9::CheckDeviceType", 24),
        (0, 10) => ("IDirect3D9::CheckDeviceFormat", 28),
        (0, 11) => ("IDirect3D9::CheckDeviceMultiSampleType", 28),
        (0, 12) => ("IDirect3D9::CheckDepthStencilMatch", 24),
        (0, 13) => ("IDirect3D9::CheckDeviceFormatConversion", 20),
        (0, 14) => ("IDirect3D9::GetDeviceCaps", 16),
        (0, 15) => ("IDirect3D9::GetAdapterMonitor", 8),
        (0, 5) => ("IDirect3D9::GetAdapterIdentifier", 16),
        (0, 16) => ("IDirect3D9::CreateDevice", 28),
        (1, 3) => ("IDirect3DDevice9::TestCooperativeLevel", 4),
        (1, 26) => ("IDirect3DDevice9::CreateVertexBuffer", 28),
        (1, 81) => ("IDirect3DDevice9::DrawPrimitive", 16),
        (1, 89) => ("IDirect3DDevice9::SetFVF", 8),
        (1, 100) => ("IDirect3DDevice9::SetStreamSource", 20),
        (3, 3) => ("IDirect3DVertexBuffer9::GetDevice", 8),
        (3, 10) => ("IDirect3DVertexBuffer9::GetType", 4),
        (3, 11) => ("IDirect3DVertexBuffer9::Lock", 20),
        (3, 12) => ("IDirect3DVertexBuffer9::Unlock", 4),
        (3, 13) => ("IDirect3DVertexBuffer9::GetDesc", 8),
        (1, 6) => ("IDirect3DDevice9::GetDirect3D", 8),
        (1, 9) => ("IDirect3DDevice9::GetCreationParameters", 8),
        (1, 17) => ("IDirect3DDevice9::Present", 20),
        (1, 18) => ("IDirect3DDevice9::GetBackBuffer", 20),
        (1, 32) => ("IDirect3DDevice9::GetRenderTargetData", 12),
        (1, 36) => ("IDirect3DDevice9::CreateOffscreenPlainSurface", 28),
        (1, 41) => ("IDirect3DDevice9::BeginScene", 4),
        (1, 42) => ("IDirect3DDevice9::EndScene", 4),
        (1, 43) => ("IDirect3DDevice9::Clear", 28),
        (2, 3) => ("IDirect3DSurface9::GetDevice", 8),
        (2, 10) => ("IDirect3DSurface9::GetType", 4),
        (2, 12) => ("IDirect3DSurface9::GetDesc", 8),
        (2, 13) => ("IDirect3DSurface9::LockRect", 16),
        (2, 14) => ("IDirect3DSurface9::UnlockRect", 4),
        _ => ("unimplemented D3D9 COM method", 4),
    })
}
fn checked(image: &LoadedImage32<'_>, at: u32, len: usize, write: bool) {
    if at == 0 || !image.permits(at as usize, len, write) {
        unsupported("D3D9 pointer outside owned guest storage");
    }
}
fn read<const N: usize>(image: &LoadedImage32<'_>, at: u32) -> [u32; N] {
    checked(image, at, N * 4, false);
    unsafe { (at as *const [u32; N]).read_unaligned() }
}
fn write(image: &LoadedImage32<'_>, at: u32, words: &[u32]) {
    checked(image, at, words.len() * 4, true);
    for (i, w) in words.iter().enumerate() {
        unsafe { (at as *mut u32).add(i).write_unaligned(*w) }
    }
}
fn memory(cpu: &Cpu32, bytes: &[u8], writable: bool, executable: bool) -> Result<u32> {
    if cpu.d3d9_storage.borrow().len() >= 512 {
        return Err("D3D9 low-storage budget exhausted".into());
    }
    let m = cpu.mapping(bytes.len().div_ceil(PAGE) * PAGE, PAGE, 0)?;
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), m.base, bytes.len());
    }
    m.protect(
        0,
        m.len,
        1 | if writable { 2 } else { 0 } | if executable { 4 } else { 0 },
    )?;
    let address = m.base as u32;
    cpu.d3d9_storage.borrow_mut().push((m, writable));
    Ok(address)
}
fn table(cpu: &Cpu32, state: &mut State, kind: u32) -> Result<u32> {
    if let Some(t) = state.tables.get(&kind) {
        return Ok(*t);
    }
    let count = match kind {
        1 => 119,
        3 => 14,
        _ => 17,
    };
    let m = cpu.mapping(2 * PAGE, PAGE, 0)?;
    let base = m.base as u32;
    let mut bytes = vec![0u8; 2 * PAGE];
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
    cpu.d3d9_storage.borrow_mut().push((m, false));
    state.tables.insert(kind, base);
    Ok(base)
}
fn object(cpu: &Cpu32, state: &mut State, body: Body, refs: u32) -> u32 {
    if state.objects.len() >= 128 {
        unsupported("D3D9 object budget exhausted");
    }
    let t = table(cpu, state, body.kind()).unwrap_or_else(|e| unsupported(&e));
    let pointer = memory(cpu, &t.to_le_bytes(), false, false).unwrap_or_else(|e| unsupported(&e));
    state.objects.insert(pointer, Object { refs, body });
    pointer
}
fn addref(s: &mut State, p: u32) -> u32 {
    let o = s
        .objects
        .get_mut(&p)
        .unwrap_or_else(|| unsupported("D3D9 reference to released object"));
    if o.refs == 0 {
        unsupported("D3D9 AddRef without an external reference");
    }
    o.refs = o
        .refs
        .checked_add(1)
        .unwrap_or_else(|| unsupported("D3D9 reference overflow"));
    o.refs
}
fn release(s: &mut State, p: u32) -> u32 {
    let o = s
        .objects
        .get_mut(&p)
        .unwrap_or_else(|| unsupported("D3D9 Release on dead object"));
    if o.refs == 0 {
        unsupported("D3D9 Release without an external reference");
    }
    o.refs -= 1;
    let refs = o.refs;
    if refs != 0 {
        return refs;
    }
    let parent = match &o.body {
        Body::Factory(_) => None,
        Body::Device { parent, .. }
        | Body::Surface { parent, .. }
        | Body::VertexBuffer { parent, .. } => Some(*parent),
    };
    let cached = matches!(
        o.body,
        Body::Surface { back: true, .. } | Body::VertexBuffer { bindings: 1.., .. }
    );
    if !cached {
        let removed = s.objects.remove(&p).unwrap();
        if let Body::Device { back, stream, .. } = removed.body {
            if back != 0 {
                s.objects.remove(&back);
            }
            if let Some((buffer, _, _)) = stream {
                unbind(s, buffer);
            }
        }
    }
    if let Some(parent) = parent {
        release(s, parent);
    }
    0
}
pub(super) fn create(image: &LoadedImage32<'_>, sdk: u32) -> u32 {
    if sdk != 32 {
        return 0;
    }
    let gpu = match Gpu::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("D3D9 adapter creation failed: {e}");
            return 0;
        }
    };
    if !gpu.hardware() {
        eprintln!("D3D9 HAL factory refuses a software adapter");
        return 0;
    }
    eprintln!(
        "PE32 Direct3DCreate9: actual GPU={} backend={}",
        gpu.adapter_name(),
        gpu.backend_name()
    );
    object(
        &image.cpu,
        &mut image.cpu.d3d9.borrow_mut(),
        Body::Factory(Rc::new(gpu)),
        1,
    )
}
fn identifier(image: &LoadedImage32<'_>, s: &State, a: &[u32; 16]) -> u32 {
    if a[1] != 0 || a[2] & !2 != 0 {
        return INVALID;
    }
    let Body::Factory(gpu) = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let mut bytes = [0u8; 1100];
    for (at, cap, text) in [
        (0, 512, "Alkyhol wgpu D3D9".to_string()),
        (512, 512, gpu.adapter_name().to_owned()),
        (1024, 32, "Alkyhol adapter 0".to_string()),
    ] {
        let b = text.as_bytes();
        let n = b.len().min(cap - 1);
        bytes[at..at + n].copy_from_slice(&b[..n]);
    }
    let (vendor, device) = gpu.vendor_device();
    bytes[1064..1068].copy_from_slice(&vendor.to_le_bytes());
    bytes[1068..1072].copy_from_slice(&device.to_le_bytes());
    // Explicit synthetic compatibility-adapter identity, never a claimed Windows driver GUID/WHQL record.
    bytes[1080..1096].copy_from_slice(b"Alkyhol-wgpu-d9\0");
    checked(image, a[3], bytes.len(), true);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), a[3] as *mut u8, bytes.len());
    }
    0
}
fn device(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[6] == 0 {
        return INVALID;
    }
    write(image, a[6], &[0]);
    if a[1] != 0 || a[2] != 1 || a[4] != 0x20 || a[5] == 0 {
        return UNAVAILABLE;
    }
    let mut p = read::<14>(image, a[5]);
    if p[3] > 1
        || p[4] != 0
        || p[5] != 0
        || p[6] != 1
        || p[8] != 1
        || p[9] != 0
        || p[11] != 0
        || p[12] != 0
        || p[13] != 0
    {
        return UNAVAILABLE;
    }
    let hwnd = if p[7] == 0 { a[3] } else { p[7] };
    let (w, h) = match image.cpu.ui.borrow().graphics_client(hwnd) {
        Ok(v) => v,
        Err(_) => return INVALID,
    };
    if p[0] == 0 {
        p[0] = w;
    }
    if p[1] == 0 {
        p[1] = h;
    }
    if p[2] == 0 {
        p[2] = 22;
    }
    if ![21, 22].contains(&p[2]) {
        return UNAVAILABLE;
    }
    p[3] = 1;
    let Body::Factory(gpu) = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let gpu = gpu.clone();
    let target = match gpu.create_target(p[0], p[1]) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("D3D9 target allocation: {e}");
            return UNAVAILABLE;
        }
    };
    addref(s, a[0]);
    let o = object(
        &image.cpu,
        s,
        Body::Device {
            parent: a[0],
            gpu,
            target,
            hwnd,
            scene: false,
            parameters: p,
            back: 0,
            fvf: 0,
            stream: None,
        },
        1,
    );
    write(image, a[5], &p);
    write(image, a[6], &[o]);
    0
}
fn backbuffer(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[4] == 0 {
        return INVALID;
    }
    write(image, a[4], &[0]);
    if a[1] != 0 || a[2] != 0 || a[3] != 0 {
        return INVALID;
    }
    let Body::Device {
        parameters, back, ..
    } = &s.objects[&a[0]].body
    else {
        unreachable!()
    };
    let mut b = *back;
    let p = *parameters;
    if b == 0 {
        b = object(
            &image.cpu,
            s,
            Body::Surface {
                parent: a[0],
                back: true,
                data: 0,
                width: p[0],
                height: p[1],
                format: p[2],
                initialized: false,
                locked: false,
            },
            0,
        );
        let Body::Device { back, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
            unreachable!()
        };
        *back = b;
    }
    let surface = s.objects.get_mut(&b).unwrap();
    if surface.refs == 0 {
        surface.refs = 1;
        addref(s, a[0]);
    } else {
        surface.refs += 1;
    }
    write(image, a[4], &[b]);
    0
}
fn offscreen(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[5] == 0 {
        return INVALID;
    }
    write(image, a[5], &[0]);
    if a[1] == 0
        || a[2] == 0
        || a[1] > 8192
        || a[2] > 8192
        || ![21, 22].contains(&a[3])
        || a[4] != 2
        || a[6] != 0
    {
        return UNAVAILABLE;
    }
    let data = memory(
        &image.cpu,
        &vec![0u8; a[1] as usize * a[2] as usize * 4],
        true,
        false,
    )
    .unwrap_or_else(|e| unsupported(&e));
    addref(s, a[0]);
    let o = object(
        &image.cpu,
        s,
        Body::Surface {
            parent: a[0],
            back: false,
            data,
            width: a[1],
            height: a[2],
            format: a[3],
            initialized: false,
            locked: false,
        },
        1,
    );
    write(image, a[5], &[o]);
    0
}
fn transfer(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    let Some(Object {
        refs: 1..,
        body: Body::Surface {
            parent, back: true, ..
        },
    }) = s.objects.get(&a[1])
    else {
        return INVALID;
    };
    if *parent != a[0] {
        return INVALID;
    }
    let Some(Object {
        refs: 1..,
        body:
            Body::Surface {
                parent,
                back: false,
                data,
                width,
                height,
                format,
                locked: false,
                ..
            },
    }) = s.objects.get(&a[2])
    else {
        return INVALID;
    };
    if *parent != a[0] {
        return INVALID;
    }
    let (data, w, h, f) = (*data, *width, *height, *format);
    let Body::Device {
        gpu,
        target,
        parameters,
        ..
    } = &s.objects[&a[0]].body
    else {
        unreachable!()
    };
    if (w, h, f) != (parameters[0], parameters[1], parameters[2]) {
        return INVALID;
    }
    let bytes = gpu.readback(target).unwrap_or_else(|e| unsupported(&e));
    checked(image, data, bytes.len(), true);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), data as *mut u8, bytes.len());
    }
    let Body::Surface { initialized, .. } = &mut s.objects.get_mut(&a[2]).unwrap().body else {
        unreachable!()
    };
    *initialized = true;
    0
}
// The host enforces owned-storage safety even for invalid calls a native driver
// happens to accept. Only valid byte ranges are in the portable lock contract.
fn lock_length(length: u32, offset: u32, size: u32) -> Option<u32> {
    let n = if size == 0 {
        length.checked_sub(offset)?
    } else {
        size
    };
    (n != 0 && offset.checked_add(n)? <= length).then_some(n)
}
fn vertex_buffer(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[5] == 0 {
        return INVALID;
    }
    write(image, a[5], &[0]);
    if a[1] == 0 || a[1] > 1024 * 1024 || a[2] != 0 || a[3] != 0x44 || a[4] != 0 || a[6] != 0 {
        return UNAVAILABLE;
    }
    let data = memory(&image.cpu, &vec![0u8; a[1] as usize], true, false)
        .unwrap_or_else(|e| unsupported(&e));
    addref(s, a[0]);
    let p = object(
        &image.cpu,
        s,
        Body::VertexBuffer {
            parent: a[0],
            data,
            length: a[1],
            fvf: a[3],
            lock: None,
            initialized: Vec::new(),
            bindings: 0,
        },
        1,
    );
    write(image, a[5], &[p]);
    0
}
fn unbind(s: &mut State, p: u32) {
    let Some(o) = s.objects.get_mut(&p) else {
        unsupported("missing bound vertex buffer");
    };
    let Body::VertexBuffer { bindings, .. } = &mut o.body else {
        unreachable!()
    };
    *bindings = bindings
        .checked_sub(1)
        .unwrap_or_else(|| unsupported("vertex binding count underflow"));
    if *bindings == 0 && o.refs == 0 {
        s.objects.remove(&p);
    }
}
fn stream_source(s: &mut State, a: &[u32; 16]) -> u32 {
    if a[1] != 0 {
        return UNAVAILABLE;
    }
    if a[2] != 0 {
        let Some(Object {
            refs: 1..,
            body:
                Body::VertexBuffer {
                    parent,
                    length,
                    lock: None,
                    ..
                },
        }) = s.objects.get(&a[2])
        else {
            return INVALID;
        };
        if *parent != a[0] || a[3] > *length || a[4] != 20 {
            return INVALID;
        }
    }
    let Body::Device { stream, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let old = *stream;
    if old.map(|v| v.0) != Some(a[2]) {
        if a[2] != 0 {
            let Body::VertexBuffer { bindings, .. } = &mut s.objects.get_mut(&a[2]).unwrap().body
            else {
                unreachable!()
            };
            *bindings += 1;
        }
        if let Some((buffer, _, _)) = old {
            unbind(s, buffer);
        }
    }
    let Body::Device { stream, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
        unreachable!()
    };
    *stream = if a[2] == 0 {
        None
    } else {
        Some((a[2], a[3], a[4]))
    };
    0
}
fn draw(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[1] != 4 || a[3] == 0 {
        return UNAVAILABLE;
    }
    let Body::Device {
        scene, fvf, stream, ..
    } = &s.objects[&a[0]].body
    else {
        unreachable!()
    };
    if !*scene || *fvf != 0x44 {
        return INVALID;
    }
    let Some((p, offset, stride)) = *stream else {
        return INVALID;
    };
    let Body::VertexBuffer {
        data,
        length,
        lock,
        initialized,
        ..
    } = &s.objects[&p].body
    else {
        unreachable!()
    };
    if lock.is_some() {
        return INVALID;
    }
    let Some(first) = a[2].checked_mul(stride).and_then(|n| n.checked_add(offset)) else {
        return INVALID;
    };
    let Some(n) = a[3].checked_mul(60) else {
        return INVALID;
    };
    let Some(end) = first.checked_add(n) else {
        return INVALID;
    };
    if n > 1024 * 1024 || end > *length || !initialized.iter().any(|&(a, b)| first >= a && end <= b)
    {
        return INVALID;
    }
    let address = *data + first;
    checked(image, address, n as usize, false);
    let bytes = unsafe { std::slice::from_raw_parts(address as *const u8, n as usize) };
    let Body::Device { gpu, target, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
        unreachable!()
    };
    gpu.draw_transformed(target, bytes)
        .unwrap_or_else(|e| unsupported(&e));
    0
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, a: &[u32; 16]) -> u32 {
    let kind = (id - BASE) / 256;
    let slot = (id - BASE) % 256;
    let mut s = image.cpu.d3d9.borrow_mut();
    let Some(o) = s.objects.get(&a[0]) else {
        unsupported("D3D9 COM this pointer is not a live object");
    };
    if o.refs == 0 || o.body.kind() != kind {
        unsupported("D3D9 COM this/type mismatch or missing external reference");
    }
    eprintln!(
        "PE32 D3D9 COM: kind={kind} slot={slot} this={:x} args={:x?}",
        a[0],
        &a[1..signature(id).unwrap().1 as usize / 4]
    );
    match slot {
        0 => {
            if a[2] == 0 {
                return POINTER;
            }
            write(image, a[2], &[0]);
            if a[1] == 0 {
                return POINTER;
            }
            checked(image, a[1], 16, false);
            let iid = unsafe { (a[1] as *const [u8; 16]).read_unaligned() };
            if iid != IID_UNKNOWN
                && iid != IIDS[kind as usize]
                && !((kind == 2 || kind == 3) && iid == IID_RESOURCE)
            {
                return NO_INTERFACE;
            }
            addref(&mut s, a[0]);
            write(image, a[2], &[a[0]]);
            return 0;
        }
        1 => return addref(&mut s, a[0]),
        2 => return release(&mut s, a[0]),
        _ => (),
    }
    match (kind, slot) {
        (0, 4) => 1,
        (0, 5) => identifier(image, &s, a),
        (0, 16) => device(image, &mut s, a),
        (1, 3) => {
            let Body::Device { gpu, .. } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            match gpu.check_status() {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("D3D9 device status: {e}");
                    0x88760868
                }
            }
        }
        (1, 6) | (2, 3) | (3, 3) => {
            if a[1] == 0 {
                return INVALID;
            }
            let parent = match s.objects[&a[0]].body {
                Body::Device { parent, .. }
                | Body::Surface { parent, .. }
                | Body::VertexBuffer { parent, .. } => parent,
                _ => unreachable!(),
            };
            addref(&mut s, parent);
            write(image, a[1], &[parent]);
            0
        }
        (1, 9) => {
            let Body::Device { hwnd, .. } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            write(image, a[1], &[0, 1, *hwnd, 0x20]);
            0
        }
        (1, 26) => vertex_buffer(image, &mut s, a),
        (1, 89) => {
            if a[1] != 0x44 {
                return UNAVAILABLE;
            }
            let Body::Device { fvf, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
                unreachable!()
            };
            *fvf = a[1];
            0
        }
        (1, 100) => stream_source(&mut s, a),
        (1, 81) => draw(image, &mut s, a),
        (3, 10) => 6,
        (3, 11) => {
            if a[3] == 0 {
                return INVALID;
            }
            write(image, a[3], &[0]);
            if a[4] != 0 {
                return UNAVAILABLE;
            }
            let Body::VertexBuffer {
                data, length, lock, ..
            } = &mut s.objects.get_mut(&a[0]).unwrap().body
            else {
                unreachable!()
            };
            let Some(n) = lock_length(*length, a[1], a[2]) else {
                return INVALID;
            };
            if lock.is_some() {
                return INVALID;
            }
            *lock = Some((a[1], n));
            write(image, a[3], &[*data + a[1]]);
            0
        }
        (3, 12) => {
            let Body::VertexBuffer {
                lock, initialized, ..
            } = &mut s.objects.get_mut(&a[0]).unwrap().body
            else {
                unreachable!()
            };
            let Some((first, n)) = lock.take() else {
                return INVALID;
            };
            initialized.push((first, first + n));
            initialized.sort_unstable();
            let mut joined: Vec<(u32, u32)> = Vec::new();
            for &(a, b) in initialized.iter() {
                if let Some(last) = joined.last_mut()
                    && a <= last.1
                {
                    last.1 = last.1.max(b);
                } else {
                    joined.push((a, b));
                }
            }
            *initialized = joined;
            0
        }
        (3, 13) => {
            let Body::VertexBuffer { length, fvf, .. } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            write(image, a[1], &[100, 6, 0, 0, *length, *fvf]);
            0
        }
        (1, 18) => backbuffer(image, &mut s, a),
        (1, 36) => offscreen(image, &mut s, a),
        (1, 32) => transfer(image, &mut s, a),
        (1, 41) | (1, 42) => {
            let Body::Device { scene, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
                unreachable!()
            };
            let begin = slot == 41;
            if *scene == begin {
                return INVALID;
            }
            *scene = begin;
            0
        }
        (1, 43) => {
            if a[1] != 0 || a[2] != 0 || a[3] != 1 {
                return UNAVAILABLE;
            }
            let Body::Device {
                gpu,
                target,
                parameters,
                ..
            } = &mut s.objects.get_mut(&a[0]).unwrap().body
            else {
                unreachable!()
            };
            let color = if parameters[2] == 22 {
                a[4] | 0xff000000
            } else {
                a[4]
            };
            gpu.clear(target, color).unwrap_or_else(|e| unsupported(&e));
            0
        }
        (1, 17) => {
            if a[1..5].iter().any(|n| *n != 0) {
                return UNAVAILABLE;
            }
            let Body::Device {
                gpu,
                target,
                hwnd,
                scene,
                ..
            } = &s.objects[&a[0]].body
            else {
                unreachable!()
            };
            if *scene {
                return INVALID;
            }
            let mut bytes = gpu.readback(target).unwrap_or_else(|e| unsupported(&e));
            for p in bytes.as_chunks_mut::<4>().0 {
                p[3] = 255;
            }
            let (w, h) = target.dimensions();
            image
                .cpu
                .ui
                .borrow()
                .graphics_present(*hwnd, w, h, &bytes)
                .unwrap_or_else(|e| unsupported(&e));
            0
        }
        (2, 10) => 1,
        (2, 12) => {
            let Body::Surface {
                back,
                width,
                height,
                format,
                ..
            } = &s.objects[&a[0]].body
            else {
                unreachable!()
            };
            write(
                image,
                a[1],
                &[
                    *format,
                    1,
                    u32::from(*back),
                    if *back { 0 } else { 2 },
                    0,
                    0,
                    *width,
                    *height,
                ],
            );
            0
        }
        (2, 13) => {
            if a[1] == 0 || a[2] != 0 || ![0, 16].contains(&a[3]) {
                return INVALID;
            }
            let Body::Surface {
                back,
                data,
                width,
                initialized,
                locked,
                ..
            } = &mut s.objects.get_mut(&a[0]).unwrap().body
            else {
                unreachable!()
            };
            if *back {
                return INVALID;
            }
            if *locked {
                return INVALID;
            }
            if !*initialized {
                unsupported("D3D9 read of uninitialized system-memory surface");
            }
            write(image, a[1], &[*width * 4, *data]);
            *locked = true;
            0
        }
        (2, 14) => {
            let Body::Surface { locked, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
                unreachable!()
            };
            if !*locked {
                return INVALID;
            }
            *locked = false;
            0
        }
        _ => unsupported(&format!(
            "unimplemented D3D9 COM method: interface={kind} slot={slot}; no successful stub returned"
        )),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vertex_lock_ranges_enforce_owned_storage_without_overflow() {
        for (length, offset, size) in [
            (60, 60, 1),
            (60, 61, 0),
            (60, 60, 0),
            (u32::MAX, u32::MAX, 1),
            (60, 4, u32::MAX),
        ] {
            assert_eq!(lock_length(length, offset, size), None);
        }
        assert_eq!(lock_length(60, 0, 0), Some(60));
        assert_eq!(lock_length(60, 4, 0), Some(56));
        assert_eq!(lock_length(60, 40, 20), Some(20));
    }
    #[test]
    fn com_stack_contracts_and_bounds() {
        assert_eq!(signature(BASE + 16), Some(("IDirect3D9::CreateDevice", 28)));
        assert_eq!(
            signature(BASE + 256 + 43),
            Some(("IDirect3DDevice9::Clear", 28))
        );
        assert_eq!(
            signature(BASE + 512 + 13),
            Some(("IDirect3DSurface9::LockRect", 16))
        );
        for id in [
            BASE - 1,
            BASE + 17,
            BASE + 256 + 119,
            BASE + 512 + 17,
            BASE + 768 + 14,
        ] {
            assert!(signature(id).is_none());
        }
    }
}
