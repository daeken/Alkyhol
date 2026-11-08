//! Checked PE32 activation scopes, references and read-only DLL sections for an
//! explicit exact unsigned assembly inventory. The default empty-only profile
//! remains separate. Unknown metadata must never become a fabricated miss.
use super::{Cpu32, LoadedImage32, LowMapping, PAGE, startup32, unsupported};
use crate::{Result, activation_binding::Template};
use std::{collections::BTreeMap, ptr, rc::Rc};

// A single-file DLL section. Offsets/header/hash are checked against the complete
// native section, not just a successful Boolean result. Larger schemas stop.
fn section(file: &str, roster: u32) -> (Vec<u8>, usize) {
    let name: Vec<_> = file.encode_utf16().collect();
    let offset = (68 + (name.len() + 1) * 2 + 3) & !3;
    let mut bytes = vec![0; offset + 20];
    let hash = file.bytes().fold(0u32, |h, c| {
        h.wrapping_mul(65599)
            .wrapping_add(u32::from(c.to_ascii_uppercase()))
    });
    let header = [
        0x64487353,
        44,
        1,
        1,
        3,
        1,
        44,
        1,
        0,
        0,
        0,
        hash,
        68,
        (name.len() * 2) as u32,
        offset as u32,
        20,
        roster,
    ];
    for (i, value) in header.into_iter().enumerate() {
        bytes[i * 4..i * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    for (i, value) in name.into_iter().enumerate() {
        bytes[68 + i * 2..70 + i * 2].copy_from_slice(&value.to_le_bytes());
    }
    for (i, value) in [20u32, 2, 0, 0, 0].into_iter().enumerate() {
        bytes[offset + i * 4..offset + i * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    (bytes, offset)
}
struct Context {
    _token: LowMapping,
    data: LowMapping,
    length: usize,
    offset: usize,
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
}
impl State {
    pub(super) fn new(
        templates: Vec<(u32, Result<Option<Template>>)>,
        owners: &[usize],
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
        let Some(file) = &template.file else {
            return Ok(0);
        };
        if self.contexts.len() >= 128 {
            return Err("activation context allocation budget exceeded".into());
        }
        let (bytes, offset) = section(file, template.roster);
        let length = bytes.len();
        let rounded = (length + PAGE - 1) & !(PAGE - 1);
        let data = LowMapping::new(rounded, PAGE, 0)?;
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), data.base, length);
        }
        data.protect(0, rounded, 1)?;
        let token = LowMapping::new(PAGE, PAGE, 0)?;
        token.protect(0, PAGE, 0)?;
        let handle = token.base as u32;
        self.contexts.insert(
            handle,
            Context {
                _token: token,
                data,
                length,
                offset,
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
                    address >= c.data.base as usize && end <= c.data.base as usize + c.length
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
                if t.file.is_none() {
                    return Err("explicit empty-context handles are unsupported".into());
                }
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
            if c.template
                .file
                .as_ref()
                .is_some_and(|file| file.eq_ignore_ascii_case(&name))
            {
                let values = [
                    1,
                    c.data.base as u32 + c.offset as u32,
                    20,
                    0,
                    0,
                    c.data.base as u32,
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
        let (bytes, offset) = section("ctx-provider.dll", 2);
        let expected = "537348642c000000010000000100000003000000010000002c000000010000000000000000000000000000008df2a41a44000000200000006800000014000000020000006300740078002d00700072006f00760069006400650072002e0064006c006c00000000001400000002000000000000000000000000000000";
        assert_eq!(
            bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            expected
        );
        assert_eq!((offset, bytes.len()), (104, 124));
    }
}
