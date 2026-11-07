//! Whole RGBA8 texture copies on real GPU backends; no CPU copy substitution.
use alkyhol_d3d12::*;

fn device() -> Device {
    let software = std::env::var("ALKYHOL_ALLOW_SOFTWARE").as_deref() == Ok("1");
    let device =
        pollster::block_on(D3D12CreateDevice(software)).expect("real GPU backend required");
    eprintln!("adapter: {:?}", device.adapter_info());
    device
}
fn filled_source(device: &Device) -> u64 {
    let source = device
        .CreateCommittedResource(65, 3, ResourceState::RenderTarget)
        .unwrap();
    let mut list = device.CreateCommandList();
    list.record(Command::ClearRenderTargetView {
        resource: source,
        color: [0.0, 1.0, 0.0, 1.0],
    })
    .unwrap();
    list.record(Command::Transition {
        resource: source,
        before: ResourceState::RenderTarget,
        after: ResourceState::CopySource,
    })
    .unwrap();
    list.close().unwrap();
    device.ExecuteCommandLists(&list).unwrap();
    source
}

#[test]
fn chained_copies_preserve_every_pixel_and_leave_the_source_intact() {
    let device = device();
    let a = filled_source(&device);
    let b = device
        .CreateCommittedResource(65, 3, ResourceState::CopyDest)
        .unwrap();
    let c = device
        .CreateCommittedResource(65, 3, ResourceState::CopyDest)
        .unwrap();
    let mut list = device.CreateCommandList();
    list.record(Command::CopyResource {
        source: a,
        destination: b,
    })
    .unwrap();
    list.record(Command::Transition {
        resource: b,
        before: ResourceState::CopyDest,
        after: ResourceState::CopySource,
    })
    .unwrap();
    list.record(Command::CopyResource {
        source: b,
        destination: c,
    })
    .unwrap();
    list.record(Command::Transition {
        resource: c,
        before: ResourceState::CopyDest,
        after: ResourceState::CopySource,
    })
    .unwrap();
    list.close().unwrap();
    device.ExecuteCommandLists(&list).unwrap();
    let expected = [0_u8, 255, 0, 255].repeat(65 * 3);
    for resource in [a, b, c] {
        assert_eq!(device.ReadbackRGBA8(resource).unwrap(), expected);
    }
}

#[test]
fn mismatched_copy_rejects_the_entire_list_before_gpu_writes_or_state_commit() {
    let device = device();
    let source = filled_source(&device);
    for (width, height) in [(64, 3), (65, 2)] {
        let destination = device
            .CreateCommittedResource(width, height, ResourceState::CopyDest)
            .unwrap();
        let mut list = device.CreateCommandList();
        list.record(Command::Transition {
            resource: source,
            before: ResourceState::CopySource,
            after: ResourceState::RenderTarget,
        })
        .unwrap();
        list.record(Command::ClearRenderTargetView {
            resource: source,
            color: [1.0; 4],
        })
        .unwrap();
        list.record(Command::Transition {
            resource: source,
            before: ResourceState::RenderTarget,
            after: ResourceState::CopySource,
        })
        .unwrap();
        list.record(Command::CopyResource {
            source,
            destination,
        })
        .unwrap();
        list.record(Command::Transition {
            resource: destination,
            before: ResourceState::CopyDest,
            after: ResourceState::CopySource,
        })
        .unwrap();
        list.close().unwrap();
        assert_eq!(
            device.ExecuteCommandLists(&list),
            Err(GraphicsError::InvalidArgument)
        );
        assert_eq!(
            device.ReadbackRGBA8(source).unwrap(),
            [0_u8, 255, 0, 255].repeat(65 * 3)
        );
        assert_eq!(
            device.ReadbackRGBA8(destination),
            Err(GraphicsError::WrongState)
        );
        // A valid transition from the ORIGINAL destination state must still work.
        list.reset().unwrap();
        list.record(Command::Transition {
            resource: destination,
            before: ResourceState::CopyDest,
            after: ResourceState::RenderTarget,
        })
        .unwrap();
        list.record(Command::ClearRenderTargetView {
            resource: destination,
            color: [1.0; 4],
        })
        .unwrap();
        list.record(Command::Transition {
            resource: destination,
            before: ResourceState::RenderTarget,
            after: ResourceState::CopySource,
        })
        .unwrap();
        list.close().unwrap();
        device.ExecuteCommandLists(&list).unwrap();
        assert_eq!(
            device.ReadbackRGBA8(destination).unwrap(),
            [255_u8; 4].repeat((width * height) as usize)
        );
    }
}
