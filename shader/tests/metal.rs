#![cfg(target_os = "macos")]
use alkyhol_d3d12::{D3D12CreateDevice, complete};
use alkyhol_shader::*;
use metal::*;
use std::path::{Path, PathBuf};

fn translate(file: &str, entry: &str, stage: Stage) -> Translation {
    let dxc = std::env::var_os("DXC")
        .map(PathBuf::from)
        .unwrap_or_else(|| "dxc".into());
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(file);
    translate_dxil(&compile_hlsl(&dxc, &path, entry, stage).unwrap(), entry).unwrap()
}
fn slot(shader: &Translation, class: RegisterClass, space: u32) -> u64 {
    shader
        .resources
        .iter()
        .find(|r| r.class == class && r.space == space && r.register == 0)
        .expect("reflected binding")
        .metal_index as u64
}
fn buffer(device: &DeviceRef, words: &[u32]) -> Buffer {
    device.new_buffer_with_data(
        words.as_ptr().cast(),
        std::mem::size_of_val(words) as u64,
        MTLResourceOptions::StorageModeShared,
    )
}
fn pipeline(device: &DeviceRef, shader: &Translation) -> ComputePipelineState {
    let options = CompileOptions::new();
    options.set_fast_math_enabled(false);
    let library = device
        .new_library_with_source(&shader.msl, &options)
        .expect("translated MSL must compile");
    device
        .new_compute_pipeline_state_with_function(
            &library.get_function(&shader.entry_point, None).unwrap(),
        )
        .unwrap()
}
#[test]
fn translated_compute_executes_matrix_layout_barriers_spaces_and_wraparound() {
    let host = pollster::block_on(D3D12CreateDevice(false)).unwrap();
    eprintln!("physical/hosted identity: {:?}", host.adapter_info());
    let device = host.metal_device();
    let shader = translate("compute.hlsl", "CSMain", Stage::Compute);
    let pso = pipeline(device, &shader);
    let count = 19u32;
    let salt = 0x12345678u32;
    let mut settings = vec![count, salt, 2f32.to_bits(), 0.5f32.to_bits()];
    settings.extend([1f32, 1., 1., 1., 2., 3., 4., 5., -1., 2., -3., 4.].map(f32::to_bits));
    let a: Vec<u32> = (0..count).map(|i| 0xfffffff0u32.wrapping_add(i)).collect();
    let b: Vec<u32> = (0..count).map(|i| 3 * i + 7).collect();
    let cb = buffer(device, &settings);
    let aa = buffer(device, &a);
    let bb = buffer(device, &b);
    // Sentinel tail detects writes beyond the shader's count.
    let output = buffer(device, &vec![0xdeadbeef; (count as usize + 5) * 4]);
    let command = host.metal_queue().new_command_buffer();
    let encoder = command.new_compute_command_encoder();
    encoder.set_compute_pipeline_state(&pso);
    for (class, space, data) in [
        (RegisterClass::Cbv, 0, &cb),
        (RegisterClass::Srv, 0, &aa),
        (RegisterClass::Srv, 2, &bb),
        (RegisterClass::Uav, 0, &output),
    ] {
        encoder.set_buffer(slot(&shader, class, space), Some(data), 0);
    }
    let [x, y, z] = shader.threads_per_group.unwrap();
    encoder.dispatch_thread_groups(
        MTLSize::new(count.div_ceil(x) as u64, 1, 1),
        MTLSize::new(x as u64, y as u64, z as u64),
    );
    encoder.end_encoding();
    complete(command).unwrap();
    let actual = unsafe {
        std::slice::from_raw_parts(output.contents().cast::<u32>(), (count as usize + 5) * 4)
    };
    for i in 0..count {
        let source = i / 8 * 8 + 7 - i % 8;
        let av = if source < count {
            a[source as usize]
        } else {
            0
        };
        let bv = b[i as usize];
        assert_eq!(
            &actual[i as usize * 4..i as usize * 4 + 4],
            &[
                av.wrapping_add(salt),
                (bv << (i & 7)) ^ salt,
                (4. * i as f32 + 22.).to_bits(),
                if i & 1 != 0 { bv } else { av }
            ],
            "element {i}"
        );
    }
    assert!(
        actual[count as usize * 4..]
            .iter()
            .all(|&v| v == 0xdeadbeef)
    );
}

#[test]
fn translated_texture_sampler_and_uav_use_reflected_independent_namespaces() {
    let host = pollster::block_on(D3D12CreateDevice(false)).unwrap();
    let device = host.metal_device();
    let shader = translate("texture.hlsl", "CSMain", Stage::Compute);
    let pso = pipeline(device, &shader);
    let desc = TextureDescriptor::new();
    desc.set_texture_type(MTLTextureType::D2);
    desc.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
    desc.set_width(2);
    desc.set_height(2);
    desc.set_usage(MTLTextureUsage::ShaderRead);
    desc.set_storage_mode(MTLStorageMode::Shared);
    let texture = device.new_texture(&desc);
    let pixels: [u8; 16] = [
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
    ];
    texture.replace_region(MTLRegion::new_2d(0, 0, 2, 2), 0, pixels.as_ptr().cast(), 8);
    let sampler_desc = SamplerDescriptor::new();
    sampler_desc.set_min_filter(MTLSamplerMinMagFilter::Nearest);
    sampler_desc.set_mag_filter(MTLSamplerMinMagFilter::Nearest);
    let sampler = device.new_sampler(&sampler_desc);
    let output = buffer(device, &[0xdeadbeef; 16]);
    let command = host.metal_queue().new_command_buffer();
    let encoder = command.new_compute_command_encoder();
    encoder.set_compute_pipeline_state(&pso);
    encoder.set_buffer(slot(&shader, RegisterClass::Uav, 0), Some(&output), 0);
    encoder.set_texture(slot(&shader, RegisterClass::Srv, 0), Some(&texture));
    encoder.set_sampler_state(slot(&shader, RegisterClass::Sampler, 0), Some(&sampler));
    encoder.dispatch_thread_groups(MTLSize::new(1, 1, 1), MTLSize::new(2, 2, 1));
    encoder.end_encoding();
    complete(command).unwrap();
    let actual = unsafe { std::slice::from_raw_parts(output.contents().cast::<f32>(), 16) };
    for (i, (&got, &byte)) in actual.iter().zip(&pixels).enumerate() {
        assert_eq!(got, byte as f32 / 255., "channel {i}");
    }
}

