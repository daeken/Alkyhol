//! Exact observed bytes for the independent native D3D12 checkerboard scenario.
use alkyhol_d3d12::*;
use std::fmt::Write;

fn hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(result, "{byte:02x}").unwrap();
    }
    result
}
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let software = std::env::args().any(|arg| arg == "--allow-software");
    let device = pollster::block_on(D3D12CreateDevice(software))?;
    eprintln!("adapter: {:?}", device.adapter_info());
    let (width, height) = (65_u32, 4_u32);
    let source = device.CreateCommittedResource(width, height, ResourceState::RenderTarget)?;
    let destination = device.CreateCommittedResource(width, height, ResourceState::CopyDest)?;
    let pipeline = device.CreateGraphicsPipelineState(include_str!("checker.wgsl"))?;
    let mut list = device.CreateCommandList();
    list.record(Command::ClearRenderTargetView {
        resource: source,
        color: [0.0, 1.0, 0.0, 1.0],
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
    let original = device.ReadbackRGBA8(source)?;
    let copied = device.ReadbackRGBA8(destination)?;
    assert_eq!(original.len(), (width * height * 4) as usize);
    assert_eq!(copied, original, "source/destination bytes differ");
    for y in 0..height {
        for x in 0..width {
            let offset = ((y * width + x) * 4) as usize;
            let expected = [
                ((x & 1) * 255) as u8,
                ((y & 1) * 255) as u8,
                ((((x >> 1) + y) & 1) * 255) as u8,
                255,
            ];
            assert_eq!(
                &original[offset..offset + 4],
                &expected,
                "checkerboard pixel {x},{y}"
            );
        }
    }
    println!(
        "{{\"scenario\":\"rgba8-checker-copy-v1\",\"width\":{width},\"height\":{height},\"source_rgba8\":\"{}\",\"destination_rgba8\":\"{}\"}}",
        hex(&original),
        hex(&copied)
    );
    Ok(())
}
