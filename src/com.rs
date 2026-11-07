//! Native-host COM ABI foundation. C++ virtual calls enter Rust vtables directly.
//! This is source ABI support; it does not implement Windows binary loading.
use std::{
    ffi::c_void,
    ptr,
    sync::atomic::{AtomicU32, Ordering, fence},
};

pub const S_OK: i32 = 0;
pub const E_NOINTERFACE: i32 = 0x80004002u32 as i32;
pub const E_POINTER: i32 = 0x80004003u32 as i32;
pub const E_OUTOFMEMORY: i32 = 0x8007000eu32 as i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}
pub const IID_IUNKNOWN: Guid = Guid {
    data1: 0,
    data2: 0,
    data3: 0,
    data4: [0xc0, 0, 0, 0, 0, 0, 0, 0x46],
};
pub const IID_ID3DBLOB: Guid = Guid {
    data1: 0x8ba5fb08,
    data2: 0x5195,
    data3: 0x40e2,
    data4: [0xac, 0x58, 0x0d, 0x98, 0x9c, 0x3a, 1, 2],
};
#[repr(C)]
pub struct BlobVtable {
    pub query_interface: unsafe extern "C" fn(*mut Blob, *const Guid, *mut *mut c_void) -> i32,
    pub add_ref: unsafe extern "C" fn(*mut Blob) -> u32,
    pub release: unsafe extern "C" fn(*mut Blob) -> u32,
    pub get_buffer_pointer: unsafe extern "C" fn(*mut Blob) -> *mut c_void,
    pub get_buffer_size: unsafe extern "C" fn(*mut Blob) -> usize,
}
#[repr(C)]
pub struct Blob {
    pub vtable: *const BlobVtable,
    refs: AtomicU32,
    data: *mut u8,
    size: usize,
}
impl Drop for Blob {
    fn drop(&mut self) {
        // The final Release owns the allocation; no live interface may still use it.
        unsafe {
            drop(Box::from_raw(ptr::slice_from_raw_parts_mut(
                self.data, self.size,
            )));
        }
    }
}
unsafe extern "C" fn query_interface(
    this: *mut Blob,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = ptr::null_mut();
    }
    if iid.is_null() {
        return E_POINTER;
    }
    if unsafe { *iid } != IID_IUNKNOWN && unsafe { *iid } != IID_ID3DBLOB {
        return E_NOINTERFACE;
    }
    unsafe {
        add_ref(this);
        *out = this.cast();
    }
    S_OK
}
unsafe extern "C" fn add_ref(this: *mut Blob) -> u32 {
    let refs = unsafe { &(*this).refs };
    match refs.try_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1)) {
        Ok(previous) => previous + 1,
        Err(_) => std::process::abort(),
    }
}
unsafe extern "C" fn release(this: *mut Blob) -> u32 {
    let previous = unsafe { (*this).refs.fetch_sub(1, Ordering::Release) };
    if previous == 1 {
        fence(Ordering::Acquire);
        unsafe {
            drop(Box::from_raw(this));
        }
        return 0;
    }
    previous - 1
}
unsafe extern "C" fn get_buffer_pointer(this: *mut Blob) -> *mut c_void {
    unsafe { (*this).data.cast() }
}
unsafe extern "C" fn get_buffer_size(this: *mut Blob) -> usize {
    unsafe { (*this).size }
}
static BLOB_VTABLE: BlobVtable = BlobVtable {
    query_interface,
    add_ref,
    release,
    get_buffer_pointer,
    get_buffer_size,
};
/// Allocate an owned blob with real COM identity and atomic reference ownership.
///
/// # Safety
/// `out` must be null or point to writable storage for one interface pointer.
/// Calls through the returned vtable require a live reference. Buffer writes
/// and reads require caller synchronization; no pointer remains valid after the
/// final Release. Native Windows edge-case return codes remain separate evidence.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn D3DCreateBlob(size: usize, out: *mut *mut Blob) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = ptr::null_mut();
    }
    let mut data = Vec::new();
    if data.try_reserve_exact(size).is_err() {
        return E_OUTOFMEMORY;
    }
    data.resize(size, 0);
    let blob = Box::new(Blob {
        vtable: &BLOB_VTABLE,
        refs: AtomicU32::new(1),
        data: Box::into_raw(data.into_boxed_slice()).cast::<u8>(),
        size,
    });
    unsafe {
        *out = Box::into_raw(blob);
    }
    S_OK
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blob_identity_buffer_and_balanced_ownership() {
        unsafe {
            assert_eq!(std::mem::size_of::<Guid>(), 16);
            assert_eq!(std::mem::offset_of!(Blob, vtable), 0);
            let mut blob = ptr::null_mut();
            assert_eq!(D3DCreateBlob(37, &mut blob), S_OK);
            let v = &*(*blob).vtable;
            assert_eq!((v.get_buffer_size)(blob), 37);
            let bytes =
                std::slice::from_raw_parts_mut((v.get_buffer_pointer)(blob).cast::<u8>(), 37);
            for (i, b) in bytes.iter_mut().enumerate() {
                *b = (i * 7) as u8;
            }
            for iid in [IID_IUNKNOWN, IID_ID3DBLOB] {
                let mut queried = ptr::null_mut();
                assert_eq!((v.query_interface)(blob, &iid, &mut queried), S_OK);
                assert_eq!(queried, blob.cast());
                assert_eq!((v.release)(queried.cast()), 1);
            }
            assert_eq!(bytes[36], 252);
            assert_eq!((v.add_ref)(blob), 2);
            assert_eq!((v.release)(blob), 1);
            assert_eq!((v.release)(blob), 0);
        }
    }
    #[test]
    fn failures_clear_outputs_and_do_not_acquire_references() {
        unsafe {
            assert_eq!(D3DCreateBlob(16, ptr::null_mut()), E_POINTER);
            let mut blob = ptr::dangling_mut();
            assert_eq!(D3DCreateBlob(usize::MAX, &mut blob), E_OUTOFMEMORY);
            assert!(blob.is_null());
            assert_eq!(D3DCreateBlob(0, &mut blob), S_OK);
            let v = &*(*blob).vtable;
            assert_eq!((v.get_buffer_size)(blob), 0);
            let mut out = ptr::dangling_mut();
            let unknown = Guid {
                data1: 7,
                ..IID_IUNKNOWN
            };
            assert_eq!((v.query_interface)(blob, &unknown, &mut out), E_NOINTERFACE);
            assert!(out.is_null());
            assert_eq!((v.query_interface)(blob, ptr::null(), &mut out), E_POINTER);
            assert_eq!(
                (v.query_interface)(blob, &IID_IUNKNOWN, ptr::null_mut()),
                E_POINTER
            );
            assert_eq!((v.release)(blob), 0);
        }
    }
    #[test]
    fn shared_interface_ownership_is_atomic() {
        unsafe {
            let mut blob = ptr::null_mut();
            assert_eq!(D3DCreateBlob(8, &mut blob), S_OK);
            let address = blob as usize;
            let threads: Vec<_> = (0..8)
                .map(|_| {
                    std::thread::spawn(move || {
                        let b = address as *mut Blob;
                        let v = &*(*b).vtable;
                        for _ in 0..2000 {
                            (v.add_ref)(b);
                            (v.release)(b);
                        }
                    })
                })
                .collect();
            for t in threads {
                t.join().unwrap();
            }
            assert_eq!(((*(*blob).vtable).release)(blob), 0);
        }
    }
}

