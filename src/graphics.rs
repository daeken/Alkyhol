//! Backend-independent, restricted D3D12-shaped command validation.
//! No COM ABI or D3D12 capability level is implied by this interface.
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceState {
    RenderTarget,
    CopySource,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Transition {
        resource: u64,
        before: ResourceState,
        after: ResourceState,
    },
    ClearRenderTargetView {
        resource: u64,
        color: [f64; 4],
    },
    DrawInstanced {
        resource: u64,
        pipeline: u64,
        vertices: u32,
        instances: u32,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphicsError {
    InvalidArgument,
    WrongDevice,
    WrongState,
    ListOpen,
    ListClosed,
    Unsupported,
    Backend(String),
}
impl std::fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for GraphicsError {}
pub type Result<T> = std::result::Result<T, GraphicsError>;

#[derive(Debug)]
pub struct GraphicsCommandList {
    device: u64,
    closed: bool,
    commands: Vec<Command>,
}
impl GraphicsCommandList {
    pub fn new(device: u64) -> Self {
        Self {
            device,
            closed: false,
            commands: Vec::new(),
        }
    }
    pub fn device(&self) -> u64 {
        self.device
    }
    pub fn record(&mut self, command: Command) -> Result<()> {
        if self.closed {
            return Err(GraphicsError::ListClosed);
        }
        match &command {
            Command::ClearRenderTargetView { color, .. }
                if color
                    .iter()
                    .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x)) =>
            {
                return Err(GraphicsError::InvalidArgument);
            }
            _ => (),
        }
        self.commands.push(command);
        Ok(())
    }
    pub fn close(&mut self) -> Result<()> {
        if self.closed {
            return Err(GraphicsError::ListClosed);
        }
        self.closed = true;
        Ok(())
    }
    pub fn reset(&mut self) -> Result<()> {
        if !self.closed {
            return Err(GraphicsError::ListOpen);
        }
        self.commands.clear();
        self.closed = false;
        Ok(())
    }
    pub fn commands(&self) -> Result<&[Command]> {
        if !self.closed {
            return Err(GraphicsError::ListOpen);
        }
        Ok(&self.commands)
    }
}

/// Validate the entire submission against a private state copy. Callers commit
/// returned states only after successful GPU encoding/submission.
pub fn validate_submission(
    device: u64,
    list: &GraphicsCommandList,
    resources: &HashMap<u64, ResourceState>,
    pipelines: &[u64],
) -> Result<HashMap<u64, ResourceState>> {
    if list.device() != device {
        return Err(GraphicsError::WrongDevice);
    }
    let mut states = resources.clone();
    for command in list.commands()? {
        match command {
            Command::Transition {
                resource,
                before,
                after,
            } => {
                let state = states
                    .get_mut(resource)
                    .ok_or(GraphicsError::InvalidArgument)?;
                if *state != *before {
                    return Err(GraphicsError::WrongState);
                }
                *state = *after;
            }
            Command::ClearRenderTargetView { resource, .. } => require_target(&states, *resource)?,
            Command::DrawInstanced {
                resource,
                pipeline,
                vertices,
                instances,
            } => {
                require_target(&states, *resource)?;
                if !pipelines.contains(pipeline) {
                    return Err(GraphicsError::InvalidArgument);
                }
                // Current shader interface uses no vertex buffers and exactly one triangle.
                if *vertices != 3 || *instances != 1 {
                    return Err(GraphicsError::Unsupported);
                }
            }
        }
    }
    Ok(states)
}
fn require_target(states: &HashMap<u64, ResourceState>, id: u64) -> Result<()> {
    match states.get(&id) {
        None => Err(GraphicsError::InvalidArgument),
        Some(ResourceState::RenderTarget) => Ok(()),
        _ => Err(GraphicsError::WrongState),
    }
}

/// RGBA8 copy footprints use D3D12/WebGPU's 256-byte row alignment.
pub fn copy_footprint(width: u32, height: u32) -> Result<(u32, u64)> {
    if width == 0 || height == 0 {
        return Err(GraphicsError::InvalidArgument);
    }
    let bytes = width.checked_mul(4).ok_or(GraphicsError::InvalidArgument)?;
    let pitch = bytes
        .checked_add(255)
        .ok_or(GraphicsError::InvalidArgument)?
        & !255;
    Ok((pitch, u64::from(pitch) * u64::from(height)))
}
