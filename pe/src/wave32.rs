//! Actual CoreAudio input/output-device enumeration for FMOD's WinMM fallback.
//! Enumeration alone does not implement waveOut playback or claim audio output.
use super::{LoadedImage32, unsupported};
use crate::Result;
unsafe extern "C" {
    fn alkyhol_audio_devices(input: u32, devices: *mut u32, capacity: u32, count: *mut u32) -> i32;
}
#[derive(Default)]
pub(super) struct State {
    outputs: Option<Vec<u32>>,
    inputs: Option<Vec<u32>>,
}
fn devices(input: bool) -> Result<Vec<u32>> {
    let mut devices = vec![0; 256];
    let mut count = 0;
    let status = unsafe {
        alkyhol_audio_devices(
            input as u32,
            devices.as_mut_ptr(),
            devices.len() as u32,
            &mut count,
        )
    };
    if status != 0 {
        return Err(format!(
            "CoreAudio device enumeration failed: OSStatus {status}"
        ));
    }
    if count as usize > devices.len() {
        return Err("CoreAudio device count exceeded allocated extent".into());
    }
    devices.truncate(count as usize);
    if devices.contains(&0) {
        return Err("CoreAudio returned an invalid device object".into());
    }
    Ok(devices)
}
pub(super) fn count(image: &LoadedImage32<'_>, input: bool) -> u32 {
    // Re-query the real device list: FMOD uses these counts to detect hotplug.
    let next = devices(input).unwrap_or_else(|e| unsupported(&e));
    let mut state = image.cpu.wave.borrow_mut();
    let current = if input {
        &mut state.inputs
    } else {
        &mut state.outputs
    };
    if current.as_ref() != Some(&next) {
        eprintln!(
            "PE32 wave{} actual CoreAudio object IDs: {next:x?}",
            if input { "In" } else { "Out" }
        );
    }
    let count = next.len() as u32;
    *current = Some(next);
    count
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_coreaudio_snapshot_has_bounded_distinct_input_and_output_objects() {
        let devices = devices(false).unwrap();
        assert!(devices.len() <= 256);
        let mut ids = devices.clone();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), devices.len());
        // Zero is valid only when CoreAudio actually reports no output devices.
        eprintln!("actual host CoreAudio outputs: {devices:?}");
        let inputs = super::devices(true).unwrap();
        assert!(inputs.len() <= 256);
        let mut unique = inputs.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), inputs.len());
        eprintln!("actual host CoreAudio inputs: {inputs:?}");
    }
}
