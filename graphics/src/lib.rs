//! Native Rust D3D12-shaped subset over direct Metal. No COM ABI or DXIL translation.
#![allow(non_snake_case)]
pub use alkyhol::graphics::{Command, GraphicsCommandList, GraphicsError, ResourceState, Result};
use alkyhol::graphics::{copy_footprint, validate_submission};
use metal::*;
use objc::rc::autoreleasepool;
use std::{
    collections::HashMap,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

static NEXT_DEVICE: AtomicU64 = AtomicU64::new(1);
static NEXT_OBJECT: AtomicU64 = AtomicU64::new(1);
#[derive(Debug)]
pub enum Backend {
    Metal,
}
#[derive(Debug)]
pub struct AdapterInfo {
    pub name: String,
    pub backend: Backend,
    pub ray_tracing: bool,
}
struct Texture {
    texture: metal::Texture,
    width: u32,
    height: u32,
}
#[derive(Default)]
struct Objects {
    textures: HashMap<u64, Texture>,
    states: HashMap<u64, ResourceState>,
    pipelines: HashMap<u64, RenderPipelineState>,
}
pub struct Device {
    id: u64,
    device: metal::Device,
    queue: CommandQueue,
    info: AdapterInfo,
    objects: Mutex<Objects>,
}
fn backend(error: impl std::fmt::Display) -> GraphicsError {
    GraphicsError::Backend(error.to_string())
}
/// Commit and check actual completion. The initial source facade blocks here;
/// asynchronous native Metal access is available to advanced renderers below.
pub fn complete(command: &CommandBufferRef) -> Result<()> {
    command.commit();
    command.wait_until_completed();
    if command.status() != MTLCommandBufferStatus::Completed {
        return Err(backend(format!(
            "Metal command buffer ended with {:?}",
            command.status()
        )));
    }
    Ok(())
}

pub async fn D3D12CreateDevice(allow_software: bool) -> Result<Device> {
    if allow_software {
        return Err(GraphicsError::Unsupported);
    }
    autoreleasepool(|| {
        let device = metal::Device::system_default().ok_or_else(|| backend("no Metal device"))?;
        let info = AdapterInfo {
            name: device.name().to_owned(),
            backend: Backend::Metal,
            ray_tracing: device.supports_raytracing(),
        };
        let queue = device.new_command_queue();
        Ok(Device {
            id: NEXT_DEVICE.fetch_add(1, Ordering::Relaxed),
            device,
            queue,
            info,
            objects: Mutex::default(),
        })
    })
}
impl Device {
    pub fn adapter_info(&self) -> &AdapterInfo {
        &self.info
    }
    /// Native compute/presentation/ray-tracing access; host objects only, no COM ABI.
    pub fn metal_device(&self) -> &metal::DeviceRef {
        &self.device
    }
    pub fn metal_queue(&self) -> &CommandQueueRef {
        &self.queue
    }
    pub fn CreateCommandList(&self) -> GraphicsCommandList {
        GraphicsCommandList::new(self.id)
    }
    pub fn CreateCommittedResource(
        &self,
        width: u32,
        height: u32,
        state: ResourceState,
    ) -> Result<u64> {
        let (_, bytes) = copy_footprint(width, height)?;
        // Conservative supported Metal 2D extent; broader limits need capability contracts.
        if width > 16384 || height > 16384 || bytes > self.device.max_buffer_length() {
            return Err(GraphicsError::InvalidArgument);
        }
        autoreleasepool(|| {
            let descriptor = TextureDescriptor::new();
            descriptor.set_texture_type(MTLTextureType::D2);
            descriptor.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
            descriptor.set_width(width as u64);
            descriptor.set_height(height as u64);
            descriptor.set_mipmap_level_count(1);
            descriptor.set_sample_count(1);
            descriptor.set_storage_mode(MTLStorageMode::Private);
            descriptor.set_usage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
            let texture = self.device.new_texture(&descriptor);
            let id = NEXT_OBJECT.fetch_add(1, Ordering::Relaxed);
            let mut objects = self.objects.lock().unwrap();
            objects.textures.insert(
                id,
                Texture {
                    texture,
                    width,
                    height,
                },
            );
            objects.states.insert(id, state);
            Ok(id)
        })
    }
    /// Native MSL with vertex vs_main and fragment fs_main, no bindings/buffers.
    pub fn CreateGraphicsPipelineState(&self, msl: &str) -> Result<u64> {
        autoreleasepool(|| {
            let options = CompileOptions::new();
            options.set_fast_math_enabled(false);
            let library = self
                .device
                .new_library_with_source(msl, &options)
                .map_err(backend)?;
            let vertex = library.get_function("vs_main", None).map_err(backend)?;
            let fragment = library.get_function("fs_main", None).map_err(backend)?;
            let descriptor = RenderPipelineDescriptor::new();
            descriptor.set_vertex_function(Some(&vertex));
            descriptor.set_fragment_function(Some(&fragment));
            descriptor
                .color_attachments()
                .object_at(0)
                .unwrap()
                .set_pixel_format(MTLPixelFormat::RGBA8Unorm);
            let pipeline = self
                .device
                .new_render_pipeline_state(&descriptor)
                .map_err(backend)?;
            let id = NEXT_OBJECT.fetch_add(1, Ordering::Relaxed);
            self.objects.lock().unwrap().pipelines.insert(id, pipeline);
            Ok(id)
        })
    }
    pub fn ExecuteCommandLists(&self, list: &GraphicsCommandList) -> Result<()> {
        let mut objects = self.objects.lock().unwrap();
        let pipelines: Vec<_> = objects.pipelines.keys().copied().collect();
        let states = validate_submission(self.id, list, &objects.states, &pipelines)?;
        for command in list.commands()? {
            if let Command::CopyResource {
                source,
                destination,
            } = command
            {
                let source = &objects.textures[source];
                let destination = &objects.textures[destination];
                if (source.width, source.height) != (destination.width, destination.height) {
                    return Err(GraphicsError::InvalidArgument);
                }
            }
        }
        autoreleasepool(|| {
            let command_buffer = self.queue.new_command_buffer();
            for command in list.commands()? {
                match command {
                    Command::Transition { .. } => (), // Logical states above; Metal tracks physical hazards.
                    Command::ClearRenderTargetView { resource, color } => {
                        let target = &objects.textures[resource];
                        let descriptor = RenderPassDescriptor::new();
                        let attachment = descriptor.color_attachments().object_at(0).unwrap();
                        attachment.set_texture(Some(&target.texture));
                        attachment.set_load_action(MTLLoadAction::Clear);
                        attachment.set_store_action(MTLStoreAction::Store);
                        attachment.set_clear_color(MTLClearColor::new(
                            color[0], color[1], color[2], color[3],
                        ));
                        let encoder = command_buffer.new_render_command_encoder(descriptor);
                        encoder.end_encoding();
                    }
                    Command::DrawInstanced {
                        resource,
                        pipeline,
                        vertices,
                        instances: _,
                    } => {
                        let target = &objects.textures[resource];
                        let descriptor = RenderPassDescriptor::new();
                        let attachment = descriptor.color_attachments().object_at(0).unwrap();
                        attachment.set_texture(Some(&target.texture));
                        attachment.set_load_action(MTLLoadAction::Load);
                        attachment.set_store_action(MTLStoreAction::Store);
                        let encoder = command_buffer.new_render_command_encoder(descriptor);
                        encoder.set_render_pipeline_state(&objects.pipelines[pipeline]);
                        encoder.set_cull_mode(MTLCullMode::None);
                        encoder.set_viewport(MTLViewport {
                            originX: 0.0,
                            originY: 0.0,
                            width: target.width as f64,
                            height: target.height as f64,
                            znear: 0.0,
                            zfar: 1.0,
                        });
                        encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, *vertices as u64);
                        encoder.end_encoding();
                    }
                    Command::CopyResource {
                        source,
                        destination,
                    } => {
                        let source = &objects.textures[source];
                        let destination = &objects.textures[destination];
                        let encoder = command_buffer.new_blit_command_encoder();
                        encoder.copy_from_texture(
                            &source.texture,
                            0,
                            0,
                            MTLOrigin::default(),
                            MTLSize::new(source.width as u64, source.height as u64, 1),
                            &destination.texture,
                            0,
                            0,
                            MTLOrigin::default(),
                        );
                        encoder.end_encoding();
                    }
                }
            }
            complete(command_buffer)?;
            objects.states = states;
            Ok(())
        })
    }
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
        autoreleasepool(|| {
            let buffer = self
                .device
                .new_buffer(bytes, MTLResourceOptions::StorageModeShared);
            let command = self.queue.new_command_buffer();
            let encoder = command.new_blit_command_encoder();
            encoder.copy_from_texture_to_buffer(
                &texture.texture,
                0,
                0,
                MTLOrigin::default(),
                MTLSize::new(texture.width as u64, texture.height as u64, 1),
                &buffer,
                0,
                pitch as u64,
                bytes,
                MTLBlitOption::None,
            );
            encoder.end_encoding();
            complete(command)?;
            let data = unsafe {
                std::slice::from_raw_parts(buffer.contents().cast::<u8>(), bytes as usize)
            };
            let mut tight =
                Vec::with_capacity(texture.width as usize * texture.height as usize * 4);
            for row in data
                .chunks_exact(pitch as usize)
                .take(texture.height as usize)
            {
                tight.extend_from_slice(&row[..texture.width as usize * 4]);
            }
            Ok(tight)
        })
    }
}
