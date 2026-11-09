//! Actual BGRA8 mip storage and D3D9 sampling state. Unknown state fails.
use crate::{Device, Result, Target, extent};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
#[derive(Clone, Copy, Debug)]
pub struct SamplerState {
    values: [u32; 14],
}
impl Default for SamplerState {
    fn default() -> Self {
        Self {
            values: [0, 1, 1, 1, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0],
        }
    }
}
impl SamplerState {
    pub fn get(&self, ty: u32) -> Result<u32> {
        self.values
            .get(ty as usize)
            .copied()
            .filter(|_| (1..=13).contains(&ty))
            .ok_or("unknown sampler state".into())
    }
    pub fn set(&mut self, ty: u32, value: u32) -> Result<()> {
        let supported = match ty {
            1..=3 => (1..=3).contains(&value),
            4 | 8 | 12 | 13 => value == 0,
            5 | 6 => (1..=2).contains(&value),
            7 => value <= 2,
            9 => value <= 13,
            10 => value == 1,
            11 => true,
            _ => false,
        };
        if !supported {
            return Err("sampler state outside implemented profile".into());
        }
        self.values[ty as usize] = value;
        Ok(())
    }
    pub(crate) fn descriptor(&self) -> wgpu::SamplerDescriptor<'static> {
        let address = |n| match n {
            1 => wgpu::AddressMode::Repeat,
            2 => wgpu::AddressMode::MirrorRepeat,
            3 => wgpu::AddressMode::ClampToEdge,
            _ => unreachable!(),
        };
        let filter = |n| {
            if n == 2 {
                wgpu::FilterMode::Linear
            } else {
                wgpu::FilterMode::Nearest
            }
        };
        wgpu::SamplerDescriptor {
            label: Some("D3D9 actual sampler state"),
            address_mode_u: address(self.values[1]),
            address_mode_v: address(self.values[2]),
            address_mode_w: address(self.values[3]),
            mag_filter: filter(self.values[5]),
            min_filter: filter(self.values[6]),
            mipmap_filter: filter(self.values[7]),
            lod_min_clamp: self.values[9] as f32,
            lod_max_clamp: if self.values[7] == 0 {
                self.values[9] as f32
            } else {
                32.
            },
            ..Default::default()
        }
    }
    pub(crate) fn srgb(&self) -> bool {
        self.values[11] != 0
    }
}
/// D3D9's BC1/2/3 data stays compressed all the way to the GPU. Support is
/// conditional on the real adapter; there is no successful fallback texture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleFormat {
    Bgra8,
    Bc1,
    Bc2,
    Bc3,
}
impl SampleFormat {
    fn native(self) -> wgpu::TextureFormat {
        match self {
            Self::Bgra8 => wgpu::TextureFormat::Bgra8Unorm,
            Self::Bc1 => wgpu::TextureFormat::Bc1RgbaUnorm,
            Self::Bc2 => wgpu::TextureFormat::Bc2RgbaUnorm,
            Self::Bc3 => wgpu::TextureFormat::Bc3RgbaUnorm,
        }
    }
    pub fn block_layout(self, width: u32, height: u32) -> Result<(u32, u32)> {
        if width == 0 || height == 0 {
            return Err("zero texture block extent".into());
        }
        let (w, h) = self.native().block_dimensions();
        let bytes = self.native().block_copy_size(None).unwrap();
        let pitch = width
            .div_ceil(w)
            .checked_mul(bytes)
            .ok_or("texture block pitch overflow")?;
        let rows = height.div_ceil(h);
        pitch
            .checked_mul(rows)
            .ok_or("texture block length overflow")?;
        Ok((pitch, rows))
    }
    pub fn compressed(self) -> bool {
        self != Self::Bgra8
    }
}
pub struct Texture2D {
    texture: wgpu::Texture,
    format: SampleFormat,
    width: u32,
    height: u32,
    levels: u32,
    initialized: Vec<Arc<AtomicBool>>,
}
impl Texture2D {
    pub fn dimensions(&self, level: u32) -> Result<(u32, u32)> {
        if level >= self.levels {
            return Err("texture level outside allocation".into());
        }
        Ok(((self.width >> level).max(1), (self.height >> level).max(1)))
    }
    pub fn shares_target(&self, target: &Target) -> bool {
        self.initialized
            .first()
            .is_some_and(|v| Arc::ptr_eq(v, &target.initialized))
    }
    pub fn byte_layout(&self, level: u32) -> Result<(u32, u32)> {
        let (width, height) = self.dimensions(level)?;
        let (pitch, rows) = self.format.block_layout(width, height)?;
        Ok((pitch, pitch * rows))
    }
    pub fn levels(&self) -> u32 {
        self.levels
    }
    pub(crate) fn view(&self, srgb: bool) -> Result<wgpu::TextureView> {
        if self.initialized.iter().any(|v| !v.load(Ordering::Acquire)) {
            return Err("sampling an uninitialized texture mip".into());
        }
        Ok(self.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(if srgb {
                self.format.native().add_srgb_suffix()
            } else {
                self.format.native()
            }),
            ..Default::default()
        }))
    }
}
pub struct TextureBinding<'a> {
    pub slot: u32,
    pub texture: &'a Texture2D,
    pub sampler: SamplerState,
}
impl Device {
    /// The render attachment and sampled texture are two views of the same
    /// allocation; clear/draw initialization is shared without a CPU copy.
    pub fn create_render_texture(&self, width: u32, height: u32) -> Result<(Texture2D, Target)> {
        let target = self.create_target(width, height)?;
        let sampled = Texture2D {
            texture: target.texture.clone(),
            format: SampleFormat::Bgra8,
            width,
            height,
            levels: 1,
            initialized: vec![target.initialized.clone()],
        };
        Ok((sampled, target))
    }
    pub fn create_texture(&self, width: u32, height: u32, levels: u32) -> Result<Texture2D> {
        self.create_sampled_texture(width, height, levels, SampleFormat::Bgra8)
    }
    pub fn supports_bc(&self) -> bool {
        self.device
            .features()
            .contains(wgpu::Features::TEXTURE_COMPRESSION_BC)
    }
    pub fn create_sampled_texture(
        &self,
        width: u32,
        height: u32,
        levels: u32,
        format: SampleFormat,
    ) -> Result<Texture2D> {
        if format.compressed()
            && (!self.supports_bc() || !width.is_multiple_of(4) || !height.is_multiple_of(4))
        {
            return Err(
                "compressed texture needs actual BC support and block-aligned base dimensions"
                    .into(),
            );
        }
        extent(
            width,
            height,
            self.device.limits().max_texture_dimension_2d.min(8192),
        )?;
        let maximum = 32 - width.max(height).leading_zeros();
        let levels = if levels == 0 { maximum } else { levels };
        if levels > maximum {
            return Err("too many texture mip levels".into());
        }
        self.scope();
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("D3D9 real BGRA8 texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format.native(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[format.native().add_srgb_suffix()],
        });
        self.finish()?;
        Ok(Texture2D {
            texture,
            format,
            width,
            height,
            levels,
            initialized: (0..levels)
                .map(|_| Arc::new(AtomicBool::new(false)))
                .collect(),
        })
    }
    pub fn upload_texture(&self, texture: &mut Texture2D, level: u32, bytes: &[u8]) -> Result<()> {
        let (width, height) = texture.dimensions(level)?;
        let (pitch, rows) = texture.format.block_layout(width, height)?;
        if bytes.len() != pitch as usize * rows as usize {
            return Err("texture upload byte extent differs".into());
        }
        self.scope();
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture.texture,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(pitch),
                rows_per_image: Some(rows),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            }
            .physical_size(texture.format.native()),
        );
        self.finish()?;
        texture.initialized[level as usize].store(true, Ordering::Release);
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn block_rows_and_mip_tails_preserve_complete_encoded_blocks() {
        assert_eq!(SampleFormat::Bgra8.block_layout(3, 2).unwrap(), (12, 2));
        assert_eq!(SampleFormat::Bc1.block_layout(8, 8).unwrap(), (16, 2));
        for format in [SampleFormat::Bc2, SampleFormat::Bc3] {
            assert_eq!(format.block_layout(8, 8).unwrap(), (32, 2));
            for (width, height) in [(1, 1), (2, 2), (4, 4)] {
                assert_eq!(format.block_layout(width, height).unwrap(), (16, 1));
            }
            assert!(format.block_layout(0, 4).is_err());
            assert!(format.block_layout(u32::MAX, u32::MAX).is_err());
        }
    }
    #[test]
    fn unsupported_sampler_settings_preserve_prior_state() {
        let mut s = SamplerState::default();
        assert_eq!(s.get(11).unwrap(), 0);
        assert_eq!(s.get(5).unwrap(), 1);
        for (ty, value) in [(0, 0), (14, 0), (1, 4), (5, 3), (10, 16), (8, 1)] {
            assert!(s.set(ty, value).is_err());
        }
        assert_eq!(s.get(1).unwrap(), 1);
        s.set(11, 7).unwrap();
        assert!(s.srgb());
        assert!(s.get(0).is_err());
        assert!(s.get(14).is_err());
    }
}
