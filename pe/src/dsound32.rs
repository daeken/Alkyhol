//! PE32 DirectSound ABI over real software mixing and AudioQueue playback.
//! Public DirectSound interfaces follow Microsoft's documented ABI; proprietary
//! FMOD/game code remains unchanged. Unsupported features return errors or stop.
use super::{
    CallingConvention32, Cpu32, LoadedImage32, PAGE, d3d9_32,
    sound32::{Format, Mixer, Output, Voice},
    unsupported,
};
use crate::Result;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
pub(super) const BASE: u32 = 0xb000;
const INVALID: u32 = 0x80070057;
const NOIF: u32 = 0x80004002;
const UNSUP: u32 = 0x80004001;
const CONTROL: u32 = 0x8878001e;
const UNKNOWN: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0xc0, 0, 0, 0, 0, 0, 0, 0x46];
const DEVICE: [u8; 16] = [
    0x83, 0xfa, 0x9a, 0x27, 0x81, 0x49, 0xce, 0x11, 0xa5, 0x21, 0, 0x20, 0xaf, 0xb, 0xe5, 0x60,
];
const DEVICE8: [u8; 16] = [
    0x93, 0x7e, 0xa, 0xc5, 0x95, 0xf3, 0x34, 0x48, 0x9e, 0xf6, 0x7f, 0xa9, 0x9d, 0xe5, 9, 0x66,
];
const BUFFER: [u8; 16] = [
    0x85, 0xfa, 0x9a, 0x27, 0x81, 0x49, 0xce, 0x11, 0xa5, 0x21, 0, 0x20, 0xaf, 0xb, 0xe5, 0x60,
];
const BUFFER8: [u8; 16] = [
    0x49, 0xa4, 0x25, 0x68, 0x24, 0x75, 0x82, 0x4d, 0x92, 0xf, 0x50, 0xe3, 0x6a, 0xb3, 0xab, 0x1e,
];
const LISTENER: [u8; 16] = [
    0x84, 0xfa, 0x9a, 0x27, 0x81, 0x49, 0xce, 0x11, 0xa5, 0x21, 0, 0x20, 0xaf, 0xb, 0xe5, 0x60,
];
const NOTIFY: [u8; 16] = [
    0x83, 7, 0x21, 0xb0, 0xcd, 0x89, 0xd0, 0x11, 0xaf, 8, 0, 0xa0, 0xc9, 0x25, 0xcd, 0x16,
];
// Identity of this software adapter, not a Windows hardware/device GUID.
const ADAPTER: [u8; 16] = [
    0x41, 0x4c, 0x4b, 0x59, 0x48, 0x4f, 0x4c, 0x41, 0x80, 0x44, 0x49, 0x4f, 0, 0, 0, 1,
];
const DEVICE_METHODS: [(&str, u16); 12] = [
    ("DirectSound::QueryInterface", 12),
    ("DirectSound::AddRef", 4),
    ("DirectSound::Release", 4),
    ("DirectSound::CreateSoundBuffer", 16),
    ("DirectSound::GetCaps", 8),
    ("DirectSound::DuplicateSoundBuffer", 12),
    ("DirectSound::SetCooperativeLevel", 12),
    ("DirectSound::Compact", 4),
    ("DirectSound::GetSpeakerConfig", 8),
    ("DirectSound::SetSpeakerConfig", 8),
    ("DirectSound::Initialize", 8),
    ("DirectSound8::VerifyCertification", 8),
];
const BUFFER_METHODS: [(&str, u16); 24] = [
    ("DirectSoundBuffer::QueryInterface", 12),
    ("DirectSoundBuffer::AddRef", 4),
    ("DirectSoundBuffer::Release", 4),
    ("DirectSoundBuffer::GetCaps", 8),
    ("DirectSoundBuffer::GetCurrentPosition", 12),
    ("DirectSoundBuffer::GetFormat", 16),
    ("DirectSoundBuffer::GetVolume", 8),
    ("DirectSoundBuffer::GetPan", 8),
    ("DirectSoundBuffer::GetFrequency", 8),
    ("DirectSoundBuffer::GetStatus", 8),
    ("DirectSoundBuffer::Initialize", 12),
    ("DirectSoundBuffer::Lock", 32),
    ("DirectSoundBuffer::Play", 16),
    ("DirectSoundBuffer::SetCurrentPosition", 8),
    ("DirectSoundBuffer::SetFormat", 8),
    ("DirectSoundBuffer::SetVolume", 8),
    ("DirectSoundBuffer::SetPan", 8),
    ("DirectSoundBuffer::SetFrequency", 8),
    ("DirectSoundBuffer::Stop", 4),
    ("DirectSoundBuffer::Unlock", 20),
    ("DirectSoundBuffer::Restore", 4),
    ("DirectSoundBuffer8::SetFX", 16),
    ("DirectSoundBuffer8::AcquireResources", 16),
    ("DirectSoundBuffer8::GetObjectInPath", 20),
];
#[derive(Default)]
pub(super) struct State {
    tables: BTreeMap<u32, u32>,
    objects: BTreeMap<u32, Object>,
    aliases: BTreeMap<u32, (u32, u32)>,
    listeners: BTreeMap<u32, Listener>,
    notification_events: BTreeMap<u32, Vec<u32>>,
    pub module_refs: u32,
    pub static_loaded: bool,
    output: Option<Output>,
}
struct Object {
    refs: u32,
    body: Body,
}
enum Body {
    Device {
        cooperative: u32,
        format: Format,
    },
    Buffer {
        parent: u32,
        flags: u32,
        storage: u32,
        locked: Option<[u32; 4]>,
        notify: u32,
    },
}
pub(super) fn signature(id: u32) -> Option<(&'static str, u16)> {
    let n = id.checked_sub(BASE)?;
    let slot = (n % 256) as usize;
    match n / 256 {
        0 => DEVICE_METHODS.get(slot).copied(),
        1 => BUFFER_METHODS.get(slot).copied(),
        2 => [
            ("DirectSoundNotify::QueryInterface", 12),
            ("DirectSoundNotify::AddRef", 4),
            ("DirectSoundNotify::Release", 4),
            ("DirectSoundNotify::SetNotificationPositions", 12),
        ]
        .get(slot)
        .copied(),
        3 => LISTENER_METHODS.get(slot).copied(),
        _ => None,
    }
}
const LISTENER_METHODS: [(&str, u16); 18] = [
    ("DirectSound3DListener::QueryInterface", 12),
    ("DirectSound3DListener::AddRef", 4),
    ("DirectSound3DListener::Release", 4),
    ("DirectSound3DListener::GetAllParameters", 8),
    ("DirectSound3DListener::GetDistanceFactor", 8),
    ("DirectSound3DListener::GetDopplerFactor", 8),
    ("DirectSound3DListener::GetOrientation", 12),
    ("DirectSound3DListener::GetPosition", 8),
    ("DirectSound3DListener::GetRolloffFactor", 8),
    ("DirectSound3DListener::GetVelocity", 8),
    ("DirectSound3DListener::SetAllParameters", 12),
    ("DirectSound3DListener::SetDistanceFactor", 12),
    ("DirectSound3DListener::SetDopplerFactor", 12),
    ("DirectSound3DListener::SetOrientation", 32),
    ("DirectSound3DListener::SetPosition", 20),
    ("DirectSound3DListener::SetRolloffFactor", 12),
    ("DirectSound3DListener::SetVelocity", 20),
    ("DirectSound3DListener::CommitDeferredSettings", 4),
];
struct Listener {
    alias: u32,
    active: [u32; 16],
    pending: [u32; 16],
}
impl Default for Listener {
    fn default() -> Self {
        let mut active = [0; 16];
        active[0] = 64;
        // position, velocity, +Z front, +Y top, distance/rolloff/doppler 1.
        for i in [9, 11, 13, 14, 15] {
            active[i] = 1f32.to_bits();
        }
        Self {
            alias: 0,
            active,
            pending: active,
        }
    }
}
impl Listener {
    fn update(&mut self, first: usize, values: &[u32], apply: u32) -> u32 {
        if apply > 1 || values.iter().any(|v| !f32::from_bits(*v).is_finite()) {
            return INVALID;
        }
        for (n, bits) in values.iter().enumerate() {
            let i = first + n;
            let f = f32::from_bits(*bits);
            if i == 13 && f <= 0. || matches!(i, 14 | 15) && !(0. ..=10.).contains(&f) {
                return INVALID;
            }
        }
        self.pending[first..first + values.len()].copy_from_slice(values);
        if apply == 0 {
            self.active[first..first + values.len()].copy_from_slice(values);
        }
        0
    }
    fn commit(&mut self) {
        self.active = self.pending;
    }
}
fn listener_dispatch(
    image: &LoadedImage32<'_>,
    listener: &mut Listener,
    slot: u32,
    a: &[u32; 16],
) -> u32 {
    match slot {
        3 => {
            if read::<1>(image, a[1])[0] != 64 {
                return INVALID;
            }
            write(image, a[1], &listener.active);
            0
        }
        4 | 5 | 8 => {
            let index = match slot {
                4 => 13,
                5 => 15,
                _ => 14,
            };
            write(image, a[1], &[listener.active[index]]);
            0
        }
        6 => {
            write(image, a[1], &listener.active[7..10]);
            write(image, a[2], &listener.active[10..13]);
            0
        }
        7 | 9 => {
            let first = if slot == 7 { 1 } else { 4 };
            write(image, a[1], &listener.active[first..first + 3]);
            0
        }
        10 => {
            let all = read::<16>(image, a[1]);
            if all[0] != 64 {
                return INVALID;
            }
            listener.update(1, &all[1..], a[2])
        }
        11 | 12 | 15 => listener.update(
            match slot {
                11 => 13,
                12 => 15,
                _ => 14,
            },
            &a[1..2],
            a[2],
        ),
        13 => listener.update(7, &a[1..7], a[7]),
        14 | 16 => listener.update(if slot == 14 { 1 } else { 4 }, &a[1..4], a[4]),
        17 => {
            listener.commit();
            0
        }
        _ => unreachable!(),
    }
}
fn checked(image: &LoadedImage32<'_>, at: u32, len: usize, write: bool) {
    if at == 0 || !image.permits(at as usize, len, write) {
        unsupported("DirectSound pointer outside checked guest storage");
    }
}
fn read<const N: usize>(image: &LoadedImage32<'_>, at: u32) -> [u32; N] {
    checked(image, at, N * 4, false);
    unsafe { (at as *const [u32; N]).read_unaligned() }
}
fn write(image: &LoadedImage32<'_>, at: u32, words: &[u32]) {
    checked(image, at, words.len() * 4, true);
    for (i, w) in words.iter().enumerate() {
        unsafe { (at as *mut u32).add(i).write_unaligned(*w) }
    }
}
fn guid(image: &LoadedImage32<'_>, at: u32) -> [u8; 16] {
    checked(image, at, 16, false);
    unsafe { (at as *const [u8; 16]).read_unaligned() }
}
fn default_guid(g: [u8; 16]) -> bool {
    g == [0; 16]
        || g == ADAPTER
        || g == [
            0, 0, 0xf0, 0xde, 0x6d, 0x9c, 0xed, 0x47, 0xaa, 0xf1, 0x4d, 0xda, 0x8f, 0x2b, 0x5c, 3,
        ]
        || g == [
            2, 0, 0xf0, 0xde, 0x6d, 0x9c, 0xed, 0x47, 0xaa, 0xf1, 0x4d, 0xda, 0x8f, 0x2b, 0x5c, 3,
        ]
}
fn format(image: &LoadedImage32<'_>, at: u32) -> Result<Format> {
    checked(image, at, 16, false);
    Format::parse(unsafe { std::slice::from_raw_parts(at as *const u8, 16) })
}
fn table(cpu: &Cpu32, s: &mut State, kind: u32) -> Result<u32> {
    if let Some(&v) = s.tables.get(&kind) {
        return Ok(v);
    }
    let count = match kind {
        0 => 12,
        1 => 24,
        2 => 4,
        3 => 18,
        _ => unreachable!(),
    };
    let m = cpu.mapping(PAGE * 2, PAGE, 0)?;
    let base = m.base as u32;
    let mut bytes = vec![0; PAGE * 2];
    let mut cs = 0u16;
    #[cfg(target_arch = "x86_64")]
    unsafe {
        std::arch::asm!("mov {0:x}, cs",out(reg)cs,options(nomem,nostack,preserves_flags));
    }
    for slot in 0..count {
        let id = BASE + kind * 256 + slot;
        let at = PAGE + slot as usize * 32;
        let address = base + at as u32;
        #[cfg(target_arch = "aarch64")]
        cpu.native_gates
            .borrow_mut()
            .insert(address, (id, signature(id).unwrap().1));
        bytes[slot as usize * 4..slot as usize * 4 + 4].copy_from_slice(&address.to_le_bytes());
        bytes[at] = 0x68;
        bytes[at + 1..at + 5].copy_from_slice(&(address + 16).to_le_bytes());
        bytes[at + 5] = 0x68;
        bytes[at + 6..at + 10].copy_from_slice(&id.to_le_bytes());
        bytes[at + 10..at + 12].copy_from_slice(&[0xff, 0x2d]);
        bytes[at + 12..at + 16].copy_from_slice(&(address + 20).to_le_bytes());
        bytes[at + 16] = 0xc2;
        bytes[at + 17..at + 19].copy_from_slice(&signature(id).unwrap().1.to_le_bytes());
        bytes[at + 20..at + 24].copy_from_slice(&(cpu.service_gates.get() + 64).to_le_bytes());
        bytes[at + 24..at + 26].copy_from_slice(&cs.to_le_bytes());
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), m.base, bytes.len());
    }
    m.protect(0, PAGE, 1)?;
    m.protect(PAGE, PAGE, 5)?;
    #[cfg(target_arch = "aarch64")]
    cpu.native_code
        .borrow_mut()
        .push((base + PAGE as u32, base + 2 * PAGE as u32));
    cpu.com_storage.borrow_mut().push((m, false));
    s.tables.insert(kind, base);
    Ok(base)
}
fn object(cpu: &Cpu32, s: &mut State, body: Body) -> u32 {
    let kind = if matches!(body, Body::Device { .. }) {
        0
    } else {
        1
    };
    let t = table(cpu, s, kind).unwrap_or_else(|e| unsupported(&e));
    let at =
        d3d9_32::memory(cpu, &t.to_le_bytes(), true, false).unwrap_or_else(|e| unsupported(&e));
    s.objects.insert(at, Object { refs: 1, body });
    s.aliases.insert(at, (at, kind));
    at
}
fn mixer(s: &State) -> Arc<Mutex<Mixer>> {
    s.output.as_ref().unwrap().mixer.clone()
}
pub(super) fn create(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if a[1] == 0 {
        return INVALID;
    }
    write(image, a[1], &[0]);
    if a[2] != 0 {
        return 0x80040110;
    }
    if a[0] != 0 && !default_guid(guid(image, a[0])) {
        return 0x88780078;
    }
    let mut s = image.cpu.dsound.borrow_mut();
    if s.output.is_none() {
        match Output::new() {
            Ok(out) => s.output = Some(out),
            Err(e) => {
                eprintln!("DirectSound output unavailable: {e}");
                return 0x88780078;
            }
        }
    }
    let at = object(
        &image.cpu,
        &mut s,
        Body::Device {
            cooperative: 1,
            format: Format {
                rate: 44100,
                channels: 2,
                bits: 16,
            },
        },
    );
    write(image, a[1], &[at]);
    0
}
pub(super) fn enumerate(image: &LoadedImage32<'_>, a: &[u32; 8], wide: bool) -> u32 {
    if a[0] == 0 {
        return INVALID;
    }
    let data = if wide {
        "Alkyhol CoreAudio default output\0"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>()
    } else {
        b"Alkyhol CoreAudio default output\0".to_vec()
    };
    let name = d3d9_32::memory(&image.cpu, &data, false, false).unwrap_or_else(|e| unsupported(&e));
    let module = if wide { vec![0u8; 2] } else { vec![0u8] };
    let module =
        d3d9_32::memory(&image.cpu, &module, false, false).unwrap_or_else(|e| unsupported(&e));
    let mut devices = [0u32; 256];
    let mut count = 0;
    unsafe extern "C" {
        fn alkyhol_audio_devices(input: u32, ids: *mut u32, capacity: u32, count: *mut u32) -> i32;
    }
    let status = unsafe { alkyhol_audio_devices(0, devices.as_mut_ptr(), 256, &mut count) };
    if status != 0 {
        return 0x88780078;
    }
    if count == 0 {
        return 0x88780078;
    }
    let keep = unsafe {
        image.callback_frame(a[0], &[0, name, module, a[1]], CallingConvention32::Stdcall)
    }
    .unwrap_or_else(|e| unsupported(&e))
    .unwrap_or_else(|| unsupported("non-returning sound enumeration callback"));
    // The default alias is enumerated separately from the same adapter's GUID,
    // as DirectSound specifies. Callers commonly omit the NULL/default alias.
    if keep != 0 {
        let adapter =
            d3d9_32::memory(&image.cpu, &ADAPTER, false, false).unwrap_or_else(|e| unsupported(&e));
        unsafe {
            image.callback_frame(
                a[0],
                &[adapter, name, module, a[1]],
                CallingConvention32::Stdcall,
            )
        }
        .unwrap_or_else(|e| unsupported(&e))
        .unwrap_or_else(|| unsupported("non-returning sound enumeration callback"));
    }
    0
}
pub(super) fn get_device_id(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if a[1] == 0 {
        return INVALID;
    }
    checked(image, a[1], 16, true);
    if a[0] != 0 && !default_guid(guid(image, a[0])) {
        return 0x88780078;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(ADAPTER.as_ptr(), a[1] as *mut u8, 16);
    }
    0
}
fn release(s: &mut State, at: u32) -> u32 {
    let object = s.objects.get_mut(&at).unwrap();
    object.refs -= 1;
    let refs = object.refs;
    if refs == 0 {
        let object = s.objects.remove(&at).unwrap();
        if let Body::Buffer { parent, .. } = object.body {
            mixer(s).lock().unwrap().voices.remove(&at);
            s.notification_events.remove(&at);
            s.listeners.remove(&at);
            release(s, parent);
        }
        s.aliases.retain(|_, (canonical, _)| *canonical != at);
        if s.objects.is_empty() {
            s.output = None;
        }
    }
    refs
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, a: &[u32; 16]) -> u32 {
    let result = dispatch_inner(image, id, a);
    if (result as i32) < 0 {
        let (name, bytes) = signature(id).unwrap();
        eprintln!(
            "PE32 DirectSound failure: {name} -> {result:08x}; args={:x?}",
            &a[..usize::from(bytes) / 4]
        );
    }
    result
}
fn dispatch_inner(image: &LoadedImage32<'_>, id: u32, a: &[u32; 16]) -> u32 {
    let kind = (id - BASE) / 256;
    let slot = (id - BASE) % 256;
    let mut s = image.cpu.dsound.borrow_mut();
    let (at, actual) = *s
        .aliases
        .get(&a[0])
        .unwrap_or_else(|| unsupported("dead DirectSound interface"));
    if kind != actual {
        unsupported("DirectSound interface type mismatch");
    }
    if slot == 0 {
        write(image, a[2], &[0]);
        let iid = guid(image, a[1]);
        let device = matches!(s.objects[&at].body, Body::Device { .. });
        let target = if iid == UNKNOWN
            || device && [DEVICE, DEVICE8].contains(&iid)
            || !device && [BUFFER, BUFFER8].contains(&iid)
        {
            at
        } else if !device && iid == LISTENER && s.listeners.contains_key(&at) {
            let alias = s.listeners[&at].alias;
            if alias != 0 {
                alias
            } else {
                let t = table(&image.cpu, &mut s, 3).unwrap_or_else(|e| unsupported(&e));
                let alias = d3d9_32::memory(&image.cpu, &t.to_le_bytes(), true, false)
                    .unwrap_or_else(|e| unsupported(&e));
                s.aliases.insert(alias, (at, 3));
                s.listeners.get_mut(&at).unwrap().alias = alias;
                alias
            }
        } else if !device && iid == NOTIFY {
            let flags = match s.objects[&at].body {
                Body::Buffer { flags, .. } => flags,
                _ => unreachable!(),
            };
            if flags & 0x100 == 0 {
                return NOIF;
            }
            let existing = match s.objects[&at].body {
                Body::Buffer { notify, .. } => notify,
                _ => 0,
            };
            if existing != 0 {
                existing
            } else {
                let t = table(&image.cpu, &mut s, 2).unwrap_or_else(|e| unsupported(&e));
                let alias = d3d9_32::memory(&image.cpu, &t.to_le_bytes(), true, false)
                    .unwrap_or_else(|e| unsupported(&e));
                s.aliases.insert(alias, (at, 2));
                if let Body::Buffer { notify, .. } = &mut s.objects.get_mut(&at).unwrap().body {
                    *notify = alias;
                }
                alias
            }
        } else {
            return NOIF;
        };
        s.objects.get_mut(&at).unwrap().refs += 1;
        write(image, a[2], &[target]);
        return 0;
    }
    if slot == 1 {
        s.objects.get_mut(&at).unwrap().refs += 1;
        return s.objects[&at].refs;
    }
    if slot == 2 {
        return release(&mut s, at);
    }
    if kind == 3 {
        return listener_dispatch(image, s.listeners.get_mut(&at).unwrap(), slot, a);
    }
    let mix = mixer(&s);
    if kind == 2 {
        if a[1] > 128 {
            return INVALID;
        }
        if a[1] != 0 {
            checked(image, a[2], a[1] as usize * 8, false);
        }
        let mut notices = Vec::new();
        let mut handles = Vec::new();
        let mut m = mix.lock().unwrap();
        let consumed = m.consumed;
        let v = m.voices.get_mut(&at).unwrap();
        if v.playing(consumed) {
            return 0x88780032;
        }
        for n in 0..a[1] {
            let words = read::<2>(image, a[2] + n * 8);
            if words[0] != u32::MAX && words[0] as usize >= v.data.len() {
                return INVALID;
            }
            let event = match alkyhol::kernel::kernel().event_notifier(words[1] as usize) {
                Ok(v) => v,
                Err(_) => return INVALID,
            };
            notices.push((words[0], event));
            handles.push(words[1]);
        }
        v.notices = notices;
        s.notification_events.insert(at, handles);
        return 0;
    }
    if kind == 0 {
        return match slot {
            3 => {
                write(image, a[2], &[0]);
                if a[3] != 0 {
                    return 0x80040110;
                }
                let d = read::<5>(image, a[1]);
                if ![20, 36].contains(&d[0]) || d[1] & !0x7c1ff != 0 {
                    return INVALID;
                }
                let primary = d[1] & 1 != 0;
                if d[1] & 4 != 0 || d[1] & 0x200 != 0 || !primary && d[1] & 0x10 != 0 {
                    return CONTROL;
                }
                let fmt = if primary {
                    if d[4] != 0 || d[2] != 0 {
                        return INVALID;
                    }
                    match s.objects[&at].body {
                        Body::Device { format, .. } => format,
                        _ => unreachable!(),
                    }
                } else {
                    if d[4] == 0 {
                        return INVALID;
                    }
                    match format(image, d[4]) {
                        Ok(v) => v,
                        Err(_) => return 0x88780064,
                    }
                };
                let size = if primary { 8192 } else { d[2] as usize };
                if size < fmt.align()
                    || size > 16 * 1024 * 1024
                    || !size.is_multiple_of(fmt.align())
                {
                    return INVALID;
                }
                let storage = d3d9_32::memory(
                    &image.cpu,
                    &vec![if fmt.bits == 8 { 128 } else { 0 }; size],
                    true,
                    false,
                )
                .unwrap_or_else(|e| unsupported(&e));
                let obj = object(
                    &image.cpu,
                    &mut s,
                    Body::Buffer {
                        parent: at,
                        flags: d[1] | 8,
                        storage,
                        locked: None,
                        notify: 0,
                    },
                );
                s.objects.get_mut(&at).unwrap().refs += 1;
                if primary && d[1] & 0x10 != 0 {
                    s.listeners.insert(obj, Listener::default());
                }
                mix.lock()
                    .unwrap()
                    .voices
                    .insert(obj, Voice::new(size, fmt));
                write(image, a[2], &[obj]);
                eprintln!(
                    "DirectSound real PCM buffer: bytes={size} format={fmt:?} flags={:x}",
                    d[1]
                );
                0
            }
            4 => {
                let d = read::<1>(image, a[1]);
                if d[0] != 96 {
                    return INVALID;
                }
                let mut caps = [0u32; 24];
                caps[0] = 96;
                caps[1] = 0xf3f;
                caps[2] = 8000;
                caps[3] = 96000;
                caps[4] = 1;
                write(image, a[1], &caps);
                0
            }
            5 => UNSUP,
            6 => {
                if !image.cpu.ui.borrow().audio_window(a[1]) || !(1..=4).contains(&a[2]) {
                    return INVALID;
                }
                if a[2] > 2 {
                    return UNSUP;
                }
                if let Body::Device { cooperative, .. } = &mut s.objects.get_mut(&at).unwrap().body
                {
                    *cooperative = a[2];
                }
                0
            }
            7 => UNSUP,
            8 => {
                write(image, a[1], &[4]);
                0
            }
            9 => {
                if a[1] == 4 {
                    0
                } else {
                    UNSUP
                }
            }
            10 => 0x88780082,
            11 => {
                write(image, a[1], &[1]);
                0
            }
            _ => unsupported("unimplemented DirectSound device method"),
        };
    }
    if s.output.as_ref().unwrap().status() != 0 && ![3, 5, 6, 7, 8, 9, 18].contains(&slot) {
        return 0x88780096;
    }
    let (flags, storage, locked, parent) = match s.objects[&at].body {
        Body::Buffer {
            flags,
            storage,
            locked,
            parent,
            ..
        } => (flags, storage, locked, parent),
        _ => unreachable!(),
    };
    let mut m = mix.lock().unwrap();
    let consumed = m.consumed;
    let produced = m.produced;
    let v = m.voices.get_mut(&at).unwrap();
    match slot {
        3 => {
            if read::<1>(image, a[1])[0] != 20 {
                return INVALID;
            }
            write(image, a[1], &[20, flags, v.data.len() as u32, 0, 0]);
            0
        }
        4 => {
            let play = v.position(consumed);
            let ahead = (produced.saturating_sub(consumed) as u128 * v.frequency as u128 / 48000)
                as u32
                * v.format.align() as u32;
            let cursor = (play + ahead) % (v.data.len() as u32);
            if a[1] != 0 {
                write(image, a[1], &[play]);
            }
            if a[2] != 0 {
                write(image, a[2], &[cursor]);
            }
            0
        }
        5 => {
            let bytes = v.format.bytes();
            if a[3] != 0 {
                write(image, a[3], &[18]);
            }
            if a[1] == 0 {
                return 0;
            }
            if a[2] < 18 {
                return INVALID;
            }
            checked(image, a[1], 18, true);
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), a[1] as *mut u8, 18);
            }
            0
        }
        6..=8 => {
            write(
                image,
                a[1],
                &[match slot {
                    6 => v.volume as u32,
                    7 => v.pan as u32,
                    _ => v.frequency,
                }],
            );
            0
        }
        9 => {
            if s.output.as_ref().unwrap().status() != 0 {
                write(image, a[1], &[2]);
                return 0;
            }
            write(
                image,
                a[1],
                &[if v.playing(consumed) {
                    1 | if v.looping { 4 } else { 0 }
                } else {
                    0
                }],
            );
            0
        }
        10 => 0x88780082,
        11 => {
            if locked.is_some() {
                return 0x88780032;
            }
            if flags & 1 != 0 {
                return 0x88780046;
            }
            if a[7] & !3 != 0 {
                return INVALID;
            }
            let length = v.data.len() as u32;
            let mut offset = a[1];
            let mut count = a[2];
            if a[7] & 1 != 0 {
                offset = (v.position(consumed)
                    + ((produced - consumed) as u128 * v.frequency as u128 / 48000) as u32
                        * v.format.align() as u32)
                    % length;
            }
            if a[7] & 2 != 0 {
                offset = 0;
                count = length;
            }
            if offset >= length || count == 0 || count > length {
                return INVALID;
            }
            let first = count.min(length - offset);
            let mut second = count - first;
            if a[5] == 0 && a[6] == 0 {
                second = 0;
            } else if a[5] == 0 || a[6] == 0 {
                return INVALID;
            }
            checked(image, a[3], 4, true);
            checked(image, a[4], 4, true);
            if a[5] != 0 {
                checked(image, a[5], 4, true);
            }
            if a[6] != 0 {
                checked(image, a[6], 4, true);
            }
            unsafe {
                std::ptr::copy_nonoverlapping(v.data.as_ptr(), storage as *mut u8, v.data.len());
            }
            let regions = [
                storage + offset,
                first,
                if second == 0 { 0 } else { storage },
                second,
            ];
            write(image, a[3], &regions[..1]);
            write(image, a[4], &regions[1..2]);
            if a[5] != 0 {
                write(image, a[5], &regions[2..3]);
            }
            if a[6] != 0 {
                write(image, a[6], &regions[3..]);
            }
            if let Body::Buffer { locked, .. } = &mut s.objects.get_mut(&at).unwrap().body {
                *locked = Some(regions);
            }
            0
        }
        12 => {
            if a[1] != 0 || a[2] != 0 || a[3] & !5 != 0 {
                return INVALID;
            }
            if !v.playing(consumed) {
                v.start = produced;
                v.requested = true;
            }
            v.looping = a[3] & 1 != 0;
            0
        }
        13 => {
            if a[1] as usize >= v.data.len() || !(a[1] as usize).is_multiple_of(v.format.align()) {
                return INVALID;
            }
            v.base = a[1] as u64 / v.format.align() as u64;
            v.start = produced;
            0
        }
        14 => {
            if flags & 1 == 0 {
                return 0x88780032;
            }
            if let Body::Device { cooperative, .. } = s.objects[&parent].body
                && cooperative < 2
            {
                return 0x88780046;
            }

            let f = match format(image, a[1]) {
                Ok(f) => f,
                Err(_) => return 0x88780064,
            };
            v.format = f;
            v.frequency = f.rate;
            if let Body::Device { format, .. } = &mut s.objects.get_mut(&parent).unwrap().body {
                *format = f;
            }
            0
        }
        15 => {
            if flags & 0x80 == 0 {
                return CONTROL;
            }
            let value = a[1] as i32;
            if !(-10000..=0).contains(&value) {
                return INVALID;
            }
            v.volume = value;
            0
        }
        16 => {
            if flags & 0x40 == 0 {
                return CONTROL;
            }
            let value = a[1] as i32;
            if !(-10000..=10000).contains(&value) {
                return INVALID;
            }
            v.pan = value;
            0
        }
        17 => {
            if flags & 0x20 == 0 {
                return CONTROL;
            }
            let value = if a[1] == 0 { v.format.rate } else { a[1] };
            if !(100..=200000).contains(&value) {
                return INVALID;
            }
            v.base = v.position(consumed) as u64 / v.format.align() as u64;
            v.frequency = value;
            v.start = produced;
            0
        }
        18 => {
            v.stop(consumed);
            0
        }
        19 => {
            let Some(regions) = locked else {
                return 0x88780032;
            };
            if a[1] != regions[0]
                || a[2] > regions[1]
                || a[4] > regions[3]
                || (a[4] != 0 && a[3] != regions[2])
                || (a[4] == 0 && a[3] != 0 && a[3] != regions[2])
            {
                return INVALID;
            }
            for (pointer, count) in [(a[1], a[2]), (a[3], a[4])] {
                if count != 0 {
                    checked(image, pointer, count as usize, false);
                    let offset = (pointer - storage) as usize;
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            pointer as *const u8,
                            v.data.as_mut_ptr().add(offset),
                            count as usize,
                        );
                    }
                }
            }
            if let Body::Buffer { locked, .. } = &mut s.objects.get_mut(&at).unwrap().body {
                *locked = None;
            }
            0
        }
        20 => 0,
        21 => {
            if a[1] == 0 {
                0
            } else {
                0x887800dc
            }
        }
        22 | 23 => UNSUP,
        _ => unsupported("unimplemented DirectSound buffer method"),
    }
}
pub(super) fn clear_output(image: &LoadedImage32<'_>, pointer: u32) {
    write(image, pointer, &[0]);
}
pub(super) fn completion_event(image: &LoadedImage32<'_>, handle: u32) -> bool {
    image
        .cpu
        .dsound
        .borrow()
        .notification_events
        .values()
        .any(|v| v.contains(&handle))
}
pub(super) fn loaded(cpu: &Cpu32) -> bool {
    let s = cpu.dsound.borrow();
    s.static_loaded || s.module_refs != 0 || !s.objects.is_empty()
}
pub(super) fn load(image: &LoadedImage32<'_>) -> u32 {
    let mut builtins = image.cpu.builtins.borrow_mut();
    if !builtins.contains_key("dsound.dll") {
        let token = image
            .cpu
            .mapping(PAGE, PAGE, 0)
            .unwrap_or_else(|e| unsupported(&e));
        token
            .protect(0, PAGE, 0)
            .unwrap_or_else(|e| unsupported(&e));
        builtins.insert("dsound.dll".into(), token);
    }
    let base = builtins["dsound.dll"].base as u32;
    let mut s = image.cpu.dsound.borrow_mut();
    s.module_refs = s
        .module_refs
        .checked_add(1)
        .unwrap_or_else(|| unsupported("DirectSound DLL reference overflow"));
    base
}
pub(super) fn free(image: &LoadedImage32<'_>, base: u32) -> Option<u32> {
    let builtin = image
        .cpu
        .builtins
        .borrow()
        .get("dsound.dll")
        .map(|m| m.base as u32);
    if builtin != Some(base) {
        return None;
    }
    let mut s = image.cpu.dsound.borrow_mut();
    if s.module_refs == 0 {
        alkyhol::ffi::SetLastError(6);
        return Some(0);
    }
    s.module_refs -= 1;
    Some(1)
}
pub(super) fn report(image: &LoadedImage32<'_>) {
    let s = image.cpu.dsound.borrow();
    if let Some(out) = &s.output {
        let m = out.mixer.lock().unwrap();
        eprintln!(
            "DirectSound AudioQueue evidence: consumed_frames={} rendered_frames={} nonzero_frames={} voices={}",
            m.consumed,
            m.produced,
            m.nonzero,
            m.voices.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn listener_parameters_commit_deferred_fields_and_reject_invalid_changes_atomically() {
        let mut l = Listener::default();
        assert_eq!(l.active[9], 1f32.to_bits());
        assert_eq!(l.active[11], 1f32.to_bits());
        assert_eq!(
            l.update(1, &[1f32.to_bits(), 2f32.to_bits(), 3f32.to_bits()], 1),
            0
        );
        assert_eq!(&l.active[1..4], &[0; 3]);
        assert_eq!(l.update(13, &[2f32.to_bits()], 0), 0);
        l.commit();
        assert_eq!(l.active[1], 1f32.to_bits());
        assert_eq!(l.active[13], 2f32.to_bits());
        let before = l.pending;
        assert_eq!(
            l.update(1, &[4f32.to_bits(), f32::NAN.to_bits()], 0),
            INVALID
        );
        assert_eq!(l.update(13, &[0], 1), INVALID);
        assert_eq!(l.update(14, &[11f32.to_bits()], 1), INVALID);
        assert_eq!(l.update(15, &[(-1f32).to_bits()], 0), INVALID);
        assert_eq!(l.update(4, &[1f32.to_bits()], 2), INVALID);
        assert_eq!(l.active, before);
        assert_eq!(l.pending, before);
    }
}
