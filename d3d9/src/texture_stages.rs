//! D3D9 fixed-function texture-stage state. Programmable pixel shaders bypass
//! these combiners; unsupported fixed-function combinations fail before drawing.
use crate::Result;
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub struct TextureStages {
    values: BTreeMap<(u32, u32), u32>,
}
impl Default for TextureStages {
    fn default() -> Self {
        let mut values = BTreeMap::new();
        for stage in 0..8 {
            for (ty, value) in [
                (1, if stage == 0 { 4 } else { 1 }),
                (2, 2),
                (3, 1),
                (4, if stage == 0 { 2 } else { 1 }),
                (5, 2),
                (6, 1),
                (7, 0),
                (8, 0),
                (9, 0),
                (10, 0),
                (11, stage),
                (22, 0),
                (23, 0),
                (24, 0),
                (26, 1),
                (27, 1),
                (28, 1),
                (32, 0),
            ] {
                values.insert((stage, ty), value);
            }
        }
        Self { values }
    }
}
impl TextureStages {
    pub fn get(&self, stage: u32, ty: u32) -> Result<u32> {
        self.values
            .get(&(stage, ty))
            .copied()
            .ok_or("unimplemented texture-stage index/state".into())
    }
    pub fn set(&mut self, stage: u32, ty: u32, value: u32) -> Result<()> {
        self.get(stage, ty)?;
        let supported = match ty {
            1 | 4 => (1..=26).contains(&value),
            2 | 3 | 5 | 6 | 26 | 27 => value & !0x3f == 0 && value & 15 <= 6,
            7..=11 | 22 | 23 | 32 => true,
            24 => value & !0x1ff == 0 && value & 255 <= 4,
            28 => [1, 5].contains(&value),
            _ => false,
        };
        if !supported {
            return Err("texture-stage value outside documented encoding".into());
        }
        self.values.insert((stage, ty), value);
        Ok(())
    }
    pub fn validate_transformed_color(&self, texture_zero_bound: bool) -> Result<()> {
        if (1..8).any(|stage| self.values[&(stage, 1)] != 1) {
            return Err("fixed-function multistage combiner is unimplemented".into());
        }
        let op = self.values[&(0, 1)];
        if op == 1
            || (!texture_zero_bound
                && op == 4
                && self.values[&(0, 2)] == 2
                && self.values[&(0, 3)] == 1)
        {
            Ok(())
        } else {
            Err("fixed-function draw requires an unimplemented texture combiner".into())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_color_defaults_and_disabled_combiners_are_explicit() {
        let mut stages = TextureStages::default();
        assert_eq!(stages.get(0, 1).unwrap(), 4);
        assert_eq!(stages.get(1, 1).unwrap(), 1);
        stages.validate_transformed_color(false).unwrap();
        assert!(stages.validate_transformed_color(true).is_err());
        stages.set(0, 1, 1).unwrap();
        stages.validate_transformed_color(true).unwrap();
        stages.set(1, 1, 4).unwrap();
        assert!(stages.validate_transformed_color(false).is_err());
        assert!(stages.set(8, 1, 1).is_err());
        assert!(stages.set(0, 1, 27).is_err());
        assert_eq!(stages.get(0, 1).unwrap(), 1);
    }
}
