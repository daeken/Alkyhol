//! Legacy shader lowering adapters. Triangle-only point-size specialization is
//! explicit: point primitives remain unsupported. Shader math is preserved.
use crate::Result;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Vertex,
    Pixel,
}
fn instructions(words: &[u32]) -> Result<Vec<&[u32]>> {
    if words.len() < 5 || words[0] != 0x07230203 || words[4] != 0 {
        return Err("invalid SPIR-V header".into());
    }
    let mut at = 5;
    let mut result = Vec::new();
    while at < words.len() {
        let n = (words[at] >> 16) as usize;
        if n == 0 || at.checked_add(n).is_none_or(|end| end > words.len()) {
            return Err("truncated SPIR-V instruction".into());
        }
        result.push(&words[at..at + n]);
        at += n;
    }
    Ok(result)
}
/// Vulkan requires a point-size output even when drawing triangles. WGSL has no
/// corresponding builtin. Preserve its calculations in private storage, remove
/// only its entry-point interface decoration, and reject point draws elsewhere.
fn triangle_point_size(words: &[u32], stage: Stage) -> Result<Vec<u32>> {
    let code = instructions(words)?;
    let points: BTreeSet<_> = code
        .iter()
        .filter(|i| i.len() == 4 && i[0] & 65535 == 71 && i[2] == 11 && i[3] == 1)
        .map(|i| i[1])
        .collect();
    if points.is_empty() {
        return Ok(words.to_vec());
    }
    if stage != Stage::Vertex {
        return Err("point-size output outside a triangle vertex shader".into());
    }
    let mut bound = words[3];
    let mut pointers = BTreeMap::new();
    for i in &code {
        if i.len() >= 4 && i[0] & 65535 == 59 && points.contains(&i[2]) {
            if i[3] != 3 {
                return Err("point-size variable is not an output".into());
            }
            if let std::collections::btree_map::Entry::Vacant(e) = pointers.entry(i[1]) {
                e.insert(bound);
                bound = bound.checked_add(1).ok_or("SPIR-V id overflow")?;
            }
        }
    }
    if pointers.is_empty() {
        return Err("point-size decoration has no variable".into());
    }
    let mut out = words[..5].to_vec();
    out[3] = bound;
    let mut variables = BTreeSet::new();
    let mut declared = BTreeSet::new();
    for i in code {
        let op = i[0] & 65535;
        if op == 71 && i.len() == 4 && points.contains(&i[1]) && i[2] == 11 && i[3] == 1 {
            continue;
        }
        if op == 15 {
            let Some(last) = (3..i.len()).find(|&n| i[n].to_le_bytes().contains(&0)) else {
                return Err("SPIR-V entry name is unterminated".into());
            };
            let mut entry = i[..last + 1].to_vec();
            entry.extend(i[last + 1..].iter().filter(|id| !points.contains(id)));
            entry[0] = ((entry.len() as u32) << 16) | op;
            out.extend(entry);
            continue;
        }
        if op == 59 && i.len() >= 4 && points.contains(&i[2]) {
            let mut variable = i.to_vec();
            variable[1] = *pointers.get(&i[1]).ok_or("point-size pointer missing")?;
            variable[3] = 6;
            out.extend(variable);
            variables.insert(i[2]);
            continue;
        }
        out.extend_from_slice(i);
        if op == 32
            && i.len() == 4
            && let Some(&id) = pointers.get(&i[1])
        {
            out.extend([4 << 16 | 32, id, 6, i[3]]);
            declared.insert(i[1]);
        }
    }
    if variables != points || declared.len() != pointers.len() {
        return Err("unsupported point-size pointer topology".into());
    }
    Ok(out)
}
pub fn lower_spirv(bytes: &[u8], stage: Stage) -> Result<naga::Module> {
    if !bytes.len().is_multiple_of(4) || bytes.len() > 16 * 1024 * 1024 {
        return Err("SPIR-V size outside the bounded profile".into());
    }
    let words: Vec<_> = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| u32::from_le_bytes(*b))
        .collect();
    let words = triangle_point_size(&words, stage)?;
    let module = naga::front::spv::Frontend::new(
        words.into_iter(),
        &naga::front::spv::Options {
            adjust_coordinate_space: false,
            ..Default::default()
        },
    )
    .parse()
    .map_err(|e| format!("legacy SPIR-V lowering: {e}"))?;
    let expected = match stage {
        Stage::Vertex => naga::ShaderStage::Vertex,
        Stage::Pixel => naga::ShaderStage::Fragment,
    };
    if module.entry_points.len() != 1 || module.entry_points[0].stage != expected {
        return Err("legacy shader stage does not match its entry point".into());
    }
    Ok(module)
}
pub fn wgsl(module: &naga::Module) -> Result<String> {
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(module)
    .map_err(|e| e.to_string())?;
    naga::back::wgsl::write_string(module, &info, naga::back::wgsl::WriterFlags::EXPLICIT_TYPES)
        .map_err(|e| e.to_string())
}
/// Add D3D9's half-pixel convention after the original vertex shader returns.
/// Its instructions remain in an ordinary helper function. The wrapper shifts
/// clip x/y by +/-w/viewport while preserving z, w and every other varying.
pub fn vertex_pixel_centers(module: &mut naga::Module) -> Result<()> {
    use naga::{Binding, BuiltIn, Expression as E, Span, Statement as S, TypeInner};
    if module.entry_points.len() != 1 || module.entry_points[0].stage != naga::ShaderStage::Vertex {
        return Err("expected one legacy vertex entry".into());
    }
    let mut entry = module.entry_points.remove(0);
    let mut original = std::mem::take(&mut entry.function);
    let result = original
        .result
        .clone()
        .ok_or("vertex shader has no output")?;
    let position =
        |b: &Option<Binding>| matches!(b, Some(Binding::BuiltIn(BuiltIn::Position { .. })));
    let (member, position_type) = if position(&result.binding) {
        (None, result.ty)
    } else {
        let TypeInner::Struct { members, .. } = &module.types[result.ty].inner else {
            return Err("vertex output has no position structure".into());
        };
        let index = members
            .iter()
            .position(|m| position(&m.binding))
            .ok_or("vertex position output absent")?;
        (Some(index as u32), members[index].ty)
    };
    if !matches!(
        module.types[position_type].inner,
        TypeInner::Vector {
            size: naga::VectorSize::Quad,
            scalar: naga::Scalar {
                kind: naga::ScalarKind::Float,
                width: 4
            }
        }
    ) {
        return Err("vertex position must be float4".into());
    }
    let arguments = original.arguments.clone();
    for argument in &mut original.arguments {
        argument.binding = None;
    }
    original.result.as_mut().unwrap().binding = None;
    original.name = Some("alkyhol_original_vertex".into());
    let function = module.functions.append(original, Span::UNDEFINED);
    let viewport = module.global_variables.append(
        naga::GlobalVariable {
            name: Some("alkyhol_viewport".into()),
            space: naga::AddressSpace::Uniform,
            binding: Some(naga::ResourceBinding {
                group: 2,
                binding: 0,
            }),
            ty: position_type,
            init: None,
        },
        Span::UNDEFINED,
    );
    let mut wrapper = naga::Function {
        name: Some("alkyhol_vertex_entry".into()),
        arguments,
        result: Some(result.clone()),
        ..Default::default()
    };
    let args = (0..wrapper.arguments.len())
        .map(|n| {
            wrapper
                .expressions
                .append(E::FunctionArgument(n as u32), Span::UNDEFINED)
        })
        .collect();
    let call = wrapper
        .expressions
        .append(E::CallResult(function), Span::UNDEFINED);
    wrapper.body.push(
        S::Call {
            function,
            arguments: args,
            result: Some(call),
        },
        Span::UNDEFINED,
    );
    let global = wrapper
        .expressions
        .append(E::GlobalVariable(viewport), Span::UNDEFINED);
    let start = wrapper.expressions.len();
    let expressions = &mut wrapper.expressions;
    let mut append = |e| expressions.append(e, Span::UNDEFINED);
    let vp = append(E::Load { pointer: global });
    let width = append(E::AccessIndex { base: vp, index: 0 });
    let height = append(E::AccessIndex { base: vp, index: 1 });
    let p = if let Some(index) = member {
        append(E::AccessIndex { base: call, index })
    } else {
        call
    };
    let x = append(E::AccessIndex { base: p, index: 0 });
    let y = append(E::AccessIndex { base: p, index: 1 });
    let z = append(E::AccessIndex { base: p, index: 2 });
    let w = append(E::AccessIndex { base: p, index: 3 });
    let dx = append(E::Binary {
        op: naga::BinaryOperator::Divide,
        left: w,
        right: width,
    });
    let dy = append(E::Binary {
        op: naga::BinaryOperator::Divide,
        left: w,
        right: height,
    });
    let x = append(E::Binary {
        op: naga::BinaryOperator::Add,
        left: x,
        right: dx,
    });
    let y = append(E::Binary {
        op: naga::BinaryOperator::Subtract,
        left: y,
        right: dy,
    });
    let fixed = append(E::Compose {
        ty: position_type,
        components: vec![x, y, z, w],
    });
    let value = if let Some(position_member) = member {
        let TypeInner::Struct { members, .. } = &module.types[result.ty].inner else {
            unreachable!()
        };
        let mut components = Vec::new();
        for index in 0..members.len() {
            components.push(if index as u32 == position_member {
                fixed
            } else {
                append(E::AccessIndex {
                    base: call,
                    index: index as u32,
                })
            });
        }
        append(E::Compose {
            ty: result.ty,
            components,
        })
    } else {
        fixed
    };
    wrapper.body.push(
        S::Emit(wrapper.expressions.range_from(start)),
        Span::UNDEFINED,
    );
    wrapper
        .body
        .push(S::Return { value: Some(value) }, Span::UNDEFINED);
    entry.function = wrapper;
    module.entry_points.push(entry);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_instruction_lengths_fail_before_rewriting() {
        for tail in [vec![0], vec![3 << 16 | 32, 1]] {
            let mut w = vec![0x07230203, 0x10000, 0, 10, 0];
            w.extend(tail);
            assert!(triangle_point_size(&w, Stage::Vertex).is_err());
        }
        assert!(lower_spirv(&[0; 3], Stage::Vertex).is_err());
    }
}
