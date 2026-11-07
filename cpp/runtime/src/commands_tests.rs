use super::*;
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
        assert_eq!(batches[0].commands[0].length, 256);
        assert_eq!((*l).recording.lock().unwrap().commands[0].length, 16);
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
