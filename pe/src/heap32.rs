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
    lock: super::sync32::Section,
    serialized: bool,
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
                lock: super::sync32::Section::new(0),
                serialized: flags & 1 == 0,
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
    pub(super) fn reallocate(
        &mut self,
        heap: u32,
        flags: u32,
        address: u32,
        size: u32,
    ) -> Result<u32> {
        if flags & !0x19 != 0 {
            return Err("unsupported PE32 HeapReAlloc flags/exception mode".into());
        }
        let block = self
            .objects
            .get(&heap)
            .ok_or("unknown PE32 reallocation heap handle")?
            .blocks
            .get(&address)
            .ok_or("PE32 HeapReAlloc requires a live owned allocation base")?;
        let old_size = block.size;
        if size > 512 * 1024 * 1024 {
            // The owned native impossible-size sequence observes error 8.
            // Other budget failures retain the prior bounded allocation profile.
            if size == u32::MAX {
                alkyhol::ffi::SetLastError(8);
            }
            return Ok(0); // The original allocation remains live and unchanged.
        }
        if size as usize <= block.capacity {
            if flags & 8 != 0 && size as usize > old_size {
                unsafe {
                    std::ptr::write_bytes(
                        (address as *mut u8).add(old_size),
                        0,
                        size as usize - old_size,
                    );
                }
            }
            self.objects
                .get_mut(&heap)
                .expect("checked heap")
                .blocks
                .get_mut(&address)
                .expect("checked block")
                .size = size as usize;
            return Ok(address);
        }
        if flags & 0x10 != 0 {
            return Ok(0); // Owned capacity cannot satisfy this in-place-only growth.
        }
        let replacement = self.allocate(heap, flags & 9, size)?;
        if replacement == 0 {
            return Ok(0);
        }
        unsafe {
            std::ptr::copy_nonoverlapping(address as *const u8, replacement as *mut u8, old_size);
        }
        self.free(heap, flags & 1, address)?;
        Ok(replacement)
    }
    pub(super) fn lock(&mut self, heap: u32, thread: u32, enter: bool) -> Result<u32> {
        let object = self
            .objects
            .get_mut(&heap)
            .ok_or("unknown PE32 heap lock handle")?;
        if !object.serialized {
            return Err("PE32 heap lock requires a serialized owned heap".into());
        }
        if enter {
            object.lock.enter(thread)?;
        } else {
            object.lock.leave(thread)?;
        }
        Ok(1) // Real owned recursion/ownership state changed.
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
    pub(super) fn size(&self, heap: u32, flags: u32, address: u32) -> Result<u32> {
        if flags & !1 != 0 {
            return Err("unsupported PE32 HeapSize flags".into());
        }
        let object = self
            .objects
            .get(&heap)
            .ok_or("unknown PE32 heap size handle")?;
        let block = object
            .blocks
            .get(&address)
            .ok_or("PE32 HeapSize requires a live owned allocation base")?;
        // Windows reports the requested size, not this allocator's alignment
        // padding. Successful size queries preserve the caller's last error.
        Ok(block.size as u32)
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn size_uses_requested_extent_and_rejects_interior_foreign_and_freed_bases() {
        // Metadata-only ownership test; no fabricated mapping is dereferenced.
        let mut heap = std::mem::ManuallyDrop::new(Heap::default());
        let object = Object {
            _handle: LowMapping {
                base: 0x10000usize as *mut u8,
                len: 0,
            },
            regions: Vec::new(),
            blocks: BTreeMap::from([
                (
                    0x20000,
                    Block {
                        size: 37,
                        capacity: 40,
                    },
                ),
                (
                    0x30000,
                    Block {
                        size: 0,
                        capacity: 8,
                    },
                ),
            ]),
            free: Vec::new(),
            lock: super::super::sync32::Section::new(0),
            serialized: true,
        };
        heap.objects.insert(0x10000, object);
        assert!(heap.lock(0x10000, 7, false).is_err());
        assert_eq!(heap.lock(0x10000, 7, true).unwrap(), 1);
        assert_eq!(heap.lock(0x10000, 7, true).unwrap(), 1);
        assert!(heap.lock(0x10000, 8, true).is_err());
        assert!(heap.lock(0x10000, 8, false).is_err());
        heap.lock(0x10000, 7, false).unwrap();
        assert!(heap.lock(0x10000, 8, true).is_err());
        heap.lock(0x10000, 7, false).unwrap();
        assert!(heap.lock(0x10001, 7, true).is_err());
        heap.objects.get_mut(&0x10000).unwrap().serialized = false;
        assert!(heap.lock(0x10000, 7, true).is_err());
        heap.objects.get_mut(&0x10000).unwrap().serialized = true;
        let error = alkyhol::ffi::GetLastError();
        alkyhol::ffi::SetLastError(0x1234);
        assert_eq!(heap.size(0x10000, 0, 0x20000).unwrap(), 37);
        assert_eq!(heap.size(0x10000, 1, 0x30000).unwrap(), 0);
        // Rejected relocation must retain size/ownership and never touch memory.
        assert_eq!(heap.reallocate(0x10000, 16, 0x20000, 41).unwrap(), 0);
        assert_eq!(alkyhol::ffi::GetLastError(), 0x1234);
        assert_eq!(heap.reallocate(0x10000, 0, 0x20000, u32::MAX).unwrap(), 0);
        assert_eq!(heap.size(0x10000, 0, 0x20000).unwrap(), 37);
        assert!(heap.reallocate(0x10000, 4, 0x20000, 41).is_err());
        assert!(heap.reallocate(0x10000, 0, 0x20001, 41).is_err());
        assert!(heap.reallocate(0x10001, 0, 0x20000, 41).is_err());
        assert_eq!(alkyhol::ffi::GetLastError(), 8);
        for (h, f, p) in [
            (0x10000, 0, 0x20001),
            (0x10001, 0, 0x20000),
            (0x10000, 8, 0x20000),
        ] {
            assert!(heap.size(h, f, p).is_err());
        }
        heap.objects
            .get_mut(&0x10000)
            .unwrap()
            .blocks
            .remove(&0x20000);
        assert!(heap.size(0x10000, 0, 0x20000).is_err());
        assert!(heap.reallocate(0x10000, 0, 0x20000, 41).is_err());
        alkyhol::ffi::SetLastError(error);
    }
}
