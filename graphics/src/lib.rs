//! Small native Rust D3D12-shaped facade. Not a COM ABI, DXIL compiler or feature-level claim.
#![allow(non_snake_case)]
pub use alkyhol::graphics::{Command, GraphicsCommandList, GraphicsError, ResourceState, Result};
use alkyhol::graphics::{copy_footprint, validate_submission};
use std::{
    collections::HashMap,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
static NEXT_DEVICE: AtomicU64 = AtomicU64::new(1);
static NEXT_OBJECT: AtomicU64 = AtomicU64::new(1);
struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}
#[derive(Default)]
struct Objects {
    textures: HashMap<u64, Texture>,
    states: HashMap<u64, ResourceState>,
    pipelines: HashMap<u64, wgpu::RenderPipeline>,
}
/// Owns a single direct command queue and serializes validation and submission.
pub struct Device {
    id: u64,
    device: wgpu::Device,
    queue: wgpu::Queue,
    info: wgpu::AdapterInfo,
    objects: Mutex<Objects>,
}
fn backend(e: impl std::fmt::Display) -> GraphicsError {
    GraphicsError::Backend(e.to_string())
}

/// Create a real backend device. No noop backend or success-on-absence fallback.
/// `allow_software` must be explicitly true for a CPU/Vulkan adapter (e.g. CI).
pub async fn D3D12CreateDevice(allow_software: bool) -> Result<Device> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: if cfg!(target_os = "macos") {
            wgpu::Backends::METAL
        } else if cfg!(target_os = "windows") {
            wgpu::Backends::DX12
        } else {
            wgpu::Backends::VULKAN
        },
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: allow_software,
            compatible_surface: None,
        })
        .await
        .map_err(backend)?;
    let info = adapter.get_info();
    if info.device_type == wgpu::DeviceType::Cpu && !allow_software {
        return Err(GraphicsError::Unsupported);
    }
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("Alkyhol restricted D3D12"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            ..Default::default()
        })
        .await
        .map_err(backend)?;
    Ok(Device {
        id: NEXT_DEVICE.fetch_add(1, Ordering::Relaxed),
        device,
        queue,
        info,
        objects: Mutex::default(),
    })
}
impl Device {
    pub fn adapter_info(&self) -> &wgpu::AdapterInfo {
        &self.info
    }
    pub fn CreateCommandList(&self) -> GraphicsCommandList {
        GraphicsCommandList::new(self.id)
    }
    /// RGBA8, single-sample, single-mip, 2D default-heap resource only.
    pub fn CreateCommittedResource(
        &self,
        width: u32,
        height: u32,
        state: ResourceState,
    ) -> Result<u64> {
        let (_, bytes) = copy_footprint(width, height)?;
        let limit = self.device.limits().max_texture_dimension_2d;
        if width > limit || height > limit || bytes > self.device.limits().max_buffer_size {
            return Err(GraphicsError::InvalidArgument);
        }
        let mut objects = self.objects.lock().unwrap();
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Alkyhol RGBA8 resource"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let id = NEXT_OBJECT.fetch_add(1, Ordering::Relaxed);
        objects.textures.insert(
            id,
            Texture {
                texture,
                view,
                width,
                height,
            },
        );
        objects.states.insert(id, state);
        Ok(id)
    }
    /// WGSL with vs_main(vertex_index) and fs_main, no bindings/vertex buffers.
    /// Shader compilation errors are reported, not substituted with a default shader.
    pub fn CreateGraphicsPipelineState(&self, wgsl: &str) -> Result<u64> {
        let mut objects = self.objects.lock().unwrap();
        self.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Alkyhol translated sample shader"),
                source: wgpu::ShaderSource::Wgsl(wgsl.into()),
            });
        let layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Alkyhol binding-free root signature"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Alkyhol sample PSO"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(FORMAT.into())],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
                cache: None,
            });
        if let Some(error) = pollster::block_on(self.device.pop_error_scope()) {
            return Err(backend(error));
        }
        let id = NEXT_OBJECT.fetch_add(1, Ordering::Relaxed);
        objects.pipelines.insert(id, pipeline);
        Ok(id)
    }
    /// All commands validated before encoding; resource states advance at submission,
    /// never at recording time. One queue; no allocator/fence/COM lifetime emulation yet.
    pub fn ExecuteCommandLists(&self, list: &GraphicsCommandList) -> Result<()> {
        let mut objects = self.objects.lock().unwrap();
        let pipelines: Vec<_> = objects.pipelines.keys().copied().collect();
        let states = validate_submission(self.id, list, &objects.states, &pipelines)?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Alkyhol direct list"),
            });
        for command in list.commands()? {
            match command {
                Command::Transition { .. } => (), // wgpu tracks the physical barriers; states above enforce the declared contract.
                Command::ClearRenderTargetView { resource, color } => {
                    let target = &objects.textures[resource];
                    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("ClearRenderTargetView"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &target.view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: color[0],
                                    g: color[1],
                                    b: color[2],
                                    a: color[3],
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                }
                Command::DrawInstanced {
                    resource,
                    pipeline,
                    vertices,
                    instances,
                } => {
                    let target = &objects.textures[resource];
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("DrawInstanced"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &target.view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    pass.set_pipeline(&objects.pipelines[pipeline]);
                    pass.draw(0..*vertices, 0..*instances);
                }
            }
        }
        self.queue.submit([encoder.finish()]);
        objects.states = states;
        Ok(())
    }
    /// Blocking CPU readback is an explicit sample helper, not an ID3D12Resource::Map implementation.
    pub fn ReadbackRGBA8(&self, resource: u64) -> Result<Vec<u8>> {
        let objects = self.objects.lock().unwrap();
        if objects.states.get(&resource) != Some(&ResourceState::CopySource) {
            return Err(GraphicsError::WrongState);
        }
        let texture = objects
            .textures
            .get(&resource)
            .ok_or(GraphicsError::InvalidArgument)?;
        let (pitch, bytes) = copy_footprint(texture.width, texture.height)?;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Alkyhol readback"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(pitch),
                    rows_per_image: Some(texture.height),
                },
            },
            wgpu::Extent3d {
                width: texture.width,
                height: texture.height,
                depth_or_array_layers: 1,
            },
        );
        let submission = self.queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
            });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(Duration::from_secs(10)),
            })
            .map_err(backend)?;
        rx.recv_timeout(Duration::from_secs(10))
            .map_err(backend)?
            .map_err(backend)?;
        let data = buffer.slice(..).get_mapped_range();
        let mut tight =
            Vec::with_capacity((texture.width as usize) * 4 * (texture.height as usize));
        for row in data
            .chunks_exact(pitch as usize)
            .take(texture.height as usize)
        {
            tight.extend_from_slice(&row[..texture.width as usize * 4]);
        }
        drop(data);
        buffer.unmap();
        Ok(tight)
    }
}
