//! PE32 stdcall COM over owned low vtables. No host pointer is published.
//! Unsupported virtual methods stop before returning; no successful COM stubs.
#[path = "d3d9_transfers32.rs"]
mod transfers;
#[path = "d3d9_transforms32.rs"]
mod transforms;
#[path = "d3d9_volume32.rs"]
mod volume;
use super::{Cpu32, LoadedImage32, PAGE, unsupported};
use crate::Result;
use alkyhol_d3d9::{
    Depth, Device as Gpu, LegacyDraw, RenderState, SampleFormat, SamplerState, Target, Texture2D,
    TextureBinding, TextureStages, VertexAttribute, VertexFormat,
    compiler::{self, Compiled, Compiler},
    legacy::Stage,
};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use transforms::Transforms;
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
const IIDS: [[u8; 16]; 14] = [
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
    [
        0x5e, 0xd6, 0x9d, 0x7c, 0xf7, 0xd3, 0x29, 0x45, 0xac, 0xee, 0x78, 0x58, 0x30, 0xac, 0xde,
        0x35,
    ],
    [
        0xf2, 0x50, 0x49, 0x79, 0xfc, 0xad, 0x8a, 0x45, 0x90, 0x5e, 0x10, 0xa1, 0x0b, 0x0b, 0x50,
        0x3b,
    ],
    [
        0x81, 0x2f, 0xf3, 0xff, 0x53, 0xd9, 0x3a, 0x47, 0x92, 0x23, 0x93, 0xd6, 0x52, 0xab, 0xa9,
        0x3f,
    ],
    [
        0x60, 0x14, 0x77, 0xd9, 0x95, 0xa6, 0x26, 0x4f, 0xbb, 0xd3, 0x27, 0xb8, 0x40, 0xb5, 0x41,
        0xcc,
    ],
    [
        0x6c, 0x52, 0x18, 0x25, 0x89, 0xe7, 0x11, 0x41, 0xa7, 0xb9, 0x47, 0xef, 0x32, 0x8d, 0x13,
        0xe6,
    ],
    [
        0xe6, 0x16, 0xf4, 0x24, 0x67, 0x1f, 0xa7, 0x4a, 0xb8, 0x8e, 0xd3, 0x3f, 0x6f, 0x31, 0x28,
        0xa1,
    ],
];
struct Object {
    refs: u32,
    body: Body,
}
struct TextureLevel {
    dirty: bool,
    depth: u32,
    data: u32,
    width: u32,
    height: u32,
    pitch: u32,
    length: u32,
    surface: u32,
    lock: Option<bool>,
    initialized: bool,
}
enum Body {
    Volume {
        parent: u32,
        container: u32,
        level: u32,
    },
    Query {
        parent: u32,
        query: alkyhol_d3d9::OcclusionQuery,
    },
    Texture {
        cube: bool,
        volume: bool,
        parent: u32,
        gpu: Rc<RefCell<Texture2D>>,
        levels: Vec<TextureLevel>,
        format: u32,
        pool: u32,
        bindings: u32,
        usage: u32,
        attachment: Option<Target>,
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
    SwapChain {
        parent: u32,
    },
    Factory(Rc<Gpu>, Rc<super::window_host32::DisplayModes>),
    Device {
        parent: u32,
        gpu: Rc<Gpu>,
        target: Box<Target>,
        render_target: u32,
        depth_surface: u32,
        auto_depth: u32,
        swapchain: u32,
        hwnd: u32,
        scene: bool,
        parameters: [u32; 14],
        viewport: [u32; 6],
        transforms: Transforms,
        creation_flags: u32,
        render_state: RenderState,
        texture_stages: TextureStages,
        back: u32,
        fvf: u32,
        streams: Box<[Option<(u32, u32, u32)>; 16]>,
        indices: u32,
        textures: Box<[u32; 16]>,
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
        attachment: Option<Target>,
        depth: Option<Depth>,
        implicit_depth: bool,
        container: u32,
        texture_level: u32,
        bindings: u32,
        data: u32,
        width: u32,
        height: u32,
        format: u32,
        initialized: bool,
        locked: bool,
    },
    IndexBuffer {
        parent: u32,
        data: u32,
        length: u32,
        usage: u32,
        format: u32,
        lock: Option<(u32, u32)>,
        initialized: Vec<(u32, u32)>,
        bindings: u32,
    },
    VertexBuffer {
        parent: u32,
        data: u32,
        length: u32,
        usage: u32,
        fvf: u32,
        lock: Option<(u32, u32)>,
        initialized: Vec<(u32, u32)>,
        bindings: u32,
    },
}
impl Body {
    fn kind(&self) -> u32 {
        match self {
            Self::Query { .. } => 11,
            Self::Volume { .. } => 13,
            Self::Factory(..) => 0,
            Self::Device { .. } => 1,
            Self::Surface { .. } => 2,
            Self::VertexBuffer { .. } => 3,
            Self::IndexBuffer { .. } => 8,
            Self::SwapChain { .. } => 9,
            Self::Shader {
                stage: Stage::Vertex,
                ..
            } => 4,
            Self::Shader {
                stage: Stage::Pixel,
                ..
            } => 5,
            Self::Declaration { .. } => 6,
            Self::Texture { cube, volume, .. } => {
                if *volume {
                    12
                } else if *cube {
                    10
                } else {
                    7
                }
            }
        }
    }
}
#[derive(Default)]
pub(super) struct State {
    tables: BTreeMap<u32, u32>,
    objects: BTreeMap<u32, Object>,
    compiler: Option<Compiler>,
    programs: BTreeMap<(u32, u32), Compiled>,
    headers: Vec<(u32, u32)>,
    buffer_arenas: Vec<(u32, u32)>,
}
// Exact stack sizes for implemented slots. Unknown slots never return to the
// guest, so their unsupported gate only snapshots this and terminates explicitly.
// Documented IDirect3DDevice9 method order and i386 stack extents. These
// metadata entries expose useful diagnostics, not implemented method semantics.
const DEVICE_METHODS: [(&str, u16); 119] = [
    ("IDirect3DDevice9::QueryInterface", 12),
    ("IDirect3DDevice9::AddRef", 4),
    ("IDirect3DDevice9::Release", 4),
    ("IDirect3DDevice9::TestCooperativeLevel", 4),
    ("IDirect3DDevice9::GetAvailableTextureMem", 4),
    ("IDirect3DDevice9::EvictManagedResources", 4),
    ("IDirect3DDevice9::GetDirect3D", 8),
    ("IDirect3DDevice9::GetDeviceCaps", 8),
    ("IDirect3DDevice9::GetDisplayMode", 12),
    ("IDirect3DDevice9::GetCreationParameters", 8),
    ("IDirect3DDevice9::SetCursorProperties", 16),
    ("IDirect3DDevice9::SetCursorPosition", 16),
    ("IDirect3DDevice9::ShowCursor", 8),
    ("IDirect3DDevice9::CreateAdditionalSwapChain", 12),
    ("IDirect3DDevice9::GetSwapChain", 12),
    ("IDirect3DDevice9::GetNumberOfSwapChains", 4),
    ("IDirect3DDevice9::Reset", 8),
    ("IDirect3DDevice9::Present", 20),
    ("IDirect3DDevice9::GetBackBuffer", 20),
    ("IDirect3DDevice9::GetRasterStatus", 12),
    ("IDirect3DDevice9::SetDialogBoxMode", 8),
    ("IDirect3DDevice9::SetGammaRamp", 16),
    ("IDirect3DDevice9::GetGammaRamp", 12),
    ("IDirect3DDevice9::CreateTexture", 36),
    ("IDirect3DDevice9::CreateVolumeTexture", 40),
    ("IDirect3DDevice9::CreateCubeTexture", 32),
    ("IDirect3DDevice9::CreateVertexBuffer", 28),
    ("IDirect3DDevice9::CreateIndexBuffer", 28),
    ("IDirect3DDevice9::CreateRenderTarget", 36),
    ("IDirect3DDevice9::CreateDepthStencilSurface", 36),
    ("IDirect3DDevice9::UpdateSurface", 20),
    ("IDirect3DDevice9::UpdateTexture", 12),
    ("IDirect3DDevice9::GetRenderTargetData", 12),
    ("IDirect3DDevice9::GetFrontBufferData", 12),
    ("IDirect3DDevice9::StretchRect", 24),
    ("IDirect3DDevice9::ColorFill", 16),
    ("IDirect3DDevice9::CreateOffscreenPlainSurface", 28),
    ("IDirect3DDevice9::SetRenderTarget", 12),
    ("IDirect3DDevice9::GetRenderTarget", 12),
    ("IDirect3DDevice9::SetDepthStencilSurface", 8),
    ("IDirect3DDevice9::GetDepthStencilSurface", 8),
    ("IDirect3DDevice9::BeginScene", 4),
    ("IDirect3DDevice9::EndScene", 4),
    ("IDirect3DDevice9::Clear", 28),
    ("IDirect3DDevice9::SetTransform", 12),
    ("IDirect3DDevice9::GetTransform", 12),
    ("IDirect3DDevice9::MultiplyTransform", 12),
    ("IDirect3DDevice9::SetViewport", 8),
    ("IDirect3DDevice9::GetViewport", 8),
    ("IDirect3DDevice9::SetMaterial", 8),
    ("IDirect3DDevice9::GetMaterial", 8),
    ("IDirect3DDevice9::SetLight", 12),
    ("IDirect3DDevice9::GetLight", 12),
    ("IDirect3DDevice9::LightEnable", 12),
    ("IDirect3DDevice9::GetLightEnable", 12),
    ("IDirect3DDevice9::SetClipPlane", 12),
    ("IDirect3DDevice9::GetClipPlane", 12),
    ("IDirect3DDevice9::SetRenderState", 12),
    ("IDirect3DDevice9::GetRenderState", 12),
    ("IDirect3DDevice9::CreateStateBlock", 12),
    ("IDirect3DDevice9::BeginStateBlock", 4),
    ("IDirect3DDevice9::EndStateBlock", 8),
    ("IDirect3DDevice9::SetClipStatus", 8),
    ("IDirect3DDevice9::GetClipStatus", 8),
    ("IDirect3DDevice9::GetTexture", 12),
    ("IDirect3DDevice9::SetTexture", 12),
    ("IDirect3DDevice9::GetTextureStageState", 16),
    ("IDirect3DDevice9::SetTextureStageState", 16),
    ("IDirect3DDevice9::GetSamplerState", 16),
    ("IDirect3DDevice9::SetSamplerState", 16),
    ("IDirect3DDevice9::ValidateDevice", 8),
    ("IDirect3DDevice9::SetPaletteEntries", 12),
    ("IDirect3DDevice9::GetPaletteEntries", 12),
    ("IDirect3DDevice9::SetCurrentTexturePalette", 8),
    ("IDirect3DDevice9::GetCurrentTexturePalette", 8),
    ("IDirect3DDevice9::SetScissorRect", 8),
    ("IDirect3DDevice9::GetScissorRect", 8),
    ("IDirect3DDevice9::SetSoftwareVertexProcessing", 8),
    ("IDirect3DDevice9::GetSoftwareVertexProcessing", 4),
    ("IDirect3DDevice9::SetNPatchMode", 8),
    ("IDirect3DDevice9::GetNPatchMode", 4),
    ("IDirect3DDevice9::DrawPrimitive", 16),
    ("IDirect3DDevice9::DrawIndexedPrimitive", 28),
    ("IDirect3DDevice9::DrawPrimitiveUP", 20),
    ("IDirect3DDevice9::DrawIndexedPrimitiveUP", 36),
    ("IDirect3DDevice9::ProcessVertices", 28),
    ("IDirect3DDevice9::CreateVertexDeclaration", 12),
    ("IDirect3DDevice9::SetVertexDeclaration", 8),
    ("IDirect3DDevice9::GetVertexDeclaration", 8),
    ("IDirect3DDevice9::SetFVF", 8),
    ("IDirect3DDevice9::GetFVF", 8),
    ("IDirect3DDevice9::CreateVertexShader", 12),
    ("IDirect3DDevice9::SetVertexShader", 8),
    ("IDirect3DDevice9::GetVertexShader", 8),
    ("IDirect3DDevice9::SetVertexShaderConstantF", 16),
    ("IDirect3DDevice9::GetVertexShaderConstantF", 16),
    ("IDirect3DDevice9::SetVertexShaderConstantI", 16),
    ("IDirect3DDevice9::GetVertexShaderConstantI", 16),
    ("IDirect3DDevice9::SetVertexShaderConstantB", 16),
    ("IDirect3DDevice9::GetVertexShaderConstantB", 16),
    ("IDirect3DDevice9::SetStreamSource", 20),
    ("IDirect3DDevice9::GetStreamSource", 20),
    ("IDirect3DDevice9::SetStreamSourceFreq", 12),
    ("IDirect3DDevice9::GetStreamSourceFreq", 12),
    ("IDirect3DDevice9::SetIndices", 8),
    ("IDirect3DDevice9::GetIndices", 8),
    ("IDirect3DDevice9::CreatePixelShader", 12),
    ("IDirect3DDevice9::SetPixelShader", 8),
    ("IDirect3DDevice9::GetPixelShader", 8),
    ("IDirect3DDevice9::SetPixelShaderConstantF", 16),
    ("IDirect3DDevice9::GetPixelShaderConstantF", 16),
    ("IDirect3DDevice9::SetPixelShaderConstantI", 16),
    ("IDirect3DDevice9::GetPixelShaderConstantI", 16),
    ("IDirect3DDevice9::SetPixelShaderConstantB", 16),
    ("IDirect3DDevice9::GetPixelShaderConstantB", 16),
    ("IDirect3DDevice9::DrawRectPatch", 16),
    ("IDirect3DDevice9::DrawTriPatch", 16),
    ("IDirect3DDevice9::DeletePatch", 8),
    ("IDirect3DDevice9::CreateQuery", 12),
];
pub(super) fn signature(id: u32) -> Option<(&'static str, u16)> {
    let n = id.checked_sub(BASE)?;
    let kind = n / 256;
    let slot = n % 256;
    if slot
        >= match kind {
            0 => 17,
            1 => 119,
            2 => 17,
            3 | 8 => 14,
            9 => 10,
            11 => 8,
            4..=6 => 5,
            7 | 10 | 12 => 22,
            13 => 11,
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
    if kind == 1 {
        return Some(DEVICE_METHODS[slot as usize]);
    }
    Some(match (kind, slot) {
        (12, 3) => ("IDirect3DVolumeTexture9::GetDevice", 8),
        (12, 10) => ("IDirect3DVolumeTexture9::GetType", 4),
        (12, 13) => ("IDirect3DVolumeTexture9::GetLevelCount", 4),
        (12, 17) => ("IDirect3DVolumeTexture9::GetLevelDesc", 12),
        (12, 18) => ("IDirect3DVolumeTexture9::GetVolumeLevel", 12),
        (12, 19) => ("IDirect3DVolumeTexture9::LockBox", 20),
        (12, 20) => ("IDirect3DVolumeTexture9::UnlockBox", 8),
        (12, 21) => ("IDirect3DVolumeTexture9::AddDirtyBox", 8),
        (13, 3) => ("IDirect3DVolume9::GetDevice", 8),
        (13, 7) => ("IDirect3DVolume9::GetContainer", 12),
        (13, 8) => ("IDirect3DVolume9::GetDesc", 8),
        (13, 9) => ("IDirect3DVolume9::LockBox", 16),
        (13, 10) => ("IDirect3DVolume9::UnlockBox", 4),
        (11, 3) => ("IDirect3DQuery9::GetDevice", 8),
        (11, 4) => ("IDirect3DQuery9::GetType", 4),
        (11, 5) => ("IDirect3DQuery9::GetDataSize", 4),
        (11, 6) => ("IDirect3DQuery9::Issue", 8),
        (11, 7) => ("IDirect3DQuery9::GetData", 16),
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
        (7, 3) => ("IDirect3DTexture9::GetDevice", 8),
        (7, 10) => ("IDirect3DTexture9::GetType", 4),
        (7, 13) => ("IDirect3DTexture9::GetLevelCount", 4),
        (7, 17) => ("IDirect3DTexture9::GetLevelDesc", 12),
        (7, 18) => ("IDirect3DTexture9::GetSurfaceLevel", 12),
        (7, 19) => ("IDirect3DTexture9::LockRect", 20),
        (7, 20) => ("IDirect3DTexture9::UnlockRect", 8),
        (7, 21) => ("IDirect3DTexture9::AddDirtyRect", 8),
        (10, 3) => ("IDirect3DCubeTexture9::GetDevice", 8),
        (10, 10) => ("IDirect3DCubeTexture9::GetType", 4),
        (10, 13) => ("IDirect3DCubeTexture9::GetLevelCount", 4),
        (10, 17) => ("IDirect3DCubeTexture9::GetLevelDesc", 12),
        (10, 18) => ("IDirect3DCubeTexture9::GetCubeMapSurface", 16),
        (10, 19) => ("IDirect3DCubeTexture9::LockRect", 24),
        (10, 20) => ("IDirect3DCubeTexture9::UnlockRect", 12),
        (10, 21) => ("IDirect3DCubeTexture9::AddDirtyRect", 12),
        (4..=6, 3) => ("D3D9 shader/declaration GetDevice", 8),
        (4 | 5, 4) => ("D3D9 shader GetFunction", 12),
        (6, 4) => ("D3D9 declaration GetDeclaration", 12),
        (3, 3) => ("IDirect3DVertexBuffer9::GetDevice", 8),
        (3, 10) => ("IDirect3DVertexBuffer9::GetType", 4),
        (3, 11) => ("IDirect3DVertexBuffer9::Lock", 20),
        (3, 12) => ("IDirect3DVertexBuffer9::Unlock", 4),
        (3, 13) => ("IDirect3DVertexBuffer9::GetDesc", 8),
        (9, 3) => ("IDirect3DSwapChain9::Present", 24),
        (9, 4) => ("IDirect3DSwapChain9::GetFrontBufferData", 8),
        (9, 5) => ("IDirect3DSwapChain9::GetBackBuffer", 16),
        (9, 6) => ("IDirect3DSwapChain9::GetRasterStatus", 8),
        (9, 7) => ("IDirect3DSwapChain9::GetDisplayMode", 8),
        (9, 8) => ("IDirect3DSwapChain9::GetDevice", 8),
        (9, 9) => ("IDirect3DSwapChain9::GetPresentParameters", 8),
        (8, 3) => ("IDirect3DIndexBuffer9::GetDevice", 8),
        (8, 10) => ("IDirect3DIndexBuffer9::GetType", 4),
        (8, 11) => ("IDirect3DIndexBuffer9::Lock", 20),
        (8, 12) => ("IDirect3DIndexBuffer9::Unlock", 4),
        (8, 13) => ("IDirect3DIndexBuffer9::GetDesc", 8),
        (2, 3) => ("IDirect3DSurface9::GetDevice", 8),
        (2, 10) => ("IDirect3DSurface9::GetType", 4),
        (2, 11) => ("IDirect3DSurface9::GetContainer", 12),
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
const STORAGE_ARENA: u32 = 4 * 1024 * 1024;
const MAX_STORAGE_MAPPINGS: usize = 4096;
fn storage_slot(used: u32, length: u32) -> Option<(u32, u32)> {
    if length == 0 {
        return None;
    }
    let start = used.checked_add(15)? & !15;
    let end = start.checked_add(length)?;
    (end <= STORAGE_ARENA).then_some((start, end))
}
pub(super) fn memory(cpu: &Cpu32, bytes: &[u8], writable: bool, executable: bool) -> Result<u32> {
    let length = u32::try_from(bytes.len()).map_err(|_| "D3D9 storage size exceeds PE32 extent")?;
    if length == 0 {
        return Err("D3D9 storage requires nonempty bytes".into());
    }
    // Only RW, nonexecutable allocations share pages. Immutable callback records
    // and executable allocations keep independent OS protections. Slots never
    // move or overlap and are retained until process teardown, like prior maps.
    let packed = writable && !executable && length <= STORAGE_ARENA / 4;
    if packed {
        let mut arenas = cpu.com_arenas.borrow_mut();
        if let Some((base, used)) = arenas.last_mut()
            && let Some((offset, end)) = storage_slot(*used, length)
        {
            let address = base
                .checked_add(offset)
                .ok_or("D3D9 arena address overflow")?;
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), address as *mut u8, bytes.len());
            }
            *used = end;
            return Ok(address);
        }
    }
    if cpu.com_storage.borrow().len() >= MAX_STORAGE_MAPPINGS {
        return Err("D3D9 low-storage mapping budget exhausted (4096 actual mappings)".into());
    }
    let size = if packed {
        STORAGE_ARENA as usize
    } else {
        bytes
            .len()
            .checked_add(PAGE - 1)
            .ok_or("D3D9 storage page extent overflow")?
            / PAGE
            * PAGE
    };
    let m = cpu.mapping(size, PAGE, 0)?;
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), m.base, bytes.len());
    }
    m.protect(
        0,
        m.len,
        1 | if writable { 2 } else { 0 } | if executable { 4 } else { 0 },
    )?;
    let address = m.base as u32;
    cpu.com_storage.borrow_mut().push((m, writable));
    if packed {
        cpu.com_arenas.borrow_mut().push((address, length));
    }
    Ok(address)
}