#[test]
fn translated_vertex_pixel_pair_matches_unchanged_checker_oracle() {
    let pixels = render_pair("checker.hlsl", false);
    for y in 0..4usize {
        for x in 0..65usize {
            assert_eq!(
                &pixels[(y * 65 + x) * 4..(y * 65 + x + 1) * 4],
                &[
                    ((x & 1) * 255) as u8,
                    ((y & 1) * 255) as u8,
                    ((((x >> 1) + y) & 1) * 255) as u8,
                    255
                ],
                "pixel {x},{y}"
            );
        }
    }
}

#[test]
fn original_microsoft_shaders_render_interpolated_vertex_colours() {
    let pixels = render_pair("microsoft-hello-triangle.hlsl", true);
    for y in 0..4usize {
        for x in 0..65usize {
            let green = 0.5 - (y as f32 + 0.5) / 8.;
            let blue = (x as f32 + 0.5) / 130.;
            let expected = [1. - green - blue, green, blue, 1.];
            for c in 0..4 {
                assert!(
                    (pixels[(y * 65 + x) * 4 + c] as f32 - expected[c] * 255.).abs() <= 1.,
                    "interpolated pixel {x},{y} channel {c}"
                );
            }
        }
    }
}

fn render_pair(file: &str, with_vertices: bool) -> Vec<u8> {
    let host = pollster::block_on(D3D12CreateDevice(false)).unwrap();
    let device = host.metal_device();
    let vs = translate(file, "VSMain", Stage::Vertex);
    let ps = translate(file, "PSMain", Stage::Pixel);
    let options = CompileOptions::new();
    options.set_fast_math_enabled(false);
    let vl = device.new_library_with_source(&vs.msl, &options).unwrap();
    let pl = device.new_library_with_source(&ps.msl, &options).unwrap();
    let descriptor = RenderPipelineDescriptor::new();
    descriptor.set_vertex_function(Some(&vl.get_function(&vs.entry_point, None).unwrap()));
    descriptor.set_fragment_function(Some(&pl.get_function(&ps.entry_point, None).unwrap()));
    let vertices = if with_vertices {
        assert_eq!(vs.vertex_inputs.len(), 2);
        let layout = VertexDescriptor::new();
        for input in &vs.vertex_inputs {
            assert_eq!(input.rows, 1);
            let attribute = layout
                .attributes()
                .object_at(input.location as u64)
                .unwrap();
            attribute.set_format(MTLVertexFormat::Float4);
            attribute.set_buffer_index(24);
            attribute.set_offset(match input.semantic.as_str() {
                "POSITION" => 0,
                "COLOR" => 16,
                other => panic!("unexpected semantic {other}"),
            });
        }
        layout.layouts().object_at(24).unwrap().set_stride(32);
        descriptor.set_vertex_descriptor(Some(layout));
        let data: [f32; 24] = [
            -1., -1., 0., 1., 1., 0., 0., 1., -1., 3., 0., 1., 0., 1., 0., 1., 3., -1., 0., 1., 0.,
            0., 1., 1.,
        ];
        Some(device.new_buffer_with_data(
            data.as_ptr().cast(),
            96,
            MTLResourceOptions::StorageModeShared,
        ))
    } else {
        None
    };
    descriptor
        .color_attachments()
        .object_at(0)
        .unwrap()
        .set_pixel_format(MTLPixelFormat::RGBA8Unorm);
    let pso = device.new_render_pipeline_state(&descriptor).unwrap();
    let td = TextureDescriptor::new();
    td.set_texture_type(MTLTextureType::D2);
    td.set_width(65);
    td.set_height(4);
    td.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
    td.set_storage_mode(MTLStorageMode::Shared);
    td.set_usage(MTLTextureUsage::RenderTarget);
    let texture = device.new_texture(&td);
    let pass = RenderPassDescriptor::new();
    let attachment = pass.color_attachments().object_at(0).unwrap();
    attachment.set_texture(Some(&texture));
    attachment.set_load_action(MTLLoadAction::Clear);
    attachment.set_clear_color(MTLClearColor::new(0., 1., 0., 1.));
    attachment.set_store_action(MTLStoreAction::Store);
    let command = host.metal_queue().new_command_buffer();
    let encoder = command.new_render_command_encoder(pass);
    encoder.set_render_pipeline_state(&pso);
    if let Some(vertices) = &vertices {
        encoder.set_vertex_buffer(24, Some(vertices), 0);
    }
    encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, 3);
    encoder.end_encoding();
    complete(command).unwrap();
    let mut pixels = vec![0u8; 65 * 4 * 4];
    texture.get_bytes(
        pixels.as_mut_ptr().cast(),
        65 * 4,
        MTLRegion::new_2d(0, 0, 65, 4),
        0,
    );
    pixels
}
