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
    let key = startup32::read_units::<u16>(image, args[1], -1)
        .unwrap_or_else(|| unsupported("registry subkey outside readable guest string storage"));
    let text = String::from_utf16(&key[..key.len() - 1])
        .unwrap_or_else(|_| unsupported("invalid registry subkey UTF-16"));
    let mut state = catalog.borrow_mut();
    let (hive, key) = state
        .target(args[0], &text)
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
#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &str = include_str!("../fixtures/pe32-startup/registry-open.xml");
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
    fn ascii_case_paths_and_scope_boundaries_are_checked() {
        assert_eq!(path("Software\\MiXeD").unwrap(), "software\\mixed");
        for p in ["", "\\leading", "trailing\\", "a\\\\b", "a\0b", "wideλ"] {
            assert!(path(p).is_err());
        }
        assert!(contains("software\\owned", "software\\owned\\child"));
        assert!(!contains("software\\owned", "software\\owned-other"));
    }
}
