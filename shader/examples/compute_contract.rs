//! Observe the same DXIL compute container executed by the native Windows probe.
#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use alkyhol_d3d12::{D3D12CreateDevice, complete};
    use alkyhol_shader::{RegisterClass, Stage, translate_dxil};
    use metal::*;
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 {
        return Err("usage: compute_contract SHADER.dxil".into());
    }
    let shader = translate_dxil(&std::fs::read(&args[0])?, "CSMain")?;
    if shader.stage != Stage::Compute
        || shader.threads_per_group != Some([8, 1, 1])
        || shader.resources.len() != 4
        || shader.requires_ray_query
    {
        return Err("unexpected compute contract interface".into());
    }
    let host = pollster::block_on(D3D12CreateDevice(false))?;
    eprintln!("adapter: translated Metal {:?}", host.adapter_info());
    let device = host.metal_device();
    let options = CompileOptions::new();
    options.set_fast_math_enabled(false);
    let library = device.new_library_with_source(&shader.msl, &options)?;
    let function = library.get_function(&shader.entry_point, None)?;
    let pipeline = device.new_compute_pipeline_state_with_function(&function)?;
    let count = 19u32;
    let mut settings = vec![count, 0x12345678, 2f32.to_bits(), 0.5f32.to_bits()];
    settings.extend([1f32, 1., 1., 1., 2., 3., 4., 5., -1., 2., -3., 4.].map(f32::to_bits));
    let a: Vec<_> = (0..count).map(|i| 0xfffffff0u32.wrapping_add(i)).collect();
    let b: Vec<_> = (0..count).map(|i| 3 * i + 7).collect();
    let buffers: Vec<_> = [settings, a, b, vec![0xdeadbeef; 24 * 4]]
        .iter()
        .map(|words| {
            device.new_buffer_with_data(
                words.as_ptr().cast(),
                std::mem::size_of_val(words.as_slice()) as u64,
                MTLResourceOptions::StorageModeShared,
            )
        })
        .collect();
    let command = host.metal_queue().new_command_buffer();
    let encoder = command.new_compute_command_encoder();
    encoder.set_compute_pipeline_state(&pipeline);
    for ((class, space), buffer) in [
        (RegisterClass::Cbv, 0),
        (RegisterClass::Srv, 0),
        (RegisterClass::Srv, 2),
        (RegisterClass::Uav, 0),
    ]
    .into_iter()
    .zip(&buffers)
    {
        let binding = shader
            .resources
            .iter()
            .find(|r| r.class == class && r.space == space && r.register == 0 && r.count == 1)
            .ok_or("missing compute binding")?;
        encoder.set_buffer(binding.metal_index as u64, Some(buffer), 0);
    }
    encoder.dispatch_thread_groups(MTLSize::new(3, 1, 1), MTLSize::new(8, 1, 1));
    encoder.end_encoding();
    complete(command)?;
    let words = unsafe { std::slice::from_raw_parts(buffers[3].contents().cast::<u32>(), 96) };
    println!(
        "{}",
        serde_json::json!({"scenario":"dxil-compute-v1", "count":count, "output_words":words})
    );
    Ok(())
}
#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("compute_contract requires native Metal on macOS");
    std::process::exit(1);
}