#[derive(Clone, Copy)]
struct Apartment {
    model: i32,
    depth: u32,
}
thread_local! { static APARTMENT: std::cell::RefCell<Option<Apartment>> = const { std::cell::RefCell::new(None) }; }
/// Track the current thread's supported COM concurrency model and nesting.
/// Object activation, proxy/stub marshaling and WinRT classes are separate APIs.
#[unsafe(no_mangle)]
pub extern "C" fn RoInitialize(model: i32) -> i32 {
    if model != 0 && model != 1 {
        return 0x80070057u32 as i32;
    }
    APARTMENT.with(|slot| {
        let mut state = slot.borrow_mut();
        match *state {
            None => {
                *state = Some(Apartment { model, depth: 1 });
                S_OK
            }
            Some(current) if current.model != model => 0x80010106u32 as i32,
            Some(current) => match current.depth.checked_add(1) {
                Some(depth) => {
                    *state = Some(Apartment { model, depth });
                    1
                }
                None => E_OUTOFMEMORY,
            },
        }
    })
}
/// Release one successful RoInitialize on the current thread.
#[unsafe(no_mangle)]
pub extern "C" fn RoUninitialize() {
    APARTMENT.with(|slot| {
        let mut state = slot.borrow_mut();
        if let Some(current) = *state {
            *state = if current.depth == 1 {
                None
            } else {
                Some(Apartment {
                    model: current.model,
                    depth: current.depth - 1,
                })
            };
        }
    });
}
#[cfg(test)]
mod apartment_tests {
    use super::*;
    #[test]
    fn apartment_modes_nesting_and_failed_changes_preserve_the_original_mode() {
        std::thread::spawn(|| {
            assert_eq!(RoInitialize(7), 0x80070057u32 as i32);
            assert_eq!(RoInitialize(0), S_OK);
            assert_eq!(RoInitialize(0), 1);
            assert_eq!(RoInitialize(1), 0x80010106u32 as i32);
            RoUninitialize();
            assert_eq!(RoInitialize(1), 0x80010106u32 as i32);
            RoUninitialize();
            assert_eq!(RoInitialize(1), S_OK);
            RoUninitialize();
        })
        .join()
        .unwrap();
    }
    #[test]
    fn concurrency_models_are_thread_local() {
        std::thread::spawn(|| {
            assert_eq!(RoInitialize(1), S_OK);
            std::thread::spawn(|| {
                assert_eq!(RoInitialize(0), S_OK);
                RoUninitialize();
            })
            .join()
            .unwrap();
            assert_eq!(RoInitialize(0), 0x80010106u32 as i32);
            RoUninitialize();
        })
        .join()
        .unwrap();
    }
}
