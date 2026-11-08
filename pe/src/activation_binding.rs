//! Explicit, bounded assembly metadata for the PE32 activation path. This is
//! not a Windows installation/search policy. Unknown XML/bindings stay errors.
use crate::{Image, ResourceName, Result};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
};

pub(crate) const NS: &str = "urn:schemas-microsoft-com:asm.v1";
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Identity {
    pub name: String,
    pub version: [u16; 4],
    pub token: Option<[u8; 8]>,
}
#[derive(Clone, Debug)]
struct Manifest {
    identity: Option<Identity>,
    dependency: Option<Identity>,
    files: Vec<String>,
    no_inheritable: bool,
}
#[derive(Clone, Debug)]
pub(crate) struct Template {
    pub files: Vec<String>,
    pub active: bool,
    pub roster: u32,
}

pub(crate) fn attrs(node: roxmltree::Node<'_, '_>, allowed: &[&str]) -> Result<()> {
    if node
        .attributes()
        .any(|a| a.namespace().is_some() || !allowed.contains(&a.name()))
    {
        return Err("unsupported activation manifest attributes".into());
    }
    Ok(())
}
pub(crate) fn children<'a, 'i>(
    node: roxmltree::Node<'a, 'i>,
) -> Result<Vec<roxmltree::Node<'a, 'i>>> {
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
pub(crate) fn identity(
    node: roxmltree::Node<'_, '_>,
    kind: &str,
    with_version: bool,
) -> Result<Identity> {
    attrs(
        node,
        if with_version {
            &[
                "type",
                "name",
                "version",
                "processorArchitecture",
                "publicKeyToken",
            ]
        } else {
            &["type", "name", "processorArchitecture", "publicKeyToken"]
        },
    )?;
    if node.attribute("type") != Some(kind)
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
    let version = if with_version {
        version(
            node.attribute("version")
                .ok_or("missing assembly version")?,
        )?
    } else {
        [0; 4]
    };
    // The zero version in a policy's family selector is internal, never an
    // installed version. Selectors compare only the name/token/platform fields.
    Ok(Identity {
        name: name.to_ascii_lowercase(),
        version,
        token: node.attribute("publicKeyToken").map(hex::<8>).transpose()?,
    })
}
pub(crate) fn version(text: &str) -> Result<[u16; 4]> {
    let parts: Vec<_> = text.split('.').collect();
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
    Ok(version)
}
fn hex<const N: usize>(text: &str) -> Result<[u8; N]> {
    if text.len() != N * 2 || !text.is_ascii() {
        return Err("invalid activation hexadecimal metadata length".into());
    }
    let mut out = [0; N];
    for (i, part) in text.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let high = (part[0] as char)
            .to_digit(16)
            .ok_or("invalid activation hexadecimal metadata")?;
        let low = (part[1] as char)
            .to_digit(16)
            .ok_or("invalid activation hexadecimal metadata")?;
        out[i] = (high * 16 + low) as u8;
    }
    Ok(out)
}
fn xml_elements<'a, 'i>(node: roxmltree::Node<'a, 'i>) -> Result<Vec<roxmltree::Node<'a, 'i>>> {
    let mut out = Vec::new();
    for n in node.children() {
        if n.is_comment() || n.is_text() && n.text().is_some_and(|s| s.trim().is_empty()) {
            continue;
        }
        if !n.is_element() {
            return Err("unsupported assembly hash content".into());
        }
        out.push(n);
    }
    Ok(out)
}
fn sha1_metadata(node: roxmltree::Node<'_, '_>) -> Result<()> {
    const V2: &str = "urn:schemas-microsoft-com:asm.v2";
    const DS: &str = "http://www.w3.org/2000/09/xmldsig#";
    attrs(node, &[])?;
    let c = xml_elements(node)?;
    if !node.has_tag_name((V2, "hash"))
        || c.len() != 3
        || !c[0].has_tag_name((DS, "Transforms"))
        || !c[1].has_tag_name((DS, "DigestMethod"))
        || !c[2].has_tag_name((DS, "DigestValue"))
    {
        return Err("unsupported assembly hash schema".into());
    }
    attrs(c[0], &[])?;
    let transforms = xml_elements(c[0])?;
    if transforms.len() != 1 || !transforms[0].has_tag_name((DS, "Transform")) {
        return Err("unsupported assembly hash transform list".into());
    }
    for (n, algorithm) in [
        (
            transforms[0],
            "urn:schemas-microsoft-com:HashTransforms.Identity",
        ),
        (c[1], "http://www.w3.org/2000/09/xmldsig#sha1"),
    ] {
        attrs(n, &["Algorithm"])?;
        if n.attribute("Algorithm") != Some(algorithm) || !xml_elements(n)?.is_empty() {
            return Err("unsupported assembly hash algorithm".into());
        }
    }
    attrs(c[2], &[])?;
    if c[2].children().any(|n| !n.is_text()) {
        return Err("unsupported digest value content".into());
    }
    let value = c[2].text().unwrap_or("").trim();
    const BASE64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    if value.len() != 28 || !value.ends_with('=') {
        return Err("invalid SHA1 digest value length".into());
    }
    let mut bits = 0u32;
    let mut count = 0;
    let mut length = 0;
    for b in value.as_bytes().iter().take(27) {
        let digit = BASE64
            .iter()
            .position(|v| v == b)
            .ok_or("invalid SHA1 digest base64")? as u32;
        bits = (bits << 6) | digit;
        count += 6;
        if count >= 8 {
            count -= 8;
            length += 1;
            bits &= (1 << count) - 1;
        }
    }
    if length != 20 || bits != 0 {
        return Err("noncanonical SHA1 digest value".into());
    }
    Ok(())
}
pub(crate) fn document(bytes: &[u8]) -> Result<roxmltree::Document<'_>> {
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
    Ok(doc)
}
fn parse(bytes: &[u8]) -> Result<Manifest> {
    let doc = document(bytes)?;
    let root = doc.root_element();
    let mut out = Manifest {
        identity: None,
        dependency: None,
        files: Vec::new(),
        no_inheritable: false,
    };
    for node in children(root)? {
        match node.tag_name().name() {
            "assemblyIdentity" if out.identity.is_none() => {
                out.identity = Some(identity(node, "win32", true)?)
            }
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
                out.dependency = Some(identity(ids[0], "win32", true)?);
            }
            "file" => {
                attrs(node, &["name", "hash", "hashalg"])?;
                match (node.attribute("hash"), node.attribute("hashalg")) {
                    (None, None) => (),
                    (Some(hash), Some(alg)) if alg.eq_ignore_ascii_case("SHA1") => {
                        hex::<20>(hash)?;
                    }
                    _ => return Err("unsupported assembly file hash declaration".into()),
                }
                let metadata = xml_elements(node)?;
                if metadata.len() > 1 {
                    return Err("duplicate assembly hash metadata".into());
                }
                for n in metadata {
                    sha1_metadata(n)?;
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
                if out.files.len() >= 64 || out.files.iter().any(|n| n == name) {
                    return Err("duplicate or excessive assembly file entries".into());
                }
                out.files.push(name.to_owned());
            }
            "noInheritable"
                if !out.no_inheritable
                    && out.identity.is_none()
                    && out.files.is_empty()
                    && out.dependency.is_none() =>
            {
                attrs(node, &[])?;
                if !children(node)?.is_empty() {
                    return Err("invalid noInheritable content".into());
                }
                // Only one guest thread exists; creation/inheritance APIs remain
                // unsupported. No foreign thread/window context is fabricated.
                out.no_inheritable = true;
            }
            _ => return Err("assembly manifest element/multiplicity is unsupported".into()),
        }
    }
    Ok(out)
}
pub(crate) fn read(path: &Path) -> Result<Vec<u8>> {
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
    assemblies: Vec<(Identity, Vec<String>)>,
    policies: Vec<crate::activation_policy::Policy>,
}
impl Inventory {
    pub(crate) fn new(
        paths: &[PathBuf],
        policy_paths: &[PathBuf],
        origins: &BTreeMap<String, PathBuf>,
    ) -> Result<Self> {
        if paths.len() > 64 || policy_paths.len() > 64 {
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
            if ![1, 3].contains(&manifest.files.len()) {
                return Err(
                    "assembly file count outside measured one/three-file section profiles".into(),
                );
            }
            if manifest.dependency.is_some() {
                return Err("nested assembly dependencies are unsupported".into());
            }
            if assemblies.iter().any(|(old, _)| old == &id) {
                return Err("duplicate assembly identity".into());
            }
            for file in &manifest.files {
                let source = std::fs::canonicalize(
                    path.parent().ok_or("assembly manifest parent")?.join(file),
                )
                .map_err(|e| format!("assembly DLL source: {e}"))?;
                let meta = std::fs::metadata(&source).map_err(|e| e.to_string())?;
                if !meta.is_file() || meta.len() > 1024 * 1024 * 1024 {
                    return Err("assembly member is not a bounded regular file".into());
                }
                if origins.get(file).is_some_and(|origin| origin != &source) {
                    return Err("assembly DLL is not the provided file-backed module".into());
                }
            }
            assemblies.push((id, manifest.files));
        }
        let mut policies = Vec::new();
        for path in policy_paths {
            let policy = crate::activation_policy::Policy::parse(&read(path)?)?;
            if policies.iter().any(|old| policy.same_family(old)) {
                return Err("ambiguous installed publisher policy family".into());
            }
            policies.push(policy);
        }
        Ok(Self {
            assemblies,
            policies,
        })
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
        if m.identity.is_some() || !m.files.is_empty() || m.no_inheritable {
            return Err("root/module assembly identity/files are unsupported".into());
        }
        if root && m.dependency.is_some() {
            return Err("nonempty executable default activation scopes are unsupported".into());
        }
        let files = match m.dependency {
            None => Vec::new(),
            Some(mut id) => {
                for policy in &self.policies {
                    id = policy.apply(&id);
                }
                self.assemblies
                    .iter()
                    .find(|(candidate, _)| candidate == &id)
                    .map(|(_, file)| file.clone())
                    .ok_or("unresolved exact assembly dependency after explicit installed policy")?
            }
        };
        Ok(Template {
            files,
            active: !root,
            roster: 2,
        })
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
                version: [1, 0, 0, 0],
                token: None,
            }
        );
        assert_eq!(m.files, ["ctx-provider.dll"]);
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
                    token: None,
                },
                vec!["ctx-provider.dll".into()],
            )],
            policies: Vec::new(),
        };
        assert!(!inventory.bind(empty, true).unwrap().active);
        assert!(inventory.bind(empty, false).unwrap().active);
        assert_eq!(
            inventory.bind(importer, false).unwrap().files,
            ["ctx-provider.dll"]
        );
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
            "<noInheritable/><noInheritable/>",
            "<dependency/>",
            "<file name='a.dll'/><file name='a.dll'/>",
            "<file xmlns='unknown' name='a.dll'/>",
            "<?x y?>",
            "text",
        ] {
            assert!(parse(wrap(body).as_bytes()).is_err(), "{body}");
        }
        for attr in ["publicKeyToken='0123'", "language='*'", "unknown='x'"] {
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
    #[test]
    fn private_hash_metadata_is_checked_without_inventing_runtime_integrity_enforcement() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <noInheritable/>
  <assemblyIdentity type="win32" name="Alkyhol.OwnedCRT" version="1.0.0.0" processorArchitecture="x86"/>
  <file name="ctx-provider.dll" hash="0000000000000000000000000000000000000000" hashalg="SHA1">
    <asmv2:hash xmlns:asmv2="urn:schemas-microsoft-com:asm.v2" xmlns:dsig="http://www.w3.org/2000/09/xmldsig#">
      <dsig:Transforms><dsig:Transform Algorithm="urn:schemas-microsoft-com:HashTransforms.Identity"/></dsig:Transforms>
      <dsig:DigestMethod Algorithm="http://www.w3.org/2000/09/xmldsig#sha1"/>
      <dsig:DigestValue>AAAAAAAAAAAAAAAAAAAAAAAAAAA=</dsig:DigestValue>
    </asmv2:hash>
  </file>
</assembly>
"#;
        let manifest = parse(xml.as_bytes()).unwrap();
        assert!(manifest.no_inheritable);
        assert_eq!(manifest.files, ["ctx-provider.dll"]);
        for (from, to) in [
            ("HashTransforms.Identity", "HashTransforms.Unknown"),
            ("#sha1", "#sha256"),
            (
                "AAAAAAAAAAAAAAAAAAAAAAAAAAA=",
                "AAAAAAAAAAAAAAAAAAAAAAAAAAB=",
            ),
            ("0000000000000000000000000000000000000000", "bad"),
            ("urn:schemas-microsoft-com:asm.v2", "unknown"),
        ] {
            assert!(parse(xml.replace(from, to).as_bytes()).is_err(), "{from}");
        }
    }
}
