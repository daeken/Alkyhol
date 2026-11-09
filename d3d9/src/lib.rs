//! D3D9 GPU work. Guest COM pointers and Windows handles stay in the ABI layer.
//! Initial contract: single-sample BGRA8 render targets, clear and real readback.
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};
use wgpu::util::DeviceExt;
pub mod compiler;
mod depth;
pub mod legacy;
mod legacy_gpu;
pub mod vertex;
pub use depth::Depth;
mod render_state;
pub use render_state::RenderState;
mod texture;
mod texture_stages;
pub use legacy_gpu::{LegacyDraw, viewport_from_words};
pub mod fixed_vertex;
pub use texture::{SampleFormat, SampledTexture, SamplerState, Texture2D, TextureBinding};
pub use texture_stages::TextureStages;
pub use wgpu::{VertexAttribute, VertexFormat};
pub type Result<T> = std::result::Result<T, String>;

pub struct Device {
    device: wgpu::Device,
    queue: wgpu::Queue,
    info: wgpu::AdapterInfo,
}
#[derive(Clone)]
pub struct Target {
    texture: wgpu::Texture,
    width: u32,
    height: u32,
    initialized: Arc<AtomicBool>,
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
fn validate_vertices(bytes: &[u8]) -> Result<()> {
    if bytes.is_empty() || !bytes.len().is_multiple_of(60) || bytes.len() > 1024 * 1024 {
        return Err("transformed draw requires bounded complete triangles".into());
    }
    for vertex in bytes.as_chunks::<20>().0 {
        let p: Vec<f32> = vertex[..16]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_le_bytes(*b))
            .collect();
        if p.iter().any(|v| !v.is_finite()) || p[3] <= 0.0 {
            return Err("transformed position/rhw outside the finite positive-rhw profile".into());
        }
    }
    Ok(())
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
            required_features: adapter.features() & wgpu::Features::TEXTURE_COMPRESSION_BC,
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
        });
        self.finish()?;
        Ok(Target {
            texture,
            width,
            height,
            initialized: Arc::new(AtomicBool::new(false)),
        })
    }
    pub fn clear(&self, target: &mut Target, argb: u32) -> Result<()> {
        self.clear_state(target, argb, &RenderState::default())
    }
    pub fn clear_state(&self, target: &mut Target, argb: u32, state: &RenderState) -> Result<()> {
        let [r, g, b, a] = argb_channels(argb);
        self.scope();
        let view = target.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(state.format()),
            ..Default::default()
        });
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
        target.initialized.store(true, Ordering::Release);
        Ok(())
    }
    /// Copy a complete equal-size color attachment without changing draw state.
    /// X8 formats request opaque alpha; RGB bytes remain exactly those read.
    pub fn copy_target(
        &self,
        source: &Target,
        destination: &mut Target,
        opaque: bool,
    ) -> Result<()> {
        if source.dimensions() != destination.dimensions()
            || Arc::ptr_eq(&source.initialized, &destination.initialized)
        {
            return Err("color copy requires distinct equal-size attachments".into());
        }
        if !source.initialized.load(Ordering::Acquire) {
            return Err("copy from uninitialized color attachment".into());
        }
        if opaque {
            let mut bytes = self.readback(source)?;
            for pixel in bytes.as_chunks_mut::<4>().0 {
                pixel[3] = 255;
            }
            self.scope();
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &destination.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(source.width * 4),
                    rows_per_image: Some(source.height),
                },
                wgpu::Extent3d {
                    width: source.width,
                    height: source.height,
                    depth_or_array_layers: 1,
                },
            );
        } else {
            self.scope();
            let mut encoder = self.device.create_command_encoder(&Default::default());
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &source.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &destination.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: source.width,
                    height: source.height,
                    depth_or_array_layers: 1,
                },
            );
            self.queue.submit([encoder.finish()]);
        }
        self.finish()?;
        destination.initialized.store(true, Ordering::Release);
        Ok(())
    }
    /// XYZRHW|DIFFUSE triangle-list vertex bytes (20 bytes/vertex). D3D9's
    /// default clockwise faces are front-facing; no depth attachment is enabled.
    pub fn draw_transformed(&self, target: &mut Target, bytes: &[u8]) -> Result<()> {
        self.draw_transformed_state(target, bytes, &RenderState::default())
    }
    pub fn draw_transformed_state(
        &self,
        target: &mut Target,
        bytes: &[u8],
        state: &RenderState,
    ) -> Result<()> {
        self.draw_transformed_depth_state(target, bytes, state, None)
    }
    pub fn draw_transformed_depth_state(
        &self,
        target: &mut Target,
        bytes: &[u8],
        state: &RenderState,
        depth: Option<&Depth>,
    ) -> Result<()> {
        validate_vertices(bytes)?;
        let depth = if state.get(7)? != 0 || state.get(52)? != 0 {
            depth
        } else {
            None
        };
        depth::validate_depth(target, depth)?;
        let depth_state = state.depth_state(depth)?;
        let depth_view = depth.map(Depth::attachment);
        if !target.initialized.load(Ordering::Acquire) {
            return Err("draw requires an initialized backbuffer".into());
        }
        self.scope();
        let vertices = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("D3D9 transformed vertex stream"),
                contents: bytes,
                usage: wgpu::BufferUsages::VERTEX,
            });
        let uniform: Vec<u8> = [target.width as f32, target.height as f32, 0.0, 0.0]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let uniform = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("D3D9 viewport"),
                contents: &uniform,
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("D3D9 fixed-function XYZRHW diffuse"),
                source: wgpu::ShaderSource::Wgsl(include_str!("transformed.wgsl").into()),
            });
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("D3D9 transformed triangle list"),
                layout: None,
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: 20,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Unorm8x4,
                                offset: 16,
                                shader_location: 1,
                            },
                        ],
                    }],
                },
                primitive: state.primitive(),
                depth_stencil: depth_state,
                multisample: state.multisample(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: state.format(),
                        blend: state.blend(),
                        write_mask: state.writes(),
                    })],
                }),
                multiview: None,
                cache: None,
            });
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("D3D9 viewport"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let view = target.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(state.format()),
            ..Default::default()
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("D3D9 DrawPrimitive"),
                depth_stencil_attachment: depth_view.as_ref().map(|view| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view,
                        depth_ops: depth.unwrap().depth_ops(state),
                        stencil_ops: depth.unwrap().stencil_ops(state),
                    }
                }),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&pipeline);
            pass.set_stencil_reference(state.get(57)? & 255);
            pass.set_blend_constant(state.constant());
            pass.set_bind_group(0, &bind, &[]);
            pass.set_vertex_buffer(0, vertices.slice(..));
            pass.draw(0..(bytes.len() / 20) as u32, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        self.finish()
    }
    /// Returns tightly packed actual GPU BGRA8 bytes, never a CPU clear substitute.
    pub fn readback(&self, target: &Target) -> Result<Vec<u8>> {
        if !target.initialized.load(Ordering::Acquire) {
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
    fn reject_incomplete_nonfinite_and_zero_rhw_vertices() {
        assert!(validate_vertices(&[]).is_err());
        assert!(validate_vertices(&[0; 59]).is_err());
        let mut bytes = Vec::new();
        for _ in 0..3 {
            for f in [1f32, 1., 0.5, 1.] {
                bytes.extend(f.to_le_bytes());
            }
            bytes.extend(0xffabcdefu32.to_le_bytes());
        }
        assert!(validate_vertices(&bytes).is_ok());
        bytes[..4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(validate_vertices(&bytes).is_err());
        bytes[..4].copy_from_slice(&1f32.to_le_bytes());
        bytes[12..16].fill(0);
        assert!(validate_vertices(&bytes).is_err());
    }
    #[test]
    #[ignore = "requires a real native GPU; run explicitly"]
    fn gpu_transformed_triangle_preserves_background_and_culls_reverse_winding() {
        let d = Device::new().unwrap();
        let mut t = d.create_target(32, 32).unwrap();
        let mut data = Vec::new();
        for (x, y) in [(16f32, 4f32), (28., 28.), (4., 28.)] {
            for f in [x, y, 0.5, 1.] {
                data.extend(f.to_le_bytes());
            }
            data.extend(0xffff0000u32.to_le_bytes());
        }
        d.clear(&mut t, 0xff0000ff).unwrap();
        d.draw_transformed(&mut t, &data).unwrap();
        let pixels = d.readback(&t).unwrap();
        let pixel = |x: usize, y: usize| &pixels[(y * 32 + x) * 4..(y * 32 + x) * 4 + 4];
        assert_eq!(pixel(16, 16), [0, 0, 255, 255]);
        assert_eq!(pixel(0, 0), [255, 0, 0, 255]);
        d.clear(&mut t, 0xff0000ff).unwrap();
        let (a, b) = data[20..].split_at_mut(20);
        a.swap_with_slice(b);
        d.draw_transformed(&mut t, &data).unwrap();
        assert_eq!(d.readback(&t).unwrap(), [255, 0, 0, 255].repeat(32 * 32));
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
