//! Explicit AMD64 dynamic function-table bookkeeping and lookup only.
//! No unwind-code interpretation, OS stack registration or exception dispatch.
#![allow(non_snake_case)]
use alkyhol::ffi;
use std::{ffi::c_void, ptr};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct RuntimeFunction {
    pub begin: u32,
    pub end: u32,
    pub unwind: u32,
}
struct Table {
    original: usize,
    base: u64,
    first: u64,
    last: u64,
    entries: Vec<RuntimeFunction>,
}
#[derive(Default)]
pub(crate) struct Registry {
    tables: Vec<Table>,
    entries: usize,
}
fn fail(error: u32) -> u8 {
    ffi::SetLastError(error);
    0
}
fn unsupported(reason: &str) -> ! {
    eprintln!("Alkyhol PE function-table lookup unsupported: {reason}");
    std::process::exit(78)
}
impl Registry {
    // Caller owns immutable readable records and their code/unwind storage
    // until deletion. The returned pointers identify the ORIGINAL records.
    unsafe fn add(&mut self, original: *const RuntimeFunction, count: u32, base: u64) -> u8 {
        let count = count as usize;
        if count > 65536 || self.entries + count > 65536 || self.tables.len() == 64 {
            return fail(50);
        }
        if original.is_null()
            || !(original as usize).is_multiple_of(4)
            || count == 0
            || (original as usize)
                .checked_add(count * std::mem::size_of::<RuntimeFunction>())
                .is_none()
            || self.tables.iter().any(|t| t.original == original as usize)
        {
            return fail(87);
        }
        let mut entries = Vec::new();
        if entries.try_reserve_exact(count).is_err() || self.tables.try_reserve(1).is_err() {
            return fail(8);
        }
        entries.extend_from_slice(unsafe { std::slice::from_raw_parts(original, count) });
        let mut end = 0;
        for e in &entries {
            if e.begin >= e.end
                || e.begin < end
                || base.checked_add(u64::from(e.end)).is_none()
                || base.checked_add(u64::from(e.unwind)).is_none()
            {
                return fail(87);
            }
            // Only ordinary direct, DWORD-aligned unwind RVAs. Indirect/chained
            // entry forms and zero offsets are outside this finite subset.
            if e.unwind == 0 || e.unwind & 3 != 0 {
                return fail(50);
            }
            end = e.end;
        }
        let first = base + u64::from(entries[0].begin);
        let last = base + u64::from(end);
        if self.tables.iter().any(|t| first < t.last && t.first < last) {
            return fail(50);
        }
        self.entries += count;
        self.tables.push(Table {
            original: original as usize,
            base,
            first,
            last,
            entries,
        });
        1
    }
    fn delete(&mut self, original: *const RuntimeFunction) -> u8 {
        let Some(index) = self
            .tables
            .iter()
            .position(|t| t.original == original as usize)
        else {
            return 0;
        };
        self.entries -= self.tables.swap_remove(index).entries.len();
        1
    }
    fn lookup(&self, pc: u64) -> Option<(*const RuntimeFunction, u64)> {
        for t in &self.tables {
            if pc < t.first || pc >= t.last {
                continue;
            }
            let relative = pc - t.base;
            let index = t
                .entries
                .partition_point(|e| u64::from(e.begin) <= relative)
                .checked_sub(1)?;
            if relative < u64::from(t.entries[index].end) {
                let record = (t.original as *const RuntimeFunction).wrapping_add(index);
                return Some((record, t.base));
            }
        }
        None
    }
    pub(crate) fn selected(
        &self,
        pointer: *const RuntimeFunction,
        base: u64,
        pc: u64,
    ) -> Option<RuntimeFunction> {
        for table in &self.tables {
            if table.base != base {
                continue;
            }
            let Some(offset) = (pointer as usize).checked_sub(table.original) else {
                continue;
            };
            let stride = std::mem::size_of::<RuntimeFunction>();
            if !offset.is_multiple_of(stride) {
                continue;
            }
            let Some(entry) = table.entries.get(offset / stride) else {
                continue;
            };
            if pc >= base + u64::from(entry.begin) && pc < base + u64::from(entry.end) {
                return Some(*entry);
            }
        }
        None
    }
}
unsafe extern "win64" fn RtlAddFunctionTable(
    table: *const RuntimeFunction,
    count: u32,
    base: u64,
) -> u8 {
    crate::process::with_function_tables(|registry| match registry {
        Some(registry) => unsafe { registry.add(table, count, base) },
        None => fail(50),
    })
}
unsafe extern "win64" fn RtlDeleteFunctionTable(table: *const RuntimeFunction) -> u8 {
    crate::process::with_function_tables(|registry| match registry {
        Some(registry) => registry.delete(table),
        None => fail(50),
    })
}
unsafe extern "win64" fn RtlLookupFunctionEntry(
    pc: u64,
    image_base: *mut u64,
    history: *mut c_void,
) -> *const RuntimeFunction {
    if image_base.is_null() {
        unsupported("null image-base output");
    }
    if !history.is_null() {
        unsupported("history tables");
    }
    crate::process::with_function_tables(|registry| {
        let Some(registry) = registry else {
            unsupported("no process context");
        };
        match registry.lookup(pc) {
            Some((record, base)) => {
                unsafe {
                    image_base.write_unaligned(base);
                }
                record
            }
            None => ptr::null(), // ImageBase on a miss is not native parity evidence.
        }
    })
}
pub(crate) fn address(name: &str) -> Option<usize> {
    match name {
        "RtlAddFunctionTable" => Some(RtlAddFunctionTable as *const () as usize),
        "RtlDeleteFunctionTable" => Some(RtlDeleteFunctionTable as *const () as usize),
        "RtlLookupFunctionEntry" => Some(RtlLookupFunctionEntry as *const () as usize),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context() -> crate::ProcessContext {
        crate::ProcessContext::install(&["function-table-probe".into()], &[]).unwrap()
    }
    fn bridges() -> (
        unsafe extern "win64" fn(*const RuntimeFunction, u32, u64) -> u8,
        unsafe extern "win64" fn(*const RuntimeFunction) -> u8,
        unsafe extern "win64" fn(u64, *mut u64, *mut c_void) -> *const RuntimeFunction,
    ) {
        let add: unsafe extern "win64" fn(*const RuntimeFunction, u32, u64) -> u8 = unsafe {
            std::mem::transmute(crate::abi::resolve("kernel32.dll", "RtlAddFunctionTable").unwrap())
        };
        let delete: unsafe extern "win64" fn(*const RuntimeFunction) -> u8 = unsafe {
            std::mem::transmute(
                crate::abi::resolve("kernel32.dll", "RtlDeleteFunctionTable").unwrap(),
            )
        };
        let lookup: unsafe extern "win64" fn(u64, *mut u64, *mut c_void) -> *const RuntimeFunction = unsafe {
            std::mem::transmute(
                crate::abi::resolve("kernel32.dll", "RtlLookupFunctionEntry").unwrap(),
            )
        };
        (add, delete, lookup)
    }
    fn entry(begin: u32, end: u32) -> RuntimeFunction {
        RuntimeFunction {
            begin,
            end,
            unwind: 0x200,
        }
    }
    #[test]
    fn win64_tables_preserve_original_records_full_width_base_boundaries_and_deletion() {
        let _context = context();
        let (add, delete, lookup) = bridges();
        let rows = [entry(64, 74), entry(96, 106)];
        let other = [entry(128, 138)];
        let base = 0x1234500000u64;
        let mut image = 0xdeadbeef;
        assert_eq!(std::mem::size_of::<RuntimeFunction>(), 12);
        assert_eq!(std::mem::align_of::<RuntimeFunction>(), 4);
        unsafe {
            assert!(lookup(base + 64, &mut image, ptr::null_mut()).is_null());
            assert_eq!(add(rows.as_ptr(), 2, base), 1);
            assert_eq!(add(other.as_ptr(), 1, base), 1);
            for (pc, index) in [(64, 0), (73, 0), (96, 1), (105, 1)] {
                let actual = lookup(base + pc, &mut image, ptr::null_mut());
                assert_eq!(actual, rows.as_ptr().add(index));
                assert_eq!(image, base);
                assert_eq!((*actual).begin, rows[index].begin);
            }
            for pc in [
                base - 1,
                base + 63,
                base + 74,
                base + 80,
                base + 106,
                base + 138,
                (base + 64) as u32 as u64,
            ] {
                image = 0xfeed;
                assert!(lookup(pc, &mut image, ptr::null_mut()).is_null());
                assert_eq!(image, 0xfeed); // Host miss-output contract, not native parity.
            }
            assert_eq!(delete(rows.as_ptr()), 1);
            assert!(lookup(base + 64, &mut image, ptr::null_mut()).is_null());
            assert_eq!(
                lookup(base + 128, &mut image, ptr::null_mut()),
                other.as_ptr()
            );
            assert_eq!(delete(rows.as_ptr()), 0);
            assert_eq!(add(rows.as_ptr(), 2, base), 1);
            assert_eq!(
                lookup(base + 100, &mut image, ptr::null_mut()),
                rows.as_ptr().add(1)
            );
            assert_eq!(delete(rows.as_ptr()), 1);
            assert_eq!(delete(other.as_ptr()), 1);
        }
    }
    #[test]
    fn unwind_selection_requires_original_record_identity_full_base_and_matching_half_open_range() {
        let mut registry = Registry::default();
        let rows = [entry(16, 24), entry(32, 48)];
        let original = rows.as_ptr();
        let base = 0x123456780000u64;
        assert_eq!(unsafe { registry.add(original, 2, base) }, 1);
        for (index, pc) in [(0, 16), (0, 23), (1, 32), (1, 47)] {
            let record = original.wrapping_add(index);
            let selected = registry.selected(record, base, base + pc).unwrap();
            assert_eq!(selected.begin, rows[index].begin);
            assert_eq!(selected.end, rows[index].end);
            assert_eq!(selected.unwind, rows[index].unwind);
        }
        let copy = rows;
        for pointer in [
            ptr::null(),
            original.wrapping_sub(1),
            original.wrapping_add(2),
            original.cast::<u8>().wrapping_add(4).cast(),
            copy.as_ptr(),
        ] {
            assert!(registry.selected(pointer, base, base + 16).is_none());
        }
        for pc in [base - 1, base + 15, base + 24, base + 32, base + 48] {
            assert!(registry.selected(original, base, pc).is_none());
        }
        assert!(
            registry
                .selected(original, base as u32 as u64, base + 16)
                .is_none()
        );
        assert!(registry.selected(original, base + 1, base + 16).is_none());
        assert_eq!(registry.delete(original), 1);
        assert!(registry.selected(original, base, base + 16).is_none());
    }
    #[test]
    fn metadata_bounds_and_rejected_shapes_do_not_corrupt_registered_entries() {
        let mut registry = Registry::default();
        let rows = [entry(16, 32)];
        unsafe {
            for (pointer, count, base) in [
                (ptr::null(), 1, 0),
                (rows.as_ptr(), 0, 0),
                (rows.as_ptr(), 65537, 0),
                ((usize::MAX - 3) as *const RuntimeFunction, 1, 0),
                (rows.as_ptr(), 1, u64::MAX - 8),
            ] {
                assert_eq!(registry.add(pointer, count, base), 0);
            }
            for rows in [
                [entry(16, 16), entry(32, 40)],
                [entry(32, 40), entry(16, 24)],
                [entry(16, 40), entry(24, 48)],
                [
                    RuntimeFunction {
                        unwind: 1,
                        ..entry(16, 24)
                    },
                    entry(32, 40),
                ],
                [
                    RuntimeFunction {
                        unwind: 0,
                        ..entry(16, 24)
                    },
                    entry(32, 40),
                ],
            ] {
                assert_eq!(registry.add(rows.as_ptr(), 2, 0x1000), 0);
            }
            assert_eq!(registry.add(rows.as_ptr(), 1, 0x1000), 1);
            assert_eq!(registry.add(rows.as_ptr(), 1, 0x2000), 0); // Duplicate original identity.
            let overlap = [entry(24, 40)];
            assert_eq!(registry.add(overlap.as_ptr(), 1, 0x1000), 0);
            assert_eq!(registry.lookup(0x1018).unwrap().0, rows.as_ptr());
            assert_eq!(registry.delete(rows.as_ptr()), 1);
            let many: Vec<_> = (0..65536).map(|i| entry(i * 16, i * 16 + 8)).collect();
            assert_eq!(registry.add(many.as_ptr(), 65536, 0x100000000), 1);
            assert_eq!(registry.add(rows.as_ptr(), 1, 0x400000000), 0);
            assert_eq!(registry.delete(many.as_ptr()), 1);
            let tables = vec![entry(16, 32); 65];
            for i in 0..64 {
                assert_eq!(
                    registry.add(tables.as_ptr().add(i), 1, 0x100000000 + i as u64 * 0x10000),
                    1
                );
            }
            assert_eq!(registry.add(tables.as_ptr().add(64), 1, 0x200000000), 0);
            for i in 0..64 {
                assert_eq!(registry.delete(tables.as_ptr().add(i)), 1);
            }
            assert_eq!(registry.entries, 0);
        }
    }
    #[test]
    fn registrations_are_context_owned_and_do_not_leak_to_host_threads() {
        let (add, delete, lookup) = bridges();
        let rows = [entry(16, 32)];
        assert_eq!(unsafe { add(rows.as_ptr(), 1, 0x1000) }, 0);
        assert_eq!(ffi::GetLastError(), 50);
        let scope = context();
        assert_eq!(unsafe { add(rows.as_ptr(), 1, 0x1000) }, 1);
        std::thread::spawn(|| {
            let (add, delete, _) = bridges();
            let rows = [entry(16, 32)];
            assert_eq!(unsafe { add(rows.as_ptr(), 1, 0x1000) }, 0);
            assert_eq!(ffi::GetLastError(), 50);
            assert_eq!(unsafe { delete(rows.as_ptr()) }, 0);
        })
        .join()
        .unwrap();
        let mut base = 0;
        assert_eq!(
            unsafe { lookup(0x1010, &mut base, ptr::null_mut()) },
            rows.as_ptr()
        );
        drop(scope); // Drops snapshots only; no access to original guest records.
        let _fresh = context();
        assert!(unsafe { lookup(0x1010, &mut base, ptr::null_mut()) }.is_null());
        assert_eq!(unsafe { delete(rows.as_ptr()) }, 0);
        assert_eq!(unsafe { add(rows.as_ptr(), 1, 0x1000) }, 1);
        assert_eq!(unsafe { delete(rows.as_ptr()) }, 1);
    }
    #[test]
    fn unsupported_lookup_shapes_fail_loudly_instead_of_returning_leaf_success() {
        const CHILD: &str = "ALKYHOL_PE_FUNCTION_TABLE_CHILD";
        if let Ok(mode) = std::env::var(CHILD) {
            let _scope = (mode != "no-context").then(context);
            let mut base = 0;
            unsafe {
                RtlLookupFunctionEntry(
                    0x1000,
                    if mode == "null-base" {
                        ptr::null_mut()
                    } else {
                        &mut base
                    },
                    if mode == "history" {
                        ptr::dangling_mut()
                    } else {
                        ptr::null_mut()
                    },
                );
            }
            panic!("unsupported lookup returned as a supported miss");
        }
        for (mode, reason) in [
            ("no-context", "no process context"),
            ("null-base", "null image-base output"),
            ("history", "history tables"),
        ] {
            let run=std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact","function_tables::tests::unsupported_lookup_shapes_fail_loudly_instead_of_returning_leaf_success","--nocapture"])
                .env(CHILD,mode).output().unwrap();
            assert_eq!(run.status.code(), Some(78));
            assert!(String::from_utf8_lossy(&run.stderr).contains(&format!(
                "Alkyhol PE function-table lookup unsupported: {reason}"
            )));
        }
    }
}
