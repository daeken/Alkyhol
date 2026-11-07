//! Bounded non-null UPLOAD CBVs and transactional descriptor copies.
//! CPU/GPU handles stay opaque; only registered live intervals may be accessed.
use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{OnceLock, Weak},
};
pub(super) struct BufferMemory {
    pub(super) buffer: metal::Buffer,
    pub(super) offset: u64,
    pub(super) width: u64,
    pub(super) heap: i32,
}
#[derive(Clone)]
struct Cbv {
    memory: Arc<BufferMemory>,
    offset: u64,
    length: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Record {
    tag: u64,
    address: u64,
    length: u64,
    reserved: [u64; 5],
}
pub(super) struct HeapData {
    pub(super) storage: metal::Buffer,
    pub(super) kind: i32,
    pub(super) flags: u32,
    pub(super) count: u32,
    slots: Mutex<Vec<Option<Cbv>>>,
}
impl HeapData {
    pub(super) fn new(
        storage: metal::Buffer,
        kind: i32,
        flags: u32,
        count: u32,
    ) -> Result<Self, i32> {
        let mut slots = Vec::new();
        slots
            .try_reserve(count as usize)
            .map_err(|_| E_OUTOFMEMORY)?;
        slots.resize_with(count as usize, || None);
        Ok(Self {
            storage,
            kind,
            flags,
            count,
            slots: Mutex::new(slots),
        })
    }
    fn start(&self) -> usize {
        self.storage.contents() as usize
    }
    fn write(&self, index: usize, value: Cbv) {
        let record = Record {
            tag: 1,
            address: value.memory.buffer.gpu_address() + value.memory.offset + value.offset,
            length: u64::from(value.length),
            reserved: [0; 5],
        };
        unsafe {
            ptr::write_unaligned(
                self.storage
                    .contents()
                    .cast::<u8>()
                    .add(index * 64)
                    .cast::<Record>(),
                record,
            );
        }
        self.slots.lock().unwrap()[index] = Some(value);
    }
}
#[derive(Default)]
pub(super) struct Registry {
    heaps: BTreeMap<usize, Weak<HeapData>>,
    buffers: BTreeMap<u64, Weak<BufferMemory>>,
    probe: OnceLock<Result<metal::ComputePipelineState, String>>,
}
impl Registry {
    pub(super) fn heap(&mut self, data: &Arc<HeapData>) {
        self.heaps.retain(|_, v| v.strong_count() != 0);
        self.heaps.insert(data.start(), Arc::downgrade(data));
    }
    pub(super) fn buffer(&mut self, data: &Arc<BufferMemory>) {
        self.buffers.retain(|_, v| v.strong_count() != 0);
        self.buffers.insert(
            data.buffer.gpu_address() + data.offset,
            Arc::downgrade(data),
        );
    }
    fn span(
        &self,
        handle: usize,
        count: u32,
        source: bool,
    ) -> Result<Vec<(Arc<HeapData>, usize)>, i32> {
        if count == 0 {
            return Err(E_NOTIMPL);
        }
        let (&start, weak) = self
            .heaps
            .range(..=handle)
            .next_back()
            .ok_or(E_INVALIDARG)?;
        let heap = weak.upgrade().ok_or(E_INVALIDARG)?;
        let delta = handle.checked_sub(start).ok_or(E_INVALIDARG)?;
        if !delta.is_multiple_of(64) || heap.kind != 0 {
            return Err(E_INVALIDARG);
        }
        if source && heap.flags != 0 {
            return Err(E_NOTIMPL);
        }
        let index = delta / 64;
        if index
            .checked_add(count as usize)
            .is_none_or(|end| end > heap.count as usize)
        {
            return Err(E_INVALIDARG);
        }
        let mut rows = Vec::new();
        rows.try_reserve(count as usize)
            .map_err(|_| E_OUTOFMEMORY)?;
        for i in index..index + count as usize {
            rows.push((Arc::clone(&heap), i));
        }
        Ok(rows)
    }
    fn cbv(&self, desc: Description) -> Result<Cbv, i32> {
        if desc.address == 0 {
            return Err(E_NOTIMPL);
        }
        if !desc.address.is_multiple_of(256)
            || desc.length == 0
            || !desc.length.is_multiple_of(256)
            || desc.length > 65536
        {
            return Err(E_INVALIDARG);
        }
        let (&start, weak) = self
            .buffers
            .range(..=desc.address)
            .next_back()
            .ok_or(E_INVALIDARG)?;
        let memory = weak.upgrade().ok_or(E_INVALIDARG)?;
        let offset = desc.address.checked_sub(start).ok_or(E_INVALIDARG)?;
        if memory.heap != 2 {
            return Err(E_NOTIMPL);
        }
        if offset
            .checked_add(u64::from(desc.length))
            .is_none_or(|end| end > memory.width)
        {
            return Err(E_INVALIDARG);
        }
        Ok(Cbv {
            memory,
            offset,
            length: desc.length,
        })
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Description {
    address: u64,
    length: u32,
}
unsafe fn initialize(device: *mut Device, desc: usize, handle: usize) -> Result<(), i32> {
    let registry = unsafe { (*device).descriptors.lock().unwrap() };
    let target = registry.span(handle, 1, false)?;
    if desc == 0 {
        return Err(E_NOTIMPL);
    }
    let cbv = registry.cbv(unsafe { *(desc as *const Description) })?;
    target[0].0.write(target[0].1, cbv);
    Ok(())
}
unsafe fn ranges(
    registry: &Registry,
    count: u32,
    starts: usize,
    sizes: usize,
    source: bool,
) -> Result<Vec<(Arc<HeapData>, usize)>, i32> {
    if count == 0 {
        return Err(E_NOTIMPL);
    }
    if starts == 0 {
        return Err(E_POINTER);
    }
    let mut result = Vec::new();
    for i in 0..count as usize {
        let start = unsafe { *((starts as *const usize).add(i)) };
        let size = if sizes == 0 {
            1
        } else {
            unsafe { *((sizes as *const u32).add(i)) }
        };
        let rows = registry.span(start, size, source)?;
        result.try_reserve(rows.len()).map_err(|_| E_OUTOFMEMORY)?;
        result.extend(rows);
    }
    Ok(result)
}
#[allow(clippy::too_many_arguments)] // Mirrors the D3D12 descriptor-copy ABI.
unsafe fn copy(
    device: *mut Device,
    dcount: u32,
    dstarts: usize,
    dsizes: usize,
    scount: u32,
    sstarts: usize,
    ssizes: usize,
    kind: i32,
) -> Result<(), i32> {
    if kind != 0 {
        return Err(E_NOTIMPL);
    }
    let registry = unsafe { (*device).descriptors.lock().unwrap() };
    let destinations = unsafe { ranges(&registry, dcount, dstarts, dsizes, false) }?;
    let sources = unsafe { ranges(&registry, scount, sstarts, ssizes, true) }?;
    if destinations.len() != sources.len() {
        return Err(E_INVALIDARG);
    }
    let mut addresses = BTreeSet::new();
    for (heap, index) in &destinations {
        if !addresses.insert((heap.start(), *index)) {
            return Err(E_INVALIDARG);
        }
    }
    let mut values = Vec::new();
    values
        .try_reserve(sources.len())
        .map_err(|_| E_OUTOFMEMORY)?;
    for (heap, index) in &sources {
        if addresses.contains(&(heap.start(), *index)) {
            return Err(E_NOTIMPL);
        }
        values.push(
            heap.slots.lock().unwrap()[*index]
                .clone()
                .ok_or(E_INVALIDARG)?,
        );
    }
    // Everything, including all source slots, is validated before any write.
    for ((heap, index), value) in destinations.into_iter().zip(values) {
        heap.write(index, value);
    }
    Ok(())
}
fn checked(result: Result<(), i32>, name: &str) {
    if let Err(error) = result {
        eprintln!("Alkyhol {name} rejected: 0x{:08x}", error as u32);
        unsupported_void(name);
    }
}
pub(super) unsafe extern "C" fn create_cbv(device: *mut Device, desc: usize, handle: usize) {
    checked(
        unsafe { initialize(device, desc, handle) },
        "constant-buffer view",
    );
}
pub(super) unsafe extern "C" fn copy_descriptors(
    device: *mut Device,
    dcount: u32,
    dstarts: usize,
    dsizes: usize,
    scount: u32,
    sstarts: usize,
    ssizes: usize,
    kind: i32,
) {
    checked(
        unsafe {
            copy(
                device, dcount, dstarts, dsizes, scount, sstarts, ssizes, kind,
            )
        },
        "descriptor copy",
    );
}
pub(super) unsafe extern "C" fn copy_simple(
    device: *mut Device,
    count: u32,
    destination: usize,
    source: usize,
    kind: i32,
) {
    checked(
        unsafe {
            copy(
                device,
                1,
                (&destination as *const usize) as usize,
                (&count as *const u32) as usize,
                1,
                (&source as *const usize) as usize,
                (&count as *const u32) as usize,
                kind,
            )
        },
        "simple descriptor copy",
    );
}
// Explicit bounded evidence API, not GraphicsCommandList::Dispatch or root tables.
pub(super) unsafe fn probe(
    device: *mut Device,
    data: &HeapData,
    first: u32,
    out: *mut u32,
    capacity: u32,
) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    if capacity < 24
        || data.kind != 0
        || data.flags != 1
        || first.checked_add(4).is_none_or(|end| end > data.count)
    {
        return E_INVALIDARG;
    }
    autoreleasepool(|| {
        let registry = unsafe { (*device).descriptors.lock().unwrap() };
        let slots = data.slots.lock().unwrap();
        let mut resources = Vec::new();
        for slot in &slots[first as usize..first as usize + 4] {
            let Some(cbv) = slot else {
                return E_INVALIDARG;
            };
            resources.push(Arc::clone(&cbv.memory));
        }
        let pipeline = match registry.probe.get_or_init(|| {
            let options = metal::CompileOptions::new();
            options.set_language_version(metal::MTLLanguageVersion::V3_0);
            let library = unsafe {
                (*device)
                    .metal
                    .new_library_with_source(include_str!("cbv_probe.metal"), &options)
            }?;
            let function = library.get_function("read_cbvs", None)?;
            unsafe {
                (*device)
                    .metal
                    .new_compute_pipeline_state_with_function(&function)
            }
        }) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Alkyhol CBV evidence kernel failed: {e}");
                return 0x80004005u32 as i32;
            }
        };
        let output = unsafe {
            (*device)
                .metal
                .new_buffer(96, metal::MTLResourceOptions::StorageModeShared)
        };
        unsafe {
            ptr::write_bytes(output.contents().cast::<u8>(), 0xcd, 96);
        }
        let queue = unsafe { (*device).metal.new_command_queue() };
        let command = queue.new_command_buffer();
        let encoder = command.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(pipeline);
        encoder.set_buffer(0, Some(&data.storage), u64::from(first) * 64);
        encoder.set_buffer(1, Some(&output), 0);
        for resource in &resources {
            encoder.use_resource(&resource.buffer, metal::MTLResourceUsage::Read);
        }
        encoder.dispatch_thread_groups(metal::MTLSize::new(1, 1, 1), metal::MTLSize::new(4, 1, 1));
        encoder.end_encoding();
        command.commit();
        command.wait_until_completed();
        if command.status() != metal::MTLCommandBufferStatus::Completed {
            return 0x80004005u32 as i32;
        }
        unsafe {
            ptr::copy_nonoverlapping(output.contents().cast::<u32>(), out, 24);
        }
        S_OK
    })
}

#[cfg(test)]
#[path = "cbv_descriptors_tests.rs"]
mod tests;
