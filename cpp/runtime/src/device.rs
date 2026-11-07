//! Restricted, explicitly constructed Metal device/resource COM subset.
//! D3D12CreateDevice remains unavailable: this is not a feature-level claim.
use super::*;
use alkyhol::com::E_OUTOFMEMORY;
use alkyhol::com_object::IID_ID3D12OBJECT;
use metal::foreign_types::ForeignType;
use std::sync::{Arc, Mutex};
const DEVICE: Guid = guid!(
    0x189819f1, 0x1db6, 0x4b57, 0xbe, 0x54, 0x18, 0x21, 0x33, 0x9b, 0x85, 0xf7
);
const CHILD: Guid = guid!(
    0x905db94b, 0xa00c, 0x4140, 0x9d, 0xf5, 0x2b, 0x64, 0xca, 0x9e, 0xa3, 0x57
);
const PAGEABLE: Guid = guid!(
    0x63ee58fb, 0x1268, 0x4835, 0x86, 0xda, 0xf0, 0x08, 0xce, 0x62, 0xf0, 0xd6
);
const RESOURCE: Guid = guid!(
    0x696442be, 0xa72e, 0x4059, 0xbc, 0x79, 0x5b, 0x5c, 0x98, 0x04, 0x0f, 0xad
);
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeapProperties {
    kind: i32,
    page: i32,
    pool: i32,
    creation: u32,
    visible: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceDesc {
    dimension: i32,
    alignment: u64,
    width: u64,
    height: u32,
    depth: u16,
    mips: u16,
    format: i32,
    samples: u32,
    quality: u32,
    layout: i32,
    flags: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct AllocationInfo {
    size: u64,
    alignment: u64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Range {
    begin: usize,
    end: usize,
}
#[repr(C)]
pub struct Device {
    vtable: *const DeviceVtable,
    refs: AtomicU32,
    metadata: Metadata,
    metal: metal::Device,
    byte_copy: byte_copy::Kernel,
}
struct Parent(*mut Device);
impl Drop for Parent {
    fn drop(&mut self) {
        unsafe {
            device_release(self.0);
        }
    }
}
struct BufferUsage {
    current: u32,
    pending: usize,
    owner: Option<Arc<()>>,
    failed: bool,
}
fn valid_default_state(state: u32) -> bool {
    state == 0x400 || state & !0xac3 == 0
}
#[repr(C)]
pub struct Resource {
    vtable: *const ResourceVtable,
    refs: AtomicU32,
    metadata: Metadata,
    buffer: metal::Buffer,
    offset: usize,
    desc: ResourceDesc,
    heap: HeapProperties,
    usage: Arc<Mutex<BufferUsage>>,
    parent: Parent,
}
unsafe extern "C" fn device_query(
    this: *mut Device,
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
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, DEVICE].contains(&id) {
        return E_NOINTERFACE;
    }
    unsafe {
        device_add(this);
        *out = this.cast();
    }
    S_OK
}
unsafe extern "C" fn device_add(this: *mut Device) -> u32 {
    increment(unsafe { &(*this).refs })
}
unsafe extern "C" fn device_release(this: *mut Device) -> u32 {
    let n = unsafe { (*this).refs.fetch_sub(1, Ordering::Release) };
    if n == 1 {
        fence(Ordering::Acquire);
        unsafe {
            drop(Box::from_raw(this));
        }
        0
    } else {
        n - 1
    }
}
unsafe extern "C" fn resource_query(
    this: *mut Resource,
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
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, PAGEABLE, RESOURCE].contains(&id) {
        return E_NOINTERFACE;
    }
    unsafe {
        resource_add(this);
        *out = this.cast();
    }
    S_OK
}
unsafe extern "C" fn resource_add(this: *mut Resource) -> u32 {
    increment(unsafe { &(*this).refs })
}
unsafe extern "C" fn resource_release(this: *mut Resource) -> u32 {
    let n = unsafe { (*this).refs.fetch_sub(1, Ordering::Release) };
    if n == 1 {
        fence(Ordering::Acquire);
        unsafe {
            drop(Box::from_raw(this));
        }
        0
    } else {
        n - 1
    }
}
metadata_methods!(Device, device_set, device_interface, device_get);
metadata_methods!(Resource, resource_set, resource_interface, resource_get);
unsafe extern "C" fn device_name(this: *mut Device, name: *const u32) -> i32 {
    let m = unsafe { (*this).metadata.0 };
    unsafe { ((*(*m).vtable).set_name)(m, name) }
}
unsafe extern "C" fn resource_name(this: *mut Resource, name: *const u32) -> i32 {
    let m = unsafe { (*this).metadata.0 };
    unsafe { ((*(*m).vtable).set_name)(m, name) }
}
unsafe extern "C" fn nodes(_: *mut Device) -> u32 {
    1
}
unsafe extern "C" fn adapter_luid(this: *mut Device) -> Luid {
    Luid::from_id(unsafe { (*this).metal.registry_id() })
}
unsafe extern "C" fn resource_device(
    this: *mut Resource,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    unsafe { device_query((*this).parent.0, iid, out) }
}
unsafe extern "C" fn feature(_: *mut Device, kind: i32, out: *mut c_void, size: u32) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    if kind == 27 {
        if size != 12 {
            return E_INVALIDARG;
        }
        unsafe {
            ptr::write_bytes(out.cast::<u8>(), 0, 12);
        }
        return S_OK;
    }
    unavailable("D3D12 feature support")
}
fn valid_buffer(
    heap: &HeapProperties,
    flags: u32,
    desc: &ResourceDesc,
    state: u32,
    clear_value: *const c_void,
) -> Result<(), i32> {
    if flags != 0
        || !clear_value.is_null()
        || desc.dimension != 1
        || desc.flags != 0
        || desc.alignment != 0
    {
        return Err(E_NOTIMPL);
    }
    if ![1, 2, 3].contains(&heap.kind) {
        return Err(E_NOTIMPL);
    }
    if heap.page != 0 || heap.pool != 0 || heap.creation != 1 || heap.visible != 1 {
        return Err(E_NOTIMPL);
    }
    if desc.width == 0
        || desc.height != 1
        || desc.depth != 1
        || desc.mips != 1
        || desc.format != 0
        || desc.samples != 1
        || desc.quality != 0
        || desc.layout != 1
    {
        return Err(E_INVALIDARG);
    }
    if heap.kind == 1 && !valid_default_state(state) {
        return Err(E_NOTIMPL);
    }
    if (heap.kind == 2 && state != 0xac3) || (heap.kind == 3 && state != 0x400) {
        return Err(E_INVALIDARG);
    }
    Ok(())
}
unsafe extern "C" fn committed(
    this: *mut Device,
    heap: *const HeapProperties,
    flags: u32,
    desc: *const ResourceDesc,
    state: u32,
    clear_value: *const c_void,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if !out.is_null() {
        unsafe {
            *out = ptr::null_mut();
        }
    }
    if heap.is_null() || desc.is_null() || iid.is_null() {
        return E_POINTER;
    }
    let heap = unsafe { *heap };
    let mut desc = unsafe { *desc };
    if let Err(e) = valid_buffer(&heap, flags, &desc, state, clear_value) {
        return e;
    }
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, PAGEABLE, RESOURCE].contains(unsafe { &*iid }) {
        return E_NOINTERFACE;
    }
    let Some(length) = desc.width.checked_add(65535) else {
        return E_INVALIDARG;
    };
    if length > unsafe { (*this).metal.max_buffer_length() } {
        return E_INVALIDARG;
    }
    if out.is_null() {
        return 1;
    }
    autoreleasepool(|| {
        let storage = if heap.kind == 1 {
            metal::MTLResourceOptions::StorageModePrivate
        } else {
            metal::MTLResourceOptions::StorageModeShared
        };
        let buffer = unsafe {
            (*this).metal.new_buffer(
                length,
                storage | metal::MTLResourceOptions::HazardTrackingModeTracked,
            )
        };
        if buffer.as_ptr().is_null() {
            return E_OUTOFMEMORY;
        }
        let address = buffer.gpu_address();
        if address == 0 {
            return E_NOTIMPL;
        }
        let offset = ((65536 - (address & 65535)) & 65535) as usize;
        // Preserve the measured committed-buffer descriptor and actual address
        // alignment by exposing an aligned subspan, not a manufactured GPU VA.
        desc.alignment = 65536;
        unsafe {
            device_add(this);
        }
        let resource = Box::new(Resource {
            vtable: &RESOURCE_VTABLE,
            refs: AtomicU32::new(1),
            metadata: Metadata::new(),
            buffer,
            offset,
            desc,
            heap,
            usage: Arc::new(Mutex::new(BufferUsage {
                current: state,
                pending: 0,
                owner: None,
                failed: false,
            })),
            parent: Parent(this),
        });
        unsafe {
            *out = Box::into_raw(resource).cast();
        }
        S_OK
    })
}
unsafe extern "C" fn map(
    this: *mut Resource,
    subresource: u32,
    range: *const Range,
    out: *mut *mut c_void,
) -> i32 {
    if !out.is_null() {
        unsafe {
            *out = ptr::null_mut();
        }
    }
    if subresource != 0 {
        return E_INVALIDARG;
    }
    if !range.is_null() {
        let range = unsafe { *range };
        if range.begin < range.end && range.end as u64 > unsafe { (*this).desc.width } {
            return E_INVALIDARG;
        }
    }
    if unsafe { (*this).heap.kind } == 1 {
        return E_INVALIDARG;
    }
    let contents = unsafe { (*this).buffer.contents() };
    if contents.is_null() {
        return E_NOTIMPL;
    }
    if !out.is_null() {
        unsafe {
            *out = contents.cast::<u8>().add((*this).offset).cast();
        }
    }
    S_OK
}
unsafe extern "C" fn unmap(this: *mut Resource, subresource: u32, range: *const Range) {
    if unsafe { (*this).heap.kind } == 1 {
        unsupported_void("Unmap of non-CPU-accessible DEFAULT buffer");
    }
    if subresource != 0 {
        unsupported_void("invalid buffer subresource in Unmap");
    }
    if !range.is_null() {
        let range = unsafe { *range };
        if range.begin < range.end && range.end as u64 > unsafe { (*this).desc.width } {
            unsupported_void("invalid written range");
        }
    }
    // Shared Metal storage is coherent; no mapping count or cache flush is required.
}
unsafe extern "C" fn resource_desc(this: *mut Resource) -> ResourceDesc {
    unsafe { (*this).desc }
}
unsafe extern "C" fn gpu_address(this: *mut Resource) -> u64 {
    unsafe { (*this).buffer.gpu_address() + (*this).offset as u64 }
}
unsafe extern "C" fn heap_properties(
    this: *mut Resource,
    heap: *mut HeapProperties,
    flags: *mut u32,
) -> i32 {
    if !heap.is_null() {
        unsafe {
            *heap = (*this).heap;
        }
    }
    if !flags.is_null() {
        unsafe {
            *flags = 0xc0;
        }
    }
    S_OK
}
fn unsupported_void(name: &str) -> ! {
    unavailable(name);
    std::process::abort()
}
unsafe extern "C" fn desc_object(
    _: *mut Device,
    _: *const c_void,
    _: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    unsafe { unsupported_out("D3D12 object creation", out) }
}
unsafe extern "C" fn descriptor_increment(_: *mut Device, _: i32) -> u32 {
    unsupported_void("descriptor increment size")
}
unsafe extern "C" fn root_signature(
    _: *mut Device,
    _: u32,
    _: usize,
    _: usize,
    _: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    unsafe { unsupported_out("root signature", out) }
}
unsafe extern "C" fn view2(_: *mut Device, _: usize, _: usize) {
    unsupported_void("descriptor view")
}
unsafe extern "C" fn view3(_: *mut Device, _: usize, _: usize, _: usize) {
    unsupported_void("resource view")
}
unsafe extern "C" fn view4(_: *mut Device, _: usize, _: usize, _: usize, _: usize) {
    unsupported_void("unordered access view")
}
unsafe extern "C" fn copy_descriptors(
    _: *mut Device,
    _: u32,
    _: usize,
    _: usize,
    _: u32,
    _: usize,
    _: usize,
    _: i32,
) {
    unsupported_void("descriptor copy")
}
unsafe extern "C" fn copy_simple(_: *mut Device, _: u32, _: usize, _: usize, _: i32) {
    unsupported_void("descriptor copy")
}
unsafe extern "C" fn allocation_info(_: *mut Device, _: u32, _: u32, _: usize) -> AllocationInfo {
    unavailable("resource allocation info");
    AllocationInfo {
        size: u64::MAX,
        alignment: 0,
    }
}
unsafe extern "C" fn custom_heap(_: *mut Device, _: u32, _: i32) -> HeapProperties {
    unsupported_void("custom heap properties")
}
unsafe extern "C" fn placed(
    _: *mut Device,
    _: usize,
    _: u64,
    _: usize,
    _: u32,
    _: usize,
    _: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    unsafe { unsupported_out("placed resource", out) }
}
unsafe extern "C" fn reserved(
    _: *mut Device,
    _: usize,
    _: u32,
    _: usize,
    _: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    unsafe { unsupported_out("reserved resource", out) }
}
unsafe extern "C" fn shared(
    _: *mut Device,
    _: usize,
    _: usize,
    _: u32,
    _: usize,
    out: *mut usize,
) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = 0;
    }
    unavailable("shared handle")
}
unsafe extern "C" fn open(_: *mut Device, _: usize, _: *const Guid, out: *mut *mut c_void) -> i32 {
    unsafe { unsupported_out("shared resource", out) }
}
unsafe extern "C" fn open_name(_: *mut Device, _: usize, _: u32, out: *mut usize) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = 0;
    }
    unavailable("named shared resource")
}
unsafe extern "C" fn resident(_: *mut Device, _: u32, _: usize) -> i32 {
    unavailable("residency control")
}
unsafe extern "C" fn removed(_: *mut Device) -> i32 {
    unavailable("device removal status")
}
unsafe extern "C" fn footprints(
    _: *mut Device,
    _: usize,
    _: u32,
    _: u32,
    _: u64,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
) {
    unsupported_void("copyable footprints")
}
unsafe extern "C" fn stable(_: *mut Device, _: i32) -> i32 {
    unavailable("stable power state")
}
unsafe extern "C" fn signature(
    _: *mut Device,
    _: usize,
    _: usize,
    _: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    unsafe { unsupported_out("command signature", out) }
}
unsafe extern "C" fn tiling(
    _: *mut Device,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    _: u32,
    _: usize,
) {
    unsupported_void("resource tiling")
}
unsafe extern "C" fn write_sub(
    _: *mut Resource,
    _: u32,
    _: usize,
    _: usize,
    _: u32,
    _: u32,
) -> i32 {
    unavailable("WriteToSubresource")
}
unsafe extern "C" fn read_sub(_: *mut Resource, _: usize, _: u32, _: u32, _: u32, _: usize) -> i32 {
    unavailable("ReadFromSubresource")
}
type DescObject =
    unsafe extern "C" fn(*mut Device, *const c_void, *const Guid, *mut *mut c_void) -> i32;
