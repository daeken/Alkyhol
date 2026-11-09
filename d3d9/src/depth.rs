//! Real D16 and depth/stencil attachments with independently initialized aspects.
use crate::{Device, RenderState, Result, Target, extent};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
#[derive(Clone)]
pub struct Depth {
    pub(crate) texture: wgpu::Texture,
    width: u32,
    height: u32,
    stencil: bool,
    initialized: Arc<AtomicBool>,
    stencil_initialized: Arc<AtomicBool>,
}
impl Depth {
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn has_stencil(&self) -> bool {
        self.stencil
    }
    pub(crate) fn attachment(&self) -> wgpu::TextureView {
        self.texture.create_view(&Default::default())
    }
    pub(crate) fn depth_ops(&self, state: &RenderState) -> Option<wgpu::Operations<f32>> {
        (state.get(7).unwrap() != 0).then_some(wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
        })
    }
    pub(crate) fn stencil_ops(&self, state: &RenderState) -> Option<wgpu::Operations<u32>> {
        (self.stencil && state.get(52).unwrap() != 0).then_some(wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
        })
    }
}
fn compare(value: u32) -> wgpu::CompareFunction {
    match value {
        1 => wgpu::CompareFunction::Never,
        2 => wgpu::CompareFunction::Less,
        3 => wgpu::CompareFunction::Equal,
        4 => wgpu::CompareFunction::LessEqual,
        5 => wgpu::CompareFunction::Greater,
        6 => wgpu::CompareFunction::NotEqual,
        7 => wgpu::CompareFunction::GreaterEqual,
        8 => wgpu::CompareFunction::Always,
        _ => unreachable!(),
    }
}
fn operation(value: u32) -> wgpu::StencilOperation {
    match value {
        1 => wgpu::StencilOperation::Keep,
        2 => wgpu::StencilOperation::Zero,
        3 => wgpu::StencilOperation::Replace,
        4 => wgpu::StencilOperation::IncrementClamp,
        5 => wgpu::StencilOperation::DecrementClamp,
        6 => wgpu::StencilOperation::Invert,
        7 => wgpu::StencilOperation::IncrementWrap,
        8 => wgpu::StencilOperation::DecrementWrap,
        _ => unreachable!(),
    }
}
impl RenderState {
    pub(crate) fn depth_state(
        &self,
        depth: Option<&Depth>,
    ) -> Result<Option<wgpu::DepthStencilState>> {
        let z = self.get(7)? != 0;
        let stencil = self.get(52)? != 0;
        if !z && !stencil {
            return Ok(None);
        }
        let depth = depth.ok_or("depth/stencil testing requires a bound depth surface")?;
        if z && !depth.initialized.load(Ordering::Acquire) {
            return Err("depth contents have not been initialized".into());
        }
        if stencil && !depth.stencil {
            return Err("stencil testing requires a surface with a stencil aspect".into());
        }
        if stencil && !depth.stencil_initialized.load(Ordering::Acquire) {
            let states = [7, 52, 53, 54, 55, 56, 57, 58, 59, 185, 186, 187, 188, 189]
                .map(|ty| (ty, self.get(ty).unwrap()));
            return Err(format!("stencil testing requires an initialized stencil aspect; dimensions={:?}; render_states={states:?}", depth.dimensions()).into());
        }
        let stencil = if stencil {
            let face = |fail, zfail, pass, func| -> Result<_> {
                Ok(wgpu::StencilFaceState {
                    compare: compare(self.get(func)?),
                    fail_op: operation(self.get(fail)?),
                    depth_fail_op: operation(self.get(zfail)?),
                    pass_op: operation(self.get(pass)?),
                })
            };
            let clockwise = face(53, 54, 55, 56)?;
            let ccw = if self.get(185)? != 0 {
                face(186, 187, 188, 189)?
            } else {
                clockwise
            };
            let (front, back) = if self.get(22)? == 2 {
                (ccw, clockwise)
            } else {
                (clockwise, ccw)
            };
            wgpu::StencilState {
                front,
                back,
                read_mask: self.get(58)? & 255,
                write_mask: self.get(59)? & 255,
            }
        } else {
            Default::default()
        };
        Ok(Some(wgpu::DepthStencilState {
            format: if depth.stencil {
                wgpu::TextureFormat::Depth24PlusStencil8
            } else {
                wgpu::TextureFormat::Depth16Unorm
            },
            depth_write_enabled: z && self.get(14)? != 0,
            depth_compare: if z {
                compare(self.get(23)?)
            } else {
                wgpu::CompareFunction::Always
            },
            stencil,
            bias: Default::default(),
        }))
    }
}
pub(crate) fn validate_depth(target: &Target, depth: Option<&Depth>) -> Result<()> {
    if depth.is_some_and(|d| d.dimensions() != target.dimensions()) {
        return Err(
            "depth/color attachment extents differ; larger-depth subregions are not implemented"
                .into(),
        );
    }
    Ok(())
}
impl Device {
    pub fn create_depth16(&self, width: u32, height: u32) -> Result<Depth> {
        self.create_depth(width, height, false)
    }
    /// D24S8 facade over wgpu Depth24PlusStencil8. Native depth precision at
    /// quantization boundaries remains separate from finite stencil evidence.
    pub fn create_depth_stencil(&self, width: u32, height: u32) -> Result<Depth> {
        self.create_depth(width, height, true)
    }
    fn create_depth(&self, width: u32, height: u32, stencil: bool) -> Result<Depth> {
        extent(
            width,
            height,
            self.device.limits().max_texture_dimension_2d.min(8192),
        )?;
        self.scope();
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("D3D9 depth/stencil"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if stencil {
                wgpu::TextureFormat::Depth24PlusStencil8
            } else {
                wgpu::TextureFormat::Depth16Unorm
            },
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.finish()?;
        Ok(Depth {
            texture,
            width,
            height,
            stencil,
            initialized: Arc::new(AtomicBool::new(false)),
            stencil_initialized: Arc::new(AtomicBool::new(false)),
        })
    }
    pub fn clear_depth(&self, depth: &Depth, value: f32) -> Result<()> {
        self.clear_depth_stencil(depth, Some(value), None)
    }
    pub fn clear_depth_stencil(
        &self,
        depth: &Depth,
        value: Option<f32>,
        stencil: Option<u32>,
    ) -> Result<()> {
        if value.is_none() && stencil.is_none()
            || value.is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v))
            || stencil.is_some() && !depth.stencil
        {
            return Err("invalid depth/stencil clear aspect or value".into());
        }
        self.scope();
        let view = depth.attachment();
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("D3D9 depth/stencil aspect clear"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &view,
                    depth_ops: value.map(|v| wgpu::Operations {
                        load: wgpu::LoadOp::Clear(v),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: stencil.map(|v| wgpu::Operations {
                        load: wgpu::LoadOp::Clear(v & 255),
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                ..Default::default()
            });
        }
        self.queue.submit([encoder.finish()]);
        self.finish()?;
        if value.is_some() {
            depth.initialized.store(true, Ordering::Release);
        }
        if stencil.is_some() {
            depth.stencil_initialized.store(true, Ordering::Release);
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_stencil_ops_and_compare_functions_have_distinct_backed_mappings() {
        assert_eq!(operation(4), wgpu::StencilOperation::IncrementClamp);
        assert_eq!(operation(7), wgpu::StencilOperation::IncrementWrap);
        assert_eq!(operation(5), wgpu::StencilOperation::DecrementClamp);
        assert_eq!(operation(8), wgpu::StencilOperation::DecrementWrap);
        let operations: Vec<_> = (1..=8).map(operation).collect();
        for (i, op) in operations.iter().enumerate() {
            assert!(!operations[..i].contains(op));
        }
        let comparisons: Vec<_> = (1..=8).map(compare).collect();
        for (i, c) in comparisons.iter().enumerate() {
            assert!(!comparisons[..i].contains(c));
        }
    }
}
