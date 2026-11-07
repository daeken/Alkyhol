use super::*;

struct Fixture {
    device: *mut Device,
    source: Arc<HeapData>,
    visible: Arc<HeapData>,
    memory: Arc<BufferMemory>,
}
impl Fixture {
    fn new() -> Self {
        unsafe {
            let mut raw = ptr::null_mut();
            assert_eq!(
                AlkyholCreateMetalDeviceSubset(ptr::null_mut(), &DEVICE, &mut raw),
                S_OK
            );
            let device = raw.cast::<Device>();
            let native = &(*device).metal;
            let source = Arc::new(
                HeapData::new(
                    native.new_buffer(512, metal::MTLResourceOptions::StorageModeShared),
                    0,
                    0,
                    8,
                )
                .unwrap(),
            );
            let visible = Arc::new(
                HeapData::new(
                    native.new_buffer(512, metal::MTLResourceOptions::StorageModeShared),
                    0,
                    1,
                    8,
                )
                .unwrap(),
            );
            let buffer = native.new_buffer(4096, metal::MTLResourceOptions::StorageModeShared);
            let offset = (256 - buffer.gpu_address() % 256) % 256;
            let memory = Arc::new(BufferMemory {
                buffer,
                offset,
                width: 1024,
                heap: 2,
                flags: 0,
                usage: Arc::new(Mutex::new(BufferUsage {
                    current: 0xac3,
                    pending: 0,
                    owner: None,
                    failed: false,
                })),
            });
            let mut registry = (*device).descriptors.lock().unwrap();
            registry.heap(&source);
            registry.heap(&visible);
            registry.buffer(&memory);
            drop(registry);
            Self {
                device,
                source,
                visible,
                memory,
            }
        }
    }
    fn base(&self) -> u64 {
        self.memory.buffer.gpu_address() + self.memory.offset
    }
    fn initialize(&self, index: usize, block: u64) {
        let desc = Description {
            address: self.base() + block * 256,
            length: 256,
        };
        assert_eq!(
            unsafe {
                initialize(
                    self.device,
                    (&desc as *const Description) as usize,
                    self.source.start() + index * 64,
                )
            },
            Ok(())
        );
    }
    fn copy(
        &self,
        dest: &[usize],
        ds: Option<&[u32]>,
        source: &[usize],
        ss: Option<&[u32]>,
    ) -> Result<(), i32> {
        unsafe {
            copy(
                self.device,
                dest.len() as u32,
                dest.as_ptr() as usize,
                ds.map_or(0, |s| s.as_ptr() as usize),
                source.len() as u32,
                source.as_ptr() as usize,
                ss.map_or(0, |s| s.as_ptr() as usize),
                0,
            )
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert_eq!(unsafe { device_release(self.device) }, 0);
    }
}

#[test]
fn record_layout_and_cbv_alignment_size_heap_and_live_address_boundaries() {
    assert_eq!(std::mem::size_of::<Record>(), 64);
    assert_eq!(std::mem::offset_of!(Record, address), 8);
    assert_eq!(std::mem::size_of::<Description>(), 16);
    autoreleasepool(|| {
        let f = Fixture::new();
        let registry = unsafe { (*f.device).descriptors.lock().unwrap() };
        let desc = Description {
            address: f.base(),
            length: 256,
        };
        assert_eq!(registry.cbv(desc).unwrap().offset, 0);
        assert_eq!(
            registry
                .cbv(Description {
                    address: f.base() + 768,
                    ..desc
                })
                .unwrap()
                .offset,
            768
        );
        for bad in [
            Description {
                address: f.base() + 1,
                ..desc
            },
            Description { length: 0, ..desc },
            Description {
                length: 255,
                ..desc
            },
            Description {
                length: 65792,
                ..desc
            },
            Description {
                address: f.base() + 1024,
                ..desc
            },
            Description {
                address: u64::MAX - 255,
                ..desc
            },
        ] {
            assert_eq!(registry.cbv(bad).err(), Some(E_INVALIDARG));
        }
        assert_eq!(
            registry.cbv(Description { address: 0, ..desc }).err(),
            Some(E_NOTIMPL)
        );
        let mut other = Registry::default();
        assert_eq!(other.cbv(desc).err(), Some(E_INVALIDARG));
        let private = Arc::new(BufferMemory {
            buffer: f.memory.buffer.clone(),
            offset: f.memory.offset,
            width: 1024,
            heap: 1,
            flags: 0,
            usage: Arc::new(Mutex::new(BufferUsage {
                current: 0x400,
                pending: 0,
                owner: None,
                failed: false,
            })),
        });
        other.buffer(&private);
        assert_eq!(other.cbv(desc).err(), Some(E_NOTIMPL));
        drop(private);
        assert_eq!(other.cbv(desc).err(), Some(E_INVALIDARG));
    });
}

#[test]
fn handles_require_live_typed_aligned_intervals_and_cpu_copy_sources() {
    autoreleasepool(|| {
        let f = Fixture::new();
        let registry = unsafe { (*f.device).descriptors.lock().unwrap() };
        assert_eq!(
            registry.span(f.source.start() + 7 * 64, 1, true).unwrap()[0].1,
            7
        );
        for (handle, count) in [
            (0, 1),
            (f.source.start() + 1, 1),
            (f.source.start() + 7 * 64, 2),
            (f.source.start() + 512, 1),
            (usize::MAX, 1),
        ] {
            assert_eq!(registry.span(handle, count, true).err(), Some(E_INVALIDARG));
        }
        assert_eq!(
            registry.span(f.visible.start(), 1, true).err(),
            Some(E_NOTIMPL)
        );
        assert_eq!(
            registry.span(f.source.start(), 0, false).err(),
            Some(E_NOTIMPL)
        );
        drop(registry);
        let mut registry = Registry::default();
        let wrong = Arc::new(HeapData::new(f.source.storage.clone(), 1, 0, 8).unwrap());
        registry.heap(&wrong);
        assert_eq!(
            registry.span(wrong.start(), 1, false).err(),
            Some(E_INVALIDARG)
        );
        drop(wrong);
        assert_eq!(
            registry.span(f.source.start(), 1, false).err(),
            Some(E_INVALIDARG)
        );
        registry.heap(&f.source);
        assert_eq!(registry.heaps.len(), 1);
    });
}

#[test]
fn copies_snapshot_values_and_flatten_ranges_with_optional_unit_sizes() {
    autoreleasepool(|| {
        let f = Fixture::new();
        for i in 0..4 {
            f.initialize(i, i as u64);
        }
        let d = f.visible.start();
        let s = f.source.start();
        assert_eq!(f.copy(&[d], Some(&[4]), &[s], Some(&[4])), Ok(()));
        f.initialize(0, 3);
        assert_eq!(
            f.visible.slots.lock().unwrap()[0].as_ref().unwrap().offset,
            0
        );
        assert_eq!(
            f.copy(
                &[d + 4 * 64, d + 6 * 64],
                Some(&[2, 2]),
                &[s + 2 * 64, s],
                Some(&[2, 2])
            ),
            Ok(())
        );
        let offsets: Vec<_> = f.visible.slots.lock().unwrap()[4..8]
            .iter()
            .map(|v| v.as_ref().unwrap().offset)
            .collect();
        assert_eq!(offsets, [512, 768, 768, 256]);
        assert_eq!(
            f.copy(
                &[d, d + 64, d + 128, d + 192],
                None,
                &[s + 192, s + 128, s + 64, s],
                None
            ),
            Ok(())
        );
        let offsets: Vec<_> = f.visible.slots.lock().unwrap()[..4]
            .iter()
            .map(|v| v.as_ref().unwrap().offset)
            .collect();
        assert_eq!(offsets, [768, 512, 256, 768]);
        let record = unsafe { ptr::read_unaligned(f.visible.storage.contents().cast::<Record>()) };
        assert_eq!(
            (record.tag, record.address, record.length, record.reserved),
            (1, f.base() + 768, 256, [0; 5])
        );
    });
}

#[test]
fn failed_copies_validate_all_sources_before_any_destination_write() {
    autoreleasepool(|| {
        let f = Fixture::new();
        f.initialize(0, 1);
        let d = f.visible.start();
        let s = f.source.start();
        for (dest, ds, src, ss, error) in [
            (vec![d], vec![2], vec![s], vec![2], E_INVALIDARG), // second source uninitialized
            (vec![d], vec![1], vec![s], vec![2], E_INVALIDARG), // totals differ
            (vec![d, d], vec![1, 1], vec![s, s], vec![1, 1], E_INVALIDARG), // duplicate destination
            (vec![d], vec![2], vec![s, s + 512], vec![1, 1], E_INVALIDARG), // later invalid handle
            (vec![s], vec![1], vec![s], vec![1], E_NOTIMPL),    // overlap explicitly unsupported
        ] {
            assert_eq!(f.copy(&dest, Some(&ds), &src, Some(&ss)), Err(error));
            assert!(f.visible.slots.lock().unwrap().iter().all(Option::is_none));
        }
        assert_eq!(unsafe { initialize(f.device, 0, s) }, Err(E_NOTIMPL));
        assert_eq!(
            unsafe { ranges(&(*f.device).descriptors.lock().unwrap(), 1, 0, 0, true) }.err(),
            Some(E_POINTER)
        );
        assert_eq!(
            unsafe { copy(f.device, 0, 0, 0, 0, 0, 0, 1) },
            Err(E_NOTIMPL)
        );
    });
}

#[test]
fn actual_gpu_reads_copied_native_addresses_and_preserves_guard_words() {
    autoreleasepool(|| {
        let f = Fixture::new();
        unsafe {
            for block in 0..4 {
                for word in 0..64 {
                    ptr::write(
                        f.memory
                            .buffer
                            .contents()
                            .cast::<u8>()
                            .add(f.memory.offset as usize)
                            .cast::<u32>()
                            .add(block * 64 + word),
                        (block as u32 + 1) * 100 + word as u32,
                    );
                }
            }
        }
        for i in 0..4 {
            f.initialize(i, i as u64);
        }
        assert_eq!(
            f.copy(
                &[f.visible.start() + 64],
                Some(&[4]),
                &[f.source.start()],
                Some(&[4])
            ),
            Ok(())
        );
        f.initialize(0, 3);
        let mut words = [0u32; 28];
        assert_eq!(
            unsafe { probe(f.device, &f.visible, 1, words.as_mut_ptr(), 28) },
            S_OK
        );
        for block in 0..4 {
            for word in 0..4 {
                assert_eq!(
                    words[block * 4 + word],
                    (block as u32 + 1) * 100 + word as u32
                );
            }
        }
        assert_eq!(&words[16..24], &[0xcdcdcdcd; 8]);
        assert_eq!(&words[24..], &[0; 4]);
        assert!(
            unsafe { (*f.device).descriptors.lock().unwrap() }
                .probe
                .get()
                .unwrap()
                .is_ok()
        );
    });
}

#[test]
fn probe_rejects_uninitialized_ranges_capacity_and_visibility_without_writing_output() {
    autoreleasepool(|| {
        let f = Fixture::new();
        let mut words = [0xa5u32; 24];
        for (heap, first, capacity) in [
            (&f.visible, 0, 24),
            (&f.visible, 0, 23),
            (&f.visible, u32::MAX, 24),
            (&f.source, 0, 24),
        ] {
            assert_eq!(
                unsafe { probe(f.device, heap, first, words.as_mut_ptr(), capacity) },
                E_INVALIDARG
            );
            assert_eq!(words, [0xa5; 24]);
        }
        assert_eq!(
            unsafe { probe(f.device, &f.visible, 0, ptr::null_mut(), 24) },
            E_POINTER
        );
        assert!(
            unsafe { (*f.device).descriptors.lock().unwrap() }
                .probe
                .get()
                .is_none()
        );
        assert_eq!(
            unsafe {
                descriptor_heaps::AlkyholProbeCbvHeap(f.device.cast(), 0, words.as_mut_ptr(), 24)
            },
            E_INVALIDARG
        );
        assert_eq!(
            unsafe {
                descriptor_heaps::AlkyholProbeCbvHeap(ptr::null_mut(), 0, words.as_mut_ptr(), 24)
            },
            E_POINTER
        );
    });
}

#[test]
fn copied_slots_retain_native_memory_after_resource_shell_release_without_registry_cycles() {
    autoreleasepool(|| unsafe {
        let f = Fixture::new();
        let mut raw = ptr::null_mut();
        let heap = HeapProperties {
            kind: 2,
            page: 0,
            pool: 0,
            creation: 1,
            visible: 1,
        };
        let desc = ResourceDesc {
            dimension: 1,
            alignment: 0,
            width: 1024,
            height: 1,
            depth: 1,
            mips: 1,
            format: 0,
            samples: 1,
            quality: 0,
            layout: 1,
            flags: 0,
        };
        assert_eq!(
            committed(
                f.device,
                &heap,
                0,
                &desc,
                0xac3,
                ptr::null(),
                &RESOURCE,
                &mut raw
            ),
            S_OK
        );
        let resource = raw.cast::<Resource>();
        let weak = Arc::downgrade(&(*resource)._descriptor_memory);
        let mut mapped = ptr::null_mut();
        assert_eq!(map(resource, 0, ptr::null(), &mut mapped), S_OK);
        ptr::write_bytes(mapped.cast::<u8>(), 0x17, 1024);
        let cbv = Description {
            address: gpu_address(resource),
            length: 256,
        };
        for i in 0..4 {
            assert_eq!(
                initialize(
                    f.device,
                    (&cbv as *const Description) as usize,
                    f.source.start() + i * 64
                ),
                Ok(())
            );
        }
        assert_eq!(
            f.copy(
                &[f.visible.start()],
                Some(&[4]),
                &[f.source.start()],
                Some(&[4])
            ),
            Ok(())
        );
        assert_eq!(resource_release(resource), 0);
        assert!(weak.upgrade().is_some());
        f.source.slots.lock().unwrap().fill(None);
        let mut words = [0; 24];
        assert_eq!(probe(f.device, &f.visible, 0, words.as_mut_ptr(), 24), S_OK);
        assert_eq!(&words[..16], &[0x17171717; 16]);
        f.visible.slots.lock().unwrap().fill(None);
        assert!(weak.upgrade().is_none()); // Registry entries themselves are weak.
        assert_eq!(
            (*f.device).descriptors.lock().unwrap().cbv(cbv).err(),
            Some(E_INVALIDARG)
        );
    });
}
