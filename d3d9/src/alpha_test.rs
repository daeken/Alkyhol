//! D3D9 post-shader alpha comparison, before color/depth/stencil writes.
use crate::Result;
use naga::{Binding, Expression as E, Span, Statement as S, TypeInner};
/// Wrap the original fragment function without changing its body or outputs.
pub(crate) fn lower(
    module: &mut naga::Module,
    enabled: bool,
    function: u32,
    reference: u32,
) -> Result<()> {
    if !(1..=8).contains(&function) || reference > 255 {
        return Err("invalid alpha comparison or reference".into());
    }
    if !enabled || function == 8 {
        return Ok(());
    }
    let fragments: Vec<_> = module
        .entry_points
        .iter()
        .enumerate()
        .filter_map(|(i, e)| (e.stage == naga::ShaderStage::Fragment).then_some(i))
        .collect();
    if fragments.len() != 1 {
        return Err("alpha test requires one fragment entry".into());
    }
    let mut entry = module.entry_points.remove(fragments[0]);
    if entry.early_depth_test.is_some() {
        return Err("alpha test with explicit early depth tests is unsupported".into());
    }
    let mut original = std::mem::take(&mut entry.function);
    let result = original
        .result
        .clone()
        .ok_or("alpha test requires COLOR0")?;
    let color0 = |b: &Option<Binding>| matches!(b, Some(Binding::Location { location: 0, .. }));
    let member = match &module.types[result.ty].inner {
        TypeInner::Struct { members, .. } => Some(
            members
                .iter()
                .position(|m| color0(&m.binding))
                .ok_or("alpha test requires COLOR0")? as u32,
        ),
        _ if color0(&result.binding) => None,
        _ => return Err("alpha test requires COLOR0".into()),
    };
    let ty = if let Some(index) = member {
        let TypeInner::Struct { members, .. } = &module.types[result.ty].inner else {
            unreachable!()
        };
        members[index as usize].ty
    } else {
        result.ty
    };
    if !matches!(
        module.types[ty].inner,
        TypeInner::Vector {
            size: naga::VectorSize::Quad,
            scalar: naga::Scalar::F32
        }
    ) {
        return Err("alpha test requires float4 COLOR0".into());
    }
    let arguments = original.arguments.clone();
    for a in &mut original.arguments {
        a.binding = None;
    }
    original.result.as_mut().unwrap().binding = None;
    original.name = Some("alkyhol_alpha_original_fragment".into());
    let function_handle = module.functions.append(original, Span::UNDEFINED);
    let mut wrapper = naga::Function {
        name: Some("alkyhol_alpha_fragment".into()),
        arguments,
        result: Some(result),
        ..Default::default()
    };
    let arguments = (0..wrapper.arguments.len())
        .map(|i| {
            wrapper
                .expressions
                .append(E::FunctionArgument(i as u32), Span::UNDEFINED)
        })
        .collect();
    let reference = wrapper.expressions.append(
        E::Literal(naga::Literal::F32(reference as f32 / 255.)),
        Span::UNDEFINED,
    );
    let call = wrapper
        .expressions
        .append(E::CallResult(function_handle), Span::UNDEFINED);
    wrapper.body.push(
        S::Call {
            function: function_handle,
            arguments,
            result: Some(call),
        },
        Span::UNDEFINED,
    );
    if function == 1 {
        // MSL discard_fragment() is not declared noreturn. Keep a structured
        // branch and a typed return, even though no fragment can survive it.
        let never = wrapper
            .expressions
            .append(E::Literal(naga::Literal::Bool(false)), Span::UNDEFINED);
        let mut reject = naga::Block::new();
        reject.push(S::Kill, Span::UNDEFINED);
        wrapper.body.push(
            S::If {
                condition: never,
                accept: naga::Block::new(),
                reject,
            },
            Span::UNDEFINED,
        );
        wrapper
            .body
            .push(S::Return { value: Some(call) }, Span::UNDEFINED);
    } else {
        let start = wrapper.expressions.len();
        let color = if let Some(index) = member {
            wrapper
                .expressions
                .append(E::AccessIndex { base: call, index }, Span::UNDEFINED)
        } else {
            call
        };
        let alpha = wrapper.expressions.append(
            E::AccessIndex {
                base: color,
                index: 3,
            },
            Span::UNDEFINED,
        );
        let op = match function {
            2 => naga::BinaryOperator::Less,
            3 => naga::BinaryOperator::Equal,
            4 => naga::BinaryOperator::LessEqual,
            5 => naga::BinaryOperator::Greater,
            6 => naga::BinaryOperator::NotEqual,
            7 => naga::BinaryOperator::GreaterEqual,
            _ => unreachable!(),
        };
        let passes = wrapper.expressions.append(
            E::Binary {
                op,
                left: alpha,
                right: reference,
            },
            Span::UNDEFINED,
        );
        wrapper.body.push(
            S::Emit(wrapper.expressions.range_from(start)),
            Span::UNDEFINED,
        );
        let mut reject = naga::Block::new();
        reject.push(S::Kill, Span::UNDEFINED);
        wrapper.body.push(
            S::If {
                condition: passes,
                accept: naga::Block::new(),
                reject,
            },
            Span::UNDEFINED,
        );
        wrapper
            .body
            .push(S::Return { value: Some(call) }, Span::UNDEFINED);
    }
    entry.function = wrapper;
    module.entry_points.push(entry);
    crate::legacy::wgsl(module)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scalar_and_struct_color_outputs_keep_all_interfaces() {
        for source in [
            "@fragment fn main()->@location(0) vec4<f32>{return vec4<f32>(1);}",
            "struct O{@location(1) other:vec4<f32>,@location(0) color:vec4<f32>,@builtin(frag_depth) depth:f32};@fragment fn main()->O{return O(vec4<f32>(0),vec4<f32>(1),0.5);}",
        ] {
            for function in 1..=8 {
                let mut module = naga::front::wgsl::parse_str(source).unwrap();
                lower(&mut module, true, function, 128).unwrap();
                let text = crate::legacy::wgsl(&module).unwrap();
                naga::front::wgsl::parse_str(&text).unwrap();
            }
        }
    }
    #[test]
    fn disabled_and_always_preserve_identity_and_invalid_contracts_fail() {
        let source = "@fragment fn main()->@location(0) vec4<f32>{return vec4<f32>(1);}";
        for (enabled, function) in [(false, 1), (true, 8)] {
            let mut module = naga::front::wgsl::parse_str(source).unwrap();
            let before = crate::legacy::wgsl(&module).unwrap();
            lower(&mut module, enabled, function, 255).unwrap();
            assert_eq!(before, crate::legacy::wgsl(&module).unwrap());
        }
        for (function, reference) in [(0, 0), (9, 0), (7, 256)] {
            let mut module = naga::front::wgsl::parse_str(source).unwrap();
            assert!(lower(&mut module, true, function, reference).is_err());
        }
        let mut module = naga::front::wgsl::parse_str(
            "@fragment fn main()->@location(1) vec4<f32>{return vec4<f32>(1);}",
        )
        .unwrap();
        assert!(lower(&mut module, true, 7, 128).is_err());
    }
}
