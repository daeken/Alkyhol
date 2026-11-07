//! Real DXIL -> open translator -> MSL -> Metal compute PSOs.
//! Creation does not enable command-list shader binding or Dispatch.
use super::*;
use alkyhol::root_signature::{Binding, Layout};
use alkyhol_shader::{MetalNamespace, RegisterClass, Translation, translate_compute_pipeline};
const PIPELINE: Guid = guid!(
    0x765a30f3, 0xf624, 0x4c6f, 0xa8, 0x28, 0xac, 0xe9, 0x48, 0x62, 0x24, 0x45
);
const E_FAIL: i32 = 0x80004005u32 as i32;
#[repr(C)]
#[derive(Clone, Copy)]
struct Bytes {
    pointer: usize,
    length: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Description {
    root: usize,
    shader: Bytes,
    node: u32,
    cache: Bytes,
    flags: u32,
}
struct Data {
    _native: metal::ComputePipelineState,
    _translation: Translation,
    _layout: Layout,
    _dxil: Box<[u8]>,
}
#[repr(C)]
struct Pipeline {
    vtable: *const Vtable,
    refs: AtomicU32,
    metadata: Metadata,
    data: Arc<Data>,
    parent: Parent,
}
unsafe extern "C" fn query(this: *mut Pipeline, iid: *const Guid, out: *mut *mut c_void) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    if iid.is_null() {
        return E_POINTER;
    }
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, PAGEABLE, PIPELINE].contains(unsafe { &*iid }) {
        return E_NOINTERFACE;
    }
    unsafe {
        add(this);
        *out = this.cast();
    }
    S_OK
}
unsafe extern "C" fn add(this: *mut Pipeline) -> u32 {
    super::increment(unsafe { &(*this).refs })
}
unsafe extern "C" fn release(this: *mut Pipeline) -> u32 {
    let previous = unsafe { (*this).refs.fetch_sub(1, Ordering::Release) };
    if previous == 1 {
        fence(Ordering::Acquire);
        unsafe {
            drop(Box::from_raw(this));
        }
        0
    } else {
        previous - 1
    }
}
metadata_methods!(Pipeline, set, interface, get);
unsafe extern "C" fn name(this: *mut Pipeline, text: *const u32) -> i32 {
    let m = unsafe { (*this).metadata.0 };
    unsafe { ((*(*m).vtable).set_name)(m, text) }
}
unsafe extern "C" fn device(this: *mut Pipeline, iid: *const Guid, out: *mut *mut c_void) -> i32 {
    unsafe { device_query((*this).parent.0, iid, out) }
}
unsafe extern "C" fn cached(_: *mut Pipeline, out: *mut *mut c_void) -> i32 {
    unsafe { unsupported_out("cached compute pipeline blob", out) }
}
#[repr(C)]
struct Vtable {
    query: unsafe extern "C" fn(*mut Pipeline, *const Guid, *mut *mut c_void) -> i32,
    add: unsafe extern "C" fn(*mut Pipeline) -> u32,
    release: unsafe extern "C" fn(*mut Pipeline) -> u32,
    get: unsafe extern "C" fn(*mut Pipeline, *const Guid, *mut u32, *mut c_void) -> i32,
    set: unsafe extern "C" fn(*mut Pipeline, *const Guid, u32, *const c_void) -> i32,
    interface: unsafe extern "C" fn(*mut Pipeline, *const Guid, *mut c_void) -> i32,
    name: unsafe extern "C" fn(*mut Pipeline, *const u32) -> i32,
    device: unsafe extern "C" fn(*mut Pipeline, *const Guid, *mut *mut c_void) -> i32,
    cached: unsafe extern "C" fn(*mut Pipeline, *mut *mut c_void) -> i32,
}
static VTABLE: Vtable = Vtable {
    query,
    add,
    release,
    get,
    set,
    interface,
    name,
    device,
    cached,
};
fn validate_description(d: Description) -> Result<(), i32> {
    if d.node > 1 || d.flags != 0 || d.cache.pointer != 0 || d.cache.length != 0 {
        return Err(E_NOTIMPL);
    }
    if d.root == 0 {
        return Err(E_NOTIMPL);
    }
    if d.shader.pointer == 0 {
        return Err(E_POINTER);
    }
    if !(32..=16 * 1024 * 1024).contains(&d.shader.length) {
        return Err(E_INVALIDARG);
    }
    Ok(())
}
fn validate_bindings(layout: &Layout, translation: &Translation) -> Result<(), i32> {
    if translation.requires_ray_query || translation.buffer_sizes.is_some() {
        return Err(E_NOTIMPL);
    }
    for resource in &translation.resources {
        if resource.namespace != MetalNamespace::Buffer || resource.count != 1 {
            return Err(E_NOTIMPL);
        }
        let class = match resource.class {
            RegisterClass::Srv => 0,
            RegisterClass::Uav => 1,
            RegisterClass::Cbv => 2,
            RegisterClass::Sampler => return Err(E_NOTIMPL),
        };
        let mut constants = false;
        let covered = layout.parameters.iter().any(|parameter| {
            if parameter.visibility != 0 {
                return false;
            }
            match &parameter.binding {
                Binding::Table(ranges) => ranges.iter().any(|&[c, count, first, space, _]| {
                    c == class
                        && space == resource.space
                        && resource.register >= first
                        && resource.register - first < count
                }),
                Binding::Descriptor {
                    kind,
                    register,
                    space,
                } => {
                    *kind
                        == match class {
                            0 => 3,
                            1 => 4,
                            _ => 2,
                        }
                        && *register == resource.register
                        && *space == resource.space
                }
                Binding::Constants {
                    register, space, ..
                } => {
                    constants |=
                        class == 2 && *register == resource.register && *space == resource.space;
                    false
                }
            }
        });
        if !covered {
            return Err(if constants { E_NOTIMPL } else { E_INVALIDARG });
        }
    }
    Ok(())
}
fn compile(native: &metal::DeviceRef, layout: Layout, dxil: &[u8]) -> Result<Arc<Data>, i32> {
    let translation = translate_compute_pipeline(dxil).map_err(|e| {
        eprintln!("Alkyhol compute DXIL lowering failed: {e}");
        E_NOTIMPL
    })?;
    validate_bindings(&layout, &translation)?;
    let threads = translation.threads_per_group.ok_or(E_INVALIDARG)?;
    let total = threads
        .into_iter()
        .try_fold(1u32, |total, v| total.checked_mul(v))
        .filter(|&n| n > 0 && n <= 1024)
        .ok_or(E_INVALIDARG)?;
    let options = metal::CompileOptions::new();
    options.set_fast_math_enabled(false);
    options.set_language_version(metal::MTLLanguageVersion::V3_1);
    let native = (|| -> Result<metal::ComputePipelineState, String> {
        let library = native.new_library_with_source(&translation.msl, &options)?;
        let function = library.get_function(&translation.entry_point, None)?;
        native.new_compute_pipeline_state_with_function(&function)
    })()
    .map_err(|e| {
        eprintln!("Alkyhol compute Metal compilation failed: {e}");
        E_FAIL
    })?;
    if u64::from(total) > native.max_total_threads_per_threadgroup() {
        return Err(E_NOTIMPL);
    }
    Ok(Arc::new(Data {
        _native: native,
        _translation: translation,
        _layout: layout,
        _dxil: dxil.into(),
    }))
}
pub(super) unsafe extern "C" fn create(
    this: *mut Device,
    description: *const c_void,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if !out.is_null() {
        unsafe {
            *out = ptr::null_mut();
        }
    }
    if description.is_null() || iid.is_null() {
        return E_POINTER;
    }
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, PAGEABLE, PIPELINE].contains(unsafe { &*iid }) {
        return E_NOINTERFACE;
    }
    let desc = unsafe { *description.cast::<Description>() };
    if let Err(e) = validate_description(desc) {
        return e;
    }
    let layout = match unsafe { root_signatures::snapshot(this, desc.root) } {
        Ok(v) => v,
        Err(e) => return e,
    };
    autoreleasepool(|| {
        let dxil = unsafe {
            std::slice::from_raw_parts(desc.shader.pointer as *const u8, desc.shader.length)
        };
        let data = match compile(unsafe { &(*this).metal }, layout, dxil) {
            Ok(v) => v,
            Err(e) => return e,
        };
        if out.is_null() {
            return 1;
        }
        unsafe {
            device_add(this);
            *out = Box::into_raw(Box::new(Pipeline {
                vtable: &VTABLE,
                refs: AtomicU32::new(1),
                metadata: Metadata::new(),
                data,
                parent: Parent(this),
            }))
            .cast();
        }
        S_OK
    })
}

#[cfg(test)]
#[path = "compute_pipelines_tests.rs"]
mod tests;
