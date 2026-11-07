use super::*;
use alkyhol::root_signature::{Binding, Parameter, serialize};
const DXIL: &[u8] = include_bytes!("../../tests/compute_probe.dxil");
fn layout() -> Layout {
    Layout {
        parameters: [(2, 0, 0), (3, 0, 0), (3, 0, 2), (4, 0, 0)]
            .into_iter()
            .map(|(kind, register, space)| Parameter {
                visibility: 0,
                binding: Binding::Descriptor {
                    kind,
                    register,
                    space,
                },
            })
            .collect(),
        samplers: vec![],
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
unsafe fn root(d: *mut Device, layout: &Layout) -> *mut c_void {
    let bytes = serialize(layout).unwrap();
    let mut out = ptr::null_mut();
    // The device accepts inherited interface requests with canonical root identity.
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
unsafe fn drop_root(root: *mut c_void) {
    let vtable = unsafe { *(root as *const *const usize) };
    let release: unsafe extern "C" fn(*mut c_void) -> u32 =
        unsafe { std::mem::transmute(*vtable.add(2)) };
    assert_eq!(unsafe { release(root) }, 0);
}
fn desc(root: *mut c_void) -> Description {
    Description {
        root: root as usize,
        shader: Bytes {
            pointer: DXIL.as_ptr() as usize,
            length: DXIL.len(),
        },
        node: 1,
        cache: Bytes {
            pointer: 0,
            length: 0,
        },
        flags: 0,
    }
}
unsafe fn pipeline(d: *mut Device, root: *mut c_void) -> *mut Pipeline {
    let desc = desc(root);
    let mut out = ptr::null_mut();
    assert_eq!(
        unsafe { create(d, (&desc as *const Description).cast(), &PIPELINE, &mut out) },
        S_OK
    );
    out.cast()
}

#[test]
fn typed_sdk_description_and_cached_blob_boundaries_are_exact() {
    assert_eq!(std::mem::size_of::<Description>(), 56);
    assert_eq!(std::mem::offset_of!(Description, node), 24);
    assert_eq!(std::mem::offset_of!(Description, cache), 32);
    assert_eq!(std::mem::offset_of!(Description, flags), 48);
    assert_eq!(
        std::mem::size_of::<Vtable>(),
        9 * std::mem::size_of::<usize>()
    );
    let valid = desc(ptr::dangling_mut());
    assert_eq!(validate_description(valid), Ok(()));
    for (candidate, error) in [
        (Description { root: 0, ..valid }, E_NOTIMPL),
        (Description { node: 2, ..valid }, E_NOTIMPL),
        (Description { flags: 1, ..valid }, E_NOTIMPL),
        (
            Description {
                cache: Bytes {
                    pointer: 1,
                    length: 0,
                },
                ..valid
            },
            E_NOTIMPL,
        ),
        (
            Description {
                shader: Bytes {
                    pointer: 0,
                    length: DXIL.len(),
                },
                ..valid
            },
            E_POINTER,
        ),
        (
            Description {
                shader: Bytes {
                    pointer: 1,
                    length: 0,
                },
                ..valid
            },
            E_INVALIDARG,
        ),
        (
            Description {
                shader: Bytes {
                    pointer: 1,
                    length: 16 * 1024 * 1024 + 1,
                },
                ..valid
            },
            E_INVALIDARG,
        ),
    ] {
        assert_eq!(validate_description(candidate), Err(error));
    }
}
#[test]
fn reflected_bindings_require_compute_visibility_namespace_and_register_space_coverage() {
    let mut translation = translate_compute_pipeline(DXIL).unwrap();
    assert_eq!(translation.threads_per_group, Some([8, 1, 1]));
    assert_eq!(validate_bindings(&layout(), &translation), Ok(()));
    let mut missing = layout();
    missing.parameters[2].binding = Binding::Descriptor {
        kind: 3,
        register: 0,
        space: 1,
    };
    assert_eq!(validate_bindings(&missing, &translation), Err(E_INVALIDARG));
    missing = layout();
    missing.parameters[0].visibility = 5;
    assert_eq!(validate_bindings(&missing, &translation), Err(E_INVALIDARG));
    missing = layout();
    missing.parameters[0].binding = Binding::Constants {
        register: 0,
        space: 0,
        count: 16,
    };
    assert_eq!(validate_bindings(&missing, &translation), Err(E_NOTIMPL));
    let tables = Layout {
        parameters: [(2, 0), (0, 0), (0, 2), (1, 0)]
            .into_iter()
            .map(|(class, space)| Parameter {
                visibility: 0,
                binding: Binding::Table(vec![[class, 1, 0, space, u32::MAX]]),
            })
            .collect(),
        samplers: vec![],
        flags: 0,
    };
    assert_eq!(validate_bindings(&tables, &translation), Ok(()));
    translation.resources[0].count = 2;
    assert_eq!(validate_bindings(&layout(), &translation), Err(E_NOTIMPL));
    translation.resources[0].count = 1;
    translation.requires_ray_query = true;
    assert_eq!(validate_bindings(&layout(), &translation), Err(E_NOTIMPL));
}
#[test]
fn real_pipeline_owns_bytecode_layout_metadata_and_parent_after_caller_release() {
    autoreleasepool(|| unsafe {
        let d = make();
        let r = root(d, &layout());
        let mut bytes = DXIL.to_vec();
        let mut description = desc(r);
        description.shader.pointer = bytes.as_ptr() as usize;
        let mut out = ptr::null_mut();
        assert_eq!(
            create(
                d,
                (&description as *const Description).cast(),
                &PIPELINE,
                &mut out
            ),
            S_OK
        );
        let p = out.cast::<Pipeline>();
        bytes.fill(0);
        drop_root(r);
        let owned = &(*p).data;
        assert_eq!(owned._dxil.as_ref(), DXIL);
        assert_eq!(owned._layout, layout());
        assert!(owned._native.max_total_threads_per_threadgroup() >= 8);
        for iid in [IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, PAGEABLE, PIPELINE] {
            assert_eq!(query(p, &iid, &mut out), S_OK);
            assert_eq!(out, p.cast());
            assert_eq!(release(out.cast()), 1);
        }
        assert_eq!(query(p, &DEVICE, &mut out), E_NOINTERFACE);
        assert!(out.is_null());
        assert_eq!(name(p, [b'p' as u32, 0x1f338, 0].as_ptr()), S_OK);
        let key = guid!(19, 1, 8, 1, 2, 3, 4, 5, 6, 7, 8);
        let input = [0x91u8, 0x17, 0xff];
        let mut actual = [0; 3];
        let mut size = 3;
        assert_eq!(set(p, &key, 3, input.as_ptr().cast()), S_OK);
        assert_eq!(get(p, &key, &mut size, actual.as_mut_ptr().cast()), S_OK);
        assert_eq!(actual, input);
        assert_eq!(cached(p, &mut out), E_NOTIMPL);
        assert!(out.is_null());
        assert_eq!(cached(p, ptr::null_mut()), E_POINTER);
        assert_eq!(device_release(d), 1);
        assert_eq!(device(p, &DEVICE, &mut out), S_OK);
        assert_eq!(out, d.cast());
        assert_eq!(device_release(out.cast()), 1);
        assert_eq!(release(p), 0);
    });
}
#[test]
fn actual_com_compiled_pipeline_executes_the_shared_dxil_and_preserves_all_twenty_tail_words() {
    autoreleasepool(|| unsafe {
        let d = make();
        let r = root(d, &layout());
        let p = pipeline(d, r);
        let data = Arc::clone(&(*p).data);
        drop_root(r);
        assert_eq!(release(p), 0);
        let mut settings = vec![19, 0x12345678, 2f32.to_bits(), 0.5f32.to_bits()];
        settings.extend([1f32, 1., 1., 1., 2., 3., 4., 5., -1., 2., -3., 4.].map(f32::to_bits));
        let a: Vec<_> = (0..19).map(|i| 0xfffffff0u32.wrapping_add(i)).collect();
        let b: Vec<_> = (0..19).map(|i| 3 * i + 7).collect();
        let buffers: Vec<_> = [settings, a, b, vec![0xdeadbeef; 96]]
            .iter()
            .map(|words| {
                (*d).metal.new_buffer_with_data(
                    words.as_ptr().cast(),
                    std::mem::size_of_val(words.as_slice()) as u64,
                    metal::MTLResourceOptions::StorageModeShared,
                )
            })
            .collect();
        let queue = (*d).metal.new_command_queue();
        let command = queue.new_command_buffer();
        let encoder = command.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&data._native);
        for ((class, space), buffer) in [
            (RegisterClass::Cbv, 0),
            (RegisterClass::Srv, 0),
            (RegisterClass::Srv, 2),
            (RegisterClass::Uav, 0),
        ]
        .into_iter()
        .zip(&buffers)
        {
            let binding = data
                ._translation
                .resources
                .iter()
                .find(|r| r.class == class && r.space == space && r.register == 0 && r.count == 1)
                .unwrap();
            encoder.set_buffer(u64::from(binding.metal_index), Some(buffer), 0);
        }
        encoder.dispatch_thread_groups(metal::MTLSize::new(3, 1, 1), metal::MTLSize::new(8, 1, 1));
        encoder.end_encoding();
        command.commit();
        command.wait_until_completed();
        assert_eq!(command.status(), metal::MTLCommandBufferStatus::Completed);
        let words = std::slice::from_raw_parts(buffers[3].contents().cast::<u32>(), 96);
        for i in 0..19 {
            let source = i / 8 * 8 + 7 - i % 8;
            let a = if source < 19 {
                0xfffffff0u32.wrapping_add(source as u32)
            } else {
                0
            };
            let b = 3 * i as u32 + 7;
            assert_eq!(
                &words[i * 4..i * 4 + 4],
                &[
                    a.wrapping_add(0x12345678),
                    (b << (i & 7)) ^ 0x12345678,
                    (4f32 * i as f32 + 22.).to_bits(),
                    if i & 1 != 0 { b } else { a }
                ]
            );
        }
        assert_eq!(&words[76..], &[0xdeadbeef; 20]);
        drop(data);
        assert_eq!(device_release(d), 0);
    });
}
#[test]
fn failed_creation_and_capability_checks_do_not_leak_interfaces_or_device_ownership() {
    autoreleasepool(|| unsafe {
        let d = make();
        let r = root(d, &layout());
        let mut out = ptr::dangling_mut();
        let valid = desc(r);
        let other_device = make();
        let other_root = root(other_device, &layout());
        let foreign = Description {
            root: other_root as usize,
            ..valid
        };
        assert_eq!(
            create(
                d,
                (&foreign as *const Description).cast(),
                &PIPELINE,
                &mut out
            ),
            E_INVALIDARG
        );
        assert!(out.is_null());
        drop_root(other_root);
        assert_eq!(device_release(other_device), 0);
        assert_eq!(create(d, ptr::null(), &PIPELINE, &mut out), E_POINTER);
        assert!(out.is_null());
        assert_eq!(
            create(d, (&valid as *const Description).cast(), &DEVICE, &mut out),
            E_NOINTERFACE
        );
        assert!(out.is_null());
        for candidate in [
            Description { node: 2, ..valid },
            Description {
                root: d as usize,
                ..valid
            },
            Description { root: 0, ..valid },
        ] {
            assert!(
                create(
                    d,
                    (&candidate as *const Description).cast(),
                    &PIPELINE,
                    &mut out
                ) < 0
            );
            assert!(out.is_null());
        }
        let mut corrupt = DXIL.to_vec();
        corrupt[0] = 0;
        let invalid = Description {
            shader: Bytes {
                pointer: corrupt.as_ptr() as usize,
                length: corrupt.len(),
            },
            ..valid
        };
        assert!(
            create(
                d,
                (&invalid as *const Description).cast(),
                &PIPELINE,
                &mut out
            ) < 0
        );
        assert!(out.is_null());
        assert_eq!(
            create(
                d,
                (&valid as *const Description).cast(),
                &PIPELINE,
                ptr::null_mut()
            ),
            1
        );
        drop_root(r);
        assert_eq!(device_release(d), 0);
    });
}

#[test]
fn compiled_pipeline_references_are_atomic_with_one_live_owner() {
    autoreleasepool(|| unsafe {
        let d = make();
        let r = root(d, &layout());
        let p = pipeline(d, r);
        drop_root(r);
        let address = p as usize;
        let workers: Vec<_> = (0..8)
            .map(|_| {
                std::thread::spawn(move || {
                    let p = address as *mut Pipeline;
                    for _ in 0..1000 {
                        add(p);
                        release(p);
                    }
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
        assert_eq!(release(p), 0);
        assert_eq!(device_release(d), 0);
    });
}
