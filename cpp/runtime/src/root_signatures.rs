//! Owned, validated CPU root layouts. Pipeline/binding commands stay unavailable.
use super::*;
use alkyhol::root_signature::{Layout, parse};
const ROOT: Guid = guid!(
    0xc54a6b66, 0x72df, 0x4ee8, 0x8b, 0xe5, 0xa9, 0x46, 0xa1, 0x42, 0x92, 0x14
);
#[repr(C)]
struct Root {
    vtable: *const Vtable,
    refs: AtomicU32,
    metadata: Metadata,
    _layout: Layout,
    identity: Arc<()>,
    parent: Parent,
}
unsafe extern "C" fn query(this: *mut Root, iid: *const Guid, out: *mut *mut c_void) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    if iid.is_null() {
        return E_POINTER;
    }
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, ROOT].contains(unsafe { &*iid }) {
        return E_NOINTERFACE;
    }
    unsafe {
        add(this);
        *out = this.cast();
    }
    S_OK
}
unsafe extern "C" fn add(this: *mut Root) -> u32 {
    super::increment(unsafe { &(*this).refs })
}
unsafe extern "C" fn release(this: *mut Root) -> u32 {
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
metadata_methods!(Root, set, interface, get);
unsafe extern "C" fn name(this: *mut Root, text: *const u32) -> i32 {
    let m = unsafe { (*this).metadata.0 };
    unsafe { ((*(*m).vtable).set_name)(m, text) }
}
unsafe extern "C" fn device(this: *mut Root, iid: *const Guid, out: *mut *mut c_void) -> i32 {
    unsafe { device_query((*this).parent.0, iid, out) }
}
#[repr(C)]
struct Vtable {
    query: unsafe extern "C" fn(*mut Root, *const Guid, *mut *mut c_void) -> i32,
    add: unsafe extern "C" fn(*mut Root) -> u32,
    release: unsafe extern "C" fn(*mut Root) -> u32,
    get: unsafe extern "C" fn(*mut Root, *const Guid, *mut u32, *mut c_void) -> i32,
    set: unsafe extern "C" fn(*mut Root, *const Guid, u32, *const c_void) -> i32,
    interface: unsafe extern "C" fn(*mut Root, *const Guid, *mut c_void) -> i32,
    name: unsafe extern "C" fn(*mut Root, *const u32) -> i32,
    device: unsafe extern "C" fn(*mut Root, *const Guid, *mut *mut c_void) -> i32,
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
};
pub(super) unsafe fn snapshot(parent: *mut Device, object: usize) -> Result<Layout, i32> {
    if object == 0 {
        return Err(E_NOTIMPL);
    }
    let root = object as *mut Root;
    if unsafe { (*root).vtable } != &VTABLE || unsafe { (*root).parent.0 } != parent {
        return Err(E_INVALIDARG);
    }
    Ok(unsafe { (*root)._layout.clone() })
}
pub(super) unsafe fn binding_snapshot(
    parent: *mut Device,
    object: usize,
) -> Result<(Layout, Arc<()>), i32> {
    let layout = unsafe { snapshot(parent, object) }?;
    let root = object as *mut Root;
    Ok((layout, unsafe { Arc::clone(&(*root).identity) }))
}
pub(super) unsafe extern "C" fn create(
    this: *mut Device,
    node: u32,
    data: usize,
    size: usize,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if !out.is_null() {
        unsafe {
            *out = ptr::null_mut();
        }
    }
    if iid.is_null() || data == 0 {
        return E_POINTER;
    }
    if node > 1 {
        return E_NOTIMPL;
    }
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, ROOT].contains(unsafe { &*iid }) {
        return E_NOINTERFACE;
    }
    if size == 0 || size > 1_048_576 {
        return E_INVALIDARG;
    }
    let bytes = unsafe { std::slice::from_raw_parts(data as *const u8, size) };
    let layout = match parse(bytes) {
        Ok(layout) => layout,
        Err(error) => return error.code,
    };
    if out.is_null() {
        return 1;
    }
    unsafe {
        device_add(this);
        *out = Box::into_raw(Box::new(Root {
            vtable: &VTABLE,
            refs: AtomicU32::new(1),
            metadata: Metadata::new(),
            _layout: layout,
            identity: Arc::new(()),
            parent: Parent(this),
        }))
        .cast();
    }
    S_OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use alkyhol::root_signature::{Binding, Parameter, serialize};
    unsafe fn make() -> *mut Device {
        let mut out = ptr::null_mut();
        assert_eq!(
            unsafe { AlkyholCreateMetalDeviceSubset(ptr::null_mut(), &DEVICE, &mut out) },
            S_OK
        );
        out.cast()
    }
    fn layout() -> Layout {
        Layout {
            parameters: vec![Parameter {
                visibility: 0,
                binding: Binding::Table(vec![[2, 3, 0, 2, u32::MAX]]),
            }],
            samplers: vec![],
            flags: 0,
        }
    }
    #[test]
    fn source_abi_layout_is_owned_and_retains_parent_identity_and_metadata() {
        autoreleasepool(|| unsafe {
            assert_eq!(
                std::mem::size_of::<Vtable>(),
                8 * std::mem::size_of::<usize>()
            );
            let d = make();
            let original = layout();
            let mut bytes = serialize(&original).unwrap();
            let mut out = ptr::null_mut();
            assert_eq!(
                create(d, 0, bytes.as_ptr() as usize, bytes.len(), &ROOT, &mut out),
                S_OK
            );
            let root = out.cast::<Root>();
            bytes.fill(0);
            assert_eq!((*root)._layout, original);
            assert_eq!(device_release(d), 1);
            for iid in [IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, ROOT] {
                assert_eq!(query(root, &iid, &mut out), S_OK);
                assert_eq!(out, root.cast());
                assert_eq!(release(out.cast()), 1);
            }
            assert_eq!(query(root, &PAGEABLE, &mut out), E_NOINTERFACE);
            assert!(out.is_null());
            assert_eq!(device(root, &DEVICE, &mut out), S_OK);
            assert_eq!(out, d.cast());
            assert_eq!(device_release(out.cast()), 1);
            let tag = guid!(19, 7, 3, 1, 2, 3, 4, 5, 6, 7, 8);
            let input = [0x17u8, 0x93, 0xff];
            assert_eq!(set(root, &tag, 3, input.as_ptr().cast()), S_OK);
            let mut actual = [0; 3];
            let mut size = 3;
            assert_eq!(get(root, &tag, &mut size, actual.as_mut_ptr().cast()), S_OK);
            assert_eq!(actual, input);
            assert_eq!(name(root, [b'r' as u32, 0x1f338, 0].as_ptr()), S_OK);
            assert_eq!(release(root), 0);
        });
    }
    #[test]
    fn invalid_or_unavailable_creation_clears_outputs_and_never_retains_device() {
        autoreleasepool(|| unsafe {
            let d = make();
            let bytes = serialize(&layout()).unwrap();
            let mut out = ptr::dangling_mut();
            for (node, data, size, iid, error) in [
                (2, bytes.as_ptr() as usize, bytes.len(), &ROOT, E_NOTIMPL),
                (0, 0, bytes.len(), &ROOT, E_POINTER),
                (0, bytes.as_ptr() as usize, 0, &ROOT, E_INVALIDARG),
                (0, bytes.as_ptr() as usize, 1_048_577, &ROOT, E_INVALIDARG),
                (
                    0,
                    bytes.as_ptr() as usize,
                    bytes.len(),
                    &DEVICE,
                    E_NOINTERFACE,
                ),
            ] {
                assert_eq!(create(d, node, data, size, iid, &mut out), error);
                assert!(out.is_null());
            }
            let mut corrupt = bytes.clone();
            corrupt[4] ^= 1;
            assert_eq!(
                create(
                    d,
                    1,
                    corrupt.as_ptr() as usize,
                    corrupt.len(),
                    &ROOT,
                    &mut out
                ),
                E_INVALIDARG
            );
            assert!(out.is_null());
            assert_eq!(
                create(
                    d,
                    1,
                    bytes.as_ptr() as usize,
                    bytes.len(),
                    &ROOT,
                    ptr::null_mut()
                ),
                1
            );
            assert_eq!(device_release(d), 0);
        });
    }

    #[test]
    fn root_interface_ownership_is_atomic_with_a_live_coordinator_reference() {
        autoreleasepool(|| unsafe {
            let d = make();
            let bytes = serialize(&layout()).unwrap();
            let mut out = ptr::null_mut();
            assert_eq!(
                create(d, 0, bytes.as_ptr() as usize, bytes.len(), &ROOT, &mut out),
                S_OK
            );
            let address = out as usize;
            let workers: Vec<_> = (0..8)
                .map(|_| {
                    std::thread::spawn(move || {
                        let root = address as *mut Root;
                        for _ in 0..1000 {
                            add(root);
                            release(root);
                        }
                    })
                })
                .collect();
            for worker in workers {
                worker.join().unwrap();
            }
            assert_eq!(release(out.cast()), 0);
            assert_eq!(device_release(d), 0);
        });
    }
}
