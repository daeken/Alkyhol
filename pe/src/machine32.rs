//! Explicit host-backed x86 machine profile and its enforced memory bounds.
use super::{LoadedImage32, unsupported};
use crate::Result;
#[derive(Clone, Copy, Debug)]
pub(super) struct Bounds {
    pub minimum: usize,
    pub end: u64,
}
impl Bounds {
    pub(super) fn contains(self, start: usize, len: usize) -> bool {
        start >= self.minimum
            && (start as u64)
                .checked_add(len as u64)
                .is_some_and(|end| end <= self.end)
    }
}
pub(super) struct Machine {
    pub bounds: Bounds,
    words: [u32; 9],
}
impl Machine {
    fn observed(
        large: bool,
        page: i32,
        count: usize,
        vendor: [u32; 4],
        signature: u32,
    ) -> Result<Self> {
        if page != 4096 || !(1..=32).contains(&count) {
            return Err(
                "host machine profile requires 4 KiB pages and 1..32 active processors".into(),
            );
        }
        let intel = vendor[1..] == [0x756e6547, 0x6c65746e, 0x49656e69];
        let amd = vendor[1..] == [0x68747541, 0x444d4163, 0x69746e65];
        let base_family = (signature >> 8) & 15;
        let family = base_family
            + if base_family == 15 {
                (signature >> 20) & 255
            } else {
                0
            };
        if vendor[0] < 1 || !((intel && family == 6) || (amd && family == 25)) {
            return Err("host x86 processor family outside the admitted machine profile".into());
        }
        let model = ((signature >> 4) & 15)
            | if [6, 15].contains(&base_family) {
                (signature >> 12) & 0xf0
            } else {
                0
            };
        let revision = (model << 8) | (signature & 15);
        let bounds = Bounds {
            minimum: 65536,
            end: if large { 0xffff0000 } else { 0x7fff0000 },
        };
        let mask = if count == 32 {
            u32::MAX
        } else {
            (1u32 << count) - 1
        };
        Ok(Self {
            bounds,
            words: [
                0,
                4096,
                65536,
                (bounds.end - 1) as u32,
                mask,
                count as u32,
                586,
                65536,
                (revision << 16) | family,
            ],
        })
    }
    pub(super) fn host(large: bool) -> Result<Self> {
        let vendor = std::arch::x86_64::__cpuid(0);
        let processor = std::arch::x86_64::__cpuid(1);
        let count = std::thread::available_parallelism()
            .map_err(|e| format!("host processor availability: {e}"))?
            .get();
        Self::observed(
            large,
            unsafe { super::getpagesize() },
            count,
            [vendor.eax, vendor.ebx, vendor.ecx, vendor.edx],
            processor.eax,
        )
    }
}
pub(super) fn information(image: &LoadedImage32<'_>, output: u32) -> u32 {
    let machine =
        image.cpu.machine.as_ref().unwrap_or_else(|| {
            unsupported("GetSystemInfo requires the explicit host-machine profile")
        });
    if !output.is_multiple_of(4) || !image.permits(output as usize, 36, true) {
        unsupported("SYSTEM_INFO outside aligned writable guest storage");
    }
    unsafe {
        std::ptr::copy_nonoverlapping(machine.words.as_ptr(), output as *mut u32, 9);
    }
    0 // void: the complete backed 36-byte structure was written.
}
#[cfg(test)]
mod tests {
    use super::*;
    const INTEL: [u32; 4] = [13, 0x756e6547, 0x6c65746e, 0x49656e69];
    #[test]
    fn observed_cpu_fields_and_laa_memory_ranges_use_defined_widths() {
        let m = Machine::observed(false, 4096, 10, INTEL, 0x206c0).unwrap();
        assert_eq!(
            m.words,
            [0, 4096, 65536, 0x7ffeffff, 1023, 10, 586, 65536, 0x2c000006]
        );
        let m = Machine::observed(true, 4096, 32, INTEL, 0x806e9).unwrap();
        assert_eq!(m.words[3], 0xfffeffff);
        assert_eq!(m.words[4], u32::MAX);
        assert_eq!(m.words[8], 0x8e090006);
        let amd = Machine::observed(
            false,
            4096,
            4,
            [13, 0x68747541, 0x444d4163, 0x69746e65],
            0xa00f11,
        )
        .unwrap();
        assert_eq!(amd.words[8], 0x01010019);
        assert!(m.bounds.contains(0x10000, 4096));
        assert!(m.bounds.contains(0xfffef000, 4096));
        for (p, n) in [(0, 1), (0xffff, 1), (0xffff0000, 1), (usize::MAX, 4096)] {
            assert!(!m.bounds.contains(p, n));
        }
    }
    #[test]
    fn unverified_machine_descriptors_are_not_invented() {
        for (page, count, signature) in [
            (16384, 10, 0x206c0),
            (4096, 0, 0x206c0),
            (4096, 33, 0x206c0),
            (4096, 10, 0x10ff0),
        ] {
            assert!(Machine::observed(false, page, count, INTEL, signature).is_err());
        }
        assert!(Machine::observed(false, 4096, 10, [0, 0, 0, 0], 0x206c0).is_err());
    }
}
