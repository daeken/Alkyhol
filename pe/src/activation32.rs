//! Checked PE32 activation scopes, references and read-only DLL sections for an
//! explicit exact unsigned assembly inventory. The default empty-only profile
//! remains separate. Unknown metadata must never become a fabricated miss.
use super::{Cpu32, LoadedImage32, LowMapping, PAGE, startup32, unsupported};
use crate::{Result, activation_binding::Template};
use std::{collections::BTreeMap, ptr, rc::Rc};

// The observed one- and three-file section layouts. Larger/unmeasured
// dictionaries stop in inventory validation rather than emit guessed layouts.
fn section(files: &[String], roster: u32) -> (Vec<u8>, Vec<usize>) {
    let count = files.len();
    let mut bytes = vec![0; 44 + count * 24];
    let mut offsets = Vec::new();
    let mut buckets = [Vec::new(), Vec::new(), Vec::new()];
    fn put(bytes: &mut [u8], at: usize, value: u32) {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    for (index, file) in files.iter().enumerate() {
        let string_at = bytes.len();
        let name: Vec<_> = file.encode_utf16().collect();
        for unit in &name {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.extend_from_slice(&[0, 0]);
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        let data_at = bytes.len();
        for word in [20u32, 2, 0, 0, 0] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        let hash = file.bytes().fold(0u32, |h, c| {
            h.wrapping_mul(65599)
                .wrapping_add(u32::from(c.to_ascii_uppercase()))
        });
        let entry = 44 + index * 24;
        for (i, value) in [
            hash,
            string_at as u32,
            (name.len() * 2) as u32,
            data_at as u32,
            20,
            roster,
        ]
        .into_iter()
        .enumerate()
        {
            put(&mut bytes, entry + i * 4, value);
        }
        offsets.push(data_at);
        buckets[(hash % 3) as usize].push(entry as u32);
    }
    let search = if count == 3 {
        let at = bytes.len();
        bytes.resize(at + 32, 0);
        put(&mut bytes, at, 3);
        put(&mut bytes, at + 4, (at + 8) as u32);
        for (i, bucket) in buckets.into_iter().enumerate() {
            let list = if bucket.is_empty() { 0 } else { bytes.len() };
            put(&mut bytes, at + 8 + i * 8, bucket.len() as u32);
            put(&mut bytes, at + 12 + i * 8, list as u32);
            for entry in bucket {
                bytes.extend_from_slice(&entry.to_le_bytes());
            }
        }
        at as u32
    } else {
        0
    };
    for (i, value) in [
        0x64487353,
        44,
        1,
        1,
        if count == 1 { 3 } else { 1 },
        count as u32,
        44,
        1,
        search,
        0,
        0,
    ]
    .into_iter()
    .enumerate()
    {
        put(&mut bytes, i * 4, value);
    }
    (bytes, offsets)
}
struct Context {
    _token: LowMapping,
    data: Option<LowMapping>,
    length: usize,
    offsets: Vec<usize>,
    template: Template,
    references: u32,
}
pub(super) struct State {
    contexts: BTreeMap<u32, Context>,
    templates: BTreeMap<u32, Result<Option<Template>>>,
    scopes: BTreeMap<u32, Result<u32>>,
    current: Result<u32>,
    stack: Vec<(u32, Result<u32>, u32)>,
    next_cookie: u32,
    bounds: Option<super::machine32::Bounds>,
}
impl State {
    pub(super) fn new(
        templates: Vec<(u32, Result<Option<Template>>)>,
        owners: &[usize],
        bounds: Option<super::machine32::Bounds>,
    ) -> Result<Self> {
        if templates.is_empty() || templates.len() != owners.len() {
            return Err("invalid activation scope graph".into());
        }
        let mut out = Self {
            contexts: BTreeMap::new(),
            templates: BTreeMap::new(),
            scopes: BTreeMap::new(),
            current: Ok(0),
            stack: Vec::new(),
            next_cookie: 1,
            bounds,
        };
        let mut tokens = Vec::new();
        for (base, template) in &templates {
            let token = match template {
                Ok(Some(t)) => Ok(out.create(t.clone())?),
                Ok(None) => Ok(0),
                // The opt-in binder validates its complete graph before any
                // initializer. An unresolved assembly may not execute merely
                // because its code happens not to query the activation APIs.
                Err(e) => return Err(e.clone()),
            };
            tokens.push(token);
            out.templates.insert(*base, template.clone());
        }
        for ((base, _), &owner) in templates.iter().zip(owners) {
            out.scopes.insert(
                *base,
                tokens
                    .get(owner)
                    .ok_or("activation scope owner outside graph")?
                    .clone(),
            );
        }
        out.current = tokens[0].clone();
        Ok(out)
    }
    fn create(&mut self, template: Template) -> Result<u32> {
        if !template.active {
            return Ok(0);
        }
        if !template.files.is_empty() && ![1, 3].contains(&template.files.len()) {
            return Err("unmeasured activation section file count".into());
        }
        if self.contexts.len() >= 128 {
            return Err("activation context allocation budget exceeded".into());
        }
        let (data, length, offsets) = if template.files.is_empty() {
            (None, 0, Vec::new())
        } else {
            let (bytes, offsets) = section(&template.files, template.roster);
            let length = bytes.len();
            let rounded = (length + PAGE - 1) & !(PAGE - 1);
            let data = LowMapping::new_in(rounded, PAGE, 0, self.bounds)?;
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), data.base, length);
            }
            data.protect(0, rounded, 1)?;
            (Some(data), length, offsets)
        };
        let token = LowMapping::new_in(PAGE, PAGE, 0, self.bounds)?;
        token.protect(0, PAGE, 0)?;
        let handle = token.base as u32;
        self.contexts.insert(
            handle,
            Context {
                _token: token,
                data,
                length,
                offsets,
                template,
                references: 1,
            },
        );
        Ok(handle)
    }
    fn add_ref(&mut self, handle: u32) -> Result<()> {
        if handle == 0 {
            return Ok(());
        }
        let c = self
            .contexts
            .get_mut(&handle)
            .ok_or("unknown activation context handle")?;
        c.references = c
            .references
            .checked_add(1)
            .ok_or("activation reference count overflow")?;
        Ok(())
    }
    fn release(&mut self, handle: u32) -> Result<()> {
        if handle == 0 {
            return Ok(());
        }
        let c = self
            .contexts
            .get_mut(&handle)
            .ok_or("unknown/released activation context handle")?;
        c.references = c
            .references
            .checked_sub(1)
            .ok_or("activation reference count underflow")?;
        if c.references == 0 {
            self.contexts.remove(&handle);
        }
        Ok(())
    }
    pub(super) fn permits(&self, address: usize, len: usize, write: bool) -> bool {
        !write
            && address.checked_add(len).is_some_and(|end| {
                self.contexts.values().any(|c| {
                    c.data.as_ref().is_some_and(|data| {
                        address >= data.base as usize && end <= data.base as usize + c.length
                    })
                })
            })
    }
}
pub(super) struct Scope {
    cpu: Rc<Cpu32>,
    previous: Result<u32>,
    depth: usize,
}
pub(super) fn enter_scope(cpu: &Rc<Cpu32>, base: u32) -> Option<Scope> {
    let mut optional = cpu.bound_activation.borrow_mut();
    let state = optional.as_mut()?;
    let previous = state.current.clone();
    let depth = state.stack.len();
    state.current = state
        .scopes
        .get(&base)
        .cloned()
        .unwrap_or_else(|| Err("module lacks an activation scope".into()));
    Some(Scope {
        cpu: cpu.clone(),
        previous,
        depth,
    })
}
impl Drop for Scope {
    fn drop(&mut self) {
        let mut state = self.cpu.bound_activation.borrow_mut();
        let state = state.as_mut().expect("owned activation state");
        if state.stack.len() != self.depth {
            unsupported("unbalanced explicit activation across DLL initializer");
        }
        state.current = self.previous.clone();
    }
}
fn checked(image: &LoadedImage32<'_>, address: u32, length: usize, write: bool) {
    if !address.is_multiple_of(4) || !image.permits(address as usize, length, write) {
        unsupported("activation API buffer outside checked guest storage");
    }
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, args: &[u32; 8]) -> u32 {
    // Validate buffers before borrowing state: permits() also inspects its owned
    // read-only section ranges. No RefCell borrow survives a guest transition.
    match id {
        59 | 62 => checked(image, args[if id == 59 { 0 } else { 1 }], 4, true),
        61 => checked(image, args[0], 32, false),
        _ => (),
    }
    let input = if id == 61 {
        Some(unsafe { std::slice::from_raw_parts(args[0] as usize as *const u32, 8) }.to_vec())
    } else {
        None
    };
    let mut state = image.cpu.bound_activation.borrow_mut();
    let state = state.as_mut().unwrap_or_else(|| {
        unsupported("activation stack APIs require the explicit assembly profile")
    });
    let result = (|| -> Result<u32> {
        match id {
            59 => {
                let current = state.current.clone()?;
                state.add_ref(current)?;
                unsafe {
                    (args[0] as usize as *mut u32).write(current);
                }
                Ok(1)
            }
            60 => {
                state.release(args[0])?;
                Ok(0)
            }
            61 => {
                let v = input.as_ref().unwrap();
                if v[..7] != [32, 0x88, 0, 0, 0, 2, 0] {
                    return Err("unsupported CreateActCtxW fields/resource form".into());
                }
                let t = state
                    .templates
                    .get(&v[7])
                    .ok_or("CreateActCtxW module is not registered")?
                    .clone()?
                    .ok_or("module has no embedded activation manifest")?;
                state.create(t)
            }
            62 => {
                if state.stack.len() >= 64 {
                    return Err("activation stack depth budget exceeded".into());
                }
                if args[0] == 0 {
                    return Err("explicit NULL activation is unsupported".into());
                }
                let cookie = state.next_cookie;
                let next = cookie
                    .checked_add(1)
                    .ok_or("activation cookie budget exhausted")?;
                state.add_ref(args[0])?;
                state.stack.push((cookie, state.current.clone(), args[0]));
                state.current = Ok(args[0]);
                state.next_cookie = next;
                unsafe {
                    (args[1] as usize as *mut u32).write(cookie);
                }
                Ok(1)
            }
            63 => {
                if args[0] != 0 {
                    return Err("activation deactivation flags are unsupported".into());
                }
                let (cookie, previous, handle) = state
                    .stack
                    .last()
                    .ok_or("activation cookie stack is empty")?
                    .clone();
                if cookie != args[1] {
                    return Err("activation cookie is stale or not top of stack".into());
                }
                state.release(handle)?;
                state.stack.pop();
                state.current = previous;
                Ok(1)
            }
            _ => unreachable!(),
        }
    })();
    result.unwrap_or_else(|e| unsupported(&e))
}

