use super::super::tests::{allocator, buffer, device};
use super::*;
use alkyhol::root_signature::{Binding, Layout, Parameter, serialize};
const DXIL: &[u8] = include_bytes!("../../tests/compute_probe.dxil");
fn layout() -> Layout {
    Layout {
        parameters: [(2, 0), (3, 0), (3, 2), (4, 0)]
            .into_iter()
            .map(|(kind, space)| Parameter {
                visibility: 0,
                binding: Binding::Descriptor {
                    kind,
                    register: 0,
                    space,
                },
            })
            .collect(),
        samplers: vec![],
        flags: 0,
    }
}
unsafe fn root(d: *mut Device, layout: &Layout) -> *mut c_void {
    let bytes = serialize(layout).unwrap();
    let mut out = ptr::null_mut();
    assert_eq!(
        unsafe {
            root_signatures::create(
                d,
                0,
                bytes.as_ptr() as usize,
                bytes.len(),
                &IID_IUNKNOWN,
                &mut out,
            )
        },
        S_OK
    );
    out
}
unsafe fn release_unknown(object: *mut c_void) {
    let table = unsafe { *(object as *const *const usize) };
    let release: unsafe extern "C" fn(*mut c_void) -> u32 =
        unsafe { std::mem::transmute(*table.add(2)) };
    assert_eq!(unsafe { release(object) }, 0);
}
#[repr(C)]
struct PsoDesc {
    root: *mut c_void,
    shader: *const u8,
    size: usize,
    node: u32,
    cache: usize,
    cache_size: usize,
    flags: u32,
}
unsafe fn pipeline(d: *mut Device, root: *mut c_void) -> *mut c_void {
    let desc = PsoDesc {
        root,
        shader: DXIL.as_ptr(),
        size: DXIL.len(),
        node: 0,
        cache: 0,
        cache_size: 0,
        flags: 0,
    };
    let mut out = ptr::null_mut();
    assert_eq!(
        unsafe {
            compute_pipelines::create(d, (&desc as *const PsoDesc).cast(), &IID_IUNKNOWN, &mut out)
        },
        S_OK
    );
    out
}
unsafe fn writable(d: *mut Device) -> *mut Resource {
    let heap = HeapProperties {
        kind: 1,
        page: 0,
        pool: 0,
        creation: 1,
        visible: 1,
    };
    let desc = ResourceDesc {
        dimension: 1,
        alignment: 0,
        width: 416,
        height: 1,
        depth: 1,
        mips: 1,
        format: 0,
        samples: 1,
        quality: 0,
        layout: 1,
        flags: 4,
    };
    let mut out = ptr::null_mut();
    assert_eq!(
        unsafe { committed(d, &heap, 0, &desc, 0x400, ptr::null(), &RESOURCE, &mut out) },
        S_OK
    );
    out.cast()
}
struct Fixture {
    d: *mut Device,
    a: *mut Allocator,
    l: *mut List,
    root: *mut c_void,
    pso: *mut c_void,
    resources: Vec<*mut Resource>,
}
impl Fixture {
    unsafe fn new(kind: i32, initial: bool) -> Self {
        unsafe {
            let d = device();
            let a = allocator(d, kind);
            let root = root(d, &layout());
            let pso = pipeline(d, root);
            let mut out = ptr::null_mut();
            assert_eq!(
                create_list(
                    d,
                    0,
                    kind,
                    a as usize,
                    if initial { pso as usize } else { 0 },
                    &GRAPHICS,
                    &mut out
                ),
                S_OK
            );
            Self {
                d,
                a,
                l: out.cast(),
                root,
                pso,
                resources: vec![
                    buffer(d, 2, 512),
                    buffer(d, 2, 128),
                    buffer(d, 2, 128),
                    buffer(d, 2, 128),
                    writable(d),
                ],
            }
        }
    }
    unsafe fn bind(&self) {
        unsafe {
            set_pipeline(self.l, self.pso as usize);
            set_root(self.l, self.root as usize);
            cbv(self.l, 0, gpu_address(self.resources[0]) + 256);
            srv(self.l, 1, gpu_address(self.resources[1]) + 16);
            srv(self.l, 2, gpu_address(self.resources[2]) + 8);
            uav(self.l, 3, gpu_address(self.resources[4]) + 16);
        }
    }
    unsafe fn transition(&self, before: u32, after: u32) {
        let b = Barrier {
            kind: 0,
            flags: 0,
            resource: self.resources[4],
            subresource: u32::MAX,
            before,
            after,
        };
        unsafe { barriers(self.l, 1, (&b as *const Barrier) as usize) }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        unsafe {
            list_release(self.l);
            allocator_release(self.a);
            for &r in &self.resources {
                resource_release(r);
            }
            if !self.pso.is_null() {
                release_unknown(self.pso);
            }
            if !self.root.is_null() {
                release_unknown(self.root);
            }
            assert_eq!(device_release(self.d), 0);
        }
    }
}

