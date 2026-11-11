//! Reuse draw destinations and suballocate asynchronous staging writes. Copies
//! and their draw share one encoder; the mutex is held through queue submission.
use crate::Device;
use std::collections::HashSet;
use wgpu::util::DeviceExt;

const STAGING_CHUNK_SIZE: u64 = 64 * 1024;
pub(crate) const STAGING_BUDGET: u64 = 32 * 1024 * 1024;
pub(crate) const DRAW_BATCH_LIMIT: usize = 32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LegacyUploadStats {
    pub allocations: u64,
    pub writes: u64,
    pub resident_bytes: u64,
    pub staging_drains: u64,
    pub staging_peak_bytes: u64,
    pub submissions: u64,
}
pub(crate) struct Uploads {
    buffers: [Option<wgpu::Buffer>; 5],
    pub(crate) reuse: bool,
    belt: wgpu::util::StagingBelt,
    staging_buffers: HashSet<wgpu::Buffer>,
    staging_bytes: u64,
    pub(crate) pending: Vec<wgpu::CommandBuffer>,
    pub(crate) stats: LegacyUploadStats,
    pub(crate) profile: crate::LegacyDrawProfile,
}
impl Default for Uploads {
    fn default() -> Self {
        Self {
            buffers: Default::default(),
            reuse: std::env::var_os("ALKYHOL_D3D9_DISABLE_UPLOAD_REUSE").is_none(),
            belt: wgpu::util::StagingBelt::new(STAGING_CHUNK_SIZE),
            staging_buffers: HashSet::new(),
            staging_bytes: 0,
            pending: Vec::new(),
            stats: Default::default(),
            profile: Default::default(),
        }
    }
}
impl Uploads {
    // A single checked draw can exceed the soft budget. Reclaim the belt before
    // the next draw, after all copies from its staging buffers have completed.
    pub(crate) fn drain_if_over_budget(
        &mut self,
        device: &Device,
        budget: u64,
    ) -> crate::Result<()> {
        if self.staging_bytes <= budget {
            return Ok(());
        }
        self.submit_pending(device)?;
        device
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| e.to_string())?;
        self.belt = wgpu::util::StagingBelt::new(STAGING_CHUNK_SIZE);
        self.staging_buffers.clear();
        self.staging_bytes = 0;
        self.stats.staging_drains += 1;
        Ok(())
    }
    pub(crate) fn write(
        &mut self,
        device: &Device,
        encoder: &mut wgpu::CommandEncoder,
        slot: usize,
        data: &[u8],
    ) -> wgpu::Buffer {
        let size = data.len() as u64;
        let usage = match slot {
            0..=2 => wgpu::BufferUsages::UNIFORM,
            3 => wgpu::BufferUsages::VERTEX,
            4 => wgpu::BufferUsages::INDEX,
            _ => unreachable!(),
        };
        if !self.reuse {
            self.stats.allocations += 1;
            self.stats.writes += 1;
            return device
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("D3D9 uncached comparison upload"),
                    contents: data,
                    usage,
                });
        }
        if self.buffers[slot].as_ref().is_none_or(|b| b.size() < size) {
            let capacity = size
                .next_power_of_two()
                .max(16)
                .min(device.device.limits().max_buffer_size);
            let buffer = device.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("D3D9 reusable draw upload"),
                size: capacity,
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.stats.resident_bytes -= self.buffers[slot].as_ref().map_or(0, wgpu::Buffer::size);
            self.stats.resident_bytes += capacity;
            self.stats.allocations += 1;
            self.buffers[slot] = Some(buffer);
        }
        let buffer = self.buffers[slot].as_ref().unwrap();
        let slice = self.belt.allocate(
            wgpu::BufferSize::new(size).unwrap(),
            wgpu::BufferSize::new(wgpu::COPY_BUFFER_ALIGNMENT).unwrap(),
            &device.device,
        );
        let staging = slice.buffer();
        // A chunk can serve several writes and return for later submissions.
        // Count the backing allocation once, including its unused capacity.
        if !self.staging_buffers.contains(staging) {
            self.staging_buffers.insert(staging.clone());
            self.staging_bytes += staging.size();
            self.stats.staging_peak_bytes = self.stats.staging_peak_bytes.max(self.staging_bytes);
        }
        slice.get_mapped_range_mut().copy_from_slice(data);
        encoder.copy_buffer_to_buffer(staging, slice.offset(), buffer, 0, size);
        self.stats.writes += 1;
        buffer.clone()
    }
    pub(crate) fn submit(&mut self, device: &Device, command: Option<wgpu::CommandBuffer>) {
        self.belt.finish();
        device.queue.submit(self.pending.drain(..).chain(command));
        self.belt.recall();
        self.stats.submissions += 1;
    }

    pub(crate) fn submit_pending(&mut self, device: &Device) -> crate::Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let started = device
            .legacy_profile_enabled()
            .then(std::time::Instant::now);
        device.scope();
        self.submit(device, None);
        let result = device.finish();
        if let Some(started) = started {
            self.profile.submit += started.elapsed();
        }
        result
    }

    pub(crate) fn enqueue(
        &mut self,
        device: &Device,
        command: wgpu::CommandBuffer,
    ) -> crate::Result<()> {
        self.pending.push(command);
        if self.pending.len() >= DRAW_BATCH_LIMIT || self.staging_bytes > STAGING_BUDGET {
            self.submit_pending(device)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LegacyDraw, VertexAttribute, VertexFormat};
    #[test]
    #[ignore = "requires an actual GPU"]
    fn queued_draws_keep_their_data_across_growth_shrink_and_staging_drain() {
        let device = Device::new().unwrap();
        let attributes = [
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 16,
                shader_location: 1,
            },
        ];
        let mut targets = Vec::new();
        for _ in 0..16 {
            let mut target = device.create_target(32, 32).unwrap();
            device.clear(&mut target, 0xff0000ff).unwrap();
            targets.push(target);
        }
        let mut frames = Vec::new();
        for (n, mut target) in targets.into_iter().enumerate() {
            if n == 8 {
                let mut uploads = device.legacy_uploads.lock().unwrap();
                let bytes = uploads.staging_bytes;
                assert!(bytes > 0 && bytes <= n as u64 * STAGING_CHUNK_SIZE);
                uploads.drain_if_over_budget(&device, bytes).unwrap();
                assert_eq!(uploads.stats.staging_drains, 0);
                uploads.drain_if_over_budget(&device, bytes - 1).unwrap();
                assert!(uploads.staging_buffers.is_empty());
                assert_eq!(uploads.staging_bytes, 0);
            }
            let green = n % 2 != 0;
            let color = if green {
                [0f32, 1., 0., 1.]
            } else {
                [1f32, 0., 0., 1.]
            };
            let mut vertices = Vec::new();
            for _ in 0..if green { 2 } else { 1 } {
                for (x, y) in [(0f32, 0.75f32), (0.75, -0.75), (-0.75, -0.75)] {
                    for v in [x, y, 0.5, 1.].into_iter().chain(color) {
                        vertices.extend(v.to_le_bytes());
                    }
                }
            }
            let mut constants: Vec<u8> = [
                1f32, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
            ]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect();
            if green {
                constants.extend_from_within(..);
            }
            let indices: Vec<u32> = (0..if green { 6 } else { 3 }).collect();
            let mut draw = LegacyDraw {
                indices: Some(&indices),
                vertex_spirv: include_bytes!("../tests/fixtures/legacy-color.vs.spv"),
                pixel_spirv: include_bytes!("../tests/fixtures/legacy-color.ps.spv"),
                vertices: &vertices,
                stride: 32,
                attributes: &attributes,
                vertex_constants: &constants,
                pixel_constants: &[],
                textures: &[],
            };
            device
                .queue_legacy_viewport_depth_state(
                    &mut target,
                    &draw,
                    &crate::RenderState::default(),
                    None,
                    [0., 0., 32., 32., 0., 1.],
                )
                .unwrap();
            if n == 7 {
                draw.vertex_constants = &constants[..16];
                assert!(
                    device
                        .queue_legacy_viewport_depth_state(
                            &mut target,
                            &draw,
                            &crate::RenderState::default(),
                            None,
                            [0., 0., 32., 32., 0., 1.],
                        )
                        .is_err()
                );
            }
            frames.push((target, green));
        }
        device.flush_legacy_draws().unwrap();
        // Deliberately defer all readbacks: a later upload must not overwrite
        // an earlier submission's constants, vertices or normalized indices.
        for (target, green) in frames {
            let pixels = device.readback(&target).unwrap();
            let at = (20 * 32 + 22) * 4;
            assert_eq!(
                &pixels[at..at + 4],
                if green {
                    &[0, 255, 0, 255]
                } else {
                    &[0, 0, 255, 255]
                }
            );
            assert_eq!(&pixels[..4], &[255, 0, 0, 255]);
        }
        let stats = device.legacy_upload_stats();
        assert_eq!(stats.allocations, 7);
        assert_eq!(stats.writes, 68);
        assert_eq!(stats.resident_bytes, 432);
        assert_eq!(stats.staging_drains, 1);
        assert_eq!(stats.submissions, 2);
    }
}
