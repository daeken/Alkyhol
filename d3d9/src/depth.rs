//! Real non-lockable D16 depth attachments. Other depth/stencil formats fail.
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
    initialized: Arc<AtomicBool>,
}
impl Depth {
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub(crate) fn attachment(&self) -> wgpu::TextureView {
        self.texture.create_view(&Default::default())
    }
}
impl RenderState {
    pub(crate) fn depth_state(
        &self,
        depth: Option<&Depth>,
    ) -> Result<Option<wgpu::DepthStencilState>> {
        if self.get(7)? == 0 {
            return Ok(None);
        }
        let depth = depth.ok_or("depth testing requires a bound depth surface")?;
        if !depth.initialized.load(Ordering::Acquire) {
            return Err("depth contents have not been initialized".into());
        }
        let compare = match self.get(23)? {
            1 => wgpu::CompareFunction::Never,
            2 => wgpu::CompareFunction::Less,
            3 => wgpu::CompareFunction::Equal,
            4 => wgpu::CompareFunction::LessEqual,
            5 => wgpu::CompareFunction::Greater,
            6 => wgpu::CompareFunction::NotEqual,
            7 => wgpu::CompareFunction::GreaterEqual,
            8 => wgpu::CompareFunction::Always,
            _ => unreachable!(),
        };
        Ok(Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth16Unorm,
            depth_write_enabled: self.get(14)? != 0,
            depth_compare: compare,
            stencil: Default::default(),
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
        extent(
            width,
            height,
            self.device.limits().max_texture_dimension_2d.min(8192),
        )?;
        self.scope();
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("D3D9 real D16 depth"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth16Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.finish()?;
        Ok(Depth {
            texture,
            width,
            height,
            initialized: Arc::new(AtomicBool::new(false)),
        })
    }
    pub fn clear_depth(&self, depth: &Depth, value: f32) -> Result<()> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err("depth clear outside [0,1]".into());
        }
        self.scope();
        let view = depth.attachment();
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("D3D9 actual depth clear"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(value),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
        }
        self.queue.submit([encoder.finish()]);
        self.finish()?;
        depth.initialized.store(true, Ordering::Release);
        Ok(())
    }
}