#[test]
fn same_root_preserves_arguments_different_root_clears_and_pipeline_switch_does_not_clear() {
    autoreleasepool(|| unsafe {
        let f = Fixture::new(0, true);
        f.bind();
        let before = (*f.l).recording.lock().unwrap().compute.arguments.clone();
        set_root(f.l, f.root as usize);
        set_pipeline(f.l, f.pso as usize);
        assert_eq!((*f.l).recording.lock().unwrap().compute.arguments, before);
        let other = root(f.d, &layout());
        set_root(f.l, other as usize);
        release_unknown(other);
        assert_eq!(
            (*f.l).recording.lock().unwrap().compute.arguments,
            vec![None; 4]
        );
        assert_eq!(close(f.l), S_OK);
        assert_eq!(allocator_reset(f.a), S_OK);
        assert_eq!(reset(f.l, f.a, f.pso as usize), S_OK);
        let recording = (*f.l).recording.lock().unwrap();
        assert!(recording.compute.pipeline.is_some());
        assert!(recording.compute.root.is_none());
        assert!(recording.compute.arguments.is_empty());
    });
}
#[test]
fn dispatch_snapshots_offsets_binding_addresses_and_native_lifetimes_before_later_mutation() {
    autoreleasepool(|| unsafe {
        let mut f = Fixture::new(2, true);
        f.bind();
        f.transition(0x400, 8);
        dispatch(f.l, 3, 1, 1);
        srv(f.l, 1, gpu_address(f.resources[3]) + 4);
        dispatch(f.l, 3, 1, 1);
        let record = (*f.l).recording.lock().unwrap();
        let commands: Vec<_> = record
            .commands
            .iter()
            .filter_map(|c| {
                if let Command::Dispatch(d) = c {
                    Some(d)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(commands.len(), 2);
        assert!(Arc::ptr_eq(
            &commands[0].buffers[1].memory,
            &(*f.resources[1])._descriptor_memory
        ));
        assert!(Arc::ptr_eq(
            &commands[1].buffers[1].memory,
            &(*f.resources[3])._descriptor_memory
        ));
        assert_eq!(
            commands[0]
                .buffers
                .iter()
                .map(|b| b.offset)
                .collect::<Vec<_>>(),
            [256, 16, 8, 16]
        );
        assert_eq!(
            commands[0]
                .buffers
                .iter()
                .map(|b| b.required)
                .collect::<Vec<_>>(),
            [1, 0x40, 0x40, 8]
        );
        let memory = Arc::downgrade(&commands[0].buffers[1].memory);
        drop(commands);
        drop(record);
        for r in f.resources.drain(..) {
            resource_release(r);
        }
        release_unknown(f.root);
        f.root = ptr::null_mut();
        release_unknown(f.pso);
        f.pso = ptr::null_mut();
        assert!(memory.upgrade().is_some());
        assert_eq!(close(f.l), S_OK);
    });
}
#[test]
fn zero_dimensions_record_no_work_and_excessive_group_counts_fail_close() {
    autoreleasepool(|| unsafe {
        let f = Fixture::new(0, true);
        f.bind();
        dispatch(f.l, 0, 1, 1);
        dispatch(f.l, 1, 0, 1);
        dispatch(f.l, 1, 1, 0);
        assert!((*f.l).recording.lock().unwrap().commands.is_empty());
        dispatch(f.l, 65536, 1, 1);
        assert_eq!(close(f.l), E_INVALIDARG);
    });
}
#[test]
fn missing_or_wrong_typed_bindings_preserve_the_first_recording_error() {
    autoreleasepool(|| unsafe {
        let f = Fixture::new(0, false);
        set_root(f.l, f.root as usize);
        cbv(f.l, 1, gpu_address(f.resources[0]));
        dispatch(f.l, 3, 1, 1);
        assert_eq!(close(f.l), E_INVALIDARG);
        assert_eq!(reset(f.l, f.a, f.pso as usize), E_INVALIDARG);
        let f = Fixture::new(0, false);
        set_root(f.l, f.root as usize);
        dispatch(f.l, 3, 1, 1);
        assert_eq!(close(f.l), E_INVALIDARG);
        let f = Fixture::new(0, true);
        set_root(f.l, f.root as usize);
        dispatch(f.l, 3, 1, 1);
        assert_eq!(close(f.l), E_INVALIDARG);
        let f = Fixture::new(0, true);
        f.bind();
        cbv(f.l, 0, gpu_address(f.resources[0]) + 4);
        assert_eq!(close(f.l), E_INVALIDARG);
        let f = Fixture::new(0, true);
        f.bind();
        srv(f.l, 1, gpu_address(f.resources[1]) + 128);
        dispatch(f.l, 3, 1, 1);
        assert_eq!(close(f.l), E_INVALIDARG);
    });
}
#[test]
fn uav_access_requires_a_default_writable_buffer_and_same_device_pso() {
    autoreleasepool(|| unsafe {
        for kind in [1, 2, 3] {
            let f = Fixture::new(0, true);
            f.bind();
            let wrong = buffer(f.d, kind, 512);
            uav(f.l, 3, gpu_address(wrong));
            dispatch(f.l, 3, 1, 1);
            assert_eq!(close(f.l), E_INVALIDARG);
            resource_release(wrong);
        }
        let f = Fixture::new(0, true);
        let other = Fixture::new(0, true);
        set_pipeline(f.l, other.pso as usize);
        assert_eq!(close(f.l), E_INVALIDARG);
    });
}
#[test]
fn uav_barrier_reads_only_its_active_union_and_compute_rejects_pixel_transitions() {
    autoreleasepool(|| unsafe {
        let f = Fixture::new(2, true);
        let mut storage = std::mem::MaybeUninit::<Barrier>::uninit();
        let raw = storage.as_mut_ptr().cast::<u8>();
        ptr::write(raw.cast::<i32>(), 2);
        ptr::write(raw.add(4).cast::<i32>(), 0);
        ptr::write(raw.add(8).cast::<*mut Resource>(), f.resources[4]);
        barriers(f.l, 1, raw as usize);
        assert!(matches!(
            (*f.l).recording.lock().unwrap().commands.last(),
            Some(Command::UavBarrier(_))
        ));
        f.transition(0x400, 0x80);
        assert_eq!(close(f.l), E_NOTIMPL);
    });
}
#[test]
fn private_shader_access_validates_the_entire_batch_and_rolls_back_uncommitted_reservations() {
    autoreleasepool(|| unsafe {
        let f = Fixture::new(0, true);
        f.bind();
        f.transition(0x400, 8);
        dispatch(f.l, 3, 1, 1);
        f.transition(8, 0x800);
        assert_eq!(close(f.l), S_OK);
        let second_allocator = allocator(f.d, 0);
        let second = super::super::tests::list(f.d, second_allocator, 0);
        let invalid = Barrier {
            kind: 0,
            flags: 0,
            resource: f.resources[4],
            subresource: u32::MAX,
            before: 8,
            after: 0x400,
        };
        barriers(second, 1, (&invalid as *const Barrier) as usize);
        assert_eq!(close(second), S_OK);
        let lists = [f.l, second];
        let owner = Arc::new(());
        assert!(matches!(
            super::super::prepare(f.d, 0, 2, lists.as_ptr() as usize, &owner),
            Err(E_INVALIDARG)
        ));
        assert_eq!((*f.resources[4]).usage.lock().unwrap().current, 0x400);
        assert_eq!((*f.resources[4]).usage.lock().unwrap().pending, 0);
        let one = [f.l];
        let batches = super::super::prepare(f.d, 0, 1, one.as_ptr() as usize, &owner).unwrap();
        assert_eq!((*f.resources[4]).usage.lock().unwrap().pending, 1);
        drop(batches);
        assert_eq!((*f.resources[4]).usage.lock().unwrap().current, 0x400);
        assert_eq!((*f.resources[4]).usage.lock().unwrap().pending, 0);
        list_release(second);
        allocator_release(second_allocator);
        let f = Fixture::new(0, true);
        f.bind();
        dispatch(f.l, 3, 1, 1);
        assert_eq!(close(f.l), S_OK);
        let one = [f.l];
        assert!(matches!(
            super::super::prepare(f.d, 0, 1, one.as_ptr() as usize, &Arc::new(())),
            Err(E_INVALIDARG)
        ));
    });
}
