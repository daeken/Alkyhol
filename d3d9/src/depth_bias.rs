//! Float D3D9 depth bias applied after clipping, preserving original shader bodies.
use crate::Result;
use naga::{Binding, BuiltIn, Expression as E, Span, Statement as S, TypeInner};
fn position(binding: &Option<Binding>) -> bool {
    matches!(binding, Some(Binding::BuiltIn(BuiltIn::Position { .. })))
}
/// Adapt the fragment entry while retaining all original color outputs. Explicit
/// shader depth output is rejected until its interaction has native evidence.
pub(crate) fn lower(module: &mut naga::Module, constant: f32, slope: f32) -> Result<()> {
    if !constant.is_finite() || !slope.is_finite() {
        return Err("depth bias must be finite".into());
    }
    if constant == 0. && slope == 0. {
        return Ok(());
    }
    let fragments: Vec<_> = module
        .entry_points
        .iter()
        .enumerate()
        .filter_map(|(i, e)| (e.stage == naga::ShaderStage::Fragment).then_some(i))
        .collect();
    if fragments.len() != 1 {
        return Err("depth bias requires one fragment entry".into());
    }
    let mut entry = module.entry_points.remove(fragments[0]);
    if entry.early_depth_test.is_some() {
        return Err("depth bias with explicit early depth tests is not implemented".into());
    }
    let mut original = std::mem::take(&mut entry.function);
    let result = original
        .result
        .clone()
        .ok_or("depth bias requires fragment color output")?;
    let (mut members, span) = match &module.types[result.ty].inner {
        TypeInner::Struct { members, span } => (members.clone(), *span),
        TypeInner::Vector {
            size: naga::VectorSize::Quad,
            scalar: naga::Scalar::F32,
        } => (
            vec![naga::StructMember {
                name: Some("original_color".into()),
                ty: result.ty,
                binding: result.binding.clone(),
                offset: 0,
            }],
            16,
        ),
        _ => return Err("fragment output outside depth bias profile".into()),
    };
    if members
        .iter()
        .any(|m| matches!(m.binding, Some(Binding::BuiltIn(BuiltIn::FragDepth))))
    {
        return Err("depth bias with explicit shader depth output is not implemented".into());
    }
    let float = module.types.insert(
        naga::Type {
            name: None,
            inner: TypeInner::Scalar(naga::Scalar::F32),
        },
        Span::UNDEFINED,
    );
    let float4 = module.types.insert(
        naga::Type {
            name: None,
            inner: TypeInner::Vector {
                size: naga::VectorSize::Quad,
                scalar: naga::Scalar::F32,
            },
        },
        Span::UNDEFINED,
    );
    members.push(naga::StructMember {
        name: Some("biased_depth".into()),
        ty: float,
        binding: Some(Binding::BuiltIn(BuiltIn::FragDepth)),
        offset: span,
    });
    let output = module.types.insert(
        naga::Type {
            name: Some("AlkyholBiasedFragment".into()),
            inner: TypeInner::Struct {
                members: members.clone(),
                span: (span + 4).div_ceil(16) * 16,
            },
        },
        Span::UNDEFINED,
    );
    let mut arguments = original.arguments.clone();
    let count = arguments.len();
    let mut position_input = None;
    for (n, arg) in arguments.iter().enumerate() {
        if position(&arg.binding) {
            position_input = Some((n, None));
            break;
        }
        if let TypeInner::Struct { members, .. } = &module.types[arg.ty].inner
            && let Some(i) = members.iter().position(|m| position(&m.binding))
        {
            position_input = Some((n, Some(i as u32)));
            break;
        }
    }
    let (position_arg, position_member) = position_input.unwrap_or_else(|| {
        arguments.push(naga::FunctionArgument {
            name: Some("alkyhol_fragment_position".into()),
            ty: float4,
            binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
        });
        (count, None)
    });
    for arg in &mut original.arguments {
        arg.binding = None;
    }
    original.result.as_mut().unwrap().binding = None;
    original.name = Some("alkyhol_original_fragment".into());
    let function = module.functions.append(original, Span::UNDEFINED);
    let mut wrapper = naga::Function {
        name: Some("alkyhol_biased_fragment".into()),
        arguments,
        result: Some(naga::FunctionResult {
            ty: output,
            binding: None,
        }),
        ..Default::default()
    };
    let inputs: Vec<_> = (0..wrapper.arguments.len())
        .map(|n| {
            wrapper
                .expressions
                .append(E::FunctionArgument(n as u32), Span::UNDEFINED)
        })
        .collect();
    let bias = wrapper
        .expressions
        .append(E::Literal(naga::Literal::F32(constant)), Span::UNDEFINED);
    let factor = wrapper
        .expressions
        .append(E::Literal(naga::Literal::F32(slope)), Span::UNDEFINED);
    let zero = wrapper
        .expressions
        .append(E::Literal(naga::Literal::F32(0.)), Span::UNDEFINED);
    let one = wrapper
        .expressions
        .append(E::Literal(naga::Literal::F32(1.)), Span::UNDEFINED);
    let start = wrapper.expressions.len();
    let mut add = |e| wrapper.expressions.append(e, Span::UNDEFINED);
    let p = if let Some(index) = position_member {
        add(E::AccessIndex {
            base: inputs[position_arg],
            index,
        })
    } else {
        inputs[position_arg]
    };
    let z = add(E::AccessIndex { base: p, index: 2 });
    let offset = if slope != 0. {
        let dx = add(E::Derivative {
            axis: naga::DerivativeAxis::X,
            ctrl: naga::DerivativeControl::None,
            expr: z,
        });
        let dy = add(E::Derivative {
            axis: naga::DerivativeAxis::Y,
            ctrl: naga::DerivativeControl::None,
            expr: z,
        });
        let dx = add(E::Math {
            fun: naga::MathFunction::Abs,
            arg: dx,
            arg1: None,
            arg2: None,
            arg3: None,
        });
        let dy = add(E::Math {
            fun: naga::MathFunction::Abs,
            arg: dy,
            arg1: None,
            arg2: None,
            arg3: None,
        });
        let m = add(E::Math {
            fun: naga::MathFunction::Max,
            arg: dx,
            arg1: Some(dy),
            arg2: None,
            arg3: None,
        });
        let scaled = add(E::Binary {
            op: naga::BinaryOperator::Multiply,
            left: m,
            right: factor,
        });
        add(E::Binary {
            op: naga::BinaryOperator::Add,
            left: scaled,
            right: bias,
        })
    } else {
        bias
    };
    let biased = add(E::Binary {
        op: naga::BinaryOperator::Add,
        left: z,
        right: offset,
    });
    let biased = add(E::Math {
        fun: naga::MathFunction::Clamp,
        arg: biased,
        arg1: Some(zero),
        arg2: Some(one),
        arg3: None,
    });
    wrapper.body.push(
        S::Emit(wrapper.expressions.range_from(start)),
        Span::UNDEFINED,
    );
    let call = wrapper
        .expressions
        .append(E::CallResult(function), Span::UNDEFINED);
    wrapper.body.push(
        S::Call {
            function,
            arguments: inputs[..count].to_vec(),
            result: Some(call),
        },
        Span::UNDEFINED,
    );
    let start = wrapper.expressions.len();
    let components = if matches!(module.types[result.ty].inner, TypeInner::Struct { .. }) {
        (0..members.len() - 1)
            .map(|n| {
                wrapper.expressions.append(
                    E::AccessIndex {
                        base: call,
                        index: n as u32,
                    },
                    Span::UNDEFINED,
                )
            })
            .chain(std::iter::once(biased))
            .collect()
    } else {
        vec![call, biased]
    };
    let value = wrapper.expressions.append(
        E::Compose {
            ty: output,
            components,
        },
        Span::UNDEFINED,
    );
    wrapper.body.push(
        S::Emit(wrapper.expressions.range_from(start)),
        Span::UNDEFINED,
    );
    wrapper
        .body
        .push(S::Return { value: Some(value) }, Span::UNDEFINED);
    entry.function = wrapper;
    module.entry_points.push(entry);
    crate::legacy::wgsl(module)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scalar_color_struct_color_and_existing_position_preserve_interfaces() {
        for source in [
            "@fragment fn main()->@location(0) vec4<f32>{return vec4<f32>(1);}",
            "struct I{@builtin(position) p:vec4<f32>,@location(0) c:vec4<f32>};struct O{@location(0)c:vec4<f32>};@fragment fn main(i:I)->O{return O(i.c);}",
        ] {
            for (bias, slope) in [(0.125, 0.), (-0.125, 2.)] {
                let mut module = naga::front::wgsl::parse_str(source).unwrap();
                lower(&mut module, bias, slope).unwrap();
                let text = crate::legacy::wgsl(&module).unwrap();
                assert!(text.contains("@builtin(frag_depth)"));
                let parsed = naga::front::wgsl::parse_str(&text).unwrap();
                crate::legacy::wgsl(&parsed).unwrap();
            }
        }
    }
    #[test]
    fn invalid_values_and_shader_depth_outputs_fail_explicitly() {
        let source = "struct O{@location(0)c:vec4<f32>,@builtin(frag_depth)d:f32};@fragment fn main()->O{return O(vec4<f32>(1),0.5);}";
        for (bias, slope) in [(f32::NAN, 0.), (0., f32::INFINITY), (0.1, 0.)] {
            let mut module = naga::front::wgsl::parse_str(source).unwrap();
            assert!(lower(&mut module, bias, slope).is_err());
        }
        let mut module = naga::front::wgsl::parse_str(source).unwrap();
        let original = crate::legacy::wgsl(&module).unwrap();
        lower(&mut module, -0., 0.).unwrap();
        assert_eq!(crate::legacy::wgsl(&module).unwrap(), original);
    }
}
