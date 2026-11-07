//! Compute root-buffer bindings and immutable dispatch snapshots.
use super::*;
use alkyhol::root_signature::{Binding, Layout};
use alkyhol_shader::RegisterClass;
#[derive(Default)]
pub(super) struct State {
    pipeline: Option<Arc<compute_pipelines::Data>>,
    root: Option<(Layout, Arc<()>)>,
    arguments: Vec<Option<u64>>,
}
pub(super) unsafe fn initial(
    device: *mut Device,
    kind: i32,
    pipeline: usize,
) -> Result<State, i32> {
    if pipeline == 0 {
        return Ok(State::default());
    }
    if kind == 3 {
        return Err(E_NOTIMPL);
    }
    Ok(State {
        pipeline: Some(unsafe { compute_pipelines::snapshot(device, pipeline) }?),
        ..State::default()
    })
}
#[derive(Clone)]
pub(super) struct Buffer {
    pub(super) memory: Arc<cbv_descriptors::BufferMemory>,
    pub(super) offset: u64,
    pub(super) metal_index: u32,
    pub(super) required: u32,
}
impl Buffer {
    pub(super) fn usage(&self) -> Option<&Arc<Mutex<BufferUsage>>> {
        (self.memory.heap == 1).then_some(&self.memory.usage)
    }
}
#[derive(Clone)]
pub(super) struct Dispatch {
    pub(super) pipeline: Arc<compute_pipelines::Data>,
    pub(super) buffers: Vec<Buffer>,
    pub(super) groups: [u32; 3],
}
fn live(recording: &mut Recording) -> Result<(), i32> {
    if let Some(error) = recording.error {
        return Err(error);
    }
    if !recording.open {
        return Err(E_FAIL);
    }
    Ok(())
}
unsafe fn change(this: *mut List, update: impl FnOnce(&mut State) -> Result<(), i32>) {
    if unsafe { (*this).kind } == 3 {
        unsupported_void("compute binding on COPY list");
    }
    let mut r = unsafe { (*this).recording.lock().unwrap() };
    let result = live(&mut r).and_then(|_| update(&mut r.compute));
    if r.error.is_none() {
        r.error = result.err();
    }
}
pub(super) unsafe extern "C" fn set_pipeline(this: *mut List, object: usize) {
    unsafe {
        change(this, |state| {
            state.pipeline = Some(compute_pipelines::snapshot((*this).parent.0, object)?);
            Ok(())
        });
    }
}
pub(super) unsafe extern "C" fn set_root(this: *mut List, object: usize) {
    unsafe {
        change(this, |state| {
            let snapshot = root_signatures::binding_snapshot((*this).parent.0, object)?;
            if state
                .root
                .as_ref()
                .is_some_and(|(_, old)| Arc::ptr_eq(old, &snapshot.1))
            {
                return Ok(());
            }
            state.arguments = vec![None; snapshot.0.parameters.len()];
            state.root = Some(snapshot);
            Ok(())
        });
    }
}
unsafe fn buffer(this: *mut List, index: u32, address: u64, kind: u32) {
    unsafe {
        change(this, |state| {
            let (root, _) = state.root.as_ref().ok_or(E_INVALIDARG)?;
            let parameter = root.parameters.get(index as usize).ok_or(E_INVALIDARG)?;
            if parameter.visibility != 0
                || !matches!(&parameter.binding,Binding::Descriptor {kind:k,..} if *k==kind)
            {
                return Err(E_INVALIDARG);
            }
            if !address.is_multiple_of(if kind == 2 { 256 } else { 4 }) {
                return Err(E_INVALIDARG);
            }
            state.arguments[index as usize] = Some(address);
            Ok(())
        });
    }
}
pub(super) unsafe extern "C" fn cbv(this: *mut List, index: u32, address: u64) {
    unsafe { buffer(this, index, address, 2) }
}
pub(super) unsafe extern "C" fn srv(this: *mut List, index: u32, address: u64) {
    unsafe { buffer(this, index, address, 3) }
}
pub(super) unsafe extern "C" fn uav(this: *mut List, index: u32, address: u64) {
    unsafe { buffer(this, index, address, 4) }
}
fn snapshot(
    state: &State,
    registry: &cbv_descriptors::Registry,
    groups: [u32; 3],
) -> Result<Option<Dispatch>, i32> {
    if groups.iter().any(|&n| n > 65535) {
        return Err(E_INVALIDARG);
    }
    let pipeline = state.pipeline.as_ref().ok_or(E_INVALIDARG)?;
    let (root, _) = state.root.as_ref().ok_or(E_INVALIDARG)?;
    if &pipeline._layout != root {
        return Err(E_INVALIDARG);
    }
    let mut buffers = Vec::new();
    for resource in &pipeline._translation.resources {
        let (kind, required) = match resource.class {
            RegisterClass::Cbv => (2, 1),
            RegisterClass::Srv => (3, 0x40),
            RegisterClass::Uav => (4, 8),
            _ => return Err(E_NOTIMPL),
        };
        let index=root.parameters.iter().position(|p|p.visibility==0&&matches!(&p.binding,Binding::Descriptor {kind:k,register,space} if *k==kind&&*register==resource.register&&*space==resource.space)).ok_or(E_NOTIMPL)?;
        let address = state
            .arguments
            .get(index)
            .copied()
            .flatten()
            .ok_or(E_INVALIDARG)?;
        let (memory, offset) = registry.resolve(address)?;
        if ![1, 2].contains(&memory.heap)
            || (kind == 4 && (memory.heap != 1 || memory.flags & 4 == 0))
        {
            return Err(E_INVALIDARG);
        }
        buffers.push(Buffer {
            memory,
            offset,
            metal_index: resource.metal_index,
            required,
        });
    }
    if groups.contains(&0) {
        return Ok(None);
    }
    Ok(Some(Dispatch {
        pipeline: Arc::clone(pipeline),
        buffers,
        groups,
    }))
}
pub(super) unsafe extern "C" fn dispatch(this: *mut List, x: u32, y: u32, z: u32) {
    if unsafe { (*this).kind } == 3 {
        unsupported_void("Dispatch on COPY list");
    }
    let mut r = unsafe { (*this).recording.lock().unwrap() };
    let result = live(&mut r).and_then(|_| {
        let registry = unsafe { (*(*this).parent.0).descriptors.lock().unwrap() };
        let command = snapshot(&r.compute, &registry, [x, y, z])?;
        if let Some(command) = command {
            r.commands.try_reserve(1).map_err(|_| E_OUTOFMEMORY)?;
            r.commands.push(Command::Dispatch(command));
        }
        Ok(())
    });
    if r.error.is_none() {
        r.error = result.err();
    }
}

#[cfg(test)]
#[path = "compute_dispatch_tests.rs"]
mod tests;
