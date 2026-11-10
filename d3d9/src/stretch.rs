//! Point/no-filter attachment scaling for the original save thumbnail path.
use crate::{Device,Result,Target};
use std::sync::{Arc,atomic::Ordering};
fn rectangle(r:[u32;4],size:(u32,u32))->Result<(u32,u32)> {
    if r[0]>=r[2] || r[1]>=r[3] || r[2]>size.0 || r[3]>size.1 {return Err("invalid stretch rectangle".into());}
    Ok((r[2]-r[0],r[3]-r[1]))
}
impl Device {
    /// Copy actual GPU pixels through point sampling at destination pixel centers.
    /// This bounded readback path is for infrequent surface transfers, not draws.
    pub fn stretch_target_region(&self,source:&Target,source_rect:[u32;4],destination:&mut Target,destination_rect:[u32;4],opaque:bool)->Result<()> {
        let (sw,sh)=rectangle(source_rect,source.dimensions())?;
        let (dw,dh)=rectangle(destination_rect,destination.dimensions())?;
        if Arc::ptr_eq(&source.initialized,&destination.initialized) || !source.initialized.load(Ordering::Acquire) {
            return Err("stretch requires distinct initialized source storage".into());
        }
        if source_rect==[0,0,source.width,source.height] && destination_rect==[0,0,destination.width,destination.height] && source.dimensions()==destination.dimensions() {
            return self.copy_target(source,destination,opaque);
        }
        let bytes=self.readback(source)?;
        let mut output=vec![0u8;dw as usize*dh as usize*4];
        for y in 0..dh {
            let sy=source_rect[1]+((2*y as u64+1)*sh as u64/(2*dh as u64)) as u32;
            for x in 0..dw {
                let sx=source_rect[0]+((2*x as u64+1)*sw as u64/(2*dw as u64)) as u32;
                let from=(sy as usize*source.width as usize+sx as usize)*4;
                let at=(y as usize*dw as usize+x as usize)*4;
                output[at..at+4].copy_from_slice(&bytes[from..from+4]);
                if opaque {output[at+3]=255;}
            }
        }
        self.write_target_region(destination,destination_rect,&output,dw*4)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore="requires an actual GPU"]
    fn actual_point_crop_scale_preserves_outside_pixels_and_rejects_invalid_transfers() {
        let device=Device::new().unwrap();let mut src=device.create_target(4,4).unwrap();let mut dst=device.create_target(4,4).unwrap();
        assert!(device.stretch_target_region(&src,[0,0,4,4],&mut dst,[0,0,2,2],false).is_err());
        let bytes:Vec<u8>=(0..16u8).flat_map(|n|[n,n+16,n+32,64]).collect();
        device.write_target_region(&mut src,[0,0,4,4],&bytes,16).unwrap();device.clear(&mut dst,0xff123456).unwrap();
        device.stretch_target_region(&src,[0,0,4,4],&mut dst,[1,1,3,3],false).unwrap();
        let out=device.readback(&dst).unwrap();
        for y in 0..4 {for x in 0..4 {let at=(y*4+x)*4;let expected=if (1..3).contains(&x)&&(1..3).contains(&y) {let i=(2*y-1)*4+2*x-1;[i as u8,i as u8+16,i as u8+32,64]}else{[0x56,0x34,0x12,255]};assert_eq!(&out[at..at+4],&expected);}}
        device.stretch_target_region(&src,[1,1,3,3],&mut dst,[0,0,4,4],true).unwrap();
        let out=device.readback(&dst).unwrap();assert_eq!(&out[..4],&[5,21,37,255]);assert_eq!(&out[60..],&[10,26,42,255]);
        let before=out;
        assert!(device.stretch_target_region(&src,[0,0,5,4],&mut dst,[0,0,4,4],false).is_err());assert_eq!(device.readback(&dst).unwrap(),before);
        let alias=src.clone();assert!(device.stretch_target_region(&alias,[0,0,4,4],&mut src,[0,0,4,4],false).is_err());
    }
}
