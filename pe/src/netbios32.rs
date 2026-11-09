//! No NetBIOS transport is installed. A synchronous adapter reset fails honestly.
//! This is not LAN enumeration, hardware identity or an implemented transport.
use super::{LoadedImage32, unsupported};
use crate::Result;
const NRC_OPENERR: u8 = 0x3f;
fn unavailable(ncb: &mut [u8; 64]) -> Result<u32> {
    // Win32 NCB: command 0, return 1, post 44, lana 48, completion 49, event 60.
    // Only the original caller's synchronous NCBRESET is admitted. No async
    // request is accepted and no completion routine/event is fabricated.
    if ncb[0] != 0x32 || ncb[44..48] != [0; 4] || ncb[60..64] != [0; 4] {
        return Err(
            "Netbios supports only an explicit unavailable synchronous adapter reset".into(),
        );
    }
    ncb[1] = NRC_OPENERR;
    ncb[49] = NRC_OPENERR;
    Ok(NRC_OPENERR.into())
}
pub(super) fn reset_unavailable(image: &LoadedImage32<'_>, address: u32) -> u32 {
    if !image.permits(address as usize, 64, true) {
        unsupported("Netbios NCB outside checked writable guest storage");
    }
    let ncb = unsafe { &mut *(address as *mut [u8; 64]) };
    let result = unavailable(ncb).unwrap_or_else(|e| unsupported(&e));
    eprintln!("PE32 Netbios NCBRESET unavailable: NRC_OPENERR (63); no NetBIOS transport");
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absent_transport_fails_without_inventing_adapter_data_or_async_completion() {
        let mut original = [0xa5; 64];
        original[0] = 0x32;
        original[44..48].fill(0);
        original[60..64].fill(0);
        let mut ncb = original;
        assert_eq!(unavailable(&mut ncb).unwrap(), 63);
        for i in 0..64 {
            assert_eq!(ncb[i], if i == 1 || i == 49 { 63 } else { original[i] });
        }
        for offset in [0, 44, 60] {
            let mut ncb = original;
            ncb[offset] = if offset == 0 { 0xb2 } else { 1 };
            let before = ncb;
            assert!(unavailable(&mut ncb).is_err());
            assert_eq!(ncb, before);
        }
    }
}
