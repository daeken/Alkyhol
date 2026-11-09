//! PE32 stdcall COM over owned low vtables. No host pointer is published.
//! Unsupported virtual methods stop before returning; no successful COM stubs.
use super::{Cpu32, LoadedImage32, PAGE, unsupported};
use crate::Result;
use alkyhol_d3d9::{
    Device as Gpu, LegacyDraw, SamplerState, Target, Texture2D, TextureBinding, VertexAttribute,
    VertexFormat,
    compiler::{self, Compiled, Compiler},
    legacy::Stage,
};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
pub(super) const BASE: u32 = 0x9000;
const INVALID: u32 = 0x8876086c;
const UNAVAILABLE: u32 = 0x8876086a;
const NO_INTERFACE: u32 = 0x80004002;
const POINTER: u32 = 0x80004003;
const IID_UNKNOWN: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0xc0, 0, 0, 0, 0, 0, 0, 0x46];
const IID_RESOURCE: [u8; 16] = [
    0x5d, 0xc0, 0xee, 0x05, 0x7d, 0x8f, 0x62, 0x43, 0xb9, 0x99, 0xd1, 0xba, 0xf3, 0x57, 0xc7, 0x04,
];
const IID_BASE_TEXTURE: [u8; 16] = [
    0x7e, 0xa8, 0x0c, 0x58, 0x3c, 0x1d, 0x54, 0x4d, 0x99, 0x1d, 0xb7, 0xd3, 0xe3, 0xc2, 0x98, 0xce,
];
const IIDS: [[u8; 16]; 8] = [
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
    [
        0x7e, 0x55, 0xc5, 0xef, 0x65, 0x62, 0x13, 0x46, 0x8a, 0x94, 0x43, 0x85, 0x78, 0x89, 0xeb,
        0x36,
    ],
    [
        0xdc, 0xdb, 0x3b, 0x6d, 0x02, 0x5b, 0x15, 0x44, 0xb8, 0x52, 0xce, 0x5e, 0x8b, 0xcc, 0xb2,
        0x89,
    ],
    [
        0x9c, 0xc5, 0x13, 0xdd, 0xfa, 0x36, 0x98, 0x40, 0xa8, 0xfb, 0xc7, 0xed, 0x39, 0xdc, 0x85,
        0x46,
    ],
    [
        0x27, 0x12, 0xc3, 0x85, 0xe5, 0x3d, 0x00, 0x4f, 0x9b, 0x3a, 0xf1, 0x1a, 0xc3, 0x8c, 0x18,
        0xb5,
    ],
];
struct Object {
    refs: u32,
    body: Body,
}
struct TextureLevel {
    data: u32,
    width: u32,
    height: u32,
    lock: Option<bool>,
    initialized: bool,
}
enum Body {
    Texture {
        parent: u32,
        gpu: Rc<RefCell<Texture2D>>,
        levels: Vec<TextureLevel>,
        format: u32,
        pool: u32,
        bindings: u32,
    },
    Shader {
        parent: u32,
        stage: Stage,
        code: Vec<u8>,
        compiled: Compiled,
        bindings: u32,
    },
    Declaration {
        parent: u32,
        elements: Vec<[u32; 2]>,
        bindings: u32,
    },
    Factory(Rc<Gpu>, Rc<super::window_host32::DisplayModes>),
    Device {
        parent: u32,
        gpu: Rc<Gpu>,
        target: Box<Target>,
        hwnd: u32,
        scene: bool,
        parameters: [u32; 14],
        creation_flags: u32,
        back: u32,
        fvf: u32,
        stream: Option<(u32, u32, u32)>,
        textures: [u32; 16],
        samplers: Box<[SamplerState; 16]>,
        vertex_shader: u32,
        pixel_shader: u32,
        declaration: u32,
        vertex_constants: Vec<u8>,
        pixel_constants: Vec<u8>,
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
            Self::Factory(..) => 0,
            Self::Device { .. } => 1,
            Self::Surface { .. } => 2,
            Self::VertexBuffer { .. } => 3,
            Self::Shader {
                stage: Stage::Vertex,
                ..
            } => 4,
            Self::Shader {
                stage: Stage::Pixel,
                ..
            } => 5,
            Self::Declaration { .. } => 6,
            Self::Texture { .. } => 7,
        }
    }
}
#[derive(Default)]
pub(super) struct State {
    tables: BTreeMap<u32, u32>,
    objects: BTreeMap<u32, Object>,
    compiler: Option<Compiler>,
    programs: BTreeMap<(u32, u32), Compiled>,
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
            4..=6 => 5,
            7 => 22,
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
        (1, 23) => ("IDirect3DDevice9::CreateTexture", 36),
        (1, 64) => ("IDirect3DDevice9::GetTexture", 12),
        (1, 65) => ("IDirect3DDevice9::SetTexture", 12),
        (1, 68) => ("IDirect3DDevice9::GetSamplerState", 16),
        (1, 69) => ("IDirect3DDevice9::SetSamplerState", 16),
        (7, 3) => ("IDirect3DTexture9::GetDevice", 8),
        (7, 10) => ("IDirect3DTexture9::GetType", 4),
        (7, 13) => ("IDirect3DTexture9::GetLevelCount", 4),
        (7, 17) => ("IDirect3DTexture9::GetLevelDesc", 12),
        (7, 19) => ("IDirect3DTexture9::LockRect", 20),
        (7, 20) => ("IDirect3DTexture9::UnlockRect", 8),
        (1, 86) => ("IDirect3DDevice9::CreateVertexDeclaration", 12),
        (1, 87) => ("IDirect3DDevice9::SetVertexDeclaration", 8),
        (1, 88) => ("IDirect3DDevice9::GetVertexDeclaration", 8),
        (1, 91) => ("IDirect3DDevice9::CreateVertexShader", 12),
        (1, 92) => ("IDirect3DDevice9::SetVertexShader", 8),
        (1, 93) => ("IDirect3DDevice9::GetVertexShader", 8),
        (1, 94) => ("IDirect3DDevice9::SetVertexShaderConstantF", 16),
        (1, 95) => ("IDirect3DDevice9::GetVertexShaderConstantF", 16),
        (1, 106) => ("IDirect3DDevice9::CreatePixelShader", 12),
        (1, 107) => ("IDirect3DDevice9::SetPixelShader", 8),
        (1, 108) => ("IDirect3DDevice9::GetPixelShader", 8),
        (1, 109) => ("IDirect3DDevice9::SetPixelShaderConstantF", 16),
        (1, 110) => ("IDirect3DDevice9::GetPixelShaderConstantF", 16),
        (4..=6, 3) => ("D3D9 shader/declaration GetDevice", 8),
        (4 | 5, 4) => ("D3D9 shader GetFunction", 12),
        (6, 4) => ("D3D9 declaration GetDeclaration", 12),
        (1, 81) => ("IDirect3DDevice9::DrawPrimitive", 16),
        (1, 89) => ("IDirect3DDevice9::SetFVF", 8),
        (1, 100) => ("IDirect3DDevice9::SetStreamSource", 20),
        (3, 3) => ("IDirect3DVertexBuffer9::GetDevice", 8),
        (3, 10) => ("IDirect3DVertexBuffer9::GetType", 4),
        (3, 11) => ("IDirect3DVertexBuffer9::Lock", 20),
        (3, 12) => ("IDirect3DVertexBuffer9::Unlock", 4),
        (3, 13) => ("IDirect3DVertexBuffer9::GetDesc", 8),
        (1, 6) => ("IDirect3DDevice9::GetDirect3D", 8),
        (1, 7) => ("IDirect3DDevice9::GetDeviceCaps", 8),
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
        4..=6 => 5,
        7 => 22,
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
        Body::Factory(..) => None,
        Body::Device { parent, .. }
        | Body::Surface { parent, .. }
        | Body::VertexBuffer { parent, .. }
        | Body::Shader { parent, .. }
        | Body::Declaration { parent, .. }
        | Body::Texture { parent, .. } => Some(*parent),
    };
    let cached = matches!(
        o.body,
        Body::Surface { back: true, .. }
            | Body::VertexBuffer { bindings: 1.., .. }
            | Body::Shader { bindings: 1.., .. }
            | Body::Declaration { bindings: 1.., .. }
            | Body::Texture { bindings: 1.., .. }
    );
    if !cached {
        s.programs.retain(|&(vs, ps), _| vs != p && ps != p);
        let removed = s.objects.remove(&p).unwrap();
        if let Body::Device {
            back,
            stream,
            vertex_shader,
            pixel_shader,
            declaration,
            textures,
            ..
        } = removed.body
        {
            for bound in [vertex_shader, pixel_shader, declaration]
                .into_iter()
                .chain(textures)
            {
                if bound != 0 {
                    unbind(s, bound);
                }
            }
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
    if image.cpu.diagnostic.is_some() {
        eprintln!(
            "PE32 D3D9 guest executable base={:08x}",
            image.cpu.executable.get()
        );
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
    let modes = match super::window_host32::display_modes() {
        Ok(modes) => modes,
        Err(e) => {
            eprintln!("D3D9 native display inventory: {e}");
            return 0;
        }
    };
    eprintln!(
        "PE32 D3D9 mode snapshot: native_display={} count={} current={:?}",
        modes.display,
        modes.modes.len(),
        modes.current
    );
    if std::env::var_os("ALKYHOL_D3D9_SHADER_COMPILER").is_some() {
        let available = (|| {
            let compiler = Compiler::from_environment()?;
            compiler.compile(
                include_bytes!("../../d3d9/tests/fixtures/legacy-model2.vs.bin"),
                Stage::Vertex,
                None,
            )?;
            compiler.compile(
                include_bytes!("../../d3d9/tests/fixtures/legacy-slots128.ps.bin"),
                Stage::Pixel,
                None,
            )?;
            Ok::<_, String>(compiler)
        })();
        match available {
            Ok(compiler) => image.cpu.d3d9.borrow_mut().compiler = Some(compiler),
            Err(e) => eprintln!("D3D9 programmable profile unavailable: {e}"),
        }
    }
    object(
        &image.cpu,
        &mut image.cpu.d3d9.borrow_mut(),
        Body::Factory(Rc::new(gpu), Rc::new(modes)),
        1,
    )
}
fn identifier(image: &LoadedImage32<'_>, s: &State, a: &[u32; 16]) -> u32 {
    if a[1] != 0 || a[2] & !2 != 0 {
        return INVALID;
    }
    let Body::Factory(gpu, _) = &s.objects[&a[0]].body else {
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
/// Only the operations already implemented by this adapter are exposed. In
/// particular, an actual modern GPU is not evidence that our legacy bytecode
/// frontend implements shader model 3, texture stages or arbitrary declarations.
fn capabilities(programmable: bool) -> [u32; 76] {
    let mut caps = [0; 76];
    caps[0] = 1; // HAL
    caps[7] = 0x80000 | 0x400 | 0x40; // real rasterization and TL system-memory vertices
    caps[8] = 0x40; // default CCW cull mode
    caps[14] = 8; // diffuse Gouraud colors, measured by the original tutorial
    caps[45] = 1024 * 1024 / 60; // bounded triangle-list upload
    caps[47] = 1; // stream zero
    caps[48] = 20; // implemented XYZRHW|DIFFUSE stride
    caps[58] = 1; // one adapter in this adapter group
    caps[60] = 1; // one color target
    if programmable {
        // Bounded open SM2 compiler/draw profile, not a generic SM3 promise.
        caps[15] = 2048;
        caps[16] = 2048;
        caps[48] = 2048;
        caps[49] = 0xfffe0200;
        caps[50] = 256;
        caps[51] = 0xffff0200;
        caps[64] = 12; // minimum SM2 vertex temporary registers
        caps[68] = 12; // minimum SM2 pixel temporary registers
        caps[70] = 128; // actual 128-ADD/FRC shader executes in the paired witness
    }
    caps
}
fn device(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[6] == 0 {
        return INVALID;
    }
    write(image, a[6], &[0]);
    if a[1] != 0 || a[2] != 1 || a[4] & !4 != 0x20 || a[5] == 0 {
        return UNAVAILABLE;
    }
    let mut p = read::<14>(image, a[5]);
    eprintln!(
        "D3D9 actual CreateDevice flags={:x} presentation={:x?}",
        a[4], p
    );
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
    let Body::Factory(gpu, _) = &s.objects[&a[0]].body else {
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
            target: Box::new(target),
            hwnd,
            scene: false,
            parameters: p,
            creation_flags: a[4],
            back: 0,
            fvf: 0,
            stream: None,
            textures: [0; 16],
            samplers: Box::new([SamplerState::default(); 16]),
            vertex_shader: 0,
            pixel_shader: 0,
            declaration: 0,
            vertex_constants: vec![0; 256 * 16],
            pixel_constants: vec![0; 224 * 16],
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
    if a[1] == 0
        || a[1] > 1024 * 1024
        || a[2] != 0
        || ![0, 0x44].contains(&a[3])
        || a[4] != 0
        || a[6] != 0
    {
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
    let bindings = match &mut o.body {
        Body::VertexBuffer { bindings, .. }
        | Body::Shader { bindings, .. }
        | Body::Declaration { bindings, .. }
        | Body::Texture { bindings, .. } => bindings,
        _ => unreachable!(),
    };
    *bindings = bindings
        .checked_sub(1)
        .unwrap_or_else(|| unsupported("vertex binding count underflow"));
    if *bindings == 0 && o.refs == 0 {
        s.programs.retain(|&(vs, ps), _| vs != p && ps != p);
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
        if *parent != a[0] || a[3] > *length || a[4] == 0 || a[4] > 2048 {
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
        scene,
        fvf,
        stream,
        vertex_shader,
        pixel_shader,
        declaration,
        ..
    } = &s.objects[&a[0]].body
    else {
        unreachable!()
    };
    if !*scene {
        return INVALID;
    }
    let programmable = (*vertex_shader, *pixel_shader, *declaration);
    let fixed = *fvf == 0x44 && programmable.0 == 0 && programmable.1 == 0;
    if !fixed && (programmable.0 == 0 || programmable.1 == 0 || programmable.2 == 0) {
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
    let Some(n) = a[3].checked_mul(3).and_then(|v| v.checked_mul(stride)) else {
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
    let mut attributes = Vec::new();
    if !fixed {
        let (vs, ps, decl) = programmable;
        if !s.programs.contains_key(&(vs, ps)) {
            let Body::Shader { code: vertex, .. } = &s.objects[&vs].body else {
                unreachable!()
            };
            let Body::Shader { code: pixel, .. } = &s.objects[&ps].body else {
                unreachable!()
            };
            let compiled = s
                .compiler
                .as_ref()
                .unwrap()
                .compile(vertex, Stage::Vertex, Some(pixel))
                .unwrap_or_else(|e| unsupported(&e));
            s.programs.insert((vs, ps), compiled);
        }
        let Body::Declaration { elements, .. } = &s.objects[&decl].body else {
            unreachable!()
        };
        for input in &s.programs[&(vs, ps)].inputs {
            let Some(element) = elements.iter().find(|e| {
                semantic(e[1] >> 16 & 255) == Some(input.semantic.as_str())
                    && e[1] >> 24 == input.index
            }) else {
                return INVALID;
            };
            let offset = (element[0] >> 16) as u64;
            let (format, size) = declaration_format(element[1] & 255).unwrap();
            if offset + size > stride as u64 {
                return INVALID;
            }
            attributes.push(VertexAttribute {
                format,
                offset,
                shader_location: input.register,
            });
        }
    }
    let Body::Device {
        textures, samplers, ..
    } = &s.objects[&a[0]].body
    else {
        unreachable!()
    };
    let texture_handles: Vec<_> = textures
        .iter()
        .enumerate()
        .filter_map(|(slot, &pointer)| {
            if pointer == 0 {
                return None;
            }
            let Body::Texture { gpu, levels, .. } = &s.objects[&pointer].body else {
                unreachable!()
            };
            if levels.iter().any(|l| l.lock.is_some()) {
                unsupported("D3D9 draw using a locked texture");
            }
            Some((slot as u32, gpu.clone(), samplers[slot]))
        })
        .collect();
    let borrowed: Vec<_> = texture_handles
        .iter()
        .map(|(_, gpu, _)| gpu.borrow())
        .collect();
    let textures: Vec<_> = texture_handles
        .iter()
        .zip(&borrowed)
        .map(|((slot, _, sampler), texture)| TextureBinding {
            slot: *slot,
            texture,
            sampler: *sampler,
        })
        .collect();
    let program = s.programs.get(&(programmable.0, programmable.1));
    let pixel_spirv = if fixed {
        Vec::new()
    } else {
        let Body::Shader { compiled, .. } = &s.objects[&programmable.1].body else {
            unreachable!()
        };
        compiled.spirv.clone()
    };
    let Body::Device {
        gpu,
        target,
        vertex_constants,
        pixel_constants,
        ..
    } = &mut s.objects.get_mut(&a[0]).unwrap().body
    else {
        unreachable!()
    };
    if fixed {
        if stride != 20 {
            return INVALID;
        }
        gpu.draw_transformed(target, bytes)
            .unwrap_or_else(|e| unsupported(&e));
    } else {
        gpu.draw_legacy(
            target,
            &LegacyDraw {
                textures: &textures,
                vertex_spirv: &program.unwrap().spirv,
                pixel_spirv: &pixel_spirv,
                vertices: bytes,
                stride: stride as u64,
                attributes: &attributes,
                vertex_constants,
                pixel_constants,
            },
        )
        .unwrap_or_else(|e| unsupported(&e));
    }
    0
}
fn semantic(usage: u32) -> Option<&'static str> {
    Some(match usage {
        0 => "POSITION",
        1 => "BLENDWEIGHT",
        2 => "BLENDINDICES",
        3 => "NORMAL",
        4 => "PSIZE",
        5 => "TEXCOORD",
        6 => "TANGENT",
        7 => "BINORMAL",
        10 => "COLOR",
        11 => "FOG",
        12 => "DEPTH",
        13 => "SAMPLE",
        _ => return None,
    })
}
fn declaration_format(ty: u32) -> Option<(VertexFormat, u64)> {
    Some(match ty {
        0 => (VertexFormat::Float32, 4),
        1 => (VertexFormat::Float32x2, 8),
        2 => (VertexFormat::Float32x3, 12),
        3 => (VertexFormat::Float32x4, 16),
        _ => return None,
    })
}
fn create_shader(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], stage: Stage) -> u32 {
    if a[2] == 0 || a[1] == 0 {
        return INVALID;
    }
    write(image, a[2], &[0]);
    let code = match compiler::bytecode(
        |n| {
            let address = (a[1] as usize)
                .checked_add(n * 4)
                .ok_or("shader pointer overflow")?;
            if !image.permits(address, 4, false) {
                return Err("shader token outside readable storage".into());
            }
            Ok(unsafe { (address as *const u32).read_unaligned() })
        },
        stage,
    ) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("D3D9 bytecode: {e}");
            return INVALID;
        }
    };
    if s.compiler.is_none() {
        match Compiler::from_environment() {
            Ok(c) => s.compiler = Some(c),
            Err(e) => {
                eprintln!("D3D9 compiler configuration: {e}");
                return UNAVAILABLE;
            }
        }
    }
    let compiled = match s.compiler.as_ref().unwrap().compile(&code, stage, None) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("D3D9 shader creation: {e}");
            return INVALID;
        }
    };
    addref(s, a[0]);
    let shader = object(
        &image.cpu,
        s,
        Body::Shader {
            parent: a[0],
            stage,
            code,
            compiled,
            bindings: 0,
        },
        1,
    );
    write(image, a[2], &[shader]);
    0
}
fn create_declaration(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[1] == 0 || a[2] == 0 {
        return INVALID;
    }
    write(image, a[2], &[0]);
    let mut elements = Vec::new();
    for i in 0..65u32 {
        let Some(address) = a[1].checked_add(i * 8) else {
            return INVALID;
        };
        let element = read::<2>(image, address);
        if element == [255, 17] {
            if elements.is_empty() {
                return INVALID;
            }
            elements.push(element);
            addref(s, a[0]);
            let declaration = object(
                &image.cpu,
                s,
                Body::Declaration {
                    parent: a[0],
                    elements,
                    bindings: 0,
                },
                1,
            );
            write(image, a[2], &[declaration]);
            return 0;
        }
        if element[0] & 65535 != 0
            || element[0] >> 16 & 3 != 0
            || element[1] >> 8 & 255 != 0
            || semantic(element[1] >> 16 & 255).is_none()
            || declaration_format(element[1] & 255).is_none()
        {
            return UNAVAILABLE;
        }
        if elements
            .iter()
            .any(|e: &[u32; 2]| e[1] >> 16 == element[1] >> 16)
        {
            return INVALID;
        }
        elements.push(element);
    }
    INVALID
}
fn set_program_binding(s: &mut State, a: &[u32; 16], slot: u32) -> u32 {
    let kind = match slot {
        87 => 6,
        92 => 4,
        107 => 5,
        _ => unreachable!(),
    };
    if a[1] != 0 {
        let Some(o) = s.objects.get(&a[1]) else {
            return INVALID;
        };
        let parent = match o.body {
            Body::Shader { parent, .. }
            | Body::Declaration { parent, .. }
            | Body::Texture { parent, .. } => parent,
            _ => return INVALID,
        };
        if o.refs == 0 || o.body.kind() != kind || parent != a[0] {
            return INVALID;
        }
    }
    let Body::Device {
        declaration,
        vertex_shader,
        pixel_shader,
        ..
    } = &mut s.objects.get_mut(&a[0]).unwrap().body
    else {
        unreachable!()
    };
    let value = match slot {
        87 => declaration,
        92 => vertex_shader,
        107 => pixel_shader,
        _ => unreachable!(),
    };
    let old = *value;
    if old == a[1] {
        return 0;
    }
    *value = a[1];
    if a[1] != 0 {
        let bindings = match &mut s.objects.get_mut(&a[1]).unwrap().body {
            Body::Shader { bindings, .. }
            | Body::Declaration { bindings, .. }
            | Body::Texture { bindings, .. } => bindings,
            _ => unreachable!(),
        };
        *bindings = bindings
            .checked_add(1)
            .unwrap_or_else(|| unsupported("shader binding overflow"));
    }
    if old != 0 {
        unbind(s, old);
    }
    0
}
fn get_program_binding(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], slot: u32) -> u32 {
    if a[1] == 0 {
        return INVALID;
    }
    let Body::Device {
        declaration,
        vertex_shader,
        pixel_shader,
        ..
    } = &s.objects[&a[0]].body
    else {
        unreachable!()
    };
    let value = match slot {
        88 => *declaration,
        93 => *vertex_shader,
        108 => *pixel_shader,
        _ => unreachable!(),
    };
    if value != 0 {
        let o = s.objects.get_mut(&value).unwrap();
        if o.refs == 0 {
            o.refs = 1;
            addref(s, a[0]);
        } else {
            addref(s, value);
        }
    }
    write(image, a[1], &[value]);
    0
}
fn get_function(image: &LoadedImage32<'_>, s: &State, a: &[u32; 16]) -> u32 {
    if a[2] == 0 {
        return INVALID;
    }
    let (bytes, count) = match &s.objects[&a[0]].body {
        Body::Shader { code, .. } => (code.clone(), code.len() as u32),
        Body::Declaration { elements, .. } => (
            elements
                .iter()
                .flatten()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
            elements.len() as u32,
        ),
        _ => unreachable!(),
    };
    if a[1] != 0 {
        let capacity = read::<1>(image, a[2])[0];
        if capacity < count {
            return INVALID;
        }
        checked(image, a[1], bytes.len(), true);
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), a[1] as *mut u8, bytes.len());
        }
    }
    write(image, a[2], &[count]);
    0
}
fn constants(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], slot: u32) -> u32 {
    let Body::Device {
        vertex_constants,
        pixel_constants,
        ..
    } = &mut s.objects.get_mut(&a[0]).unwrap().body
    else {
        unreachable!()
    };
    let buffer = if slot == 94 || slot == 95 {
        vertex_constants
    } else {
        pixel_constants
    };
    let Some(end) = a[1].checked_add(a[3]).and_then(|v| v.checked_mul(16)) else {
        return INVALID;
    };
    if end as usize > buffer.len() || (a[3] != 0 && a[2] == 0) {
        return INVALID;
    }
    if a[3] == 0 {
        return 0;
    }
    let start = a[1] as usize * 16;
    let bytes = &mut buffer[start..end as usize];
    let get = slot == 95 || slot == 110;
    checked(image, a[2], bytes.len(), get);
    unsafe {
        if get {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), a[2] as *mut u8, bytes.len());
        } else {
            std::ptr::copy_nonoverlapping(a[2] as *const u8, bytes.as_mut_ptr(), bytes.len());
        }
    }
    0
}

