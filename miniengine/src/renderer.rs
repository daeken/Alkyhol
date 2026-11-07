use crate::{camera::Camera, h3d};
use alkyhol_d3d12::{
    D3D12CreateDevice, Device, complete,
    raytrace::{TriangleInstance, TriangleScene, build_triangle_scene},
};
use bytemuck::{Pod, Zeroable};
use metal::*;
use std::path::Path;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    eye: [f32; 4],
    right: [f32; 4],
    up: [f32; 4],
    forward: [f32; 4],
    width: u32,
    height: u32,
    mode: u32,
    padding: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameStats {
    pub per_mesh: [u32; 34],
    pub misses: u32,
    pub shadow_rays: u32,
    pub shadow_hits: u32,
    pub reflection_rays: u32,
    pub reflection_hits: u32,
    pub repaired_frames: u32,
    pub short_self_reflections: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum RenderMode {
    Geometry = 0,
    Lit = 1,
    ReflectionBary = 3,
    Shadows = 4,
    LitNoShadows = 5,
    LitShadowRays = 6,
    Reflections = 7,
    Bary = 2,
    Normals = 8,
    Depth = 9,
}
impl RenderMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Lit => "Textured + shadow rays",
            Self::ReflectionBary => "Reflection barycentrics",
            Self::Shadows => "Shadow visibility",
            Self::LitNoShadows => "Textured (no shadows)",
            Self::LitShadowRays => "Diffuse + shadow rays",
            Self::Reflections => "Textured + reflections",
            Self::Geometry => "Geometry diagnostic",
            Self::Bary => "Bary Rays",
            Self::Normals => "Normal diagnostic",
            Self::Depth => "Depth diagnostic",
        }
    }
}
pub struct Renderer {
    textures: Vec<Texture>,
    texture_arguments: Buffer,
    vertices: Buffer,
    offsets: Buffer,
    mode: RenderMode,
    pub device: Device,
    scene: TriangleScene,
    pipeline: ComputePipelineState,
    presentation: RenderPipelineState,
    output: Texture,
    width: u32,
    height: u32,
    counts: Buffer,
    original_bary: crate::original::BaryPipeline,
    original_bary_enabled: bool,
    original_camera: crate::original::CameraPipeline,
    camera_rays: Buffer,
}
impl Renderer {
    pub fn new(assets: &Path, width: u32, height: u32) -> Result<Self, String> {
        let device = pollster::block_on(D3D12CreateDevice(false)).map_err(|e| e.to_string())?;
        eprintln!(
            "MiniEngine adapter: {:?}; native Metal rays and material effects",
            device.adapter_info()
        );
        if !device.adapter_info().ray_tracing {
            return Err("MiniEngine geometry view requires Metal ray tracing".into());
        }
        let bytes = std::fs::read(assets.join("Sponza/sponza.h3d")).map_err(|e| e.to_string())?;
        let model = h3d::parse(&bytes)?;
        if model.meshes.len() != 34 || model.materials.len() != 27 {
            return Err("this pinned scene expects exactly 34 meshes and 27 materials".into());
        }
        eprintln!(
            "Sponza: {} meshes, {} materials, {} triangles",
            model.meshes.len(),
            model.materials.len(),
            model
                .meshes
                .iter()
                .map(|m| m.vertices.len() / 3)
                .sum::<usize>()
        );
        let positions = model
            .meshes
            .iter()
            .map(|m| m.vertices.iter().map(|v| v.position).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let slices = positions.iter().map(|p| p.as_slice()).collect::<Vec<_>>();
        let transforms = (0..model.meshes.len())
            .map(|i| TriangleInstance {
                mesh: i as u32,
                transform: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.], [0., 0., 0.]],
                mask: 1,
                disable_culling: false,
            })
            .collect::<Vec<_>>();
        let scene =
            build_triangle_scene(&device, &slices, &transforms).map_err(|e| e.to_string())?;
        let native = device.metal_device();
        let mut flat = Vec::new();
        let mut offsets = Vec::new();
        for mesh in &model.meshes {
            offsets.push([flat.len() as u32, mesh.material as u32]);
            flat.extend_from_slice(&mesh.vertices);
        }
        let vertices = native.new_buffer_with_data(
            flat.as_ptr().cast(),
            std::mem::size_of_val(flat.as_slice()) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let offsets = native.new_buffer_with_data(
            offsets.as_ptr().cast(),
            std::mem::size_of_val(offsets.as_slice()) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let options = CompileOptions::new();
        options.set_fast_math_enabled(false);
        let library = native.new_library_with_source(
            concat!(
                include_str!("render.metal"),
                "\n",
                include_str!("shading.metal"),
                "\n",
                include_str!("kernel.metal")
            ),
            &options,
        )?;
        let kernel = library.get_function("render_scene", None)?;
        let pipeline = native.new_compute_pipeline_state_with_function(&kernel)?;
        if native.argument_buffers_support() != MTLArgumentBuffersTier::Tier2 {
            return Err("Metal Tier 2 argument buffers are required for material textures".into());
        }
        let encoder = kernel.new_argument_encoder(5);
        let texture_arguments = native.new_buffer(
            encoder.encoded_length(),
            MTLResourceOptions::StorageModeShared,
        );
        encoder.set_argument_buffer(&texture_arguments, 0);
        let defaults = [
            crate::dds::solid(native, [255; 4]),
            crate::dds::solid(native, [128, 128, 255, 0]),
            crate::dds::solid(native, [0, 0, 0, 255]),
        ];
        let mut cache = std::collections::HashMap::new();
        let mut textures = Vec::new();
        for role in 0..3 {
            for (material, paths) in crate::material_paths::MATERIAL_PATHS.iter().enumerate() {
                let texture = if let Some(path) = paths[role] {
                    if !cache.contains_key(path) {
                        let bytes = std::fs::read(assets.join("Sponza").join(path))
                            .map_err(|e| format!("missing material {path}: {e}"))?;
                        cache.insert(
                            path,
                            crate::dds::upload(native, &bytes)
                                .map_err(|e| format!("{path}: {e}"))?,
                        );
                    }
                    cache[path].clone()
                } else {
                    defaults[role].clone()
                };
                encoder.set_texture((role * 27 + material) as u64, &texture);
                textures.push(texture);
            }
        }
        eprintln!(
            "Loaded {} original DDS textures, including original BC1 mip chains; documented engine defaults for absent slots",
            cache.len()
        );
        let vertex = library.get_function("screen_vertex", None)?;
        let fragment = library.get_function("screen_fragment", None)?;
        let descriptor = RenderPipelineDescriptor::new();
        descriptor.set_vertex_function(Some(&vertex));
        descriptor.set_fragment_function(Some(&fragment));
        descriptor
            .color_attachments()
            .object_at(0)
            .unwrap()
            .set_pixel_format(MTLPixelFormat::BGRA8Unorm);
        let presentation = native.new_render_pipeline_state(&descriptor)?;
        let output = output_texture(native, width, height)?;
        let counts = native.new_buffer(168, MTLResourceOptions::StorageModeShared);
        let original_bary = crate::original::BaryPipeline::new(native)?;
        let original_camera = crate::original::CameraPipeline::new(native)?;
        let camera_rays = native.new_buffer(
            width as u64 * height as u64 * 32,
            MTLResourceOptions::StorageModePrivate,
        );
        Ok(Self {
            textures,
            texture_arguments,
            vertices,
            offsets,
            mode: RenderMode::Geometry,
            device,
            scene,
            pipeline,
            presentation,
            output,
            width,
            height,
            counts,
            original_bary,
            original_bary_enabled: true,
            original_camera,
            camera_rays,
        })
    }
    pub fn set_mode(&mut self, mode: RenderMode) {
        self.mode = mode;
    }
    pub fn mode(&self) -> RenderMode {
        self.mode
    }
    /// Explicit A/B reference control; original HLSL is the default for view 2.
    /// A compilation failure never silently switches to the manual reference.
    pub fn set_original_bary_enabled(&mut self, enabled: bool) {
        self.original_bary_enabled = enabled;
    }
    pub fn extent(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.output = output_texture(self.device.metal_device(), width, height)?;
        self.camera_rays = self.device.metal_device().new_buffer(
            width as u64 * height as u64 * 32,
            MTLResourceOptions::StorageModePrivate,
        );
        self.width = width;
        self.height = height;
        Ok(())
    }
    /// Waits for actual GPU completion so callers can safely reuse parameters/readback/resize.
    pub fn render(
        &mut self,
        camera: &Camera,
        drawable: Option<&MetalDrawableRef>,
    ) -> Result<FrameStats, String> {
        objc::rc::autoreleasepool(|| {
            let [right, up, forward] = camera.basis();
            let params = Params {
                eye: camera.eye.extend(1.).to_array(),
                right: right.extend(0.).to_array(),
                up: up.extend(0.).to_array(),
                forward: forward.extend(0.).to_array(),
                width: self.width,
                height: self.height,
                mode: self.mode as u32,
                padding: 0,
            };
            unsafe {
                std::ptr::write_bytes(self.counts.contents(), 0, 168);
            }
            let command = self.device.metal_queue().new_command_buffer();
            if self.mode == RenderMode::Bary && self.original_bary_enabled {
                self.original_bary
                    .encode(command, &self.scene, camera, &self.output, &self.counts);
            } else {
                self.original_camera.encode(
                    command,
                    camera,
                    self.width as u64,
                    self.height as u64,
                    &self.camera_rays,
                );
                let encoder = command.new_compute_command_encoder();
                encoder.set_compute_pipeline_state(&self.pipeline);
                encoder.set_acceleration_structure(0, Some(&self.scene.acceleration));
                encoder.set_bytes(
                    1,
                    std::mem::size_of::<Params>() as u64,
                    (&params as *const Params).cast(),
                );
                encoder.set_buffer(2, Some(&self.counts), 0);
                encoder.set_buffer(3, Some(&self.vertices), 0);
                encoder.set_buffer(4, Some(&self.offsets), 0);
                encoder.set_buffer(5, Some(&self.texture_arguments), 0);
                encoder.set_buffer(6, Some(&self.camera_rays), 0);
                for texture in &self.textures {
                    encoder.use_resource(texture, MTLResourceUsage::Read);
                }
                encoder.set_texture(0, Some(&self.output));
                self.scene.use_resources(encoder);
                encoder.dispatch_threads(
                    MTLSize::new(self.width as u64, self.height as u64, 1),
                    MTLSize::new(8, 8, 1),
                );
                encoder.end_encoding();
            }
            if let Some(drawable) = drawable {
                let pass = RenderPassDescriptor::new();
                let a = pass.color_attachments().object_at(0).unwrap();
                a.set_texture(Some(drawable.texture()));
                a.set_load_action(MTLLoadAction::DontCare);
                a.set_store_action(MTLStoreAction::Store);
                let encoder = command.new_render_command_encoder(pass);
                encoder.set_render_pipeline_state(&self.presentation);
                encoder.set_fragment_texture(0, Some(&self.output));
                encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, 3);
                encoder.end_encoding();
                command.present_drawable(drawable);
            }
            complete(command).map_err(|e| e.to_string())?;
            let counts = unsafe { *(self.counts.contents().cast::<[u32; 42]>()) };
            if counts[41] != 0 {
                return Err(format!(
                    "{} nonfinite shaded surfaces; frame rejected",
                    counts[41]
                ));
            }
            Ok(FrameStats {
                per_mesh: counts[..34].try_into().unwrap(),
                misses: counts[34],
                shadow_rays: counts[35],
                shadow_hits: counts[36],
                reflection_rays: counts[37],
                reflection_hits: counts[38],
                repaired_frames: counts[39],
                short_self_reflections: counts[40],
            })
        })
    }
    pub fn capture(&self, path: &Path) -> Result<(), String> {
        let row = (self.width as u64 * 4).div_ceil(256) * 256;
        let buffer = self.device.metal_device().new_buffer(
            row * self.height as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let command = self.device.metal_queue().new_command_buffer();
        let encoder = command.new_blit_command_encoder();
        encoder.copy_from_texture_to_buffer(
            &self.output,
            0,
            0,
            MTLOrigin { x: 0, y: 0, z: 0 },
            MTLSize::new(self.width as u64, self.height as u64, 1),
            &buffer,
            0,
            row,
            row * self.height as u64,
            MTLBlitOption::None,
        );
        encoder.end_encoding();
        complete(command).map_err(|e| e.to_string())?;
        let mut pixels = Vec::with_capacity(self.width as usize * self.height as usize * 4);
        let bytes = unsafe {
            std::slice::from_raw_parts(buffer.contents().cast::<u8>(), buffer.length() as usize)
        };
        for y in 0..self.height as usize {
            pixels.extend_from_slice(
                &bytes[y * row as usize..y * row as usize + self.width as usize * 4],
            );
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        image::save_buffer(
            path,
            &pixels,
            self.width,
            self.height,
            image::ColorType::Rgba8,
        )
        .map_err(|e| e.to_string())
    }
}
fn output_texture(device: &DeviceRef, width: u32, height: u32) -> Result<Texture, String> {
    if width == 0 || height == 0 || width > 4096 || height > 4096 {
        return Err("render extent must be between 1 and 4096".into());
    }
    let d = TextureDescriptor::new();
    d.set_texture_type(MTLTextureType::D2);
    d.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
    d.set_width(width as u64);
    d.set_height(height as u64);
    d.set_storage_mode(MTLStorageMode::Private);
    d.set_usage(MTLTextureUsage::ShaderRead | MTLTextureUsage::ShaderWrite);
    Ok(device.new_texture(&d))
}
