//! Independent CPU triangle intersections versus real Metal queries of the complete scene.
//! This checks host/GPU geometry and nearest-hit agreement, not native Windows/DXR parity.
use alkyhol_d3d12::{
    D3D12CreateDevice, complete,
    raytrace::{TriangleInstance, build_triangle_scene},
};
use alkyhol_sakura::scene::{Camera, Mesh, instances, load_meshes};
use glam::Vec3;
use metal::*;

struct Bounds {
    min: Vec3,
    max: Vec3,
}
impl Bounds {
    fn mesh(mesh: &Mesh) -> Self {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for v in &mesh.vertices {
            let p = Vec3::from_array(v.position);
            min = min.min(p);
            max = max.max(p);
        }
        Self { min, max }
    }
    fn crosses(&self, origin: Vec3, direction: Vec3, limit: f32) -> bool {
        let mut near = 0.001f32;
        let mut far = limit;
        for axis in 0..3 {
            if direction[axis].abs() < 1e-12 {
                if origin[axis] < self.min[axis] || origin[axis] > self.max[axis] {
                    return false;
                }
            } else {
                let a = (self.min[axis] - origin[axis]) / direction[axis];
                let b = (self.max[axis] - origin[axis]) / direction[axis];
                near = near.max(a.min(b));
                far = far.min(a.max(b));
                if near > far {
                    return false;
                }
            }
        }
        true
    }
}
fn triangle(origin: Vec3, direction: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    let e1 = b - a;
    let e2 = c - a;
    let p = direction.cross(e2);
    let determinant = e1.dot(p);
    if determinant.abs() < 1e-12 {
        return None;
    }
    let inverse = 1. / determinant;
    let offset = origin - a;
    let u = offset.dot(p) * inverse;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = offset.cross(e1);
    let v = direction.dot(q) * inverse;
    if v < 0. || u + v > 1. {
        return None;
    }
    let distance = e2.dot(q) * inverse;
    (0.001..10000.).contains(&distance).then_some(distance)
}

