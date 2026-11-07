use alkyhol_d3d12::*;
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let software = std::env::args().any(|arg| arg == "--allow-software");
    let device = pollster::block_on(D3D12CreateDevice(software))?;
    println!("adapter: {:?}", device.adapter_info());
    // Nonaligned width exercises 256-byte padded row removal on readback.
    let (width, height) = (65_u32, 65_u32);
    let target = device.CreateCommittedResource(width, height, ResourceState::RenderTarget)?;
    let pipeline = device.CreateGraphicsPipelineState(include_str!("triangle.wgsl"))?;
    let mut list = device.CreateCommandList();
    list.record(Command::ClearRenderTargetView {
        resource: target,
        color: [0.0, 0.0, 1.0, 1.0],
    })?;
    list.record(Command::Transition {
        resource: target,
        before: ResourceState::RenderTarget,
        after: ResourceState::CopySource,
    })?;
    list.close()?;
    device.ExecuteCommandLists(&list)?;
    let clear = device.ReadbackRGBA8(target)?;
    assert_eq!(clear.len(), width as usize * height as usize * 4);
    assert!(
        clear
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [0, 0, 255, 255]),
        "clear/copy pixels differ"
    );
    list.reset()?;
    list.record(Command::Transition {
        resource: target,
        before: ResourceState::CopySource,
        after: ResourceState::RenderTarget,
    })?;
    list.record(Command::DrawInstanced {
        resource: target,
        pipeline,
        vertices: 3,
        instances: 1,
    })?;
    list.record(Command::Transition {
        resource: target,
        before: ResourceState::RenderTarget,
        after: ResourceState::CopySource,
    })?;
    list.close()?;
    device.ExecuteCommandLists(&list)?;
    let pixels = device.ReadbackRGBA8(target)?;
    let pixel = |x: u32, y: u32| {
        &pixels[((y * width + x) * 4) as usize..((y * width + x) * 4 + 4) as usize]
    };
    assert_eq!(pixel(32, 32), [255, 0, 0, 255], "triangle center");
    for (x, y) in [(0, 0), (64, 0), (0, 64), (64, 64)] {
        assert_eq!(pixel(x, y), [0, 0, 255, 255], "background");
    }
    assert!(
        pixels
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [255, 0, 0, 255] || *p == [0, 0, 255, 255])
    );
    // A closed list can be replayed when its declared initial states still match.
    device.ExecuteCommandLists(&list)?;
    assert_eq!(device.ReadbackRGBA8(target)?, pixels);
    list.reset()?;
    list.record(Command::Transition {
        resource: target,
        before: ResourceState::RenderTarget,
        after: ResourceState::CopySource,
    })?;
    list.close()?;
    assert_eq!(
        device.ExecuteCommandLists(&list),
        Err(GraphicsError::WrongState)
    );
    // Wrong-state clear must fail before either GPU work or logical state mutation.
    list.reset()?;
    list.record(Command::ClearRenderTargetView {
        resource: target,
        color: [0.0; 4],
    })?;
    list.close()?;
    assert_eq!(
        device.ExecuteCommandLists(&list),
        Err(GraphicsError::WrongState)
    );
    assert_eq!(device.ReadbackRGBA8(target)?, pixels);
    assert!(
        device
            .CreateGraphicsPipelineState("invalid shader")
            .is_err()
    );
    if let Some(path) = std::env::var_os("ALKYHOL_PPM") {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        write!(file, "P6\n{width} {height}\n255\n")?;
        for p in pixels.as_chunks::<4>().0.iter() {
            file.write_all(&p[..3])?;
        }
    }
    println!(
        "clear, aligned readback, triangle pixels, state rejection and invalid shader checks passed"
    );
    Ok(())
}
