//! D3D9 GPU work. Guest COM pointers and Windows handles stay in the ABI layer.
//! Initial contract: single-sample BGRA8 render targets, clear and real readback.
use std::{sync::mpsc, time::Duration};
pub type Result<T> = std::result::Result<T, String>;

pub struct Device {
    device: wgpu::Device,
    queue: wgpu::Queue,
    info: wgpu::AdapterInfo,
}
pub struct Target {
    texture: wgpu::Texture,
    width: u32,
    height: u32,
    initialized: bool,
}
fn extent(width: u32, height: u32, max: u32) -> Result<()> {
    if width == 0 || height == 0 || width > max || height > max {
        return Err("render target dimensions outside the device limits".into());
    }
    Ok(())
}
pub fn argb_channels(color: u32) -> [u8; 4] {
    [
        (color >> 16) as u8,
        (color >> 8) as u8,
        color as u8,
        (color >> 24) as u8,
    ]
}
impl Device {
    /// Select a real native adapter. No backend or adapter identity is fabricated.
    pub fn new() -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
        .map_err(|e| e.to_string())?;
        let info = adapter.get_info();
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Alkyhol D3D9"),
            required_limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
            ..Default::default()
        }))
        .map_err(|e| e.to_string())?;
        Ok(Self {
            device,
            queue,
            info,
        })
    }
    pub fn adapter_name(&self) -> &str {
        &self.info.name
    }
    pub fn backend_name(&self) -> String {
        format!("{:?}", self.info.backend)
    }
    pub fn hardware(&self) -> bool {
        !matches!(self.info.device_type, wgpu::DeviceType::Cpu)
    }
    pub fn vendor_device(&self) -> (u32, u32) {
        (self.info.vendor, self.info.device)
    }
    pub fn check_status(&self) -> Result<()> {
        self.device
            .poll(wgpu::PollType::Poll)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    fn scope(&self) {
        self.device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        self.device.push_error_scope(wgpu::ErrorFilter::Internal);
        self.device.push_error_scope(wgpu::ErrorFilter::Validation);
    }
    fn finish(&self) -> Result<()> {
        let mut error = None;
        for _ in 0..3 {
            if let Some(e) = pollster::block_on(self.device.pop_error_scope()) {
                error.get_or_insert_with(|| e.to_string());
            }
        }
        error.map_or(Ok(()), Err)
    }
    pub fn create_target(&self, width: u32, height: u32) -> Result<Target> {
        extent(
            width,
            height,
            self.device.limits().max_texture_dimension_2d.min(8192),
        )?;
        self.scope();
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("D3D9 BGRA8 backbuffer"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        self.finish()?;
        Ok(Target {
            texture,
            width,
            height,
            initialized: false,
        })
    }
    pub fn clear(&self, target: &mut Target, argb: u32) -> Result<()> {
        let [r, g, b, a] = argb_channels(argb);
        self.scope();
        let view = target.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("D3D9 Clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r as f64 / 255.0,
                            g: g as f64 / 255.0,
                            b: b as f64 / 255.0,
                            a: a as f64 / 255.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }
        self.queue.submit([encoder.finish()]);
        self.finish()?;
        target.initialized = true;
        Ok(())
    }
    /// Returns tightly packed actual GPU BGRA8 bytes, never a CPU clear substitute.
    pub fn readback(&self, target: &Target) -> Result<Vec<u8>> {
        if !target.initialized {
            return Err("backbuffer has not been initialized by rendering".into());
        }
        let pitch = (target.width * 4).div_ceil(256) * 256;
        self.scope();
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("D3D9 GPU readback"),
            size: pitch as u64 * target.height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            target.texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(pitch),
                    rows_per_image: Some(target.height),
                },
            },
            wgpu::Extent3d {
                width: target.width,
                height: target.height,
                depth_or_array_layers: 1,
            },
        );
        let submission = self.queue.submit([encoder.finish()]);
        let (send, receive) = mpsc::sync_channel(1);
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = send.send(r);
        });
        let polled = self
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(Duration::from_secs(30)),
            })
            .map_err(|e| e.to_string());
        let scoped = self.finish();
        polled?;
        scoped?;
        receive
            .recv_timeout(Duration::from_secs(1))
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
        let mapped = buffer.slice(..).get_mapped_range();
        let mut bytes = Vec::with_capacity(target.width as usize * target.height as usize * 4);
        for row in mapped.chunks_exact(pitch as usize) {
            bytes.extend_from_slice(&row[..target.width as usize * 4]);
        }
        drop(mapped);
        buffer.unmap();
        Ok(bytes)
    }
}
impl Target {
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn d3d_argb_is_not_rgba_or_native_endian() {
        assert_eq!(argb_channels(0x715e2ac3), [0x5e, 0x2a, 0xc3, 0x71]);
    }
    #[test]
    fn dimensions_reject_empty_oversized_and_wrapping_inputs() {
        for (w, h) in [(0, 1), (1, 0), (8193, 1), (1, u32::MAX)] {
            assert!(extent(w, h, 8192).is_err());
        }
        assert!(extent(8192, 8192, 8192).is_ok());
    }
    #[test]
    #[ignore = "requires a real native GPU; run explicitly"]
    fn gpu_clear_reads_every_pixel_with_padded_rows_and_preserves_other_targets() {
        let device = Device::new().unwrap();
        eprintln!(
            "D3D9 GPU evidence: {} / {}; hardware={}",
            device.adapter_name(),
            device.backend_name(),
            device.hardware()
        );
        let mut a = device.create_target(65, 3).unwrap();
        let mut b = device.create_target(7, 2).unwrap();
        assert!(device.readback(&a).is_err());
        for color in [0xff715e2a, 0xff0000ff, 0xff102030] {
            device.clear(&mut a, color).unwrap();
            let [r, g, blue, alpha] = argb_channels(color);
            assert_eq!(
                device.readback(&a).unwrap(),
                [blue, g, r, alpha].repeat(65 * 3)
            );
        }
        device.clear(&mut b, 0xffabcdef).unwrap();
        assert_eq!(
            device.readback(&b).unwrap(),
            [0xef, 0xcd, 0xab, 0xff].repeat(7 * 2)
        );
        assert_eq!(
            device.readback(&a).unwrap(),
            [0x30, 0x20, 0x10, 0xff].repeat(65 * 3)
        );
        assert!(device.create_target(0, 8).is_err());
    }
}
