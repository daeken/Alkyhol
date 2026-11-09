//! Fixed-function world/view/projection for color-disabled XYZ visibility draws.
//! Lighting/material/texture color is not modeled; callers must disable color
//! writes and alpha test, and reject vertex blending before using these stages.
use crate::{
    Result,
    compiler::{Compiled, Input},
    legacy,
};
use std::sync::OnceLock;
fn compile(source: &str, stage: legacy::Stage, inputs: Vec<Input>) -> Result<Compiled> {
    let module = naga::front::wgsl::parse_str(source).map_err(|e| e.emit_to_string(source))?;
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .map_err(|e| e.to_string())?;
    let mut options = naga::back::spv::Options {
        flags: naga::back::spv::WriterFlags::LABEL_VARYINGS,
        fake_missing_bindings: false,
        ..Default::default()
    };
    options.binding_map.insert(
        naga::ResourceBinding {
            group: 0,
            binding: 0,
        },
        naga::back::spv::BindingInfo {
            descriptor_set: 0,
            binding: 0,
            binding_array_size: None,
        },
    );
    let words = naga::back::spv::write_vec(
        &module,
        &info,
        &options,
        Some(&naga::back::spv::PipelineOptions {
            shader_stage: match stage {
                legacy::Stage::Vertex => naga::ShaderStage::Vertex,
                legacy::Stage::Pixel => naga::ShaderStage::Fragment,
            },
            entry_point: "main".into(),
        }),
    )
    .map_err(|e| e.to_string())?;
    let spirv = words
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>();
    legacy::wgsl(&legacy::lower_spirv(&spirv, stage)?)?;
    Ok(Compiled { spirv, inputs })
}
/// Generated fixed-function stages, not a replacement for an application shader.
/// Constants contain WORLD, VIEW, PROJECTION in original D3DMATRIX row order.
/// WGSL reads those rows as columns, so matrix*column implements Windows' row
/// vector multiplication, with separate stages preserving transform order.
pub fn shaders() -> Result<&'static (Compiled, Compiled)> {
    static SHADERS: OnceLock<Result<(Compiled, Compiled)>> = OnceLock::new();
    SHADERS.get_or_init(|| {
        let vertex=compile("@group(0) @binding(0) var<uniform> transform:array<mat4x4<f32>,3>; @vertex fn main(@location(0) p:vec4<f32>)->@builtin(position) vec4<f32>{return transform[2]*(transform[1]*(transform[0]*p));}",legacy::Stage::Vertex,vec![Input {semantic:"POSITION".into(),index:0,register:0}])?;
        let pixel=compile("@fragment fn main()->@location(0) vec4<f32>{return vec4<f32>(0);}",legacy::Stage::Pixel,vec![])?;
        Ok((vertex,pixel))
    }).as_ref().map_err(Clone::clone)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_pair_has_only_real_position_and_transform_inputs() {
        let (vertex, pixel) = shaders().unwrap();
        assert!(std::ptr::eq(shaders().unwrap(), shaders().unwrap()));
        assert_eq!(vertex.inputs.len(), 1);
        assert_eq!(
            (
                vertex.inputs[0].semantic.as_str(),
                vertex.inputs[0].index,
                vertex.inputs[0].register
            ),
            ("POSITION", 0, 0)
        );
        assert!(pixel.inputs.is_empty());
        let mut vs = legacy::lower_spirv(&vertex.spirv, legacy::Stage::Vertex).unwrap();
        assert_eq!(
            vs.global_variables
                .iter()
                .filter(|(_, v)| v.binding.is_some())
                .count(),
            1
        );
        legacy::vertex_pixel_centers(&mut vs).unwrap();
        legacy::wgsl(&vs).unwrap();
        let ps = legacy::lower_spirv(&pixel.spirv, legacy::Stage::Pixel).unwrap();
        assert!(ps.global_variables.iter().all(|(_, v)| v.binding.is_none()));
        assert_eq!(
            vs.global_variables
                .iter()
                .filter(|(_, v)| v.binding.is_some())
                .count(),
            2 // The pixel-center adapter adds its separate viewport uniform.
        );
    }
}
