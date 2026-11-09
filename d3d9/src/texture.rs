//! Actual BGRA8 mip storage and D3D9 sampling state. Unknown state fails.
use crate::{Device, Result, extent};
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
pub struct Texture2D {
    texture: wgpu::Texture,
    width: u32,
    height: u32,
    levels: u32,
    initialized: Vec<bool>,
}
impl Texture2D {
    pub fn dimensions(&self, level: u32) -> Result<(u32, u32)> {
        if level >= self.levels {
            return Err("texture level outside allocation".into());
        }
        Ok(((self.width >> level).max(1), (self.height >> level).max(1)))
    }
    pub fn levels(&self) -> u32 {
        self.levels
    }
    pub(crate) fn view(&self, srgb: bool) -> Result<wgpu::TextureView> {
        if self.initialized.iter().any(|v| !*v) {
            return Err("sampling an uninitialized texture mip".into());
        }
        Ok(self.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(if srgb {
                wgpu::TextureFormat::Bgra8UnormSrgb
            } else {
                wgpu::TextureFormat::Bgra8Unorm
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
    pub fn create_texture(&self, width: u32, height: u32, levels: u32) -> Result<Texture2D> {
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
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
        });
        self.finish()?;
        Ok(Texture2D {
            texture,
            width,
            height,
            levels,
            initialized: vec![false; levels as usize],
        })
    }
    pub fn upload_texture(&self, texture: &mut Texture2D, level: u32, bytes: &[u8]) -> Result<()> {
        let (width, height) = texture.dimensions(level)?;
        if bytes.len() != width as usize * height as usize * 4 {
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
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.finish()?;
        texture.initialized[level as usize] = true;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
