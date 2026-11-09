//! Finite render state with real pipeline effects. Unsupported enabled effects
//! fail; state relevant only to unsupported primitive/vertex paths is retained.
use crate::Result;
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub struct RenderState {
    values: BTreeMap<u32, u32>,
}
impl Default for RenderState {
    fn default() -> Self {
        Self {
            values: [
                (7, 0),
                (8, 3),
                (9, 2),
                (14, 1),
                (15, 0),
                (16, 1),
                (19, 2),
                (20, 1),
                (22, 3),
                (23, 4),
                (24, 0),
                (25, 8),
                (26, 0),
                (27, 0),
                (28, 0),
                (29, 0),
                (34, 0),
                (35, 0),
                (36, 0),
                (37, 0x3f800000),
                (38, 0x3f800000),
                (48, 0),
                (52, 0),
                (53, 1),
                (54, 1),
                (55, 1),
                (56, 8),
                (57, 0),
                (58, u32::MAX),
                (59, u32::MAX),
                (60, u32::MAX),
                (136, 1),
                (137, 1),
                (139, 0),
                (140, 0),
                (141, 1),
                (142, 1),
                (143, 0),
                (145, 1),
                (146, 2),
                (147, 0),
                (148, 0),
                (151, 0),
                (152, 0),
                (154, 0x3f800000),
                (155, 0x3f800000),
                (156, 0),
                (157, 0),
                (158, 0x3f800000),
                (159, 0),
                (160, 0),
                (161, 1),
                (162, u32::MAX),
                (163, 0),
                (165, 0),
                (166, 0x42800000),
                (167, 0),
                (172, 3),
                (173, 1),
                (178, 0x3f800000),
                (179, 0x3f800000),
                (180, 0),
                (181, 0),
                (182, 0x3f800000),
                (183, 0),
                (184, 0),
                (185, 0),
                (186, 1),
                (187, 1),
                (188, 1),
                (189, 8),
                (190, 15),
                (191, 15),
                (192, 15),
                (168, 15),
                (170, 0),
                (171, 1),
                (174, 0),
                (175, 0),
                (176, 0),
                (193, u32::MAX),
                (194, 0),
                (195, 0),
                (206, 0),
                (207, 2),
                (208, 1),
                (209, 1),
            ]
            .into_iter()
            .chain((128..=135).chain(198..=205).map(|ty| (ty, 0)))
            .collect(),
        }
    }
}
fn factor(n: u32) -> Result<wgpu::BlendFactor> {
    Ok(match n {
        1 => wgpu::BlendFactor::Zero,
        2 => wgpu::BlendFactor::One,
        3 => wgpu::BlendFactor::Src,
        4 => wgpu::BlendFactor::OneMinusSrc,
        5 => wgpu::BlendFactor::SrcAlpha,
        6 => wgpu::BlendFactor::OneMinusSrcAlpha,
        7 => wgpu::BlendFactor::DstAlpha,
        8 => wgpu::BlendFactor::OneMinusDstAlpha,
        9 => wgpu::BlendFactor::Dst,
        10 => wgpu::BlendFactor::OneMinusDst,
        11 => wgpu::BlendFactor::SrcAlphaSaturated,
        14 => wgpu::BlendFactor::Constant,
        15 => wgpu::BlendFactor::OneMinusConstant,
        _ => return Err("unsupported D3D9 blend factor".into()),
    })
}
fn operation(n: u32) -> Result<wgpu::BlendOperation> {
    Ok(match n {
        1 => wgpu::BlendOperation::Add,
        2 => wgpu::BlendOperation::Subtract,
        3 => wgpu::BlendOperation::ReverseSubtract,
        4 => wgpu::BlendOperation::Min,
        5 => wgpu::BlendOperation::Max,
        _ => return Err("unknown D3D9 blend operation".into()),
    })
}
impl RenderState {
    pub fn get(&self, ty: u32) -> Result<u32> {
        self.values
            .get(&ty)
            .copied()
            .ok_or_else(|| format!("unimplemented render state {ty}"))
    }
    pub fn set(&mut self, ty: u32, value: u32) -> Result<()> {
        let old = self.get(ty)?;
        let supported = match ty {
            7 | 14 | 16 | 27 | 52 | 185 | 136 | 137 | 141 | 142 | 143 | 156 | 157 | 161 | 170
            | 194 | 206 => value <= 1,
            8 => value == 3,
            9 => value == 2,
            15
            | 26
            | 28
            | 29
            | 48
            | 128..=135
            | 151
            | 152
            | 167
            | 174
            | 175
            | 176
            | 184
            | 195
            | 198..=205 => value == 0,
            19 | 20 | 207 | 208 => factor(value).is_ok(),
            22 => (1..=3).contains(&value),
            23 | 25 | 56 => (1..=8).contains(&value),
            24 => value <= 255,
            168 | 190..=192 => value & !15 == 0,
            171 | 209 => operation(value).is_ok(),
            // D16 consumes depth test/write/compare. Stencil enable is rejected;
            // disabled stencil state is retained without pretending to apply it.
            53..=55 | 186..=189 => (1..=8).contains(&value),
            57..=60 => true,
            // Fog/lighting and point parameters are retained; those paths are
            // bypassed by programmable/TL triangle vertices or disabled above.
            34..=38
            | 139
            | 140
            | 145..=148
            | 154
            | 155
            | 158..=160
            | 162
            | 163
            | 165
            | 166
            | 172
            | 173
            | 178..=183
            | 193 => true,
            _ => false,
        };
        if !supported {
            return Err(format!(
                "render state {ty}={value:#x} outside implemented profile (old={old:#x})"
            ));
        }
        self.values.insert(ty, value);
        Ok(())
    }
    pub(crate) fn primitive(&self) -> wgpu::PrimitiveState {
        wgpu::PrimitiveState {
            front_face: if self.values[&22] == 2 {
                wgpu::FrontFace::Ccw
            } else {
                wgpu::FrontFace::Cw
            },
            cull_mode: if self.values[&22] == 1 {
                None
            } else {
                Some(wgpu::Face::Back)
            },
            ..Default::default()
        }
    }
    pub(crate) fn blend(&self) -> Option<wgpu::BlendState> {
        if self.values[&27] == 0 {
            return None;
        }
        let component = |source, destination, op| {
            let operation = operation(self.values[&op]).unwrap();
            let (src_factor, dst_factor) = if matches!(
                operation,
                wgpu::BlendOperation::Min | wgpu::BlendOperation::Max
            ) {
                (wgpu::BlendFactor::One, wgpu::BlendFactor::One)
            } else {
                (
                    factor(self.values[&source]).unwrap(),
                    factor(self.values[&destination]).unwrap(),
                )
            };
            wgpu::BlendComponent {
                src_factor,
                dst_factor,
                operation,
            }
        };
        let color = component(19, 20, 171);
        let alpha = if self.values[&206] != 0 {
            component(207, 208, 209)
        } else {
            color
        };
        Some(wgpu::BlendState { color, alpha })
    }
    pub(crate) fn multisample(&self) -> wgpu::MultisampleState {
        wgpu::MultisampleState {
            // D3D9 MULTISAMPLEMASK has no effect on a single-sample target.
            // Vulkan applies a one-sample mask whereas Metal ignores it; map
            // the D3D9 rule explicitly rather than inheriting backend behavior.
            mask: u64::MAX,
            ..Default::default()
        }
    }
    pub(crate) fn writes(&self) -> wgpu::ColorWrites {
        wgpu::ColorWrites::from_bits(self.values[&168]).unwrap()
    }
    pub(crate) fn format(&self) -> wgpu::TextureFormat {
        if self.values[&194] != 0 {
            wgpu::TextureFormat::Bgra8UnormSrgb
        } else {
            wgpu::TextureFormat::Bgra8Unorm
        }
    }
    pub(crate) fn constant(&self) -> wgpu::Color {
        let [r, g, b, a] = crate::argb_channels(self.values[&193]);
        wgpu::Color {
            r: r as f64 / 255.,
            g: g as f64 / 255.,
            b: b as f64 / 255.,
            a: a as f64 / 255.,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_declared_default_can_be_restored_without_enabling_an_unknown_path() {
        let mut state = RenderState::default();
        for (ty, value) in state.values.clone() {
            state
                .set(ty, value)
                .unwrap_or_else(|e| panic!("default state {ty}: {e}"));
            assert_eq!(state.get(ty).unwrap(), value);
        }
    }
    #[test]
    fn unsupported_enabled_effects_do_not_mutate_state() {
        let mut state = RenderState::default();
        for ty in [15, 26, 28, 152, 174, 176] {
            assert!(state.set(ty, 1).is_err());
            assert_eq!(state.get(ty).unwrap(), 0);
        }
        assert!(state.set(999, 0).is_err());
        assert!(state.set(19, 16).is_err());
        assert!(state.set(168, 16).is_err());
        state.set(22, 1).unwrap();
        assert_eq!(state.primitive().cull_mode, None);
        state.set(27, 1).unwrap();
        state.set(19, 5).unwrap();
        assert_eq!(
            state.blend().unwrap().color.src_factor,
            wgpu::BlendFactor::SrcAlpha
        );
        state.set(194, 1).unwrap();
        assert_eq!(state.format(), wgpu::TextureFormat::Bgra8UnormSrgb);
    }
}
