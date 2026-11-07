//! Original sample exercising the restricted CopyResource contract, not a port of
//! Microsoft's multi-adapter example or a native D3D12 differential reference.
use alkyhol_d3d12::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let software = std::env::args().any(|arg| arg == "--allow-software");
    let device = pollster::block_on(D3D12CreateDevice(software))?;
    println!("adapter: {:?}", device.adapter_info());
    let (width, height) = (65_u32, 65_u32);
    let source = device.CreateCommittedResource(width, height, ResourceState::RenderTarget)?;
    let destination = device.CreateCommittedResource(width, height, ResourceState::CopyDest)?;
    let pipeline = device.CreateGraphicsPipelineState(include_str!("triangle.wgsl"))?;
    let mut list = device.CreateCommandList();
    list.record(Command::ClearRenderTargetView {
        resource: source,
        color: [0.0, 0.0, 1.0, 1.0],
    })?;
    list.record(Command::DrawInstanced {
        resource: source,
        pipeline,
        vertices: 3,
        instances: 1,
    })?;
    list.record(Command::Transition {
        resource: source,
        before: ResourceState::RenderTarget,
        after: ResourceState::CopySource,
    })?;
    list.close()?;
    device.ExecuteCommandLists(&list)?;
    let original = device.ReadbackRGBA8(source)?;
    assert_eq!(original.len(), (width * height * 4) as usize);
    let center = ((32 * width + 32) * 4) as usize;
    assert_eq!(&original[center..center + 4], &[255, 0, 0, 255]);
    assert_eq!(&original[..4], &[0, 0, 255, 255]);
    list.reset()?;
    list.record(Command::CopyResource {
        source,
        destination,
    })?;
    list.record(Command::Transition {
        resource: destination,
        before: ResourceState::CopyDest,
        after: ResourceState::CopySource,
    })?;
    list.close()?;
    device.ExecuteCommandLists(&list)?;
    assert_eq!(
        device.ReadbackRGBA8(destination)?,
        original,
        "copied patterned pixels differ"
    );
    assert_eq!(
        device.ReadbackRGBA8(source)?,
        original,
        "copy modified its source"
    );
    println!("whole-resource patterned copy, padded readback and source preservation passed");
    Ok(())
}
