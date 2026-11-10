//! Process-owned top-level filter registration. Exception delivery is separate.
use super::{LoadedImage32, unsupported};

fn executable(pointer: u32, regions: &[(usize, usize, u32)]) -> bool {
    pointer == 0
        || regions.iter().any(|&(first, last, rights)| {
            rights & 0x20000000 != 0 && super::host_code_address(pointer) >= first && super::host_code_address(pointer) < last
        })
}
pub(super) fn replace(image: &LoadedImage32<'_>, filter: u32) -> u32 {
    if !executable(filter, &image.regions) {
        unsupported("top-level exception filter outside executable guest storage");
    }
    image.cpu.unhandled_filter.replace(filter)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registration_admits_null_or_actual_executable_extents_only() {
        let regions = [(0x1000, 0x2000, 0x60000000), (0x2000, 0x3000, 0xc0000000)];
        for pointer in [0, 0x1000, 0x1fff] {
            assert!(executable(pointer, &regions));
        }
        for pointer in [1, 0xfff, 0x2000, 0x2fff, 0x3000, u32::MAX] {
            assert!(!executable(pointer, &regions));
        }
    }
}
