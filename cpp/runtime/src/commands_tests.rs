use super::*;
unsafe fn prepare(
    d: *mut Device,
    kind: i32,
    count: u32,
    lists: usize,
) -> Result<Vec<Arc<Submission>>, i32> {
    unsafe { super::prepare(d, kind, count, lists, &Arc::new(())) }
}
fn copy_length(command: &Command) -> u64 {
    match command {
        Command::Copy(c) => c.length,
        Command::Transition(_) => panic!("expected copy"),
    }
}
unsafe fn device() -> *mut Device {
    let mut raw = ptr::null_mut();
    assert_eq!(
        unsafe { AlkyholCreateMetalDeviceSubset(ptr::null_mut(), &DEVICE, &mut raw) },
        S_OK
    );
    raw.cast()
}
unsafe fn allocator(device: *mut Device, kind: i32) -> *mut Allocator {
    let mut raw = ptr::null_mut();
    assert_eq!(
        unsafe { create_allocator(device, kind, &ALLOCATOR, &mut raw) },
        S_OK
    );
    raw.cast()
}
unsafe fn list(device: *mut Device, allocator: *mut Allocator, kind: i32) -> *mut List {
    let mut raw = ptr::null_mut();
    assert_eq!(
        unsafe { create_list(device, 1, kind, allocator as usize, 0, &GRAPHICS, &mut raw) },
        S_OK
    );
    raw.cast()
}
unsafe fn buffer(device: *mut Device, kind: i32, width: u64) -> *mut Resource {
    let heap = HeapProperties {
        kind,
        page: 0,
        pool: 0,
        creation: 1,
        visible: 1,
    };
    let desc = ResourceDesc {
        dimension: 1,
        alignment: 0,
        width,
        height: 1,
        depth: 1,
        mips: 1,
        format: 0,
        samples: 1,
        quality: 0,
        layout: 1,
        flags: 0,
    };
    let mut raw = ptr::null_mut();
    assert_eq!(
        unsafe {
            committed(
                device,
                &heap,
                0,
                &desc,
                if kind == 2 { 0xac3 } else { 0x400 },
                ptr::null(),
                &RESOURCE,
                &mut raw,
            )
        },
        S_OK
    );
    raw.cast()
}
#[test]
fn typed_layouts_preserve_all_sdk_slots_and_handle_argument_abi() {
    assert_eq!(
        std::mem::size_of::<AllocatorVtable>(),
        9 * std::mem::size_of::<usize>()
    );
    assert_eq!(
        std::mem::size_of::<ListVtable>(),
        60 * std::mem::size_of::<usize>()
    );
    assert_eq!(
        std::mem::offset_of!(ListVtable, CopyBufferRegion),
        15 * std::mem::size_of::<usize>()
    );
    assert_eq!(
        std::mem::offset_of!(ListVtable, CopyResource),
        17 * std::mem::size_of::<usize>()
    );
    assert_eq!(
        std::mem::offset_of!(ListVtable, ExecuteIndirect),
        59 * std::mem::size_of::<usize>()
    );
    assert_eq!(std::mem::size_of::<CpuHandle>(), 8);
    assert_eq!(std::mem::size_of::<GpuHandle>(), 8);
}
#[test]
fn copy_extents_reject_overflow_truncation_and_unsupported_byte_granularity() {
    assert_eq!(extent(0, 512, 512), Ok(()));
    assert_eq!(extent(512, 0, 512), Ok(()));
    for (offset, len) in [(u64::MAX, 4), (512, 4), (508, 8)] {
        assert_eq!(extent(offset, len, 512), Err(E_INVALIDARG));
    }
    for (offset, len) in [(1, 4), (0, 3)] {
        assert_eq!(extent(offset, len, 512), Err(E_NOTIMPL));
    }
}
#[test]
fn allocator_and_list_recording_reset_rules_and_release_are_consistent() {
    autoreleasepool(|| unsafe {
        let d = device();
        let a = allocator(d, 3);
        let l = list(d, a, 3);
        assert_eq!(kind(l), 3);
        assert_eq!(allocator_reset(a), E_FAIL);
        assert_eq!(reset(l, a, 0), E_FAIL);
        assert_eq!(close(l), S_OK);
        assert_eq!(allocator_reset(a), S_OK);
        assert_eq!(reset(l, a, 0), S_OK);
        assert_eq!(allocator_reset(a), E_FAIL);
        assert_eq!(list_release(l), 0);
        assert_eq!(allocator_reset(a), S_OK);
        let invalid = list(d, a, 3);
        assert_eq!(close(invalid), S_OK);
        assert_eq!(close(invalid), E_FAIL);
        assert_eq!(reset(invalid, a, 0), E_FAIL);
        assert_eq!(list_release(invalid), 0);
        assert_eq!(allocator_release(a), 0);
        assert_eq!(device_release(d), 0);
    });
}
#[test]
fn command_identity_metadata_and_parent_survive_allocator_shell_release() {
    autoreleasepool(|| unsafe {
        let d = device();
        let a = allocator(d, 0);
        let l = list(d, a, 0);
        let mut out = ptr::null_mut();
        for id in [IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, LIST, GRAPHICS] {
            assert_eq!(list_query(l, &id, &mut out), S_OK);
            assert_eq!(out, l.cast());
            list_release(out.cast());
        }
        assert_eq!(list_query(l, &PAGEABLE, &mut out), E_NOINTERFACE);
        assert!(out.is_null());
        assert_eq!(allocator_query(a, &PAGEABLE, &mut out), S_OK);
        allocator_release(out.cast());
        let key = guid!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
        let bytes = [3u8, 5, 7];
        assert_eq!(list_set(l, &key, 3, bytes.as_ptr().cast()), S_OK);
        let mut value = [0u8; 5];
        let mut size = 5;
        assert_eq!(
            list_get(l, &key, &mut size, value.as_mut_ptr().cast()),
            S_OK
        );
        assert_eq!(value, [3, 5, 7, 0, 0]);
        assert_eq!(size, 3);
        assert_eq!(list_name(l, [0x1f338u32, 0].as_ptr()), S_OK);
        assert_eq!(device_release(d), 2);
        assert_eq!(allocator_release(a), 0);
        assert_eq!(list_device(l, &DEVICE, &mut out), S_OK);
        assert_eq!(out, d.cast());
        assert_eq!(device_release(out.cast()), 1);
        assert_eq!(close(l), S_OK);
        assert_eq!(list_release(l), 0);
    });
}
#[test]
fn constructors_reject_foreign_busy_or_mismatched_allocators_and_clear_outputs() {
    autoreleasepool(|| unsafe {
        let d = device();
        let other = device();
        let a = allocator(d, 3);
        let foreign = allocator(other, 3);
        let compute = allocator(d, 2);
        let l = list(d, a, 3);
        let mut out = ptr::dangling_mut();
        for (node, kind, alloc, pipeline, error) in [
            (1, 3, foreign as usize, 0, E_INVALIDARG),
            (1, 3, compute as usize, 0, E_INVALIDARG),
            (1, 3, a as usize, 0, E_INVALIDARG),
            (1, 3, 0, 0, E_POINTER),
            (2, 3, a as usize, 0, E_NOTIMPL),
            (1, 1, a as usize, 0, E_NOTIMPL),
            (1, 3, a as usize, 1, E_NOTIMPL),
        ] {
            assert_eq!(
                create_list(d, node, kind, alloc, pipeline, &GRAPHICS, &mut out),
                error
            );
            assert!(out.is_null());
        }
        assert_eq!(
            create_list(d, 1, 3, a as usize, 0, &PAGEABLE, &mut out),
            E_NOINTERFACE
        );
        assert!(out.is_null());
        assert_eq!(create_allocator(d, 1, &ALLOCATOR, &mut out), E_NOTIMPL);
        assert!(out.is_null());
        assert_eq!(create_allocator(d, 3, ptr::null(), &mut out), E_POINTER);
        assert!(out.is_null());
        assert_eq!(create_allocator(d, 3, &GRAPHICS, &mut out), E_NOINTERFACE);
        assert!(out.is_null());
        assert_eq!(
            create_allocator(d, 3, &ALLOCATOR, ptr::null_mut()),
            E_POINTER
        );
        list_release(l);
        allocator_release(a);
        allocator_release(foreign);
        allocator_release(compute);
        assert_eq!(device_release(d), 0);
        assert_eq!(device_release(other), 0);
    });
}
#[test]
fn invalid_copy_is_sticky_and_never_repaired_by_close_or_reset() {
    autoreleasepool(|| unsafe {
        let d = device();
        let a = allocator(d, 3);
        let l = list(d, a, 3);
        let src = buffer(d, 2, 512);
        let dst = buffer(d, 3, 512);
        copy_buffer(l, dst, 508, src, 0, 8);
        assert_eq!(close(l), E_INVALIDARG);
        assert_eq!(reset(l, a, 0), E_INVALIDARG);
        assert_eq!(close(l), E_INVALIDARG);
        let lists = [l];
        assert!(matches!(
            prepare(d, 3, 1, lists.as_ptr() as usize),
            Err(E_FAIL)
        ));
        assert!(matches!(
            copy_command(d, dst, 0, src, 0, Some(1)),
            Err(E_NOTIMPL)
        ));
        assert!(matches!(
            copy_command(d, src, 0, dst, 0, Some(4)),
            Err(E_NOTIMPL)
        ));
        assert!(matches!(
            copy_command(d, ptr::null_mut(), 0, src, 0, Some(4)),
            Err(E_POINTER)
        ));
        list_release(l);
        resource_release(src);
        resource_release(dst);
        allocator_release(a);
        assert_eq!(device_release(d), 0);
    });
}
#[test]
fn execution_reservations_reject_stale_open_duplicate_and_incompatible_lists() {
    autoreleasepool(|| unsafe {
        let d = device();
        let a = allocator(d, 3);
        let l = list(d, a, 3);
        let lists = [l, l];
        assert!(matches!(
            prepare(d, 3, 1, lists.as_ptr() as usize),
            Err(E_FAIL)
        ));
        assert_eq!(close(l), S_OK);
        assert_eq!(allocator_reset(a), S_OK);
        assert!(matches!(
            prepare(d, 3, 1, lists.as_ptr() as usize),
            Err(E_FAIL)
        ));
        assert_eq!(reset(l, a, 0), S_OK);
        assert_eq!(close(l), S_OK);
        assert!(matches!(
            prepare(d, 0, 1, lists.as_ptr() as usize),
            Err(E_INVALIDARG)
        ));
        assert!(matches!(prepare(d, 3, 0, 0), Err(E_NOTIMPL)));
        assert!(matches!(prepare(d, 3, 1, 0), Err(E_POINTER)));
        assert!(matches!(
            prepare(d, 3, 2, lists.as_ptr() as usize),
            Err(E_FAIL)
        ));
        assert_eq!((*a).state.lock().unwrap().in_flight, 0);
        let batches = prepare(d, 3, 1, lists.as_ptr() as usize).unwrap();
        assert_eq!(allocator_reset(a), E_FAIL);
        assert!(matches!(
            prepare(d, 3, 1, lists.as_ptr() as usize),
            Err(E_FAIL)
        ));
        drop(batches);
        assert_eq!(allocator_reset(a), S_OK);
        list_release(l);
        allocator_release(a);
        assert_eq!(device_release(d), 0);
    });
}
#[test]
fn resetting_list_preserves_pending_snapshot_and_finishes_reservations_exactly_once() {
    autoreleasepool(|| unsafe {
        let d = device();
        let a = allocator(d, 3);
        let second = allocator(d, 3);
        let l = list(d, a, 3);
        let src = buffer(d, 2, 512);
        let dst = buffer(d, 3, 512);
        copy_buffer(l, dst, 32, src, 4, 256);
        assert_eq!(close(l), S_OK);
        let lists = [l];
        let batches = prepare(d, 3, 1, lists.as_ptr() as usize).unwrap();
        assert_eq!(reset(l, second, 0), S_OK);
        copy_buffer(l, dst, 0, src, 0, 16);
        assert_eq!(batches[0].commands.len(), 1);
        assert_eq!(copy_length(&batches[0].commands[0]), 256);
        assert_eq!(copy_length(&(*l).recording.lock().unwrap().commands[0]), 16);
        assert_eq!(allocator_reset(a), E_FAIL);
        batches[0].complete();
        batches[0].complete();
        drop(batches);
        assert_eq!(allocator_reset(a), S_OK);
        assert_eq!((*second).state.lock().unwrap().in_flight, 0);
        assert_eq!(allocator_reset(second), E_FAIL);
        list_release(l);
        assert_eq!(allocator_reset(second), S_OK);
        allocator_release(a);
        allocator_release(second);
        resource_release(src);
        resource_release(dst);
        assert_eq!(device_release(d), 0);
    });
}
#[test]
fn immutable_full_copy_encodes_actual_metal_work_and_retains_released_com_buffers() {
    autoreleasepool(|| unsafe {
        let d = device();
        let a = allocator(d, 3);
        let l = list(d, a, 3);
        let src = buffer(d, 2, 512);
        let dst = buffer(d, 3, 512);
        let short = buffer(d, 3, 256);
        assert!(matches!(
            copy_command(d, short, 0, src, 0, None),
            Err(E_INVALIDARG)
        ));
        let mut read = ptr::null_mut();
        assert_eq!(map(dst, 0, ptr::null(), &mut read), S_OK);
        std::ptr::write_bytes(read.cast::<u8>(), 0xa5, 512);
        let mut write = ptr::null_mut();
        assert_eq!(map(src, 0, ptr::null(), &mut write), S_OK);
        std::ptr::write_bytes(write.cast::<u8>(), 0x19, 512);
        copy_resource(l, dst, src);
        assert_eq!(close(l), S_OK);
        assert!(
            std::slice::from_raw_parts(read.cast::<u8>(), 512)
                .iter()
                .all(|&b| b == 0xa5)
        );
        // Later upload contents, not a recording-time CPU snapshot, are copied.
        for (i, b) in std::slice::from_raw_parts_mut(write.cast::<u8>(), 512)
            .iter_mut()
            .enumerate()
        {
            *b = (i * 29 + 7) as u8;
        }
        let lists = [l];
        let batches = prepare(d, 3, 1, lists.as_ptr() as usize).unwrap();
        let native_dst = (*dst).buffer.to_owned();
        let offset = (*dst).offset;
        unmap(dst, 0, ptr::null());
        unmap(src, 0, ptr::null());
        resource_release(src);
        resource_release(dst);
        resource_release(short);
        list_release(l);
        allocator_release(a);
        let queue = (*d).metal.new_command_queue();
        let command = queue.new_command_buffer();
        let encoder = command.new_blit_command_encoder();
        for batch in &batches {
            batch.encode(encoder);
        }
        encoder.end_encoding();
        command.commit();
        command.wait_until_completed();
        assert_eq!(command.status(), metal::MTLCommandBufferStatus::Completed);
        for batch in &batches {
            batch.complete();
        }
        drop(batches);
        let actual =
            std::slice::from_raw_parts(native_dst.contents().cast::<u8>().add(offset), 512);
        for (i, &b) in actual.iter().enumerate() {
            assert_eq!(b, (i * 29 + 7) as u8);
        }
        assert_eq!(device_release(d), 0);
    });
}

