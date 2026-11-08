//! Explicit, bounded assembly metadata for the PE32 activation path. This is
//! not a Windows installation/search policy. Unknown XML/bindings stay errors.
use crate::{Image, ResourceName, Result};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
};

const NS: &str = "urn:schemas-microsoft-com:asm.v1";
#[derive(Clone, Debug, PartialEq, Eq)]
struct Identity {
    name: String,
    version: [u16; 4],
}
#[derive(Clone, Debug)]
struct Manifest {
    identity: Option<Identity>,
    dependency: Option<Identity>,
    file: Option<String>,
}
#[derive(Clone, Debug)]
pub(crate) struct Template {
    pub file: Option<String>,
    pub roster: u32,
}

fn attrs(node: roxmltree::Node<'_, '_>, allowed: &[&str]) -> Result<()> {
    if node
        .attributes()
        .any(|a| a.namespace().is_some() || !allowed.contains(&a.name()))
    {
        return Err("unsupported activation manifest attributes".into());
    }
    Ok(())
}
fn children<'a, 'i>(node: roxmltree::Node<'a, 'i>) -> Result<Vec<roxmltree::Node<'a, 'i>>> {
    let mut out = Vec::new();
    for child in node.children() {
        if child.is_comment()
            || child.is_text()
                && child
                    .text()
                    .is_some_and(|s| s.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')))
        {
            continue;
        }
        if !child.is_element() || child.tag_name().namespace() != Some(NS) {
            return Err("unsupported activation manifest content/namespace".into());
        }
        out.push(child);
    }
    Ok(out)
}
fn identity(node: roxmltree::Node<'_, '_>) -> Result<Identity> {
    attrs(node, &["type", "name", "version", "processorArchitecture"])?;
    if node.attribute("type") != Some("win32")
        || node.attribute("processorArchitecture") != Some("x86")
        || !children(node)?.is_empty()
    {
        return Err("unsupported activation assembly identity/platform".into());
    }
    let name = node
        .attribute("name")
        .ok_or("missing assembly identity name")?;
    if name.is_empty()
        || name.len() > 255
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err("unsupported assembly identity name".into());
    }
    let parts: Vec<_> = node
        .attribute("version")
        .ok_or("missing assembly version")?
        .split('.')
        .collect();
    if parts.len() != 4 {
        return Err("unsupported assembly version".into());
    }
    let mut version = [0; 4];
    for (i, s) in parts.iter().enumerate() {
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err("invalid assembly version".into());
        }
        version[i] = s
            .parse()
            .map_err(|_| "assembly version component exceeds WORD")?;
    }
    Ok(Identity {
        name: name.to_ascii_lowercase(),
        version,
    })
}
fn parse(bytes: &[u8]) -> Result<Manifest> {
    if bytes.len() > 1024 * 1024 {
        return Err("activation manifest size budget exceeded".into());
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "non-UTF-8 activation manifest")?;
    let doc = roxmltree::Document::parse_with_options(
        text.strip_prefix('\u{feff}').unwrap_or(text),
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: 1024,
        },
    )
    .map_err(|_| "invalid or unsupported activation manifest XML")?;
    let root = doc.root_element();
    attrs(root, &["manifestVersion"])?;
    if !root.has_tag_name((NS, "assembly")) || root.attribute("manifestVersion") != Some("1.0") {
        return Err("unsupported activation manifest root".into());
    }
    // Processing instructions outside the assembly must not be silently lost.
    if doc.root().children().any(|n| n.is_pi()) {
        return Err("unsupported manifest processing instruction".into());
    }
    let mut out = Manifest {
        identity: None,
        dependency: None,
        file: None,
    };
    for node in children(root)? {
        match node.tag_name().name() {
            "assemblyIdentity" if out.identity.is_none() => out.identity = Some(identity(node)?),
            "dependency" if out.dependency.is_none() => {
                attrs(node, &[])?;
                let child = children(node)?;
                if child.len() != 1 || !child[0].has_tag_name((NS, "dependentAssembly")) {
                    return Err("unsupported assembly dependency".into());
                }
                attrs(child[0], &[])?;
                let ids = children(child[0])?;
                if ids.len() != 1 || !ids[0].has_tag_name((NS, "assemblyIdentity")) {
                    return Err("unsupported dependent assembly".into());
                }
                out.dependency = Some(identity(ids[0])?);
            }
            "file" if out.file.is_none() => {
                attrs(node, &["name"])?;
                if !children(node)?.is_empty() {
                    return Err("assembly file metadata is unsupported".into());
                }
                let name = node.attribute("name").ok_or("missing assembly file name")?;
                if name.len() > 255
                    || !name.ends_with(".dll")
                    || name.starts_with('.')
                    || !name.bytes().all(|b| {
                        b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b)
                    })
                {
                    return Err("unsupported assembly DLL basename".into());
                }
                out.file = Some(name.to_owned());
            }
            _ => return Err("assembly manifest element/multiplicity is unsupported".into()),
        }
    }
    Ok(out)
}
fn read(path: &Path) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path).map_err(|e| format!("assembly manifest open: {e}"))?;
    let meta = file.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > 1024 * 1024 {
        return Err("assembly manifest must be a bounded regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 1024 * 1024 {
        return Err("assembly manifest read budget exceeded".into());
    }
    Ok(bytes)
}
pub(crate) struct Inventory {
    assemblies: Vec<(Identity, String)>,
}
impl Inventory {
    pub(crate) fn new(paths: &[PathBuf], origins: &BTreeMap<String, PathBuf>) -> Result<Self> {
        if paths.len() > 64 {
            return Err("assembly inventory exceeds 64 manifests".into());
        }
        let mut assemblies = Vec::new();
        for path in paths {
            let path = std::fs::canonicalize(path)
                .map_err(|e| format!("assembly manifest source: {e}"))?;
            let manifest = parse(&read(&path)?)?;
            let id = manifest
                .identity
                .ok_or("assembly manifest lacks identity")?;
            let file = manifest.file.ok_or("assembly manifest lacks file")?;
            if manifest.dependency.is_some() {
                return Err("nested assembly dependencies are unsupported".into());
            }
            if assemblies.iter().any(|(old, _)| old == &id) {
                return Err("duplicate assembly identity".into());
            }
            let source =
                std::fs::canonicalize(path.parent().ok_or("assembly manifest parent")?.join(&file))
                    .map_err(|e| format!("assembly DLL source: {e}"))?;
            if origins.get(&file) != Some(&source) {
                return Err("assembly DLL is not the provided file-backed module".into());
            }
            assemblies.push((id, file));
        }
        Ok(Self { assemblies })
    }
    pub(crate) fn template(&self, image: &Image<'_>, root: bool) -> Result<Option<Template>> {
        let resources: Vec<_> = image
            .resources()?
            .into_iter()
            .filter(|r| r.kind == ResourceName::Id(24))
            .collect();
        if resources.is_empty() && !root {
            return Ok(None);
        }
        if resources.len() != 1 || resources[0].name != ResourceName::Id(if root { 1 } else { 2 }) {
            return Err(
                "activation requires one explicit manifest resource with unambiguous language/role"
                    .into(),
            );
        }
        self.bind(resources[0].bytes, root).map(Some)
    }
    fn bind(&self, bytes: &[u8], root: bool) -> Result<Template> {
        let m = parse(bytes)?;
        if m.identity.is_some() || m.file.is_some() {
            return Err("root/module assembly identity/files are unsupported".into());
        }
        if root && m.dependency.is_some() {
            return Err("nonempty executable default activation scopes are unsupported".into());
        }
        if !root && m.dependency.is_none() {
            return Err("empty resource-2 DLL activation scopes are not yet measured".into());
        }
        let file=match m.dependency {
            None=>None,
            Some(id)=>Some(self.assemblies.iter().find(|(candidate,_)|candidate==&id).map(|(_,file)|file.clone()).ok_or("unresolved exact assembly dependency; policy/version fallback is unsupported")?),
        };
        Ok(Template { file, roster: 2 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wrap(body: &str) -> String {
        format!("<assembly xmlns='{NS}' manifestVersion='1.0'>{body}</assembly>")
    }
    #[test]
    fn owned_identity_and_dependency_are_exact_and_unsigned() {
        let m = parse(include_bytes!(
            "../fixtures/pe32-startup/ctx-assembly.manifest"
        ))
        .unwrap();
        assert_eq!(
            m.identity.unwrap(),
            Identity {
                name: "alkyhol.ownedcrt".into(),
                version: [1, 0, 0, 0]
            }
        );
        assert_eq!(m.file.as_deref(), Some("ctx-provider.dll"));
        assert_eq!(
            parse(include_bytes!(
                "../fixtures/pe32-startup/ctx-importer.manifest"
            ))
            .unwrap()
            .dependency
            .unwrap()
            .version,
            [1, 0, 0, 0]
        );
        assert!(
            parse(include_bytes!(
                "../fixtures/pe32-startup/activation-empty.manifest"
            ))
            .unwrap()
            .dependency
            .is_none()
        );
    }
    #[test]
    fn only_measured_default_and_dll_roles_bind_without_version_fallback() {
        let empty = include_bytes!("../fixtures/pe32-startup/activation-empty.manifest");
        let importer = include_bytes!("../fixtures/pe32-startup/ctx-importer.manifest");
        let mut inventory = Inventory {
            assemblies: vec![(
                Identity {
                    name: "alkyhol.ownedcrt".into(),
                    version: [1, 0, 0, 0],
                },
                "ctx-provider.dll".into(),
            )],
        };
        assert!(inventory.bind(empty, true).unwrap().file.is_none());
        assert_eq!(
            inventory.bind(importer, false).unwrap().file.as_deref(),
            Some("ctx-provider.dll")
        );
        assert!(inventory.bind(empty, false).is_err());
        assert!(inventory.bind(importer, true).is_err());
        inventory.assemblies[0].0.version = [2, 0, 0, 0];
        assert!(inventory.bind(importer, false).is_err());
        inventory.assemblies.clear();
        assert!(inventory.bind(importer, false).is_err());
    }
    #[test]
    fn unknown_security_paths_namespaces_and_multiple_entries_fail() {
        for body in [
            "<file name='../escape.dll'/>",
            "<file name='owned.dll' hash='anything'/>",
            "<file name='owned.dll'><other/></file>",
            "<noInheritable/>",
            "<dependency/>",
            "<file name='a.dll'/><file name='b.dll'/>",
            "<file xmlns='unknown' name='a.dll'/>",
            "<?x y?>",
            "text",
        ] {
            assert!(parse(wrap(body).as_bytes()).is_err(), "{body}");
        }
        for attr in [
            "publicKeyToken='0123456789abcdef'",
            "language='*'",
            "unknown='x'",
        ] {
            let xml = wrap(&format!(
                "<assemblyIdentity type='win32' name='Owned' version='1.0.0.0' processorArchitecture='x86' {attr}/>"
            ));
            assert!(parse(xml.as_bytes()).is_err());
        }
        for version in ["1.2.3", "1.2.3.65536", "1.2.-3.4", "1..3.4"] {
            assert!(parse(wrap(&format!("<assemblyIdentity type='win32' name='Owned' version='{version}' processorArchitecture='x86'/>")).as_bytes()).is_err());
        }
        assert!(parse(b"<!DOCTYPE x [<!ENTITY y 'z'>]><x/>").is_err());
        assert!(parse(&vec![b' '; 1024 * 1024 + 1]).is_err());
    }
}
