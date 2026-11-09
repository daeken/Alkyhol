//! Actual user-available/total/free bytes on the checked path's host filesystem.
use super::{LoadedImage32, bulk32, directory32, files32, unsupported};
use crate::Result;
use std::ffi::CString;
unsafe extern "C" {
    fn alkyhol_disk_blocks(path: *const std::ffi::c_char, blocks: *mut u64) -> i32;
}
fn sizes(blocks: [u64; 4]) -> Result<[u64; 3]> {
    if blocks[0] == 0 || blocks[1] > blocks[2] || blocks[3] > blocks[2] {
        return Err("filesystem returned unsupported block accounting".into());
    }
    let mut sizes = [0; 3];
    for i in 0..3 {
        sizes[i] = blocks[0]
            .checked_mul(blocks[i + 1])
            .ok_or("disk-space byte count overflow")?;
    }
    Ok(sizes)
}
fn fail(error: u32) -> u32 {
    alkyhol::ffi::SetLastError(error);
    0
}
pub(super) fn query(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    for &address in &a[1..4] {
        if address != 0 && !image.permits(address as usize, 8, true) {
            unsupported("disk-space output outside writable guest storage");
        }
    }
    let path = if a[0] == 0 {
        directory32::current_path(image).unwrap_or_else(|e| unsupported(&e))
    } else {
        directory32::ordinary_path(image, &bulk32::ascii(image, a[0]))
            .unwrap_or_else(|e| unsupported(&e))
    };
    let (host, metadata) = match files32::existing(&path).unwrap_or_else(|e| unsupported(&e)) {
        Ok(value) => value,
        Err(2 | 3) => return fail(3),
        Err(error) => return fail(error),
    };
    if !metadata.is_dir() {
        return fail(267);
    }
    let path = CString::new(host.as_os_str().as_encoded_bytes()).unwrap();
    let mut blocks = [0; 4];
    let status = unsafe { alkyhol_disk_blocks(path.as_ptr(), blocks.as_mut_ptr()) };
    if status != 0 {
        return match status {
            2 | 20 => fail(3),
            13 => fail(5),
            _ => unsupported(&format!(
                "native filesystem space query failed: errno {status}"
            )),
        };
    }
    let sizes = sizes(blocks).unwrap_or_else(|e| unsupported(&e));
    if image.cpu.diagnostic.is_some() {
        eprintln!(
            "PE32 actual filesystem bytes: available={} total={} free={}",
            sizes[0], sizes[1], sizes[2]
        );
    }
    for (&address, size) in a[1..4].iter().zip(sizes) {
        if address != 0 {
            unsafe {
                (address as *mut u64).write_unaligned(size);
            }
        }
    }
    1
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn block_counts_keep_available_total_and_free_distinct_and_checked() {
        assert_eq!(sizes([4096, 3, 7, 5]).unwrap(), [12288, 28672, 20480]);
        assert_eq!(sizes([512, 0, 0, 0]).unwrap(), [0, 0, 0]);
        for blocks in [
            [0, 0, 1, 1],
            [1, 2, 1, 0],
            [1, 0, 1, 2],
            [u64::MAX, 1, 2, 1],
        ] {
            assert!(sizes(blocks).is_err());
        }
    }
    #[test]
    fn actual_native_filesystem_query_returns_real_bounded_accounting() {
        let path = CString::new(
            std::env::current_dir()
                .unwrap()
                .as_os_str()
                .as_encoded_bytes(),
        )
        .unwrap();
        let mut blocks = [0; 4];
        assert_eq!(
            unsafe { alkyhol_disk_blocks(path.as_ptr(), blocks.as_mut_ptr()) },
            0
        );
        let actual = sizes(blocks).unwrap();
        assert!(actual[1] > 0);
        assert!(actual[2] <= actual[1]);
    }
}
