//! Real GPU integration tests. Missing backends fail rather than silently skip.
use alkyhol_d3d12::*;

fn device() -> Device {
    let software = std::env::var("ALKYHOL_ALLOW_SOFTWARE").as_deref() == Ok("1");
    let device =
        pollster::block_on(D3D12CreateDevice(software)).expect("real GPU backend required");
    eprintln!("adapter: {:?}", device.adapter_info());
    device
}

#[test]
fn native_backend_access_matches_reported_adapter_and_rejects_software_requests() {
    let device = device();
    assert_eq!(device.adapter_info().name, device.metal_device().name());
    assert_eq!(
        device.adapter_info().ray_tracing,
        device.metal_device().supports_raytracing()
    );
    assert!(matches!(
        pollster::block_on(D3D12CreateDevice(true)),
        Err(GraphicsError::Unsupported)
    ));
}

#[test]
fn resource_creation_rejects_zero_and_oversized_extents() {
    let device = device();
    for (width, height) in [(0, 1), (1, 0), (u32::MAX, 1), (1, u32::MAX)] {
        assert_eq!(
            device.CreateCommittedResource(width, height, ResourceState::RenderTarget),
            Err(GraphicsError::InvalidArgument)
        );
    }
    assert!(
        device
            .CreateCommittedResource(1, 1, ResourceState::CopySource)
            .is_ok()
    );
}

#[test]
fn submissions_reject_open_foreign_lists_and_foreign_resources() {
    let a = device();
    let b = device();
    let target = a
        .CreateCommittedResource(1, 1, ResourceState::RenderTarget)
        .unwrap();
    let mut list = a.CreateCommandList();
    assert_eq!(a.ExecuteCommandLists(&list), Err(GraphicsError::ListOpen));
    list.close().unwrap();
    assert_eq!(
        b.ExecuteCommandLists(&list),
        Err(GraphicsError::WrongDevice)
    );
    let mut foreign = b.CreateCommandList();
    foreign
        .record(Command::ClearRenderTargetView {
            resource: target,
            color: [0.0; 4],
        })
        .unwrap();
    foreign.close().unwrap();
    assert_eq!(
        b.ExecuteCommandLists(&foreign),
        Err(GraphicsError::InvalidArgument)
    );
    assert_eq!(a.ReadbackRGBA8(target), Err(GraphicsError::WrongState));
}

#[test]
fn padded_readback_and_failed_submissions_preserve_pixels_and_states() {
    let device = device();
    let target = device
        .CreateCommittedResource(65, 3, ResourceState::RenderTarget)
        .unwrap();
    let mut list = device.CreateCommandList();
    list.record(Command::ClearRenderTargetView {
        resource: target,
        color: [0.0, 1.0, 0.0, 1.0],
    })
    .unwrap();
    list.record(Command::Transition {
        resource: target,
        before: ResourceState::RenderTarget,
        after: ResourceState::CopySource,
    })
    .unwrap();
    list.close().unwrap();
    device.ExecuteCommandLists(&list).unwrap();
    let expected = [0_u8, 255, 0, 255].repeat(65 * 3);
    assert_eq!(device.ReadbackRGBA8(target).unwrap(), expected);
    // Clear and transition would both change observable state if partially submitted.
    list.reset().unwrap();
    list.record(Command::Transition {
        resource: target,
        before: ResourceState::CopySource,
        after: ResourceState::RenderTarget,
    })
    .unwrap();
    list.record(Command::ClearRenderTargetView {
        resource: target,
        color: [1.0; 4],
    })
    .unwrap();
    list.record(Command::DrawInstanced {
        resource: target,
        pipeline: u64::MAX,
        vertices: 3,
        instances: 1,
    })
    .unwrap();
    list.close().unwrap();
    assert_eq!(
        device.ExecuteCommandLists(&list),
        Err(GraphicsError::InvalidArgument)
    );
    assert_eq!(device.ReadbackRGBA8(target).unwrap(), expected);
}

#[test]
fn shader_failures_do_not_poison_later_pipeline_creation_or_allow_foreign_tokens() {
    let a = device();
    let b = device();
    assert!(matches!(
        a.CreateGraphicsPipelineState("invalid shader"),
        Err(GraphicsError::Backend(_))
    ));
    let pipeline = a
        .CreateGraphicsPipelineState(include_str!("../examples/triangle.metal"))
        .unwrap();
    let target = b
        .CreateCommittedResource(1, 1, ResourceState::RenderTarget)
        .unwrap();
    let mut list = b.CreateCommandList();
    list.record(Command::DrawInstanced {
        resource: target,
        pipeline,
        vertices: 3,
        instances: 1,
    })
    .unwrap();
    list.close().unwrap();
    assert_eq!(
        b.ExecuteCommandLists(&list),
        Err(GraphicsError::InvalidArgument)
    );
}
