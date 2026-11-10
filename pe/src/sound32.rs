//! Software DirectSound voices. Output positions advance from actual AudioQueue
//! consumption, never a successful silent sink or a wall-clock simulation.
use crate::Result;
use std::{
    collections::BTreeMap,
    ffi::c_void,
    sync::{Arc, Mutex},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Format {
    pub rate: u32,
    pub channels: u16,
    pub bits: u16,
}
impl Format {
    pub fn align(self) -> usize {
        usize::from(self.channels) * usize::from(self.bits) / 8
    }
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 16 {
            return Err("short PCM format".into());
        }
        let word = |at| u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap());
        let long = |at| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let f = Self {
            rate: long(4),
            channels: word(2),
            bits: word(14),
        };
        if word(0) != 1
            || ![1, 2].contains(&f.channels)
            || ![8, 16].contains(&f.bits)
            || !(8000..=96000).contains(&f.rate)
            || usize::from(word(12)) != f.align()
            || long(8) != f.rate * f.align() as u32
        {
            return Err("sound format outside mono/stereo PCM8/16 at 8–96kHz".into());
        }
        Ok(f)
    }
    pub fn bytes(self) -> [u8; 18] {
        let mut v = [0; 18];
        v[..2].copy_from_slice(&1u16.to_le_bytes());
        v[2..4].copy_from_slice(&self.channels.to_le_bytes());
        v[4..8].copy_from_slice(&self.rate.to_le_bytes());
        v[8..12].copy_from_slice(&(self.rate * self.align() as u32).to_le_bytes());
        v[12..14].copy_from_slice(&(self.align() as u16).to_le_bytes());
        v[14..16].copy_from_slice(&self.bits.to_le_bytes());
        v
    }
}
pub(super) struct Voice {
    pub data: Vec<u8>,
    pub format: Format,
    pub frequency: u32,
    pub volume: i32,
    pub pan: i32,
    pub requested: bool,
    pub looping: bool,
    pub base: u64,
    pub start: u64,
    pub notices: Vec<(u32, alkyhol::kernel::EventNotifier)>,
}
impl Voice {
    pub fn new(size: usize, format: Format) -> Self {
        Self {
            data: vec![if format.bits == 8 { 128 } else { 0 }; size],
            format,
            frequency: format.rate,
            volume: 0,
            pan: 0,
            requested: false,
            looping: false,
            base: 0,
            start: 0,
            notices: Vec::new(),
        }
    }
    fn frames(&self) -> u64 {
        (self.data.len() / self.format.align()) as u64
    }
    pub fn position(&self, consumed: u64) -> u32 {
        let n = self.frames();
        if n == 0 {
            return 0;
        }
        let at = self.base
            + if self.requested {
                (consumed.saturating_sub(self.start) as u128 * self.frequency as u128 / 48000)
                    as u64
            } else {
                0
            };
        let at = if self.looping { at % n } else { at.min(n - 1) };
        (at * self.format.align() as u64) as u32
    }
    pub fn playing(&self, consumed: u64) -> bool {
        self.requested
            && (self.looping
                || self.base
                    + ((consumed.saturating_sub(self.start) as u128 * self.frequency as u128
                        / 48000) as u64)
                    < self.frames())
    }
    pub fn stop(&mut self, consumed: u64) {
        self.base = u64::from(self.position(consumed)) / self.format.align() as u64;
        self.requested = false;
        for (offset, event) in &self.notices {
            if *offset == u32::MAX {
                event.signal();
            }
        }
    }
    fn sample(&self, frame: u64, channel: usize) -> f32 {
        let at = frame as usize * self.format.align()
            + channel.min(self.format.channels as usize - 1) * (self.format.bits as usize / 8);
        if self.format.bits == 8 {
            (f32::from(self.data[at]) - 128.) / 128.
        } else {
            f32::from(i16::from_le_bytes(
                self.data[at..at + 2].try_into().unwrap(),
            )) / 32768.
        }
    }
    fn mix(&self, at: u64, out: &mut [f32]) {
        let count = self.frames();
        if !self.requested || count == 0 {
            return;
        }
        let gain = if self.volume == -10000 {
            0.
        } else {
            10f32.powf(self.volume as f32 / 2000.)
        };
        let left = gain
            * if self.pan > 0 {
                10f32.powf(-(self.pan as f32) / 2000.)
            } else {
                1.
            };
        let right = gain
            * if self.pan < 0 {
                10f32.powf(self.pan as f32 / 2000.)
            } else {
                1.
            };
        for (n, pair) in out.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            let global = at + n as u64;
            if global < self.start {
                continue;
            }
            let phase =
                self.base as f64 + (global - self.start) as f64 * self.frequency as f64 / 48000.;
            let source = phase.floor() as u64;
            if !self.looping && source >= count {
                continue;
            }
            let a = source % count;
            let b = if self.looping {
                (source + 1) % count
            } else {
                (source + 1).min(count - 1)
            };
            let t = (phase - phase.floor()) as f32;
            pair[0] += (self.sample(a, 0) * (1. - t) + self.sample(b, 0) * t) * left;
            pair[1] += (self.sample(a, 1) * (1. - t) + self.sample(b, 1) * t) * right;
        }
    }
}
#[derive(Default)]
pub(super) struct Mixer {
    pub voices: BTreeMap<u32, Voice>,
    pub produced: u64,
    pub consumed: u64,
    pub nonzero: u64,
}
impl Mixer {
    fn fill(&mut self, out: &mut [f32], consumed: bool) {
        let frames = out.len() / 2;
        let previous = self.consumed;
        if consumed {
            self.consumed += frames as u64;
        }
        for voice in self.voices.values() {
            if !voice.requested || voice.frames() == 0 || self.consumed < voice.start {
                continue;
            }
            let source = |frame: u64| {
                voice.base
                    + ((frame.saturating_sub(voice.start) as u128 * voice.frequency as u128 / 48000)
                        as u64)
            };
            let now = source(self.consumed);
            let before = if previous < voice.start {
                None
            } else {
                Some(source(previous))
            };
            for (offset, event) in &voice.notices {
                let end = voice.frames();
                let hit = if *offset == u32::MAX {
                    !voice.looping && before.is_none_or(|b| b < end) && now >= end
                } else {
                    let target = u64::from(*offset) / voice.format.align() as u64;
                    if voice.looping {
                        let next = before.map_or(target, |b| {
                            let cycle = b / end;
                            let p = cycle * end + target;
                            if p > b { p } else { p + end }
                        });
                        next <= now
                    } else {
                        target <= now && before.is_none_or(|b| target > b)
                    }
                };
                if hit {
                    event.signal();
                }
            }
        }
        out.fill(0.);
        for voice in self.voices.values() {
            voice.mix(self.produced, out);
        }
        for pair in out.as_chunks_mut::<2>().0 {
            for v in pair.iter_mut() {
                *v = v.clamp(-1., 1.);
            }
            if pair.iter().any(|v| v.abs() > 1e-8) {
                self.nonzero += 1;
            }
        }
        self.produced += frames as u64;
    }
}
unsafe extern "C" {
    fn alkyhol_sound_open(
        user: *mut c_void,
        fill: extern "C" fn(*mut c_void, *mut f32, u32, u32),
        out: *mut *mut c_void,
    ) -> i32;
    fn alkyhol_sound_close(out: *mut c_void);
    fn alkyhol_sound_status(out: *mut c_void) -> i32;
}
extern "C" fn fill(user: *mut c_void, data: *mut f32, frames: u32, consumed: u32) {
    let shared = unsafe { &*(user as *const Mutex<Mixer>) };
    let out = unsafe { std::slice::from_raw_parts_mut(data, frames as usize * 2) };
    shared
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .fill(out, consumed != 0);
}
pub(super) struct Output {
    raw: *mut c_void,
    pub mixer: Arc<Mutex<Mixer>>,
}
impl Output {
    pub fn status(&self) -> i32 {
        unsafe { alkyhol_sound_status(self.raw) }
    }
    pub fn new() -> Result<Self> {
        let mixer = Arc::new(Mutex::new(Mixer::default()));
        let mut raw = std::ptr::null_mut();
        let status =
            unsafe { alkyhol_sound_open(Arc::as_ptr(&mixer) as *mut c_void, fill, &mut raw) };
        if status != 0 || raw.is_null() {
            return Err(format!(
                "actual AudioQueue output initialization failed: {status}"
            ));
        }
        Ok(Self { raw, mixer })
    }
}
impl Drop for Output {
    fn drop(&mut self) {
        unsafe {
            alkyhol_sound_close(self.raw);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pcm_formats_positions_looping_and_stereo_pan_are_actual_sample_operations() {
        let format = Format {
            rate: 48000,
            channels: 1,
            bits: 16,
        };
        assert_eq!(Format::parse(&format.bytes()).unwrap(), format);
        for at in [0, 2, 4, 8, 12, 14] {
            let mut b = format.bytes();
            b[at] ^= 0xff;
            assert!(Format::parse(&b).is_err());
        }
        let mut v = Voice::new(4, format);
        v.data = [16384i16, -16384]
            .into_iter()
            .flat_map(i16::to_le_bytes)
            .collect();
        v.requested = true;
        v.looping = true;
        v.start = 10;
        assert_eq!(v.position(9), 0);
        assert_eq!(v.position(11), 2);
        assert_eq!(v.position(12), 0);
        let mut out = [0.; 8];
        v.mix(10, &mut out);
        assert_eq!(out, [0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, -0.5]);
        v.pan = 10000;
        v.volume = -2000;
        out.fill(0.);
        v.mix(10, &mut out);
        assert!((out[1] - 0.05).abs() < 1e-6);
        assert!(out[0].abs() < 1e-6);
        v.stop(11);
        assert_eq!(v.position(100), 2);
        assert!(!v.playing(100));
    }
    #[test]
    fn resampling_consumption_and_clipping_do_not_use_host_wall_time() {
        let f = Format {
            rate: 24000,
            channels: 1,
            bits: 8,
        };
        let mut v = Voice::new(2, f);
        v.data = vec![128, 255];
        v.requested = true;
        v.looping = true;
        let mut m = Mixer::default();
        m.voices.insert(1, v);
        let mut out = [0.; 8];
        m.fill(&mut out, false);
        assert_eq!(m.consumed, 0);
        assert_eq!(m.produced, 4);
        assert_eq!(out[0], 0.);
        assert!((out[2] - 127. / 256.).abs() < 1e-6);
        m.fill(&mut out, true);
        assert_eq!(m.consumed, 4);
        assert!(m.nonzero > 0);
    }
    #[test]
    fn voices_sum_clip_and_exact_mute_without_changing_source_data() {
        let format = Format {
            rate: 48000,
            channels: 2,
            bits: 16,
        };
        let mut m = Mixer::default();
        for id in 1..=2 {
            let mut v = Voice::new(4, format);
            v.data = [30000i16.to_le_bytes(), (-30000i16).to_le_bytes()].concat();
            v.requested = true;
            v.looping = true;
            m.voices.insert(id, v);
        }
        let mut out = [0.; 4];
        m.fill(&mut out, false);
        assert_eq!(out, [1., -1., 1., -1.]);
        for v in m.voices.values_mut() {
            v.volume = -10000;
        }
        m.fill(&mut out, true);
        assert_eq!(out, [0.; 4]);
        assert_eq!(
            m.voices[&1].data,
            [30000i16.to_le_bytes(), (-30000i16).to_le_bytes()].concat()
        );
    }
    #[test]
    fn notification_follows_consumption_and_retains_event_identity() {
        let handle =
            unsafe { alkyhol::ffi::CreateEventW(std::ptr::null(), 0, 0, std::ptr::null()) };
        assert_ne!(handle, 0);
        let event = alkyhol::kernel::kernel().event_notifier(handle).unwrap();
        let mut v = Voice::new(
            16,
            Format {
                rate: 48000,
                channels: 1,
                bits: 16,
            },
        );
        v.requested = true;
        v.looping = true;
        v.notices.push((4, event));
        let mut m = Mixer::default();
        m.voices.insert(1, v);
        m.fill(&mut [0.; 8], false);
        assert_eq!(alkyhol::ffi::WaitForSingleObject(handle, 0), 258);
        m.fill(&mut [0.; 8], true);
        assert_eq!(alkyhol::ffi::WaitForSingleObject(handle, 0), 0);
        assert_eq!(alkyhol::ffi::WaitForSingleObject(handle, 0), 258);
        m.fill(&mut [0.; 8], true);
        assert_eq!(alkyhol::ffi::WaitForSingleObject(handle, 0), 258);
        m.fill(&mut [0.; 8], true);
        assert_eq!(alkyhol::ffi::WaitForSingleObject(handle, 0), 0);
        assert_ne!(alkyhol::ffi::CloseHandle(handle), 0);
        // Closing the caller's handle does not invalidate the retained notifier.
        m.fill(&mut [0.; 16], true);
    }
    #[test]
    #[ignore = "plays a quiet original test tone on the actual default output"]
    fn physical_audioqueue_consumes_nonzero_pcm_and_tears_down() {
        let out = Output::new().unwrap();
        {
            let mut m = out.mixer.lock().unwrap();
            let f = Format {
                rate: 48000,
                channels: 2,
                bits: 16,
            };
            let mut v = Voice::new(4800 * 4, f);
            for n in 0..4800 {
                let s = ((n as f32 * 440. * std::f32::consts::TAU / 48000.).sin() * 500.) as i16;
                for c in 0..2 {
                    v.data[n * 4 + c * 2..n * 4 + c * 2 + 2].copy_from_slice(&s.to_le_bytes());
                }
            }
            v.looping = true;
            v.requested = true;
            v.start = m.produced;
            m.voices.insert(1, v);
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
        let m = out.mixer.lock().unwrap();
        assert!(m.consumed >= 2048);
        assert!(m.nonzero >= 1024);
    }
}
