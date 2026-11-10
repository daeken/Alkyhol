//! Single-guest-thread kernel state used by the JIT marshaller.
use crate::{
    Result,
    memory::{Memory, READ, WRITE},
};
use alkyhol::kernel::{
    ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER, ERROR_NOT_ENOUGH_MEMORY, set_last_error,
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Copy)]
pub(crate) enum Call {
    GetEnvironmentVariableA,
    GetEnvironmentVariableW,
    SetEnvironmentVariableA,
    SetEnvironmentVariableW,
    GetEnvironmentStringsA,
    GetEnvironmentStringsW,
    FreeEnvironmentStringsA,
    FreeEnvironmentStringsW,
    GetVersion,
    GetVersionExA,
    GetVersionExW,
    GetProcessHeap,
    HeapCreate,
    HeapDestroy,
    HeapAlloc,
    HeapReAlloc,
    HeapFree,
    HeapSize,
    HeapValidate,
    InitializeCriticalSection,
    InitializeCriticalSectionAndSpinCount,
    InitializeCriticalSectionEx,
    EnterCriticalSection,
    LeaveCriticalSection,
    TryEnterCriticalSection,
    DeleteCriticalSection,
    TlsAlloc,
    TlsFree,
    TlsGetValue,
    TlsSetValue,
    EncodePointer,
    DecodePointer,
    GetCurrentProcessId,
    GetCurrentThreadId,
    GetSystemTimeAsFileTime,
    QueryPerformanceCounter,
    QueryPerformanceFrequency,
    GetTickCount,
}
impl Call {
    pub fn resolve(name: &str) -> Option<Self> {
        Some(match name {
            "GetEnvironmentVariableA" => Self::GetEnvironmentVariableA,
            "GetEnvironmentVariableW" => Self::GetEnvironmentVariableW,
            "SetEnvironmentVariableA" => Self::SetEnvironmentVariableA,
            "SetEnvironmentVariableW" => Self::SetEnvironmentVariableW,
            "GetEnvironmentStringsA" => Self::GetEnvironmentStringsA,
            "GetEnvironmentStringsW" => Self::GetEnvironmentStringsW,
            "FreeEnvironmentStringsA" => Self::FreeEnvironmentStringsA,
            "FreeEnvironmentStringsW" => Self::FreeEnvironmentStringsW,
            "GetVersion" => Self::GetVersion,
            "GetVersionExA" => Self::GetVersionExA,
            "GetVersionExW" => Self::GetVersionExW,
            "GetProcessHeap" => Self::GetProcessHeap,
            "HeapCreate" => Self::HeapCreate,
            "HeapDestroy" => Self::HeapDestroy,
            "HeapAlloc" => Self::HeapAlloc,
            "HeapReAlloc" => Self::HeapReAlloc,
            "HeapFree" => Self::HeapFree,
            "HeapSize" => Self::HeapSize,
            "HeapValidate" => Self::HeapValidate,
            "InitializeCriticalSection" => Self::InitializeCriticalSection,
            "InitializeCriticalSectionAndSpinCount" => Self::InitializeCriticalSectionAndSpinCount,
            "InitializeCriticalSectionEx" => Self::InitializeCriticalSectionEx,
            "EnterCriticalSection" => Self::EnterCriticalSection,
            "LeaveCriticalSection" => Self::LeaveCriticalSection,
            "TryEnterCriticalSection" => Self::TryEnterCriticalSection,
            "DeleteCriticalSection" => Self::DeleteCriticalSection,
            "TlsAlloc" => Self::TlsAlloc,
            "TlsFree" => Self::TlsFree,
            "TlsGetValue" => Self::TlsGetValue,
            "TlsSetValue" => Self::TlsSetValue,
            "EncodePointer" => Self::EncodePointer,
            "DecodePointer" => Self::DecodePointer,
            "GetCurrentProcessId" => Self::GetCurrentProcessId,
            "GetCurrentThreadId" => Self::GetCurrentThreadId,
            "GetSystemTimeAsFileTime" => Self::GetSystemTimeAsFileTime,
            "QueryPerformanceCounter" => Self::QueryPerformanceCounter,
            "QueryPerformanceFrequency" => Self::QueryPerformanceFrequency,
            "GetTickCount" => Self::GetTickCount,
            _ => return None,
        })
    }
    pub fn args(self) -> u32 {
        match self {
            Self::GetEnvironmentStringsA
            | Self::GetEnvironmentStringsW
            | Self::GetVersion
            | Self::GetProcessHeap
            | Self::TlsAlloc
            | Self::GetCurrentProcessId
            | Self::GetCurrentThreadId
            | Self::GetTickCount => 0,
            Self::FreeEnvironmentStringsA
            | Self::FreeEnvironmentStringsW
            | Self::GetVersionExA
            | Self::GetVersionExW
            | Self::HeapDestroy
            | Self::InitializeCriticalSection
            | Self::EnterCriticalSection
            | Self::LeaveCriticalSection
            | Self::TryEnterCriticalSection
            | Self::DeleteCriticalSection
            | Self::TlsFree
            | Self::TlsGetValue
            | Self::EncodePointer
            | Self::DecodePointer
            | Self::GetSystemTimeAsFileTime
            | Self::QueryPerformanceCounter
            | Self::QueryPerformanceFrequency => 1,
            Self::SetEnvironmentVariableA
            | Self::SetEnvironmentVariableW
            | Self::InitializeCriticalSectionAndSpinCount
            | Self::TlsSetValue => 2,
            Self::HeapReAlloc => 4,
            _ => 3,
        }
    }
}
#[derive(Default)]
struct Heap {
    maximum: Option<u32>,
    allocations: BTreeMap<u32, u32>,
    used: u64,
}
#[derive(Default)]
struct Critical {
    owner: u32,
    depth: u32,
    spin: u32,
}
pub(crate) struct State {
    heaps: BTreeMap<u32, Heap>,
    next_heap: u32,
    next_memory: u32,
    critical: BTreeMap<u32, Critical>,
    tls: BTreeMap<u32, u32>,
    key: u32,
    pub legacy_nt62: bool,
    pub environment: BTreeMap<String, String>,
    environment_blocks: BTreeSet<u32>,
}
fn fail(error: u32) -> u32 {
    set_last_error(error);
    0
}
fn output(memory: &mut Memory, address: u32, value: u32) -> Result<()> {
    memory.write(address as u64, &value.to_le_bytes())
}
fn text(memory: &Memory, address: u32, wide: bool) -> Result<String> {
    let mut bytes = vec![];
    let width = if wide { 2 } else { 1 };
    for i in 0..32768u64 {
        let raw = memory.read(address as u64 + i * width, width as usize, READ)?;
        let value = if wide {
            u16::from_le_bytes(raw.try_into().unwrap())
        } else {
            raw[0] as u16
        };
        if value == 0 {
            return String::from_utf8(bytes).map_err(|e| e.to_string());
        }
        if value > 127 {
            return Err("guest environment currently requires ASCII text".into());
        }
        bytes.push(value as u8);
    }
    Err("unterminated environment text".into())
}
impl Default for State {
    fn default() -> Self {
        use std::hash::{BuildHasher, Hasher};
        Self {
            legacy_nt62: false,
            environment: BTreeMap::new(),
            environment_blocks: BTreeSet::new(),
            heaps: BTreeMap::from([(0x200, Heap::default())]),
            next_heap: 0x201,
            next_memory: 0x71000000,
            critical: BTreeMap::new(),
            tls: BTreeMap::new(),
            key: std::collections::hash_map::RandomState::new()
                .build_hasher()
                .finish() as u32
                | 1,
        }
    }
}
impl State {
    fn allocate(&mut self, memory: &mut Memory, heap: u32, flags: u32, size: u32) -> Result<u32> {
        if flags & !9 != 0 {
            return Err("HeapAlloc flags outside admitted zero-memory/no-serialize profile".into());
        }
        let Some(h) = self.heaps.get(&heap) else {
            return Ok(fail(ERROR_INVALID_HANDLE));
        };
        if size > 64 * 1024 * 1024
            || h.maximum
                .is_some_and(|max| h.used + size as u64 > max as u64)
        {
            return Ok(fail(ERROR_NOT_ENOUGH_MEMORY));
        }
        let start = self
            .next_memory
            .checked_add(15)
            .map(|v| v & !15)
            .ok_or("heap address overflow")?;
        let Some(end) = start.checked_add(size.max(1)).filter(|&v| v <= 0x7d000000) else {
            return Ok(fail(ERROR_NOT_ENOUGH_MEMORY));
        };
        memory.map(start, size.max(1) as usize, READ | WRITE, &[])?;
        self.next_memory = end;
        let h = self.heaps.get_mut(&heap).unwrap();
        h.allocations.insert(start, size);
        h.used += size as u64;
        Ok(start)
    }
    fn release(&mut self, memory: &mut Memory, heap: u32, pointer: u32) -> Result<u32> {
        let Some(h) = self.heaps.get_mut(&heap) else {
            return Ok(fail(ERROR_INVALID_HANDLE));
        };
        if pointer == 0 {
            return Ok(1);
        }
        let Some(size) = h.allocations.get(&pointer).copied() else {
            return Ok(fail(ERROR_INVALID_PARAMETER));
        };
        memory.unmap(pointer, size.max(1) as usize)?;
        h.allocations.remove(&pointer);
        h.used -= size as u64;
        Ok(1)
    }
    fn critical_bytes(memory: &mut Memory, address: u32, c: &Critical) -> Result<()> {
        let lock = if c.depth == 0 { u32::MAX } else { c.depth - 1 };
        let words = [0, lock, c.depth, c.owner, 0, c.spin];
        memory.write(
            address as u64,
            &words
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>(),
        )
    }
    pub fn dispatch(
        &mut self,
        call: Call,
        a: &[u32; 5],
        memory: &mut Memory,
        teb: Option<u32>,
    ) -> Result<u32> {
        use Call::*;
        Ok(match call {
            GetEnvironmentVariableA | GetEnvironmentVariableW => {
                let wide = matches!(call, GetEnvironmentVariableW);
                let key = text(memory, a[0], wide)?.to_ascii_uppercase();
                let Some(value) = self.environment.get(&key) else {
                    return Ok(fail(203));
                };
                let required = value.len() as u32 + 1;
                if a[2] < required {
                    required
                } else {
                    let mut bytes = value.as_bytes().to_vec();
                    bytes.push(0);
                    let bytes = if wide {
                        bytes
                            .iter()
                            .flat_map(|&b| (b as u16).to_le_bytes())
                            .collect::<Vec<_>>()
                    } else {
                        bytes
                    };
                    memory.write(a[1] as u64, &bytes)?;
                    required - 1
                }
            }
            SetEnvironmentVariableA | SetEnvironmentVariableW => {
                let wide = matches!(call, SetEnvironmentVariableW);
                let key = text(memory, a[0], wide)?.to_ascii_uppercase();
                if key.is_empty() || key.contains('=') {
                    return Ok(fail(ERROR_INVALID_PARAMETER));
                }
                if a[1] == 0 {
                    self.environment.remove(&key);
                } else {
                    let value = text(memory, a[1], wide)?;
                    if self.environment.len() >= 1024 && !self.environment.contains_key(&key) {
                        return Err("guest environment entry budget exceeded".into());
                    }
                    self.environment.insert(key, value);
                }
                1
            }
            GetEnvironmentStringsA | GetEnvironmentStringsW => {
                let mut bytes = vec![];
                for (key, value) in &self.environment {
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.push(b'=');
                    bytes.extend_from_slice(value.as_bytes());
                    bytes.push(0);
                }
                if bytes.is_empty() {
                    bytes.push(0);
                }
                bytes.push(0);
                if matches!(call, GetEnvironmentStringsW) {
                    bytes = bytes
                        .iter()
                        .flat_map(|&b| (b as u16).to_le_bytes())
                        .collect();
                }
                if bytes.len() > 1024 * 1024 {
                    return Err("environment block budget exceeded".into());
                }
                let ptr = self.allocate(memory, 0x200, 8, bytes.len() as u32)?;
                if ptr == 0 {
                    return Ok(0);
                }
                memory.write(ptr as u64, &bytes)?;
                self.environment_blocks.insert(ptr);
                ptr
            }
            FreeEnvironmentStringsA | FreeEnvironmentStringsW => {
                if !self.environment_blocks.remove(&a[0]) {
                    fail(ERROR_INVALID_PARAMETER)
                } else {
                    self.release(memory, 0x200, a[0])?
                }
            }
            GetVersion => {
                if !self.legacy_nt62 {
                    return Err("explicit legacy NT 6.2 guest persona required".into());
                }
                0x23f00206
            }
            GetVersionExA | GetVersionExW => {
                if !self.legacy_nt62 {
                    return Err("explicit legacy NT 6.2 guest persona required".into());
                }
                let size = memory.word(a[0] as u64)?;
                let basic = if matches!(call, GetVersionExA) {
                    148
                } else {
                    276
                };
                if size != basic && size != basic + 8 {
                    fail(ERROR_INVALID_PARAMETER)
                } else {
                    let mut bytes = vec![0u8; size as usize];
                    for (i, v) in [size, 6, 2, 9200, 2].iter().enumerate() {
                        bytes[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
                    }
                    if size == basic + 8 {
                        bytes[basic as usize + 6] = 1;
                    }
                    memory.write(a[0] as u64, &bytes)?;
                    1
                }
            }
            GetProcessHeap => 0x200,
            HeapCreate => {
                if a[0] & !1 != 0 {
                    return Err("HeapCreate options unsupported".into());
                }
                if a[1] > 64 * 1024 * 1024 {
                    return Ok(fail(ERROR_NOT_ENOUGH_MEMORY));
                }
                if a[2] != 0 && a[1] > a[2] {
                    fail(ERROR_INVALID_PARAMETER)
                } else {
                    let id = self.next_heap;
                    self.next_heap = self.next_heap.checked_add(1).ok_or("heap token budget")?;
                    if self.heaps.len() >= 1024 {
                        return Ok(fail(ERROR_NOT_ENOUGH_MEMORY));
                    }
                    self.heaps.insert(
                        id,
                        Heap {
                            maximum: if a[2] == 0 { None } else { Some(a[2]) },
                            ..Heap::default()
                        },
                    );
                    id
                }
            }
            HeapAlloc => self.allocate(memory, a[0], a[1], a[2])?,
            HeapFree => {
                if a[1] & !1 != 0 {
                    return Err("HeapFree flags unsupported".into());
                }
                self.release(memory, a[0], a[2])?
            }
            HeapSize | HeapValidate => {
                if a[1] & !1 != 0 {
                    return Err("heap inspection flags unsupported".into());
                }
                let valid = self
                    .heaps
                    .get(&a[0])
                    .and_then(|h| h.allocations.get(&a[2]))
                    .copied();
                match call {
                    HeapSize => valid.unwrap_or_else(|| {
                        set_last_error(ERROR_INVALID_PARAMETER);
                        u32::MAX
                    }),
                    _ => {
                        if valid.is_some() || a[2] == 0 && self.heaps.contains_key(&a[0]) {
                            1
                        } else {
                            fail(ERROR_INVALID_PARAMETER)
                        }
                    }
                }
            }
            HeapReAlloc => {
                if a[1] & !0x19 != 0 {
                    return Err(
                        "HeapReAlloc flags outside admitted zero-memory/in-place profile".into(),
                    );
                }
                let Some(old) = self
                    .heaps
                    .get(&a[0])
                    .and_then(|h| h.allocations.get(&a[2]))
                    .copied()
                else {
                    return Ok(fail(ERROR_INVALID_PARAMETER));
                };
                let bytes = if old.min(a[3]) == 0 {
                    vec![]
                } else {
                    memory
                        .read(a[2] as u64, old.min(a[3]) as usize, READ)?
                        .to_vec()
                };
                if a[1] & 16 != 0 {
                    if a[3] > old {
                        return Ok(fail(ERROR_NOT_ENOUGH_MEMORY));
                    }
                    memory.shrink(a[2], old.max(1) as usize, a[3].max(1) as usize)?;
                    let h = self.heaps.get_mut(&a[0]).unwrap();
                    h.used -= old as u64 - a[3] as u64;
                    h.allocations.insert(a[2], a[3]);
                    return Ok(a[2]);
                }
                // Replacement quota counts the resulting allocation, not both
                // old and new simultaneously; preserve the old block on failure.
                self.heaps.get_mut(&a[0]).unwrap().used -= old as u64;
                let allocated = self.allocate(memory, a[0], a[1], a[3]);
                self.heaps.get_mut(&a[0]).unwrap().used += old as u64;
                let new = allocated?;
                if new == 0 {
                    return Ok(0);
                }
                if !bytes.is_empty() {
                    memory.write(new as u64, &bytes)?;
                }
                self.release(memory, a[0], a[2])?;
                new
            }
            HeapDestroy => {
                if a[0] == 0x200 {
                    return Ok(fail(ERROR_INVALID_PARAMETER));
                }
                let Some(h) = self.heaps.remove(&a[0]) else {
                    return Ok(fail(ERROR_INVALID_HANDLE));
                };
                for (pointer, size) in h.allocations {
                    memory.unmap(pointer, size.max(1) as usize)?;
                }
                1
            }
            InitializeCriticalSection
            | InitializeCriticalSectionAndSpinCount
            | InitializeCriticalSectionEx => {
                if matches!(call, InitializeCriticalSectionEx) && a[2] & !0x01000000 != 0 {
                    return Err("critical section flags unsupported".into());
                }
                if self.critical.contains_key(&a[0]) {
                    return Err("critical section initialized twice".into());
                }
                let c = Critical {
                    spin: if matches!(call, InitializeCriticalSection) {
                        0
                    } else {
                        a[1]
                    },
                    ..Critical::default()
                };
                Self::critical_bytes(memory, a[0], &c)?;
                self.critical.insert(a[0], c);
                1
            }
            EnterCriticalSection
            | TryEnterCriticalSection
            | LeaveCriticalSection
            | DeleteCriticalSection => {
                memory
                    .check(a[0] as u64, 24, WRITE)
                    .map_err(|f| format!("critical section fault: {f:?}"))?;
                let c = self
                    .critical
                    .get_mut(&a[0])
                    .ok_or("critical section was not initialized")?;
                let owner = alkyhol::kernel::current_thread_id();
                if matches!(call, DeleteCriticalSection) {
                    if c.depth != 0 {
                        return Err("deleting owned critical section".into());
                    }
                    self.critical.remove(&a[0]);
                    1
                } else {
                    if matches!(call, LeaveCriticalSection) {
                        if c.depth == 0 || c.owner != owner {
                            return Err("critical section leave without ownership".into());
                        }
                        c.depth -= 1;
                        if c.depth == 0 {
                            c.owner = 0;
                        }
                    } else {
                        if c.depth != 0 && c.owner != owner {
                            return Err(
                                "contended multi-guest-thread critical section unsupported".into(),
                            );
                        }
                        c.depth = c
                            .depth
                            .checked_add(1)
                            .ok_or("critical section recursion overflow")?;
                        c.owner = owner;
                    }
                    Self::critical_bytes(memory, a[0], c)?;
                    1
                }
            }
            TlsAlloc => {
                let free = (0..64).find(|i| !self.tls.contains_key(i));
                if let Some(i) = free {
                    self.tls.insert(i, 0);
                    if let Some(teb) = teb {
                        output(memory, teb + 0xe10 + i * 4, 0)?;
                    }
                    i
                } else {
                    set_last_error(ERROR_NOT_ENOUGH_MEMORY);
                    u32::MAX
                }
            }
            TlsFree => {
                if self.tls.remove(&a[0]).is_none() {
                    fail(ERROR_INVALID_PARAMETER)
                } else {
                    if let Some(teb) = teb {
                        output(memory, teb + 0xe10 + a[0] * 4, 0)?;
                    }
                    1
                }
            }
            TlsGetValue => match self.tls.get(&a[0]) {
                Some(&value) => {
                    let value = if let Some(teb) = teb {
                        memory.word(teb as u64 + 0xe10 + a[0] as u64 * 4)?
                    } else {
                        value
                    };
                    set_last_error(0);
                    value
                }
                None => fail(ERROR_INVALID_PARAMETER),
            },
            TlsSetValue => {
                if let Some(value) = self.tls.get_mut(&a[0]) {
                    *value = a[1];
                    if let Some(teb) = teb {
                        output(memory, teb + 0xe10 + a[0] * 4, a[1])?;
                    }
                    1
                } else {
                    fail(ERROR_INVALID_PARAMETER)
                }
            }
            EncodePointer => (a[0] ^ self.key).rotate_right(11),
            DecodePointer => a[0].rotate_left(11) ^ self.key,
            GetCurrentProcessId => std::process::id(),
            GetCurrentThreadId => alkyhol::kernel::current_thread_id(),
            GetSystemTimeAsFileTime => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| e.to_string())?;
                let value = u64::try_from(now.as_nanos() / 100)
                    .map_err(|_| "FILETIME overflow")?
                    .checked_add(116444736000000000)
                    .ok_or("FILETIME overflow")?;
                memory.write(a[0] as u64, &value.to_le_bytes())?;
                0
            }
            QueryPerformanceCounter => {
                memory.write(
                    a[0] as u64,
                    &alkyhol::kernel::performance_counter().to_le_bytes(),
                )?;
                1
            }
            QueryPerformanceFrequency => {
                memory.write(a[0] as u64, &1_000_000_000u64.to_le_bytes())?;
                1
            }
            GetTickCount => {
                let mut time = libc::timespec {
                    tv_sec: 0,
                    tv_nsec: 0,
                };
                if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) } != 0 {
                    return Err(std::io::Error::last_os_error().to_string());
                }
                (time.tv_sec as u64 * 1000 + time.tv_nsec as u64 / 1_000_000) as u32
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (State, Memory) {
        let mut m = Memory::new().unwrap();
        m.map(0x10000, 4096, READ | WRITE, &[]).unwrap();
        m.map(0x20000, 4096, READ | WRITE, &[]).unwrap();
        (State::default(), m)
    }
    fn call(s: &mut State, m: &mut Memory, c: Call, args: &[u32]) -> u32 {
        let mut a = [0; 5];
        a[..args.len()].copy_from_slice(args);
        s.dispatch(c, &a, m, Some(0x10000)).unwrap()
    }
    #[test]
    fn heaps_preserve_bytes_enforce_ownership_and_revoke_freed_storage() {
        let (mut s, mut m) = setup();
        let h = call(&mut s, &mut m, Call::GetProcessHeap, &[]);
        assert_eq!(h, call(&mut s, &mut m, Call::GetProcessHeap, &[]));
        set_last_error(0xface1234);
        assert_eq!(call(&mut s, &mut m, Call::HeapFree, &[h, 1, 0]), 1);
        assert_eq!(alkyhol::kernel::get_last_error(), 0xface1234);
        let other = call(&mut s, &mut m, Call::HeapCreate, &[0, 0, 32]);
        let p = call(&mut s, &mut m, Call::HeapAlloc, &[other, 8, 16]);
        assert!(p != 0 && p & 15 == 0);
        assert_eq!(m.read(p as u64, 16, READ).unwrap(), [0; 16]);
        m.write(p as u64, &[1, 2, 3, 4]).unwrap();
        assert_eq!(call(&mut s, &mut m, Call::HeapSize, &[other, 0, p]), 16);
        assert_eq!(call(&mut s, &mut m, Call::HeapValidate, &[other, 0, p]), 1);
        assert_eq!(call(&mut s, &mut m, Call::HeapFree, &[h, 0, p]), 0);
        let q = call(&mut s, &mut m, Call::HeapReAlloc, &[other, 8, p, 32]);
        assert_ne!(q, 0);
        assert_eq!(m.read(q as u64, 4, READ).unwrap(), [1, 2, 3, 4]);
        assert!(m.read(p as u64, 1, READ).is_err());
        assert_eq!(m.read(q as u64 + 16, 16, READ).unwrap(), [0; 16]);
        assert_eq!(
            call(&mut s, &mut m, Call::HeapReAlloc, &[other, 16, q, 3]),
            q
        );
        assert!(m.read(q as u64 + 3, 1, READ).is_err());
        assert_eq!(call(&mut s, &mut m, Call::HeapAlloc, &[other, 0, 30]), 0); // max32, live3
        assert_eq!(
            call(&mut s, &mut m, Call::HeapReAlloc, &[other, 16, q, 5]),
            0
        );
        assert_eq!(m.read(q as u64, 3, READ).unwrap(), [1, 2, 3]);
        assert_eq!(call(&mut s, &mut m, Call::HeapDestroy, &[other]), 1);
        assert!(m.read(q as u64, 1, READ).is_err());
        assert_eq!(call(&mut s, &mut m, Call::HeapDestroy, &[h]), 0);
        assert!(
            s.dispatch(Call::HeapCreate, &[0x40000, 0, 0, 0, 0], &mut m, None)
                .is_err()
        );
        let zero = call(&mut s, &mut m, Call::HeapAlloc, &[h, 0, 0]);
        assert_ne!(zero, 0);
        assert_eq!(call(&mut s, &mut m, Call::HeapFree, &[h, 0, zero]), 1);
        assert!(m.word(zero as u64).is_err());
    }
    #[test]
    fn recursive_critical_sections_have_real_state_and_reject_invalid_transitions() {
        let (mut s, mut m) = setup();
        for (offset, init) in [
            (0, Call::InitializeCriticalSection),
            (32, Call::InitializeCriticalSectionAndSpinCount),
            (64, Call::InitializeCriticalSectionEx),
        ] {
            let at = 0x20000 + offset;
            assert_eq!(call(&mut s, &mut m, init, &[at, 4000, 0]), 1);
            assert_eq!(m.word(at as u64 + 4).unwrap(), u32::MAX);
            call(&mut s, &mut m, Call::EnterCriticalSection, &[at]);
            call(&mut s, &mut m, Call::EnterCriticalSection, &[at]);
            assert_eq!(
                call(&mut s, &mut m, Call::TryEnterCriticalSection, &[at]),
                1
            );
            assert_eq!(m.word(at as u64 + 8).unwrap(), 3);
            assert_eq!(
                m.word(at as u64 + 12).unwrap(),
                alkyhol::kernel::current_thread_id()
            );
            assert!(
                s.dispatch(Call::DeleteCriticalSection, &[at, 0, 0, 0, 0], &mut m, None)
                    .is_err()
            );
            for _ in 0..3 {
                call(&mut s, &mut m, Call::LeaveCriticalSection, &[at]);
            }
            assert_eq!(m.word(at as u64 + 4).unwrap(), u32::MAX);
            assert!(
                s.dispatch(Call::LeaveCriticalSection, &[at, 0, 0, 0, 0], &mut m, None)
                    .is_err()
            );
            call(&mut s, &mut m, Call::DeleteCriticalSection, &[at]);
            assert!(
                s.dispatch(Call::EnterCriticalSection, &[at, 0, 0, 0, 0], &mut m, None)
                    .is_err()
            );
        }
    }
    #[test]
    fn dynamic_tls_slots_are_backed_reused_and_clear_last_error_on_read() {
        let (mut s, mut m) = setup();
        for i in 0..64 {
            assert_eq!(call(&mut s, &mut m, Call::TlsAlloc, &[]), i);
        }
        assert_eq!(call(&mut s, &mut m, Call::TlsAlloc, &[]), u32::MAX);
        call(&mut s, &mut m, Call::TlsSetValue, &[7, 0x12345678]);
        assert_eq!(m.word(0x10e10 + 28).unwrap(), 0x12345678);
        m.write(0x10e10 + 28, &0xabcdef01u32.to_le_bytes()).unwrap();
        set_last_error(99);
        assert_eq!(call(&mut s, &mut m, Call::TlsGetValue, &[7]), 0xabcdef01);
        assert_eq!(alkyhol::kernel::get_last_error(), 0);
        assert_eq!(call(&mut s, &mut m, Call::TlsFree, &[7]), 1);
        assert_eq!(call(&mut s, &mut m, Call::TlsAlloc, &[]), 7);
        assert_eq!(call(&mut s, &mut m, Call::TlsGetValue, &[7]), 0);
        assert_eq!(call(&mut s, &mut m, Call::TlsSetValue, &[65, 1]), 0);
    }
    #[test]
    fn pointer_codec_is_stable_and_clocks_write_actual_outputs() {
        let (mut s, mut m) = setup();
        for value in [0, 1, 0xffffffff, 0xabcdef01] {
            let encoded = call(&mut s, &mut m, Call::EncodePointer, &[value]);
            assert_eq!(encoded, call(&mut s, &mut m, Call::EncodePointer, &[value]));
            assert_eq!(call(&mut s, &mut m, Call::DecodePointer, &[encoded]), value);
        }
        assert_eq!(
            call(&mut s, &mut m, Call::GetCurrentProcessId, &[]),
            std::process::id()
        );
        assert_eq!(
            call(&mut s, &mut m, Call::GetCurrentThreadId, &[]),
            alkyhol::kernel::current_thread_id()
        );
        call(&mut s, &mut m, Call::QueryPerformanceFrequency, &[0x20000]);
        assert_eq!(m.word(0x20000).unwrap(), 1_000_000_000);
        call(&mut s, &mut m, Call::QueryPerformanceCounter, &[0x20008]);
        let before = u64::from_le_bytes(m.read(0x20008, 8, READ).unwrap().try_into().unwrap());
        call(&mut s, &mut m, Call::QueryPerformanceCounter, &[0x20008]);
        assert!(
            u64::from_le_bytes(m.read(0x20008, 8, READ).unwrap().try_into().unwrap()) >= before
        );
        let before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        call(&mut s, &mut m, Call::GetSystemTimeAsFileTime, &[0x20010]);
        let ticks = u64::from_le_bytes(m.read(0x20010, 8, READ).unwrap().try_into().unwrap());
        assert!(ticks / 10_000_000 - 11644473600 >= before);
        let a = call(&mut s, &mut m, Call::GetTickCount, &[]);
        let b = call(&mut s, &mut m, Call::GetTickCount, &[]);
        assert!(b.wrapping_sub(a) < 1000);
    }
    #[test]
    fn environment_is_owned_case_insensitive_and_sizes_are_in_characters() {
        let (mut s, mut m) = setup();
        m.write(0x20000, b"MiXeD\0value\0").unwrap();
        assert_eq!(
            call(
                &mut s,
                &mut m,
                Call::GetEnvironmentVariableA,
                &[0x20000, 0, 0]
            ),
            0
        );
        assert_eq!(alkyhol::kernel::get_last_error(), 203);
        call(
            &mut s,
            &mut m,
            Call::SetEnvironmentVariableA,
            &[0x20000, 0x20006],
        );
        assert_eq!(
            call(
                &mut s,
                &mut m,
                Call::GetEnvironmentVariableA,
                &[0x20000, 0, 0]
            ),
            6
        );
        m.write(0x20020, &[0x5a; 12]).unwrap();
        assert_eq!(
            call(
                &mut s,
                &mut m,
                Call::GetEnvironmentVariableA,
                &[0x20000, 0x20020, 5]
            ),
            6
        );
        assert_eq!(m.read(0x20020, 6, READ).unwrap(), [0x5a; 6]);
        assert_eq!(
            call(
                &mut s,
                &mut m,
                Call::GetEnvironmentVariableA,
                &[0x20000, 0x20020, 6]
            ),
            5
        );
        assert_eq!(m.read(0x20020, 6, READ).unwrap(), b"value\0");
        let wide = "mixed\0"
            .encode_utf16()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>();
        m.write(0x20040, &wide).unwrap();
        assert_eq!(
            call(
                &mut s,
                &mut m,
                Call::GetEnvironmentVariableW,
                &[0x20040, 0x20060, 6]
            ),
            5
        );
        assert_eq!(m.read(0x20060, 12, READ).unwrap(), b"v\0a\0l\0u\0e\0\0\0");
        for c in [Call::GetEnvironmentStringsA, Call::GetEnvironmentStringsW] {
            let p = call(&mut s, &mut m, c, &[]);
            assert_ne!(p, 0);
            assert_eq!(call(&mut s, &mut m, Call::FreeEnvironmentStringsW, &[p]), 1);
            assert!(m.read(p as u64, 1, READ).is_err());
        }
        call(&mut s, &mut m, Call::SetEnvironmentVariableW, &[0x20040, 0]);
        assert!(s.environment.is_empty());
        m.write(
            0x20080,
            &b"other\0wide\0"
                .iter()
                .flat_map(|&v| (v as u16).to_le_bytes())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        call(
            &mut s,
            &mut m,
            Call::SetEnvironmentVariableW,
            &[0x20080, 0x2008c],
        );
        let p = call(&mut s, &mut m, Call::GetEnvironmentStringsA, &[]);
        assert_eq!(m.read(p as u64, 12, READ).unwrap(), b"OTHER=wide\0\0");
        call(&mut s, &mut m, Call::FreeEnvironmentStringsA, &[p]);
    }
    #[test]
    fn version_persona_is_explicit_and_structures_keep_exact_layout() {
        let (mut s, mut m) = setup();
        assert!(s.dispatch(Call::GetVersion, &[0; 5], &mut m, None).is_err());
        s.legacy_nt62 = true;
        assert_eq!(call(&mut s, &mut m, Call::GetVersion, &[]), 0x23f00206);
        for (c, size) in [
            (Call::GetVersionExA, 148),
            (Call::GetVersionExA, 156),
            (Call::GetVersionExW, 276),
            (Call::GetVersionExW, 284),
        ] {
            output(&mut m, 0x20000, size).unwrap();
            assert_eq!(call(&mut s, &mut m, c, &[0x20000]), 1);
            assert_eq!(m.word(0x20004).unwrap(), 6);
            assert_eq!(m.word(0x20008).unwrap(), 2);
            assert_eq!(m.word(0x2000c).unwrap(), 9200);
            if size == 156 || size == 284 {
                assert_eq!(m.read(0x20000 + size as u64 - 2, 1, READ).unwrap(), [1]);
            }
        }
        output(&mut m, 0x20000, 3).unwrap();
        assert_eq!(call(&mut s, &mut m, Call::GetVersionExA, &[0x20000]), 0);
        assert!(Call::resolve("missing").is_none());
    }
}
