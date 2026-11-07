#![cfg(target_os = "macos")]
use alkyhol_d3d12::{D3D12CreateDevice, complete};
use metal::*;
#[test]
fn original_sakura_helpers_and_exhausted_budget_are_observed_on_metal() {
    let shader: serde_json::Value =
        serde_json::from_str(include_str!("../samples/sakura/units.generated.json")).unwrap();
    assert_eq!(shader["requires_ray_query"], false);
    assert_eq!(shader["threads_per_group"], serde_json::json!([1, 1, 1]));
    let resources = shader["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 2);
    let slot = |register: u32| {
        resources
            .iter()
            .find(|r| {
                r["class"] == "Uav"
                    && r["register"] == register
                    && r["space"] == 0
                    && r["namespace"] == "Buffer"
            })
            .unwrap()["metal_index"]
            .as_u64()
            .unwrap()
    };
    let host = pollster::block_on(D3D12CreateDevice(false)).unwrap();
    eprintln!("original Sakura helper adapter: {:?}", host.adapter_info());
    let device = host.metal_device();
    let options = CompileOptions::new();
    options.set_fast_math_enabled(false);
    let library = device
        .new_library_with_source(shader["msl"].as_str().unwrap(), &options)
        .unwrap();
    let function = library
        .get_function(shader["entry_point"].as_str().unwrap(), None)
        .unwrap();
    let pipeline = device
        .new_compute_pipeline_state_with_function(&function)
        .unwrap();
    let counts = device.new_buffer_with_data(
        [0u32; 12].as_ptr().cast(),
        48,
        MTLResourceOptions::StorageModeShared,
    );
    let output = device.new_buffer_with_data(
        [0xdeadbeefu32; 28].as_ptr().cast(),
        112,
        MTLResourceOptions::StorageModeShared,
    );
    let command = host.metal_queue().new_command_buffer();
    let encoder = command.new_compute_command_encoder();
    encoder.set_compute_pipeline_state(&pipeline);
    encoder.set_buffer(slot(20), Some(&counts), 0);
    encoder.set_buffer(slot(30), Some(&output), 0);
    encoder.dispatch_thread_groups(MTLSize::new(7, 1, 1), MTLSize::new(1, 1, 1));
    encoder.end_encoding();
    complete(command).unwrap();
    let data = unsafe { std::slice::from_raw_parts(output.contents().cast::<[f32; 4]>(), 7) };
    let expected = [
        [0.2f32, 0.4, 0.8, 1.],
        [1., 1., 1., 1.],
        [0.5, 0.2, 0.3, 1.],
        [0.4, 0.35, 0., 1.],
        [1., 2., 1., 1.],
        [1., 3., 0., 0.],
    ];
    for (i, want) in expected.iter().enumerate() {
        for c in 0..4 {
            assert!(
                (data[i][c] - want[c]).abs() < 1e-6,
                "fixture {i} channel {c}: {:?}",
                data[i]
            );
        }
    }
    let counts = unsafe { *(counts.contents().cast::<[u32; 12]>()) };
    assert_eq!(counts[10], 1);
    assert!(counts[..10].iter().all(|&v| v == 0));
    assert_eq!(counts[11], 0);
    assert!(
        data[6][..3]
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
    );
    eprintln!(
        "Original 200×50 HLSL Star Nest observation: {:?}; no host/Windows equality claim",
        data[6]
    );
}
