//! Checked BC1 mip-chain intake for the pinned MiniEngine DDS assets. No re-encoding.
use metal::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Linear,
    Srgb,
}
pub struct Mip<'a> {
    pub width: u32,
    pub height: u32,
    pub row_bytes: u64,
    pub data: &'a [u8],
}
pub struct Dds<'a> {
    pub width: u32,
    pub height: u32,
    pub encoding: Encoding,
    pub mips: Vec<Mip<'a>>,
}
fn word(bytes: &[u8], offset: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or("truncated DDS field")?
            .try_into()
            .unwrap(),
    ))
}
pub fn parse(bytes: &[u8]) -> Result<Dds<'_>, String> {
    if bytes.get(..4) != Some(b"DDS ")
        || word(bytes, 4)? != 124
        || word(bytes, 76)? != 32
        || word(bytes, 80)? & 4 == 0
    {
        return Err("invalid DDS header".into());
    }
    let width = word(bytes, 16)?;
    let height = word(bytes, 12)?;
    let levels = word(bytes, 28)?.max(1);
    if width == 0
        || height == 0
        || width > 16384
        || height > 16384
        || levels > 32 - width.max(height).leading_zeros()
    {
        return Err("invalid DDS dimensions/mip count".into());
    }
    let (mut offset, encoding) = match bytes.get(84..88) {
        Some(b"DXT1") => (128, Encoding::Linear),
        Some(b"DX10") => {
            if word(bytes, 132)? != 3 || word(bytes, 136)? != 0 || word(bytes, 140)? != 1 {
                return Err("only single 2D DDS textures are supported".into());
            }
            (
                148,
                match word(bytes, 128)? {
                    71 => Encoding::Linear,
                    72 => Encoding::Srgb,
                    _ => return Err("only BC1 DDS encoding is supported".into()),
                },
            )
        }
        _ => return Err("unsupported DDS FourCC".into()),
    };
    let mut mips = Vec::new();
    let (mut w, mut h) = (width, height);
    for _ in 0..levels {
        let row_bytes = (w as u64).div_ceil(4) * 8;
        let length = (row_bytes * (h as u64).div_ceil(4)) as usize;
        let data = bytes
            .get(offset..offset + length)
            .ok_or("truncated DDS mip data")?;
        mips.push(Mip {
            width: w,
            height: h,
            row_bytes,
            data,
        });
        offset += length;
        w = (w / 2).max(1);
        h = (h / 2).max(1);
    }
    if offset != bytes.len() {
        return Err("DDS payload length differs from mip chain".into());
    }
    Ok(Dds {
        width,
        height,
        encoding,
        mips,
    })
}
pub fn upload(device: &DeviceRef, bytes: &[u8]) -> Result<Texture, String> {
    if !device.supports_BC_texture_compression() {
        return Err("Metal BC texture compression is required".into());
    }
    let dds = parse(bytes)?;
    let descriptor = TextureDescriptor::new();
    descriptor.set_texture_type(MTLTextureType::D2);
    descriptor.set_width(dds.width as u64);
    descriptor.set_height(dds.height as u64);
    descriptor.set_pixel_format(match dds.encoding {
        Encoding::Linear => MTLPixelFormat::BC1_RGBA,
        Encoding::Srgb => MTLPixelFormat::BC1_RGBA_sRGB,
    });
    descriptor.set_mipmap_level_count(dds.mips.len() as u64);
    descriptor.set_storage_mode(MTLStorageMode::Shared);
    descriptor.set_usage(MTLTextureUsage::ShaderRead);
    let texture = device.new_texture(&descriptor);
    for (level, mip) in dds.mips.iter().enumerate() {
        texture.replace_region(
            MTLRegion::new_2d(0, 0, mip.width as u64, mip.height as u64),
            level as u64,
            mip.data.as_ptr().cast(),
            mip.row_bytes,
        );
    }
    Ok(texture)
}
pub fn solid(device: &DeviceRef, pixel: [u8; 4]) -> Texture {
    let descriptor = TextureDescriptor::new();
    descriptor.set_texture_type(MTLTextureType::D2);
    descriptor.set_width(1);
    descriptor.set_height(1);
    descriptor.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
    descriptor.set_storage_mode(MTLStorageMode::Shared);
    descriptor.set_usage(MTLTextureUsage::ShaderRead);
    let texture = device.new_texture(&descriptor);
    texture.replace_region(MTLRegion::new_2d(0, 0, 1, 1), 0, pixel.as_ptr().cast(), 4);
    texture
}
