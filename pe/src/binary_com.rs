//! Win64 ID3DBlob vtables over the existing native Rust COM blob implementation.
//! A wrapper exclusively owns a fresh native blob. Native interface pointers
//! never escape to guest callers; every guest virtual slot uses Win64 ABI.
#![allow(non_snake_case)]
use alkyhol::com::{self, Blob, E_OUTOFMEMORY, E_POINTER, Guid, S_OK};
use alkyhol::root_signature::{self, Description, VersionedDescription};
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
// Consume exactly one independently owned native reference, including failure.
// The serializer and factory both return fresh blobs with no native aliases.
unsafe fn wrap_owned(native: *mut Blob) -> Result<*mut BinaryBlob, i32> {
    if native.is_null() {
        return Ok(ptr::null_mut());
    }
    let wrapper = unsafe { alloc(Layout::new::<BinaryBlob>()).cast::<BinaryBlob>() };
    if wrapper.is_null() {
        unsafe { ((*(*native).vtable).release)(native) };
        return Err(E_OUTOFMEMORY);
    }
    unsafe {
        wrapper.write(BinaryBlob {
            vtable: &VTABLE,
            native,
        });
    }
    Ok(wrapper)
}
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
    match unsafe { wrap_owned(native) } {
        Ok(wrapper) => unsafe { *out = wrapper },
        Err(hr) => return hr,
    }
    S_OK
}

