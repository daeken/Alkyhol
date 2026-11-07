//! Owned descriptor-heap storage and opaque handles. No initialized views/binding.
use super::*;
const HEAP: Guid = guid!(
    0x8efb471d, 0x616c, 0x4f49, 0x90, 0xf7, 0x12, 0x7b, 0xb7, 0x63, 0xfa, 0x51
);
// Virtual descriptor records are 64 bytes. This is our backend stride, not a
// claimed Windows driver descriptor encoding. Uninitialized slots cannot bind.
const STRIDE: u32 = 64;
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Description {
    kind: i32,
    count: u32,
    flags: u32,
    node: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct CpuHandle {
    ptr: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct GpuHandle {
    ptr: u64,
}
#[repr(C)]
struct Heap {
    vtable: *const Vtable,
    refs: AtomicU32,
    metadata: Metadata,
    description: Description,
    storage: metal::Buffer,
    parent: Parent,
}
unsafe extern "C" fn query(this: *mut Heap, iid: *const Guid, out: *mut *mut c_void) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    if iid.is_null() {
        return E_POINTER;
    }
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, PAGEABLE, HEAP].contains(unsafe { &*iid }) {
        return E_NOINTERFACE;
    }
    unsafe {
        add(this);
        *out = this.cast();
    }
    S_OK
}
unsafe extern "C" fn add(this: *mut Heap) -> u32 {
    super::increment(unsafe { &(*this).refs })
}
unsafe extern "C" fn release(this: *mut Heap) -> u32 {
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
metadata_methods!(Heap, set, interface, get);
unsafe extern "C" fn name(this: *mut Heap, name: *const u32) -> i32 {
    let m = unsafe { (*this).metadata.0 };
    unsafe { ((*(*m).vtable).set_name)(m, name) }
}
unsafe extern "C" fn device(this: *mut Heap, iid: *const Guid, out: *mut *mut c_void) -> i32 {
    unsafe { device_query((*this).parent.0, iid, out) }
}
unsafe extern "C" fn description(this: *mut Heap) -> Description {
    unsafe { (*this).description }
}
unsafe extern "C" fn cpu(this: *mut Heap) -> CpuHandle {
    CpuHandle {
        ptr: unsafe { (*this).storage.contents() as usize },
    }
}
unsafe extern "C" fn gpu(this: *mut Heap) -> GpuHandle {
    // Measured native WARP returns nonzero opaque GPU handles for CPU-only
    // heaps too. Visibility is the descriptor flag, not the address bits.
    GpuHandle {
        ptr: unsafe { (*this).storage.gpu_address() },
    }
}
#[repr(C)]
struct Vtable {
    query: unsafe extern "C" fn(*mut Heap, *const Guid, *mut *mut c_void) -> i32,
    add: unsafe extern "C" fn(*mut Heap) -> u32,
    release: unsafe extern "C" fn(*mut Heap) -> u32,
    get: unsafe extern "C" fn(*mut Heap, *const Guid, *mut u32, *mut c_void) -> i32,
    set: unsafe extern "C" fn(*mut Heap, *const Guid, u32, *const c_void) -> i32,
    interface: unsafe extern "C" fn(*mut Heap, *const Guid, *mut c_void) -> i32,
    name: unsafe extern "C" fn(*mut Heap, *const u32) -> i32,
    device: unsafe extern "C" fn(*mut Heap, *const Guid, *mut *mut c_void) -> i32,
    description: unsafe extern "C" fn(*mut Heap) -> Description,
    cpu: unsafe extern "C" fn(*mut Heap) -> CpuHandle,
    gpu: unsafe extern "C" fn(*mut Heap) -> GpuHandle,
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
    description,
    cpu,
    gpu,
};
fn validate(desc: Description) -> Result<u64, i32> {
    if !(0..4).contains(&desc.kind) {
        return Err(E_INVALIDARG);
    }
    if desc.count == 0 {
        return Err(E_INVALIDARG);
    }
    if desc.flags > 1 || desc.node > 1 {
        return Err(E_NOTIMPL);
    }
    if desc.flags == 1 && desc.kind >= 2 {
        return Err(E_INVALIDARG);
    }
    let limit = if desc.kind == 1 { 2048 } else { 1_000_000 };
    if desc.count > limit {
        return Err(E_NOTIMPL);
    }
    Ok(u64::from(desc.count) * u64::from(STRIDE))
}
pub(super) unsafe extern "C" fn create(
    this: *mut Device,
    desc: *const c_void,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if !out.is_null() {
        unsafe {
            *out = ptr::null_mut();
        }
    }
    if desc.is_null() || iid.is_null() {
        return E_POINTER;
    }
    let mut desc = unsafe { *desc.cast::<Description>() };
    let length = match validate(desc) {
        Ok(n) => n,
        Err(e) => return e,
    };
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, PAGEABLE, HEAP].contains(unsafe { &*iid }) {
        return E_NOINTERFACE;
    }
    if length > unsafe { (*this).metal.max_buffer_length() } {
        return E_OUTOFMEMORY;
    }
    if out.is_null() {
        return 1;
    }
    desc.node = 1;
    autoreleasepool(|| {
        let storage = unsafe {
            (*this).metal.new_buffer(
                length,
                metal::MTLResourceOptions::StorageModeShared
                    | metal::MTLResourceOptions::HazardTrackingModeTracked,
            )
        };
        if storage.as_ptr().is_null() || storage.contents().is_null() {
            return E_OUTOFMEMORY;
        }
        if storage.gpu_address() == 0 {
            return E_NOTIMPL;
        }
        unsafe {
            device_add(this);
            *out = Box::into_raw(Box::new(Heap {
                vtable: &VTABLE,
                refs: AtomicU32::new(1),
                metadata: Metadata::new(),
                description: desc,
                storage,
                parent: Parent(this),
            }))
            .cast();
        }
        S_OK
    })
}
pub(super) unsafe extern "C" fn increment(_: *mut Device, kind: i32) -> u32 {
    if (0..4).contains(&kind) {
        STRIDE
    } else {
        unavailable("unknown descriptor heap type");
        0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    unsafe fn make() -> *mut Device {
        let mut raw = ptr::null_mut();
        assert_eq!(
            unsafe { AlkyholCreateMetalDeviceSubset(ptr::null_mut(), &DEVICE, &mut raw) },
            S_OK
        );
        raw.cast()
    }
    unsafe fn heap(device: *mut Device, desc: Description) -> *mut Heap {
        let mut raw = ptr::null_mut();
        assert_eq!(
            unsafe {
                create(
                    device,
                    (&desc as *const Description).cast(),
                    &HEAP,
                    &mut raw,
                )
            },
            S_OK
        );
        raw.cast()
    }
    #[test]
    fn typed_heap_layout_and_bounds_cover_each_type_visibility_and_overflow_limit() {
        assert_eq!(
            std::mem::size_of::<Vtable>(),
            11 * std::mem::size_of::<usize>()
        );
        assert_eq!(std::mem::size_of::<Description>(), 16);
        assert_eq!(std::mem::size_of::<CpuHandle>(), 8);
        assert_eq!(std::mem::size_of::<GpuHandle>(), 8);
        let valid = Description {
            kind: 0,
            count: 3,
            flags: 0,
            node: 0,
        };
        assert_eq!(validate(valid), Ok(192));
        for kind in 0..4 {
            assert_eq!(validate(Description { kind, ..valid }), Ok(192));
        }
        for (desc, error) in [
            (Description { kind: -1, ..valid }, E_INVALIDARG),
            (Description { count: 0, ..valid }, E_INVALIDARG),
            (Description { flags: 2, ..valid }, E_NOTIMPL),
            (Description { node: 2, ..valid }, E_NOTIMPL),
            (
                Description {
                    kind: 2,
                    flags: 1,
                    ..valid
                },
                E_INVALIDARG,
            ),
            (
                Description {
                    count: u32::MAX,
                    ..valid
                },
                E_NOTIMPL,
            ),
            (
                Description {
                    kind: 1,
                    count: 2049,
                    ..valid
                },
                E_NOTIMPL,
            ),
        ] {
            assert_eq!(validate(desc), Err(error));
        }
        assert_eq!(
            validate(Description {
                kind: 1,
                count: 2048,
                flags: 1,
                ..valid
            }),
            Ok(2048 * 64)
        );
    }
    #[test]
    fn heap_constructor_validation_clears_outputs_and_null_output_never_allocates_or_retains() {
        autoreleasepool(|| unsafe {
            let d = make();
            let desc = Description {
                kind: 0,
                count: 4,
                flags: 1,
                node: 0,
            };
            let mut raw = ptr::dangling_mut();
            assert_eq!(
                create(
                    d,
                    (&desc as *const Description).cast(),
                    &HEAP,
                    ptr::null_mut()
                ),
                1
            );
            assert_eq!(create(d, ptr::null(), &HEAP, &mut raw), E_POINTER);
            assert!(raw.is_null());
            assert_eq!(
                create(
                    d,
                    (&desc as *const Description).cast(),
                    ptr::null(),
                    &mut raw
                ),
                E_POINTER
            );
            assert!(raw.is_null());
            assert_eq!(
                create(d, (&desc as *const Description).cast(), &RESOURCE, &mut raw),
                E_NOINTERFACE
            );
            assert!(raw.is_null());
            assert_eq!(increment(d, -1), 0);
            assert_eq!(device_release(d), 0);
        });
    }
    #[test]
    fn actual_heap_storage_handles_identity_metadata_and_parent_ownership_remain_live() {
        autoreleasepool(|| unsafe {
            let d = make();
            let h = heap(
                d,
                Description {
                    kind: 0,
                    count: 5,
                    flags: 1,
                    node: 0,
                },
            );
            assert_eq!(description(h).node, 1);
            let cpu_start = cpu(h).ptr;
            let gpu_start = gpu(h).ptr;
            assert_eq!(cpu_start, (*h).storage.contents() as usize);
            assert_eq!(gpu_start, (*h).storage.gpu_address());
            assert_ne!(cpu_start, 0);
            assert_ne!(gpu_start, 0);
            assert_eq!((*h).storage.length(), 5 * u64::from(increment(d, 0)));
            let mut raw = ptr::null_mut();
            for id in [IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, PAGEABLE, HEAP] {
                assert_eq!(query(h, &id, &mut raw), S_OK);
                assert_eq!(raw, h.cast());
                release(raw.cast());
            }
            assert_eq!(query(h, &RESOURCE, &mut raw), E_NOINTERFACE);
            assert!(raw.is_null());
            assert_eq!(name(h, [0x1f338u32, 0].as_ptr()), S_OK);
            let key = guid!(9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 9);
            let value = [0x31u8, 0x45];
            assert_eq!(set(h, &key, 2, value.as_ptr().cast()), S_OK);
            let mut actual = [0u8; 4];
            let mut count = 4;
            assert_eq!(get(h, &key, &mut count, actual.as_mut_ptr().cast()), S_OK);
            assert_eq!(actual, [0x31, 0x45, 0, 0]);
            assert_eq!(count, 2);
            assert_eq!(device_release(d), 1);
            assert_eq!(device(h, &DEVICE, &mut raw), S_OK);
            assert_eq!(raw, d.cast());
            assert_eq!(device_release(raw.cast()), 1);
            assert_eq!(cpu(h).ptr, cpu_start);
            assert_eq!(gpu(h).ptr, gpu_start);
            assert_eq!(release(h), 0);
        });
    }
    #[test]
    fn cpu_heap_visibility_is_not_inferred_from_opaque_gpu_handle_bits() {
        autoreleasepool(|| unsafe {
            let d = make();
            let mut heaps = Vec::new();
            for kind in 0..4 {
                let h = heap(
                    d,
                    Description {
                        kind,
                        count: 7,
                        flags: 0,
                        node: 1,
                    },
                );
                assert_eq!(description(h).flags, 0);
                assert_ne!(gpu(h).ptr, 0);
                assert_eq!(gpu(h).ptr, (*h).storage.gpu_address());
                assert_eq!(increment(d, kind), 64);
                heaps.push(h);
            }
            for (i, &a) in heaps.iter().enumerate() {
                for &b in &heaps[i + 1..] {
                    let start = cpu(a).ptr;
                    let other = cpu(b).ptr;
                    let length = 7 * 64;
                    assert!(start + length <= other || other + length <= start);
                }
            }
            for h in heaps {
                release(h);
            }
            assert_eq!(device_release(d), 0);
        });
    }
    #[test]
    fn descriptor_backing_storage_is_real_gpu_accessible_without_claiming_initialized_views() {
        autoreleasepool(|| unsafe {
            let d = make();
            let h = heap(
                d,
                Description {
                    kind: 0,
                    count: 3,
                    flags: 1,
                    node: 1,
                },
            );
            let length = 3 * 64;
            let data = std::slice::from_raw_parts_mut(cpu(h).ptr as *mut u8, length);
            for (i, b) in data.iter_mut().enumerate() {
                *b = (i * 13 + 1) as u8;
            }
            let output = (*d)
                .metal
                .new_buffer(length as u64, metal::MTLResourceOptions::StorageModeShared);
            let queue = (*d).metal.new_command_queue();
            let command = queue.new_command_buffer();
            let encoder = command.new_blit_command_encoder();
            encoder.copy_from_buffer(&(*h).storage, 0, &output, 0, length as u64);
            encoder.end_encoding();
            command.commit();
            command.wait_until_completed();
            assert_eq!(command.status(), metal::MTLCommandBufferStatus::Completed);
            let actual = std::slice::from_raw_parts(output.contents().cast::<u8>(), length);
            for (i, &byte) in actual.iter().enumerate() {
                assert_eq!(byte, (i * 13 + 1) as u8);
            }
            release(h);
            assert_eq!(device_release(d), 0);
        });
    }
}
