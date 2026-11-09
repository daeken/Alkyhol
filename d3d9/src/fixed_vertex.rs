//! Fixed-function POSITIONT processing when D3D9 supplies no vertex shader.
//! The original pixel shader remains unchanged; no missing varying is invented.
use crate::{
    Result,
    compiler::{Compiled, Input},
    legacy,
};
use std::collections::BTreeSet;
/// Generate the pretransformed vertex stage for the real following pixel inputs.
/// Private vertex constants carry [viewport width,height,x,y]. Pixel constants
/// and the application's vertex-shader binding/constant state remain separate.
pub fn vertex_shader(pixel_inputs: &[Input]) -> Result<Compiled> {
    if pixel_inputs.len() > 15 {
        return Err("fixed vertex varying budget exceeded".into());
    }
    let mut locations = BTreeSet::new();
    let mut semantics = BTreeSet::new();
    let mut inputs = vec![Input {
        semantic: "POSITIONT".into(),
        index: 0,
        register: 0,
    }];
    let mut source = String::from(
        "@group(0) @binding(0) var<uniform> viewport: vec4<f32>;\nstruct Input { @location(0) position: vec4<f32>,\n",
    );
    for (n, input) in pixel_inputs.iter().enumerate() {
        if input.register >= 16
            || !locations.insert(input.register)
            || !semantics.insert((input.semantic.as_str(), input.index))
            || !matches!(
                (input.semantic.as_str(), input.index),
                ("TEXCOORD", 0..=7) | ("COLOR", 0..=1)
            )
        {
            return Err("fixed vertex varying outside unique COLOR/TEXCOORD profile".into());
        }
        inputs.push(Input {
            semantic: input.semantic.clone(),
            index: input.index,
            register: n as u32 + 1,
        });
        source.push_str(&format!("@location({}) value{n}: vec4<f32>,\n", n + 1));
    }
    source.push_str("}\nstruct Output { @builtin(position) position: vec4<f32>,\n");
    for (n, input) in pixel_inputs.iter().enumerate() {
        source.push_str(&format!(
            "@location({}) value{n}: vec4<f32>,\n",
            input.register
        ));
    }
    source.push_str("}\n@vertex fn main(i: Input) -> Output {\nvar o: Output;\nlet w = 1.0 / i.position.w;\nlet xy = vec2<f32>(2.0*(i.position.x-viewport.z)/viewport.x-1.0, 1.0-2.0*(i.position.y-viewport.w)/viewport.y);\no.position=vec4<f32>(xy*w, i.position.z*w, w);\n");
    for n in 0..pixel_inputs.len() {
        source.push_str(&format!("o.value{n}=i.value{n};\n"));
    }
    source.push_str("return o;\n}\n");
    let module = naga::front::wgsl::parse_str(&source).map_err(|e| e.emit_to_string(&source))?;
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
            shader_stage: naga::ShaderStage::Vertex,
            entry_point: "main".into(),
        }),
    )
    .map_err(|e| e.to_string())?;
    let spirv: Vec<_> = words.into_iter().flat_map(u32::to_le_bytes).collect();
    // Validate the emitted module through the exact same runtime path.
    legacy::wgsl(&legacy::lower_spirv(&spirv, legacy::Stage::Vertex)?)?;
    Ok(Compiled { spirv, inputs })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_stage_links_real_semantics_and_sparse_pixel_locations() {
        let shader = vertex_shader(&[
            Input {
                semantic: "TEXCOORD".into(),
                index: 0,
                register: 2,
            },
            Input {
                semantic: "COLOR".into(),
                index: 1,
                register: 9,
            },
        ])
        .unwrap();
        assert_eq!(
            (
                shader.inputs[0].semantic.as_str(),
                shader.inputs[0].register
            ),
            ("POSITIONT", 0)
        );
        assert_eq!(
            (
                shader.inputs[1].semantic.as_str(),
                shader.inputs[1].index,
                shader.inputs[1].register
            ),
            ("TEXCOORD", 0, 1)
        );
        assert_eq!(
            (
                shader.inputs[2].semantic.as_str(),
                shader.inputs[2].index,
                shader.inputs[2].register
            ),
            ("COLOR", 1, 2)
        );
        let mut module = legacy::lower_spirv(&shader.spirv, legacy::Stage::Vertex).unwrap();
        legacy::vertex_pixel_centers(&mut module).unwrap();
        legacy::wgsl(&module).unwrap();
        vertex_shader(&[]).unwrap();
    }
    #[test]
    fn unsupported_or_conflicting_varyings_never_become_successful_zero_inputs() {
        for inputs in [
            vec![Input {
                semantic: "NORMAL".into(),
                index: 0,
                register: 0,
            }],
            vec![Input {
                semantic: "TEXCOORD".into(),
                index: 8,
                register: 0,
            }],
            vec![Input {
                semantic: "COLOR".into(),
                index: 0,
                register: 16,
            }],
            vec![
                Input {
                    semantic: "COLOR".into(),
                    index: 0,
                    register: 0,
                },
                Input {
                    semantic: "TEXCOORD".into(),
                    index: 0,
                    register: 0,
                },
            ],
            vec![
                Input {
                    semantic: "COLOR".into(),
                    index: 0,
                    register: 0,
                },
                Input {
                    semantic: "COLOR".into(),
                    index: 0,
                    register: 1,
                },
            ],
        ] {
            assert!(vertex_shader(&inputs).is_err());
        }
    }
}