#[test]
#[ignore = "requires physical Metal RT and verified full scene; run --include-ignored"]
fn full_scene_gpu_nearest_hits_agree_with_independent_cpu_triangles() {
    objc::rc::autoreleasepool(|| {
        let device = pollster::block_on(D3D12CreateDevice(false)).unwrap();
        assert!(device.adapter_info().ray_tracing);
        eprintln!("Geometry audit adapter: {:?}", device.adapter_info());
        let assets =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../local/sakura/assets");
        let meshes = load_meshes(&assets).unwrap();
        let placement = instances();
        let bounds = meshes.iter().map(Bounds::mesh).collect::<Vec<_>>();
        let positions = meshes
            .iter()
            .map(|m| m.vertices.iter().map(|v| v.position).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let slices = positions.iter().map(|v| v.as_slice()).collect::<Vec<_>>();
        let descriptors = placement
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
        let scene = build_triangle_scene(&device, &slices, &descriptors).unwrap();
        let camera = Camera::default();
        let [right, up, forward] = camera.basis();
        let mut rays = Vec::new();
        // Pixel centers across the actual default view; no hand-selected expected-hit IDs.
        for y in [35, 100, 180, 270, 350, 440] {
            for x in [71, 215, 393, 567, 745, 889] {
                let sx = ((x as f32 + 0.5) / 960. * 2. - 1.)
                    * (960. / 540.)
                    * (std::f32::consts::PI / 8.).tan();
                let sy = -((y as f32 + 0.5) / 540. * 2. - 1.) * (std::f32::consts::PI / 8.).tan();
                let direction = (forward + right * sx + up * sy).normalize();
                rays.push([
                    camera.eye.x,
                    camera.eye.y,
                    camera.eye.z,
                    0.,
                    direction.x,
                    direction.y,
                    direction.z,
                    0.,
                ]);
            }
        }
        // Look directly toward every nearby trunk's bounds center at eye height. The CPU
        // oracle still considers all occluders; it does not assume the target wins.
        for instance in &placement[1402..2363] {
            let position = Vec3::from_array(instance.position);
            if (position.xz() - camera.eye.xz()).length() < 5. {
                let center = (bounds[1].min + bounds[1].max) * 0.5 * instance.scale + position;
                let direction = (Vec3::new(center.x, 2.3, center.z) - camera.eye).normalize();
                rays.push([
                    camera.eye.x,
                    camera.eye.y,
                    camera.eye.z,
                    0.,
                    direction.x,
                    direction.y,
                    direction.z,
                    0.,
                ]);
            }
        }
        let native = device.metal_device();
        let buffer = native.new_buffer_with_data(
            rays.as_ptr().cast(),
            std::mem::size_of_val(rays.as_slice()) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        let output = native.new_buffer(
            rays.len() as u64 * 16,
            MTLResourceOptions::StorageModeShared,
        );
        let source = r#"
#include <metal_stdlib>
#include <metal_raytracing>
using namespace metal;using namespace raytracing;
struct ProbeRay {float4 origin;float4 direction;};
kernel void trace_geometry(instance_acceleration_structure scene [[buffer(0)]],device const ProbeRay* rays [[buffer(1)]],device float4* out [[buffer(2)]],uint id [[thread_position_in_grid]]) {
 ray r;r.origin=rays[id].origin.xyz;r.direction=rays[id].direction.xyz;r.min_distance=.001f;r.max_distance=10000;
 intersector<triangle_data,instancing> tracer;tracer.assume_geometry_type(geometry_type::triangle);tracer.force_opacity(forced_opacity::opaque);
 auto hit=tracer.intersect(r,scene,1);
 out[id]=hit.type==intersection_type::none?float4(-1):float4(hit.distance,float(hit.instance_id),float(hit.primitive_id),1);
}
"#;
        let options = CompileOptions::new();
        options.set_fast_math_enabled(false);
        let library = native.new_library_with_source(source, &options).unwrap();
        let function = library.get_function("trace_geometry", None).unwrap();
        let pipeline = native
            .new_compute_pipeline_state_with_function(&function)
            .unwrap();
        let command = device.metal_queue().new_command_buffer();
        let encoder = command.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&pipeline);
        encoder.set_acceleration_structure(0, Some(&scene.acceleration));
        encoder.set_buffer(1, Some(&buffer), 0);
        encoder.set_buffer(2, Some(&output), 0);
        scene.use_resources(encoder);
        encoder.dispatch_threads(
            MTLSize::new(rays.len() as u64, 1, 1),
            MTLSize::new(32, 1, 1),
        );
        encoder.end_encoding();
        complete(command).unwrap();
        let hits =
            unsafe { std::slice::from_raw_parts(output.contents().cast::<[f32; 4]>(), rays.len()) };
        let mut material_counts = [0u32; 6];
        for (ray_index, (ray, gpu)) in rays.iter().zip(hits).enumerate() {
            let origin = Vec3::new(ray[0], ray[1], ray[2]);
            let direction = Vec3::new(ray[4], ray[5], ray[6]);
            let mut closest = 10000f32;
            let mut closest_id = None;
            for (id, instance) in placement.iter().enumerate() {
                let local_origin = (origin - Vec3::from_array(instance.position)) / instance.scale;
                let local_direction = direction / instance.scale;
                if !bounds[instance.mesh as usize].crosses(local_origin, local_direction, closest) {
                    continue;
                }
                for vertices in meshes[instance.mesh as usize].vertices.as_chunks::<3>().0 {
                    if let Some(distance) = triangle(
                        local_origin,
                        local_direction,
                        Vec3::from_array(vertices[0].position),
                        Vec3::from_array(vertices[1].position),
                        Vec3::from_array(vertices[2].position),
                    ) && distance < closest
                    {
                        closest = distance;
                        closest_id = Some(id);
                    }
                }
            }
            let Some(cpu_id) = closest_id else {
                assert_eq!(*gpu, [-1.; 4], "GPU false hit on ray {ray_index}");
                material_counts[5] += 1;
                continue;
            };
            assert_eq!(
                gpu[3], 1.,
                "GPU missed CPU hit at t={closest}, instance={cpu_id}, ray={ray_index}"
            );
            let tolerance = 0.0002f32.max(closest * 0.00002);
            assert!(
                (gpu[0] - closest).abs() <= tolerance,
                "nearest hit mismatch ray={ray_index}, GPU={gpu:?}, CPU t={closest}, id={cpu_id}"
            );
            let gpu_id = gpu[1] as usize;
            let primitive = gpu[2] as usize;
            let instance = placement[gpu_id];
            material_counts[instance.material as usize] += 1;
            // Coincident primitives/instances can choose different IDs. Verify the actual
            // returned triangle independently and require it to lie at the CPU minimum.
            let start = primitive * 3;
            let v = &meshes[instance.mesh as usize].vertices[start..start + 3];
            let local_origin = (origin - Vec3::from_array(instance.position)) / instance.scale;
            let local_direction = direction / instance.scale;
            let reported = triangle(
                local_origin,
                local_direction,
                Vec3::from_array(v[0].position),
                Vec3::from_array(v[1].position),
                Vec3::from_array(v[2].position),
            )
            .expect("GPU returned a triangle not intersected by this ray");
            assert!(
                (reported - closest).abs() <= tolerance,
                "returned triangle is behind CPU nearest"
            );
        }
        eprintln!(
            "{} full-scene rays agreed with independent CPU nearest triangles; material counts {material_counts:?}",
            rays.len()
        );
        assert!(
            material_counts[2] >= 5,
            "audit must actually hit multiple trunks"
        );
        assert!(material_counts[3] > 0);
        assert!(material_counts[0] > 0);
    });
}

use glam::Vec3Swizzles;
