//! Pinned ModelH3D serialization: 64-byte Header, 336-byte Mesh, 992-byte Material.
//! Read fields explicitly; serialized Windows SIMD alignment is not host Rust layout.
use bytemuck::{Pod, Zeroable};
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}
#[derive(Debug)]
pub struct Mesh {
    pub material: usize,
    pub vertices: Vec<Vertex>,
}
#[derive(Debug)]
pub struct Material {
    pub diffuse: [f32; 4],
    pub texture_paths: [String; 6],
    pub name: String,
}
#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub bounds: [[f32; 3]; 2],
}
fn u32_at(bytes: &[u8], offset: usize) -> Result<usize, String> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or("truncated H3D integer")?
            .try_into()
            .unwrap(),
    ) as usize)
}
fn float_at(bytes: &[u8], offset: usize) -> Result<f32, String> {
    let v = f32::from_bits(u32_at(bytes, offset)? as u32);
    if !v.is_finite() {
        return Err("nonfinite H3D float".into());
    }
    Ok(v)
}
fn triple(bytes: &[u8], offset: usize) -> Result<[f32; 3], String> {
    Ok([
        float_at(bytes, offset)?,
        float_at(bytes, offset + 4)?,
        float_at(bytes, offset + 8)?,
    ])
}
// The pinned H3D contains nonfinite tangent frames in several meshes. Preserve
// their raw values; Bary Rays never reads them. A lit port must handle this explicitly.
fn raw_triple(bytes: &[u8], offset: usize) -> Result<[f32; 3], String> {
    Ok([
        f32::from_bits(u32_at(bytes, offset)? as u32),
        f32::from_bits(u32_at(bytes, offset + 4)? as u32),
        f32::from_bits(u32_at(bytes, offset + 8)? as u32),
    ])
}
fn string_at(bytes: &[u8], offset: usize) -> Result<String, String> {
    let b = bytes
        .get(offset..offset + 128)
        .ok_or("truncated H3D string")?;
    let end = b
        .iter()
        .position(|&b| b == 0)
        .ok_or("unterminated H3D string")?;
    String::from_utf8(b[..end].to_vec()).map_err(|_| "invalid H3D string".into())
}
pub fn parse(bytes: &[u8]) -> Result<Model, String> {
    if bytes.len() < 64 {
        return Err("truncated H3D header".into());
    }
    let mesh_count = u32_at(bytes, 0)?;
    let material_count = u32_at(bytes, 4)?;
    let vertex_bytes = u32_at(bytes, 8)?;
    let index_bytes = u32_at(bytes, 12)?;
    let depth_bytes = u32_at(bytes, 16)?;
    if mesh_count == 0 || material_count == 0 {
        return Err("empty H3D scene".into());
    }
    let material_base = 64usize
        .checked_add(mesh_count.checked_mul(336).ok_or("H3D overflow")?)
        .ok_or("H3D overflow")?;
    let vertex_base = material_base
        .checked_add(material_count.checked_mul(992).ok_or("H3D overflow")?)
        .ok_or("H3D overflow")?;
    let index_base = vertex_base
        .checked_add(vertex_bytes)
        .ok_or("H3D overflow")?;
    let end = index_base
        .checked_add(index_bytes)
        .and_then(|v| v.checked_add(depth_bytes))
        .and_then(|v| v.checked_add(index_bytes))
        .ok_or("H3D overflow")?;
    if end != bytes.len() {
        return Err("H3D declared sections do not match file size".into());
    }
    let bounds = [triple(bytes, 32)?, triple(bytes, 48)?];
    if (0..3).any(|i| bounds[0][i] > bounds[1][i]) {
        return Err("inverted H3D bounds".into());
    }
    let mut materials = Vec::with_capacity(material_count);
    for id in 0..material_count {
        let base = material_base + id * 992;
        let colour = triple(bytes, base)?;
        let paths = (0..6)
            .map(|i| string_at(bytes, base + 92 + i * 128))
            .collect::<Result<Vec<_>, _>>()?;
        materials.push(Material {
            diffuse: [colour[0], colour[1], colour[2], float_at(bytes, base + 12)?],
            texture_paths: paths.try_into().unwrap(),
            name: string_at(bytes, base + 860)?,
        });
    }
    let mut meshes = Vec::with_capacity(mesh_count);
    for id in 0..mesh_count {
        let base = 64 + id * 336;
        let material = u32_at(bytes, base + 32)?;
        let stride = u32_at(bytes, base + 44)?;
        if material >= material_count || u32_at(bytes, base + 36)? != 31 || stride != 56 {
            return Err("unsupported H3D material or vertex layout".into());
        }
        for (i, (offset, components)) in [(0, 3), (12, 2), (20, 3), (32, 3), (44, 3)]
            .into_iter()
            .enumerate()
        {
            let a = base + 52 + i * 8;
            let packed = u32_at(bytes, a)?;
            let format = u32_at(bytes, a + 4)?;
            if packed != offset || format != (components | 5 << 16) {
                return Err("unsupported H3D attribute format".into());
            }
        }
        let offset = u32_at(bytes, base + 308)?;
        let count = u32_at(bytes, base + 312)?;
        let index_offset = u32_at(bytes, base + 316)?;
        let index_count = u32_at(bytes, base + 320)?;
        if count == 0
            || index_count == 0
            || index_count % 3 != 0
            || offset
                .checked_add(count * stride)
                .is_none_or(|v| v > vertex_bytes)
            || index_offset
                .checked_add(index_count * 2)
                .is_none_or(|v| v > index_bytes)
        {
            return Err("H3D mesh outside declared buffers".into());
        }
        let mut vertices = Vec::with_capacity(index_count);
        for n in 0..index_count {
            let i = index_base + index_offset + n * 2;
            let index = u16::from_le_bytes(bytes[i..i + 2].try_into().unwrap()) as usize;
            if index >= count {
                return Err("H3D vertex index out of bounds".into());
            }
            let v = vertex_base + offset + index * stride;
            vertices.push(Vertex {
                position: triple(bytes, v)?,
                uv: [float_at(bytes, v + 12)?, float_at(bytes, v + 16)?],
                normal: triple(bytes, v + 20)?,
                tangent: raw_triple(bytes, v + 32)?,
                bitangent: raw_triple(bytes, v + 44)?,
            });
        }
        meshes.push(Mesh { material, vertices });
    }
    Ok(Model {
        meshes,
        materials,
        bounds,
    })
}
