//! Runtime for the generated original-HLSL barycentric pipeline. No shader math
//! or binding numbers are duplicated here; slots come from translation metadata.
use crate::camera::Camera;
use alkyhol_d3d12::raytrace::TriangleScene;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use metal::*;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct DynamicCB {
    camera_to_world: [f32; 16],
    eye: [f32; 4],
    resolution: [f32; 2],
    padding: [f32; 2],
}
pub(crate) struct BaryPipeline {
    pipeline: ComputePipelineState,
    slots: [u64; 10],
}
pub(crate) struct BaryFrame<'a> {
    pub scene: &'a TriangleScene,
    pub camera: &'a Camera,
    pub output: &'a TextureRef,
    pub counts: &'a BufferRef,
    pub packets: &'a BufferRef,
    pub depth: &'a TextureRef,
    pub normals: &'a TextureRef,
}
pub(crate) struct CameraPipeline {
    pipeline: ComputePipelineState,
    slots: [u64; 2],
}
impl CameraPipeline {
    pub(crate) fn new(device: &DeviceRef) -> Result<Self, String> {
        let data: serde_json::Value = serde_json::from_str(include_str!(
            "../../shader/samples/miniengine/camera.generated.json"
        ))
        .map_err(|e| e.to_string())?;
        if data["stage"] != "Compute"
            || data["requires_ray_query"] != false
            || data["threads_per_group"] != serde_json::json!([8, 8, 1])
        {
            return Err("unexpected camera shader contract".into());
        }
        let resources = data["resources"]
            .as_array()
            .ok_or("camera bindings missing")?;
        if resources.len() != 2 {
            return Err("unexpected camera bindings".into());
        }
        let mut slots = [0; 2];
        for (i, (class, register)) in [("Cbv", 1), ("Uav", 20)].into_iter().enumerate() {
            let matches: Vec<_> = resources
                .iter()
                .filter(|r| {
                    r["class"] == class
                        && r["register"] == register
                        && r["space"] == 0
                        && r["namespace"] == "Buffer"
                })
                .collect();
            if matches.len() != 1 {
                return Err("camera binding missing/duplicated".into());
            }
            slots[i] = matches[0]["metal_index"]
                .as_u64()
                .filter(|&v| v < 24)
                .ok_or("invalid camera buffer slot")?;
        }
        if slots[0] == slots[1] {
            return Err("camera buffers alias".into());
        }
        let options = CompileOptions::new();
        options.set_fast_math_enabled(false);
        let library = device
            .new_library_with_source(data["msl"].as_str().ok_or("camera MSL missing")?, &options)?;
        let function = library.get_function(
            data["entry_point"].as_str().ok_or("camera entry missing")?,
            None,
        )?;
        Ok(Self {
            pipeline: device.new_compute_pipeline_state_with_function(&function)?,
            slots,
        })
    }
    pub(crate) fn encode(
        &self,
        command: &CommandBufferRef,
        camera: &Camera,
        width: u64,
        height: u64,
        rays: &BufferRef,
    ) {
        let dynamic = dynamic_constants(camera, width, height);
        let encoder = command.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&self.pipeline);
        encoder.set_bytes(
            self.slots[0],
            std::mem::size_of::<DynamicCB>() as u64,
            (&dynamic as *const DynamicCB).cast(),
        );
        encoder.set_buffer(self.slots[1], Some(rays), 0);
        encoder.dispatch_thread_groups(
            MTLSize::new(width.div_ceil(8), height.div_ceil(8), 1),
            MTLSize::new(8, 8, 1),
        );
        encoder.end_encoding();
    }
}
pub(crate) fn view_projection(camera: &Camera, width: u64, height: u64) -> Mat4 {
    let [right, up, forward] = camera.basis();
    let y = 1. / (std::f32::consts::PI / 8.).tan();
    let x = y * height as f32 / width as f32;
    // Pinned MiniEngine Camera.cpp: reverse Z, finite range 1..10000.
    let q1 = 1. / (10000. - 1.);
    let q2 = q1 * 10000.;
    let projection = Mat4::from_cols(
        glam::Vec4::new(x, 0., 0., 0.),
        glam::Vec4::new(0., y, 0., 0.),
        glam::Vec4::new(0., 0., q1, -1.),
        glam::Vec4::new(0., 0., q2, 0.),
    );
    let camera_world = Mat4::from_cols(
        right.extend(0.),
        up.extend(0.),
        (-forward).extend(0.),
        camera.eye.extend(1.),
    );
    projection * camera_world.inverse()
}
pub(crate) fn dynamic_constants(camera: &Camera, width: u64, height: u64) -> DynamicCB {
    DynamicCB {
        camera_to_world: view_projection(camera, width, height)
            .inverse()
            .to_cols_array(),
        eye: camera.eye.extend(0.).to_array(),
        resolution: [width as f32, height as f32],
        padding: [0.; 2],
    }
}
fn metadata(source: &str) -> Result<(String, String, [u64; 10]), String> {
    let data: serde_json::Value = serde_json::from_str(source).map_err(|e| e.to_string())?;
    if data["stage"] != "Compute"
        || data["threads_per_group"] != serde_json::json!([8, 8, 1])
        || data["requires_ray_query"] != true
    {
        return Err("unexpected original bary shader contract".into());
    }
    let resources = data["resources"]
        .as_array()
        .ok_or("missing reflected resources")?;
    if resources.len() != 10 {
        return Err("unexpected original bary resources".into());
    }
    let mut slots = [0; 10];
    for (i, (class, register, namespace, kind)) in [
        ("Cbv", 0, "Buffer", "ConstantBuffer"),
        ("Cbv", 1, "Buffer", "ConstantBuffer"),
        ("Cbv", 20, "Buffer", "ConstantBuffer"),
        ("Cbv", 21, "Buffer", "ConstantBuffer"),
        ("Srv", 0, "Buffer", "RtAccelerationStructure"),
        ("Uav", 2, "Texture", "Texture2D"),
        ("Uav", 20, "Buffer", "StructuredBuffer"),
        ("Uav", 21, "Buffer", "StructuredBuffer"),
        ("Uav", 22, "Texture", "Texture2D"),
        ("Uav", 23, "Texture", "Texture2D"),
    ]
    .into_iter()
    .enumerate()
    {
        let matches: Vec<_> = resources
            .iter()
            .filter(|r| {
                r["class"] == class
                    && r["space"] == 0
                    && r["register"] == register
                    && r["namespace"] == namespace
                    && r["kind"] == kind
            })
            .collect();
        if matches.len() != 1 {
            return Err("missing/duplicate original bary binding".into());
        }
        slots[i] = matches[0]["metal_index"]
            .as_u64()
            .ok_or("invalid Metal slot")?;
        if slots[i] >= if namespace == "Texture" { 128 } else { 24 } {
            return Err("Metal slot exceeds the binding contract".into());
        }
    }
    for a in [0, 1, 2, 3, 4, 6, 7] {
        for b in [0, 1, 2, 3, 4, 6, 7] {
            if a != b && slots[a] == slots[b] {
                return Err("aliased original bary buffer bindings".into());
            }
        }
    }
    Ok((
        data["msl"].as_str().ok_or("missing generated MSL")?.into(),
        data["entry_point"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or("missing generated entry")?
            .into(),
        slots,
    ))
}
impl BaryPipeline {
    pub(crate) fn new(device: &DeviceRef) -> Result<Self, String> {
        let (source, entry, slots) = metadata(include_str!(
            "../../shader/samples/miniengine/bary.generated.json"
        ))?;
        let options = CompileOptions::new();
        options.set_fast_math_enabled(false);
        let library = device.new_library_with_source(&source, &options)?;
        let function = library.get_function(&entry, None)?;
        let pipeline = device.new_compute_pipeline_state_with_function(&function)?;
        Ok(Self { pipeline, slots })
    }
    pub(crate) fn encode(&self, command: &CommandBufferRef, frame: BaryFrame<'_>) {
        let BaryFrame {
            scene,
            camera,
            output,
            counts,
            packets,
            depth,
            normals,
        } = frame;
        let width = output.width();
        let height = output.height();
        let dynamic = dynamic_constants(camera, width, height);
        let constants = [0u32; 36]; // IsReflection=0 at byte 128; other fields are unused.
        let encoder = command.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&self.pipeline);
        encoder.set_bytes(self.slots[0], 144, constants.as_ptr().cast());
        encoder.set_bytes(
            self.slots[1],
            std::mem::size_of::<DynamicCB>() as u64,
            (&dynamic as *const DynamicCB).cast(),
        );
        let settings = [width as u32, height as u32, 0, 0];
        let projection = view_projection(camera, width, height).to_cols_array();
        encoder.set_bytes(self.slots[2], 16, settings.as_ptr().cast());
        encoder.set_bytes(self.slots[3], 64, projection.as_ptr().cast());
        encoder.set_acceleration_structure(self.slots[4], Some(&scene.acceleration));
        encoder.set_texture(self.slots[5], Some(output));
        encoder.set_buffer(self.slots[6], Some(counts), 0);
        encoder.set_buffer(self.slots[7], Some(packets), 0);
        encoder.set_texture(self.slots[8], Some(depth));
        encoder.set_texture(self.slots[9], Some(normals));
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
    fn reverse_z_projection_endpoints_and_world_round_trip() {
        for preset in 0..5 {
            let camera = Camera::preset(preset).unwrap();
            let [right, up, forward] = camera.basis();
            let matrix = view_projection(&camera, 129, 73);
            for (distance, expected) in [(1., 1.), (10000., 0.)] {
                let clip = matrix * (camera.eye + distance * forward).extend(1.);
                assert!((clip.z / clip.w - expected).abs() < 0.0003);
            }
            let near = matrix.project_point3(camera.eye + 10. * forward).z;
            let far = matrix.project_point3(camera.eye + 100. * forward).z;
            assert!(near > far && far > 0.);
            let world = camera.eye + 150. * forward + 12. * right - 7. * up;
            let ndc = matrix.project_point3(world);
            let restored = matrix.inverse().project_point3(ndc);
            assert!((restored - world).length() < 0.02, "preset {preset}");
        }
    }
    #[test]
    fn original_hlsl_ray_generation_uses_the_matrix_and_guards_padded_dispatch() {
        objc::rc::autoreleasepool(|| {
            let host = pollster::block_on(alkyhol_d3d12::D3D12CreateDevice(false)).unwrap();
            eprintln!("original raygen compute adapter: {:?}", host.adapter_info());
            let device = host.metal_device();
            let pipeline = CameraPipeline::new(device).unwrap();
            assert_eq!(std::mem::size_of::<DynamicCB>(), 96);
            let camera = Camera::preset(2).unwrap();
            let (width, height) = (13u64, 7u64);
            let words = vec![0xdecafbad_u32; (width * height * 8 + 64) as usize];
            let rays = device.new_buffer_with_data(
                words.as_ptr().cast(),
                std::mem::size_of_val(words.as_slice()) as u64,
                MTLResourceOptions::StorageModeShared,
            );
            let command = host.metal_queue().new_command_buffer();
            pipeline.encode(command, &camera, width, height, &rays);
            alkyhol_d3d12::complete(command).unwrap();
            let data =
                unsafe { std::slice::from_raw_parts(rays.contents().cast::<f32>(), words.len()) };
            let constants = dynamic_constants(&camera, width, height);
            let m = constants.camera_to_world;
            for y in 0..height {
                for x in 0..width {
                    let i = ((y * width + x) * 8) as usize;
                    assert_eq!(&data[i..i + 3], camera.eye.as_ref());
                    assert_eq!(data[i + 3], 0.);
                    assert_eq!(data[i + 7], f32::MAX);
                    let sx = ((x as f32 + 0.5) / width as f32) * 2. - 1.;
                    let sy = -(((y as f32 + 0.5) / height as f32) * 2. - 1.);
                    let mut direction = [0f32; 3];
                    for c in 0..3 {
                        direction[c] = (m[4 + c].mul_add(sy, m[c] * sx) + m[12 + c])
                            / (m[7].mul_add(sy, m[3] * sx) + m[15])
                            - constants.eye[c];
                    }
                    let length = direction.iter().map(|v| v * v).sum::<f32>().sqrt();
                    for c in 0..3 {
                        assert!(
                            (data[i + 4 + c] - direction[c] / length).abs() < 2e-6,
                            "ray {x},{y} component {c}"
                        );
                    }
                }
            }
            let tail = unsafe {
                std::slice::from_raw_parts(
                    rays.contents()
                        .cast::<u32>()
                        .add((width * height * 8) as usize),
                    64,
                )
            };
            assert!(tail.iter().all(|&v| v == 0xdecafbad));
        });
    }
    #[test]
    fn generated_contract_rejects_missing_aliasing_and_wrong_stage_metadata() {
        let source = include_str!("../../shader/samples/miniengine/bary.generated.json");
        assert!(metadata(source).is_ok());
        assert!(metadata("{}").is_err());
        let data: serde_json::Value = serde_json::from_str(source).unwrap();
        for mutate in [0, 1, 2] {
            let mut data = data.clone();
            match mutate {
                0 => data["stage"] = "Pixel".into(),
                1 => {
                    data["resources"][1]["metal_index"] =
                        data["resources"][0]["metal_index"].clone()
                }
                _ => data["resources"][0]["space"] = 1.into(),
            };
            assert!(metadata(&data.to_string()).is_err());
        }
    }
}
