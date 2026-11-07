//! Real CPU ID3D12Object metadata and seven-slot source-ABI vtable.
//! This is not a device, resource, fence or Metal object. The explicit diagnostic
//! constructor uses UTF-32 names; stored debug-name bytes retain Windows UTF-16 LE.
//!
//! All FFI pointers must be valid/aligned for their documented inputs and outputs;
//! callers retain a live object reference throughout each call. Foreign IUnknown
//! pointers must support AddRef/Release in every calling thread/apartment, with
//! external synchronization where required. No COM marshaling is performed.
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]
use crate::com::{E_NOINTERFACE, E_OUTOFMEMORY, E_POINTER, Guid, IID_IUNKNOWN, S_OK};
use std::{
    ffi::c_void,
    ptr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering, fence},
    },
};
const S_FALSE: i32 = 1;
const E_INVALIDARG: i32 = 0x80070057u32 as i32;
const NOT_FOUND: i32 = 0x887a0002u32 as i32;
const MORE_DATA: i32 = 0x887a0003u32 as i32;
pub const IID_ID3D12OBJECT: Guid = Guid {
    data1: 0xc4fec28f,
    data2: 0x7966,
    data3: 0x4e95,
    data4: [0x9f, 0x94, 0xf4, 0x31, 0xcb, 0x56, 0xc3, 0xb8],
};
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub static WKPDID_D3DDebugObjectNameW: Guid = Guid {
    data1: 0x4cca5fd8,
    data2: 0x921f,
    data3: 0x42c8,
    data4: [0x85, 0x66, 0x70, 0xca, 0xf2, 0xa9, 0xb7, 0x41],
};
#[repr(C)]
#[derive(Clone, Copy)]
struct UnknownVtable {
    query: unsafe extern "C" fn(*mut c_void, *const Guid, *mut *mut c_void) -> i32,
    add_ref: unsafe extern "C" fn(*mut c_void) -> u32,
    release: unsafe extern "C" fn(*mut c_void) -> u32,
}
unsafe fn unknown_vtable(pointer: *mut c_void) -> UnknownVtable {
    unsafe { **pointer.cast::<*const UnknownVtable>() }
}
struct OwnedInterface(*mut c_void);
// Acquisition is unsafe and requires that calls be permitted in all accessing
// threads/apartments; this does not turn an arbitrary foreign object into an agile one.
unsafe impl Send for OwnedInterface {}
unsafe impl Sync for OwnedInterface {}
impl Drop for OwnedInterface {
    fn drop(&mut self) {
        unsafe {
            (unknown_vtable(self.0).release)(self.0);
        }
    }
}
enum Entry {
    Bytes(Vec<u8>),
    Interface(Option<OwnedInterface>),
}
impl Entry {
    fn size(&self) -> u32 {
        match self {
            Self::Bytes(bytes) => bytes.len() as u32,
            Self::Interface(_) => std::mem::size_of::<*mut c_void>() as u32,
        }
    }
}
#[derive(Default)]
pub(crate) struct PrivateData {
    entries: Mutex<Vec<(Guid, Arc<Entry>)>>,
}
impl PrivateData {
    fn replace(&self, key: Guid, value: Entry) -> i32 {
        let value = Arc::new(value);
        let old = {
            let mut entries = self.entries.lock().unwrap();
            if let Some((_, slot)) = entries.iter_mut().find(|(k, _)| *k == key) {
                Some(std::mem::replace(slot, value))
            } else {
                if entries.try_reserve(1).is_err() {
                    return E_OUTOFMEMORY;
                }
                entries.push((key, value));
                None
            }
        };
        // Foreign callbacks must not execute with the store mutex held.
        drop(old);
        S_OK
    }
    fn remove(&self, key: Guid) -> i32 {
        let old = {
            let mut entries = self.entries.lock().unwrap();
            entries
                .iter()
                .position(|(k, _)| *k == key)
                .map(|i| entries.swap_remove(i).1)
        };
        if old.is_some() {
            drop(old);
            S_OK
        } else {
            S_FALSE
        }
    }
    fn bytes(&self, key: Guid, bytes: &[u8]) -> i32 {
        if bytes.len() > u32::MAX as usize {
            return E_INVALIDARG;
        }
        let mut copy = Vec::new();
        if copy.try_reserve_exact(bytes.len()).is_err() {
            return E_OUTOFMEMORY;
        }
        copy.extend_from_slice(bytes);
        self.replace(key, Entry::Bytes(copy))
    }
    unsafe fn interface(&self, key: Guid, pointer: *mut c_void) -> i32 {
        let value = if pointer.is_null() {
            None
        } else {
            unsafe {
                (unknown_vtable(pointer).add_ref)(pointer);
            }
            Some(OwnedInterface(pointer))
        };
        self.replace(key, Entry::Interface(value))
    }
    unsafe fn get(&self, key: Guid, size: *mut u32, out: *mut c_void) -> i32 {
        if size.is_null() {
            return E_POINTER;
        }
        let value = {
            let entries = self.entries.lock().unwrap();
            entries
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| Arc::clone(v))
        };
        let Some(value) = value else {
            unsafe {
                *size = 0;
            }
            return NOT_FOUND;
        };
        let available = unsafe { *size };
        let needed = value.size();
        unsafe {
            *size = needed;
        }
        if out.is_null() {
            return S_OK;
        }
        if available < needed {
            return MORE_DATA;
        }
        match &*value {
            Entry::Bytes(bytes) => unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), out.cast::<u8>(), bytes.len());
            },
            Entry::Interface(interface) => {
                let pointer = interface.as_ref().map_or(ptr::null_mut(), |p| p.0);
                if !pointer.is_null() {
                    unsafe {
                        (unknown_vtable(pointer).add_ref)(pointer);
                    }
                }
                unsafe {
                    ptr::write_unaligned(out.cast::<*mut c_void>(), pointer);
                }
            }
        }
        S_OK
    }
}
#[repr(C)]
pub struct ObjectVtable {
    pub query: unsafe extern "C" fn(*mut MetadataObject, *const Guid, *mut *mut c_void) -> i32,
    pub add_ref: unsafe extern "C" fn(*mut MetadataObject) -> u32,
    pub release: unsafe extern "C" fn(*mut MetadataObject) -> u32,
    pub get: unsafe extern "C" fn(*mut MetadataObject, *const Guid, *mut u32, *mut c_void) -> i32,
    pub set: unsafe extern "C" fn(*mut MetadataObject, *const Guid, u32, *const c_void) -> i32,
    pub set_interface: unsafe extern "C" fn(*mut MetadataObject, *const Guid, *mut c_void) -> i32,
    pub set_name: unsafe extern "C" fn(*mut MetadataObject, *const u32) -> i32,
}
#[repr(C)]
pub struct MetadataObject {
    pub vtable: *const ObjectVtable,
    refs: AtomicU32,
    data: PrivateData,
}
unsafe extern "C" fn query(
    this: *mut MetadataObject,
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
    let iid = unsafe { *iid };
    if iid != IID_IUNKNOWN && iid != IID_ID3D12OBJECT {
        return E_NOINTERFACE;
    }
    unsafe {
        add_ref(this);
        *out = this.cast();
    }
    S_OK
}
unsafe extern "C" fn add_ref(this: *mut MetadataObject) -> u32 {
    match unsafe {
        (*this)
            .refs
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
    } {
        Ok(old) => old + 1,
        Err(_) => std::process::abort(),
    }
}
unsafe extern "C" fn release(this: *mut MetadataObject) -> u32 {
    let old = unsafe { (*this).refs.fetch_sub(1, Ordering::Release) };
    if old == 1 {
        fence(Ordering::Acquire);
        unsafe {
            drop(Box::from_raw(this));
        }
        0
    } else {
        old - 1
    }
}
unsafe extern "C" fn get(
    this: *mut MetadataObject,
    key: *const Guid,
    size: *mut u32,
    out: *mut c_void,
) -> i32 {
    if key.is_null() {
        return E_POINTER;
    }
    unsafe { (*this).data.get(*key, size, out) }
}
unsafe extern "C" fn set(
    this: *mut MetadataObject,
    key: *const Guid,
    size: u32,
    input: *const c_void,
) -> i32 {
    if key.is_null() {
        return E_POINTER;
    }
    if input.is_null() {
        if size != 0 {
            return E_INVALIDARG;
        }
        return unsafe { (*this).data.remove(*key) };
    }
    unsafe {
        (*this).data.bytes(
            *key,
            std::slice::from_raw_parts(input.cast::<u8>(), size as usize),
        )
    }
}
unsafe extern "C" fn set_interface(
    this: *mut MetadataObject,
    key: *const Guid,
    input: *mut c_void,
) -> i32 {
    if key.is_null() {
        return E_POINTER;
    }
    unsafe { (*this).data.interface(*key, input) }
}
unsafe extern "C" fn set_name(this: *mut MetadataObject, input: *const u32) -> i32 {
    if input.is_null() {
        return E_POINTER;
    }
    let mut bytes = Vec::new();
    for i in 0..32768 {
        let value = unsafe { *input.add(i) };
        if value == 0 {
            bytes.extend_from_slice(&[0, 0]);
            return unsafe { (*this).data.bytes(WKPDID_D3DDebugObjectNameW, &bytes) };
        }
        let Some(character) = char::from_u32(value) else {
            return E_INVALIDARG;
        };
        let mut units = [0; 2];
        for unit in character.encode_utf16(&mut units) {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
    }
    E_INVALIDARG
}
static VTABLE: ObjectVtable = ObjectVtable {
    query,
    add_ref,
    release,
    get,
    set,
    set_interface,
    set_name,
};
/// Explicitly construct a CPU metadata base for ABI validation. No D3D12 device,
/// fence, resource or GPU capability is represented by this object. Returned
/// vtable accepts UTF-32 SetName input, and stores UTF-16 LE debug-name bytes.
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub unsafe extern "C" fn AlkyholCreateMetadataObject32(out: *mut *mut MetadataObject) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = Box::into_raw(Box::new(MetadataObject {
            vtable: &VTABLE,
            refs: AtomicU32::new(1),
            data: PrivateData::default(),
        }));
    }
    S_OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::com::{D3DCreateBlob, IID_ID3DBLOB};
    const KEY: Guid = Guid {
        data1: 17,
        data2: 0,
        data3: 0,
        data4: [0; 8],
    };
    #[test]
    fn source_vtable_identity_and_defensive_failures() {
        unsafe {
            assert_eq!(AlkyholCreateMetadataObject32(ptr::null_mut()), E_POINTER);
            let mut object = ptr::null_mut();
            assert_eq!(AlkyholCreateMetadataObject32(&mut object), S_OK);
            assert_eq!(std::mem::offset_of!(MetadataObject, vtable), 0);
            let v = &*(*object).vtable;
            for iid in [IID_IUNKNOWN, IID_ID3D12OBJECT] {
                let mut out = ptr::null_mut();
                assert_eq!((v.query)(object, &iid, &mut out), S_OK);
                assert_eq!(out, object.cast());
                assert_eq!((v.release)(out.cast()), 1);
            }
            let mut out = ptr::dangling_mut();
            assert_eq!((v.query)(object, &IID_ID3DBLOB, &mut out), E_NOINTERFACE);
            assert!(out.is_null());
            assert_eq!((v.query)(object, ptr::null(), &mut out), E_POINTER);
            assert!(out.is_null());
            assert_eq!((v.query)(object, &IID_IUNKNOWN, ptr::null_mut()), E_POINTER);
            assert_eq!(
                (v.get)(object, &KEY, ptr::null_mut(), ptr::null_mut()),
                E_POINTER
            );
            let mut size = 0;
            assert_eq!(
                (v.get)(object, ptr::null(), &mut size, ptr::null_mut()),
                E_POINTER
            );
            assert_eq!((v.set)(object, ptr::null(), 0, ptr::null()), E_POINTER);
            assert_eq!((v.set)(object, &KEY, 1, ptr::null()), E_INVALIDARG);
            assert_eq!(
                (v.set_interface)(object, ptr::null(), ptr::null_mut()),
                E_POINTER
            );
            assert_eq!((v.add_ref)(object), 2);
            assert_eq!((v.release)(object), 1);
            assert_eq!((v.release)(object), 0);
        }
    }
    #[test]
    fn copy_ownership_empty_entries_and_buffer_boundaries() {
        unsafe {
            let data = PrivateData::default();
            let mut size = 9;
            let mut out = [0xa5u8; 9];
            assert_eq!(data.get(KEY, &mut size, out.as_mut_ptr().cast()), NOT_FOUND);
            assert_eq!(size, 0);
            assert_eq!(out, [0xa5; 9]);
            let mut input = [1, 2, 3];
            assert_eq!(data.bytes(KEY, &input), S_OK);
            input[0] = 99;
            assert_eq!(input[0], 99);
            size = 2;
            assert_eq!(data.get(KEY, &mut size, out.as_mut_ptr().cast()), MORE_DATA);
            assert_eq!(size, 3);
            assert_eq!(out, [0xa5; 9]);
            size = 9;
            assert_eq!(data.get(KEY, &mut size, out.as_mut_ptr().cast()), S_OK);
            assert_eq!(size, 3);
            assert_eq!(out, [1, 2, 3, 0xa5, 0xa5, 0xa5, 0xa5, 0xa5, 0xa5]);
            assert_eq!(data.remove(KEY), S_OK);
            assert_eq!(data.remove(KEY), S_FALSE);
            assert_eq!(data.bytes(KEY, &[]), S_OK);
            size = 9;
            assert_eq!(data.get(KEY, &mut size, out.as_mut_ptr().cast()), S_OK);
            assert_eq!(size, 0);
            assert_eq!(data.interface(KEY, ptr::null_mut()), S_OK);
            size = 9;
            assert_eq!(data.get(KEY, &mut size, out.as_mut_ptr().cast()), S_OK);
            assert_eq!(size, std::mem::size_of::<usize>() as u32);
            assert_eq!(
                &out[..std::mem::size_of::<usize>()],
                &[0u8; std::mem::size_of::<usize>()]
            );
        }
    }
    #[test]
    fn retained_interfaces_size_queries_and_unaligned_pointer_outputs() {
        unsafe {
            let mut blob = ptr::null_mut();
            assert_eq!(D3DCreateBlob(13, &mut blob), S_OK);
            let b = &*(*blob).vtable;
            let data = PrivateData::default();
            assert_eq!(data.interface(KEY, blob.cast()), S_OK);
            assert_eq!((b.add_ref)(blob), 3);
            assert_eq!((b.release)(blob), 2);
            let mut size = 0;
            assert_eq!(data.get(KEY, &mut size, ptr::null_mut()), S_OK);
            assert_eq!(size, std::mem::size_of::<usize>() as u32);
            let mut bytes = [0xa5u8; 1 + std::mem::size_of::<usize>()];
            size -= 1;
            assert_eq!(
                data.get(KEY, &mut size, bytes.as_mut_ptr().add(1).cast()),
                MORE_DATA
            );
            assert!(bytes.iter().all(|b| *b == 0xa5));
            assert_eq!(
                data.get(KEY, &mut size, bytes.as_mut_ptr().add(1).cast()),
                S_OK
            );
            let retrieved = ptr::read_unaligned(bytes.as_ptr().add(1).cast::<*mut c_void>());
            assert_eq!(retrieved, blob.cast());
            assert_eq!(bytes[0], 0xa5);
            assert_eq!((b.release)(retrieved.cast()), 2);
            assert_eq!(data.interface(KEY, blob.cast()), S_OK);
            assert_eq!((b.add_ref)(blob), 3);
            assert_eq!((b.release)(blob), 2);
            assert_eq!(data.bytes(KEY, b"new bytes"), S_OK);
            assert_eq!((b.release)(blob), 0);
        }
    }
    #[test]
    fn utf32_names_preserve_utf16_le_and_reject_bad_input_without_partial_change() {
        unsafe {
            let mut object = ptr::null_mut();
            assert_eq!(AlkyholCreateMetadataObject32(&mut object), S_OK);
            let v = &*(*object).vtable;
            let name = [0x41, 0x3bb, 0x1f338, 0];
            assert_eq!((v.set_name)(object, name.as_ptr()), S_OK);
            let mut bytes = [0xa5u8; 16];
            let mut size = 16;
            assert_eq!(
                (v.get)(
                    object,
                    &WKPDID_D3DDebugObjectNameW,
                    &mut size,
                    bytes.as_mut_ptr().cast()
                ),
                S_OK
            );
            assert_eq!(size, 10);
            assert_eq!(
                &bytes[..10],
                &[0x41, 0, 0xbb, 3, 0x3c, 0xd8, 0x38, 0xdf, 0, 0]
            );
            assert_eq!(
                (v.set_name)(object, [0x41, 0xd800, 0].as_ptr()),
                E_INVALIDARG
            );
            assert_eq!((v.set_name)(object, ptr::null()), E_POINTER);
            assert_eq!(
                (v.set_name)(object, vec![0x41; 32768].as_ptr()),
                E_INVALIDARG
            );
            size = 0;
            assert_eq!(
                (v.get)(
                    object,
                    &WKPDID_D3DDebugObjectNameW,
                    &mut size,
                    ptr::null_mut()
                ),
                S_OK
            );
            assert_eq!(size, 10);
            assert_eq!((v.set_name)(object, [0].as_ptr()), S_OK);
            size = 16;
            assert_eq!(
                (v.get)(
                    object,
                    &WKPDID_D3DDebugObjectNameW,
                    &mut size,
                    bytes.as_mut_ptr().cast()
                ),
                S_OK
            );
            assert_eq!(size, 2);
            assert_eq!(&bytes[..2], &[0, 0]);
            assert_eq!((v.release)(object), 0);
        }
    }
    #[repr(C)]
    struct Reentrant {
        vtable: *const UnknownVtable,
        refs: AtomicU32,
        store: *const PrivateData,
        fire_add: std::sync::atomic::AtomicBool,
        fire_release: std::sync::atomic::AtomicBool,
        callbacks: AtomicU32,
    }
    unsafe fn callback(this: *mut c_void, on_add: bool) {
        unsafe {
            let this = &*this.cast::<Reentrant>();
            let trigger = if on_add {
                &this.fire_add
            } else {
                &this.fire_release
            };
            if trigger.swap(false, Ordering::SeqCst) {
                let store = &*this.store;
                // Detect lock retention without hanging the test if it regresses.
                let unlocked = store.entries.try_lock().is_ok();
                if unlocked && store.bytes(Guid { data1: 18, ..KEY }, b"callback") == S_OK {
                    this.callbacks.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    }
    unsafe extern "C" fn test_query(
        this: *mut c_void,
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
        if unsafe { *iid } != IID_IUNKNOWN {
            return E_NOINTERFACE;
        }
        unsafe {
            test_add(this);
            *out = this;
        }
        S_OK
    }
    unsafe extern "C" fn test_add(this: *mut c_void) -> u32 {
        unsafe {
            callback(this, true);
            (*this.cast::<Reentrant>())
                .refs
                .fetch_add(1, Ordering::SeqCst)
                + 1
        }
    }
    unsafe extern "C" fn test_release(this: *mut c_void) -> u32 {
        unsafe {
            callback(this, false);
            (*this.cast::<Reentrant>())
                .refs
                .fetch_sub(1, Ordering::SeqCst)
                - 1
        }
    }
    static REENTRANT: UnknownVtable = UnknownVtable {
        query: test_query,
        add_ref: test_add,
        release: test_release,
    };
    #[test]
    fn foreign_addref_and_release_callbacks_can_reenter_without_a_locked_store() {
        unsafe {
            let data = Box::new(PrivateData::default());
            let foreign = Box::new(Reentrant {
                vtable: &REENTRANT,
                refs: AtomicU32::new(1),
                store: &*data,
                fire_add: false.into(),
                fire_release: false.into(),
                callbacks: AtomicU32::new(0),
            });
            let pointer = (&*foreign as *const Reentrant).cast_mut().cast();
            assert_eq!(data.interface(KEY, pointer), S_OK);
            foreign.fire_add.store(true, Ordering::SeqCst);
            let mut size = std::mem::size_of::<usize>() as u32;
            let mut retrieved = ptr::null_mut();
            assert_eq!(
                data.get(KEY, &mut size, (&mut retrieved as *mut *mut c_void).cast()),
                S_OK
            );
            assert_eq!(retrieved, pointer);
            test_release(retrieved);
            foreign.fire_release.store(true, Ordering::SeqCst);
            assert_eq!(data.remove(KEY), S_OK);
            assert_eq!(foreign.callbacks.load(Ordering::SeqCst), 2);
            assert_eq!(foreign.refs.load(Ordering::SeqCst), 1);
            assert_eq!(test_release(pointer), 0);
        }
    }
    #[test]
    fn concurrent_byte_entries_keep_immutable_snapshots_and_independent_keys() {
        let data = Arc::new(PrivateData::default());
        let threads: Vec<_> = (0..8)
            .map(|i| {
                let data = Arc::clone(&data);
                std::thread::spawn(move || {
                    let key = Guid {
                        data1: 100 + i,
                        ..KEY
                    };
                    for n in 0..400 {
                        let input = [(i + n) as u8; 31];
                        assert_eq!(data.bytes(key, &input), S_OK);
                        let mut out = [0u8; 32];
                        let mut size = 32;
                        assert_eq!(
                            unsafe { data.get(key, &mut size, out.as_mut_ptr().cast()) },
                            S_OK
                        );
                        assert_eq!(size, 31);
                        assert_eq!(&out[..31], &input);
                        assert_eq!(out[31], 0);
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
    }
    #[test]
    fn metadata_interface_reference_ownership_is_atomic_while_a_caller_keeps_it_alive() {
        unsafe {
            let mut object = ptr::null_mut();
            assert_eq!(AlkyholCreateMetadataObject32(&mut object), S_OK);
            let address = object as usize;
            let threads: Vec<_> = (0..8)
                .map(|_| {
                    std::thread::spawn(move || {
                        let object = address as *mut MetadataObject;
                        let v = &*(*object).vtable;
                        for _ in 0..2000 {
                            (v.add_ref)(object);
                            (v.release)(object);
                        }
                    })
                })
                .collect();
            for thread in threads {
                thread.join().unwrap();
            }
            assert_eq!(((*(*object).vtable).release)(object), 0);
        }
    }
}
