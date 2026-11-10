//! Actual BGRA8 mip storage and D3D9 sampling state. Unknown state fails.
use crate::{Device, Result, Target, extent};
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    Luminance8,
    Alpha8,
    Argb4444,
    Bc1,
    Bc2,
    Bc3,
}
impl SampleFormat {
    fn native(self) -> wgpu::TextureFormat {
        match self {
            Self::Bgra8 | Self::Luminance8 | Self::Alpha8 | Self::Argb4444 => {
                wgpu::TextureFormat::Bgra8Unorm
            }
            Self::Bc1 => wgpu::TextureFormat::Bc1RgbaUnorm,
            Self::Bc2 => wgpu::TextureFormat::Bc2RgbaUnorm,
            Self::Bc3 => wgpu::TextureFormat::Bc3RgbaUnorm,
        }
    }
    pub fn block_layout(self, width: u32, height: u32) -> Result<(u32, u32)> {
        if width == 0 || height == 0 {
            return Err("zero texture block extent".into());
        }
        let size = match self {
            Self::Luminance8 | Self::Alpha8 => Some(1),
            Self::Argb4444 => Some(2),
            _ => None,
        };
        if let Some(size) = size {
            let pitch = width
                .checked_mul(size)
                .ok_or("linear texture pitch overflow")?;
            pitch
                .checked_mul(height)
                .ok_or("linear texture extent overflow")?;
            return Ok((pitch, height));
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
        matches!(self, Self::Bc1 | Self::Bc2 | Self::Bc3)
    }
}
pub struct SampledTexture {
    texture: wgpu::Texture,
    format: SampleFormat,
    width: u32,
    height: u32,
    levels: u32,
    layers: u32,
    depth: u32,
    volume: bool,
    initialized: Vec<Arc<AtomicBool>>,
    views: [OnceLock<wgpu::TextureView>; 2],
}
/// Historical source-compatible name for the sampled storage container.
pub type Texture2D = SampledTexture;
impl SampledTexture {
    pub fn is_cube(&self) -> bool {
        !self.volume && self.layers == 6
    }
    pub fn is_volume(&self) -> bool {
        self.volume
    }
    pub fn volume_dimensions(&self, level: u32) -> Result<(u32, u32, u32)> {
        if !self.volume {
            return Err("not volume storage".into());
        }
        let (width, height) = self.dimensions(level)?;
        Ok((width, height, (self.depth >> level).max(1)))
    }
    fn subresource(&self, level: u32, face: u32) -> Result<usize> {
        if level >= self.levels || face >= self.layers {
            return Err("texture face/mip outside allocation".into());
        }
        Ok((face * self.levels + level) as usize)
    }
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
        let depth = if self.volume {
            (self.depth >> level).max(1)
        } else {
            1
        };
        let bytes = pitch
            .checked_mul(rows)
            .and_then(|n| n.checked_mul(depth))
            .ok_or("texture mip length overflow")?;
        Ok((pitch, bytes))
    }
    pub fn levels(&self) -> u32 {
        self.levels
    }
    pub(crate) fn view(&self, srgb: bool) -> Result<wgpu::TextureView> {
        if self.initialized.iter().any(|v| !v.load(Ordering::Acquire)) {
            return Err("sampling an uninitialized texture mip".into());
        }
        if matches!(self.format, SampleFormat::Luminance8 | SampleFormat::Alpha8) && srgb {
            return Err("sRGB sampling of L8/A8 requires native format evidence".into());
        }
        Ok(self.views[usize::from(srgb)]
            .get_or_init(|| {
                self.texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(if self.volume {
                        wgpu::TextureViewDimension::D3
                    } else if self.is_cube() {
                        wgpu::TextureViewDimension::Cube
                    } else {
                        wgpu::TextureViewDimension::D2
                    }),
                    format: Some(if srgb {
                        self.format.native().add_srgb_suffix()
                    } else {
                        self.format.native()
                    }),
                    ..Default::default()
                })
            })
            .clone())
    }
}
pub struct TextureBinding<'a> {
    pub slot: u32,
    pub texture: &'a Texture2D,
    pub sampler: SamplerState,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LegacyResourceStats {
    pub sampler_builds: u64,
    pub sampler_hits: u64,
    pub sampler_evictions: u64,
    pub sampler_entries: usize,
}
#[derive(Default)]
pub(crate) struct Samplers {
    entries: std::collections::BTreeMap<SamplerState, wgpu::Sampler>,
    order: std::collections::VecDeque<SamplerState>,
    stats: LegacyResourceStats,
}
impl Device {
    pub fn legacy_resource_cache_stats(&self) -> LegacyResourceStats {
        self.legacy_samplers.lock().unwrap().stats
    }
    pub(crate) fn legacy_sampler(&self, state: SamplerState) -> Result<wgpu::Sampler> {
        let mut cache = self.legacy_samplers.lock().unwrap();
        if let Some(sampler) = cache.entries.get(&state).cloned() {
            cache.stats.sampler_hits += 1;
            return Ok(sampler);
        }
        self.scope();
        let sampler = self.device.create_sampler(&state.descriptor());
        self.finish()?;
        if cache.entries.len() >= 64 {
            let oldest = cache.order.pop_front().unwrap();
            cache.entries.remove(&oldest);
            cache.stats.sampler_evictions += 1;
        }
        cache.entries.insert(state, sampler.clone());
        cache.order.push_back(state);
        cache.stats.sampler_builds += 1;
        cache.stats.sampler_entries = cache.entries.len();
        Ok(sampler)
    }
    /// D3D9 NULL float pixel samplers read (0,0,0,1). These private, initialized
    /// views implement that API state only; a bound invalid/uninitialized
    /// texture is never replaced. Views retain the declared image dimension.
    pub(crate) fn null_texture_view(
        &self,
        dimension: naga::ImageDimension,
    ) -> Result<wgpu::TextureView> {
        let textures = self
            .null_textures
            .get_or_init(|| {
                let color = [0, 0, 0, 255];
                let mut flat = self.create_sampled_texture(1, 1, 1, SampleFormat::Bgra8)?;
                self.upload_texture(&mut flat, 0, &color)?;
                let mut cube = self.create_cube_texture(1, 1, SampleFormat::Bgra8)?;
                for face in 0..6 {
                    self.upload_cube_face(&mut cube, face, 0, &color)?;
                }
                let mut volume = self.create_volume_texture(1, 1, 1, 1)?;
                self.upload_volume(&mut volume, 0, &color)?;
                Ok([flat, cube, volume])
            })
            .as_ref()
            .map_err(Clone::clone)?;
        let index = match dimension {
            naga::ImageDimension::D2 => 0,
            naga::ImageDimension::Cube => 1,
            naga::ImageDimension::D3 => 2,
            _ => {
                return Err(
                    "NULL texture dimension is outside the D3D9 pixel sampler profile".into(),
                );
            }
        };
        textures[index].view(false)
    }
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
            layers: 1,
            depth: 1,
            volume: false,
            initialized: vec![target.initialized.clone()],
            views: Default::default(),
        };
        Ok((sampled, target))
    }
    /// Single-sample, uncompressed BGRA8 3D storage; every mip must be uploaded.
    pub fn create_volume_texture(
        &self,
        width: u32,
        height: u32,
        depth: u32,
        levels: u32,
    ) -> Result<SampledTexture> {
        self.create_sampled_volume(width, height, depth, levels, SampleFormat::Bgra8)
    }
    /// Real BGRA8 or linear L8 volume storage; no compressed-volume fallback.
    pub fn create_sampled_volume(
        &self,
        width: u32,
        height: u32,
        depth: u32,
        levels: u32,
        format: SampleFormat,
    ) -> Result<SampledTexture> {
        if !matches!(format, SampleFormat::Bgra8 | SampleFormat::Luminance8) {
            return Err("unsupported volume format".into());
        }
        let limit = self.device.limits().max_texture_dimension_3d.min(512);
        extent(width, height, limit)?;
        if depth == 0 || depth > limit {
            return Err("volume depth outside bounded device extent".into());
        }
        let maximum = 32 - width.max(height).max(depth).leading_zeros();
        let levels = if levels == 0 { maximum } else { levels };
        if levels > maximum {
            return Err("too many volume mips".into());
        }
        let bytes = (0..levels).try_fold(0u64, |sum, level| {
            sum.checked_add(
                u64::from((width >> level).max(1))
                    * u64::from((height >> level).max(1))
                    * u64::from((depth >> level).max(1))
                    * 4,
            )
        });
        if bytes.is_none_or(|n| n > 64 * 1024 * 1024) {
            return Err("volume exceeds storage budget".into());
        }
        self.scope();
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("D3D9 actual BGRA8 volume storage"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: depth,
            },
            mip_level_count: levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
        });
        self.finish()?;
        Ok(SampledTexture {
            texture,
            format,
            width,
            height,
            depth,
            volume: true,
            layers: 1,
            levels,
            initialized: (0..levels)
                .map(|_| Arc::new(AtomicBool::new(false)))
                .collect(),
            views: Default::default(),
        })
    }
    /// Upload a complete mip in tightly packed z-slice/row order.
    pub fn upload_volume(
        &self,
        texture: &mut SampledTexture,
        level: u32,
        bytes: &[u8],
    ) -> Result<()> {
        let (width, height, depth) = texture.volume_dimensions(level)?;
        let (pitch, rows) = texture.format.block_layout(width, height)?;
        if bytes.len() as u64 != u64::from(pitch) * u64::from(height) * u64::from(depth) {
            return Err("volume upload byte extent differs".into());
        }
        let expanded = expand_linear(texture.format, bytes);
        let bytes = expanded.as_deref().unwrap_or(bytes);
        let pitch = if expanded.is_some() { width * 4 } else { pitch };
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
                depth_or_array_layers: depth,
            },
        );
        self.finish()?;
        texture.initialized[level as usize].store(true, Ordering::Release);
        Ok(())
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
        self.create_sampled_layers(width, height, levels, format, 1)
    }
    /// Six independent +X,-X,+Y,-Y,+Z,-Z face/mip allocations. No render-target
    /// or generated-mip behavior is implied; every sampled subresource must be uploaded.
    pub fn create_cube_texture(
        &self,
        edge: u32,
        levels: u32,
        format: SampleFormat,
    ) -> Result<SampledTexture> {
        self.create_sampled_layers(edge, edge, levels, format, 6)
    }
    fn create_sampled_layers(
        &self,
        width: u32,
        height: u32,
        levels: u32,
        format: SampleFormat,
        layers: u32,
    ) -> Result<SampledTexture> {
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
            label: Some("D3D9 actual sampled texture storage"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: layers,
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
            layers,
            depth: 1,
            volume: false,
            initialized: (0..levels * layers)
                .map(|_| Arc::new(AtomicBool::new(false)))
                .collect(),
            views: Default::default(),
        })
    }
    pub fn upload_texture(&self, texture: &mut Texture2D, level: u32, bytes: &[u8]) -> Result<()> {
        if texture.is_cube() || texture.volume {
            return Err("cube/volume upload requires its explicit method".into());
        }
        self.upload_sampled_layer(texture, 0, level, bytes)
    }
    pub fn upload_cube_face(
        &self,
        texture: &mut SampledTexture,
        face: u32,
        level: u32,
        bytes: &[u8],
    ) -> Result<()> {
        if !texture.is_cube() {
            return Err("cube face upload requires cube storage".into());
        }
        self.upload_sampled_layer(texture, face, level, bytes)
    }
    fn upload_sampled_layer(
        &self,
        texture: &mut SampledTexture,
        face: u32,
        level: u32,
        bytes: &[u8],
    ) -> Result<()> {
        let subresource = texture.subresource(level, face)?;
        let (width, height) = texture.dimensions(level)?;
        let (pitch, rows) = texture.format.block_layout(width, height)?;
        if bytes.len() != pitch as usize * rows as usize {
            return Err("texture upload byte extent differs".into());
        }
        // Keep guest encodings unchanged; only the GPU upload is expanded.
        let expanded = expand_linear(texture.format, bytes);
        let (bytes, pitch) = if let Some(expanded) = &expanded {
            (
                expanded.as_slice(),
                width
                    .checked_mul(4)
                    .ok_or("expanded texture pitch overflow")?,
            )
        } else {
            (bytes, pitch)
        };
        self.scope();
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture.texture,
                mip_level: level,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: face,
                },
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
        texture.initialized[subresource].store(true, Ordering::Release);
        Ok(())
    }
}
fn expand_linear(format: SampleFormat, bytes: &[u8]) -> Option<Vec<u8>> {
    match format {
        SampleFormat::Luminance8 => Some(expand_luminance(bytes)),
        // A8 is the documented exception: absent RGB components are zero.
        SampleFormat::Alpha8 => Some(bytes.iter().flat_map(|&alpha| [0, 0, 0, alpha]).collect()),
        SampleFormat::Argb4444 => Some(
            bytes
                .as_chunks::<2>()
                .0
                .iter()
                .flat_map(|&pair| {
                    let word = u16::from_le_bytes(pair);
                    [
                        (word & 15) as u8 * 17,
                        ((word >> 4) & 15) as u8 * 17,
                        ((word >> 8) & 15) as u8 * 17,
                        ((word >> 12) & 15) as u8 * 17,
                    ]
                })
                .collect(),
        ),
        _ => None,
    }
}
fn expand_luminance(bytes: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .flat_map(|&value| [value, value, value, 255])
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires an actual GPU"]
    fn immutable_resource_cache_is_bounded_and_never_bypasses_mip_validation() {
        let device = Device::new().unwrap();
        assert_eq!(
            device.legacy_resource_cache_stats(),
            LegacyResourceStats::default()
        );
        for n in 0..66 {
            let mut state = SamplerState::default();
            state.set(1, 1 + n % 3).unwrap();
            state.set(2, 1 + (n / 3) % 3).unwrap();
            state.set(3, 1 + (n / 9) % 3).unwrap();
            state.set(9, n / 27).unwrap();
            device.legacy_sampler(state).unwrap();
        }
        let stats = device.legacy_resource_cache_stats();
        assert_eq!(
            (
                stats.sampler_builds,
                stats.sampler_entries,
                stats.sampler_evictions
            ),
            (66, 64, 2)
        );
        device.legacy_sampler(SamplerState::default()).unwrap();
        device.legacy_sampler(SamplerState::default()).unwrap();
        let stats = device.legacy_resource_cache_stats();
        assert_eq!(
            (
                stats.sampler_builds,
                stats.sampler_hits,
                stats.sampler_entries,
                stats.sampler_evictions
            ),
            (67, 1, 64, 3)
        );
        let mut texture = device.create_texture(2, 2, 2).unwrap();
        assert!(texture.view(false).is_err());
        assert!(texture.views[0].get().is_none());
        device.upload_texture(&mut texture, 0, &[128; 16]).unwrap();
        assert!(texture.view(false).is_err());
        device.upload_texture(&mut texture, 1, &[64; 4]).unwrap();
        texture.view(false).unwrap();
        texture.view(true).unwrap();
        assert!(texture.views.iter().all(|view| view.get().is_some()));
        texture.initialized[1].store(false, Ordering::Release);
        assert!(texture.view(false).is_err());
        assert!(texture.view(true).is_err());
    }

    #[test]
    fn alpha_and_argb4444_keep_raw_pitches_and_every_channel_value() {
        assert_eq!(SampleFormat::Alpha8.block_layout(3, 2).unwrap(), (3, 2));
        assert_eq!(SampleFormat::Argb4444.block_layout(3, 2).unwrap(), (6, 2));
        let alpha: Vec<_> = (0..=255).collect();
        let expanded = expand_linear(SampleFormat::Alpha8, &alpha).unwrap();
        for (n, pixel) in expanded.as_chunks::<4>().0.iter().enumerate() {
            assert_eq!(*pixel, [0, 0, 0, n as u8]);
        }
        let words: Vec<_> = (0..=u16::MAX).flat_map(u16::to_le_bytes).collect();
        let expanded = expand_linear(SampleFormat::Argb4444, &words).unwrap();
        for (n, pixel) in expanded.as_chunks::<4>().0.iter().enumerate() {
            assert_eq!(
                *pixel,
                [
                    (n & 15) as u8 * 17,
                    ((n >> 4) & 15) as u8 * 17,
                    ((n >> 8) & 15) as u8 * 17,
                    ((n >> 12) & 15) as u8 * 17
                ]
            );
        }
        assert!(SampleFormat::Argb4444.block_layout(u32::MAX, 1).is_err());
        assert!(SampleFormat::Alpha8.block_layout(u32::MAX, 2).is_err());
    }
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
    fn luminance_mips_keep_byte_pitch_and_expand_all_channels_without_quantization() {
        assert_eq!(SampleFormat::Luminance8.block_layout(3, 2).unwrap(), (3, 2));
        assert_eq!(SampleFormat::Luminance8.block_layout(1, 1).unwrap(), (1, 1));
        assert!(!SampleFormat::Luminance8.compressed());
        assert!(SampleFormat::Luminance8.block_layout(0, 1).is_err());
        assert!(SampleFormat::Luminance8.block_layout(u32::MAX, 2).is_err());
        let original: Vec<_> = (0..=255u8).collect();
        let expanded = expand_luminance(&original);
        for (value, pixel) in original.iter().zip(expanded.as_chunks::<4>().0) {
            assert_eq!(pixel, &[*value, *value, *value, 255]);
        }
        assert_eq!(original, (0..=255u8).collect::<Vec<_>>());
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
