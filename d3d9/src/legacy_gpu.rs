//! Real programmable draws over translated legacy bytecode. Initial binding
//! profile: float constants, triangle lists and explicit float vertex attributes.
use super::{Depth, Device, RenderState, Result, Target, TextureBinding, depth, legacy};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    sync::Arc,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
/// Per-device bounded cache counters; these measure reuse, not game frame rate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LegacyPipelineStats {
    pub hits: u64,
    pub builds: u64,
    pub evictions: u64,
    pub entries: usize,
    pub key_bytes: usize,
}

/// Host wall times of completed programmable draws on this device.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LegacyDrawProfile {
    pub draws: u64,
    pub prepare: Duration,
    pub encode: Duration,
    pub submit: Duration,
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct PipelineKey {
    vs: Vec<u8>,
    ps: Vec<u8>,
    stride: u64,
    attributes: Vec<wgpu::VertexAttribute>,
    state: Vec<(u32, u32)>,
    depth: Option<wgpu::TextureFormat>,
}
impl PipelineKey {
    fn bytes(&self) -> usize {
        self.vs.len()
            + self.ps.len()
            + self.attributes.len() * std::mem::size_of::<wgpu::VertexAttribute>()
            + self.state.len() * 8
    }
}
struct PreparedPipeline {
    pipeline: wgpu::RenderPipeline,
    used: BTreeSet<(u32, u32)>,
    textures: BTreeSet<u32>,
    dimensions: BTreeMap<u32, naga::ImageDimension>,
}
pub(crate) struct PipelineCache {
    entries: HashMap<Arc<PipelineKey>, Arc<PreparedPipeline>>,
    order: VecDeque<Arc<PipelineKey>>,
    stats: LegacyPipelineStats,
    enabled: bool,
}
impl Default for PipelineCache {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            stats: Default::default(),
            enabled: std::env::var_os("ALKYHOL_D3D9_DISABLE_PIPELINE_CACHE").is_none(),
        }
    }
}
impl PipelineCache {
    const LIMIT: usize = 256;
    const BYTE_LIMIT: usize = 32 * 1024 * 1024;
    fn insert(&mut self, key: PipelineKey, pipeline: Arc<PreparedPipeline>) {
        let bytes = key.bytes();
        if !self.enabled || bytes > Self::BYTE_LIMIT {
            return;
        }
        while self.entries.len() >= Self::LIMIT || self.stats.key_bytes + bytes > Self::BYTE_LIMIT {
            let oldest = self.order.pop_front().unwrap();
            self.stats.key_bytes -= oldest.bytes();
            self.entries.remove(&oldest);
            self.stats.evictions += 1;
        }
        let key = Arc::new(key);
        self.entries.insert(key.clone(), pipeline);
        self.order.push_back(key);
        self.stats.key_bytes += bytes;
        self.stats.entries = self.entries.len();
    }
}
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
fn texture_dimensions(
    module: &naga::Module,
    used: &BTreeSet<(u32, u32)>,
) -> Result<BTreeMap<u32, naga::ImageDimension>> {
    let mut result = BTreeMap::new();
    for (_, global) in module.global_variables.iter() {
        let Some(binding) = &global.binding else {
            continue;
        };
        if binding.group != 1
            || !(16..32).contains(&binding.binding)
            || !used.contains(&(1, binding.binding))
        {
            continue;
        }
        let naga::TypeInner::Image {
            dim,
            arrayed: false,
            class:
                naga::ImageClass::Sampled {
                    kind: naga::ScalarKind::Float,
                    multi: false,
                },
        } = module.types[global.ty].inner
        else {
            return Err("legacy pixel texture requires a nonarrayed float sampled image".into());
        };
        if !matches!(
            dim,
            naga::ImageDimension::D2 | naga::ImageDimension::D3 | naga::ImageDimension::Cube
        ) {
            return Err("legacy pixel texture dimension is outside the D3D9 profile".into());
        }
        if result.insert(binding.binding - 16, dim).is_some() {
            return Err("conflicting pixel texture binding".into());
        }
    }
    Ok(result)
}
impl Device {
    pub fn legacy_upload_stats(&self) -> crate::LegacyUploadStats {
        self.legacy_uploads.lock().unwrap().stats
    }
    pub fn legacy_pipeline_cache_stats(&self) -> LegacyPipelineStats {
        self.legacy_pipelines.lock().unwrap().stats
    }
    /// Return and reset the accumulated opt-in programmable draw timings.
    pub fn take_legacy_draw_profile(&self) -> LegacyDrawProfile {
        std::mem::take(&mut self.legacy_uploads.lock().unwrap().profile)
    }
    fn prepare_pipeline(
        &self,
        draw: &LegacyDraw<'_>,
        state: &RenderState,
        depth: Option<&Depth>,
        depth_state: Option<wgpu::DepthStencilState>,
    ) -> Result<Arc<PreparedPipeline>> {
        let key = PipelineKey {
            vs: draw.vertex_spirv.to_vec(),
            ps: draw.pixel_spirv.to_vec(),
            stride: draw.stride,
            attributes: draw.attributes.to_vec(),
            state: state.pipeline_words(),
            depth: depth.map(|d| d.texture.format()),
        };
        let mut cache = self.legacy_pipelines.lock().unwrap();
        if let Some(prepared) = cache.entries.get(&key).cloned() {
            cache.stats.hits += 1;
            return Ok(prepared);
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
        crate::alpha_test::lower(&mut ps, state.get(15)? != 0, state.get(25)?, state.get(24)?)?;
        let mut used = BTreeSet::new();
        resources(&vs, 0, &mut used)?;
        resources(&ps, 1, &mut used)?;
        let vs_text = legacy::wgsl(&vs)?;
        let ps_text = legacy::wgsl(&ps)?;
        let mut textures = BTreeSet::new();
        for &(group, binding) in &used {
            if group == 1 && binding >= 16 {
                textures.insert((binding - 16) % 16);
            }
        }
        let dimensions = texture_dimensions(&ps, &used)?;
        if textures.iter().any(|slot| !dimensions.contains_key(slot)) {
            return Err("pixel sampler has no supported sampled image declaration".into());
        }
        self.scope();
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
        self.finish()?;
        let prepared = Arc::new(PreparedPipeline {
            pipeline,
            used,
            textures,
            dimensions,
        });
        cache.stats.builds += 1;
        cache.insert(key, prepared.clone());
        Ok(prepared)
    }
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
        let started = self.legacy_profile_enabled().then(Instant::now);
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
                target.initialized.load(Ordering::Acquire),
                draw.vertices.len(),
                draw.stride,
                draw.indices.is_some()
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
        let prepared = self.prepare_pipeline(draw, state, depth, depth_state)?;
        let used = &prepared.used;
        let textures = &prepared.textures;
        let dimensions = &prepared.dimensions;
        let pipeline = &prepared.pipeline;
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
        // Validate real bindings first. A locked/uninitialized/mismatched real
        // resource must not acquire the NULL sampler behavior.
        for &slot in textures {
            if let Some(input) = draw.textures.iter().find(|t| t.slot == slot) {
                if input.texture.shares_target(target) {
                    return Err("D3D9 draw cannot sample its current render target".into());
                }
                input
                    .texture
                    .view(input.sampler.srgb())
                    .map_err(|e| format!("pixel texture slot {slot}: {e}"))?;
            }
        }
        // Initialize private NULL views before entering this draw's error scope.
        let texture_resources: Vec<_> = textures
            .iter()
            .map(|&slot| {
                let (view, sampler) =
                    if let Some(input) = draw.textures.iter().find(|t| t.slot == slot) {
                        (input.texture.view(input.sampler.srgb())?, input.sampler)
                    } else {
                        (
                            self.null_texture_view(dimensions[&slot])?,
                            crate::SamplerState::default(),
                        )
                    };
                Ok((slot, view, self.legacy_sampler(sampler)?))
            })
            .collect::<Result<_>>()?;
        if data.iter().enumerate().any(|(n, bytes)| {
            used.contains(&(n as u32, 0))
                && bytes.len() as u64 > self.device.limits().max_uniform_buffer_binding_size as u64
        }) {
            return Err("legacy constants exceed the actual uniform binding limit".into());
        }
        let index_data = draw.indices.map(|indices| {
            indices
                .iter()
                .flat_map(|n| n.to_le_bytes())
                .collect::<Vec<_>>()
        });
        let mut uploads = self.legacy_uploads.lock().unwrap();
        uploads.drain_if_over_budget(self, crate::legacy_upload::STAGING_BUDGET)?;
        let encode_started = started.map(|_| Instant::now());
        self.scope();
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let buffers: Vec<_> = (0..3)
            .map(|n| {
                used.contains(&(n as u32, 0))
                    .then(|| uploads.write(self, &mut encoder, n, data[n]))
            })
            .collect();
        let groups: Vec<_> = (0..3)
            .map(|n| {
                let mut entries = Vec::new();
                if let Some(buffer) = &buffers[n] {
                    entries.push(wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer,
                            offset: 0,
                            size: wgpu::BufferSize::new(data[n].len() as u64),
                        }),
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
        let vertices = uploads.write(self, &mut encoder, 3, draw.vertices);
        let view = target.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(state.format()),
            ..Default::default()
        });
        let indices = index_data
            .as_ref()
            .map(|bytes| uploads.write(self, &mut encoder, 4, bytes));
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
            pass.set_pipeline(pipeline);
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
            pass.set_vertex_buffer(0, vertices.slice(..draw.vertices.len() as u64));
            if query.is_some() {
                pass.begin_occlusion_query(0);
            }
            if let Some(indices) = &indices {
                pass.set_index_buffer(
                    indices.slice(..index_data.as_ref().unwrap().len() as u64),
                    wgpu::IndexFormat::Uint32,
                );
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
        uploads.finish();
        let submit_started = started.map(|_| Instant::now());
        self.queue.submit([encoder.finish()]);
        uploads.recall();
        self.finish()?;
        if let Some(query) = query {
            query.submitted()?;
        }
        if let (Some(started), Some(encode_started), Some(submit_started)) =
            (started, encode_started, submit_started)
        {
            uploads.profile.draws += 1;
            uploads.profile.prepare += encode_started - started;
            uploads.profile.encode += submit_started - encode_started;
            uploads.profile.submit += submit_started.elapsed();
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

#[cfg(test)]
mod null_tests {
    use super::*;
    #[test]
    fn null_resource_dimensions_follow_used_float_image_declarations() {
        for (ty, dimension) in [
            ("texture_2d<f32>", naga::ImageDimension::D2),
            ("texture_cube<f32>", naga::ImageDimension::Cube),
            ("texture_3d<f32>", naga::ImageDimension::D3),
        ] {
            let module =
                naga::front::wgsl::parse_str(&format!("@group(1) @binding(23) var image: {ty};"))
                    .unwrap();
            assert_eq!(
                texture_dimensions(&module, &BTreeSet::from([(1, 23)])).unwrap(),
                BTreeMap::from([(7, dimension)])
            );
            assert!(
                texture_dimensions(&module, &BTreeSet::new())
                    .unwrap()
                    .is_empty()
            );
        }
        for ty in [
            "texture_2d<u32>",
            "texture_2d_array<f32>",
            "texture_multisampled_2d<f32>",
            "texture_depth_2d",
            "texture_1d<f32>",
        ] {
            let module =
                naga::front::wgsl::parse_str(&format!("@group(1) @binding(16) var image: {ty};"))
                    .unwrap();
            assert!(texture_dimensions(&module, &BTreeSet::from([(1, 16)])).is_err());
        }
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    fn key() -> PipelineKey {
        PipelineKey {
            vs: vec![1, 2, 3],
            ps: vec![4, 5, 6],
            stride: 32,
            attributes: vec![wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            }],
            state: RenderState::default().pipeline_words(),
            depth: None,
        }
    }
    #[test]
    fn exact_pipeline_identity_covers_shaders_layout_attachment_and_every_static_state() {
        let original = key();
        for at in 0..original.state.len() {
            let mut changed = key();
            changed.state[at].1 ^= 1;
            assert!(changed != original);
        }
        let mut variants = vec![key(); 7];
        variants[0].vs[0] ^= 1;
        variants[1].ps[0] ^= 1;
        variants[2].stride += 4;
        variants[3].attributes[0].offset += 4;
        variants[4].attributes[0].shader_location += 1;
        variants[5].attributes[0].format = wgpu::VertexFormat::Float32x3;
        variants[6].depth = Some(wgpu::TextureFormat::Depth16Unorm);
        assert!(variants.iter().all(|v| *v != original));
        let mut state = RenderState::default();
        state.set(57, 123).unwrap();
        state.set(193, 0xaabbccdd).unwrap();
        assert_eq!(state.pipeline_words(), original.state);
        assert!(!original.state.iter().any(|(id, _)| [57, 193].contains(id)));
    }
    #[test]
    #[ignore = "requires an actual GPU"]
    fn bounded_fifo_admission_releases_keys_and_rejects_oversize_and_disabled_entries() {
        let device = Device::new().unwrap();
        let attributes = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 16,
                shader_location: 1,
            },
        ];
        let draw = LegacyDraw {
            indices: None,
            vertex_spirv: include_bytes!("../tests/fixtures/legacy-color.vs.spv"),
            pixel_spirv: include_bytes!("../tests/fixtures/legacy-color.ps.spv"),
            vertices: &[],
            stride: 32,
            attributes: &attributes,
            vertex_constants: &[],
            pixel_constants: &[],
            textures: &[],
        };
        let prepared = device
            .prepare_pipeline(&draw, &RenderState::default(), None, None)
            .unwrap();
        let mut cache = PipelineCache {
            enabled: true,
            ..Default::default()
        };
        for n in 0..=PipelineCache::LIMIT {
            let mut k = key();
            k.stride = n as u64;
            cache.insert(k, prepared.clone());
        }
        assert_eq!(cache.stats.entries, PipelineCache::LIMIT);
        assert_eq!(cache.stats.evictions, 1);
        let mut first = key();
        first.stride = 0;
        assert!(!cache.entries.contains_key(&first));
        let bytes = cache.stats.key_bytes;
        let mut oversized = key();
        oversized.vs = vec![0; PipelineCache::BYTE_LIMIT + 1];
        cache.insert(oversized, prepared.clone());
        assert_eq!(cache.stats.key_bytes, bytes);
        let mut large = key();
        large.ps = vec![0; PipelineCache::BYTE_LIMIT / 2];
        cache.insert(large.clone(), prepared.clone());
        large.stride += 1;
        cache.insert(large, prepared.clone());
        assert_eq!(cache.entries.len(), 1);
        assert!(cache.stats.key_bytes <= PipelineCache::BYTE_LIMIT);
        let mut disabled = PipelineCache {
            enabled: false,
            ..Default::default()
        };
        disabled.insert(key(), prepared);
        assert_eq!(disabled.stats, LegacyPipelineStats::default());
    }
}

#[cfg(test)]
mod cache_timing {
    use super::*;
    #[test]
    #[ignore = "requires an actual GPU; prints finite draw-batch timing, not game FPS"]
    fn cached_and_uncached_draws_match_and_failed_pipelines_are_not_admitted() {
        let mut device = Device::new().unwrap();
        device.legacy_profile_enabled = true;
        let mut target = device.create_target(32, 32).unwrap();
        let vertices: Vec<u8> = [(0f32, 0.75f32), (0.75, -0.75), (-0.75, -0.75)]
            .into_iter()
            .flat_map(|(x, y)| {
                [x, y, 0.5, 1., 1., 0., 0., 1.]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
            })
            .collect();
        let constants: Vec<u8> = [
            1f32, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
        ]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect();
        let attributes = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 16,
                shader_location: 1,
            },
        ];
        let mut draw = LegacyDraw {
            indices: None,
            vertex_spirv: include_bytes!("../tests/fixtures/legacy-color.vs.spv"),
            pixel_spirv: include_bytes!("../tests/fixtures/legacy-color.ps.spv"),
            vertices: &vertices,
            stride: 32,
            attributes: &attributes,
            vertex_constants: &constants,
            pixel_constants: &[],
            textures: &[],
        };
        let mut reference = None;
        for (enabled, reuse) in [(false, false), (false, true), (true, false), (true, true)] {
            {
                let mut cache = device.legacy_pipelines.lock().unwrap();
                *cache = PipelineCache {
                    enabled,
                    ..Default::default()
                };
            }
            {
                let mut uploads = device.legacy_uploads.lock().unwrap();
                *uploads = crate::legacy_upload::Uploads::default();
                uploads.reuse = reuse;
            }
            device.clear(&mut target, 0xff0000ff).unwrap();
            device.draw_legacy(&mut target, &draw).unwrap();
            device.readback(&target).unwrap();
            assert_eq!(device.take_legacy_draw_profile().draws, 1);
            let start = std::time::Instant::now();
            for _ in 0..128 {
                device.draw_legacy(&mut target, &draw).unwrap();
            }
            let pixels = device.readback(&target).unwrap();
            let elapsed = start.elapsed();
            let profile = device.take_legacy_draw_profile();
            assert_eq!(profile.draws, 128);
            let measured = profile.prepare + profile.encode + profile.submit;
            assert!(measured > Duration::ZERO && measured <= elapsed);
            assert_eq!(
                device.take_legacy_draw_profile(),
                LegacyDrawProfile::default()
            );
            let stats = device.legacy_pipeline_cache_stats();
            eprintln!(
                "finite_pipeline_batch pipeline_cache={enabled} upload_reuse={reuse} draws=128 seconds={:.6} stats={stats:?}",
                elapsed.as_secs_f64()
            );
            eprintln!("finite_draw_profile {profile:?}");
            assert_eq!(stats.builds, if enabled { 1 } else { 129 });
            assert_eq!(stats.hits, if enabled { 128 } else { 0 });
            let upload_stats = device.legacy_upload_stats();
            assert_eq!(upload_stats.allocations, if reuse { 3 } else { 387 });
            assert_eq!(upload_stats.writes, 387);
            eprintln!("finite_upload_allocations {upload_stats:?}");
            if let Some(ref pixels_before) = reference {
                assert_eq!(&pixels, pixels_before);
            } else {
                reference = Some(pixels);
            }
        }
        let before = device.legacy_pipeline_cache_stats();
        draw.stride = 16; // attributes no longer fit: genuine wgpu pipeline failure
        assert!(device.draw_legacy(&mut target, &draw).is_err());
        assert_eq!(device.legacy_pipeline_cache_stats(), before);
        draw.stride = 32;
        draw.vertex_constants = &[]; // the warm pipeline must still reject missing uniforms
        assert!(device.draw_legacy(&mut target, &draw).is_err());
        assert_eq!(device.legacy_pipeline_cache_stats().builds, before.builds);
        draw.vertex_constants = &constants[..16]; // aligned but shorter than the shader's actual binding
        assert!(device.draw_legacy(&mut target, &draw).is_err());
        assert_eq!(device.take_legacy_draw_profile().draws, 0);
        draw.vertex_constants = &constants;
        device.draw_legacy(&mut target, &draw).unwrap(); // failure did not poison scopes/cache/uploads
    }
}
