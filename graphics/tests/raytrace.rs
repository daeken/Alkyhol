use alkyhol_d3d12::{D3D12CreateDevice, complete, raytrace::*};
use metal::*;
#[test]
#[ignore = "requires physical ray-tracing-capable Metal device; run --include-ignored"]
fn native_triangle_hits_misses_transforms_barycentrics_masks_and_invalid_inputs() {
    objc::rc::autoreleasepool(|| {
        let device = pollster::block_on(D3D12CreateDevice(false)).unwrap();
        eprintln!("ray tracing adapter: {:?}", device.adapter_info());
        assert!(
            device.adapter_info().ray_tracing,
            "native ray tracing is required for this test"
        );
        let mesh = [[-1., -1., 0.], [1., -1., 0.], [0., 1., 0.]];
        let mut instance = TriangleInstance {
            mesh: 0,
            transform: [[2., 0., 0.], [0., 2., 0.], [0., 0., 2.], [3., 0., 5.]],
            mask: 1,
        };
        for bad in [&[][..], &mesh[..2], &[[f32::NAN, 0., 0.]; 3]] {
            assert!(build_triangle_scene(&device, &[bad], &[instance]).is_err());
        }
        assert!(build_triangle_scene(&device, &[&mesh], &[]).is_err());
        instance.mesh = 1;
        assert!(build_triangle_scene(&device, &[&mesh], &[instance]).is_err());
        instance.mesh = 0;
        let mut bad = instance;
        bad.transform[0] = [0.; 3];
        assert!(build_triangle_scene(&device, &[&mesh], &[bad]).is_err());
        let scene = build_triangle_scene(&device, &[&mesh], &[instance]).unwrap();
        let native = device.metal_device();
        let library=native.new_library_with_source(r#"
#include <metal_stdlib>
#include <metal_raytracing>
using namespace metal; using namespace raytracing;
kernel void test_hits(instance_acceleration_structure scene [[buffer(0)]],device float4 *out [[buffer(1)]],uint id [[thread_position_in_grid]]) {
    ray r; r.origin=float3(id==1 ? 30.0f : 3.0f,0,0);r.direction=float3(0,0,1);r.min_distance=0.001;r.max_distance=100;
    intersector<triangle_data,instancing> tracer;tracer.assume_geometry_type(geometry_type::triangle);tracer.force_opacity(forced_opacity::opaque);
    auto hit=tracer.intersect(r,scene,id==2?2:1);
    out[id]=hit.type==intersection_type::none ? float4(-1) : float4(hit.distance,hit.triangle_barycentric_coord,float(hit.instance_id));
}
"#,&CompileOptions::new()).unwrap();
        let pipeline = native
            .new_compute_pipeline_state_with_function(
                &library.get_function("test_hits", None).unwrap(),
            )
            .unwrap();
        let output = native.new_buffer(48, MTLResourceOptions::StorageModeShared);
        let command = device.metal_queue().new_command_buffer();
        let encoder = command.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&pipeline);
        encoder.set_acceleration_structure(0, Some(&scene.acceleration));
        encoder.set_buffer(1, Some(&output), 0);
        scene.use_resources(encoder);
        encoder.dispatch_threads(MTLSize::new(3, 1, 1), MTLSize::new(3, 1, 1));
        encoder.end_encoding();
        complete(command).unwrap();
        let values = unsafe { std::slice::from_raw_parts(output.contents().cast::<f32>(), 12) };
        assert!((values[0] - 5.).abs() < 1e-5);
        assert!((values[1] - 0.25).abs() < 1e-5);
        assert!((values[2] - 0.5).abs() < 1e-5);
        assert_eq!(values[3], 0.);
        assert_eq!(&values[4..], &[-1.; 8]);
    });
}
