//! Bounded process/private heaps reached by original DLL startup. Handles are
//! inaccessible opaque tokens; 8-byte-aligned blocks live in real low arenas.
use super::{LowMapping, PAGE};
use crate::Result;
use std::collections::BTreeMap;
struct Region {
    memory: LowMapping,
    cursor: usize,
}
struct Block {
    size: usize,
    capacity: usize,
}
struct Object {
    _handle: LowMapping,
    regions: Vec<Region>,
    blocks: BTreeMap<u32, Block>,
    free: Vec<(u32, usize)>,
}
#[derive(Default)]
pub(super) struct Heap {
    default: Option<u32>,
    objects: BTreeMap<u32, Object>,
}
impl Heap {
    pub(super) fn handle(&mut self) -> Result<u32> {
        if let Some(handle) = self.default {
            return Ok(handle);
        }
        let handle = self.create(0, 0, 0)?;
        if handle != 0 {
            self.default = Some(handle);
        }
        Ok(handle)
    }
    pub(super) fn create(&mut self, flags: u32, initial: u32, maximum: u32) -> Result<u32> {
        if flags & !1 != 0 || maximum != 0 {
            return Err("unsupported PE32 HeapCreate flags/fixed maximum".into());
        }
        if initial > 512 * 1024 * 1024 || self.objects.len() >= 64 {
            alkyhol::ffi::SetLastError(8);
            return Ok(0);
        }
        let memory = match LowMapping::new(PAGE, PAGE, 0) {
            Ok(v) => v,
            Err(_) => {
                alkyhol::ffi::SetLastError(8);
                return Ok(0);
            }
        };
        memory.protect(0, PAGE, 0)?;
        let mut regions = Vec::new();
        if initial != 0 {
            let length = (initial as usize).div_ceil(PAGE) * PAGE;
            let region = match LowMapping::new(length, PAGE, 0) {
                Ok(v) => v,
                Err(_) => {
                    alkyhol::ffi::SetLastError(8);
                    return Ok(0);
                }
            };
            regions.push(Region {
                memory: region,
                cursor: 0,
            });
        }
        let handle = memory.base as u32;
        self.objects.insert(
            handle,
            Object {
                _handle: memory,
                regions,
                blocks: BTreeMap::new(),
                free: Vec::new(),
            },
        );
        Ok(handle)
    }
    pub(super) fn allocate(&mut self, heap: u32, flags: u32, size: u32) -> Result<u32> {
        let object = self
            .objects
            .get_mut(&heap)
            .ok_or("unknown PE32 process heap handle")?;
        if flags & !9 != 0 {
            return Err("unsupported PE32 HeapAlloc flags/exception mode".into());
        }
        // HeapAlloc does not set last error on failure. ZERO_MEMORY explicitly
        // clears recycled blocks, rather than assuming every mmap is fresh.
        if size > 512 * 1024 * 1024 {
            return Ok(0);
        }
        let capacity = (size as usize).max(1).div_ceil(8) * 8;
        let address = if let Some(i) = object.free.iter().position(|(_, n)| *n >= capacity) {
            let (address, available) = object.free.swap_remove(i);
            if available > capacity {
                object
                    .free
                    .push((address + capacity as u32, available - capacity));
            }
            address
        } else {
            if !object
                .regions
                .iter()
                .any(|r| r.cursor + capacity <= r.memory.len)
            {
                let length = capacity.max(65536).div_ceil(PAGE) * PAGE;
                let Ok(memory) = LowMapping::new(length, PAGE, 0) else {
                    return Ok(0);
                };
                object.regions.push(Region { memory, cursor: 0 });
            }
            let region = object
                .regions
                .iter_mut()
                .find(|r| r.cursor + capacity <= r.memory.len)
                .expect("owned heap region");
            let address = region.memory.base as usize + region.cursor;
            region.cursor += capacity;
            address as u32
        };
        if flags & 8 != 0 {
            unsafe {
                std::ptr::write_bytes(address as *mut u8, 0, size as usize);
            }
        }
        object.blocks.insert(
            address,
            Block {
                size: size as usize,
                capacity,
            },
        );
        Ok(address)
    }
    pub(super) fn free(&mut self, heap: u32, flags: u32, address: u32) -> Result<u32> {
        let object = self
            .objects
            .get_mut(&heap)
            .ok_or("unknown PE32 process heap handle")?;
        if flags & !1 != 0 {
            return Err("unsupported PE32 HeapFree flags".into());
        }
        if address == 0 {
            return Ok(1);
        }
        let block = object
            .blocks
            .remove(&address)
            .ok_or("PE32 HeapFree requires an owned allocation base")?;
        object.free.push((address, block.capacity));
        Ok(1)
    }
    pub(super) fn permits(&self, address: usize, len: usize) -> bool {
        let Some(end) = address.checked_add(len) else {
            return false;
        };
        self.objects.values().any(|heap| {
            heap.blocks
                .iter()
                .any(|(&first, b)| address >= first as usize && end <= first as usize + b.size)
        })
    }
    pub(super) fn clear(&mut self) {
        self.objects.clear();
        self.default = None;
    }
}
