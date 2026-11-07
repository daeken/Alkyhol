//! Bounded version-1.0 root-signature layout/serialization, not shader execution.
//! Binary layout follows open DXC (NCSA); attribution: cpp/root-signatures.md.
#![allow(non_snake_case, clippy::missing_safety_doc)]
use crate::com::{Blob, D3DCreateBlob, E_POINTER, S_OK};
use std::{ffi::c_void, ptr};
const INVALID: i32 = 0x80070057u32 as i32;
const UNSUPPORTED: i32 = 0x80004001u32 as i32;
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error {
    pub code: i32,
    pub message: &'static str,
}
fn invalid(message: &'static str) -> Error {
    Error {
        code: INVALID,
        message,
    }
}
fn unsupported(message: &'static str) -> Error {
    Error {
        code: UNSUPPORTED,
        message,
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Binding {
    Table(Vec<[u32; 5]>),
    Constants {
        register: u32,
        space: u32,
        count: u32,
    },
    Descriptor {
        kind: u32,
        register: u32,
        space: u32,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub visibility: u32,
    pub binding: Binding,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub parameters: Vec<Parameter>,
    pub samplers: Vec<[u32; 13]>,
    pub flags: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Table {
    pub count: u32,
    pub ranges: *const [u32; 5],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union Payload {
    pub table: Table,
    pub constants: [u32; 3],
    pub descriptor: [u32; 2],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RawParameter {
    pub kind: u32,
    pub payload: Payload,
    pub visibility: u32,
}
#[repr(C)]
pub struct Description {
    pub count: u32,
    pub parameters: *const RawParameter,
    pub sampler_count: u32,
    pub samplers: *const [u32; 13],
    pub flags: u32,
}
#[repr(C)]
pub struct VersionedDescription {
    pub version: u32,
    pub description: Description,
}

// Independent binding intervals, including constants as CBVs and static samplers.
type Interval = (u32, u32, u32, u32, u32);
fn interval(
    rows: &mut Vec<Interval>,
    class: u32,
    space: u32,
    first: u32,
    count: u32,
    vis: u32,
) -> Result<()> {
    if space >= 0xfffffff0 {
        return Err(unsupported("reserved register spaces"));
    }
    let end = first
        .checked_add(count)
        .filter(|_| count != 0)
        .ok_or(invalid("empty or overflowing register range"))?;
    if rows.iter().any(|&(c, s, a, b, v)| {
        c == class && s == space && first < b && a < end && (vis == 0 || v == 0 || vis == v)
    }) {
        return Err(invalid("overlapping shader registers"));
    }
    rows.push((class, space, first, end, vis));
    Ok(())
}
impl Layout {
    pub fn validate(&self) -> Result<()> {
        if self.parameters.len() > 64 || self.samplers.len() > 16 || self.flags & !0x7f != 0 {
            return Err(unsupported("root layout count/flag subset"));
        }
        let mut rows = Vec::new();
        let mut cost = 0;
        for p in &self.parameters {
            if p.visibility > 5 {
                return Err(unsupported("shader visibility subset"));
            }
            match &p.binding {
                Binding::Table(ranges) => {
                    cost += 1;
                    if ranges.is_empty() {
                        return Err(invalid("empty descriptor table"));
                    }
                    if ranges.len() > 64 {
                        return Err(unsupported("descriptor range count"));
                    }
                    let sampler = ranges[0][0] == 3;
                    let mut occupied = Vec::new();
                    let mut previous = 0;
                    for &[class, count, register, space, offset] in ranges {
                        if class > 3 || (class == 3) != sampler {
                            return Err(invalid("mixed or invalid descriptor table type"));
                        }
                        if count == u32::MAX {
                            return Err(unsupported("unbounded descriptor range"));
                        }
                        let start = if offset == u32::MAX { previous } else { offset };
                        let end = start
                            .checked_add(count)
                            .ok_or(invalid("descriptor offset overflow"))?;
                        if end > 1_000_000 {
                            return Err(unsupported("descriptor table extent"));
                        }
                        if occupied.iter().any(|&(a, b)| start < b && a < end) {
                            return Err(invalid("overlapping descriptor offsets"));
                        }
                        occupied.push((start, end));
                        previous = end;
                        interval(&mut rows, class, space, register, count, p.visibility)?;
                    }
                }
                Binding::Constants {
                    register,
                    space,
                    count,
                } => {
                    if *count == 0 || *count > 64 {
                        return Err(invalid("root constant count"));
                    }
                    cost += count;
                    interval(&mut rows, 2, *space, *register, 1, p.visibility)?;
                }
                Binding::Descriptor {
                    kind,
                    register,
                    space,
                } => {
                    let class = match kind {
                        2 => 2,
                        3 => 0,
                        4 => 1,
                        _ => return Err(invalid("root descriptor type")),
                    };
                    cost += 2;
                    interval(&mut rows, class, *space, *register, 1, p.visibility)?;
                }
            }
        }
        if cost > 64 {
            return Err(invalid("root signature exceeds 64 DWORDs"));
        }
        for s in &self.samplers {
            if ![0, 1, 4, 5, 16, 17, 20, 21, 85].contains(&(s[0] & 0x7f))
                || s[0] & !0x1ff != 0
                || s[1..4].iter().any(|v| !(1..=5).contains(v))
                || !(1..=8).contains(&s[6])
                || s[7] > 2
            {
                return Err(invalid("static sampler fields"));
            }
            if s[12] > 5 {
                return Err(unsupported("static sampler visibility"));
            }
            if s[5] > 16 || (s[0] & 0x7f == 85 && s[5] == 0) {
                return Err(invalid("static sampler anisotropy"));
            }
            let bias = f32::from_bits(s[4]);
            let min = f32::from_bits(s[8]);
            let max = f32::from_bits(s[9]);
            if !bias.is_finite() || !min.is_finite() || !max.is_finite() {
                return Err(unsupported("non-finite sampler LOD"));
            }
            if !(-16.0..=15.99).contains(&bias) || min > max {
                return Err(invalid("static sampler LOD range"));
            }
            interval(&mut rows, 3, s[11], s[10], 1, s[12])?;
        }
        Ok(())
    }
}
unsafe fn read_array<T: Copy>(pointer: *const T, count: usize) -> Result<Vec<T>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if pointer.is_null() {
        return Err(Error {
            code: E_POINTER,
            message: "null root layout array",
        });
    }
    Ok(unsafe { std::slice::from_raw_parts(pointer, count) }.to_vec())
}
/// Snapshot SDK source-ABI arrays; all non-null pointers must be valid for counts.
pub unsafe fn snapshot(desc: &Description) -> Result<Layout> {
    if desc.count > 64 || desc.sampler_count > 16 {
        return Err(unsupported("root layout count subset"));
    }
    let mut parameters = Vec::new();
    for p in unsafe { read_array(desc.parameters, desc.count as usize) }? {
        let binding = match p.kind {
            0 => {
                let table = unsafe { p.payload.table };
                if table.count > 64 {
                    return Err(unsupported("descriptor range count"));
                }
                Binding::Table(unsafe { read_array(table.ranges, table.count as usize) }?)
            }
            1 => {
                let [register, space, count] = unsafe { p.payload.constants };
                Binding::Constants {
                    register,
                    space,
                    count,
                }
            }
            2..=4 => {
                let [register, space] = unsafe { p.payload.descriptor };
                Binding::Descriptor {
                    kind: p.kind,
                    register,
                    space,
                }
            }
            _ => return Err(invalid("unknown root parameter")),
        };
        parameters.push(Parameter {
            visibility: p.visibility,
            binding,
        });
    }
    let result = Layout {
        parameters,
        samplers: unsafe { read_array(desc.samplers, desc.sampler_count as usize) }?,
        flags: desc.flags,
    };
    result.validate()?;
    Ok(result)
}
fn put(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
fn append(bytes: &mut Vec<u8>, words: &[u32]) -> u32 {
    let start = bytes.len() as u32;
    for &word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    start
}
fn body(layout: &Layout) -> Vec<u8> {
    let mut bytes = Vec::new();
    append(
        &mut bytes,
        &[
            1,
            layout.parameters.len() as u32,
            24,
            layout.samplers.len() as u32,
            0,
            layout.flags,
        ],
    );
    bytes.resize(24 + layout.parameters.len() * 12, 0);
    for (i, p) in layout.parameters.iter().enumerate() {
        let (kind, offset) = match &p.binding {
            Binding::Table(ranges) => {
                let at = bytes.len() as u32;
                append(&mut bytes, &[ranges.len() as u32, at + 8]);
                for range in ranges {
                    append(&mut bytes, range);
                }
                (0, at)
            }
            Binding::Constants {
                register,
                space,
                count,
            } => (1, append(&mut bytes, &[*register, *space, *count])),
            Binding::Descriptor {
                kind,
                register,
                space,
            } => (*kind, append(&mut bytes, &[*register, *space])),
        };
        put(&mut bytes, 24 + i * 12, kind);
        put(&mut bytes, 28 + i * 12, p.visibility);
        put(&mut bytes, 32 + i * 12, offset);
    }
    let offset = bytes.len() as u32;
    put(&mut bytes, 16, offset);
    for sampler in &layout.samplers {
        append(&mut bytes, sampler);
    }
    bytes
}
/// Serialize the finite 1.0 layout into a checksummed standalone RTS0 container.
pub fn serialize(layout: &Layout) -> Result<Vec<u8>> {
    layout.validate()?;
    let data = body(layout);
    let mut bytes = Vec::new();
    append(
        &mut bytes,
        &[
            u32::from_le_bytes(*b"DXBC"),
            0,
            0,
            0,
            0,
            1,
            (44 + data.len()) as u32,
            1,
            36,
            u32::from_le_bytes(*b"RTS0"),
            data.len() as u32,
        ],
    );
    bytes.extend_from_slice(&data);
    let hash = super::root_hash::checksum(&bytes[20..]);
    bytes[4..20].copy_from_slice(&hash);
    Ok(bytes)
}
fn word(bytes: &[u8], offset: usize) -> Result<u32> {
    let end = offset
        .checked_add(4)
        .ok_or(invalid("root signature offset overflow"))?;
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..end)
            .ok_or(invalid("truncated root signature"))?
            .try_into()
            .unwrap(),
    ))
}
fn block(bytes: &[u8], offset: u32, count: usize) -> Result<&[u8]> {
    let start = offset as usize;
    if !start.is_multiple_of(4) {
        return Err(invalid("unaligned root signature offset"));
    }
    bytes
        .get(
            start
                ..start
                    .checked_add(count)
                    .ok_or(invalid("root signature extent overflow"))?,
        )
        .ok_or(invalid("root signature block out of bounds"))
}
/// Parse checked standalone version-1.0 root containers. Full shader containers,
/// version 1.1/1.2, noncanonical encodings and local signatures remain unsupported.
pub fn parse(bytes: &[u8]) -> Result<Layout> {
    if bytes.len() > 1_048_576 {
        return Err(unsupported("root signature byte limit"));
    }
    if bytes.get(..4) != Some(b"DXBC")
        || word(bytes, 20)? != 1
        || word(bytes, 24)? as usize != bytes.len()
    {
        return Err(invalid("root container header"));
    }
    if word(bytes, 28)? != 1 {
        return Err(unsupported("embedded or multipart root signatures"));
    }
    if word(bytes, 32)? != 36 || bytes.get(36..40) != Some(b"RTS0") {
        return Err(invalid("root container part"));
    }
    let payload = block(bytes, 44, word(bytes, 40)? as usize)?;
    if 44 + payload.len() != bytes.len() || bytes[4..20] != super::root_hash::checksum(&bytes[20..])
    {
        return Err(invalid("root container length/checksum"));
    }
    if word(payload, 0)? != 1 {
        return Err(unsupported("root signature version"));
    }
    let count = word(payload, 4)? as usize;
    let sampler_count = word(payload, 12)? as usize;
    if count > 64 || sampler_count > 16 {
        return Err(unsupported("root layout count subset"));
    }
    let params = block(payload, word(payload, 8)?, count * 12)?;
    let mut parameters = Vec::new();
    for i in 0..count {
        let kind = word(params, i * 12)?;
        let visibility = word(params, i * 12 + 4)?;
        let offset = word(params, i * 12 + 8)?;
        let binding = match kind {
            0 => {
                let table = block(payload, offset, 8)?;
                let count = word(table, 0)? as usize;
                if count > 64 {
                    return Err(unsupported("descriptor range count"));
                }
                let data = block(payload, word(table, 4)?, count * 20)?;
                let mut ranges = Vec::new();
                for row in data.as_chunks::<20>().0 {
                    ranges.push(std::array::from_fn(|i| word(row, i * 4).unwrap()));
                }
                Binding::Table(ranges)
            }
            1 => {
                let data = block(payload, offset, 12)?;
                Binding::Constants {
                    register: word(data, 0)?,
                    space: word(data, 4)?,
                    count: word(data, 8)?,
                }
            }
            2..=4 => {
                let data = block(payload, offset, 8)?;
                Binding::Descriptor {
                    kind,
                    register: word(data, 0)?,
                    space: word(data, 4)?,
                }
            }
            _ => return Err(invalid("unknown root parameter")),
        };
        parameters.push(Parameter {
            visibility,
            binding,
        });
    }
    let data = block(payload, word(payload, 16)?, sampler_count * 52)?;
    let samplers = data
        .as_chunks::<52>()
        .0
        .iter()
        .map(|row| std::array::from_fn(|i| word(row, i * 4).unwrap()))
        .collect();
    let layout = Layout {
        parameters,
        samplers,
        flags: word(payload, 20)?,
    };
    layout.validate()?;
    if body(&layout) != payload {
        return Err(unsupported("noncanonical root signature encoding"));
    }
    Ok(layout)
}
unsafe fn blob(bytes: &[u8], out: *mut *mut c_void) -> i32 {
    let mut value: *mut Blob = ptr::null_mut();
    let hr = unsafe { D3DCreateBlob(bytes.len(), &mut value) };
    if hr == S_OK {
        unsafe {
            ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                ((*(*value).vtable).get_buffer_pointer)(value).cast::<u8>(),
                bytes.len(),
            );
            *out = value.cast();
        }
    }
    hr
}
unsafe fn finish(result: Result<Vec<u8>>, out: *mut *mut c_void, error: *mut *mut c_void) -> i32 {
    match result {
        Ok(bytes) => unsafe { blob(&bytes, out) },
        Err(failure) => {
            if !error.is_null() {
                let mut text = failure.message.as_bytes().to_vec();
                text.push(0);
                let hr = unsafe { blob(&text, error) };
                if hr != S_OK {
                    return hr;
                }
            }
            failure.code
        }
    }
}
#[cfg_attr(unix, unsafe(no_mangle))]
pub unsafe extern "C" fn D3D12SerializeRootSignature(
    desc: *const Description,
    version: i32,
    out: *mut *mut c_void,
    error: *mut *mut c_void,
) -> i32 {
    if !error.is_null() {
        unsafe {
            *error = ptr::null_mut();
        }
    }
    if out.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = ptr::null_mut();
    }
    let result = if desc.is_null() {
        Err(Error {
            code: E_POINTER,
            message: "null root signature description",
        })
    } else if version != 1 {
        Err(unsupported("root signature version"))
    } else {
        unsafe { snapshot(&*desc) }.and_then(|layout| serialize(&layout))
    };
    unsafe { finish(result, out, error) }
}
#[cfg_attr(unix, unsafe(no_mangle))]
pub unsafe extern "C" fn D3D12SerializeVersionedRootSignature(
    desc: *const VersionedDescription,
    out: *mut *mut c_void,
    error: *mut *mut c_void,
) -> i32 {
    if desc.is_null() {
        return unsafe { D3D12SerializeRootSignature(ptr::null(), 1, out, error) };
    }
    let version = unsafe { (*desc).version };
    // Do not access any union payload when its version is unavailable.
    if version != 1 {
        if !error.is_null() {
            unsafe {
                *error = ptr::null_mut();
            }
        }
        if out.is_null() {
            return E_POINTER;
        }
        unsafe {
            *out = ptr::null_mut();
            return finish(Err(unsupported("root signature version")), out, error);
        }
    }
    unsafe { D3D12SerializeRootSignature(&(*desc).description, 1, out, error) }
}

#[cfg(test)]
#[path = "root_signature_tests.rs"]
mod tests;
