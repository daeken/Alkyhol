//! Establish only a proven empty embedded default context. Any semantics this
//! profile cannot represent stay unknown, never become a synthetic missing key.
use crate::{Image, ResourceName, Result};

fn empty_manifest(bytes: &[u8]) -> Result<()> {
    if bytes.len() > 1024 * 1024 {
        return Err("activation manifest size budget exceeded".into());
    }
    let text =
        std::str::from_utf8(bytes).map_err(|_| "non-UTF-8 activation manifests are unsupported")?;
    let doc = roxmltree::Document::parse_with_options(
        text.strip_prefix('\u{feff}').unwrap_or(text),
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: 1024,
        },
    )
    .map_err(|_| "invalid or unsupported activation manifest XML")?;
    let root = doc.root_element();
    if !root.has_tag_name(("urn:schemas-microsoft-com:asm.v1", "assembly"))
        || root.attribute("manifestVersion") != Some("1.0")
        || root.attributes().len() != 1
    {
        return Err("unsupported activation manifest root/attributes".into());
    }
    for node in doc.descendants() {
        if node == root || node.is_root() || node.is_comment() {
            continue;
        }
        if node.is_text()
            && node
                .text()
                .is_some_and(|v| v.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')))
        {
            continue;
        }
        return Err("activation manifest assembly binding/redirection is unsupported".into());
    }
    Ok(())
}

pub(crate) fn empty_default(exe: &Image<'_>, dlls: &[&Image<'_>]) -> Result<()> {
    let manifests: Vec<_> = exe
        .resources()?
        .into_iter()
        .filter(|r| r.kind == ResourceName::Id(24) && r.name == ResourceName::Id(1))
        .collect();
    if manifests.len() != 1 {
        return Err("activation requires one explicit embedded default manifest; external/system defaults and language selection are unsupported".into());
    }
    empty_manifest(manifests[0].bytes)?;
    for dll in dlls {
        for resource in dll.resources()? {
            if resource.kind == ResourceName::Id(24) {
                empty_manifest(resource.bytes).map_err(
                    |_| "activation of DLL manifests/dependent assemblies is unsupported",
                )?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn namespace_aware_empty_roots_bom_comments_and_whitespace_are_supported() {
        for xml in [
            "<assembly xmlns='urn:schemas-microsoft-com:asm.v1' manifestVersion='1.0'/>",
            "\u{feff}<?xml version='1.0' encoding='UTF-8'?><!--x--><a:assembly xmlns:a='urn:schemas-microsoft-com:asm.v1' manifestVersion='1.0'>\n<!--y-->\t</a:assembly>",
        ] {
            empty_manifest(xml.as_bytes()).unwrap();
        }
    }
    #[test]
    fn dependencies_unknown_namespaces_attributes_and_content_never_become_empty() {
        for inside in [
            "<dependency/>",
            "<file name='owned.dll'/>",
            "<assemblyIdentity name='Owned'/>",
            "<other:dependency xmlns:other='unknown'/>",
            "text",
            "<?unknown x?>",
        ] {
            let xml = format!(
                "<assembly xmlns='urn:schemas-microsoft-com:asm.v1' manifestVersion='1.0'>{inside}</assembly>"
            );
            assert!(empty_manifest(xml.as_bytes()).is_err());
        }
        for xml in [
            "<assembly manifestVersion='1.0'/>",
            "<assembly xmlns='unknown' manifestVersion='1.0'/>",
            "<assembly xmlns='urn:schemas-microsoft-com:asm.v1' manifestVersion='2.0'/>",
            "<assembly xmlns='urn:schemas-microsoft-com:asm.v1' manifestVersion='1.0' unknown='x'/>",
        ] {
            assert!(empty_manifest(xml.as_bytes()).is_err());
        }
    }
    #[test]
    fn malformed_xml_dtd_non_utf8_and_budgets_fail_closed() {
        for xml in [
            b"\xff".as_slice(),
            b"<assembly>",
            b"<!DOCTYPE assembly [<!ENTITY e 'x'>]><assembly/>",
            b"<assembly/><assembly/>",
            b"<assembly a='1' a='2'/>",
        ] {
            assert!(empty_manifest(xml).is_err());
        }
        assert!(empty_manifest(&vec![b' '; 1024 * 1024 + 1]).is_err());
        let xml = format!(
            "<assembly xmlns='urn:schemas-microsoft-com:asm.v1' manifestVersion='1.0'>{}</assembly>",
            "<!--x-->".repeat(1024)
        );
        assert!(empty_manifest(xml.as_bytes()).is_err());
    }
}
