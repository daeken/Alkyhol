//! Original hit/SSR bodies on queued native hits, with reflected resource arrays.
use crate::{
    camera::Camera,
    h3d::Mesh,
    original::{dynamic_constants, view_projection},
};
use alkyhol_d3d12::{complete, raytrace::TriangleScene};
use metal::*;

struct AttributeRoutes {
    infos: Vec<[u32; 8]>,
    indices: Vec<u16>,
    roots: Vec<u32>,
}
fn attribute_routes(meshes: &[Mesh]) -> Result<AttributeRoutes, String> {
    if meshes.is_empty() {
        return Err("empty material routing".into());
    }
    let mut data = AttributeRoutes {
        infos: Vec::new(),
        indices: Vec::new(),
        roots: Vec::new(),
    };
    let mut vertex_base = 0usize;
    for mesh in meshes {
        if mesh.material >= 27 || mesh.vertices.is_empty() || mesh.vertices.len() % 3 != 0 {
            return Err("invalid material mesh".into());
        }
        data.roots
            .push(u32::try_from(data.infos.len()).map_err(|_| "material table overflow")?);
        for (chunk, vertices) in mesh.vertices.chunks(65535).enumerate() {
            let base = (vertex_base + chunk * 65535)
                .checked_mul(56)
                .and_then(|v| u32::try_from(v).ok())
                .filter(|&v| v <= u32::MAX - 56)
                .ok_or("attribute range overflow")?;
            let index = u32::try_from(data.indices.len().checked_mul(2).ok_or("index overflow")?)
                .map_err(|_| "index range overflow")?;
            data.infos.push([
                index,
                base + 12,
                base + 20,
                base + 32,
                base + 44,
                base,
                56,
                mesh.material as u32,
            ]);
            data.indices.extend((0..vertices.len()).map(|v| v as u16));
        }
        vertex_base = vertex_base
            .checked_add(mesh.vertices.len())
            .ok_or("vertex count overflow")?;
    }
    // Original Load3x16BitIndices reads two aligned words. Keep the final unused
    // halfword physically present; no triangle or attribute is discarded.
    if !data.indices.len().is_multiple_of(2) {
        data.indices.push(0);
    }
    Ok(data)
}
struct Binding {
    class: String,
    space: u32,
    register: u32,
    index: u64,
    count: u32,
}
struct Kernel {
    pipeline: ComputePipelineState,
    bindings: Vec<Binding>,
}
impl Kernel {
    fn new(device: &DeviceRef, text: &str) -> Result<Self, String> {
        let data: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if data["stage"] != "Compute" || data["threads_per_group"] != serde_json::json!([8, 8, 1]) {
            return Err("wrong queued shader stage/group".into());
        }
        let mut bindings = Vec::new();
        let mut used = std::collections::BTreeSet::new();
        for resource in data["resources"]
            .as_array()
            .ok_or("queued bindings missing")?
        {
            let class = resource["class"].as_str().ok_or("binding class missing")?;
            let space = resource["space"]
                .as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or("invalid register space")?;
            let register = resource["register"]
                .as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or("invalid register")?;
            let (namespace, kind, count) = match (class, space, register) {
                ("Cbv", 0, 0 | 1 | 20 | 21) => ("Buffer", "ConstantBuffer", 1),
                ("Srv", 0, 0) => ("Buffer", "RtAccelerationStructure", 1),
                ("Srv", 0, 1 | 21 | 25) => ("Buffer", "StructuredBuffer", 1),
                ("Srv", 0, 2 | 3) => ("Buffer", "RawBuffer", 1),
                ("Srv", 0, 4 | 5 | 12 | 13) => ("Texture", "Texture2D", 1),
                ("Srv", 1..=3, 0) => ("Texture", "Texture2D", 27),
                ("Uav", 0, 20 | 21) => ("Buffer", "StructuredBuffer", 1),
                ("Uav", 0, 2 | 23) => ("Texture", "Texture2D", 1),
                ("Sampler", 0, 0 | 1) => ("Sampler", "Sampler", 1),
                _ => {
                    return Err(format!(
                        "unsupported queued binding {class}{register} space{space}"
                    ));
                }
            };
            if resource["namespace"] != namespace
                || resource["kind"] != kind
                || resource["count"] != count
            {
                return Err("queued binding type/count mismatch".into());
            }
            let index = resource["metal_index"]
                .as_u64()
                .ok_or("Metal index missing")?;
            let limit = match namespace {
                "Buffer" => 24,
                "Texture" => 128,
                _ => 16,
            };
            if index > limit || count > limit - index {
                return Err("queued Metal binding limit exceeded".into());
            }
            for i in index..index + count {
                if !used.insert((namespace, i)) {
                    return Err("aliased queued bindings".into());
                }
            }
            bindings.push(Binding {
                class: class.into(),
                space,
                register,
                index,
                count: count as u32,
            });
        }
        let options = CompileOptions::new();
        options.set_fast_math_enabled(false);
        let library = device
            .new_library_with_source(data["msl"].as_str().ok_or("queued MSL missing")?, &options)?;
        let function = library.get_function(
            data["entry_point"].as_str().ok_or("queued entry missing")?,
            None,
        )?;
        Ok(Self {
            pipeline: device.new_compute_pipeline_state_with_function(&function)?,
            bindings,
        })
    }
}
pub(crate) struct Deferred {
    material: Kernel,
    reflection: Kernel,
    diagnostic: Kernel,
    reflection_bary: Kernel,
    infos: Buffer,
    indices: Buffer,
    roots: Buffer,
    ssao: Texture,
    neutral_shadow: Texture,
    sampler: SamplerState,
    shadow_sampler: SamplerState,
}
pub(crate) struct Frame<'a> {
    pub scene: &'a TriangleScene,
    pub camera: &'a Camera,
    pub attributes: &'a BufferRef,
    pub textures: &'a [Texture],
    pub counts: &'a BufferRef,
    pub packets: &'a BufferRef,
    pub depth: &'a TextureRef,
    pub normals: &'a TextureRef,
    pub output: &'a TextureRef,
    pub mode: u32,
    pub pass: u32,
}
#[allow(clippy::approx_constant)] // Preserve the existing sample's sun literals.
fn lighting_constants(mode: u32, pass: u32) -> [u32; 36] {
    let theta = -0.5f32;
    let phi = 0.75f32 * 3.14159f32 * 0.5;
    let sun =
        glam::Vec3::new(theta.cos() * phi.cos(), phi.sin(), theta.sin() * phi.cos()).normalize();
    let mut data = [0u32; 36];
    for (i, v) in sun.to_array().into_iter().enumerate() {
        data[i] = v.to_bits();
    }
    for i in [4, 5, 6] {
        data[i] = 4f32.to_bits();
    }
    for i in [8, 9, 10] {
        data[i] = 0.1f32.to_bits();
    }
    data[12] = 1f32.to_bits();
    data[13] = 1f32.to_bits();
    data[32] = pass;
    data[33] = u32::from(mode != 5);
    data
}
fn full_ssao(device: &DeviceRef, width: u32, height: u32) -> Texture {
    let d = TextureDescriptor::new();
    d.set_texture_type(MTLTextureType::D2);
    d.set_width(width as u64);
    d.set_height(height as u64);
    d.set_pixel_format(MTLPixelFormat::R32Float);
    d.set_storage_mode(MTLStorageMode::Shared);
    d.set_usage(MTLTextureUsage::ShaderRead);
    let texture = device.new_texture(&d);
    let values = vec![1f32; width as usize * height as usize];
    texture.replace_region(
        MTLRegion::new_2d(0, 0, width as u64, height as u64),
        0,
        values.as_ptr().cast(),
        width as u64 * 4,
    );
    texture
}
impl Deferred {
    pub(crate) fn new(
        device: &DeviceRef,
        queue: &CommandQueueRef,
        meshes: &[Mesh],
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        let routes = attribute_routes(meshes)?;
        let buffer = |bytes: &[u8]| {
            device.new_buffer_with_data(
                bytes.as_ptr().cast(),
                bytes.len() as u64,
                MTLResourceOptions::StorageModeShared,
            )
        };
        let desc = SamplerDescriptor::new();
        desc.set_min_filter(MTLSamplerMinMagFilter::Linear);
        desc.set_mag_filter(MTLSamplerMinMagFilter::Linear);
        desc.set_mip_filter(MTLSamplerMipFilter::Linear);
        desc.set_max_anisotropy(8);
        desc.set_address_mode_s(MTLSamplerAddressMode::Repeat);
        desc.set_address_mode_t(MTLSamplerAddressMode::Repeat);
        let sampler = device.new_sampler(&desc);
        desc.set_max_anisotropy(1);
        desc.set_compare_function(MTLCompareFunction::GreaterEqual);
        desc.set_address_mode_s(MTLSamplerAddressMode::ClampToEdge);
        desc.set_address_mode_t(MTLSamplerAddressMode::ClampToEdge);
        desc.set_mip_filter(MTLSamplerMipFilter::NotMipmapped);
        let shadow_sampler = device.new_sampler(&desc);
        let td = TextureDescriptor::new();
        td.set_texture_type(MTLTextureType::D2);
        td.set_width(1);
        td.set_height(1);
        td.set_pixel_format(MTLPixelFormat::Depth32Float);
        td.set_storage_mode(MTLStorageMode::Private);
        td.set_usage(MTLTextureUsage::ShaderRead | MTLTextureUsage::RenderTarget);
        let neutral_shadow = device.new_texture(&td);
        // Explicit neutral input for the named no-shadows mode. Original PCF
        // executes against a real reverse-Z far-depth texture cleared to zero.
        let pass = RenderPassDescriptor::new();
        let depth = pass.depth_attachment().unwrap();
        depth.set_texture(Some(&neutral_shadow));
        depth.set_load_action(MTLLoadAction::Clear);
        depth.set_clear_depth(0.);
        depth.set_store_action(MTLStoreAction::Store);
        let command = queue.new_command_buffer();
        let encoder = command.new_render_command_encoder(pass);
        encoder.end_encoding();
        complete(command).map_err(|e| e.to_string())?;
        Ok(Self {
            material: Kernel::new(
                device,
                include_str!("../../shader/samples/miniengine/material.generated.json"),
            )?,
            reflection: Kernel::new(
                device,
                include_str!("../../shader/samples/miniengine/reflection.generated.json"),
            )?,
            diagnostic: Kernel::new(
                device,
                include_str!("../../shader/samples/miniengine/diagnostic.generated.json"),
            )?,
            reflection_bary: Kernel::new(
                device,
                include_str!("../../shader/samples/miniengine/reflection_bary.generated.json"),
            )?,
            infos: buffer(bytemuck::cast_slice(&routes.infos)),
            indices: buffer(bytemuck::cast_slice(&routes.indices)),
            roots: buffer(bytemuck::cast_slice(&routes.roots)),
            ssao: full_ssao(device, width, height),
            neutral_shadow,
            sampler,
            shadow_sampler,
        })
    }
    pub(crate) fn resize(&mut self, device: &DeviceRef, width: u32, height: u32) {
        self.ssao = full_ssao(device, width, height);
    }
    pub(crate) fn encode(&self, command: &CommandBufferRef, kind: &str, frame: Frame<'_>) {
        let kernel = match kind {
            "material" => &self.material,
            "reflection" => &self.reflection,
            "diagnostic" => &self.diagnostic,
            "reflection_bary" => &self.reflection_bary,
            _ => unreachable!("validated internal pass kind"),
        };
        let width = frame.output.width();
        let height = frame.output.height();
        let dynamic = dynamic_constants(frame.camera, width, height);
        let projection = view_projection(frame.camera, width, height).to_cols_array();
        let lighting = lighting_constants(frame.mode, frame.pass);
        let settings = [width as u32, height as u32, frame.mode, frame.pass];
        let encoder = command.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&kernel.pipeline);
        for b in &kernel.bindings {
            match (b.class.as_str(), b.space, b.register) {
                ("Cbv", 0, 0) => encoder.set_bytes(b.index, 144, lighting.as_ptr().cast()),
                ("Cbv", 0, 1) => encoder.set_bytes(
                    b.index,
                    std::mem::size_of_val(&dynamic) as u64,
                    &dynamic as *const _ as *const std::ffi::c_void,
                ),
                ("Cbv", 0, 20) => encoder.set_bytes(b.index, 16, settings.as_ptr().cast()),
                ("Cbv", 0, 21) => encoder.set_bytes(b.index, 64, projection.as_ptr().cast()),
                ("Srv", 0, 0) => {
                    encoder.set_acceleration_structure(b.index, Some(&frame.scene.acceleration))
                }
                ("Srv", 0, 1) => encoder.set_buffer(b.index, Some(&self.infos), 0),
                ("Srv", 0, 2) => encoder.set_buffer(b.index, Some(&self.indices), 0),
                ("Srv", 0, 3) => encoder.set_buffer(b.index, Some(frame.attributes), 0),
                ("Srv", 0, 4) => encoder.set_texture(b.index, Some(&self.neutral_shadow)),
                ("Srv", 0, 5) => encoder.set_texture(b.index, Some(&self.ssao)),
                ("Srv", 0, 12) => encoder.set_texture(b.index, Some(frame.depth)),
                ("Srv", 0, 13) => encoder.set_texture(b.index, Some(frame.normals)),
                ("Srv", 0, 21) => encoder.set_buffer(b.index, Some(frame.packets), 0),
                ("Srv", 0, 25) => encoder.set_buffer(b.index, Some(&self.roots), 0),
                ("Srv", space @ 1..=3, 0) => {
                    for i in 0..b.count {
                        encoder.set_texture(
                            b.index + i as u64,
                            Some(&frame.textures[(space as usize - 1) * 27 + i as usize]),
                        );
                    }
                }
                ("Uav", 0, 2) => encoder.set_texture(b.index, Some(frame.output)),
                ("Uav", 0, 20) => encoder.set_buffer(b.index, Some(frame.counts), 0),
                ("Uav", 0, 21) => {
                    encoder.set_buffer(b.index, Some(frame.packets), width * height * 64)
                }
                ("Uav", 0, 23) => encoder.set_texture(b.index, Some(frame.normals)),
                ("Sampler", 0, 0) => encoder.set_sampler_state(b.index, Some(&self.sampler)),
                ("Sampler", 0, 1) => encoder.set_sampler_state(b.index, Some(&self.shadow_sampler)),
                _ => unreachable!("bindings validated at construction"),
            }
        }
        frame.scene.use_resources(encoder);
        encoder.dispatch_thread_groups(
            MTLSize::new(width.div_ceil(8), height.div_ceil(8), 1),
            MTLSize::new(8, 8, 1),
        );
        encoder.end_encoding();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queued_metadata_rejects_overflow_aliasing_wrong_arrays_and_stage() {
        let device = Device::system_default().expect("Metal device required");
        let source: serde_json::Value = serde_json::from_str(include_str!(
            "../../shader/samples/miniengine/material.generated.json"
        ))
        .unwrap();
        for case in 0..4 {
            let mut data = source.clone();
            match case {
                0 => data["resources"][0]["metal_index"] = u64::MAX.into(),
                1 => {
                    data["resources"][1]["metal_index"] =
                        data["resources"][0]["metal_index"].clone()
                }
                2 => {
                    let array = data["resources"]
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|r| r["count"] == 27)
                        .unwrap();
                    array["count"] = 26.into();
                }
                _ => data["stage"] = "Pixel".into(),
            }
            assert!(
                Kernel::new(&device, &data.to_string()).is_err(),
                "case {case}"
            );
        }
    }
    #[test]
    fn expanded_attributes_route_across_u16_chunks_without_dropping_triangles() {
        let vertex = crate::h3d::Vertex {
            position: [0.; 3],
            uv: [0.; 2],
            normal: [0., 0., 1.],
            tangent: [1., 0., 0.],
            bitangent: [0., 1., 0.],
        };
        let mesh = Mesh {
            material: 10,
            vertices: vec![vertex; 65538],
        };
        let routes = attribute_routes(&[mesh]).unwrap();
        assert_eq!(routes.infos.len(), 2);
        assert_eq!(routes.roots, vec![0]);
        assert_eq!(routes.infos[1][0], 65535 * 2);
        assert_eq!(routes.infos[1][5], 65535 * 56);
        assert_eq!(&routes.indices[65535..65538], &[0, 1, 2]);
        assert_eq!(routes.infos[0][7], 10);
        assert_eq!(routes.indices.len(), 65538);
        assert!(attribute_routes(&[]).is_err());
        assert!(
            attribute_routes(&[Mesh {
                material: 27,
                vertices: vec![vertex; 3]
            }])
            .is_err()
        );
    }
}
