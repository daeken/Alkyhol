use super::*;
fn layout() -> Layout {
    Layout {
        parameters: vec![
            Parameter {
                visibility: 0,
                binding: Binding::Constants {
                    register: 0,
                    space: 1,
                    count: 4,
                },
            },
            Parameter {
                visibility: 5,
                binding: Binding::Table(vec![[0, 3, 0, 0, u32::MAX], [2, 2, 1, 0, 8]]),
            },
            Parameter {
                visibility: 0,
                binding: Binding::Descriptor {
                    kind: 4,
                    register: 2,
                    space: 3,
                },
            },
        ],
        samplers: vec![[0x15, 1, 2, 3, 0, 16, 8, 2, 0, f32::MAX.to_bits(), 0, 0, 5]],
        flags: 1,
    }
}
fn repaired(bytes: &mut [u8]) {
    let hash = crate::root_hash::checksum(&bytes[20..]);
    bytes[4..20].copy_from_slice(&hash);
}
#[test]
fn sdk_source_layouts_and_versioned_union_offsets_are_exact() {
    assert_eq!(std::mem::size_of::<Description>(), 40);
    assert_eq!(std::mem::size_of::<RawParameter>(), 32);
    assert_eq!(std::mem::offset_of!(RawParameter, payload), 8);
    assert_eq!(std::mem::offset_of!(RawParameter, visibility), 24);
    assert_eq!(std::mem::offset_of!(VersionedDescription, description), 8);
    assert_eq!(std::mem::size_of::<VersionedDescription>(), 48);
    assert_eq!(std::mem::size_of::<[u32; 13]>(), 52);
}
#[test]
fn owned_layout_preserves_every_register_space_append_offset_and_sampler_float_bit() {
    let expected = layout();
    let mut bytes = serialize(&expected).unwrap();
    let actual = parse(&bytes).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(&bytes[..4], b"DXBC");
    assert_eq!(&bytes[36..40], b"RTS0");
    assert_eq!(word(&bytes, 28).unwrap(), 1);
    bytes.fill(0);
    assert_eq!(actual, expected);
    assert_eq!(
        parse(
            &serialize(&Layout {
                parameters: vec![],
                samplers: vec![],
                flags: 0
            })
            .unwrap()
        )
        .unwrap()
        .parameters
        .len(),
        0
    );
}
#[test]
fn snapshots_deep_copy_sdk_pointer_arrays_and_ignore_unused_union_bytes() {
    let mut ranges = [[2u32, 4, 3, 7, u32::MAX]];
    let mut parameters = [
        RawParameter {
            kind: 0,
            payload: Payload {
                table: Table {
                    count: 1,
                    ranges: ranges.as_ptr(),
                },
            },
            visibility: 0,
        },
        RawParameter {
            kind: 1,
            payload: Payload {
                constants: [9, 3, 4],
            },
            visibility: 1,
        },
    ];
    let desc = Description {
        count: 2,
        parameters: parameters.as_ptr(),
        sampler_count: 0,
        samplers: ptr::null(),
        flags: 0,
    };
    let result = unsafe { snapshot(&desc) }.unwrap();
    ranges[0] = [0; 5];
    parameters[1].kind = u32::MAX;
    assert_eq!(ranges[0], [0; 5]);
    assert_eq!(parameters[1].kind, u32::MAX);
    assert_eq!(
        result.parameters[0].binding,
        Binding::Table(vec![[2, 4, 3, 7, u32::MAX]])
    );
    assert_eq!(
        result.parameters[1].binding,
        Binding::Constants {
            register: 9,
            space: 3,
            count: 4
        }
    );
    assert_eq!(parse(&serialize(&result).unwrap()).unwrap(), result);
}
#[test]
fn root_cost_namespace_visibility_and_table_offsets_are_checked_independently() {
    let mut l = Layout {
        parameters: vec![Parameter {
            visibility: 0,
            binding: Binding::Constants {
                register: 0,
                space: 0,
                count: 64,
            },
        }],
        samplers: vec![],
        flags: 0,
    };
    assert!(serialize(&l).is_ok());
    l.parameters.push(Parameter {
        visibility: 0,
        binding: Binding::Descriptor {
            kind: 2,
            register: 1,
            space: 0,
        },
    });
    assert_eq!(
        serialize(&l).unwrap_err().message,
        "root signature exceeds 64 DWORDs"
    );
    l.parameters[0].binding = Binding::Constants {
        register: 0,
        space: 0,
        count: 4,
    };
    l.parameters[1].binding = Binding::Descriptor {
        kind: 2,
        register: 0,
        space: 0,
    };
    assert_eq!(
        l.validate().unwrap_err().message,
        "overlapping shader registers"
    );
    l.parameters[0].visibility = 1;
    l.parameters[1].visibility = 5;
    assert!(l.validate().is_ok());
    l.parameters[0].visibility = 0;
    assert!(l.validate().is_err());
    l.parameters[1].binding = Binding::Descriptor {
        kind: 3,
        register: 0,
        space: 0,
    };
    assert!(l.validate().is_ok());
    l.parameters = vec![Parameter {
        visibility: 0,
        binding: Binding::Table(vec![[0, 3, 0, 0, 0], [1, 2, 0, 0, 2]]),
    }];
    assert_eq!(
        l.validate().unwrap_err().message,
        "overlapping descriptor offsets"
    );
    l.parameters[0].binding = Binding::Table(vec![[0, 3, 0, 0, 0], [1, 2, 0, 0, u32::MAX]]);
    assert!(l.validate().is_ok());
    l.parameters[0].binding = Binding::Table(vec![[3, 1, 0, 0, 0], [0, 1, 0, 0, 1]]);
    assert_eq!(l.validate().unwrap_err().code, INVALID);
}
#[test]
fn unsupported_versions_flags_counts_unbounded_ranges_and_sampler_fields_fail_explicitly() {
    let mut l = layout();
    l.flags = 0x80;
    assert_eq!(serialize(&l).unwrap_err().code, UNSUPPORTED);
    l = layout();
    l.parameters[0].visibility = 6;
    assert_eq!(l.validate().unwrap_err().code, UNSUPPORTED);
    l = layout();
    l.parameters[1].binding = Binding::Table(vec![[0, u32::MAX, 0, 0, 0]]);
    assert_eq!(l.validate().unwrap_err().code, UNSUPPORTED);
    l.parameters[1].binding = Binding::Table(vec![[0, 0, 0, 0, 0]]);
    assert_eq!(l.validate().unwrap_err().code, INVALID);
    for (index, value, code) in [
        (0, 2, INVALID),
        (1, 0, INVALID),
        (6, 0, INVALID),
        (7, 3, INVALID),
        (4, f32::NAN.to_bits(), UNSUPPORTED),
        (8, 3f32.to_bits(), INVALID),
        (12, 6, UNSUPPORTED),
    ] {
        let mut l = layout();
        l.samplers[0][index] = value;
        if index == 8 {
            l.samplers[0][9] = 2f32.to_bits();
        }
        assert_eq!(l.validate().unwrap_err().code, code);
    }
    let mut l = layout();
    l.samplers.push(l.samplers[0]);
    assert_eq!(
        l.validate().unwrap_err().message,
        "overlapping shader registers"
    );
}
#[test]
fn parser_rejects_all_truncations_and_checksum_repaired_out_of_bounds_offsets() {
    let original = serialize(&layout()).unwrap();
    for length in 0..original.len() {
        assert!(parse(&original[..length]).is_err(), "prefix {length}");
    }
    let mut bytes = original.clone();
    bytes[4] ^= 1;
    assert_eq!(
        parse(&bytes).unwrap_err().message,
        "root container length/checksum"
    );
    for (offset, value) in [
        (44, 2),
        (48, 65),
        (52, u32::MAX),
        (56, 17),
        (60, u32::MAX),
        (76, u32::MAX),
        (80, u32::MAX),
    ] {
        let mut bytes = original.clone();
        put(&mut bytes, offset, value);
        repaired(&mut bytes);
        assert!(parse(&bytes).is_err(), "word at {offset}");
    }
    let mut bytes = original;
    put(&mut bytes, 28, 2);
    repaired(&mut bytes);
    assert_eq!(parse(&bytes).unwrap_err().code, UNSUPPORTED);
}
unsafe fn bytes(blob: *mut c_void) -> Vec<u8> {
    let blob = blob.cast::<Blob>();
    let v = unsafe { &*(*blob).vtable };
    unsafe {
        std::slice::from_raw_parts(
            (v.get_buffer_pointer)(blob).cast::<u8>(),
            (v.get_buffer_size)(blob),
        )
    }
    .to_vec()
}
unsafe fn release(blob: *mut c_void) {
    let blob = blob.cast::<Blob>();
    assert_eq!(unsafe { ((*(*blob).vtable).release)(blob) }, 0);
}
#[test]
fn both_serialization_exports_return_real_owned_blobs_and_version_one_is_identical() {
    unsafe {
        let parameter = RawParameter {
            kind: 2,
            payload: Payload { descriptor: [7, 4] },
            visibility: 0,
        };
        let desc = Description {
            count: 1,
            parameters: &parameter,
            sampler_count: 0,
            samplers: ptr::null(),
            flags: 1,
        };
        let mut blob = ptr::null_mut();
        let mut error = ptr::dangling_mut();
        assert_eq!(
            D3D12SerializeRootSignature(&desc, 1, &mut blob, &mut error),
            S_OK
        );
        assert!(error.is_null());
        let content = bytes(blob);
        release(blob);
        let versioned = VersionedDescription {
            version: 1,
            description: desc,
        };
        assert_eq!(
            D3D12SerializeVersionedRootSignature(&versioned, &mut blob, &mut error),
            S_OK
        );
        assert!(error.is_null());
        assert_eq!(bytes(blob), content);
        release(blob);
        assert_eq!(
            parse(&content).unwrap().parameters[0].binding,
            Binding::Descriptor {
                kind: 2,
                register: 7,
                space: 4
            }
        );
    }
}
#[test]
fn serialization_errors_clear_outputs_and_provide_actual_nul_terminated_diagnostics() {
    unsafe {
        let desc = Description {
            count: 1,
            parameters: ptr::null(),
            sampler_count: 0,
            samplers: ptr::null(),
            flags: 0,
        };
        let mut blob = ptr::dangling_mut();
        let mut error = ptr::dangling_mut();
        assert_eq!(
            D3D12SerializeRootSignature(&desc, 1, &mut blob, &mut error),
            E_POINTER
        );
        assert!(blob.is_null());
        assert!(!error.is_null());
        assert_eq!(bytes(error).last(), Some(&0));
        release(error);
        assert_eq!(
            D3D12SerializeRootSignature(&desc, 1, ptr::null_mut(), &mut error),
            E_POINTER
        );
        assert!(error.is_null());
        assert_eq!(
            D3D12SerializeRootSignature(ptr::null(), 1, &mut blob, ptr::null_mut()),
            E_POINTER
        );
        assert!(blob.is_null());
        let unsupported = VersionedDescription {
            version: 2,
            description: desc,
        };
        assert_eq!(
            D3D12SerializeVersionedRootSignature(&unsupported, &mut blob, &mut error),
            UNSUPPORTED
        );
        assert!(blob.is_null());
        release(error);
        assert_eq!(
            D3D12SerializeVersionedRootSignature(ptr::null(), &mut blob, ptr::null_mut()),
            E_POINTER
        );
        let oversized = Description {
            count: u32::MAX,
            parameters: ptr::dangling(),
            sampler_count: 0,
            samplers: ptr::null(),
            flags: 0,
        };
        assert_eq!(
            D3D12SerializeRootSignature(&oversized, 1, &mut blob, ptr::null_mut()),
            UNSUPPORTED
        );
        assert!(blob.is_null());
    }
}
