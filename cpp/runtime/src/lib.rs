//! Direct-Metal DXGI source-ABI factory/adapter discovery. No D3D12 device,
//! swap chain or command submission support is implied. WCHAR is UTF-32 here.
//! FFI callers keep live references and valid host buffers; foreign metadata
//! interfaces retain the root module's thread/apartment synchronization contract.
#![allow(non_snake_case, clippy::missing_safety_doc)]
use alkyhol::{
    com::{E_NOINTERFACE, E_POINTER, Guid, IID_IUNKNOWN, S_OK},
    com_object::{AlkyholCreateMetadataObject32, MetadataObject},
};
use objc::rc::autoreleasepool;
use std::{
    ffi::c_void,
    ptr,
    sync::atomic::{AtomicU32, Ordering, fence},
};
const E_NOTIMPL: i32 = 0x80004001u32 as i32;
const E_INVALIDARG: i32 = 0x80070057u32 as i32;
const NOT_FOUND: i32 = 0x887a0002u32 as i32;
const UNSUPPORTED: i32 = 0x887a0004u32 as i32;
macro_rules! guid{($a:expr,$b:expr,$c:expr,$($d:expr),+)=>{Guid{data1:$a,data2:$b,data3:$c,data4:[$($d),+]}};}
const OBJECT: Guid = guid!(
    0xaec22fb8, 0x76f3, 0x4639, 0x9b, 0xe0, 0x28, 0xeb, 0x43, 0xa6, 0x7a, 0x2e
);
const FACTORIES: [Guid; 7] = [
    guid!(
        0x7b7166ec, 0x21c7, 0x44ae, 0xb2, 0x1a, 0xc9, 0xae, 0x32, 0x1a, 0xe3, 0x69
    ),
    guid!(
        0x770aae78, 0xf26f, 0x4dba, 0xa8, 0x29, 0x25, 0x3c, 0x83, 0xd1, 0xb3, 0x87
    ),
    guid!(
        0x50c83a1c, 0xe072, 0x4c48, 0x87, 0xb0, 0x36, 0x30, 0xfa, 0x36, 0xa6, 0xd0
    ),
    guid!(
        0x25483823, 0xcd46, 0x4c7d, 0x86, 0xca, 0x47, 0xaa, 0x95, 0xb8, 0x37, 0xbd
    ),
    guid!(
        0x1bc6ea02, 0xef36, 0x464f, 0xbf, 0x0c, 0x21, 0xca, 0x39, 0xe5, 0x16, 0x8a
    ),
    guid!(
        0x7632e1f5, 0xee65, 0x4dca, 0x87, 0xfd, 0x84, 0xcd, 0x75, 0xf8, 0x83, 0x8d
    ),
    guid!(
        0xc1b6694f, 0xff09, 0x44a9, 0xb0, 0x3c, 0x77, 0x90, 0x0a, 0x0a, 0x1d, 0x17
    ),
];
const ADAPTERS: [Guid; 2] = [
    guid!(
        0x2411e7e1, 0x12ac, 0x4ccf, 0xbd, 0x14, 0x97, 0x98, 0xe8, 0x53, 0x4d, 0xc0
    ),
    guid!(
        0x29038f61, 0x3839, 0x4626, 0x91, 0xfd, 0x08, 0x68, 0x79, 0x01, 0x1a, 0x05
    ),
];
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Luid {
    pub low: u32,
    pub high: i32,
}
impl Luid {
    fn from_id(id: u64) -> Self {
        Self {
            low: id as u32,
            high: (id >> 32) as i32,
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AdapterDesc {
    pub description: [u32; 128],
    pub vendor: u32,
    pub device: u32,
    pub subsystem: u32,
    pub revision: u32,
    pub video: usize,
    pub system: usize,
    pub shared: usize,
    pub luid: Luid,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AdapterDesc1 {
    pub base: AdapterDesc,
    pub flags: u32,
}
fn descriptor(device: &metal::DeviceRef) -> AdapterDesc {
    let mut description = [0u32; 128];
    for (dst, ch) in description.iter_mut().take(127).zip(device.name().chars()) {
        *dst = ch as u32;
    }
    // Virtual adapter budget, not an assertion of physically dedicated VRAM.
    // Unknown PCI identifiers/system-memory budgets remain zero.
    AdapterDesc {
        description,
        vendor: 0,
        device: 0,
        subsystem: 0,
        revision: 0,
        video: usize::try_from(device.recommended_max_working_set_size()).unwrap_or(usize::MAX),
        system: 0,
        shared: 0,
        luid: Luid::from_id(device.registry_id()),
    }
}
struct Metadata(*mut MetadataObject);
impl Metadata {
    fn new() -> Self {
        let mut pointer = ptr::null_mut();
        let hr = unsafe { AlkyholCreateMetadataObject32(&mut pointer) };
        assert_eq!(hr, S_OK);
        Self(pointer)
    }
}
impl Drop for Metadata {
    fn drop(&mut self) {
        unsafe {
            ((*(*self.0).vtable).release)(self.0);
        }
    }
}
#[repr(C)]
pub struct Factory {
    vtable: *const FactoryVtable,
    refs: AtomicU32,
    metadata: Metadata,
    devices: Vec<metal::Device>,
}
#[repr(C)]
pub struct Adapter {
    vtable: *const AdapterVtable,
    refs: AtomicU32,
    metadata: Metadata,
    parent: *mut Factory,
    index: usize,
}
impl Drop for Adapter {
    fn drop(&mut self) {
        unsafe {
            factory_release(self.parent);
        }
    }
}
fn increment(refs: &AtomicU32) -> u32 {
    match refs.try_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1)) {
        Ok(old) => old + 1,
        Err(_) => std::process::abort(),
    }
}
unsafe fn clear(out: *mut *mut c_void) -> Result<(), i32> {
    if out.is_null() {
        return Err(E_POINTER);
    }
    unsafe {
        *out = ptr::null_mut();
    }
    Ok(())
}
fn unavailable(name: &str) -> i32 {
    alkyhol::kernel::set_last_error(50);
    eprintln!("Alkyhol Metal DXGI: {name} is not implemented");
    E_NOTIMPL
}
unsafe fn unsupported_out(name: &str, out: *mut *mut c_void) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    unavailable(name)
}
unsafe extern "C" fn factory_query(
    this: *mut Factory,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    if iid.is_null() {
        return E_POINTER;
    }
    let id = unsafe { *iid };
    if id != IID_IUNKNOWN && id != OBJECT && !FACTORIES.contains(&id) {
        return E_NOINTERFACE;
    }
    unsafe {
        factory_add(this);
        *out = this.cast();
    }
    S_OK
}
unsafe extern "C" fn factory_add(this: *mut Factory) -> u32 {
    increment(unsafe { &(*this).refs })
}
unsafe extern "C" fn factory_release(this: *mut Factory) -> u32 {
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
unsafe extern "C" fn adapter_query(
    this: *mut Adapter,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    if iid.is_null() {
        return E_POINTER;
    }
    let id = unsafe { *iid };
    if id != IID_IUNKNOWN && id != OBJECT && !ADAPTERS.contains(&id) {
        return E_NOINTERFACE;
    }
    unsafe {
        adapter_add(this);
        *out = this.cast();
    }
    S_OK
}
unsafe extern "C" fn adapter_add(this: *mut Adapter) -> u32 {
    increment(unsafe { &(*this).refs })
}
unsafe extern "C" fn adapter_release(this: *mut Adapter) -> u32 {
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
macro_rules! metadata_methods {
    ($ty:ty,$set:ident,$interface:ident,$get:ident) => {
        unsafe extern "C" fn $set(
            this: *mut $ty,
            key: *const Guid,
            size: u32,
            input: *const c_void,
        ) -> i32 {
            let meta = unsafe { (*this).metadata.0 };
            unsafe { ((*(*meta).vtable).set)(meta, key, size, input) }
        }
        unsafe extern "C" fn $interface(
            this: *mut $ty,
            key: *const Guid,
            input: *mut c_void,
        ) -> i32 {
            let meta = unsafe { (*this).metadata.0 };
            unsafe { ((*(*meta).vtable).set_interface)(meta, key, input) }
        }
        unsafe extern "C" fn $get(
            this: *mut $ty,
            key: *const Guid,
            size: *mut u32,
            out: *mut c_void,
        ) -> i32 {
            let meta = unsafe { (*this).metadata.0 };
            unsafe { ((*(*meta).vtable).get)(meta, key, size, out) }
        }
    };
}
metadata_methods!(Factory, factory_set, factory_interface, factory_get);
metadata_methods!(Adapter, adapter_set, adapter_interface, adapter_get);
unsafe extern "C" fn factory_parent(_: *mut Factory, _: *const Guid, out: *mut *mut c_void) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    E_NOINTERFACE
}
unsafe extern "C" fn adapter_parent(
    this: *mut Adapter,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    unsafe { factory_query((*this).parent, iid, out) }
}
unsafe fn enumerate(
    this: *mut Factory,
    index: usize,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    if iid.is_null() {
        return E_POINTER;
    }
    if index >= unsafe { (*this).devices.len() } {
        return NOT_FOUND;
    }
    let id = unsafe { *iid };
    if id != IID_IUNKNOWN && id != OBJECT && !ADAPTERS.contains(&id) {
        return E_NOINTERFACE;
    }
    unsafe {
        factory_add(this);
    }
    let adapter = Box::new(Adapter {
        vtable: &ADAPTER_VTABLE,
        refs: AtomicU32::new(1),
        metadata: Metadata::new(),
        parent: this,
        index,
    });
    unsafe {
        *out = Box::into_raw(adapter).cast();
    }
    S_OK
}
unsafe extern "C" fn enum_adapters(this: *mut Factory, index: u32, out: *mut *mut c_void) -> i32 {
    unsafe { enumerate(this, index as usize, &ADAPTERS[0], out) }
}
unsafe extern "C" fn enum_adapters1(this: *mut Factory, index: u32, out: *mut *mut c_void) -> i32 {
    unsafe { enumerate(this, index as usize, &ADAPTERS[1], out) }
}
unsafe extern "C" fn current(this: *mut Factory) -> i32 {
    autoreleasepool(|| {
        let now: Vec<_> = metal::Device::all()
            .iter()
            .map(|d| d.registry_id())
            .collect();
        let saved: Vec<_> = unsafe { &(*this).devices }
            .iter()
            .map(|d| d.registry_id())
            .collect();
        i32::from(now == saved)
    })
}
unsafe extern "C" fn by_luid(
    this: *mut Factory,
    luid: Luid,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    if iid.is_null() {
        return E_POINTER;
    }
    let devices = unsafe { &(*this).devices };
    let indices: Vec<_> = devices
        .iter()
        .enumerate()
        .filter(|(_, d)| Luid::from_id(d.registry_id()) == luid)
        .map(|(i, _)| i)
        .collect();
    match indices.as_slice() {
        [index] => unsafe { enumerate(this, *index, iid, out) },
        [] => NOT_FOUND,
        _ => unavailable("ambiguous Metal registry identity"),
    }
}
unsafe extern "C" fn by_preference(
    this: *mut Factory,
    index: u32,
    preference: i32,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if !(0..=2).contains(&preference) {
        if let Err(e) = unsafe { clear(out) } {
            return e;
        }
        return E_INVALIDARG;
    }
    if preference != 0 {
        if let Err(e) = unsafe { clear(out) } {
            return e;
        }
        return unavailable("GPU preference ordering");
    }
    unsafe { enumerate(this, index as usize, iid, out) }
}
unsafe extern "C" fn get_desc(this: *mut Adapter, out: *mut AdapterDesc) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    autoreleasepool(|| {
        let d = unsafe { &(&(*(*this).parent).devices)[(*this).index] };
        unsafe {
            ptr::write(out, descriptor(d));
        }
        S_OK
    })
}
unsafe extern "C" fn get_desc1(this: *mut Adapter, out: *mut AdapterDesc1) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    autoreleasepool(|| {
        let d = unsafe { &(&(*(*this).parent).devices)[(*this).index] };
        unsafe {
            ptr::write(
                out,
                AdapterDesc1 {
                    base: descriptor(d),
                    flags: 0,
                },
            );
        }
        S_OK
    })
}
unsafe extern "C" fn outputs(_: *mut Adapter, _: u32, out: *mut *mut c_void) -> i32 {
    unsafe { unsupported_out("output enumeration", out) }
}
unsafe extern "C" fn check_interface(_: *mut Adapter, _: *const Guid, version: *mut i64) -> i32 {
    if !version.is_null() {
        unsafe {
            *version = 0;
        }
    }
    E_NOINTERFACE
}
unsafe extern "C" fn make_window(_: *mut Factory, _: usize, _: u32) -> i32 {
    unavailable("window association")
}
unsafe extern "C" fn get_window(_: *mut Factory, out: *mut usize) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = 0;
    }
    unavailable("window association")
}
unsafe extern "C" fn swap(_: *mut Factory, _: usize, _: usize, out: *mut *mut c_void) -> i32 {
    unsafe { unsupported_out("swap chain", out) }
}
unsafe extern "C" fn software(_: *mut Factory, _: usize, out: *mut *mut c_void) -> i32 {
    unsafe { unsupported_out("software adapter", out) }
}
unsafe extern "C" fn stereo(_: *mut Factory) -> i32 {
    unavailable("stereo presentation");
    0
}
unsafe extern "C" fn swap_hwnd(
    _: *mut Factory,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    out: *mut *mut c_void,
) -> i32 {
    unsafe { unsupported_out("HWND swap chain", out) }
}
unsafe extern "C" fn swap_core(
    _: *mut Factory,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    out: *mut *mut c_void,
) -> i32 {
    unsafe { unsupported_out("CoreWindow swap chain", out) }
}
unsafe extern "C" fn shared_luid(_: *mut Factory, _: usize, out: *mut Luid) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = Luid { low: 0, high: 0 };
    }
    unavailable("shared resource adapter")
}
unsafe extern "C" fn register_window(_: *mut Factory, _: usize, _: u32, cookie: *mut u32) -> i32 {
    if cookie.is_null() {
        return E_POINTER;
    }
    unsafe {
        *cookie = 0;
    }
    unavailable("window notification")
}
unsafe extern "C" fn register_event(_: *mut Factory, _: usize, cookie: *mut u32) -> i32 {
    if cookie.is_null() {
        return E_POINTER;
    }
    unsafe {
        *cookie = 0;
    }
    unavailable("event notification")
}
unsafe extern "C" fn unregister(_: *mut Factory, _: u32) {
    unavailable("notification unregister");
}
unsafe extern "C" fn composition(
    _: *mut Factory,
    _: usize,
    _: usize,
    _: usize,
    out: *mut *mut c_void,
) -> i32 {
    unsafe { unsupported_out("composition swap chain", out) }
}
unsafe extern "C" fn flags(_: *mut Factory) -> u32 {
    0
}
unsafe extern "C" fn warp(_: *mut Factory, _: *const Guid, out: *mut *mut c_void) -> i32 {
    unsafe { unsupported_out("WARP adapter", out) }
}
unsafe extern "C" fn feature(_: *mut Factory, _: i32, _: usize, _: u32) -> i32 {
    unavailable("presentation feature support")
}
#[repr(C)]
struct FactoryVtable {
    query: unsafe extern "C" fn(*mut Factory, *const Guid, *mut *mut c_void) -> i32,
    add: unsafe extern "C" fn(*mut Factory) -> u32,
    release: unsafe extern "C" fn(*mut Factory) -> u32,
    set: unsafe extern "C" fn(*mut Factory, *const Guid, u32, *const c_void) -> i32,
    interface: unsafe extern "C" fn(*mut Factory, *const Guid, *mut c_void) -> i32,
    get: unsafe extern "C" fn(*mut Factory, *const Guid, *mut u32, *mut c_void) -> i32,
    parent: unsafe extern "C" fn(*mut Factory, *const Guid, *mut *mut c_void) -> i32,
    adapters: unsafe extern "C" fn(*mut Factory, u32, *mut *mut c_void) -> i32,
    window: unsafe extern "C" fn(*mut Factory, usize, u32) -> i32,
    get_window: unsafe extern "C" fn(*mut Factory, *mut usize) -> i32,
    swap: unsafe extern "C" fn(*mut Factory, usize, usize, *mut *mut c_void) -> i32,
    software: unsafe extern "C" fn(*mut Factory, usize, *mut *mut c_void) -> i32,
    adapters1: unsafe extern "C" fn(*mut Factory, u32, *mut *mut c_void) -> i32,
    current: unsafe extern "C" fn(*mut Factory) -> i32,
    stereo: unsafe extern "C" fn(*mut Factory) -> i32,
    swap_hwnd: unsafe extern "C" fn(
        *mut Factory,
        usize,
        usize,
        usize,
        usize,
        usize,
        *mut *mut c_void,
    ) -> i32,
    swap_core:
        unsafe extern "C" fn(*mut Factory, usize, usize, usize, usize, *mut *mut c_void) -> i32,
    shared_luid: unsafe extern "C" fn(*mut Factory, usize, *mut Luid) -> i32,
    stereo_window: unsafe extern "C" fn(*mut Factory, usize, u32, *mut u32) -> i32,
    stereo_event: unsafe extern "C" fn(*mut Factory, usize, *mut u32) -> i32,
    unregister_stereo: unsafe extern "C" fn(*mut Factory, u32),
    occlusion_window: unsafe extern "C" fn(*mut Factory, usize, u32, *mut u32) -> i32,
    occlusion_event: unsafe extern "C" fn(*mut Factory, usize, *mut u32) -> i32,
    unregister_occlusion: unsafe extern "C" fn(*mut Factory, u32),
    composition: unsafe extern "C" fn(*mut Factory, usize, usize, usize, *mut *mut c_void) -> i32,
    flags: unsafe extern "C" fn(*mut Factory) -> u32,
    by_luid: unsafe extern "C" fn(*mut Factory, Luid, *const Guid, *mut *mut c_void) -> i32,
    warp: unsafe extern "C" fn(*mut Factory, *const Guid, *mut *mut c_void) -> i32,
    feature: unsafe extern "C" fn(*mut Factory, i32, usize, u32) -> i32,
    preference: unsafe extern "C" fn(*mut Factory, u32, i32, *const Guid, *mut *mut c_void) -> i32,
}
static FACTORY_VTABLE: FactoryVtable = FactoryVtable {
    query: factory_query,
    add: factory_add,
    release: factory_release,
    set: factory_set,
    interface: factory_interface,
    get: factory_get,
    parent: factory_parent,
    adapters: enum_adapters,
    window: make_window,
    get_window,
    swap,
    software,
    adapters1: enum_adapters1,
    current,
    stereo,
    swap_hwnd,
    swap_core,
    shared_luid,
    stereo_window: register_window,
    stereo_event: register_event,
    unregister_stereo: unregister,
    occlusion_window: register_window,
    occlusion_event: register_event,
    unregister_occlusion: unregister,
    composition,
    flags,
    by_luid,
    warp,
    feature,
    preference: by_preference,
};
#[repr(C)]
struct AdapterVtable {
    query: unsafe extern "C" fn(*mut Adapter, *const Guid, *mut *mut c_void) -> i32,
    add: unsafe extern "C" fn(*mut Adapter) -> u32,
    release: unsafe extern "C" fn(*mut Adapter) -> u32,
    set: unsafe extern "C" fn(*mut Adapter, *const Guid, u32, *const c_void) -> i32,
    interface: unsafe extern "C" fn(*mut Adapter, *const Guid, *mut c_void) -> i32,
    get: unsafe extern "C" fn(*mut Adapter, *const Guid, *mut u32, *mut c_void) -> i32,
    parent: unsafe extern "C" fn(*mut Adapter, *const Guid, *mut *mut c_void) -> i32,
    outputs: unsafe extern "C" fn(*mut Adapter, u32, *mut *mut c_void) -> i32,
    desc: unsafe extern "C" fn(*mut Adapter, *mut AdapterDesc) -> i32,
    check: unsafe extern "C" fn(*mut Adapter, *const Guid, *mut i64) -> i32,
    desc1: unsafe extern "C" fn(*mut Adapter, *mut AdapterDesc1) -> i32,
}
static ADAPTER_VTABLE: AdapterVtable = AdapterVtable {
    query: adapter_query,
    add: adapter_add,
    release: adapter_release,
    set: adapter_set,
    interface: adapter_interface,
    get: adapter_get,
    parent: adapter_parent,
    outputs,
    desc: get_desc,
    check: check_interface,
    desc1: get_desc1,
};
/// Create an actual Metal-backed adapter-inventory factory, supporting only flags
/// zero. Interfaces do not promise that any D3D12 device can be created.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn CreateDXGIFactory2(
    creation_flags: u32,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    if iid.is_null() {
        return E_POINTER;
    }
    if creation_flags != 0 {
        return unavailable("factory creation flags/debug layer");
    }
    if unsafe { *iid } != IID_IUNKNOWN
        && unsafe { *iid } != OBJECT
        && !FACTORIES.contains(unsafe { &*iid })
    {
        return E_NOINTERFACE;
    }
    autoreleasepool(|| {
        let devices = metal::Device::all();
        if devices.is_empty() {
            return UNSUPPORTED;
        }
        for device in &devices {
            eprintln!(
                "Metal adapter: {} registry={} budget={} RT={}",
                device.name(),
                device.registry_id(),
                device.recommended_max_working_set_size(),
                device.supports_raytracing()
            );
        }
        let factory = Box::new(Factory {
            vtable: &FACTORY_VTABLE,
            refs: AtomicU32::new(1),
            metadata: Metadata::new(),
            devices,
        });
        unsafe {
            *out = Box::into_raw(factory).cast();
        }
        S_OK
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn luid_preserves_every_identity_bit_and_source_descriptor_layouts() {
        for id in [0, 1, u32::MAX as u64, 0x8000000000000000, u64::MAX] {
            let l = Luid::from_id(id);
            assert_eq!(u64::from(l.low) | ((l.high as u32 as u64) << 32), id);
        }
        assert_eq!(std::mem::size_of::<Luid>(), 8);
        assert_eq!(std::mem::size_of::<AdapterDesc>(), 560);
        assert_eq!(std::mem::size_of::<AdapterDesc1>(), 568);
        assert_eq!(std::mem::offset_of!(AdapterDesc1, flags), 560);
        assert_eq!(
            std::mem::size_of::<FactoryVtable>(),
            30 * std::mem::size_of::<usize>()
        );
        assert_eq!(
            std::mem::size_of::<AdapterVtable>(),
            11 * std::mem::size_of::<usize>()
        );
    }
    #[test]
    fn constructor_validation_never_returns_an_interface_on_failure() {
        unsafe {
            let mut out = ptr::dangling_mut();
            assert_eq!(
                CreateDXGIFactory2(0, &FACTORIES[6], ptr::null_mut()),
                E_POINTER
            );
            assert_eq!(CreateDXGIFactory2(0, ptr::null(), &mut out), E_POINTER);
            assert!(out.is_null());
            out = ptr::dangling_mut();
            assert_eq!(CreateDXGIFactory2(1, &FACTORIES[6], &mut out), E_NOTIMPL);
            assert!(out.is_null());
            out = ptr::dangling_mut();
            assert_eq!(CreateDXGIFactory2(0, &ADAPTERS[0], &mut out), E_NOINTERFACE);
            assert!(out.is_null());
        }
    }
    #[test]
    fn actual_metal_inventory_descriptors_identity_and_parent_lifetimes() {
        autoreleasepool(|| unsafe {
            let mut out = ptr::null_mut();
            assert_eq!(CreateDXGIFactory2(0, &FACTORIES[6], &mut out), S_OK);
            let factory = out.cast::<Factory>();
            let expected = metal::Device::all();
            assert_eq!((*factory).devices.len(), expected.len());
            assert!(!expected.is_empty());
            for (i, device) in expected.iter().enumerate() {
                let d = descriptor(device);
                assert_eq!(d.video, device.recommended_max_working_set_size() as usize);
                assert_eq!(d.luid, Luid::from_id(device.registry_id()));
                assert_eq!(
                    d.description
                        .iter()
                        .copied()
                        .take_while(|v| *v != 0)
                        .map(|v| char::from_u32(v).unwrap())
                        .collect::<String>(),
                    device.name().chars().take(127).collect::<String>()
                );
                let mut adapter = ptr::null_mut();
                assert_eq!(enum_adapters1(factory, i as u32, &mut adapter), S_OK);
                let adapter = adapter.cast::<Adapter>();
                let mut desc = std::mem::MaybeUninit::uninit();
                assert_eq!(get_desc1(adapter, desc.as_mut_ptr()), S_OK);
                assert_eq!(desc.assume_init().base.luid, d.luid);
                let mut parent = ptr::null_mut();
                assert_eq!(adapter_parent(adapter, &FACTORIES[6], &mut parent), S_OK);
                assert_eq!(parent, out);
                factory_release(parent.cast());
                assert_eq!(adapter_release(adapter), 0);
            }
            let mut missing = ptr::dangling_mut();
            assert_eq!(
                enum_adapters1(factory, expected.len() as u32, &mut missing),
                NOT_FOUND
            );
            assert!(missing.is_null());
            assert_eq!(current(factory), 1);
            assert_eq!(enum_adapters1(factory, 0, &mut out), S_OK);
            let adapter = out.cast::<Adapter>();
            assert_eq!(factory_release(factory), 1);
            let mut parent = ptr::null_mut();
            assert_eq!(adapter_parent(adapter, &IID_IUNKNOWN, &mut parent), S_OK);
            assert_eq!(parent, factory.cast());
            factory_release(parent.cast());
            assert_eq!(adapter_release(adapter), 0);
        });
    }
    #[test]
    fn every_factory_version_keeps_one_canonical_identity() {
        autoreleasepool(|| unsafe {
            for iid in FACTORIES {
                let mut out = ptr::null_mut();
                assert_eq!(CreateDXGIFactory2(0, &iid, &mut out), S_OK);
                let factory = out.cast::<Factory>();
                for other in FACTORIES.into_iter().chain([IID_IUNKNOWN, OBJECT]) {
                    let mut queried = ptr::null_mut();
                    assert_eq!(factory_query(factory, &other, &mut queried), S_OK);
                    assert_eq!(queried, out);
                    assert_eq!(factory_release(queried.cast()), 1);
                }
                assert_eq!(factory_release(factory), 0);
            }
        });
    }
    #[test]
    fn invalid_and_unavailable_selection_clears_outputs_without_ref_leaks() {
        autoreleasepool(|| unsafe {
            let mut out = ptr::null_mut();
            assert_eq!(CreateDXGIFactory2(0, &FACTORIES[6], &mut out), S_OK);
            let factory = out.cast::<Factory>();
            let mut value = ptr::dangling_mut();
            assert_eq!(
                by_preference(factory, 0, 3, &ADAPTERS[1], &mut value),
                E_INVALIDARG
            );
            assert!(value.is_null());
            for preference in [1, 2] {
                value = ptr::dangling_mut();
                assert_eq!(
                    by_preference(factory, 0, preference, &ADAPTERS[1], &mut value),
                    E_NOTIMPL
                );
                assert!(value.is_null());
            }
            let id = Luid::from_id((&(*factory).devices)[0].registry_id());
            assert_eq!(by_luid(factory, id, ptr::null(), &mut value), E_POINTER);
            assert_eq!(
                enumerate(factory, 0, &FACTORIES[0], &mut value),
                E_NOINTERFACE
            );
            assert!(value.is_null());
            assert_eq!(factory_release(factory), 0);
        });
    }
    #[test]
    fn interface_refs_are_atomic_while_the_parent_snapshot_remains_owned() {
        autoreleasepool(|| unsafe {
            let mut out = ptr::null_mut();
            assert_eq!(CreateDXGIFactory2(0, &FACTORIES[6], &mut out), S_OK);
            let address = out as usize;
            let threads: Vec<_> = (0..8)
                .map(|_| {
                    std::thread::spawn(move || {
                        let factory = address as *mut Factory;
                        for _ in 0..2000 {
                            factory_add(factory);
                            factory_release(factory);
                        }
                    })
                })
                .collect();
            for thread in threads {
                thread.join().unwrap();
            }
            assert_eq!(factory_release(out.cast()), 0);
        });
    }
}