fn create_texture(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[7] == 0 {
        return INVALID;
    }
    write(image, a[7], &[0]);
    if a[4] != 0 || ![21, 22].contains(&a[5]) || ![0, 1].contains(&a[6]) || a[8] != 0 {
        return UNAVAILABLE;
    }
    let Body::Device { gpu, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let texture = match gpu.create_texture(a[1], a[2], a[3]) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("D3D9 texture allocation: {e}");
            return UNAVAILABLE;
        }
    };
    let mut total = 0usize;
    let mut levels = Vec::new();
    for level in 0..texture.levels() {
        let (width, height) = texture.dimensions(level).unwrap();
        let bytes = width as usize * height as usize * 4;
        total += bytes;
        if total > 64 * 1024 * 1024 {
            return UNAVAILABLE;
        }
        levels.push(TextureLevel {
            data: memory(&image.cpu, &vec![0; bytes], true, false)
                .unwrap_or_else(|e| unsupported(&e)),
            width,
            height,
            lock: None,
            initialized: false,
        });
    }
    addref(s, a[0]);
    let pointer = object(
        &image.cpu,
        s,
        Body::Texture {
            parent: a[0],
            gpu: Rc::new(RefCell::new(texture)),
            levels,
            format: a[5],
            pool: a[6],
            bindings: 0,
        },
        1,
    );
    write(image, a[7], &[pointer]);
    0
}
fn texture_binding(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], get: bool) -> u32 {
    if a[1] >= 16 {
        return UNAVAILABLE;
    }
    let Body::Device { textures, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let old = textures[a[1] as usize];
    if get {
        if a[2] == 0 {
            return INVALID;
        }
        if old != 0 {
            let o = s.objects.get_mut(&old).unwrap();
            if o.refs == 0 {
                o.refs = 1;
                addref(s, a[0]);
            } else {
                addref(s, old);
            }
        }
        write(image, a[2], &[old]);
        return 0;
    }
    if a[2] != 0 {
        let Some(Object {
            refs: 1..,
            body: Body::Texture { parent, .. },
        }) = s.objects.get(&a[2])
        else {
            return INVALID;
        };
        if *parent != a[0] {
            return INVALID;
        }
    }
    if old == a[2] {
        return 0;
    }
    if a[2] != 0 {
        let Body::Texture { bindings, .. } = &mut s.objects.get_mut(&a[2]).unwrap().body else {
            unreachable!()
        };
        *bindings += 1;
    }
    if old != 0 {
        unbind(s, old);
    }
    let Body::Device { textures, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
        unreachable!()
    };
    textures[a[1] as usize] = a[2];
    0
}
fn sampler_state(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], get: bool) -> u32 {
    if a[1] >= 16 {
        return UNAVAILABLE;
    }
    let Body::Device { samplers, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
        unreachable!()
    };
    let state = &mut samplers[a[1] as usize];
    if get {
        if a[3] == 0 {
            return INVALID;
        }
        match state.get(a[2]) {
            Ok(value) => {
                write(image, a[3], &[value]);
                0
            }
            Err(_) => INVALID,
        }
    } else {
        match state.set(a[2], a[3]) {
            Ok(()) => 0,
            Err(_) => UNAVAILABLE,
        }
    }
}
fn texture_operation(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], slot: u32) -> u32 {
    let Body::Texture { parent, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let Body::Device { gpu: device, .. } = &s.objects[parent].body else {
        unreachable!()
    };
    let device = device.clone();
    let Body::Texture {
        gpu,
        levels,
        format,
        pool,
        ..
    } = &mut s.objects.get_mut(&a[0]).unwrap().body
    else {
        unreachable!()
    };
    if slot == 13 {
        return levels.len() as u32;
    }
    let Some(level) = levels.get_mut(a[1] as usize) else {
        return INVALID;
    };
    match slot {
        17 => {
            if a[2] == 0 {
                return INVALID;
            }
            write(
                image,
                a[2],
                &[*format, 1, 0, *pool, 0, 0, level.width, level.height],
            );
            0
        }
        19 => {
            if a[2] == 0 || level.lock.is_some() {
                return INVALID;
            }
            if a[3] != 0 || ![0, 16].contains(&a[4]) {
                return UNAVAILABLE;
            }
            let readonly = a[4] == 16;
            if readonly && !level.initialized {
                unsupported("D3D9 read of uninitialized texture shadow");
            }
            write(image, a[2], &[level.width * 4, level.data]);
            level.lock = Some(readonly);
            0
        }
        20 => {
            let Some(readonly) = level.lock.take() else {
                return INVALID;
            };
            if !readonly {
                let length = level.width as usize * level.height as usize * 4;
                checked(image, level.data, length, false);
                let mut bytes =
                    unsafe { std::slice::from_raw_parts(level.data as *const u8, length) }.to_vec();
                if *format == 22 {
                    for pixel in bytes.as_chunks_mut::<4>().0 {
                        pixel[3] = 255;
                    }
                }
                device
                    .upload_texture(&mut gpu.borrow_mut(), a[1], &bytes)
                    .unwrap_or_else(|e| unsupported(&e));
                level.initialized = true;
            }
            0
        }
        _ => unreachable!(),
    }
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
                && !((kind == 2 || kind == 3 || kind == 7) && iid == IID_RESOURCE)
                && !(kind == 7 && iid == IID_BASE_TEXTURE)
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
        (0, 6) => {
            let Body::Factory(_, modes) = &s.objects[&a[0]].body else {
                unreachable!()
            };
            if a[1] != 0 || a[2] != 22 {
                0
            } else {
                modes.modes.len() as u32
            }
        }
        (0, 7) => {
            let Body::Factory(_, modes) = &s.objects[&a[0]].body else {
                unreachable!()
            };
            if a[1] != 0 || a[2] != 22 || a[4] == 0 {
                return INVALID;
            }
            let Some(mode) = modes.modes.get(a[3] as usize) else {
                return INVALID;
            };
            write(image, a[4], mode);
            0
        }
        (0, 8) => {
            let Body::Factory(_, modes) = &s.objects[&a[0]].body else {
                unreachable!()
            };
            if a[1] != 0 || a[2] == 0 {
                return INVALID;
            }
            write(image, a[2], &modes.current);
            0
        }
        (0, 9) => {
            if a[1] != 0 || a[2] != 1 {
                return UNAVAILABLE;
            }
            if a[5] != 1 || a[3] != 22 || ![0, 21, 22].contains(&a[4]) {
                return UNAVAILABLE;
            }
            0
        }
        (0, 10) => {
            if a[1] != 0 || a[2] != 1 || a[3] != 22 {
                return INVALID;
            }
            // Actual BGRA8 system-memory surfaces and sampled texture/sRGB reads.
            // Depth, render targets, vertex textures and other queries remain unavailable.
            if (a[4] == 0 && a[5] == 1 || a[5] == 3 && a[4] & !0x80000 == 0)
                && [21, 22].contains(&a[6])
            {
                0
            } else {
                UNAVAILABLE
            }
        }
        (0, 11) => {
            if a[1] != 0 || a[2] != 1 {
                return INVALID;
            }
            if ![21, 22].contains(&a[3]) || a[4] != 1 || a[5] != 0 {
                return UNAVAILABLE;
            }
            if a[6] != 0 {
                write(image, a[6], &[1]);
            }
            0
        }
        (0, 12) | (0, 13) => UNAVAILABLE,
        (0, 14) => {
            if a[1] != 0 || a[2] != 1 {
                return UNAVAILABLE;
            }
            if a[3] == 0 {
                return INVALID;
            }
            write(image, a[3], &capabilities(s.compiler.is_some()));
            0
        }
        (0, 16) => device(image, &mut s, a),
        (1, 7) => {
            if a[1] == 0 {
                return INVALID;
            }
            write(image, a[1], &capabilities(s.compiler.is_some()));
            0
        }
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
        (1, 6) | (2..=7, 3) => {
            if a[1] == 0 {
                return INVALID;
            }
            let parent = match s.objects[&a[0]].body {
                Body::Device { parent, .. }
                | Body::Surface { parent, .. }
                | Body::VertexBuffer { parent, .. }
                | Body::Shader { parent, .. }
                | Body::Declaration { parent, .. }
                | Body::Texture { parent, .. } => parent,
                _ => unreachable!(),
            };
            addref(&mut s, parent);
            write(image, a[1], &[parent]);
            0
        }
        (1, 9) => {
            let Body::Device {
                hwnd,
                creation_flags,
                ..
            } = &s.objects[&a[0]].body
            else {
                unreachable!()
            };
            write(image, a[1], &[0, 1, *hwnd, *creation_flags]);
            0
        }
        (1, 23) => create_texture(image, &mut s, a),
        (1, 64 | 65) => texture_binding(image, &mut s, a, slot == 64),
        (1, 68 | 69) => sampler_state(image, &mut s, a, slot == 68),
        (7, 10) => 3,
        (7, 13 | 17 | 19 | 20) => texture_operation(image, &mut s, a, slot),
        (1, 26) => vertex_buffer(image, &mut s, a),
        (1, 86) => create_declaration(image, &mut s, a),
        (1, 91 | 106) => create_shader(
            image,
            &mut s,
            a,
            if slot == 91 {
                Stage::Vertex
            } else {
                Stage::Pixel
            },
        ),
        (1, 87 | 92 | 107) => set_program_binding(&mut s, a, slot),
        (1, 88 | 93 | 108) => get_program_binding(image, &mut s, a, slot),
        (1, 94 | 95 | 109 | 110) => constants(image, &mut s, a, slot),
        (4..=6, 4) => get_function(image, &s, a),
        (1, 89) => {
            if a[1] != 0x44 {
                return UNAVAILABLE;
            }
            let result = set_program_binding(
                &mut s,
                &[a[0], 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                87,
            );
            if result != 0 {
                return result;
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
    fn capability_record_does_not_invent_unimplemented_shader_or_texture_support() {
        let c = capabilities(false);
        assert_eq!(c.len() * 4, 304);
        assert_eq!((c[0], c[47], c[48], c[58], c[60]), (1, 1, 20, 1, 1));
        assert_eq!(c[49], 0);
        assert_eq!(c[51], 0); // shader versions
        for offset in [
            60, 64, 68, 72, 88, 92, 96, 136, 144, 148, 152, 236, 248, 264, 296, 300,
        ] {
            assert_eq!(c[offset / 4], 0);
        }
        assert_eq!(c[7] & 0x100000, 0); // no pure-device promise
        let p = capabilities(true);
        assert_eq!(
            (p[49], p[50], p[51], p[70]),
            (0xfffe0200, 256, 0xffff0200, 128)
        );
        assert_eq!((p[60], p[71], p[74], p[75]), (1, 0, 0, 0)); // no MRT, vertex texture or SM3 claim
    }
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
