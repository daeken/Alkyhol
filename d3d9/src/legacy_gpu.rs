//! Real programmable draws over translated legacy bytecode. Initial binding
//! profile: float constants, triangle lists and explicit float vertex attributes.
use super::{Depth, Device, RenderState, Result, Target, TextureBinding, depth, legacy};
use std::{collections::BTreeSet, sync::atomic::Ordering};
use wgpu::util::DeviceExt;
pub struct LegacyDraw<'a> {
    /// Normalized indices into this draw's vertex slice; uploaded as a GPU INDEX buffer.
    pub indices: Option<&'a [u32]>,
    pub vertex_spirv: &'a [u8],
    pub pixel_spirv: &'a [u8],
    pub vertices: &'a [u8],
    pub stride: u64,
    pub attributes: &'a [wgpu::VertexAttribute],
    pub vertex_constants: &'a [u8],
    pub textures: &'a [TextureBinding<'a>],
    pub pixel_constants: &'a [u8],
}
/// Decode D3DVIEWPORT9 words while retaining checked native target bounds.
pub fn viewport_from_words(words: [u32; 6], width: u32, height: u32) -> Result<[f32; 6]> {
    let viewport = [
        words[0] as f32,
        words[1] as f32,
        words[2] as f32,
        words[3] as f32,
        f32::from_bits(words[4]),
        f32::from_bits(words[5]),
    ];
    validate_viewport(viewport, width, height)?;
    Ok(viewport)
}
fn validate_viewport(v: [f32; 6], width: u32, height: u32) -> Result<()> {
    if v.iter().any(|n| !n.is_finite())
        || v[0] < 0.
        || v[1] < 0.
        || v[2] <= 0.
        || v[3] <= 0.
        || v[0] + v[2] > width as f32
        || v[1] + v[3] > height as f32
        || v[4] < 0.
        || v[5] > 1.
        || v[4] > v[5]
    {
        return Err("viewport lies outside the checked target/depth profile".into());
    }
    Ok(())
}
fn resources(module: &naga::Module, stage: u32, used: &mut BTreeSet<(u32, u32)>) -> Result<()> {
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(module)
    .map_err(|e| e.to_string())?;
    if module.entry_points.len() != 1 {
        return Err("expected one legacy entry point".into());
    }
    for (handle, var) in module.global_variables.iter() {
        if info.get_entry_point(0)[handle].is_empty() {
            continue;
        }
        if let Some(binding) = &var.binding {
            let uniform = binding.binding == 0
                && [stage, 2].contains(&binding.group)
                && var.space == naga::AddressSpace::Uniform;
            let texture = stage == 1
                && binding.group == 1
                && (16..48).contains(&binding.binding)
                && var.space == naga::AddressSpace::Handle;
            if !uniform && !texture {
                return Err(
                    "legacy draw resource outside float constants/2D pixel texture profile".into(),
                );
            }
            used.insert((binding.group, binding.binding));
        }
    }
    Ok(())
}
impl Device {
    pub fn draw_legacy(&self, target: &mut Target, draw: &LegacyDraw<'_>) -> Result<()> {
        self.draw_legacy_state(target, draw, &RenderState::default())
    }
    pub fn draw_legacy_state(
        &self,
        target: &mut Target,
        draw: &LegacyDraw<'_>,
        state: &RenderState,
    ) -> Result<()> {
        self.draw_legacy_depth_state(target, draw, state, None)
    }
    pub fn draw_legacy_depth_state(
        &self,
        target: &mut Target,
        draw: &LegacyDraw<'_>,
        state: &RenderState,
        depth: Option<&Depth>,
    ) -> Result<()> {
        self.draw_legacy_viewport_depth_state(
            target,
            draw,
            state,
            depth,
            [0., 0., target.width as f32, target.height as f32, 0., 1.],
        )
    }
    pub fn draw_legacy_viewport_depth_state(
        &self,
        target: &mut Target,
        draw: &LegacyDraw<'_>,
        state: &RenderState,
        depth: Option<&Depth>,
        viewport: [f32; 6],
    ) -> Result<()> {
        validate_viewport(viewport, target.width, target.height)?;
        let depth = if state.get(7)? != 0 || state.get(52)? != 0 {
            depth
        } else {
            None
        };
        depth::validate_depth(target, depth)?;
        let depth_state = state.depth_state(depth)?;
        let depth_view = depth.map(Depth::attachment);
        if !target.initialized.load(Ordering::Acquire)
            || draw.stride == 0
            || draw.stride > 2048
            || draw.vertices.is_empty()
            || !draw.vertices.len().is_multiple_of(draw.stride as usize)
            || draw.indices.is_none()
                && !(draw.vertices.len() / draw.stride as usize).is_multiple_of(3)
            || draw.vertices.len() > 64 * 1024 * 1024
        {
            return Err(format!(
                "legacy draw extent/state outside bounded triangle profile: initialized={} bytes={} stride={} indexed={}",
                target.initialized.load(Ordering::Acquire), draw.vertices.len(), draw.stride, draw.indices.is_some()
            ));
        }
        if let Some(indices) = draw.indices
            && (indices.is_empty()
                || !indices.len().is_multiple_of(3)
                || indices.len() > 3 * 1024 * 1024
                || indices
                    .iter()
                    .any(|&index| index as usize >= draw.vertices.len() / draw.stride as usize))
        {
            return Err("indexed draw range outside vertex storage or triangle budget".into());
        }
        let mut vs = legacy::lower_spirv(draw.vertex_spirv, legacy::Stage::Vertex)?;
        legacy::vertex_pixel_centers(&mut vs)?;
        let mut ps = legacy::lower_spirv(draw.pixel_spirv, legacy::Stage::Pixel)?;
        if depth.is_some() {
            crate::depth_bias::lower(
                &mut ps,
                f32::from_bits(state.get(195)?),
                f32::from_bits(state.get(175)?),
            )?;
        }
        let mut used = BTreeSet::new();
        resources(&vs, 0, &mut used)?;
        resources(&ps, 1, &mut used)?;
        let vs_text = legacy::wgsl(&vs)?;
        let ps_text = legacy::wgsl(&ps)?;
        let viewport_data: Vec<u8> = [viewport[2], viewport[3], viewport[0], viewport[1]]
            .iter()
            .flat_map(|n| n.to_le_bytes())
            .collect();
        let data = [draw.vertex_constants, draw.pixel_constants, &viewport_data];
        for (n, bytes) in data.iter().enumerate() {
            if used.contains(&(n as u32, 0))
                && (bytes.is_empty() || !bytes.len().is_multiple_of(16))
            {
                return Err("missing/alignment-invalid legacy constant buffer".into());
            }
        }
        let mut textures = BTreeSet::new();
        for &(group, binding) in &used {
            if group == 1 && binding >= 16 {
                textures.insert((binding - 16) % 16);
            }
        }
        for &slot in &textures {
            let input = draw
                .textures
                .iter()
                .find(|t| t.slot == slot)
                .ok_or("pixel shader texture slot is not bound")?;
            if input.texture.shares_target(target) {
                return Err("D3D9 draw cannot sample its current render target".into());
            }
            input
                .texture
                .view(input.sampler.srgb())
                .map_err(|e| format!("pixel texture slot {slot}: {e}"))?;
        }
        self.scope();
        let texture_resources: Vec<_> = textures
            .iter()
            .map(|&slot| {
                let input = draw
                    .textures
                    .iter()
                    .find(|t| t.slot == slot)
                    .ok_or("pixel shader texture slot is not bound")?;
                let view = input.texture.view(input.sampler.srgb())?;
                let sampler = self.device.create_sampler(&input.sampler.descriptor());
                Ok((slot, view, sampler))
            })
            .collect::<Result<_>>()?;
        let vertex = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("translated D3D9 vertex shader"),
                source: wgpu::ShaderSource::Wgsl(vs_text.into()),
            });
        let pixel = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("translated D3D9 pixel shader"),
                source: wgpu::ShaderSource::Wgsl(ps_text.into()),
            });
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("D3D9 programmable triangle"),
                layout: None,
                vertex: wgpu::VertexState {
                    module: &vertex,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: draw.stride,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: draw.attributes,
                    }],
                },
                primitive: state.primitive(),
                depth_stencil: depth_state,
                multisample: state.multisample(),
                fragment: Some(wgpu::FragmentState {
                    module: &pixel,
                    entry_point: Some("main"),
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
        let buffers: Vec<_> = (0..3)
            .map(|n| {
                used.contains(&(n as u32, 0)).then(|| {
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("D3D9 actual constants"),
                            contents: data[n],
                            usage: wgpu::BufferUsages::UNIFORM,
                        })
                })
            })
            .collect();
        let groups: Vec<_> = (0..3)
            .map(|n| {
                let mut entries = Vec::new();
                if let Some(buffer) = &buffers[n] {
                    entries.push(wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    });
                }
                if n == 1 {
                    for (slot, view, sampler) in &texture_resources {
                        if used.contains(&(1, 16 + slot)) {
                            entries.push(wgpu::BindGroupEntry {
                                binding: 16 + slot,
                                resource: wgpu::BindingResource::TextureView(view),
                            });
                        }
                        if used.contains(&(1, 32 + slot)) {
                            entries.push(wgpu::BindGroupEntry {
                                binding: 32 + slot,
                                resource: wgpu::BindingResource::Sampler(sampler),
                            });
                        }
                    }
                }
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("D3D9 actual stage resources"),
                    layout: &pipeline.get_bind_group_layout(n as u32),
                    entries: &entries,
                })
            })
            .collect();
        let vertices = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("D3D9 actual shader vertices"),
                contents: draw.vertices,
                usage: wgpu::BufferUsages::VERTEX,
            });
        let view = target.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(state.format()),
            ..Default::default()
        });
        let indices = draw.indices.map(|indices| {
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("D3D9 actual normalized index buffer"),
                    contents: &indices
                        .iter()
                        .flat_map(|n| n.to_le_bytes())
                        .collect::<Vec<_>>(),
                    usage: wgpu::BufferUsages::INDEX,
                })
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let query = self.draw_query()?;
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                depth_stencil_attachment: depth_view.as_ref().map(|view| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view,
                        depth_ops: depth.unwrap().depth_ops(state),
                        stencil_ops: depth.unwrap().stencil_ops(state),
                    }
                }),
                label: Some("D3D9 legacy shader draw"),
                occlusion_query_set: query.as_ref().map(|q| &q.set),
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
            pass.set_viewport(
                viewport[0],
                viewport[1],
                viewport[2],
                viewport[3],
                viewport[4],
                viewport[5],
            );
            for (n, group) in groups.iter().enumerate() {
                pass.set_bind_group(n as u32, group, &[]);
            }
            pass.set_vertex_buffer(0, vertices.slice(..));
            if query.is_some() {
                pass.begin_occlusion_query(0);
            }
            if let Some(indices) = &indices {
                pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..draw.indices.unwrap().len() as u32, 0, 0..1);
            } else {
                pass.draw(0..(draw.vertices.len() / draw.stride as usize) as u32, 0..1);
            }
            if query.is_some() {
                pass.end_occlusion_query();
            }
        }
        if let Some(query) = &query {
            query.resolve(&mut encoder);
        }
        self.queue.submit([encoder.finish()]);
        self.finish()?;
        if let Some(query) = query {
            query.submitted()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod viewport_tests {
    use super::*;
    #[test]
    fn native_viewport_words_check_bounds_finite_depth_and_wraparound() {
        assert_eq!(
            viewport_from_words([3, 5, 17, 11, 0.25f32.to_bits(), 0.75f32.to_bits()], 32, 32)
                .unwrap(),
            [3., 5., 17., 11., 0.25, 0.75]
        );
        for words in [
            [0, 0, 0, 32, 0, 1f32.to_bits()],
            [31, 0, 2, 32, 0, 1f32.to_bits()],
            [u32::MAX, 0, 2, 1, 0, 1f32.to_bits()],
            [0, 31, 1, 2, 0, 1f32.to_bits()],
            [0, 0, 1, 1, f32::NAN.to_bits(), 1f32.to_bits()],
            [0, 0, 1, 1, 0, f32::INFINITY.to_bits()],
            [0, 0, 1, 1, 0.75f32.to_bits(), 0.25f32.to_bits()],
            [0, 0, 1, 1, (-0.1f32).to_bits(), 1f32.to_bits()],
        ] {
            assert!(viewport_from_words(words, 32, 32).is_err());
        }
    }
}
