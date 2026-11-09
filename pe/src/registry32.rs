//! Explicit read-only 32-bit registry scopes. Unknown state is not a missing key.
use super::{LoadedImage32, LowMapping, startup32, unsupported};
use crate::Result;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
};
const HKCU: u32 = 0x80000001;
const HKLM: u32 = 0x80000002;
const READ: u32 = 0x20019;
#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Text(String),
    Dword(u32),
    Binary(Vec<u8>),
}
impl Value {
    fn bytes(&self, ansi: bool) -> Result<(u32, Vec<u8>)> {
        Ok(match self {
            Self::Text(text) if ansi => {
                if !text.is_ascii() {
                    return Err("ANSI registry text requires ASCII data".into());
                }
                (1, text.bytes().chain([0]).collect())
            }
            Self::Text(text) => (
                1,
                text.encode_utf16()
                    .chain([0])
                    .flat_map(u16::to_le_bytes)
                    .collect(),
            ),
            Self::Dword(value) => (4, value.to_le_bytes().to_vec()),
            Self::Binary(bytes) => (3, bytes.clone()),
        })
    }
}
struct Scope {
    hive: u32,
    path: String,
}
struct OpenKey {
    _token: LowMapping,
    hive: u32,
    path: String,
    _access: u32,
}
pub(super) struct State {
    scopes: Vec<Scope>,
    keys: BTreeSet<(u32, String)>,
    handles: BTreeMap<u32, OpenKey>,
    values: BTreeMap<(u32, String, String), Value>,
}
fn path(value: &str) -> Result<String> {
    if value.is_empty()
        || value.len() > 4096
        || value.bytes().any(|b| !(32..127).contains(&b))
        || value.split('\\').any(|p| p.is_empty() || p.len() > 255)
    {
        return Err("registry key outside the admitted nonempty ASCII path profile".into());
    }
    Ok(value.to_ascii_lowercase())
}
fn contains(parent: &str, child: &str) -> bool {
    child == parent
        || child
            .strip_prefix(parent)
            .is_some_and(|s| s.starts_with('\\'))
}
fn attributes(n: roxmltree::Node<'_, '_>, names: &[&str]) -> Result<()> {
    if n.tag_name().namespace().is_some()
        || n.attributes().len() != names.len()
        || n.attributes()
            .any(|a| a.namespace().is_some() || !names.contains(&a.name()))
        || names.iter().any(|name| n.attribute(*name).is_none())
        || n.children().any(|c| {
            !c.is_element() && !c.is_comment() && !c.text().is_some_and(|s| s.trim().is_empty())
        })
    {
        return Err("unknown registry catalog attributes, namespace or content".into());
    }
    Ok(())
}
impl State {
    pub(super) fn read(file: &Path) -> Result<Self> {
        let f = std::fs::File::open(file).map_err(|e| format!("registry catalog open: {e}"))?;
        if !f
            .metadata()
            .map_err(|e| format!("registry catalog metadata: {e}"))?
            .is_file()
        {
            return Err("registry catalog must be a regular file".into());
        }
        let mut bytes = Vec::new();
        f.take(256 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("registry catalog read: {e}"))?;
        let text = std::str::from_utf8(&bytes).map_err(|_| "registry catalog must be UTF-8")?;
        Self::parse(text)
    }
    fn parse(text: &str) -> Result<Self> {
        if text.len() > 256 * 1024 {
            return Err("registry catalog exceeds its size budget".into());
        }
        let doc =
            roxmltree::Document::parse(text).map_err(|e| format!("registry catalog XML: {e}"))?;
        let root = doc.root_element();
        attributes(root, &["view"])?;
        if root.tag_name().name() != "registry" || root.attribute("view") != Some("32") {
            return Err("registry catalog requires the explicit 32-bit view".into());
        }
        let mut out = Self {
            scopes: Vec::new(),
            keys: BTreeSet::new(),
            handles: BTreeMap::new(),
            values: BTreeMap::new(),
        };
        for scope in root.children().filter(|n| n.is_element()) {
            attributes(scope, &["hive", "path", "exists"])?;
            if scope.tag_name().name() != "scope" || out.scopes.len() >= 8 {
                return Err("unsupported registry scope or scope budget".into());
            }
            let hive = match scope.attribute("hive") {
                Some("HKCU") => HKCU,
                Some("HKLM") => HKLM,
                _ => return Err("unsupported registry catalog hive".into()),
            };
            let prefix = path(scope.attribute("path").expect("checked attribute"))?;
            if out.scopes.iter().any(|s| {
                s.hive == hive && (contains(&s.path, &prefix) || contains(&prefix, &s.path))
            }) {
                return Err("overlapping authoritative registry scopes".into());
            }
            let exists = match scope.attribute("exists") {
                Some("true") => true,
                Some("false") => false,
                _ => return Err("registry scope requires a declared existence state".into()),
            };
            if exists {
                out.insert(hive, &prefix)?;
                out.children(hive, &prefix, scope, 0)?;
            } else if scope.children().any(|n| n.is_element()) {
                return Err("absent registry scope cannot contain keys".into());
            }
            out.scopes.push(Scope { hive, path: prefix });
        }
        Ok(out)
    }
    fn insert(&mut self, hive: u32, key: &str) -> Result<()> {
        if !self.keys.insert((hive, key.into())) {
            return Err("duplicate registry catalog key".into());
        }
        if self.keys.len() > 128 {
            return Err("registry key budget exceeded".into());
        }
        Ok(())
    }
    fn children(
        &mut self,
        hive: u32,
        parent: &str,
        node: roxmltree::Node<'_, '_>,
        depth: usize,
    ) -> Result<()> {
        if depth > 16 {
            return Err("registry key depth budget exceeded".into());
        }
        for child in node.children().filter(|n| n.is_element()) {
            if child.tag_name().name() == "value" {
                attributes(child, &["name", "type", "data"])?;
                if child.children().any(|n| n.is_element()) {
                    return Err("registry values cannot have children".into());
                }
                let name = child.attribute("name").unwrap();
                if name.len() > 16383 || !name.is_ascii() || name.contains('\0') {
                    return Err("registry value name outside ASCII bounds".into());
                }
                let data = child.attribute("data").unwrap();
                let value = match child.attribute("type").unwrap() {
                    "sz" if !data.contains('\0') => Value::Text(data.into()),
                    "dword" => Value::Dword(data.parse().map_err(|_| "invalid registry DWORD")?),
                    "binary" if data.len().is_multiple_of(2) && data.is_ascii() => Value::Binary(
                        (0..data.len())
                            .step_by(2)
                            .map(|n| {
                                u8::from_str_radix(&data[n..n + 2], 16)
                                    .map_err(|_| "invalid registry binary byte")
                            })
                            .collect::<std::result::Result<_, _>>()?,
                    ),
                    _ => return Err("unsupported registry value type/data".into()),
                };
                if value.bytes(false)?.1.len() > 65536
                    || self.values.len() >= 512
                    || self
                        .values
                        .insert((hive, parent.into(), name.to_ascii_lowercase()), value)
                        .is_some()
                {
                    return Err("registry value size/count/duplicate bound".into());
                }
                continue;
            }
            attributes(child, &["name"])?;
            if child.tag_name().name() != "key" {
                return Err("unsupported registry catalog element".into());
            }
            let name = path(child.attribute("name").expect("checked attribute"))?;
            if name.contains('\\') {
                return Err("registry child name must be a single component".into());
            }
            let key = path(&format!("{parent}\\{name}"))?;
            self.insert(hive, &key)?;
            self.children(hive, &key, child, depth + 1)?;
        }
        Ok(())
    }
    fn known(&self, hive: u32, key: &str) -> Result<bool> {
        if !self
            .scopes
            .iter()
            .any(|s| s.hive == hive && contains(&s.path, key))
        {
            return Err("registry lookup outside authoritative configured scopes".into());
        }
        Ok(self.keys.contains(&(hive, key.into())))
    }
    pub(super) fn class_registered(&self, guid: &str) -> Result<bool> {
        let key = path(&format!("Software\\Classes\\CLSID\\{guid}"))?;
        // Both hives must be authoritative, even if one side has no entry.
        let user = self.known(HKCU, &key)?;
        let machine = self.known(HKLM, &key)?;
        Ok(user || machine)
    }
    fn target(&self, handle: u32, child: &str) -> Result<(u32, String)> {
        let (hive, full) = if [HKCU, HKLM].contains(&handle) {
            (handle, path(child)?)
        } else {
            let parent = self
                .handles
                .get(&handle)
                .ok_or("registry open requires a predefined hive or owned live key")?;
            (
                parent.hive,
                path(&format!("{}\\{}", parent.path, path(child)?))?,
            )
        };
        Ok((hive, full))
    }
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, args: &[u32; 8]) -> u32 {
    let catalog =
        image.cpu.registry.as_ref().unwrap_or_else(|| {
            unsupported("registry API requires an explicit scoped backing catalog")
        });
    if id == 108 {
        if catalog.borrow_mut().handles.remove(&args[0]).is_none() {
            unsupported("RegCloseKey requires an owned live key handle");
        }
        return 0; // The owned key/token was actually released.
    }
    if args[2] != 0 {
        unsupported("registry open options outside the admitted ordinary-key profile");
    }
    if args[3] & 0x100 != 0 || args[3] & !(READ | 0x200) != 0 {
        unsupported("registry open requires admitted read access in the configured 32-bit view");
    }
    if !args[4].is_multiple_of(4) || !image.permits(args[4] as usize, 4, true) {
        unsupported("registry result handle outside aligned writable guest storage");
    }
    let text = if id == 216 {
        super::bulk32::ascii(image, args[1])
    } else {
        let key = startup32::read_units::<u16>(image, args[1], -1).unwrap_or_else(|| {
            unsupported("registry subkey outside readable guest string storage")
        });
        String::from_utf16(&key[..key.len() - 1])
            .unwrap_or_else(|_| unsupported("invalid registry subkey UTF-16"))
    };
    let text = text.trim_end_matches('\\');
    if image.cpu.diagnostic.is_some() {
        eprintln!(
            "PE32 registry open: hive={:x}; path={text:?}; access={:x}",
            args[0], args[3]
        );
    }
    let mut state = catalog.borrow_mut();
    let (hive, key) = state
        .target(args[0], text)
        .unwrap_or_else(|e| unsupported(&e));
    let exists = state.known(hive, &key).unwrap_or_else(|e| unsupported(&e));
    let handle = if exists {
        if state.handles.len() >= 128 {
            unsupported("registry open handle budget exceeded");
        }
        let token = image
            .cpu
            .mapping(super::PAGE, super::PAGE, 0)
            .unwrap_or_else(|e| unsupported(&e));
        token
            .protect(0, super::PAGE, 0)
            .unwrap_or_else(|e| unsupported(&e));
        let handle = token.base as u32;
        state.handles.insert(
            handle,
            OpenKey {
                _token: token,
                hive,
                path: key,
                _access: args[3] & !0x200,
            },
        );
        handle
    } else {
        0
    };
    unsafe {
        (args[4] as *mut u32).write(handle);
    }
    if exists { 0 } else { 2 }
}
/// Legacy ANSI open: the actually reached absent-key and same-hive profiles.
/// Existing subkeys require default security-mask semantics, not fabricated access.
pub(super) fn legacy_open(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if a[2] == 0 || !a[2].is_multiple_of(4) || !image.permits(a[2] as usize, 4, true) {
        unsupported("RegOpenKeyA output outside aligned writable guest storage");
    }
    let catalog = image
        .cpu
        .registry
        .as_ref()
        .unwrap_or_else(|| unsupported("RegOpenKeyA requires an explicit registry catalog"));
    let state = catalog.borrow();
    let text = if a[1] == 0 {
        String::new()
    } else {
        super::bulk32::ascii(image, a[1])
    };
    if text.is_empty() && [HKCU, HKLM].contains(&a[0]) {
        unsafe {
            (a[2] as *mut u32).write(a[0]);
        }
        return 0; // The same predefined hive is returned; no key or permission is invented.
    }
    let (hive, key) = state
        .target(a[0], text.trim_end_matches('\\'))
        .unwrap_or_else(|e| unsupported(&e));
    let exists = state.known(hive, &key).unwrap_or_else(|e| unsupported(&e));
    if image.cpu.diagnostic.is_some() {
        eprintln!("PE32 RegOpenKeyA: hive={hive:x}; path={key:?}; configured_exists={exists}");
    }
    if exists {
        unsupported("RegOpenKeyA existing-key default security access is not implemented");
    }
    unsafe {
        (a[2] as *mut u32).write(0);
    }
    2 // ERROR_FILE_NOT_FOUND from the authoritative scoped catalog; LastError untouched.
}
pub(super) fn query(image: &LoadedImage32<'_>, a: &[u32; 8], ansi: bool) -> u32 {
    if a[2] != 0 || (a[4] != 0 && a[5] == 0) {
        return 87;
    }
    let name = if a[1] == 0 {
        String::new()
    } else if ansi {
        super::bulk32::ascii(image, a[1])
    } else {
        let units = startup32::read_units::<u16>(image, a[1], -1)
            .unwrap_or_else(|| unsupported("registry value name outside checked storage"));
        String::from_utf16(&units[..units.len() - 1])
            .unwrap_or_else(|_| unsupported("registry value name is not UTF-16"))
    };
    if !name.is_ascii() || name.len() > 16383 {
        unsupported("registry value name outside ASCII bounds");
    }
    let catalog =
        image.cpu.registry.as_ref().unwrap_or_else(|| {
            unsupported("registry API requires an explicit scoped backing catalog")
        });
    let state = catalog.borrow();
    let Some(key) = state.handles.get(&a[0]) else {
        return 6;
    };
    if key._access & 1 == 0 {
        return 5;
    }
    if image.cpu.diagnostic.is_some() {
        eprintln!(
            "PE32 registry value: key={:?}; name={name:?}; ansi={ansi}",
            key.path
        );
    }
    let Some(value) = state
        .values
        .get(&(key.hive, key.path.clone(), name.to_ascii_lowercase()))
    else {
        return 2;
    };
    let (kind, bytes) = value.bytes(ansi).unwrap_or_else(|e| unsupported(&e));
    for &pointer in &[a[3], a[5]] {
        if pointer != 0 && (!pointer.is_multiple_of(4) || !image.permits(pointer as usize, 4, true))
        {
            unsupported("registry type/size output outside checked storage");
        }
    }
    let capacity = if a[5] == 0 {
        0
    } else {
        unsafe { (a[5] as *const u32).read() }
    };
    if a[4] != 0 && !image.permits(a[4] as usize, capacity as usize, true) {
        unsupported("registry data buffer outside checked writable storage");
    }
    unsafe {
        if a[3] != 0 {
            (a[3] as *mut u32).write(kind);
        }
        if a[5] != 0 {
            (a[5] as *mut u32).write(bytes.len() as u32);
        }
        if a[4] != 0 {
            if (capacity as usize) < bytes.len() {
                return 234;
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), a[4] as *mut u8, bytes.len());
        }
    }
    0
}
#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &str = include_str!("../fixtures/pe32-startup/registry-open.xml");
    #[test]
    fn typed_values_keep_unicode_width_byte_counts_and_little_endian_data() {
        let xml = r#"<registry view="32"><scope hive="HKCU" path="Software\OwnedValues" exists="true"><value name="Path" type="sz" data="Z:\actual"/><value name="Number" type="dword" data="4294967295"/><value name="Raw" type="binary" data="00ff23"/></scope></registry>"#;
        let state = State::parse(xml).unwrap();
        let get = |name: &str| {
            state
                .values
                .get(&(HKCU, "software\\ownedvalues".into(), name.into()))
                .unwrap()
        };
        assert_eq!(
            get("path").bytes(true).unwrap(),
            (1, b"Z:\\actual\0".to_vec())
        );
        assert_eq!(get("path").bytes(false).unwrap().1.len(), 20);
        assert_eq!(get("number").bytes(false).unwrap(), (4, vec![255; 4]));
        assert_eq!(get("raw").bytes(false).unwrap(), (3, vec![0, 255, 35]));
        assert!(Value::Text("wideλ".into()).bytes(true).is_err());
        for bad in [
            xml.replace("4294967295", "4294967296"),
            xml.replace("00ff23", "00xz23"),
            xml.replace("00ff23", "00f"),
            xml.replace("type=\"sz\"", "type=\"unknown\""),
        ] {
            assert!(State::parse(&bad).is_err());
        }
    }
    #[test]
    fn concrete_keys_and_known_absence_are_separate_from_unknown_scopes() {
        let state = State::parse(SOURCE).unwrap();
        let base = "software\\alkyholowneddaostartup\\{e97b4171-7a2b-4a82-a867-9af56bff70f4}";
        assert!(state.known(HKCU, base).unwrap());
        assert!(state.known(HKCU, &format!("{base}\\present")).unwrap());
        assert!(!state.known(HKCU, &format!("{base}\\absent")).unwrap());
        assert!(state.known(HKLM, base).is_err());
        assert!(state.known(HKCU, &format!("{base}suffix")).is_err());
        assert!(state.known(HKCU, "software").is_err());
        let absent=State::parse(r#"<registry view="32"><scope hive="HKLM" path="Software\OwnedApp" exists="false"/></registry>"#).unwrap();
        assert!(!absent.known(HKLM, "software\\ownedapp").unwrap());
        assert!(!absent.known(HKLM, "software\\ownedapp\\child").unwrap());
        assert!(absent.known(HKLM, "software\\otherapp").is_err());
    }
    #[test]
    fn ambiguous_unknown_or_over_budget_catalogs_are_rejected() {
        for text in [
            SOURCE.replace("view=\"32\"", "view=\"64\""),
            SOURCE.replace(
                "<key name=\"Present\"/>",
                "<key name=\"Present\"/><key name=\"pRESENT\"/>",
            ),
            SOURCE.replace("exists=\"true\"", "exists=\"false\""),
            SOURCE.replace("<key name=\"Present\"/>", "<value name=\"Present\"/>"),
            SOURCE.replace("hive=\"HKCU\"", "hive=\"UNKNOWN\""),
            SOURCE.replace(
                "<registry view=\"32\">",
                "<registry view=\"32\" unknown=\"1\">",
            ),
            SOURCE.replace("name=\"Present\"", "name=\"A\\B\""),
            SOURCE.repeat(2),
        ] {
            assert!(State::parse(&text).is_err());
        }
        assert!(State::parse(&" ".repeat(256 * 1024 + 1)).is_err());
        let nested = format!(
            "<registry view=\"32\"><scope hive=\"HKCU\" path=\"Software\\Owned\" exists=\"true\">{}{}</scope></registry>",
            "<key name=\"Child\">".repeat(18),
            "</key>".repeat(18)
        );
        assert!(State::parse(&nested).is_err());
        let overlap = r#"<registry view="32"><scope hive="HKCU" path="Software\Owned" exists="false"/><scope hive="HKCU" path="software\OWNED\Child" exists="true"/></registry>"#;
        assert!(State::parse(overlap).is_err());
        let scopes = (0..9)
            .map(|i| format!(r#"<scope hive="HKCU" path="Software\Owned{i}" exists="false"/>"#))
            .collect::<String>();
        assert!(State::parse(&format!(r#"<registry view="32">{scopes}</registry>"#)).is_err());
        let keys = (0..128)
            .map(|i| format!(r#"<key name="Child{i}"/>"#))
            .collect::<String>();
        assert!(State::parse(&format!(r#"<registry view="32"><scope hive="HKCU" path="Software\Owned" exists="true">{keys}</scope></registry>"#)).is_err());
    }
    #[test]
    fn class_lookup_requires_both_hives_and_never_turns_unknown_into_absence() {
        let machine = r#"<scope hive="HKLM" path="Software\Classes\CLSID" exists="true"/>"#;
        let user = r#"<scope hive="HKCU" path="Software\Classes\CLSID" exists="true"/>"#;
        let only_machine =
            State::parse(&format!(r#"<registry view="32">{machine}</registry>"#)).unwrap();
        assert!(only_machine.class_registered("{owned-class}").is_err());
        let empty = State::parse(&format!(
            r#"<registry view="32">{machine}{user}</registry>"#
        ))
        .unwrap();
        assert!(!empty.class_registered("{owned-class}").unwrap());
        for (hive, other) in [("HKLM", user), ("HKCU", machine)] {
            let xml = format!(
                r#"<registry view="32">{other}<scope hive="{hive}" path="Software\Classes\CLSID" exists="true"><key name="{{owned-class}}"/></scope></registry>"#
            );
            let registered = State::parse(&xml).unwrap();
            assert!(registered.class_registered("{Owned-Class}").unwrap());
            assert!(!registered.class_registered("{different-class}").unwrap());
        }
    }
    #[test]
    fn ascii_case_paths_and_scope_boundaries_are_checked() {
        assert_eq!(path("Software\\MiXeD").unwrap(), "software\\mixed");
        for p in ["", "\\leading", "trailing\\", "a\\\\b", "a\0b", "wideλ"] {
            assert!(path(p).is_err());
        }
        assert!(contains("software\\owned", "software\\owned\\child"));
        assert!(!contains("software\\owned", "software\\owned-other"));
    }
}