pub(super) fn find(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    let [flags, guid, section, key, output, ..] = *args;
    if flags > 1 || guid != 0 || section != 2 {
        unsupported("PE32 activation query flags/GUID/section are unsupported");
    }
    if !output.is_multiple_of(4) || !image.permits(output as usize, 4, false) {
        unsupported("PE32 activation keyed-data header outside checked guest storage");
    }
    let size = unsafe { (output as usize as *const u32).read() };
    if ![40, 64].contains(&size) || !image.permits(output as usize, size as usize, true) {
        unsupported("PE32 activation keyed-data size/output is unsupported");
    }
    // A miss still requires a real bounded, terminated guest string. No host
    // dereference of an unchecked pointer, and no lossy UTF-16 conversion.
    let units = startup32::read_units::<u16>(image, key, -1)
        .unwrap_or_else(|| unsupported("PE32 activation key is unsupported"));
    if image.cpu.diagnostic.is_some() {
        eprintln!(
            "PE32 diagnostic activation lookup: cbSize={size}; key_units_len={}; key_units_prefix={:x?}",
            units.len(),
            &units[..units.len().min(128)]
        );
    }
    if let Some(state) = image.cpu.bound_activation.borrow_mut().as_mut() {
        let current = state.current.clone().unwrap_or_else(|e| unsupported(&e));
        if current != 0 {
            let name = String::from_utf16(&units[..units.len() - 1])
                .unwrap_or_else(|_| unsupported("invalid activation key UTF-16"));
            if !name.is_ascii() {
                unsupported("non-ASCII DLL activation keys are unsupported in assembly profile");
            }
            let c = state
                .contexts
                .get(&current)
                .unwrap_or_else(|| unsupported("active context has no owned storage"));
            if let Some(index) = c
                .template
                .files
                .iter()
                .position(|file| file.eq_ignore_ascii_case(&name))
            {
                let data = c.data.as_ref().expect("bound DLL section");
                let values = [
                    1,
                    data.base as u32 + c.offsets[index] as u32,
                    20,
                    0,
                    0,
                    data.base as u32,
                    c.length as u32,
                    if flags == 1 { current } else { 0 },
                    c.template.roster,
                ];
                if flags == 1 {
                    state.add_ref(current).unwrap_or_else(|e| unsupported(&e));
                }
                // cbSize, ulFlags and the extended tail remain untouched, exactly
                // as observed. Only the nine native-filled DWORDs are written.
                unsafe {
                    ptr::copy_nonoverlapping(
                        values.as_ptr(),
                        (output as usize as *mut u32).add(1),
                        values.len(),
                    );
                }
                return 1;
            }
        }
        alkyhol::ffi::SetLastError(14007);
        return 0;
    }
    if let Err(error) = &*image.cpu.activation.borrow() {
        unsupported(error);
    }
    // Native observations: a missing DLL key leaves every output byte untouched
    // for both legacy/current cbSize values and sets ERROR_SXS_KEY_NOT_FOUND.
    alkyhol::ffi::SetLastError(14007);
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn single_dll_section_matches_the_full_native_bytes() {
        let (bytes, offset) = section(&["ctx-provider.dll".into()], 2);
        let expected = "537348642c000000010000000100000003000000010000002c000000010000000000000000000000000000008df2a41a44000000200000006800000014000000020000006300740078002d00700072006f00760069006400650072002e0064006c006c00000000001400000002000000000000000000000000000000";
        assert_eq!(
            bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            expected
        );
        assert_eq!((offset, bytes.len()), (vec![104], 124));
    }
    #[test]
    fn three_file_section_matches_the_complete_native_hash_dictionary() {
        let files = vec![
            "ctx-provider.dll".into(),
            "ctx-secondary.dll".into(),
            "ctx-unused.dll".into(),
        ];
        let (bytes, offsets) = section(&files, 2);
        let expected = "537348642c000000010000000100000001000000030000002c000000010000001801000000000000000000008df2a41a74000000200000009800000014000000020000004427d0bfac00000022000000d000000014000000020000007230bce9e40000001c0000000401000014000000020000006300740078002d00700072006f00760069006400650072002e0064006c006c000000000014000000020000000000000000000000000000006300740078002d007300650063006f006e0064006100720079002e0064006c006c00000014000000020000000000000000000000000000006300740078002d0075006e0075007300650064002e0064006c006c0000000000140000000200000000000000000000000000000003000000200100000100000038010000010000003c01000001000000400100002c0000005c00000044000000";
        assert_eq!(
            bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            expected
        );
        assert_eq!(offsets, vec![152, 208, 260]);
    }
}
