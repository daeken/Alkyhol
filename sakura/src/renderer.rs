use crate::scene::{Camera, Mesh, instances, load_meshes};
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
    light: [f32; 4],
    width: u32,
    height: u32,
    leaf_vertices: u32,
    debug: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameStats {
    pub primary: [u32; 6],
    pub floor_reflections: u32,
    pub cube_reflections: u32,
    pub secondary_hits: u32,
    pub secondary_misses: u32,
}
pub struct Renderer {
    pub device: Device,
    scene: TriangleScene,
    vertices: Vec<Buffer>,
    textures: Vec<Texture>,
    pipeline: ComputePipelineState,
    presentation: RenderPipelineState,
    output: Texture,
    width: u32,
    height: u32,
    leaf_vertices: u32,
    counts: Buffer,
}
impl Renderer {
    pub fn new(assets: &Path, width: u32, height: u32) -> Result<Self, String> {
        let device = pollster::block_on(D3D12CreateDevice(false)).map_err(|e| e.to_string())?;
        eprintln!("Sakura adapter: {:?}; SER off", device.adapter_info());
        if !device.adapter_info().ray_tracing {
            return Err("Sakura requires a ray-tracing-capable Metal device".into());
        }
        let meshes = load_meshes(assets)?;
        let placement = instances();
        eprintln!(
            "Loaded {} meshes, {} triangles, {} instances",
            meshes.len(),
            meshes.iter().map(|m| m.vertices.len() / 3).sum::<usize>(),
            placement.len()
        );
        let positions = meshes
            .iter()
            .map(|m| m.vertices.iter().map(|v| v.position).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let slices = positions.iter().map(|m| m.as_slice()).collect::<Vec<_>>();
        let transforms = placement
            .iter()
            .map(|i| TriangleInstance {
                mesh: i.mesh,
                transform: [
                    [i.scale, 0., 0.],
                    [0., i.scale, 0.],
                    [0., 0., i.scale],
                    i.position,
                ],
                mask: 1,
            })
            .collect::<Vec<_>>();
        let scene =
            build_triangle_scene(&device, &slices, &transforms).map_err(|e| e.to_string())?;
        let native = device.metal_device();
        let options = CompileOptions::new();
        options.set_fast_math_enabled(false);
        let library = native.new_library_with_source(include_str!("render.metal"), &options)?;
        let kernel = library.get_function("render_sakura", None)?;
        let pipeline = native.new_compute_pipeline_state_with_function(&kernel)?;
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
        let vertices = meshes.iter().map(|m| upload_mesh(native, m)).collect();
        // WIC's default loader recognizes EXIF ColorSpace=1 in the pinned trunk JPEG.
        // The bush JPEG has no sRGB metadata, so WIC uses linear UNORM for that one.
        let textures = [("tree-trunk.jpg", true), ("bush-texture.jpg", false)]
            .iter()
            .map(|(name, srgb)| load_texture(native, &assets.join(name), *srgb))
            .collect::<Result<Vec<_>, _>>()?;
        let output = output_texture(native, width, height)?;
        let counts = native.new_buffer(40, MTLResourceOptions::StorageModeShared);
        Ok(Self {
            device,
            scene,
            vertices,
            textures,
            pipeline,
            presentation,
            output,
            width,
            height,
            leaf_vertices: meshes[2].vertices.len() as u32,
            counts,
        })
    }
    pub fn extent(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.output = output_texture(self.device.metal_device(), width, height)?;
        self.width = width;
        self.height = height;
        Ok(())
    }
    /// Waits for actual GPU completion so callers can safely reuse parameters/readback/resize.
    pub fn render(
        &mut self,
        camera: &Camera,
        seconds: f32,
        drawable: Option<&MetalDrawableRef>,
    ) -> Result<FrameStats, String> {
        objc::rc::autoreleasepool(|| {
            let [right, up, forward] = camera.basis();
            let light = glam::Mat3::from_rotation_y(-seconds * std::f32::consts::TAU / 8.)
                * glam::Vec3::new(0., 15.8, 233.);
            let params = Params {
                eye: camera.eye.extend(1.).to_array(),
                right: right.extend(0.).to_array(),
                up: up.extend(0.).to_array(),
                forward: forward.extend(0.).to_array(),
                light: light.extend(1.).to_array(),
                width: self.width,
                height: self.height,
                leaf_vertices: self.leaf_vertices,
                debug: 0,
            };
            unsafe {
                std::ptr::write_bytes(self.counts.contents(), 0, 40);
            }
            let command = self.device.metal_queue().new_command_buffer();
            let encoder = command.new_compute_command_encoder();
            encoder.set_compute_pipeline_state(&self.pipeline);
            encoder.set_acceleration_structure(0, Some(&self.scene.acceleration));
            for (i, b) in self.vertices.iter().enumerate() {
                encoder.set_buffer(i as u64 + 1, Some(b), 0);
            }
            encoder.set_bytes(
                5,
                std::mem::size_of::<Params>() as u64,
                (&params as *const Params).cast(),
            );
            encoder.set_buffer(6, Some(&self.counts), 0);
            encoder.set_texture(0, Some(&self.textures[0]));
            encoder.set_texture(1, Some(&self.textures[1]));
            encoder.set_texture(2, Some(&self.output));
            self.scene.use_resources(encoder);
            encoder.dispatch_threads(
                MTLSize::new(self.width as u64, self.height as u64, 1),
                MTLSize::new(8, 8, 1),
            );
            encoder.end_encoding();
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
            let counts = unsafe { *(self.counts.contents().cast::<[u32; 10]>()) };
            Ok(FrameStats {
                primary: counts[..6].try_into().unwrap(),
                floor_reflections: counts[6],
                cube_reflections: counts[7],
                secondary_hits: counts[8],
                secondary_misses: counts[9],
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
fn upload_mesh(device: &DeviceRef, mesh: &Mesh) -> Buffer {
    device.new_buffer_with_data(
        mesh.vertices.as_ptr().cast(),
        std::mem::size_of_val(mesh.vertices.as_slice()) as u64,
        MTLResourceOptions::StorageModeShared,
    )
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
fn load_texture(device: &DeviceRef, path: &Path, srgb: bool) -> Result<Texture, String> {
    let image = image::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .to_rgba8();
    let d = TextureDescriptor::new();
    d.set_texture_type(MTLTextureType::D2);
    d.set_pixel_format(if srgb {
        MTLPixelFormat::RGBA8Unorm_sRGB
    } else {
        MTLPixelFormat::RGBA8Unorm
    });
    d.set_width(image.width() as u64);
    d.set_height(image.height() as u64);
    d.set_storage_mode(MTLStorageMode::Shared);
    d.set_usage(MTLTextureUsage::ShaderRead);
    let texture = device.new_texture(&d);
    texture.replace_region(
        MTLRegion::new_2d(0, 0, image.width() as u64, image.height() as u64),
        0,
        image.as_ptr().cast(),
        image.width() as u64 * 4,
    );
    Ok(texture)
}
