//! Explicit caller-installed publisher policy metadata. This performs bounded
//! version selection, not catalog/PKI validation or a Windows installer action.
use crate::{
    Result,
    activation_binding::{Identity, NS, attrs, children, document, identity, version},
};

pub(crate) struct Policy {
    owner: Identity,
    name: String,
    ranges: Vec<([u16; 4], [u16; 4], [u16; 4])>,
}
impl Policy {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self> {
        let doc = document(bytes)?;
        let root = children(doc.root_element())?;
        if root.len() != 2
            || !root[0].has_tag_name((NS, "assemblyIdentity"))
            || !root[1].has_tag_name((NS, "dependency"))
        {
            return Err("unsupported publisher policy root schema".into());
        }
        let owner = identity(root[0], "win32-policy", true)?;
        if owner.token.is_none() {
            return Err("publisher policy requires an explicit token identity".into());
        }
        attrs(root[1], &[])?;
        let dependents = children(root[1])?;
        if dependents.len() != 1 || !dependents[0].has_tag_name((NS, "dependentAssembly")) {
            return Err("unsupported publisher policy dependency".into());
        }
        attrs(dependents[0], &[])?;
        let rules = children(dependents[0])?;
        if rules.len() < 2 || rules.len() > 65 || !rules[0].has_tag_name((NS, "assemblyIdentity")) {
            return Err("unsupported publisher policy rule count/selector".into());
        }
        let target = identity(rules[0], "win32", false)?;
        let [major, minor, ..] = owner.version;
        if owner.name != format!("policy.{major}.{minor}.{}", target.name)
            || owner.token != target.token
        {
            return Err("publisher policy identity does not match its target family".into());
        }
        let mut ranges = Vec::new();
        for node in rules.iter().skip(1) {
            attrs(*node, &["oldVersion", "newVersion"])?;
            if !node.has_tag_name((NS, "bindingRedirect")) || !children(*node)?.is_empty() {
                return Err("unsupported publisher policy rule content".into());
            }
            let old = node
                .attribute("oldVersion")
                .ok_or("missing publisher policy source version")?;
            let (lo, hi) = old.split_once('-').unwrap_or((old, old));
            let lo = version(lo)?;
            let hi = version(hi)?;
            let new = version(
                node.attribute("newVersion")
                    .ok_or("missing publisher policy target version")?,
            )?;
            if lo > hi
                || lo[..2] != owner.version[..2]
                || hi[..2] != owner.version[..2]
                || new != owner.version
            {
                return Err("unsupported publisher policy range/family/target version".into());
            }
            if ranges.iter().any(|&(a, b, _)| lo <= b && a <= hi) {
                return Err("overlapping publisher policy ranges are unsupported".into());
            }
            ranges.push((lo, hi, new));
        }
        Ok(Self {
            owner,
            name: target.name,
            ranges,
        })
    }
    pub(crate) fn same_family(&self, other: &Self) -> bool {
        self.name == other.name
            && self.owner.token == other.owner.token
            && self.owner.version[..2] == other.owner.version[..2]
    }
    pub(crate) fn apply(&self, requested: &Identity) -> Identity {
        let mut out = requested.clone();
        if requested.name == self.name && requested.token == self.owner.token {
            for &(lo, hi, new) in &self.ranges {
                if lo <= requested.version && requested.version <= hi {
                    out.version = new;
                    break;
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn policy(rules: &str) -> String {
        format!(
            "<assembly xmlns='{NS}' manifestVersion='1.0'><assemblyIdentity type='win32-policy' name='policy.1.0.Alkyhol.OwnedCRT' version='1.0.5.0' processorArchitecture='x86' publicKeyToken='0123456789abcdef'/><dependency><dependentAssembly><assemblyIdentity type='win32' name='Alkyhol.OwnedCRT' processorArchitecture='x86' publicKeyToken='0123456789abcdef'/>{rules}</dependentAssembly></dependency></assembly>"
        )
    }
    #[test]
    fn actual_version_ranges_are_inclusive_and_cannot_change_another_family() {
        let xml = policy(
            "<bindingRedirect oldVersion='1.0.1.0-1.0.2.0' newVersion='1.0.5.0'/><bindingRedirect oldVersion='1.0.3.0-1.0.5.0' newVersion='1.0.5.0'/>",
        );
        let p = Policy::parse(xml.as_bytes()).unwrap();
        let original = Identity {
            name: "alkyhol.ownedcrt".into(),
            version: [1, 0, 0, 65535],
            token: Some([1, 35, 69, 103, 137, 171, 205, 239]),
        };
        for (v, expected) in [
            ([1, 0, 0, 65535], [1, 0, 0, 65535]),
            ([1, 0, 1, 0], [1, 0, 5, 0]),
            ([1, 0, 2, 0], [1, 0, 5, 0]),
            ([1, 0, 2, 1], [1, 0, 2, 1]),
            ([1, 0, 3, 0], [1, 0, 5, 0]),
            ([1, 0, 5, 1], [1, 0, 5, 1]),
            ([2, 0, 1, 0], [2, 0, 1, 0]),
        ] {
            let mut id = original.clone();
            id.version = v;
            assert_eq!(p.apply(&id).version, expected);
            assert_eq!(id.version, v);
        }
        let mut other = original;
        other.version = [1, 0, 1, 0];
        other.token = None;
        assert_eq!(p.apply(&other), other);
        other.name = "another.family".into();
        assert_eq!(p.apply(&other), other);
        assert!(p.same_family(&Policy::parse(xml.as_bytes()).unwrap()));
    }
    #[test]
    fn unknown_policy_schemas_ambiguous_ranges_and_version_families_stop() {
        for rule in [
            "<bindingRedirect oldVersion='1.0.2.0-1.0.1.0' newVersion='1.0.5.0'/>",
            "<bindingRedirect oldVersion='2.0.1.0' newVersion='1.0.5.0'/>",
            "<bindingRedirect oldVersion='1.0.1.0' newVersion='1.0.6.0'/>",
            "<bindingRedirect oldVersion='1.0.1.0' newVersion='1.0.5.0' unknown='x'/>",
            "<bindingRedirect oldVersion='1.0.1.0' newVersion='1.0.5.0'><unknown/></bindingRedirect>",
            "<bindingRedirect oldVersion='1.0.1.0-1.0.2.0' newVersion='1.0.5.0'/><bindingRedirect oldVersion='1.0.2.0-1.0.3.0' newVersion='1.0.5.0'/>",
        ] {
            assert!(Policy::parse(policy(rule).as_bytes()).is_err(), "{rule}");
        }
        let valid = policy("<bindingRedirect oldVersion='1.0.1.0' newVersion='1.0.5.0'/>");
        for (from, to) in [
            ("policy.1.0.", "policy.2.0."),
            ("type='win32-policy'", "type='win32'"),
            ("publicKeyToken='0123456789abcdef'", "publicKeyToken='bad'"),
        ] {
            assert!(Policy::parse(valid.replacen(from, to, 1).as_bytes()).is_err());
        }
    }
}
