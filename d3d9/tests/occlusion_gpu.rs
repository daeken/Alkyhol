//! Real visibility counters, independent readback and overlapping scopes.
use alkyhol_d3d9::{Device, OcclusionQuery, RenderState};
use std::time::{Duration, Instant};
fn triangle() -> Vec<u8> {
    [(16f32, 2f32), (30., 30.), (2., 30.)]
        .into_iter()
        .flat_map(|(x, y)| {
            [
                x.to_le_bytes(),
                y.to_le_bytes(),
                0.5f32.to_le_bytes(),
                1f32.to_le_bytes(),
                0xffff0000u32.to_le_bytes(),
            ]
            .concat()
        })
        .collect()
}
fn wait(gpu: &Device, query: &OcclusionQuery) -> u32 {
    let start = Instant::now();
    loop {
        if let Some(value) = gpu.occlusion_data(query, true).unwrap() {
            return value;
        }
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "actual query did not complete"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[test]
#[ignore = "requires an actual GPU"]
fn occlusion_counts_actual_samples_across_overlap_reuse_and_depth_rejection() {
    let gpu = Device::new().unwrap();
    let mut target = gpu.create_target(32, 32).unwrap();
    let mut state = RenderState::default();
    let outer = gpu.create_occlusion_query();
    let inner = gpu.create_occlusion_query();
    assert!(gpu.occlusion_data(&outer, false).is_err());
    assert!(gpu.end_occlusion_query(&outer).is_err());
    gpu.clear(&mut target, 0xff0000ff).unwrap();
    gpu.begin_occlusion_query(&outer).unwrap();
    assert!(gpu.begin_occlusion_query(&outer).is_err());
    assert_eq!(gpu.occlusion_data(&outer, false).unwrap(), None);
    gpu.draw_transformed_depth_state(&mut target, &triangle(), &state, None)
        .unwrap();
    gpu.begin_occlusion_query(&inner).unwrap();
    gpu.draw_transformed_depth_state(&mut target, &triangle(), &state, None)
        .unwrap();
    gpu.end_occlusion_query(&inner).unwrap();
    gpu.end_occlusion_query(&outer).unwrap();
    let pixels = gpu.readback(&target).unwrap();
    let red = pixels
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|&&p| p == [0, 0, 255, 255])
        .count() as u32;
    assert!(red > 0);
    assert_eq!(wait(&gpu, &outer), 2 * red);
    assert_eq!(wait(&gpu, &inner), red);
    assert_eq!(gpu.occlusion_data(&outer, false).unwrap(), Some(2 * red));
    let depth = gpu.create_depth16(32, 32).unwrap();
    gpu.clear_depth(&depth, 0.).unwrap();
    state.set(7, 1).unwrap();
    gpu.begin_occlusion_query(&outer).unwrap();
    gpu.draw_transformed_depth_state(&mut target, &triangle(), &state, Some(&depth))
        .unwrap();
    gpu.end_occlusion_query(&outer).unwrap();
    assert_eq!(wait(&gpu, &outer), 0);
    gpu.begin_occlusion_query(&outer).unwrap();
    gpu.end_occlusion_query(&outer).unwrap();
    assert_eq!(wait(&gpu, &outer), 0);
    let other = Device::new().unwrap();
    assert!(other.begin_occlusion_query(&outer).is_err());
    assert!(other.end_occlusion_query(&outer).is_err());
    assert!(other.occlusion_data(&outer, true).is_err());
    let scopes: Vec<_> = (0..128)
        .map(|_| {
            let query = gpu.create_occlusion_query();
            gpu.begin_occlusion_query(&query).unwrap();
            query
        })
        .collect();
    assert!(gpu.begin_occlusion_query(&outer).is_err());
    drop(scopes);
    gpu.begin_occlusion_query(&outer).unwrap();
    gpu.end_occlusion_query(&outer).unwrap();
    assert_eq!(wait(&gpu, &outer), 0);
}