#[repr(C)]
struct DeviceVtable {
    query: unsafe extern "C" fn(*mut Device, *const Guid, *mut *mut c_void) -> i32,
    add: unsafe extern "C" fn(*mut Device) -> u32,
    release: unsafe extern "C" fn(*mut Device) -> u32,
    get: unsafe extern "C" fn(*mut Device, *const Guid, *mut u32, *mut c_void) -> i32,
    set: unsafe extern "C" fn(*mut Device, *const Guid, u32, *const c_void) -> i32,
    interface: unsafe extern "C" fn(*mut Device, *const Guid, *mut c_void) -> i32,
    name: unsafe extern "C" fn(*mut Device, *const u32) -> i32,
    nodes: unsafe extern "C" fn(*mut Device) -> u32,
    queue: DescObject,
    allocator: unsafe extern "C" fn(*mut Device, i32, *const Guid, *mut *mut c_void) -> i32,
    graphics: DescObject,
    compute: DescObject,
    list: unsafe extern "C" fn(
        *mut Device,
        u32,
        i32,
        usize,
        usize,
        *const Guid,
        *mut *mut c_void,
    ) -> i32,
    feature: unsafe extern "C" fn(*mut Device, i32, *mut c_void, u32) -> i32,
    heap: DescObject,
    increment: unsafe extern "C" fn(*mut Device, i32) -> u32,
    root:
        unsafe extern "C" fn(*mut Device, u32, usize, usize, *const Guid, *mut *mut c_void) -> i32,
    cbv: unsafe extern "C" fn(*mut Device, usize, usize),
    srv: unsafe extern "C" fn(*mut Device, usize, usize, usize),
    uav: unsafe extern "C" fn(*mut Device, usize, usize, usize, usize),
    rtv: unsafe extern "C" fn(*mut Device, usize, usize, usize),
    dsv: unsafe extern "C" fn(*mut Device, usize, usize, usize),
    sampler: unsafe extern "C" fn(*mut Device, usize, usize),
    copy: unsafe extern "C" fn(*mut Device, u32, usize, usize, u32, usize, usize, i32),
    copy_simple: unsafe extern "C" fn(*mut Device, u32, usize, usize, i32),
    allocation: unsafe extern "C" fn(*mut Device, u32, u32, usize) -> AllocationInfo,
    custom: unsafe extern "C" fn(*mut Device, u32, i32) -> HeapProperties,
    committed: unsafe extern "C" fn(
        *mut Device,
        *const HeapProperties,
        u32,
        *const ResourceDesc,
        u32,
        *const c_void,
        *const Guid,
        *mut *mut c_void,
    ) -> i32,
    create_heap: DescObject,
    placed: unsafe extern "C" fn(
        *mut Device,
        usize,
        u64,
        usize,
        u32,
        usize,
        *const Guid,
        *mut *mut c_void,
    ) -> i32,
    reserved:
        unsafe extern "C" fn(*mut Device, usize, u32, usize, *const Guid, *mut *mut c_void) -> i32,
    shared: unsafe extern "C" fn(*mut Device, usize, usize, u32, usize, *mut usize) -> i32,
    open: unsafe extern "C" fn(*mut Device, usize, *const Guid, *mut *mut c_void) -> i32,
    open_name: unsafe extern "C" fn(*mut Device, usize, u32, *mut usize) -> i32,
    resident: unsafe extern "C" fn(*mut Device, u32, usize) -> i32,
    evict: unsafe extern "C" fn(*mut Device, u32, usize) -> i32,
    fence: unsafe extern "C" fn(*mut Device, u64, u32, *const Guid, *mut *mut c_void) -> i32,
    removed: unsafe extern "C" fn(*mut Device) -> i32,
    footprints: unsafe extern "C" fn(*mut Device, usize, u32, u32, u64, usize, usize, usize, usize),
    query_heap: DescObject,
    stable: unsafe extern "C" fn(*mut Device, i32) -> i32,
    signature:
        unsafe extern "C" fn(*mut Device, usize, usize, *const Guid, *mut *mut c_void) -> i32,
    tiling: unsafe extern "C" fn(*mut Device, usize, usize, usize, usize, usize, u32, usize),
    luid: unsafe extern "C" fn(*mut Device) -> Luid,
}
static DEVICE_VTABLE: DeviceVtable = DeviceVtable {
    query: device_query,
    add: device_add,
    release: device_release,
    get: device_get,
    set: device_set,
    interface: device_interface,
    name: device_name,
    nodes,
    queue: queue::create_queue,
    allocator: commands::create_allocator,
    graphics: desc_object,
    compute: desc_object,
    list: commands::create_list,
    feature,
    heap: desc_object,
    increment: descriptor_increment,
    root: root_signature,
    cbv: view2,
    srv: view3,
    uav: view4,
    rtv: view3,
    dsv: view3,
    sampler: view2,
    copy: copy_descriptors,
    copy_simple,
    allocation: allocation_info,
    custom: custom_heap,
    committed,
    create_heap: desc_object,
    placed,
    reserved,
    shared,
    open,
    open_name,
    resident,
    evict: resident,
    fence: queue::create_fence,
    removed,
    footprints,
    query_heap: desc_object,
    stable,
    signature,
    tiling,
    luid: adapter_luid,
};
#[repr(C)]
struct ResourceVtable {
    query: unsafe extern "C" fn(*mut Resource, *const Guid, *mut *mut c_void) -> i32,
    add: unsafe extern "C" fn(*mut Resource) -> u32,
    release: unsafe extern "C" fn(*mut Resource) -> u32,
    get: unsafe extern "C" fn(*mut Resource, *const Guid, *mut u32, *mut c_void) -> i32,
    set: unsafe extern "C" fn(*mut Resource, *const Guid, u32, *const c_void) -> i32,
    interface: unsafe extern "C" fn(*mut Resource, *const Guid, *mut c_void) -> i32,
    name: unsafe extern "C" fn(*mut Resource, *const u32) -> i32,
    device: unsafe extern "C" fn(*mut Resource, *const Guid, *mut *mut c_void) -> i32,
    map: unsafe extern "C" fn(*mut Resource, u32, *const Range, *mut *mut c_void) -> i32,
    unmap: unsafe extern "C" fn(*mut Resource, u32, *const Range),
    desc: unsafe extern "C" fn(*mut Resource) -> ResourceDesc,
    gpu: unsafe extern "C" fn(*mut Resource) -> u64,
    write: unsafe extern "C" fn(*mut Resource, u32, usize, usize, u32, u32) -> i32,
    read: unsafe extern "C" fn(*mut Resource, usize, u32, u32, u32, usize) -> i32,
    heap: unsafe extern "C" fn(*mut Resource, *mut HeapProperties, *mut u32) -> i32,
}
static RESOURCE_VTABLE: ResourceVtable = ResourceVtable {
    query: resource_query,
    add: resource_add,
    release: resource_release,
    get: resource_get,
    set: resource_set,
    interface: resource_interface,
    name: resource_name,
    device: resource_device,
    map,
    unmap,
    desc: resource_desc,
    gpu: gpu_address,
    write: write_sub,
    read: read_sub,
    heap: heap_properties,
};
/// Explicit partial device for resource/ABI work. This does not implement or
/// claim the feature-level contract of the still-unavailable D3D12CreateDevice.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn AlkyholCreateMetalDeviceSubset(
    adapter: *mut c_void,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    if iid.is_null() {
        return E_POINTER;
    }
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, DEVICE].contains(unsafe { &*iid }) {
        return E_NOINTERFACE;
    }
    autoreleasepool(|| {
        let metal = if adapter.is_null() {
            match metal::Device::system_default() {
                Some(d) => d,
                None => return UNSUPPORTED,
            }
        } else {
            // A caller-supplied interface must be one of this runtime's adapter objects.
            let pointer = adapter.cast::<Adapter>();
            if unsafe { (*pointer).vtable } != &ADAPTER_VTABLE {
                return E_INVALIDARG;
            }
            unsafe { (&(*(*pointer).parent).devices)[(*pointer).index].clone() }
        };
        let object = Box::new(Device {
            vtable: &DEVICE_VTABLE,
            refs: AtomicU32::new(1),
            metadata: Metadata::new(),
            metal,
            byte_copy: byte_copy::Kernel::default(),
        });
        unsafe {
            *out = Box::into_raw(object).cast();
        }
        S_OK
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn heap() -> HeapProperties {
        HeapProperties {
            kind: 2,
            page: 0,
            pool: 0,
            creation: 1,
            visible: 1,
        }
    }
    fn desc() -> ResourceDesc {
        ResourceDesc {
            dimension: 1,
            alignment: 0,
            width: 512,
            height: 1,
            depth: 1,
            mips: 1,
            format: 0,
            samples: 1,
            quality: 0,
            layout: 1,
            flags: 0,
        }
    }
    unsafe fn make() -> *mut Device {
        let mut out = ptr::null_mut();
        assert_eq!(
            unsafe { AlkyholCreateMetalDeviceSubset(ptr::null_mut(), &DEVICE, &mut out) },
            S_OK
        );
        out.cast()
    }
    #[test]
    fn subset_constructor_rejects_null_outputs_and_non_runtime_adapter_objects() {
        unsafe {
            assert_eq!(
                AlkyholCreateMetalDeviceSubset(ptr::null_mut(), &DEVICE, ptr::null_mut()),
                E_POINTER
            );
            let mut meta = ptr::null_mut();
            assert_eq!(AlkyholCreateMetadataObject32(&mut meta), S_OK);
            let mut out = ptr::dangling_mut();
            assert_eq!(
                AlkyholCreateMetalDeviceSubset(meta.cast(), &DEVICE, &mut out),
                E_INVALIDARG
            );
            assert!(out.is_null());
            ((*(*meta).vtable).release)(meta);
        }
    }
    #[test]
    fn layouts_preserve_sdk_slots_and_large_struct_returns() {
        assert_eq!(
            std::mem::size_of::<DeviceVtable>(),
            44 * std::mem::size_of::<usize>()
        );
        assert_eq!(
            std::mem::size_of::<ResourceVtable>(),
            15 * std::mem::size_of::<usize>()
        );
        assert_eq!(std::mem::size_of::<ResourceDesc>(), 56);
        assert_eq!(std::mem::size_of::<HeapProperties>(), 20);
        assert_eq!(std::mem::offset_of!(ResourceDesc, width), 16);
    }
    #[test]
    fn descriptor_validation_rejects_unimplemented_and_invalid_cases_without_allocating() {
        let h = heap();
        let d = desc();
        assert_eq!(valid_buffer(&h, 0, &d, 0xac3, ptr::null()), Ok(()));
        let mut bad = d;
        bad.width = 0;
        assert_eq!(
            valid_buffer(&h, 0, &bad, 0xac3, ptr::null()),
            Err(E_INVALIDARG)
        );
        bad = d;
        bad.dimension = 3;
        assert_eq!(
            valid_buffer(&h, 0, &bad, 0xac3, ptr::null()),
            Err(E_NOTIMPL)
        );
        let mut other = h;
        other.kind = 1;
        assert_eq!(valid_buffer(&other, 0, &d, 0, ptr::null()), Ok(()));
        assert_eq!(valid_buffer(&other, 0, &d, 0x400, ptr::null()), Ok(()));
        assert_eq!(valid_buffer(&other, 0, &d, 8, ptr::null()), Err(E_NOTIMPL));
        assert_eq!(
            valid_buffer(&h, 0, &d, 0x400, ptr::null()),
            Err(E_INVALIDARG)
        );
        other = h;
        other.visible = 2;
        assert_eq!(
            valid_buffer(&other, 0, &d, 0xac3, ptr::null()),
            Err(E_NOTIMPL)
        );
    }
    #[test]
    fn interface_identity_capability_queries_and_failure_outputs() {
        autoreleasepool(|| unsafe {
            let mut out = ptr::dangling_mut();
            assert_eq!(
                AlkyholCreateMetalDeviceSubset(ptr::null_mut(), ptr::null(), &mut out),
                E_POINTER
            );
            assert!(out.is_null());
            assert_eq!(
                AlkyholCreateMetalDeviceSubset(ptr::null_mut(), &RESOURCE, &mut out),
                E_NOINTERFACE
            );
            assert!(out.is_null());
            let device = make();
            for iid in [DEVICE, IID_IUNKNOWN, IID_ID3D12OBJECT] {
                assert_eq!(device_query(device, &iid, &mut out), S_OK);
                assert_eq!(out, device.cast());
                assert_eq!(device_release(out.cast()), 1);
            }
            let mut options = [0xa5u8; 12];
            assert_eq!(
                feature(device, 27, options.as_mut_ptr().cast(), 11),
                E_INVALIDARG
            );
            assert_eq!(options, [0xa5; 12]);
            assert_eq!(feature(device, 27, options.as_mut_ptr().cast(), 12), S_OK);
            assert_eq!(options, [0; 12]);
            assert_eq!(
                committed(
                    device,
                    &heap(),
                    0,
                    &desc(),
                    0xac3,
                    ptr::null(),
                    &RESOURCE,
                    ptr::null_mut()
                ),
                1
            );
            assert_eq!(
                committed(
                    device,
                    &heap(),
                    0,
                    &desc(),
                    0xac3,
                    ptr::null(),
                    &DEVICE,
                    &mut out
                ),
                E_NOINTERFACE
            );
            assert!(out.is_null());
            assert_eq!(device_release(device), 0);
        });
    }
    #[test]
    fn mapped_buffer_ranges_nested_maps_and_parent_ownership() {
        autoreleasepool(|| unsafe {
            let device = make();
            let mut out = ptr::null_mut();
            assert_eq!(
                committed(
                    device,
                    &heap(),
                    0,
                    &desc(),
                    0xac3,
                    ptr::null(),
                    &RESOURCE,
                    &mut out
                ),
                S_OK
            );
            let resource = out.cast::<Resource>();
            for iid in [IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, PAGEABLE, RESOURCE] {
                let mut same = ptr::null_mut();
                assert_eq!(resource_query(resource, &iid, &mut same), S_OK);
                assert_eq!(same, resource.cast());
                assert_eq!(resource_release(same.cast()), 1);
            }
            let mut mapped = ptr::dangling_mut();
            assert_eq!(map(resource, 1, ptr::null(), &mut mapped), E_INVALIDARG);
            assert!(mapped.is_null());
            assert_eq!(
                map(resource, 0, &Range { begin: 0, end: 513 }, &mut mapped),
                E_INVALIDARG
            );
            assert!(mapped.is_null());
            assert_eq!(map(resource, 0, ptr::null(), &mut mapped), S_OK);
            let mut again = ptr::null_mut();
            assert_eq!(map(resource, 0, ptr::null(), &mut again), S_OK);
            assert_eq!(mapped, again);
            ptr::write_bytes(mapped.cast::<u8>(), 0x83, 512);
            unmap(resource, 0, ptr::null());
            assert_eq!(*mapped.cast::<u8>().add(511), 0x83);
            let mut expected = desc();
            expected.alignment = 65536;
            assert_eq!(resource_desc(resource), expected);
            assert_eq!(gpu_address(resource) & 65535, 0);
            assert_ne!(gpu_address(resource), 0);
            assert_eq!(device_release(device), 1);
            let mut parent = ptr::null_mut();
            assert_eq!(resource_device(resource, &DEVICE, &mut parent), S_OK);
            assert_eq!(parent, device.cast());
            device_release(parent.cast());
            assert_eq!(resource_release(resource), 0);
        });
    }
    #[test]
    fn actual_metal_blit_writes_the_same_allocated_com_resource() {
        autoreleasepool(|| unsafe {
            let device = make();
            let mut out = ptr::null_mut();
            assert_eq!(
                committed(
                    device,
                    &heap(),
                    0,
                    &desc(),
                    0xac3,
                    ptr::null(),
                    &RESOURCE,
                    &mut out
                ),
                S_OK
            );
            let resource = out.cast::<Resource>();
            let queue = (*device).metal.new_command_queue();
            let command = queue.new_command_buffer();
            let blit = command.new_blit_command_encoder();
            blit.fill_buffer(
                &(*resource).buffer,
                metal::NSRange::new((*resource).offset as u64, 512),
                0x5a,
            );
            blit.end_encoding();
            command.commit();
            command.wait_until_completed();
            assert_eq!(command.status(), metal::MTLCommandBufferStatus::Completed);
            let mut mapped = ptr::null_mut();
            assert_eq!(map(resource, 0, ptr::null(), &mut mapped), S_OK);
            assert!(
                std::slice::from_raw_parts(mapped.cast::<u8>(), 512)
                    .iter()
                    .all(|b| *b == 0x5a)
            );
            resource_release(resource);
            device_release(device);
        });
    }
}

#[path = "queue.rs"]
mod queue;

#[path = "commands.rs"]
mod commands;

#[path = "byte_copy.rs"]
mod byte_copy;