// Do not publish either native vtable. Prepare both guest wrappers before
// publishing outputs so allocation failure cannot leak an unreported reference.
unsafe fn serialize_outputs(
    out: *mut *mut BinaryBlob,
    error: *mut *mut BinaryBlob,
    serialize: impl FnOnce(*mut *mut c_void, *mut *mut c_void) -> i32,
) -> i32 {
    if !error.is_null() {
        unsafe { *error = ptr::null_mut() };
    }
    if out.is_null() {
        return E_POINTER;
    }
    unsafe { *out = ptr::null_mut() };
    let mut native_blob = ptr::null_mut();
    let mut native_error = ptr::null_mut();
    let hr = serialize(
        &mut native_blob,
        if error.is_null() {
            ptr::null_mut()
        } else {
            &mut native_error
        },
    );
    // Wrap both even when the first allocation fails: wrap_owned consumes its
    // reference on every path, so the second native output is also released.
    let blob = unsafe { wrap_owned(native_blob.cast()) };
    let diagnostic = unsafe { wrap_owned(native_error.cast()) };
    match (blob, diagnostic) {
        (Ok(blob), Ok(diagnostic)) => {
            unsafe {
                *out = blob;
                if !error.is_null() {
                    *error = diagnostic;
                }
            }
            hr
        }
        (blob, diagnostic) => {
            for blob in [blob, diagnostic].into_iter().flatten() {
                if !blob.is_null() {
                    unsafe { release(blob) };
                }
            }
            E_OUTOFMEMORY
        }
    }
}
unsafe extern "win64" fn serialize_root(
    desc: *const Description,
    version: i32,
    out: *mut *mut BinaryBlob,
    error: *mut *mut BinaryBlob,
) -> i32 {
    unsafe {
        serialize_outputs(out, error, |blob, diagnostic| {
            root_signature::D3D12SerializeRootSignature(desc, version, blob, diagnostic)
        })
    }
}
unsafe extern "win64" fn serialize_versioned_root(
    desc: *const VersionedDescription,
    out: *mut *mut BinaryBlob,
    error: *mut *mut BinaryBlob,
) -> i32 {
    unsafe {
        serialize_outputs(out, error, |blob, diagnostic| {
            root_signature::D3D12SerializeVersionedRootSignature(desc, blob, diagnostic)
        })
    }
}
pub(crate) fn d3d12_address(name: &str) -> Option<usize> {
    match name {
        "D3D12SerializeRootSignature" => Some(serialize_root as *const () as usize),
        "D3D12SerializeVersionedRootSignature" => {
            Some(serialize_versioned_root as *const () as usize)
        }
        _ => None,
    }
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
    unsafe fn contents(blob: *mut BinaryBlob) -> Vec<u8> {
        unsafe { std::slice::from_raw_parts(pointer(blob).cast::<u8>(), size(blob)).to_vec() }
    }
    #[test]
    fn root_serializers_publish_win64_owned_blobs_with_matching_layout_bytes() {
        type Legacy = unsafe extern "win64" fn(
            *const Description,
            i32,
            *mut *mut BinaryBlob,
            *mut *mut BinaryBlob,
        ) -> i32;
        type Versioned = unsafe extern "win64" fn(
            *const VersionedDescription,
            *mut *mut BinaryBlob,
            *mut *mut BinaryBlob,
        ) -> i32;
        let legacy: Legacy = unsafe {
            std::mem::transmute(
                crate::abi::resolve("d3d12.dll", "D3D12SerializeRootSignature").unwrap(),
            )
        };
        let versioned: Versioned = unsafe {
            std::mem::transmute(
                crate::abi::resolve("d3d12.dll", "D3D12SerializeVersionedRootSignature").unwrap(),
            )
        };
        assert_eq!(std::mem::size_of::<Description>(), 40);
        assert_eq!(std::mem::size_of::<root_signature::RawParameter>(), 32);
        assert_eq!(std::mem::offset_of!(VersionedDescription, description), 8);
        let mut parameter = root_signature::RawParameter {
            kind: 2,
            payload: root_signature::Payload { descriptor: [7, 4] },
            visibility: 0,
        };
        let desc = Description {
            count: 1,
            parameters: &parameter,
            sampler_count: 0,
            samplers: ptr::null(),
            flags: 1,
        };
        unsafe {
            let mut blob = ptr::null_mut();
            let mut diagnostic = ptr::dangling_mut();
            assert_eq!(legacy(&desc, 1, &mut blob, &mut diagnostic), S_OK);
            assert!(diagnostic.is_null());
            let bytes = contents(blob);
            let version = VersionedDescription {
                version: 1,
                description: desc,
            };
            let mut other = ptr::null_mut();
            assert_eq!(versioned(&version, &mut other, ptr::null_mut()), S_OK);
            assert_ne!(other, blob);
            assert_eq!(contents(other), bytes);
            parameter.payload.descriptor = [99, 0];
            assert_eq!(parameter.payload.descriptor, [99, 0]);
            assert_eq!(contents(blob), bytes); // Native snapshot owns its bytes.
            let mut alias = ptr::null_mut();
            assert_eq!(
                ((*(*blob).vtable).query)(blob, &IID_ID3DBLOB, &mut alias),
                S_OK
            );
            assert_eq!(alias, blob.cast());
            assert_eq!(release(alias.cast()), 1);
            assert_eq!(release(other), 0);
            assert_eq!(
                root_signature::parse(&bytes).unwrap().parameters[0].binding,
                root_signature::Binding::Descriptor {
                    kind: 2,
                    register: 7,
                    space: 4
                }
            );
            assert_eq!(release(blob), 0);
        }
    }
    #[test]
    fn serializer_failures_return_real_guest_diagnostics_and_clear_optional_outputs() {
        unsafe {
            let mut blob = ptr::dangling_mut();
            let mut diagnostic = ptr::dangling_mut();
            assert_eq!(
                serialize_root(ptr::null(), 1, &mut blob, &mut diagnostic),
                E_POINTER
            );
            assert!(blob.is_null());
            let text = contents(diagnostic);
            assert_eq!(text.last(), Some(&0));
            assert!(text.len() > 1);
            let mut alias = ptr::null_mut();
            assert_eq!(query(diagnostic, &IID_IUNKNOWN, &mut alias), S_OK);
            assert_eq!(alias, diagnostic.cast());
            assert_eq!(release(alias.cast()), 1);
            assert_eq!(release(diagnostic), 0);
            assert_eq!(
                serialize_versioned_root(ptr::null(), &mut blob, ptr::null_mut()),
                E_POINTER
            );
            assert!(blob.is_null());
            diagnostic = ptr::dangling_mut();
            assert_eq!(
                serialize_outputs(ptr::null_mut(), &mut diagnostic, |_, _| panic!(
                    "invalid output must not enter serializer"
                )),
                E_POINTER
            );
            assert!(diagnostic.is_null());
            let desc = VersionedDescription {
                version: 2,
                description: Description {
                    count: u32::MAX,
                    parameters: ptr::dangling(),
                    sampler_count: 0,
                    samplers: ptr::null(),
                    flags: 0,
                },
            };
            assert_eq!(
                serialize_versioned_root(&desc, &mut blob, &mut diagnostic),
                0x80004001u32 as i32
            );
            assert!(blob.is_null());
            assert_eq!(release(diagnostic), 0);
        }
    }
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
