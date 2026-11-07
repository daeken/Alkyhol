//! Win64 ID3DBlob vtables over the existing native Rust COM blob implementation.
//! A wrapper exclusively owns a fresh native blob. Native interface pointers
//! never escape to guest callers; every guest virtual slot uses Win64 ABI.
#![allow(non_snake_case)]
use alkyhol::com::{self, Blob, E_OUTOFMEMORY, E_POINTER, Guid, S_OK};
use std::{
    alloc::{Layout, alloc},
    ffi::c_void,
    ptr,
};

#[repr(C)]
struct BlobVtable {
    query: unsafe extern "win64" fn(*mut BinaryBlob, *const Guid, *mut *mut c_void) -> i32,
    add_ref: unsafe extern "win64" fn(*mut BinaryBlob) -> u32,
    release: unsafe extern "win64" fn(*mut BinaryBlob) -> u32,
    pointer: unsafe extern "win64" fn(*mut BinaryBlob) -> *mut c_void,
    size: unsafe extern "win64" fn(*mut BinaryBlob) -> usize,
}
#[repr(C)]
struct BinaryBlob {
    vtable: *const BlobVtable,
    native: *mut Blob,
}
unsafe extern "win64" fn query(
    this: *mut BinaryBlob,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = ptr::null_mut();
    }
    let native = unsafe { (*this).native };
    let mut result = ptr::null_mut();
    let hr = unsafe { ((*(*native).vtable).query_interface)(native, iid, &mut result) };
    if hr == S_OK {
        // The underlying blob has exactly IUnknown/ID3DBlob on one identity.
        // Never publish its native-ABI vtable through a Windows interface.
        assert_eq!(result, native.cast(), "native blob identity changed");
        unsafe {
            *out = this.cast();
        }
    }
    hr
}
unsafe extern "win64" fn add_ref(this: *mut BinaryBlob) -> u32 {
    let native = unsafe { (*this).native };
    unsafe { ((*(*native).vtable).add_ref)(native) }
}
unsafe extern "win64" fn release(this: *mut BinaryBlob) -> u32 {
    let native = unsafe { (*this).native };
    let remaining = unsafe { ((*(*native).vtable).release)(native) };
    if remaining == 0 {
        // Native final Release frees the data/native object; the wrapper owns
        // only raw fields and is then freed exactly once without another Release.
        unsafe {
            drop(Box::from_raw(this));
        }
    }
    remaining
}
unsafe extern "win64" fn pointer(this: *mut BinaryBlob) -> *mut c_void {
    let native = unsafe { (*this).native };
    unsafe { ((*(*native).vtable).get_buffer_pointer)(native) }
}
unsafe extern "win64" fn size(this: *mut BinaryBlob) -> usize {
    let native = unsafe { (*this).native };
    unsafe { ((*(*native).vtable).get_buffer_size)(native) }
}
static VTABLE: BlobVtable = BlobVtable {
    query,
    add_ref,
    release,
    pointer,
    size,
};
unsafe extern "win64" fn create_blob(size: usize, out: *mut *mut BinaryBlob) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = ptr::null_mut();
    }
    let mut native = ptr::null_mut();
    let hr = unsafe { com::D3DCreateBlob(size, &mut native) };
    if hr != S_OK {
        return hr;
    }
    let wrapper = unsafe { alloc(Layout::new::<BinaryBlob>()).cast::<BinaryBlob>() };
    if wrapper.is_null() {
        unsafe {
            ((*(*native).vtable).release)(native);
        }
        return E_OUTOFMEMORY;
    }
    unsafe {
        wrapper.write(BinaryBlob {
            vtable: &VTABLE,
            native,
        });
        *out = wrapper;
    }
    S_OK
}
pub(crate) fn address(name: &str) -> Option<usize> {
    match name {
        "D3DCreateBlob" => Some(create_blob as *const () as usize),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use com::{E_NOINTERFACE, IID_ID3DBLOB, IID_IUNKNOWN};
    #[test]
    fn win64_virtual_identity_aliases_buffers_and_final_ownership_are_real() {
        let factory: unsafe extern "win64" fn(usize, *mut *mut BinaryBlob) -> i32 = unsafe {
            std::mem::transmute(crate::abi::resolve("d3dcompiler_47.dll", "D3DCreateBlob").unwrap())
        };
        unsafe {
            assert_eq!(std::mem::offset_of!(BinaryBlob, vtable), 0);
            assert_eq!(
                std::mem::size_of::<BlobVtable>(),
                5 * std::mem::size_of::<usize>()
            );
            let mut blob = ptr::null_mut();
            assert_eq!(factory(37, &mut blob), S_OK);
            let v = &*(*blob).vtable;
            assert_eq!((v.size)(blob), 37);
            let p = (v.pointer)(blob).cast::<u8>();
            for i in 0..37 {
                p.add(i).write((i * 13 + 7) as u8);
            }
            let mut unknown = ptr::null_mut();
            let mut again = ptr::null_mut();
            assert_eq!((v.query)(blob, &IID_IUNKNOWN, &mut unknown), S_OK);
            assert_eq!((v.query)(unknown.cast(), &IID_ID3DBLOB, &mut again), S_OK);
            assert_eq!(unknown, blob.cast());
            assert_eq!(again, blob.cast());
            assert_eq!((v.pointer)(again.cast()), p.cast());
            assert_eq!((v.add_ref)(blob), 4);
            assert_eq!((v.release)(again.cast()), 3);
            assert_eq!((v.release)(unknown.cast()), 2);
            assert_eq!((v.release)(blob), 1);
            for i in 0..37 {
                assert_eq!(p.add(i).read(), (i * 13 + 7) as u8);
            }
            assert_eq!((v.release)(blob), 0);
        }
    }
    #[test]
    fn host_failure_contracts_clear_outputs_without_acquiring_references() {
        unsafe {
            assert_eq!(create_blob(16, ptr::null_mut()), E_POINTER);
            let mut blob = ptr::dangling_mut();
            assert_eq!(create_blob(usize::MAX, &mut blob), E_OUTOFMEMORY);
            assert!(blob.is_null());
            assert_eq!(create_blob(0, &mut blob), S_OK);
            let v = &*(*blob).vtable;
            assert_eq!((v.size)(blob), 0);
            let mut output = ptr::dangling_mut();
            let other = Guid {
                data1: 7,
                ..IID_IUNKNOWN
            };
            assert_eq!((v.query)(blob, &other, &mut output), E_NOINTERFACE);
            assert!(output.is_null());
            assert_eq!((v.query)(blob, ptr::null(), &mut output), E_POINTER);
            assert!(output.is_null());
            assert_eq!((v.query)(blob, &IID_IUNKNOWN, ptr::null_mut()), E_POINTER);
            assert_eq!((v.release)(blob), 0);
        }
    }
}
