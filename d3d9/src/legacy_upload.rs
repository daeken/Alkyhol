//! Reuse draw destinations and suballocate asynchronous staging writes. Copies
//! and their draw share one encoder; the mutex is held through queue submission.
use crate::Device;
use wgpu::util::DeviceExt;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LegacyUploadStats {
    pub allocations: u64,
    pub writes: u64,
    pub resident_bytes: u64,
    pub staging_drains: u64,
}
pub(crate) struct Uploads {
    buffers: [Option<wgpu::Buffer>; 5],
    pub(crate) reuse: bool,
    belt: wgpu::util::StagingBelt,
    budget: u64,
    pub(crate) stats: LegacyUploadStats,
}
impl Default for Uploads {
    fn default() -> Self {
        Self {
            buffers: Default::default(),
            reuse: std::env::var_os("ALKYHOL_D3D9_DISABLE_UPLOAD_REUSE").is_none(),
            belt: wgpu::util::StagingBelt::new(65536),
            budget: 0,
            stats: Default::default(),
        }
    }
}
impl Uploads {
    // Bound staging residency between explicit GPU completion/drain checkpoints.
    // A single checked draw can exceed this soft budget without clipping data.
    pub(crate) fn reserve(
        &mut self,
        device: &Device,
        lengths: impl Iterator<Item = usize>,
    ) -> crate::Result<()> {
        if !self.reuse {
            return Ok(());
        }
        let estimate: u64 = lengths
            .filter(|n| *n != 0)
            .map(|n| (n as u64).div_ceil(65536) * 65536)
            .sum();
        if self.budget != 0 && self.budget + estimate > 32 * 1024 * 1024 {
            device
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .map_err(|e| e.to_string())?;
            // Completed callbacks own no unfinished work; dropping the belt also
            // releases peak free staging buffers instead of retaining them forever.
            self.belt = wgpu::util::StagingBelt::new(65536);
            self.budget = 0;
            self.stats.staging_drains += 1;
        }
        self.budget += estimate;
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
        self.belt
            .write_buffer(
                encoder,
                buffer,
                0,
                wgpu::BufferSize::new(size).unwrap(),
                &device.device,
            )
            .copy_from_slice(data);
        self.stats.writes += 1;
        buffer.clone()
    }
    pub(crate) fn finish(&mut self) {
        self.belt.finish();
    }
    pub(crate) fn recall(&mut self) {
        self.belt.recall();
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
        let mut frames = Vec::new();
        for n in 0..16 {
            if n == 8 {
                device.legacy_uploads.lock().unwrap().budget = 32 * 1024 * 1024;
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
            let mut target = device.create_target(32, 32).unwrap();
            device.clear(&mut target, 0xff0000ff).unwrap();
            device
                .draw_legacy(
                    &mut target,
                    &LegacyDraw {
                        indices: Some(&indices),
                        vertex_spirv: include_bytes!("../tests/fixtures/legacy-color.vs.spv"),
                        pixel_spirv: include_bytes!("../tests/fixtures/legacy-color.ps.spv"),
                        vertices: &vertices,
                        stride: 32,
                        attributes: &attributes,
                        vertex_constants: &constants,
                        pixel_constants: &[],
                        textures: &[],
                    },
                )
                .unwrap();
            frames.push((target, green));
        }
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
        assert_eq!(stats.writes, 64);
        assert_eq!(stats.resident_bytes, 432);
        assert_eq!(stats.staging_drains, 1);
    }
}
