use alkyhol_miniengine::dds::{self, Encoding};
fn put(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
fn fixture() -> Vec<u8> {
    let mut b = vec![0; 148 + 24];
    b[..4].copy_from_slice(b"DDS ");
    for (offset, value) in [
        (4, 124),
        (76, 32),
        (80, 4),
        (12, 4),
        (16, 4),
        (28, 3),
        (128, 72),
        (132, 3),
        (140, 1),
    ] {
        put(&mut b, offset, value);
    }
    b[84..88].copy_from_slice(b"DX10");
    for (level, colour) in [0xf800u16, 0x07e0, 0x001f].into_iter().enumerate() {
        b[148 + level * 8..150 + level * 8].copy_from_slice(&colour.to_le_bytes());
    }
    b
}
#[test]
fn dds_preserves_bc1_mip_dimensions_encoding_and_payload_and_rejects_invalid_data() {
    let good = fixture();
    let dds = dds::parse(&good).unwrap();
    assert_eq!(
        (dds.width, dds.height, dds.encoding),
        (4, 4, Encoding::Srgb)
    );
    assert_eq!(
        dds.mips
            .iter()
            .map(|m| (m.width, m.height, m.row_bytes, m.data.len()))
            .collect::<Vec<_>>(),
        [(4, 4, 8, 8), (2, 2, 8, 8), (1, 1, 8, 8)]
    );
    for n in 0..good.len() {
        assert!(dds::parse(&good[..n]).is_err(), "accepted prefix {n}");
    }
    for (offset, value) in [
        (4, 0),
        (76, 0),
        (80, 0),
        (12, 0),
        (16, 16385),
        (28, 4),
        (128, 74),
        (132, 4),
        (136, 4),
        (140, 2),
    ] {
        let mut bad = good.clone();
        put(&mut bad, offset, value);
        assert!(
            dds::parse(&bad).is_err(),
            "accepted invalid DDS field {offset}"
        );
    }
    let mut bad = good.clone();
    bad.push(0);
    assert!(dds::parse(&bad).is_err());
    let mut linear = good.clone();
    put(&mut linear, 128, 71);
    assert_eq!(dds::parse(&linear).unwrap().encoding, Encoding::Linear);
    let mut legacy = good[..128].to_vec();
    legacy[84..88].copy_from_slice(b"DXT1");
    legacy.extend_from_slice(&good[148..]);
    assert_eq!(dds::parse(&legacy).unwrap().encoding, Encoding::Linear);
}
#[test]
fn gpu_bc1_upload_samples_all_original_mips_and_explicit_engine_defaults() {
    use alkyhol_d3d12::{D3D12CreateDevice, complete};
    use metal::*;
    objc::rc::autoreleasepool(|| {
        let device = pollster::block_on(D3D12CreateDevice(false)).unwrap();
        eprintln!("DDS compute adapter: {:?}", device.adapter_info());
        let native = device.metal_device();
        let bytes = fixture();
        let texture = dds::upload(native, &bytes).unwrap();
        let flat = dds::solid(native, [128, 128, 255, 0]);
        assert_eq!(texture.mipmap_level_count(), 3);
        assert_eq!(texture.pixel_format(), MTLPixelFormat::BC1_RGBA_sRGB);
        assert!(dds::upload(native, b"not DDS").is_err());
        let source = r#"
#include <metal_stdlib>
using namespace metal;
kernel void sample_mips(texture2d<float> tex [[texture(0)]],texture2d<float> flat [[texture(1)]],device float4* out [[buffer(0)]],uint id [[thread_position_in_grid]]) {
 constexpr sampler s(coord::normalized,address::clamp_to_edge,filter::nearest,mip_filter::nearest);
 out[id]=id<3?tex.sample(s,float2(.5),level(id)):flat.sample(s,float2(.5),level(0));
}
"#;
        let library = native
            .new_library_with_source(source, &CompileOptions::new())
            .unwrap();
        let function = library.get_function("sample_mips", None).unwrap();
        let pipeline = native
            .new_compute_pipeline_state_with_function(&function)
            .unwrap();
        let output = native.new_buffer(64, MTLResourceOptions::StorageModeShared);
        let command = device.metal_queue().new_command_buffer();
        let encoder = command.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&pipeline);
        encoder.set_texture(0, Some(&texture));
        encoder.set_texture(1, Some(&flat));
        encoder.set_buffer(0, Some(&output), 0);
        encoder.dispatch_threads(MTLSize::new(4, 1, 1), MTLSize::new(4, 1, 1));
        encoder.end_encoding();
        complete(command).unwrap();
        let values = unsafe { std::slice::from_raw_parts(output.contents().cast::<[f32; 4]>(), 4) };
        for (actual, expected) in values.iter().zip([
            [1., 0., 0., 1.],
            [0., 1., 0., 1.],
            [0., 0., 1., 1.],
            [128. / 255., 128. / 255., 1., 0.],
        ]) {
            for (a, e) in actual.iter().zip(expected) {
                assert!((a - e).abs() < 1e-6, "{values:?}");
            }
        }
    });
}
#[test]
#[ignore = "requires 50 pinned DDS assets: python3 tools/fetch_miniengine.py"]
fn every_pinned_material_dds_is_a_complete_supported_mip_chain() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../local/miniengine/assets/Sponza");
    let mut count = 0;
    for entry in std::fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("dds"))
        {
            let bytes = std::fs::read(&path).unwrap();
            let texture = dds::parse(&bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            assert!(texture.mips.len() >= 7);
            count += 1;
        }
    }
    assert_eq!(count, 50);
}
