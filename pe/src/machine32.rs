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
        #[cfg(target_arch = "aarch64")]
        let start = super::guest_address(start);
        start >= self.minimum
            && (start as u64)
                .checked_add(len as u64)
                .is_some_and(|end| end <= self.end)
    }
}
// These instruction facilities use real Rosetta/x86 CPUID, including the
// separate extended leaf for AMD 3DNow. No general feature catalog is invented.
fn instruction_feature(feature: u32, basic: u32, extended: Option<u32>) -> Result<u32> {
    let bit = match feature {
        0 if basic & (1 << 26) != 0 => return Ok(0), // Historical Pentium FDIV erratum excluded by SSE2 domain.
        3 => basic & (1 << 23),                      // MMX
        6 => basic & (1 << 25), // SSE, with OS XMM state support in the admitted x86-64 host.
        7 => extended.unwrap_or(0) & (1 << 31), // 3DNow, never confused with SSE2.
        10 => basic & (1 << 26),
        _ => return Err(format!("PE32 processor feature {feature} is unsupported")),
    };
    Ok(u32::from(bit != 0))
}
#[cfg(target_arch = "aarch64")]
pub(super) fn processor_feature(feature: u32) -> Result<u32> {
    // Initial declared scalar emulated CPU. Do not advertise unimplemented SIMD.
    if feature == 0 {
        return Ok(0);
    } // This virtual CPU does not emulate the Pentium FDIV erratum.
    let basic = match std::env::var("ALKYHOL_JIT32_EXPERIMENTAL_SSE2") {
        Err(std::env::VarError::NotPresent) => 0,
        Ok(v) if v == "1" => (1 << 25) | (1 << 26),
        _ => return Err("invalid experimental SSE2 profile setting".into()),
    };
    instruction_feature(feature, basic, Some(0))
}
#[cfg(target_arch = "x86_64")]
pub(super) fn processor_feature(feature: u32) -> Result<u32> {
    let basic = std::arch::x86_64::__cpuid(1).edx;
    let max = std::arch::x86_64::__cpuid(0x80000000).eax;
    let extended = (max >= 0x80000001).then(|| std::arch::x86_64::__cpuid(0x80000001).edx);
    instruction_feature(feature, basic, extended)
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
        if vendor[0] < 1 || !((intel && family == 6) || (amd && [25, 26].contains(&family))) {
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
    #[cfg(target_arch = "aarch64")]
    pub(super) fn host(large: bool) -> Result<Self> {
        // Explicit emulated x86 family-6, one logical processor, 4KiB guest
        // pages. These are guest settings, not observations of an Intel host.
        Self::observed(
            large,
            4096,
            1,
            [1, 0x756e6547, 0x6c65746e, 0x49656e69],
            0x00000600,
        )
    }
    #[cfg(target_arch = "x86_64")]
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
// The explicit host-machine profile numbers the actually available processors
// 0..N. No guest affinity restrictions have been installed in this profile.
pub(super) fn affinity(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    if args[0] != u32::MAX {
        alkyhol::ffi::SetLastError(6);
        return 0;
    }
    let machine = image.cpu.machine.as_ref().unwrap_or_else(|| {
        unsupported("affinity query requires the explicit host-machine profile")
    });
    for &out in &args[1..3] {
        if !out.is_multiple_of(4) || !image.permits(out as usize, 4, true) {
            unsupported("affinity mask outside checked writable guest storage");
        }
    }
    unsafe {
        (args[1] as *mut u32).write(machine.words[4]);
        (args[2] as *mut u32).write(machine.words[4]);
    }
    1
}

pub(super) fn set_affinity(image: &LoadedImage32<'_>, args: &[u32; 8], thread: bool) -> u32 {
    let valid = if thread {
        args[0] == 0xfffffffe || image.cpu.threads.borrow().contains(args[0])
    } else {
        args[0] == u32::MAX
    };
    if !valid {
        alkyhol::ffi::SetLastError(6);
        return 0;
    }
    let machine = image.cpu.machine.as_ref().unwrap_or_else(|| {
        unsupported("affinity setter requires the explicit host-machine profile")
    });
    alkyhol::ffi::SetLastError(if args[1] == 0 || args[1] & !machine.words[4] != 0 {
        87
    } else {
        50
    });
    // A valid exact processor pin has no public macOS equivalent. Do not return
    // a fabricated old mask or claim that a native affinity tag is a CPU pin.
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_instruction_features_keep_basic_and_extended_leaves_separate() {
        for (feature, bit) in [(3, 23), (6, 25), (10, 26)] {
            assert_eq!(instruction_feature(feature, 0, Some(u32::MAX)).unwrap(), 0);
            assert_eq!(instruction_feature(feature, 1 << bit, None).unwrap(), 1);
        }
        assert_eq!(instruction_feature(7, u32::MAX, None).unwrap(), 0);
        assert_eq!(instruction_feature(7, u32::MAX, Some(0)).unwrap(), 0);
        assert_eq!(instruction_feature(7, 0, Some(1 << 31)).unwrap(), 1);
        assert_eq!(instruction_feature(0, 1 << 26, None).unwrap(), 0);
        for feature in [0, 1, 2, 4, 8, 9, 11, u32::MAX] {
            assert!(instruction_feature(feature, 0, None).is_err());
        }
    }
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
        let next = Machine::observed(
            false,
            4096,
            4,
            [13, 0x68747541, 0x444d4163, 0x69746e65],
            0xb00f21,
        )
        .unwrap();
        assert_eq!(next.words[8], 0x0201001a);
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
