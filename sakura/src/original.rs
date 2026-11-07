//! Fixed SER-off source adapter for the original Sakura shader bodies.
use crate::scene::{Camera, Mesh, Vertex};
use alkyhol_d3d12::raytrace::TriangleScene;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use metal::*;

const CUBE_INDICES: [u32; 36] = [
    3, 1, 0, 2, 1, 3, 6, 4, 5, 7, 4, 6, 11, 9, 8, 10, 9, 11, 14, 12, 13, 15, 12, 14, 19, 17, 16,
    18, 17, 19, 22, 20, 21, 23, 20, 22,
];
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct SourceVertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 3],
}
impl From<Vertex> for SourceVertex {
    fn from(v: Vertex) -> Self {
        Self {
            position: v.position,
            normal: v.normal,
            uv: [v.uv[0], v.uv[1], 0.],
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SceneCB {
    projection_to_world: [f32; 16],
    camera: [f32; 4],
    light: [f32; 4],
    ambient: [f32; 4],
    diffuse: [f32; 4],
    sort_mode: u32,
    padding: [u32; 3],
}
pub(crate) struct OriginalPipeline {
    pipeline: ComputePipelineState,
    slots: std::collections::BTreeMap<(String, u32), u64>,
    buffers: Vec<(u32, Buffer)>,
    sampler: SamplerState,
}
pub(crate) struct Frame<'a> {
    pub scene: &'a TriangleScene,
    pub camera: &'a Camera,
    pub light: Vec3,
    pub textures: &'a [Texture],
    pub output: &'a TextureRef,
    pub counts: &'a BufferRef,
}
fn source_buffers(meshes: &[Mesh]) -> Result<Vec<(u32, Vec<u8>)>, String> {
    if meshes.len() != 4
        || meshes[0].vertices.len() != 36
        || meshes
            .iter()
            .any(|m| m.vertices.is_empty() || m.vertices.len() % 3 != 0)
    {
        return Err(
            "original Sakura buffer contract requires four complete triangle meshes".into(),
        );
    }
    let mut cube = [SourceVertex::zeroed(); 24];
    let mut initialized = [false; 24];
    for (&index, vertex) in CUBE_INDICES.iter().zip(&meshes[0].vertices) {
        let value = SourceVertex::from(*vertex);
        let i = index as usize;
        if initialized[i] && cube[i] != value {
            return Err("expanded cube conflicts with original indexed cube".into());
        }
        cube[i] = value;
        initialized[i] = true;
    }
    let mut leaf_indices: Vec<u32> = (0..meshes[2].vertices.len() as u32).collect();
    // The original trunk shader reads the leaf index view. D3D out-of-view
    // ByteAddressBuffer reads yield zero, i.e. vertex index 0, not a zero normal.
    // The pinned shader never queries descriptor length. Physical zero padding
    // preserves these exact reads without relying on undefined Metal OOB loads.
    leaf_indices.resize(leaf_indices.len().max(meshes[1].vertices.len()), 0);
    let mut buffers = vec![
        (1, bytemuck::cast_slice(&CUBE_INDICES).to_vec()),
        (2, bytemuck::cast_slice(&cube).to_vec()),
        (5, bytemuck::cast_slice(&leaf_indices).to_vec()),
    ];
    for (mesh, register) in [(1, 4), (2, 6), (3, 8)] {
        let vertices: Vec<SourceVertex> = meshes[mesh]
            .vertices
            .iter()
            .copied()
            .map(SourceVertex::from)
            .collect();
        buffers.push((register, bytemuck::cast_slice(&vertices).to_vec()));
    }
    let bush_indices: Vec<u32> = (0..meshes[3].vertices.len() as u32).collect();
    buffers.push((7, bytemuck::cast_slice(&bush_indices).to_vec()));
    Ok(buffers)
}
impl OriginalPipeline {
    pub(crate) fn new(device: &DeviceRef, meshes: &[Mesh]) -> Result<Self, String> {
        let data: serde_json::Value = serde_json::from_str(include_str!(
            "../../shader/samples/sakura/ser_off.generated.json"
        ))
        .map_err(|e| e.to_string())?;
        if data["stage"] != "Compute"
            || data["requires_ray_query"] != true
            || data["threads_per_group"] != serde_json::json!([8, 8, 1])
        {
            return Err("wrong original Sakura shader contract".into());
        }
        let resources = data["resources"]
            .as_array()
            .ok_or("missing original Sakura bindings")?;
        let expected = [
            ("Cbv", 0, "Buffer"),
            ("Cbv", 20, "Buffer"),
            ("Srv", 0, "Buffer"),
            ("Srv", 1, "Buffer"),
            ("Srv", 2, "Buffer"),
            ("Srv", 4, "Buffer"),
            ("Srv", 5, "Buffer"),
            ("Srv", 6, "Buffer"),
            ("Srv", 7, "Buffer"),
            ("Srv", 8, "Buffer"),
            ("Srv", 9, "Texture"),
            ("Srv", 10, "Texture"),
            ("Uav", 0, "Texture"),
            ("Uav", 20, "Buffer"),
            ("Sampler", 0, "Sampler"),
            ("Sampler", 1, "Sampler"),
        ];
        if resources.len() != expected.len() {
            return Err("unexpected original Sakura binding count".into());
        }
        let mut slots = std::collections::BTreeMap::new();
        let mut used = std::collections::BTreeSet::new();
        for (class, register, namespace) in expected {
            let found: Vec<_> = resources
                .iter()
                .filter(|r| {
                    r["class"] == class
                        && r["register"] == register
                        && r["space"] == 0
                        && r["namespace"] == namespace
                })
                .collect();
            if found.len() != 1 {
                return Err("original Sakura binding missing/duplicated".into());
            }
            let slot = found[0]["metal_index"]
                .as_u64()
                .filter(|&v| {
                    v < if namespace == "Texture" {
                        128
                    } else if namespace == "Sampler" {
                        16
                    } else {
                        24
                    }
                })
                .ok_or("invalid original Sakura Metal slot")?;
            if !used.insert((namespace, slot)) {
                return Err("original Sakura binding aliases".into());
            }
            slots.insert((class.to_string(), register), slot);
        }
        let options = CompileOptions::new();
        options.set_fast_math_enabled(false);
        let library = device.new_library_with_source(
            data["msl"].as_str().ok_or("missing original Sakura MSL")?,
            &options,
        )?;
        let function = library.get_function(
            data["entry_point"]
                .as_str()
                .ok_or("missing original Sakura entry")?,
            None,
        )?;
        let pipeline = device.new_compute_pipeline_state_with_function(&function)?;
        let buffers = source_buffers(meshes)?
            .into_iter()
            .map(|(register, bytes)| {
                (
                    register,
                    device.new_buffer_with_data(
                        bytes.as_ptr().cast(),
                        bytes.len() as u64,
                        MTLResourceOptions::StorageModeShared,
                    ),
                )
            })
            .collect();
        let sampler = SamplerDescriptor::new();
        sampler.set_min_filter(MTLSamplerMinMagFilter::Linear);
        sampler.set_mag_filter(MTLSamplerMinMagFilter::Linear);
        sampler.set_address_mode_s(MTLSamplerAddressMode::Repeat);
        sampler.set_address_mode_t(MTLSamplerAddressMode::Repeat);
        sampler.set_address_mode_r(MTLSamplerAddressMode::Repeat);
        Ok(Self {
            pipeline,
            slots,
            buffers,
            sampler: device.new_sampler(&sampler),
        })
    }
    fn slot(&self, class: &str, register: u32) -> u64 {
        self.slots[&(class.to_string(), register)]
    }
    pub(crate) fn encode(&self, command: &CommandBufferRef, frame: Frame<'_>) {
        let Frame {
            scene,
            camera,
            light,
            textures,
            output,
            counts,
        } = frame;
        let [right, up, forward] = camera.basis();
        let tan = (std::f32::consts::PI / 8.).tan();
        let width = output.width();
        let height = output.height();
        let constants = SceneCB {
            projection_to_world: Mat4::from_cols(
                (right * (width as f32 / height as f32) * tan).extend(0.),
                (up * tan).extend(0.),
                forward.extend(0.),
                (camera.eye + forward).extend(1.),
            )
            .to_cols_array(),
            camera: camera.eye.extend(1.).to_array(),
            light: light.extend(0.).to_array(),
            ambient: [0.8, 0.8, 0.8, 1.],
            diffuse: [1.2, 1., 1., 1.],
            sort_mode: 0,
            padding: [0; 3],
        };
        let frame = [width as u32, height as u32, 0, 0];
        let encoder = command.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&self.pipeline);
        encoder.set_bytes(
            self.slot("Cbv", 0),
            std::mem::size_of::<SceneCB>() as u64,
            (&constants as *const SceneCB).cast(),
        );
        encoder.set_bytes(self.slot("Cbv", 20), 16, frame.as_ptr().cast());
        encoder.set_acceleration_structure(self.slot("Srv", 0), Some(&scene.acceleration));
        for (register, buffer) in &self.buffers {
            encoder.set_buffer(self.slot("Srv", *register), Some(buffer), 0);
        }
        encoder.set_buffer(self.slot("Uav", 20), Some(counts), 0);
        encoder.set_texture(self.slot("Uav", 0), Some(output));
        for i in 0..2 {
            encoder.set_texture(self.slot("Srv", 9 + i), Some(&textures[i as usize]));
            encoder.set_sampler_state(self.slot("Sampler", i), Some(&self.sampler));
        }
        scene.use_resources(encoder);
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
    fn original_vertex_layout_indexed_cube_and_descriptor_zero_padding_are_explicit() {
        assert_eq!(std::mem::size_of::<SourceVertex>(), 36);
        assert_eq!(std::mem::size_of::<SceneCB>(), 144);
        let cube = crate::scene::cube();
        let trunk = Mesh {
            vertices: vec![cube.vertices[0]; 9],
        };
        let leaves = Mesh {
            vertices: vec![cube.vertices[1]; 3],
        };
        let bush = Mesh {
            vertices: vec![cube.vertices[2]; 6],
        };
        let buffers = source_buffers(&[cube.clone(), trunk, leaves, bush]).unwrap();
        let leaf = &buffers
            .iter()
            .find(|(register, _)| *register == 5)
            .unwrap()
            .1;
        assert_eq!(
            bytemuck::cast_slice::<u8, u32>(leaf),
            &[0, 1, 2, 0, 0, 0, 0, 0, 0]
        );
        let vertices = bytemuck::cast_slice::<u8, SourceVertex>(
            &buffers.iter().find(|(r, _)| *r == 2).unwrap().1,
        );
        for (i, index) in CUBE_INDICES.iter().enumerate() {
            assert_eq!(
                vertices[*index as usize],
                SourceVertex::from(cube.vertices[i])
            );
        }
        assert!(source_buffers(&[]).is_err());
        let mut bad = cube.clone();
        bad.vertices[5].position[0] += 1.;
        assert!(source_buffers(&[bad, cube.clone(), cube.clone(), cube]).is_err());
    }
}