fn table(cpu: &Cpu32, state: &mut State, kind: u32) -> Result<u32> {
    if let Some(t) = state.tables.get(&kind) {
        return Ok(*t);
    }
    let count = match kind {
        1 => 119,
        3 | 8 => 14,
        9 => 10,
        11 => 8,
        4..=6 => 5,
        7 | 10 | 12 => 22,
        13 => 11,
        _ => 17,
    };
    let m = cpu.mapping(2 * PAGE, PAGE, 0)?;
    let base = m.base as u32;
    let mut bytes = vec![0u8; 2 * PAGE];
    let mut cs = 0u16;
    #[cfg(target_arch = "x86_64")]
    unsafe {
        std::arch::asm!("mov {0:x}, cs",out(reg)cs,options(nomem,nostack,preserves_flags));
    }
    for slot in 0..count {
        let id = BASE + kind * 256 + slot;
        let at = PAGE + slot as usize * 32;
        let address = base + at as u32;
        #[cfg(target_arch = "aarch64")]
        cpu.native_gates
            .borrow_mut()
            .insert(address, (id, signature(id).map_or(0, |s| s.1)));

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
    #[cfg(target_arch = "aarch64")]
    cpu.native_code
        .borrow_mut()
        .push((base + PAGE as u32, base + 2 * PAGE as u32));
    cpu.com_storage.borrow_mut().push((m, false));
    state.tables.insert(kind, base);
    Ok(base)
}
fn object(cpu: &Cpu32, state: &mut State, body: Body, refs: u32) -> u32 {
    if state.objects.len() >= 4096 {
        unsupported("D3D9 object budget exhausted");
    }
    let t = table(cpu, state, body.kind()).unwrap_or_else(|e| unsupported(&e));
    // COM's private object header is writable heap-like storage, distinct from
    // its immutable vtable/gates. Slots never move or get reused while the
    // process lives, including after a last external reference is released.
    if state.headers.last().is_none_or(|&(_, used)| used == 4096) {
        let base = memory(cpu, &vec![0; 65536], true, false).unwrap_or_else(|e| unsupported(&e));
        state.headers.push((base, 0));
    }
    let (base, used) = state.headers.last_mut().unwrap();
    let pointer = *base + *used * 16;
    *used += 1;
    // SAFETY: this fresh 16-byte slot is in the owned writable mapping above.
    unsafe {
        (pointer as *mut u32).write(t);
    }
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
        | Body::IndexBuffer { parent, .. }
        | Body::SwapChain { parent, .. }
        | Body::Shader { parent, .. }
        | Body::Declaration { parent, .. }
        | Body::Texture { parent, .. }
        | Body::Query { parent, .. }
        | Body::Volume { parent, .. } => Some(*parent),
    };
    let cached = matches!(
        o.body,
        Body::Surface { back: true, .. }
            | Body::Volume { .. }
            | Body::SwapChain { .. }
            | Body::Surface {
                implicit_depth: true,
                ..
            }
            | Body::Surface { container: 1.., .. }
            | Body::Surface { bindings: 1.., .. }
            | Body::VertexBuffer { bindings: 1.., .. }
            | Body::IndexBuffer { bindings: 1.., .. }
            | Body::Shader { bindings: 1.., .. }
            | Body::Declaration { bindings: 1.., .. }
            | Body::Texture { bindings: 1.., .. }
    );
    let container = match &o.body {
        Body::Surface {
            container,
            bindings: 0,
            ..
        } => *container,
        Body::Volume { container, .. } => *container,
        _ => 0,
    };
    if !cached {
        s.programs.retain(|&(vs, ps), _| vs != p && ps != p);
        let removed = remove_object(s, p);
        if let Body::Device {
            render_target,
            depth_surface,
            auto_depth,
            swapchain,
            back,
            streams,
            indices,
            vertex_shader,
            pixel_shader,
            declaration,
            textures,
            ..
        } = removed.body
        {
            for bound in [vertex_shader, pixel_shader, declaration]
                .into_iter()
                .chain(*textures)
            {
                if bound != 0 {
                    unbind(s, bound);
                }
            }
            if render_target != 0 {
                unbind(s, render_target);
            }
            if depth_surface != 0 {
                unbind(s, depth_surface);
            }
            if auto_depth != 0 {
                s.objects.remove(&auto_depth);
            }
            if swapchain != 0 {
                s.objects.remove(&swapchain);
            }
            if back != 0 {
                s.objects.remove(&back);
            }
            for (buffer, _, _) in (*streams).into_iter().flatten() {
                unbind(s, buffer);
            }
            if indices != 0 {
                unbind(s, indices);
            }
        }
    }
    if container != 0 {
        unbind(s, container);
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
    caps[5] = 0x80000000; // immediate presentation; no vertical-blank guarantee
    caps[7] = 0x80000 | 0x400 | 0x40; // real rasterization and TL system-memory vertices
    caps[8] = 0x40; // default CCW cull mode
    caps[14] = 8; // diffuse Gouraud colors, measured by the original tutorial
    caps[45] = 1024 * 1024 / 60; // bounded triangle-list upload
    caps[47] = 16; // independently bound vertex streams
    caps[48] = 20; // implemented XYZRHW|DIFFUSE stride
    caps[58] = 1; // one adapter in this adapter group
    caps[34] = 0xff; // Actual eight stencil operations on depth/stencil attachments.
    caps[9] |= 0x10 | 0x06000000; // depth testing and real float constant/slope bias
    caps[10] = 0xff; // all eight real D16 comparisons
    caps[60] = 1; // one color target
    if programmable {
        // Bounded open SM2 compiler/draw profile, not a generic SM3 promise.
        caps[15] = 0xe004; // alpha and mipmapped texture resources
        caps[16] = 0x03030300;
        caps[18] = 0x03030300;
        caps[20] = 7;
        caps[24] = 512; // real point/linear min, mag and mip filtering
        caps[19] = 7; // wrap, mirror and clamp; no border/mirror-once promise
        caps[22] = 2048;
        caps[23] = 2048;
        caps[25] = 2048;
        caps[26] = 2048;
        caps[27] = 1;
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
        || p[9] > 1
        || p[9] != 0 && ![75, 80].contains(&p[10])
        || p[11] != 0
        || p[12] != 0
        || ![0, 0x80000000].contains(&p[13])
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
    let mut initial_state = RenderState::default();
    if p[9] != 0 {
        initial_state.set(7, 1).unwrap();
    }
    let o = object(
        &image.cpu,
        s,
        Body::Device {
            parent: a[0],
            gpu,
            target: Box::new(target),
            render_target: 0,
            depth_surface: 0,
            auto_depth: 0,
            swapchain: 0,
            hwnd,
            scene: false,
            parameters: p,
            viewport: [0, 0, p[0], p[1], 0, 1f32.to_bits()],
            transforms: Transforms::default(),
            creation_flags: a[4],
            render_state: initial_state,
            texture_stages: TextureStages::default(),
            back: 0,
            fvf: 0,
            streams: Box::new([None; 16]),
            indices: 0,
            textures: Box::new([0; 16]),
            samplers: Box::new([SamplerState::default(); 16]),
            vertex_shader: 0,
            pixel_shader: 0,
            declaration: 0,
            vertex_constants: vec![0; 256 * 16],
            pixel_constants: vec![0; 224 * 16],
        },
        1,
    );
    if p[9] != 0 {
        let depth = allocate_depth(image, s, o, p[0], p[1], p[10], true)
            .unwrap_or_else(|e| unsupported(&e));
        bind(s, depth);
        let Body::Device {
            depth_surface,
            auto_depth,
            ..
        } = &mut s.objects.get_mut(&o).unwrap().body
        else {
            unreachable!()
        };
        *depth_surface = depth;
        *auto_depth = depth;
    }
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
                attachment: None,
                depth: None,
                implicit_depth: false,
                container: 0,
                texture_level: 0,
                bindings: 0,
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
    expose(s, b);
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
            attachment: None,
            depth: None,
            implicit_depth: false,
            container: 0,
            texture_level: 0,
            bindings: 0,
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
fn stretch(s: &State, a: &[u32; 16]) -> u32 {
    let Body::Device { gpu, target, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    if a[2] != 0 || a[4] != 0 || a[5] != 0 {
        return UNAVAILABLE;
    }
    let attachment = |pointer| match s.objects.get(&pointer) {
        Some(Object {
            refs: 1..,
            body:
                Body::Surface {
                    parent,
                    back,
                    attachment,
                    format,
                    locked: false,
                    depth: None,
                    ..
                },
        }) if *parent == a[0] && [21, 22].contains(format) => {
            if *back {
                Some(((**target).clone(), *format))
            } else {
                attachment.as_ref().map(|t| (t.clone(), *format))
            }
        }
        _ => None,
    };
    let Some((source, source_format)) = attachment(a[1]) else {
        return INVALID;
    };
    let Some((mut destination, destination_format)) = attachment(a[3]) else {
        return INVALID;
    };
    match gpu.copy_target(
        &source,
        &mut destination,
        source_format == 22 || destination_format == 22,
    ) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("D3D9 StretchRect outside full color-copy profile: {e}");
            UNAVAILABLE
        }
    }
}
fn transfer(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    let Some(Object {
        refs: 1..,
        body:
            Body::Surface {
                parent,
                back,
                attachment,
                width,
                height,
                format,
                ..
            },
    }) = s.objects.get(&a[1])
    else {
        return INVALID;
    };
    if *parent != a[0] {
        return INVALID;
    }
    let (width, height, format) = (*width, *height, *format);
    let Body::Device { gpu, target, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let source = if *back {
        (**target).clone()
    } else if let Some(target) = attachment {
        target.clone()
    } else {
        return INVALID;
    };
    let Some(Object {
        refs: 1..,
        body:
            Body::Surface {
                parent,
                back: false,
                attachment: None,
                depth: None,
                container: 0,
                data,
                width: w,
                height: h,
                format: f,
                locked: false,
                ..
            },
    }) = s.objects.get(&a[2])
    else {
        return INVALID;
    };
    if *parent != a[0] || (width, height, format) != (*w, *h, *f) {
        return INVALID;
    }
    let data = *data;
    let bytes = gpu.readback(&source).unwrap_or_else(|e| unsupported(&e));
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
const BUFFER_ARENA: u32 = 4 * 1024 * 1024;
fn buffer_slot(used: u32, length: u32) -> Option<(u32, u32)> {
    if length == 0 {
        return None;
    }
    let start = used.checked_add(15)? & !15;
    let end = start.checked_add(length)?;
    (end <= BUFFER_ARENA).then_some((start, end))
}
fn buffer_storage(cpu: &Cpu32, s: &mut State, length: u32) -> Result<u32> {
    if length > BUFFER_ARENA {
        return memory(cpu, &vec![0; length as usize], true, false);
    }
    if s.buffer_arenas
        .last()
        .is_none_or(|&(_, used)| buffer_slot(used, length).is_none())
    {
        let base = memory(cpu, &vec![0; BUFFER_ARENA as usize], true, false)?;
        s.buffer_arenas.push((base, 0));
    }
    let (base, used) = s.buffer_arenas.last_mut().unwrap();
    let (offset, end) = buffer_slot(*used, length).ok_or("D3D9 buffer arena extent is invalid")?;
    let pointer = base
        .checked_add(offset)
        .ok_or("D3D9 buffer arena address overflow")?;
    *used = end;
    // Fresh, disjoint slots stay stable for the whole process lifetime. They
    // are never reused, including while only a stream/index binding owns them.
    Ok(pointer)
}
fn vertex_buffer(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[5] == 0 {
        return INVALID;
    }
    write(image, a[5], &[0]);
    if a[1] == 0
        || a[1] > 64 * 1024 * 1024
        || ![0, 8, 0x200, 0x208].contains(&a[2])
        || ![0, 0x44].contains(&a[3])
        || a[4] != 0
        || a[6] != 0
    {
        return UNAVAILABLE;
    }
    let data = buffer_storage(&image.cpu, s, a[1]).unwrap_or_else(|e| unsupported(&e));
    addref(s, a[0]);
    let p = object(
        &image.cpu,
        s,
        Body::VertexBuffer {
            parent: a[0],
            data,
            length: a[1],
            usage: a[2],
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
// Child bindings retain storage without retaining their device, avoiding a
// device -> attachment -> texture -> device reference cycle.
fn bind(s: &mut State, p: u32) {
    let o = s.objects.get_mut(&p).unwrap();
    let external = o.refs;
    let container = match &mut o.body {
        Body::Surface {
            bindings,
            container,
            ..
        } => {
            let first = *bindings == 0 && external == 0;
            *bindings = bindings
                .checked_add(1)
                .unwrap_or_else(|| unsupported("surface binding overflow"));
            if first { *container } else { 0 }
        }
        Body::Texture { bindings, .. } | Body::IndexBuffer { bindings, .. } => {
            *bindings = bindings
                .checked_add(1)
                .unwrap_or_else(|| unsupported("texture binding overflow"));
            0
        }
        _ => unsupported("unsupported internal resource binding"),
    };
    if container != 0 {
        bind(s, container);
    }
}
fn remove_object(s: &mut State, p: u32) -> Object {
    let removed = s.objects.remove(&p).unwrap();
    if let Body::Texture { levels, .. } = &removed.body {
        for level in levels {
            if level.surface != 0 {
                let o = &s.objects[&level.surface];
                if !matches!(
                    o.body,
                    Body::Surface { bindings: 0, .. } | Body::Volume { .. }
                ) || o.refs != 0
                {
                    unsupported("live surface lost its texture storage");
                }
                s.objects.remove(&level.surface);
            }
        }
    }
    removed
}
fn unbind(s: &mut State, p: u32) {
    let Some(o) = s.objects.get_mut(&p) else {
        unsupported("missing bound resource");
    };
    let refs = o.refs;
    let (bindings, container, cached) = match &mut o.body {
        Body::VertexBuffer { bindings, .. }
        | Body::IndexBuffer { bindings, .. }
        | Body::Shader { bindings, .. }
        | Body::Declaration { bindings, .. }
        | Body::Texture { bindings, .. } => (bindings, 0, false),
        Body::Surface {
            bindings,
            container,
            back,
            implicit_depth,
            ..
        } => (
            bindings,
            *container,
            *back || *implicit_depth || *container != 0,
        ),
        _ => unreachable!(),
    };
    *bindings = bindings
        .checked_sub(1)
        .unwrap_or_else(|| unsupported("resource binding count underflow"));
    if *bindings == 0 && refs == 0 {
        if container != 0 {
            unbind(s, container);
        } else if !cached {
            s.programs.retain(|&(vs, ps), _| vs != p && ps != p);
            remove_object(s, p);
        }
    }
}
fn expose(s: &mut State, p: u32) {
    let o = s.objects.get_mut(&p).unwrap();
    if o.refs != 0 {
        addref(s, p);
        return;
    }
    let (parent, container) = match &o.body {
        Body::Surface {
            parent,
            container,
            bindings,
            ..
        } => (*parent, if *bindings == 0 { *container } else { 0 }),
        Body::Volume {
            parent, container, ..
        } => (*parent, *container),
        Body::Texture { parent, .. }
        | Body::IndexBuffer { parent, .. }
        | Body::SwapChain { parent, .. } => (*parent, 0),
        _ => unsupported("unsupported retained interface exposure"),
    };
    o.refs = 1;
    addref(s, parent);
    if container != 0 {
        bind(s, container);
    }
}
fn color_target(s: &State, p: u32) -> (Target, u32) {
    let Body::Device {
        target,
        render_target,
        parameters,
        ..
    } = &s.objects[&p].body
    else {
        unreachable!()
    };
    if *render_target == 0 {
        return ((**target).clone(), parameters[2]);
    }
    let Body::Surface {
        attachment: Some(target),
        format,
        ..
    } = &s.objects[render_target].body
    else {
        unreachable!()
    };
    (target.clone(), *format)
}
fn render_binding(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], get: bool) -> u32 {
    if a[1] != 0 {
        return UNAVAILABLE;
    }
    if a[2] == 0 {
        return INVALID;
    }
    if get {
        let Body::Device { render_target, .. } = &s.objects[&a[0]].body else {
            unreachable!()
        };
        let pointer = *render_target;
        if pointer == 0 {
            let mut arguments = [0; 16];
            arguments[0] = a[0];
            arguments[4] = a[2];
            return backbuffer(image, s, &arguments);
        }
        expose(s, pointer);
        write(image, a[2], &[pointer]);
        return 0;
    }
    let Some(Object {
        refs: 1..,
        body:
            Body::Surface {
                parent,
                back,
                attachment,
                locked: false,
                ..
            },
    }) = s.objects.get(&a[2])
    else {
        return INVALID;
    };
    if *parent != a[0] || (!*back && attachment.is_none()) {
        return INVALID;
    }
    let next = if *back { 0 } else { a[2] };
    let Body::Device { render_target, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let previous = *render_target;
    if previous != next {
        if next != 0 {
            bind(s, next);
        }
        let Body::Device { render_target, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
            unreachable!()
        };
        *render_target = next;
        if previous != 0 {
            unbind(s, previous);
        }
    }
    let (target, _) = color_target(s, a[0]);
    let (width, height) = target.dimensions();
    let Body::Device { viewport, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
        unreachable!()
    };
    *viewport = [0, 0, width, height, 0, 1f32.to_bits()];
    0
}
fn texture_surface(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[2] == 0 {
        return INVALID;
    }
    write(image, a[2], &[0]);
    let Body::Texture {
        parent,
        attachment,
        levels,
        format,
        ..
    } = &s.objects[&a[0]].body
    else {
        unreachable!()
    };
    let Some(level) = levels.get(a[1] as usize) else {
        return INVALID;
    };
    let (parent, format, mut pointer, width, height, data) = (
        *parent,
        *format,
        level.surface,
        level.width,
        level.height,
        level.data,
    );
    let attachment = attachment.clone();
    if pointer == 0 {
        pointer = object(
            &image.cpu,
            s,
            Body::Surface {
                parent,
                back: false,
                attachment,
                depth: None,
                implicit_depth: false,
                container: a[0],
                texture_level: a[1],
                bindings: 0,
                data,
                width,
                height,
                format,
                initialized: false,
                locked: false,
            },
            0,
        );
        let Body::Texture { levels, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
            unreachable!()
        };
        levels[a[1] as usize].surface = pointer;
    }
    expose(s, pointer);
    write(image, a[2], &[pointer]);
    0
}
fn depth_target(s: &State, device: u32) -> Option<Depth> {
    let Body::Device { depth_surface, .. } = &s.objects[&device].body else {
        unreachable!()
    };
    if *depth_surface == 0 {
        return None;
    }
    let Body::Surface { depth, .. } = &s.objects[depth_surface].body else {
        unreachable!()
    };
    depth.clone()
}
fn allocate_depth(
    image: &LoadedImage32<'_>,
    s: &mut State,
    parent: u32,
    width: u32,
    height: u32,
    format: u32,
    implicit: bool,
) -> Result<u32> {
    let Body::Device { gpu, .. } = &s.objects[&parent].body else {
        unreachable!()
    };
    let depth = if format == 75 {
        let depth = gpu.create_depth_stencil(width, height)?;
        // D3D9 exposes no defined initial stencil value. This adapter chooses
        // zero by actually clearing new GPU storage, not by declaring unknown
        // contents initialized or silently removing the stencil operation.
        gpu.clear_depth_stencil(&depth, None, Some(0))?;
        depth
    } else {
        gpu.create_depth16(width, height)?
    };
    if !implicit {
        addref(s, parent);
    }
    Ok(object(
        &image.cpu,
        s,
        Body::Surface {
            parent,
            back: false,
            attachment: None,
            depth: Some(depth),
            implicit_depth: implicit,
            container: 0,
            texture_level: 0,
            bindings: 0,
            data: 0,
            width,
            height,
            format,
            initialized: false,
            locked: false,
        },
        if implicit { 0 } else { 1 },
    ))
}
fn create_depth(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[7] == 0 {
        return INVALID;
    }
    write(image, a[7], &[0]);
    if ![75, 80].contains(&a[3]) || a[4] != 0 || a[5] != 0 || a[6] > 1 || a[8] != 0 {
        return UNAVAILABLE;
    }
    match allocate_depth(image, s, a[0], a[1], a[2], a[3], false) {
        Ok(p) => {
            write(image, a[7], &[p]);
            0
        }
        Err(e) => {
            eprintln!("D3D9 depth allocation: {e}");
            UNAVAILABLE
        }
    }
}
fn depth_binding(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], get: bool) -> u32 {
    let Body::Device { depth_surface, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let old = *depth_surface;
    if get {
        if a[1] == 0 {
            return INVALID;
        }
        write(image, a[1], &[0]);
        if old == 0 {
            return 0x88760866;
        }
        expose(s, old);
        write(image, a[1], &[old]);
        return 0;
    }
    if a[1] != 0 {
        let Some(Object {
            refs: 1..,
            body:
                Body::Surface {
                    parent,
                    depth: Some(_),
                    ..
                },
        }) = s.objects.get(&a[1])
        else {
            return INVALID;
        };
        if *parent != a[0] {
            return INVALID;
        }
    }
    if old != a[1] {
        if a[1] != 0 {
            bind(s, a[1]);
        }
        let Body::Device { depth_surface, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
            unreachable!()
        };
        *depth_surface = a[1];
        if old != 0 {
            unbind(s, old);
        }
    }
    0
}
fn index_buffer(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[5] == 0 {
        return INVALID;
    }
    write(image, a[5], &[0]);
    if a[1] == 0
        || a[1] > 64 * 1024 * 1024
        || ![0, 8, 0x200, 0x208].contains(&a[2])
        || ![101, 102].contains(&a[3])
        || a[4] != 0
        || a[6] != 0
    {
        return UNAVAILABLE;
    }
    let data = buffer_storage(&image.cpu, s, a[1]).unwrap_or_else(|e| unsupported(&e));
    addref(s, a[0]);
    let p = object(
        &image.cpu,
        s,
        Body::IndexBuffer {
            parent: a[0],
            data,
            length: a[1],
            usage: a[2],
            format: a[3],
            lock: None,
            initialized: Vec::new(),
            bindings: 0,
        },
        1,
    );
    write(image, a[5], &[p]);
    0
}
fn index_binding(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], get: bool) -> u32 {
    let Body::Device { indices, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let old = *indices;
    if get {
        if a[1] == 0 {
            return INVALID;
        }
        if old != 0 {
            expose(s, old);
        }
        write(image, a[1], &[old]);
        return 0;
    }
    if a[1] != 0 {
        let Some(Object {
            refs: 1..,
            body: Body::IndexBuffer { parent, .. },
        }) = s.objects.get(&a[1])
        else {
            return INVALID;
        };
        if *parent != a[0] {
            return INVALID;
        }
    }
    if old != a[1] {
        if a[1] != 0 {
            bind(s, a[1]);
        }
        let Body::Device { indices, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
            unreachable!()
        };
        *indices = a[1];
        if old != 0 {
            unbind(s, old);
        }
    }
    0
}
fn buffer_lock(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], unlock: bool) -> u32 {
    let (data, length, usage, lock, initialized) = match &mut s.objects.get_mut(&a[0]).unwrap().body
    {
        Body::VertexBuffer {
            data,
            length,
            usage,
            lock,
            initialized,
            ..
        }
        | Body::IndexBuffer {
            data,
            length,
            usage,
            lock,
            initialized,
            ..
        } => (*data, *length, *usage, lock, initialized),
        _ => unreachable!(),
    };
    if unlock {
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
        return 0;
    }
    if a[3] == 0 {
        return INVALID;
    }
    write(image, a[3], &[0]);
    if a[4] & !0x3810 != 0 {
        return UNAVAILABLE;
    }
    let readonly = a[4] & 16 != 0;
    let discard = a[4] & 0x2000 != 0;
    let nooverwrite = a[4] & 0x1000 != 0;
    if lock.is_some()
        || readonly && (usage & 8 != 0 || discard || nooverwrite)
        || (discard || nooverwrite) && usage & 0x200 == 0
        || discard && nooverwrite
    {
        return INVALID;
    }
    let Some(n) = lock_length(length, a[1], a[2]) else {
        return INVALID;
    };
    if readonly
        && !initialized
            .iter()
            .any(|&(first, last)| a[1] >= first && a[1] + n <= last)
    {
        unsupported("D3D9 READONLY lock of uninitialized buffer storage");
    }
    if discard {
        initialized.clear();
    }
    *lock = Some((a[1], n));
    write(image, a[3], &[data + a[1]]);
    0
}
fn primitive_vertices(kind: u32, count: u32) -> Option<u32> {
    if count == 0 || count > 1024 * 1024 {
        return None;
    }
    match kind {
        4 => count.checked_mul(3),
        5 => count.checked_add(2),
        _ => None,
    }
}
fn strip_triangles(source: &[u32]) -> Option<Vec<u32>> {
    if source.len() < 3 || source.len() - 2 > 1024 * 1024 {
        return None;
    }
    let mut result = Vec::with_capacity((source.len() - 2) * 3);
    for i in 0..source.len() - 2 {
        result.extend(if i % 2 == 0 {
            [source[i], source[i + 1], source[i + 2]]
        } else {
            [source[i], source[i + 2], source[i + 1]]
        });
    }
    Some(result)
}
fn draw_indexed(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if !matches!(a[1], 4 | 5) || a[4] == 0 || a[6] == 0 {
        return UNAVAILABLE;
    }
    let Body::Device { indices, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    if *indices == 0 {
        return INVALID;
    }
    let Body::IndexBuffer {
        data,
        length,
        format,
        lock,
        initialized,
        ..
    } = &s.objects[indices].body
    else {
        unreachable!()
    };
    if lock.is_some() {
        return INVALID;
    }
    let width = if *format == 101 { 2 } else { 4 };
    let Some(first) = a[5].checked_mul(width) else {
        return INVALID;
    };
    let Some(n) = primitive_vertices(a[1], a[6]).and_then(|v| v.checked_mul(width)) else {
        return INVALID;
    };
    let Some(end) = first.checked_add(n) else {
        return INVALID;
    };
    if n > 12 * 1024 * 1024
        || end > *length
        || !initialized.iter().any(|&(a, b)| first >= a && end <= b)
    {
        return INVALID;
    }
    let Some(last) = a[3].checked_add(a[4]) else {
        return INVALID;
    };
    let base = a[2] as i32 as i64 + a[3] as i64;
    if !(0..=u32::MAX as i64).contains(&base) {
        return INVALID;
    }
    let address = *data + first;
    checked(image, address, n as usize, false);
    let bytes = unsafe { std::slice::from_raw_parts(address as *const u8, n as usize) };
    let values: Vec<u32> = if width == 2 {
        bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u16::from_le_bytes(*b) as u32)
            .collect()
    } else {
        bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| u32::from_le_bytes(*b))
            .collect()
    };
    if values.iter().any(|&v| v < a[3] || v >= last) {
        return INVALID;
    }
    let normalized: Vec<_> = values.into_iter().map(|v| v - a[3]).collect();
    let normalized = if a[1] == 5 {
        let Some(values) = strip_triangles(&normalized) else {
            return INVALID;
        };
        values
    } else {
        normalized
    };
    let mut args = [0; 16];
    args[0] = a[0];
    args[1] = a[1];
    args[2] = base as u32;
    args[3] = a[6];
    draw_buffer(image, s, &args, Some((a[4], normalized)), None)
}
// Indices address the original stream-zero base; MinVertexIndex restricts the
// accessible range, rather than rebasing the caller's pointer implicitly.
fn user_indices(bytes: &[u8], format: u32, minimum: u32, count: u32) -> Option<Vec<u32>> {
    let width = match format {
        101 => 2,
        102 => 4,
        _ => return None,
    };
    let end = minimum.checked_add(count)?;
    if count == 0 || bytes.is_empty() || !bytes.len().is_multiple_of(width) {
        return None;
    }
    bytes
        .chunks_exact(width)
        .map(|b| {
            let value = if width == 2 {
                u16::from_le_bytes(b.try_into().unwrap()) as u32
            } else {
                u32::from_le_bytes(b.try_into().unwrap())
            };
            (minimum..end).contains(&value).then(|| value - minimum)
        })
        .collect()
}
fn draw_user_inner(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], indexed: bool) -> u32 {
    let (minimum, count, primitives, pointer, stride) = if indexed {
        (a[2], a[3], a[4], a[7], a[8])
    } else {
        let Some(count) = primitive_vertices(a[1], a[2]) else {
            return UNAVAILABLE;
        };
        (0, count, a[2], a[3], a[4])
    };
    let Some(index_count) = primitive_vertices(a[1], primitives) else {
        return UNAVAILABLE;
    };
    if pointer == 0 || count == 0 {
        return INVALID;
    }
    let Some((first, size)) = vertex_span(u32::MAX, 0, stride, minimum, count) else {
        return INVALID;
    };
    let Some(address) = pointer
        .checked_add(first)
        .filter(|p| p.checked_add(size).is_some())
    else {
        return INVALID;
    };
    checked(image, address, size as usize, false);
    let Body::Device { declaration, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    if let Some(Object {
        body: Body::Declaration { elements, .. },
        ..
    }) = s.objects.get(declaration)
        && elements
            .iter()
            .any(|e| e[1] & 255 != 17 && e[0] & 65535 != 0)
    {
        return INVALID;
    }
    let indices = if indexed {
        let width = match a[6] {
            101 => 2,
            102 => 4,
            _ => return INVALID,
        };
        let Some(length) = index_count
            .checked_mul(width)
            .filter(|&n| n <= 12 * 1024 * 1024)
        else {
            return INVALID;
        };
        if a[5] == 0 {
            return INVALID;
        }
        checked(image, a[5], length as usize, false);
        let bytes = unsafe { std::slice::from_raw_parts(a[5] as *const u8, length as usize) };
        let Some(values) = user_indices(bytes, a[6], minimum, count) else {
            return INVALID;
        };
        let values = if a[1] == 5 {
            strip_triangles(&values).unwrap()
        } else {
            values
        };
        Some((count, values))
    } else {
        None
    };
    let mut args = [0; 16];
    args[0] = a[0];
    args[1] = a[1];
    args[3] = primitives;
    draw_buffer(image, s, &args, indices, Some((address, size, stride)))
}
fn draw_user(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], indexed: bool) -> u32 {
    let result = draw_user_inner(image, s, a, indexed);
    // D3D9 UP calls consume user memory synchronously and clear these bindings.
    // Releasing the retained binding also honors COM resource lifetimes.
    let mut args = [0; 16];
    args[0] = a[0];
    stream_source(s, &args);
    if indexed {
        index_binding(image, s, &args, false);
    }
    result
}
fn swapchain(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16]) -> u32 {
    if a[2] == 0 {
        return INVALID;
    }
    write(image, a[2], &[0]);
    if a[1] != 0 {
        return INVALID;
    }
    let Body::Device { swapchain, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let mut pointer = *swapchain;
    if pointer == 0 {
        pointer = object(&image.cpu, s, Body::SwapChain { parent: a[0] }, 0);
        let Body::Device { swapchain, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
            unreachable!()
        };
        *swapchain = pointer;
    }
    expose(s, pointer);
    write(image, a[2], &[pointer]);
    0
}
fn device_mode(image: &LoadedImage32<'_>, s: &State, device: u32, out: u32) -> u32 {
    if out == 0 {
        return INVALID;
    }
    let Body::Device { parent, .. } = &s.objects[&device].body else {
        unreachable!()
    };
    let Body::Factory(_, modes) = &s.objects[parent].body else {
        unreachable!()
    };
    let mode = modes.current;
    write(image, out, &mode);
    0
}
fn stream_source(s: &mut State, a: &[u32; 16]) -> u32 {
    if a[1] >= 16 {
        return INVALID;
    }
    if a[2] == 0 && (a[3] != 0 || a[4] != 0) {
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
    let Body::Device { streams, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let old = streams[a[1] as usize];
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
    let Body::Device { streams, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body else {
        unreachable!()
    };
    streams[a[1] as usize] = if a[2] == 0 {
        None
    } else {
        Some((a[2], a[3], a[4]))
    };
    0
}
// Validate every source independently: offsets and strides belong to bindings,
// while the vertex/index selection is shared by all declaration streams.
fn vertex_span(
    length: u32,
    offset: u32,
    stride: u32,
    start: u32,
    count: u32,
) -> Option<(u32, u32)> {
    if stride == 0 || stride > 2048 || count == 0 {
        return None;
    }
    let first = start.checked_mul(stride)?.checked_add(offset)?;
    let size = count.checked_mul(stride)?;
    let end = first.checked_add(size)?;
    (size <= 64 * 1024 * 1024 && end <= length).then_some((first, size))
}
fn vertex_stream(
    image: &LoadedImage32<'_>,
    s: &State,
    binding: Option<(u32, u32, u32)>,
    start: u32,
    count: u32,
    indices: Option<&[u32]>,
) -> Option<(u32, u32, u32)> {
    let (pointer, offset, stride) = binding?;
    let Body::VertexBuffer {
        data,
        length,
        lock: None,
        initialized,
        ..
    } = &s.objects.get(&pointer)?.body
    else {
        return None;
    };
    let (first, size) = vertex_span(*length, offset, stride, start, count)?;
    let written = |at: u32, end: u32| initialized.iter().any(|&(a, b)| at >= a && end <= b);
    if let Some(indices) = indices {
        for &index in indices {
            if index >= count {
                return None;
            }
            let at = first + index * stride;
            if !written(at, at + stride) {
                return None;
            }
        }
    } else if !written(first, first + size) {
        return None;
    }
    let address = data.checked_add(first)?;
    checked(image, address, size as usize, false);
    Some((address, size, stride))
}
// This finite profile consumes only position/depth/query effects. Color is
// disabled, so no lighting/material/texture output is substituted.
fn xyz_visibility_state(state: &RenderState) -> bool {
    [(168, 0), (15, 0), (151, 0), (167, 0)]
        .into_iter()
        .all(|(key, value)| state.get(key) == Ok(value))
}
fn draw_buffer(
    image: &LoadedImage32<'_>,
    s: &mut State,
    a: &[u32; 16],
    indexed: Option<(u32, Vec<u32>)>,
    user: Option<(u32, u32, u32)>,
) -> u32 {
    if !matches!(a[1], 4 | 5) || a[3] == 0 {
        return UNAVAILABLE;
    }
    let Body::Device {
        scene,
        texture_stages,
        textures,
        fvf,
        streams,
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
    let pretransformed = s.objects.get(&programmable.2).is_some_and(|object| matches!(&object.body, Body::Declaration { elements, .. } if elements.iter().any(|e| e[1] >> 16 & 255 == 9)));
    let program_key = (
        if pretransformed { 0 } else { programmable.0 },
        programmable.1,
    );
    let fixed = *fvf == 0x44 && programmable.0 == 0 && programmable.1 == 0;
    let xyz = !fixed && !pretransformed && programmable.0 == 0 && programmable.1 == 0;
    if xyz {
        let Body::Device { render_state, .. } = &s.objects[&a[0]].body else {
            unreachable!()
        };
        if !xyz_visibility_state(render_state) {
            return UNAVAILABLE;
        }
        let Some(Object {
            body: Body::Declaration { elements, .. },
            ..
        }) = s.objects.get(&programmable.2)
        else {
            return INVALID;
        };
        if elements.iter().filter(|e| e[1] & 255 != 17).count() != 1
            || !elements.iter().any(|e| e[1] == 2)
        {
            return UNAVAILABLE;
        }
    }
    if fixed {
        texture_stages
            .validate_transformed_color(textures[0] != 0)
            .unwrap_or_else(|e| unsupported(&e));
    }
    if !fixed
        && !xyz
        && (programmable.0 == 0 && !pretransformed || programmable.1 == 0 || programmable.2 == 0)
    {
        return INVALID;
    }
    let streams = **streams;
    let vertices = if let Some((count, _)) = &indexed {
        Some(*count)
    } else {
        primitive_vertices(a[1], a[3])
    };
    let strip = if a[1] == 5 && indexed.is_none() {
        let Some(count) = vertices else {
            return INVALID;
        };
        let Some(indices) = strip_triangles(&(0..count).collect::<Vec<_>>()) else {
            return INVALID;
        };
        Some(indices)
    } else {
        None
    };
    let Some(vertex_count) = vertices else {
        return INVALID;
    };
    let referenced = indexed.as_ref().map(|(_, indices)| indices.as_slice());
    let fixed_source = if fixed {
        let Some(source) =
            user.or_else(|| vertex_stream(image, s, streams[0], a[2], vertex_count, referenced))
        else {
            return INVALID;
        };
        Some(source)
    } else {
        None
    };
    let mut attributes = Vec::new();
    let mut fetch = Vec::new();
    if !fixed {
        let (vs, ps, decl) = programmable;
        if !s.programs.contains_key(&program_key) {
            let compiled = if xyz {
                alkyhol_d3d9::fixed_xyz::shaders()
                    .unwrap_or_else(|e| unsupported(&e))
                    .0
                    .clone()
            } else if pretransformed {
                let Body::Shader { compiled, .. } = &s.objects[&ps].body else {
                    unreachable!()
                };
                alkyhol_d3d9::fixed_vertex::vertex_shader(&compiled.inputs)
                    .unwrap_or_else(|e| unsupported(&e))
            } else {
                let Body::Shader { code: vertex, .. } = &s.objects[&vs].body else {
                    unreachable!()
                };
                let Body::Shader { code: pixel, .. } = &s.objects[&ps].body else {
                    unreachable!()
                };
                s.compiler
                    .as_ref()
                    .unwrap()
                    .compile(vertex, Stage::Vertex, Some(pixel))
                    .unwrap_or_else(|e| unsupported(&e))
            };
            s.programs.insert(program_key, compiled);
        }
        let Body::Declaration { elements, .. } = &s.objects[&decl].body else {
            unreachable!()
        };
        for input in &s.programs[&program_key].inputs {
            let Some(element) = elements.iter().find(|e| {
                semantic(e[1] >> 16 & 255) == Some(input.semantic.as_str())
                    && e[1] >> 24 == input.index
            }) else {
                return INVALID;
            };
            let offset = (element[0] >> 16) as u64;
            let ty = element[1] & 255;
            let size = alkyhol_d3d9::vertex::element_size(ty).unwrap() as u64;
            let stream = (element[0] & 65535) as usize;
            let Some((address, length, stride)) = (if user.is_some() {
                if stream == 0 { user } else { None }
            } else {
                vertex_stream(image, s, streams[stream], a[2], vertex_count, referenced)
            }) else {
                return INVALID;
            };
            if offset + size > u64::from(stride) {
                return INVALID;
            }
            let bytes =
                unsafe { std::slice::from_raw_parts(address as *const u8, length as usize) };
            fetch.push((bytes, stride as usize, offset as usize, ty));
            attributes.push(VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: (attributes.len() * 16) as u64,
                shader_location: input.register,
            });
        }
    }
    // Canonical float4 input registers implement D3D9 fetch rules without
    // modifying guest storage, source shaders, geometry or source index values.
    let packed_stride = fetch.len() * 16;
    let mut packed = Vec::new();
    if !fixed {
        let size = (vertices.unwrap() as usize)
            .checked_mul(packed_stride)
            .filter(|&size| size != 0 && size <= 64 * 1024 * 1024);
        let Some(size) = size else {
            return INVALID;
        };
        packed.resize(size, 0);
        let mut convert = |index: usize| {
            for (component, &(bytes, stride, offset, ty)) in fetch.iter().enumerate() {
                let start = index * stride + offset;
                let count = alkyhol_d3d9::vertex::element_size(ty).unwrap();
                let value = alkyhol_d3d9::vertex::expand(ty, &bytes[start..start + count]).unwrap();
                let dest = index * packed_stride + component * 16;
                packed[dest..dest + 16].copy_from_slice(&value);
            }
        };
        if let Some((_, indices)) = &indexed {
            for &index in indices {
                convert(index as usize);
            }
        } else {
            for index in 0..vertices.unwrap() as usize {
                convert(index);
            }
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
            if pointer == 0 || xyz {
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
    let (mut active, _) = color_target(s, a[0]);
    let depth = depth_target(s, a[0]);
    let Body::Device { viewport, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let viewport_words = *viewport;
    let (width, height) = active.dimensions();
    let viewport = alkyhol_d3d9::viewport_from_words(viewport_words, width, height)
        .unwrap_or_else(|e| unsupported(&e));
    if pretransformed && viewport_words[4..] != [0, 1f32.to_bits()] {
        // POSITIONT Z has already undergone the Windows viewport transform.
        // Nondefault depth unprojection is not implemented by this adapter.
        return UNAVAILABLE;
    }
    let fixed_constants: Vec<_> = [viewport[2], viewport[3], viewport[0], viewport[1]]
        .iter()
        .flat_map(|n| n.to_le_bytes())
        .collect();
    let xyz_constants: Vec<u8> = if xyz {
        let Body::Device { transforms, .. } = &s.objects[&a[0]].body else {
            unreachable!()
        };
        [256, 2, 3]
            .into_iter()
            .flat_map(|key| transforms.get(key).unwrap())
            .flat_map(u32::to_le_bytes)
            .collect()
    } else {
        Vec::new()
    };
    let program = s.programs.get(&program_key);
    let pixel_spirv = if fixed {
        Vec::new()
    } else if xyz {
        alkyhol_d3d9::fixed_xyz::shaders()
            .unwrap_or_else(|e| unsupported(&e))
            .1
            .spirv
            .clone()
    } else {
        let Body::Shader { compiled, .. } = &s.objects[&programmable.1].body else {
            unreachable!()
        };
        compiled.spirv.clone()
    };
    let Body::Device {
        gpu,
        vertex_constants,
        pixel_constants,
        render_state,
        ..
    } = &mut s.objects.get_mut(&a[0]).unwrap().body
    else {
        unreachable!()
    };
    if fixed {
        let (address, length, stride) = fixed_source.unwrap();
        let bytes = unsafe { std::slice::from_raw_parts(address as *const u8, length as usize) };
        if stride != 20 {
            return INVALID;
        }
        if viewport_words != [0, 0, width, height, 0, 1f32.to_bits()] {
            return UNAVAILABLE;
        }
        let expanded;
        let bytes = if let Some(indices) = indexed
            .as_ref()
            .map(|(_, indices)| indices)
            .or(strip.as_ref())
        {
            expanded = indices
                .iter()
                .flat_map(|&index| {
                    bytes[index as usize * 20..(index as usize + 1) * 20]
                        .iter()
                        .copied()
                })
                .collect::<Vec<_>>();
            expanded.as_slice()
        } else {
            bytes
        };
        gpu.draw_transformed_depth_state(&mut active, bytes, render_state, depth.as_ref())
            .unwrap_or_else(|e| unsupported(&e));
    } else {
        gpu.draw_legacy_viewport_depth_state(
            &mut active,
            &LegacyDraw {
                indices: indexed
                    .as_ref()
                    .map(|(_, indices)| indices.as_slice())
                    .or(strip.as_deref()),
                textures: if xyz { &[] } else { &textures },
                vertex_spirv: &program.unwrap().spirv,
                pixel_spirv: &pixel_spirv,
                vertices: &packed,
                stride: packed_stride as u64,
                attributes: &attributes,
                vertex_constants: if xyz {
                    &xyz_constants
                } else if pretransformed {
                    &fixed_constants
                } else {
                    vertex_constants
                },
                pixel_constants,
            },
            render_state,
            depth.as_ref(),
            viewport,
        )
        .unwrap_or_else(|e| unsupported(&e));
    }
    0
}
// Opt-in completed-Present throughput, without GPU readbacks beyond the existing
// presentation path. Windows/game state is not changed by this diagnostic.
fn profile_present(gpu: &Gpu, image: &LoadedImage32<'_>) {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*ENABLED.get_or_init(|| std::env::var_os("ALKYHOL_D3D9_PROFILE").is_some()) {
        return;
    }
    std::thread_local! {
        static PERIOD: std::cell::RefCell<Option<(std::time::Instant,u64)>> = const { std::cell::RefCell::new(None) };
    }
    PERIOD.with(|slot| {
        let mut slot=slot.borrow_mut();
        let (start,frames)=slot.get_or_insert_with(|| (std::time::Instant::now(),0));
        *frames+=1;
        let seconds=start.elapsed().as_secs_f64();
        if seconds>=5. {
            let stats=gpu.legacy_pipeline_cache_stats();
            let uploads=gpu.legacy_upload_stats();
            super::dsound32::report(image);
            let (service_seconds, service_calls) = super::perf32::take();
            eprintln!("PE32 D3D9 service timing [Win32,draw,other-D3D9,CPU-handoff]: seconds={service_seconds:?} calls={service_calls:?}");
            eprintln!("PE32 D3D9 completed Present profile: frames={frames} seconds={seconds:.6} fps={:.3} pipelines={stats:?} uploads={uploads:?}",*frames as f64/seconds);
            *start=std::time::Instant::now();*frames=0;
        }
    });
}
fn capture_present(width: u32, height: u32, bytes: &[u8]) {
    use std::io::Write;
    static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let Some(directory) = std::env::var_os("ALKYHOL_D3D9_CAPTURE_DIRECTORY") else {
        return;
    };
    let serial = SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    if ![1, 5, 30, 60, 120].contains(&serial) {
        return;
    }
    let result = (|| -> std::io::Result<_> {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory)?;
        let path = directory.join(format!("present-{}-{serial:06}.ppm", std::process::id()));
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        write!(output, "P6\n{width} {height}\n255\n")?;
        let rgb: Vec<_> = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| [p[2], p[1], p[0]])
            .collect();
        output.write_all(&rgb)?;
        output.sync_all()?;
        Ok(path)
    })();
    match result {
        Ok(path) => eprintln!(
            "PE32 actual D3D9 present capture: {} ({}x{}, serial={serial})",
            path.display(),
            width,
            height
        ),
        Err(error) => eprintln!("PE32 present capture failed independently of rendering: {error}"),
    }
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
        9 => "POSITIONT",
        10 => "COLOR",
        11 => "FOG",
        12 => "DEPTH",
        13 => "SAMPLE",
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
            if elements.iter().any(|e: &[u32; 2]| e[1] >> 16 & 255 == 9)
                && elements.iter().any(|e| e[1] >> 16 & 255 == 0)
            {
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
        if element[0] & 65535 >= 16
            || element[0] >> 16 & 3 != 0
            || element[1] >> 8 & 255 != 0
            || semantic(element[1] >> 16 & 255).is_none()
            || element[1] >> 16 & 255 == 9 && (element[1] & 255 != 3 || element[1] >> 24 != 0)
            || alkyhol_d3d9::vertex::element_size(element[1] & 255).is_err()
        {
            eprintln!("D3D9 unavailable declaration element: {element:x?}");
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
            | Body::Texture { parent, .. }
            | Body::Query { parent, .. }
            | Body::Volume { parent, .. } => parent,
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

fn sample_format(format: u32) -> Option<SampleFormat> {
    match format {
        21 | 22 => Some(SampleFormat::Bgra8),
        50 => Some(SampleFormat::Luminance8),
        28 => Some(SampleFormat::Alpha8),
        26 => Some(SampleFormat::Argb4444),
        0x31545844 => Some(SampleFormat::Bc1),
        0x33545844 => Some(SampleFormat::Bc2),
        0x35545844 => Some(SampleFormat::Bc3),
        _ => None,
    }
}
fn cube_subresource(face: u32, level: u32, mips: u32) -> Option<u32> {
    if face >= 6 || level >= mips {
        return None;
    }
    face.checked_mul(mips)?.checked_add(level)
}
#[derive(Debug)]
struct CpuMip {
    offset: u32,
    width: u32,
    height: u32,
    pitch: u32,
    length: u32,
}
fn texture_cpu_layout(
    width: u32,
    height: u32,
    mips: u32,
    faces: u32,
    format: SampleFormat,
) -> Option<(Vec<CpuMip>, u32)> {
    if width == 0
        || height == 0
        || width > 8192
        || height > 8192
        || !matches!(faces, 1 | 6)
        || faces == 6 && width != height
        || mips == 0
        || mips > 32 - width.max(height).leading_zeros()
    {
        return None;
    }
    let mut levels = Vec::new();
    let mut total = 0u32;
    for _ in 0..faces {
        for level in 0..mips {
            let (w, h) = ((width >> level).max(1), (height >> level).max(1));
            let (pitch, rows) = format.block_layout(w, h).ok()?;
            let length = pitch.checked_mul(rows)?;
            let offset = total;
            total = total.checked_add(length)?.checked_add(15)? & !15;
            if total > 64 * 1024 * 1024 {
                return None;
            }
            levels.push(CpuMip {
                offset,
                width: w,
                height: h,
                pitch,
                length,
            });
        }
    }
    Some((levels, total))
}
fn create_texture(image: &LoadedImage32<'_>, s: &mut State, a: &[u32; 16], cube: bool) -> u32 {
    let mut normalized = *a;
    if cube {
        normalized[2] = a[1];
        normalized[3..9].copy_from_slice(&a[2..8]);
    }
    let a = &normalized;
    if a[7] == 0 {
        return INVALID;
    }
    write(image, a[7], &[0]);
    let Some(format) = sample_format(a[5]) else {
        return UNAVAILABLE;
    };
    if a[4] > 1
        || a[4] == 1 && format != SampleFormat::Bgra8
        || ![0, 1, 2].contains(&a[6])
        || a[8] != 0
        || a[4] == 1 && (a[3] != 1 || a[6] != 0)
        || cube && a[4] != 0
    {
        return UNAVAILABLE;
    }
    let Body::Device { gpu, .. } = &s.objects[&a[0]].body else {
        unreachable!()
    };
    let maximum = 32 - a[1].max(a[2]).leading_zeros();
    let mips = if a[3] == 0 { maximum } else { a[3] };
    if a[1] == 0 || a[2] == 0 || mips > maximum {
        return INVALID;
    }
    let faces = if cube { 6 } else { 1 };
    let Some((layout, total)) = texture_cpu_layout(a[1], a[2], mips, faces, format) else {
        return UNAVAILABLE;
    };
    let allocation = if cube {
        gpu.create_cube_texture(a[1], a[3], format)
            .map(|texture| (texture, None))
    } else if a[4] == 1 {
        gpu.create_render_texture(a[1], a[2])
            .and_then(|(texture, mut target)| {
                // D3D9 permits drawing to newly allocated targets without a
                // guest Clear. Establish real GPU storage, rather than bypass
                // the backend's initialized-content guards. Initial zero is
                // this implementation's policy, not a native-byte assertion.
                gpu.clear(&mut target, 0)?;
                Ok((texture, Some(target)))
            })
    } else {
        gpu.create_sampled_texture(a[1], a[2], a[3], format)
            .map(|texture| (texture, None))
    };
    let (texture, attachment) = match allocation {
        Ok(v) => v,
        Err(e) => {
            eprintln!("D3D9 texture allocation: {e}");
            return UNAVAILABLE;
        }
    };
    // One checked mapping per texture. Each face/mip still has independent,
    // aligned storage and initialization/lock state; no geometry or mip is omitted.
    let data = if a[4] == 1 {
        0
    } else {
        memory(&image.cpu, &vec![0; total as usize], true, false)
            .unwrap_or_else(|e| unsupported(&e))
    };
    let levels = layout
        .into_iter()
        .map(|mip| TextureLevel {
            dirty: true,
            depth: 1,
            data: if data == 0 { 0 } else { data + mip.offset },
            width: mip.width,
            height: mip.height,
            pitch: mip.pitch,
            length: mip.length,
            surface: 0,
            lock: None,
            initialized: false,
        })
        .collect();
    addref(s, a[0]);
    let pointer = object(
        &image.cpu,
        s,
        Body::Texture {
            cube,
            volume: false,
            parent: a[0],
            gpu: Rc::new(RefCell::new(texture)),
            levels,
            format: a[5],
            pool: a[6],
            bindings: 0,
            usage: a[4],
            attachment,
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
            body: Body::Texture { parent, pool, .. },
        }) = s.objects.get(&a[2])
        else {
            return INVALID;
        };
        if *parent != a[0] || *pool == 2 {
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
fn lock_rectangle_offset(
    format: u32,
    width: u32,
    height: u32,
    pitch: u32,
    r: [u32; 4],
) -> Option<u32> {
    if r[0] >= r[2] || r[1] >= r[3] || r[2] > width || r[3] > height {
        return None;
    }
    if r == [0, 0, width, height] {
        return Some(0);
    }
    let bytes = match format {
        21 | 22 => 4,
        26 => 2,
        28 | 50 => 1,
        _ => return None,
    };
    r[1].checked_mul(pitch)?
        .checked_add(r[0].checked_mul(bytes)?)
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
        usage,
        ..
    } = &mut s.objects.get_mut(&a[0]).unwrap().body
    else {
        unreachable!()
    };
    if slot == 13 {
        return gpu.borrow().levels();
    }
    let mips = gpu.borrow().levels();
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
                &[*format, 1, *usage, *pool, 0, 0, level.width, level.height],
            );
            0
        }
        19 => {
            if *usage != 0 {
                return INVALID;
            }
            if a[2] == 0 || level.lock.is_some() {
                return INVALID;
            }
            if a[4] & !0x8810 != 0 {
                return UNAVAILABLE;
            }
            let rectangle = if a[3] == 0 {
                [0, 0, level.width, level.height]
            } else {
                read::<4>(image, a[3])
            };
            let Some(offset) =
                lock_rectangle_offset(*format, level.width, level.height, level.pitch, rectangle)
            else {
                return UNAVAILABLE;
            };
            let readonly = a[4] & 16 != 0;
            if a[1].is_multiple_of(mips) && !readonly && a[4] & 0x8000 == 0 {
                level.dirty = true;
            }
            if readonly && !level.initialized && *format != 50 {
                unsupported("D3D9 read of uninitialized texture shadow");
            }
            // A fresh managed L8 mip already owns real CPU storage. READONLY
            // may expose it, but its initial contents are not a Windows-value
            // promise. This neither uploads zero pixels nor initializes the GPU;
            // sampling still requires an actual writable unlock.
            write(image, a[2], &[level.pitch, level.data + offset]);
            level.lock = Some(readonly);
            0
        }
        20 => {
            let Some(readonly) = level.lock.take() else {
                return INVALID;
            };
            // Managed textures start wholly dirty. A first read-only unlock
            // does not erase that initial dirty state: upload actual owned CPU
            // bytes, without promising their undefined initial Windows values.
            if !readonly || *format == 50 && *pool == 1 && !level.initialized {
                let length = level.length as usize;
                checked(image, level.data, length, false);
                let mut bytes =
                    unsafe { std::slice::from_raw_parts(level.data as *const u8, length) }.to_vec();
                if *format == 22 {
                    for pixel in bytes.as_chunks_mut::<4>().0 {
                        pixel[3] = 255;
                    }
                }
                if *pool == 2 {
                    level.initialized = true;
                    return 0;
                }
                let mut texture = gpu.borrow_mut();
                let mips = texture.levels();
                let upload = if texture.is_cube() {
                    device.upload_cube_face(&mut texture, a[1] / mips, a[1] % mips, &bytes)
                } else {
                    device.upload_texture(&mut texture, a[1], &bytes)
                };
                upload.unwrap_or_else(|e| unsupported(&e));
                level.initialized = true;
            }
            0
        }
        _ => unreachable!(),
    }
}

pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, a: &[u32; 16]) -> u32 {
    let result = dispatch_inner(image, id, a);
    if (result as i32) < 0 {
        eprintln!(
            "PE32 D3D9 failure: {} -> {result:08x}",
            signature(id).unwrap().0
        );
    }
    result
}
fn dispatch_inner(image: &LoadedImage32<'_>, id: u32, a: &[u32; 16]) -> u32 {
    let kind = (id - BASE) / 256;
    let slot = (id - BASE) % 256;
    let mut s = image.cpu.d3d9.borrow_mut();
    let Some(o) = s.objects.get(&a[0]) else {
        unsupported("D3D9 COM this pointer is not a live object");
    };
    if o.refs == 0 || o.body.kind() != kind {
        unsupported("D3D9 COM this/type mismatch or missing external reference");
    }
    if !image.cpu.trace_first_calls {
        eprintln!(
            "PE32 D3D9 COM: kind={kind} slot={slot} this={:x} args={:x?}",
            a[0],
            &a[1..signature(id).unwrap().1 as usize / 4]
        );
    }
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
                && !(matches!(kind, 2 | 3 | 7 | 8 | 10 | 12) && iid == IID_RESOURCE)
                && !(matches!(kind, 7 | 10 | 12) && iid == IID_BASE_TEXTURE)
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
        (1, 21) => {
            if a[1] != 0 || a[2] > 1 {
                unsupported(
                    "SetGammaRamp supports only implicit swap chain zero and documented flags",
                );
            }
            checked(image, a[3], 3 * 256 * 2, false);
            // D3D9 specifies no effect and no error for a presentation mode
            // without gamma-ramp support. Caps2 advertises neither gamma bit.
            // This is a void method: no HRESULT or successful display change.
            assert_eq!(capabilities(s.compiler.is_some())[3], 0);
            eprintln!(
                "PE32 D3D9 SetGammaRamp: gamma support is not advertised; documented no effect, no display change"
            );
            0
        }
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
            if a[5] == 3
                && a[4] & !0x80000 == 0
                && sample_format(a[6]).is_some_and(SampleFormat::compressed)
            {
                let Body::Factory(gpu, _) = &s.objects[&a[0]].body else {
                    unreachable!()
                };
                return if gpu.supports_bc() { 0 } else { UNAVAILABLE };
            }
            if a[4] == 0 && a[5] == 4 && [21, 22, 50].contains(&a[6]) {
                return 0;
            }
            // Linear L8 luminance textures are real storage; render targets,
            // vertex-texture and sRGB queries for L8 remain unavailable.
            if a[4] == 0 && a[5] == 3 && [26, 28, 50].contains(&a[6]) {
                return 0;
            }
            // Real BGRA8 color/sample storage and non-lockable D16 surfaces.
            if a[4] == 2 && a[5] == 1 && [75, 80].contains(&a[6]) {
                return 0;
            }
            // Vertex textures and other formats remain unavailable.
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
            if ![21, 22, 75, 80].contains(&a[3]) || a[4] != 1 || a[5] != 0 {
                return UNAVAILABLE;
            }
            if a[6] != 0 {
                write(image, a[6], &[1]);
            }
            0
        }
        (0, 12) => {
            if a[1] == 0
                && a[2] == 1
                && a[3] == 22
                && [21, 22].contains(&a[4])
                && [75, 80].contains(&a[5])
            {
                0
            } else {
                UNAVAILABLE
            }
        }
        (0, 13) => UNAVAILABLE,
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
        (1, 6) | (2..=8 | 10..=13, 3) => {
            if a[1] == 0 {
                return INVALID;
            }
            let parent = match s.objects[&a[0]].body {
                Body::Device { parent, .. }
                | Body::Surface { parent, .. }
                | Body::VertexBuffer { parent, .. }
                | Body::IndexBuffer { parent, .. }
                | Body::SwapChain { parent, .. }
                | Body::Shader { parent, .. }
                | Body::Declaration { parent, .. }
                | Body::Texture { parent, .. }
                | Body::Query { parent, .. }
                | Body::Volume { parent, .. } => parent,
                _ => unreachable!(),
            };
            addref(&mut s, parent);
            write(image, a[1], &[parent]);
            0
        }
        (1, 118) => {
            if a[2] != 0 {
                write(image, a[2], &[0]);
            }
            if a[1] != 9 {
                return UNAVAILABLE;
            }
            if a[2] == 0 {
                return 0;
            }
            let Body::Device { gpu, .. } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            let query = gpu.create_occlusion_query();
            addref(&mut s, a[0]);
            let pointer = object(
                &image.cpu,
                &mut s,
                Body::Query {
                    parent: a[0],
                    query,
                },
                1,
            );
            write(image, a[2], &[pointer]);
            0
        }
        (11, 4) => 9,
        (11, 5) => 4,
        (11, 6 | 7) => {
            let Body::Query { parent, query } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            let Body::Device { gpu, .. } = &s.objects[parent].body else {
                unreachable!()
            };
            if slot == 6 {
                match a[1] {
                    2 => gpu
                        .begin_occlusion_query(query)
                        .unwrap_or_else(|e| unsupported(&e)),
                    1 => gpu
                        .end_occlusion_query(query)
                        .unwrap_or_else(|e| unsupported(&e)),
                    _ => return INVALID,
                }
                0
            } else {
                if a[3] > 1 || ![0, 4].contains(&a[2]) || a[1] == 0 && a[2] != 0 {
                    return INVALID;
                }
                if a[2] == 4 {
                    checked(image, a[1], 4, true);
                }
                let Some(value) = gpu
                    .occlusion_data(query, a[3] != 0)
                    .unwrap_or_else(|e| unsupported(&e))
                else {
                    return 1;
                };
                if a[2] == 4 {
                    write(image, a[1], &[value]);
                }
                0
            }
        }
        (1, 14) => swapchain(image, &mut s, a),
        (1, 15) => 1,
        (1, 8) => {
            if a[1] == 0 {
                device_mode(image, &s, a[0], a[2])
            } else {
                INVALID
            }
        }
        (9, 3) => {
            if a[5] != 0 {
                return UNAVAILABLE;
            }
            let Body::SwapChain { parent } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            let mut args = *a;
            args[0] = *parent;
            drop(s);
            dispatch_inner(image, BASE + 256 + 17, &args)
        }
        (9, 5) => {
            let Body::SwapChain { parent } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            let mut args = [0; 16];
            args[0] = *parent;
            args[2] = a[1];
            args[3] = a[2];
            args[4] = a[3];
            backbuffer(image, &mut s, &args)
        }
        (9, 7) => {
            let Body::SwapChain { parent } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            device_mode(image, &s, *parent, a[1])
        }
        (9, 8) => {
            if a[1] == 0 {
                return INVALID;
            }
            let Body::SwapChain { parent } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            let parent = *parent;
            addref(&mut s, parent);
            write(image, a[1], &[parent]);
            0
        }
        (9, 9) => {
            if a[1] == 0 {
                return INVALID;
            }
            let Body::SwapChain { parent } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            let Body::Device { parameters, .. } = &s.objects[parent].body else {
                unreachable!()
            };
            write(image, a[1], parameters);
            0
        }
        (9, 4 | 6) => UNAVAILABLE,
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
        (1, 66 | 67) => {
            let Body::Device { texture_stages, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body
            else {
                unreachable!()
            };
            if slot == 67 {
                texture_stages
                    .set(a[1], a[2], a[3])
                    .unwrap_or_else(|e| unsupported(&e));
            } else {
                if a[3] == 0 {
                    return INVALID;
                }
                let value = texture_stages
                    .get(a[1], a[2])
                    .unwrap_or_else(|e| unsupported(&e));
                write(image, a[3], &[value]);
            }
            0
        }
        (1, 57 | 58) => {
            let Body::Device { render_state, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body
            else {
                unreachable!()
            };
            if slot == 57 {
                render_state
                    .set(a[1], a[2])
                    .unwrap_or_else(|e| unsupported(&e));
            } else {
                if a[2] == 0 {
                    return INVALID;
                }
                let value = render_state.get(a[1]).unwrap_or_else(|e| unsupported(&e));
                write(image, a[2], &[value]);
            }
            0
        }
        (1, 28) => transfers::create_render_target(image, &mut s, a),
        (1, 30) => transfers::update_surface(image, &mut s, a),
        (1, 31) => transfers::update_texture(image, &mut s, a),
        (1, 35) => transfers::color_fill(image, &mut s, a),
        (7 | 10 | 12, 21) => transfers::add_dirty(image, &mut s, a, kind),
        (1, 24) => volume::create(image, &mut s, a),
        (12, 10) => 4,
        (12, 13 | 17..=20) | (13, 7..=10) => volume::operation(image, &mut s, a, kind, slot),
        (1, 23 | 25) => create_texture(image, &mut s, a, slot == 25),
        (1, 64 | 65) => texture_binding(image, &mut s, a, slot == 64),
        (1, 68 | 69) => sampler_state(image, &mut s, a, slot == 68),
        (7, 10) => 3,
        (10, 10) => 5,
        (10, 13 | 17) => {
            let Body::Texture { gpu, .. } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            if slot == 17 && a[1] >= gpu.borrow().levels() {
                return INVALID;
            }
            texture_operation(image, &mut s, a, slot)
        }
        (10, 18..=20) => {
            let Body::Texture { gpu, .. } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            let mips = gpu.borrow().levels();
            let Some(subresource) = cube_subresource(a[1], a[2], mips) else {
                return INVALID;
            };
            let mut normalized = *a;
            normalized[1] = subresource;
            normalized[2..5].copy_from_slice(&a[3..6]);
            if slot == 18 {
                texture_surface(image, &mut s, &normalized)
            } else {
                texture_operation(image, &mut s, &normalized, slot)
            }
        }
        (7, 18) => texture_surface(image, &mut s, a),
        (1, 37 | 38) => render_binding(image, &mut s, a, slot == 38),
        (7, 13 | 17 | 19 | 20) => texture_operation(image, &mut s, a, slot),
        (1, 26) => vertex_buffer(image, &mut s, a),
        (1, 27) => index_buffer(image, &mut s, a),
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
        (1, 101) => {
            if a[1] >= 16 || [a[2], a[3], a[4]].contains(&0) {
                return INVALID;
            }
            let Body::Device { streams, .. } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            let (buffer, offset, stride) = streams[a[1] as usize].unwrap_or((0, 0, 0));
            if buffer != 0 {
                let o = s.objects.get_mut(&buffer).unwrap();
                if o.refs == 0 {
                    o.refs = 1;
                    addref(&mut s, a[0]);
                } else {
                    addref(&mut s, buffer);
                }
            }
            for (pointer, value) in [(a[2], buffer), (a[3], offset), (a[4], stride)] {
                write(image, pointer, &[value]);
            }
            0
        }
        (1, 102) => {
            if a[1] >= 16 {
                INVALID
            } else if a[2] == 1 {
                0
            } else {
                eprintln!(
                    "D3D9 unsupported instanced/divided stream frequency {}",
                    a[2]
                );
                UNAVAILABLE
            }
        }
        (1, 103) => {
            if a[1] >= 16 || a[2] == 0 {
                return INVALID;
            }
            write(image, a[2], &[1]);
            0
        }
        (1, 104 | 105) => index_binding(image, &mut s, a, slot == 105),
        (1, 81) => draw_buffer(image, &mut s, a, None, None),
        (1, 82) => draw_indexed(image, &mut s, a),
        (1, 83 | 84) => draw_user(image, &mut s, a, slot == 84),
        (3, 10) => 6,
        (8, 10) => 7,
        (3 | 8, 11 | 12) => buffer_lock(image, &mut s, a, slot == 12),
        (8, 13) => {
            let Body::IndexBuffer {
                length,
                usage,
                format,
                ..
            } = &s.objects[&a[0]].body
            else {
                unreachable!()
            };
            write(image, a[1], &[*format, 7, *usage, 0, *length]);
            0
        }
        (3, 13) => {
            let Body::VertexBuffer {
                length, usage, fvf, ..
            } = &s.objects[&a[0]].body
            else {
                unreachable!()
            };
            write(image, a[1], &[100, 6, *usage, 0, *length, *fvf]);
            0
        }
        (1, 18) => backbuffer(image, &mut s, a),
        (1, 36) => offscreen(image, &mut s, a),
        (1, 44..=46) => {
            if a[2] == 0 {
                return INVALID;
            }
            if !Transforms::valid(a[1]) {
                return UNAVAILABLE;
            }
            if slot == 45 {
                let Body::Device { transforms, .. } = &s.objects[&a[0]].body else {
                    unreachable!()
                };
                write(image, a[2], &transforms.get(a[1]).unwrap());
            } else {
                let matrix = read::<16>(image, a[2]);
                let Body::Device { transforms, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body
                else {
                    unreachable!()
                };
                if slot == 44 {
                    transforms.set(a[1], matrix);
                } else if !transforms.multiply(a[1], matrix) {
                    return UNAVAILABLE;
                }
            }
            0
        }
        (1, 47 | 48) => {
            if a[1] == 0 {
                return INVALID;
            }
            if slot == 48 {
                let Body::Device { viewport, .. } = &s.objects[&a[0]].body else {
                    unreachable!()
                };
                write(image, a[1], viewport);
            } else {
                let words = read::<6>(image, a[1]);
                let (target, _) = color_target(&s, a[0]);
                let (width, height) = target.dimensions();
                if words[0].checked_add(words[2]).is_none_or(|end| end > width)
                    || words[1]
                        .checked_add(words[3])
                        .is_none_or(|end| end > height)
                {
                    unsupported(
                        "SetViewport outside target extent needs native clipping emulation; Windows may accept this viewport",
                    );
                }
                if alkyhol_d3d9::viewport_from_words(words, width, height).is_err() {
                    return INVALID;
                }
                let Body::Device { viewport, .. } = &mut s.objects.get_mut(&a[0]).unwrap().body
                else {
                    unreachable!()
                };
                *viewport = words;
            }
            0
        }
        (1, 29) => create_depth(image, &mut s, a),
        (1, 39 | 40) => depth_binding(image, &mut s, a, slot == 40),
        (1, 32) => transfer(image, &mut s, a),
        (1, 34) => stretch(&s, a),
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
            if a[1] != 0 || a[2] != 0 || a[3] == 0 || a[3] & !7 != 0 {
                return UNAVAILABLE;
            }
            let (mut active, format) = color_target(&s, a[0]);
            let (width, height) = active.dimensions();
            let Body::Device { viewport, .. } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            if viewport[..4] != [0, 0, width, height] {
                eprintln!(
                    "D3D9 unavailable clear region: viewport={viewport:x?}; target={width}x{height}; flags={:x}",
                    a[3]
                );
                return UNAVAILABLE;
            }
            let depth = depth_target(&s, a[0]);
            let Body::Device {
                gpu, render_state, ..
            } = &s.objects[&a[0]].body
            else {
                unreachable!()
            };
            if a[3] & 6 != 0 {
                let Some(depth) = &depth else {
                    return INVALID;
                };
                if depth.dimensions() != active.dimensions() {
                    return UNAVAILABLE;
                }
                let value = (a[3] & 2 != 0).then(|| f32::from_bits(a[5]));
                if value.is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v)) {
                    return INVALID;
                }
                if a[3] & 4 != 0 && !depth.has_stencil() {
                    return UNAVAILABLE;
                }
                gpu.clear_depth_stencil(depth, value, (a[3] & 4 != 0).then_some(a[6]))
                    .unwrap_or_else(|e| unsupported(&e));
            }
            if a[3] & 1 != 0 {
                let color = if format == 22 {
                    a[4] | 0xff000000
                } else {
                    a[4]
                };
                gpu.clear_state(&mut active, color, render_state)
                    .unwrap_or_else(|e| unsupported(&e));
            }
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
            let (width, height) = target.dimensions();
            capture_present(width, height, &bytes);
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
            profile_present(gpu, image);
            0
        }
        (2, 10) => 1,
        (2, 11) => {
            if a[2] == 0 {
                return POINTER;
            }
            write(image, a[2], &[0]);
            let Body::Surface { container, .. } = &s.objects[&a[0]].body else {
                unreachable!()
            };
            let container = *container;
            if container == 0 {
                return NO_INTERFACE;
            }
            checked(image, a[1], 16, false);
            let requested = unsafe { (a[1] as *const [u8; 16]).read_unaligned() };
            let texture_iid = IIDS[s.objects[&container].body.kind() as usize];
            if ![IID_UNKNOWN, IID_RESOURCE, IID_BASE_TEXTURE, texture_iid].contains(&requested) {
                return NO_INTERFACE;
            }
            expose(&mut s, container);
            write(image, a[2], &[container]);
            0
        }
        (2, 12) => {
            let Body::Surface {
                back,
                attachment,
                depth,
                container,
                width,
                height,
                format,
                ..
            } = &s.objects[&a[0]].body
            else {
                unreachable!()
            };
            let (usage, pool) = if *container != 0 {
                let Body::Texture { usage, pool, .. } = &s.objects[container].body else {
                    unreachable!()
                };
                (*usage, *pool)
            } else if depth.is_some() {
                (2, 0)
            } else if *back || attachment.is_some() {
                (1, 0)
            } else {
                (0, 2)
            };
            write(
                image,
                a[1],
                &[*format, 1, usage, pool, 0, 0, *width, *height],
            );
            0
        }
        (2, 13) => {
            let Body::Surface {
                container,
                texture_level,
                ..
            } = &s.objects[&a[0]].body
            else {
                unreachable!()
            };
            if *container != 0 {
                let mut args = [0; 16];
                args[0] = *container;
                args[1] = *texture_level;
                args[2] = a[1];
                args[3] = a[2];
                args[4] = a[3];
                return texture_operation(image, &mut s, &args, 19);
            }
            transfers::lock_surface(image, &mut s, a)
        }
        (2, 14) => {
            let Body::Surface {
                container,
                texture_level,
                ..
            } = &s.objects[&a[0]].body
            else {
                unreachable!()
            };
            if *container != 0 {
                let mut args = [0; 16];
                args[0] = *container;
                args[1] = *texture_level;
                return texture_operation(image, &mut s, &args, 20);
            }
            let Body::Surface {
                locked,
                initialized,
                ..
            } = &mut s.objects.get_mut(&a[0]).unwrap().body
            else {
                unreachable!()
            };
            if !*locked {
                return INVALID;
            }
            *locked = false;
            *initialized = true;
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
    fn storage_slots_align_without_overflow_or_reusing_tail_bytes() {
        let mut used = 0;
        for length in [1, 15, 16, 17, 65536, 1_048_576, 1_048_575] {
            let (offset, end) = storage_slot(used, length).unwrap();
            assert_eq!(offset % 16, 0);
            assert!(offset >= used);
            assert_eq!(end - offset, length);
            used = end;
        }
        assert_eq!(storage_slot(0, STORAGE_ARENA), Some((0, STORAGE_ARENA)));
        for (used, length) in [
            (0, 0),
            (STORAGE_ARENA, 1),
            (STORAGE_ARENA - 1, 2),
            (u32::MAX, 1),
            (16, u32::MAX),
        ] {
            assert!(storage_slot(used, length).is_none());
        }
    }
    #[test]
    #[ignore = "requires the PE32 host small-PAGEZERO link; run cargo rustc --lib --profile test with -C link-arg=-Wl,-pagezero_size,0x1000"]
    fn actual_small_guest_allocations_pack_disjoint_bytes_and_keep_protection_classes() {
        let image = super::super::tests::layout();
        let cpu = &image.cpu;
        let mut allocations = Vec::new();
        for index in 0..1100u32 {
            let length = 4093 + (index % 7);
            let bytes = vec![(index % 251) as u8; length as usize];
            let p = memory(cpu, &bytes, true, false).unwrap();
            assert_eq!(p % 16, 0);
            assert!(image.permits(p as usize, length as usize, true));
            allocations.push((p, bytes));
        }
        assert_eq!(cpu.com_storage.borrow().len(), 2);
        for (p, bytes) in &allocations {
            assert_eq!(
                unsafe { std::slice::from_raw_parts(*p as *const u8, bytes.len()) },
                bytes
            );
        }
        let immutable = memory(cpu, b"immutable record", false, false).unwrap();
        assert!(image.permits(immutable as usize, 16, false));
        assert!(!image.permits(immutable as usize, 16, true));
        let large = memory(
            cpu,
            &vec![0x7b; STORAGE_ARENA as usize / 4 + 1],
            true,
            false,
        )
        .unwrap();
        assert!(image.permits(large as usize, STORAGE_ARENA as usize / 4 + 1, true));
        assert_eq!(
            unsafe { (large as *const u8).add(STORAGE_ARENA as usize / 4).read() },
            0x7b
        );
        assert_eq!(cpu.com_storage.borrow().len(), 4);
        assert!(memory(cpu, &[], true, false).is_err());
        cpu.com_arenas.borrow_mut().clear();
        cpu.com_storage.borrow_mut().clear();
    }
    #[test]
    fn packed_texture_mips_keep_aligned_disjoint_faces_and_exact_block_layouts() {
        let (layout, total) = texture_cpu_layout(4, 4, 3, 6, SampleFormat::Bgra8).unwrap();
        assert_eq!((layout.len(), total), (18, 576));
        let mut bytes = vec![0; total as usize];
        for (n, mip) in layout.iter().enumerate() {
            let face = n / 3;
            let level = n % 3;
            assert_eq!(mip.offset, face as u32 * 96 + [0, 64, 80][level]);
            assert_eq!(mip.offset % 16, 0);
            assert_eq!((mip.width, mip.height), (4 >> level, 4 >> level));
            assert_eq!(mip.pitch, mip.width * 4);
            bytes[mip.offset as usize..(mip.offset + mip.length) as usize].fill(n as u8 + 1);
        }
        for (n, mip) in layout.iter().enumerate() {
            assert!(
                bytes[mip.offset as usize..(mip.offset + mip.length) as usize]
                    .iter()
                    .all(|&v| v == n as u8 + 1)
            );
        }
        let (bc, total) = texture_cpu_layout(8, 8, 4, 6, SampleFormat::Bc1).unwrap();
        assert_eq!((bc.len(), total), (24, 480));
        assert_eq!(
            bc.iter()
                .take(4)
                .map(|m| (m.pitch, m.length))
                .collect::<Vec<_>>(),
            [(16, 32), (8, 8), (8, 8), (8, 8)]
        );
        let (l8, total) = texture_cpu_layout(3, 3, 2, 1, SampleFormat::Luminance8).unwrap();
        assert_eq!(
            (l8[0].length, l8[1].offset, l8[1].length, total),
            (9, 16, 1, 32)
        );
        for (w, h, mips, faces) in [
            (0, 1, 1, 1),
            (1, 0, 1, 1),
            (8193, 1, 1, 1),
            (4, 4, 0, 1),
            (4, 4, 4, 1),
            (4, 2, 2, 6),
            (4, 4, 1, 7),
            (8192, 8192, 1, 1),
            (4096, 4096, 1, 6),
        ] {
            assert!(texture_cpu_layout(w, h, mips, faces, SampleFormat::Bgra8).is_none());
        }
    }
    #[test]
    fn cube_faces_address_distinct_mips_and_reject_overflow_and_invalid_faces() {
        let mut seen = std::collections::BTreeSet::new();
        for face in 0..6 {
            for level in 0..14 {
                assert!(seen.insert(cube_subresource(face, level, 14).unwrap()));
            }
        }
        assert_eq!(seen.len(), 84);
        assert_eq!(cube_subresource(5, 13, 14), Some(83));
        for (face, level, mips) in [
            (6, 0, 14),
            (0, 14, 14),
            (0, 0, 0),
            (u32::MAX, 0, 1),
            (5, 0, u32::MAX),
        ] {
            assert_eq!(cube_subresource(face, level, mips), None);
        }
        let cube = BASE + 10 * 256;
        assert_eq!(
            signature(cube + 19),
            Some(("IDirect3DCubeTexture9::LockRect", 24))
        );
        assert_eq!(
            signature(cube + 20),
            Some(("IDirect3DCubeTexture9::UnlockRect", 12))
        );
        assert_eq!(
            signature(cube + 18),
            Some(("IDirect3DCubeTexture9::GetCubeMapSurface", 16))
        );
        assert!(signature(cube + 22).is_none());
        assert_ne!(IIDS[7], IIDS[10]);
    }
    #[test]
    fn subrectangle_locks_keep_full_pitch_and_checked_pixel_offsets() {
        for (format, pitch, offset) in [(21, 20, 24), (26, 10, 12), (28, 5, 6), (50, 5, 6)] {
            assert_eq!(
                lock_rectangle_offset(format, 5, 3, pitch, [1, 1, 4, 3]),
                Some(offset)
            );
            for rect in [
                [0, 0, 6, 3],
                [0, 0, 5, 4],
                [2, 2, 1, 3],
                [0, 0, 0, 0],
                [u32::MAX, 1, 4, 3],
            ] {
                assert_eq!(lock_rectangle_offset(format, 5, 3, pitch, rect), None);
            }
        }
        assert_eq!(
            lock_rectangle_offset(0x35545844, 4, 4, 16, [0, 0, 4, 4]),
            Some(0)
        );
        assert_eq!(
            lock_rectangle_offset(0x35545844, 4, 4, 16, [0, 0, 2, 4]),
            None
        );
        assert_eq!(
            lock_rectangle_offset(21, 5, 3, u32::MAX, [1, 2, 4, 3]),
            None
        );
    }
    #[test]
    fn strips_keep_every_triangle_winding_and_degenerate_indices() {
        assert_eq!(primitive_vertices(4, 2), Some(6));
        assert_eq!(primitive_vertices(5, 2), Some(4));
        assert_eq!(primitive_vertices(5, u32::MAX), None);
        assert_eq!(primitive_vertices(5, 0), None);
        assert_eq!(
            strip_triangles(&[7, 8, 9, 10, 11]).unwrap(),
            [7, 8, 9, 8, 10, 9, 9, 10, 11]
        );
        assert_eq!(strip_triangles(&[1, 1, 2, 2]).unwrap(), [1, 1, 2, 1, 2, 2]);
        assert!(strip_triangles(&[1, 2]).is_none());
    }
    #[test]
    fn capability_record_does_not_invent_unimplemented_shader_or_texture_support() {
        let c = capabilities(false);
        assert_eq!(c.len() * 4, 304);
        assert_eq!((c[0], c[47], c[48], c[58], c[60]), (1, 16, 20, 1, 1));
        assert_eq!(c[34], 0xff); // eight real stencil operations; no two-sided capability claim
        assert_eq!(c[3], 0); // no gamma-ramp/calibration support
        assert_eq!(c[49], 0);
        assert_eq!(c[51], 0); // shader versions
        for offset in [
            60, 64, 68, 72, 88, 92, 96, 144, 148, 152, 236, 248, 264, 296, 300,
        ] {
            assert_eq!(c[offset / 4], 0);
        }
        assert_eq!(c[7] & 0x100000, 0); // no pure-device promise
        let p = capabilities(true);
        assert_eq!(p[3], 0); // SetGammaRamp must have no effect in this profile
        assert_eq!((p[22], p[23]), (2048, 2048));
        assert_eq!((p[15], p[16], p[19]), (0xe004, 0x03030300, 7));
        assert_eq!(p[5], 0x80000000);
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
    fn buffer_arena_slots_are_aligned_disjoint_and_checked_before_growth() {
        let mut used = 0;
        for length in [1, 7, 16, 1040, 65536, 131071] {
            let (offset, end) = buffer_slot(used, length).unwrap();
            assert_eq!(offset % 16, 0);
            assert!(offset >= used);
            assert_eq!(end - offset, length);
            used = end;
        }
        assert_eq!(buffer_slot(0, BUFFER_ARENA), Some((0, BUFFER_ARENA)));
        assert_eq!(
            buffer_slot(BUFFER_ARENA - 16, 16),
            Some((BUFFER_ARENA - 16, BUFFER_ARENA))
        );
        for (used, length) in [
            (0, 0),
            (0, BUFFER_ARENA + 1),
            (BUFFER_ARENA, 1),
            (BUFFER_ARENA - 1, 1),
            (u32::MAX, 1),
            (16, u32::MAX),
        ] {
            assert!(buffer_slot(used, length).is_none());
        }
    }
    #[test]
    fn xyz_visibility_rejects_color_alpha_and_vertex_blending() {
        let mut state = RenderState::default();
        assert!(!xyz_visibility_state(&state));
        state.set(168, 0).unwrap();
        assert!(xyz_visibility_state(&state));
        for key in [168, 15, 151, 167] {
            let old = state.get(key).unwrap();
            // Unsupported blending is still rejected by state itself.
            if state.set(key, 1).is_ok() {
                assert!(!xyz_visibility_state(&state));
                state.set(key, old).unwrap();
            }
        }
    }
    #[test]
    fn user_index_ranges_decode_both_widths_without_rebasing_the_source() {
        let words = [2u32, 4, 3];
        for format in [101, 102] {
            let bytes: Vec<u8> = words
                .iter()
                .flat_map(|&n| n.to_le_bytes()[..if format == 101 { 2 } else { 4 }].to_vec())
                .collect();
            assert_eq!(user_indices(&bytes, format, 2, 3), Some(vec![0, 2, 1]));
            for (minimum, count) in [(3, 3), (2, 2), (2, 0), (u32::MAX, 1)] {
                assert_eq!(user_indices(&bytes, format, minimum, count), None);
            }
            assert_eq!(user_indices(&bytes[..bytes.len() - 1], format, 2, 3), None);
        }
        assert_eq!(user_indices(&[], 101, 0, 3), None);
        assert_eq!(user_indices(&[0, 0], 100, 0, 3), None);
        assert_eq!(strip_triangles(&[0, 1, 2, 3]), Some(vec![0, 1, 2, 1, 3, 2]));
        assert_eq!(
            signature(BASE + 256 + 83),
            Some(("IDirect3DDevice9::DrawPrimitiveUP", 20))
        );
        assert_eq!(
            signature(BASE + 256 + 84),
            Some(("IDirect3DDevice9::DrawIndexedPrimitiveUP", 36))
        );
    }
    #[test]
    fn separate_stream_spans_keep_offsets_strides_and_overflow_checks() {
        assert_eq!(vertex_span(256, 16, 16, 2, 3), Some((48, 48)));
        assert_eq!(vertex_span(256, 4, 8, 2, 3), Some((20, 24)));
        for args in [
            (40, 4, 8, 2, 3),
            (256, 0, 0, 0, 3),
            (256, 0, 8, 0, 0),
            (u32::MAX, 0, 2048, u32::MAX, 3),
            (u32::MAX, u32::MAX, 16, 1, 3),
            (u32::MAX, 0, 2048, 0, u32::MAX),
            (u32::MAX, 0, 2048, 0, 32769),
        ] {
            assert_eq!(vertex_span(args.0, args.1, args.2, args.3, args.4), None);
        }
    }
    #[test]
    fn occlusion_vtable_and_i386_stack_extents() {
        for (slot, bytes) in [
            (0, 12),
            (1, 4),
            (2, 4),
            (3, 8),
            (4, 4),
            (5, 4),
            (6, 8),
            (7, 16),
        ] {
            assert_eq!(signature(BASE + 11 * 256 + slot).unwrap().1, bytes);
        }
        assert!(signature(BASE + 11 * 256 + 8).is_none());
        assert_eq!(
            IIDS[11],
            [
                0x60, 0x14, 0x77, 0xd9, 0x95, 0xa6, 0x26, 0x4f, 0xbb, 0xd3, 0x27, 0xb8, 0x40, 0xb5,
                0x41, 0xcc
            ]
        );
    }
    #[test]
    fn com_stack_contracts_and_bounds() {
        assert_eq!(signature(BASE + 16), Some(("IDirect3D9::CreateDevice", 28)));
        assert_eq!(
            signature(BASE + 256 + 21),
            Some(("IDirect3DDevice9::SetGammaRamp", 16))
        );
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