#[test]
fn default_storage_is_private_and_cpu_mapping_fails_without_exposing_contents() {
    autoreleasepool(|| unsafe {
        let d = device();
        let resource = buffer(d, 1, 512);
        assert_eq!(
            (*resource).buffer.storage_mode(),
            metal::MTLStorageMode::Private
        );
        assert_eq!(
            (*resource).buffer.hazard_tracking_mode(),
            metal::MTLHazardTrackingMode::Tracked
        );
        let mut mapped = ptr::dangling_mut();
        assert_eq!(map(resource, 0, ptr::null(), &mut mapped), E_INVALIDARG);
        assert!(mapped.is_null());
        assert_eq!(gpu_address(resource) & 65535, 0);
        assert_ne!(gpu_address(resource), 0);
        assert_eq!((*resource).usage.lock().unwrap().current, 0x400);
        resource_release(resource);
        assert_eq!(device_release(d), 0);
    });
}
#[test]
fn transition_layouts_and_unsupported_barriers_never_succeed() {
    assert_eq!(std::mem::size_of::<Barrier>(), 32);
    assert_eq!(std::mem::offset_of!(Barrier, resource), 8);
    assert_eq!(std::mem::offset_of!(Barrier, after), 24);
    autoreleasepool(|| unsafe {
        let d = device();
        let resource = buffer(d, 1, 512);
        let a = allocator(d, 0);
        let l = list(d, a, 0);
        let valid = Barrier {
            kind: 0,
            flags: 0,
            resource,
            subresource: u32::MAX,
            before: 0x400,
            after: 0x800,
        };
        assert!(transition(d, 0, valid).is_ok());
        for (b, expected) in [
            (Barrier { flags: 1, ..valid }, E_NOTIMPL),
            (Barrier { kind: 2, ..valid }, E_NOTIMPL),
            (
                Barrier {
                    subresource: 1,
                    ..valid
                },
                E_INVALIDARG,
            ),
            (Barrier { after: 8, ..valid }, E_NOTIMPL),
            (
                Barrier {
                    resource: ptr::null_mut(),
                    ..valid
                },
                E_POINTER,
            ),
        ] {
            assert!(matches!(transition(d,0,b),Err(error) if error==expected));
        }
        assert!(matches!(transition(d, 3, valid), Err(E_NOTIMPL)));
        let invalid = Barrier { flags: 1, ..valid };
        barriers(l, 1, (&invalid as *const Barrier) as usize);
        assert_eq!(close(l), E_NOTIMPL);
        assert_eq!(reset(l, a, 0), E_NOTIMPL);
        assert_eq!((*resource).usage.lock().unwrap().current, 0x400);
        list_release(l);
        allocator_release(a);
        resource_release(resource);
        assert_eq!(device_release(d), 0);
    });
}
#[test]
fn private_state_reservations_roll_back_uncommitted_work_and_reject_cross_queue_overlap() {
    autoreleasepool(|| unsafe {
        let d = device();
        let a = allocator(d, 0);
        let l = list(d, a, 0);
        let src = buffer(d, 2, 512);
        let dst = buffer(d, 1, 512);
        copy_resource(l, dst, src);
        assert_eq!(close(l), S_OK);
        let owner = Arc::new(());
        let other_owner = Arc::new(());
        let lists = [l];
        let batches = super::prepare(d, 0, 1, lists.as_ptr() as usize, &owner).unwrap();
        {
            let usage = (*dst).usage.lock().unwrap();
            assert_eq!(usage.current, 0);
            assert_eq!(usage.pending, 1);
            assert!(Arc::ptr_eq(usage.owner.as_ref().unwrap(), &owner));
        }
        // A second queue is rejected before any buffer state can change.
        let other_a = allocator(d, 0);
        let other_l = list(d, other_a, 0);
        copy_resource(other_l, dst, src);
        assert_eq!(close(other_l), S_OK);
        let others = [other_l];
        assert!(matches!(
            super::prepare(d, 0, 1, others.as_ptr() as usize, &other_owner),
            Err(E_NOTIMPL)
        ));
        drop(batches);
        {
            let usage = (*dst).usage.lock().unwrap();
            assert_eq!(usage.current, 0x400);
            assert_eq!(usage.pending, 0);
            assert!(usage.owner.is_none());
        }
        let accepted = super::prepare(d, 0, 1, others.as_ptr() as usize, &other_owner).unwrap();
        for batch in &accepted {
            batch.commit();
            batch.complete();
        }
        drop(accepted);
        assert_eq!((*dst).usage.lock().unwrap().current, 0);
        list_release(l);
        list_release(other_l);
        allocator_release(a);
        allocator_release(other_a);
        resource_release(src);
        resource_release(dst);
        assert_eq!(device_release(d), 0);
    });
}
#[test]
fn a_batch_never_decays_between_lists_and_failed_validation_changes_no_resource_state() {
    autoreleasepool(|| unsafe {
        let d = device();
        let a = allocator(d, 0);
        let second = allocator(d, 0);
        let l = list(d, a, 0);
        let tail = list(d, second, 0);
        let src = buffer(d, 2, 512);
        let private = buffer(d, 1, 512);
        let dst = buffer(d, 3, 512);
        copy_resource(l, private, src);
        assert_eq!(close(l), S_OK);
        copy_resource(tail, dst, private);
        assert_eq!(close(tail), S_OK);
        let batch = [l, tail];
        let owner = Arc::new(());
        assert!(matches!(
            super::prepare(d, 0, 2, batch.as_ptr() as usize, &owner),
            Err(E_INVALIDARG)
        ));
        {
            let usage = (*private).usage.lock().unwrap();
            assert_eq!(usage.current, 0x400);
            assert_eq!(usage.pending, 0);
        }
        // Reset only the second list and insert the missing transition.
        assert_eq!(reset(tail, second, 0), S_OK);
        let barrier = Barrier {
            kind: 0,
            flags: 0,
            resource: private,
            subresource: 0,
            before: 0x400,
            after: 0x800,
        };
        barriers(tail, 1, (&barrier as *const Barrier) as usize);
        copy_resource(tail, dst, private);
        assert_eq!(close(tail), S_OK);
        let accepted = super::prepare(d, 0, 2, batch.as_ptr() as usize, &owner).unwrap();
        for member in &accepted {
            member.commit();
        }
        accepted[0].complete();
        assert_eq!((*private).usage.lock().unwrap().pending, 1);
        accepted[1].complete();
        assert_eq!((*private).usage.lock().unwrap().pending, 0);
        drop(accepted);
        assert_eq!((*private).usage.lock().unwrap().current, 0);
        list_release(l);
        list_release(tail);
        allocator_release(a);
        allocator_release(second);
        resource_release(src);
        resource_release(private);
        resource_release(dst);
        assert_eq!(device_release(d), 0);
    });
}
#[test]
fn failed_gpu_completion_poison_is_retained_and_cannot_be_reused_by_another_queue() {
    autoreleasepool(|| unsafe {
        let d = device();
        let a = allocator(d, 0);
        let l = list(d, a, 0);
        let src = buffer(d, 2, 512);
        let dst = buffer(d, 1, 512);
        copy_resource(l, dst, src);
        assert_eq!(close(l), S_OK);
        let lists = [l];
        let owner = Arc::new(());
        let batches = super::prepare(d, 0, 1, lists.as_ptr() as usize, &owner).unwrap();
        batches[0].commit();
        batches[0].finish(true);
        drop(batches);
        assert_eq!((*dst).usage.lock().unwrap().pending, 0);
        assert!((*dst).usage.lock().unwrap().failed);
        assert!(
            matches!(super::prepare(d,0,1,lists.as_ptr() as usize,&Arc::new(())),Err(error) if error==0x887a0005u32 as i32)
        );
        list_release(l);
        allocator_release(a);
        resource_release(src);
        resource_release(dst);
        assert_eq!(device_release(d), 0);
    });
}
