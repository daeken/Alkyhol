//! Bounded ASCII scalar INI grammar measured against native profile APIs.
//! Unknown syntax/encoding/duplicates are errors, never absent keys or defaults.
use crate::Result;
#[derive(Default)]
pub(super) struct Ini {
    sections: Vec<Section>,
}
struct Section {
    name: String,
    entries: Vec<(String, String)>,
}
pub(super) fn name(text: &str) -> Result<()> {
    if text.is_empty()
        || text.len() > 1024
        || !text
            .bytes()
            .all(|b| (32..127).contains(&b) && !b"[]=;".contains(&b))
        || text.trim() != text
    {
        return Err("INI names require bounded ordinary ASCII spelling".into());
    }
    Ok(())
}
pub(super) fn value(text: &str) -> Result<()> {
    if text.len() > 32768
        || !text
            .bytes()
            .all(|b| (32..127).contains(&b) && b != b'\'' && b != b'"')
        || text.trim() != text
    {
        return Err("INI values require bounded unquoted ASCII without edge whitespace".into());
    }
    Ok(())
}
impl Ini {
    pub(super) fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 65536 || !bytes.is_ascii() {
            return Err("INI contents exceed bounded ASCII encoding profile".into());
        }
        let mut result = Self::default();
        if bytes.is_empty() {
            return Ok(result);
        }
        let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
        if !text.ends_with("\r\n") {
            return Err("INI contents require measured CRLF termination".into());
        }
        for line in text[..text.len() - 2].split("\r\n") {
            if let Some(section) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                name(section)?;
                if result
                    .sections
                    .iter()
                    .any(|s| s.name.eq_ignore_ascii_case(section))
                {
                    return Err("duplicate INI sections are outside the measured grammar".into());
                }
                result.sections.push(Section {
                    name: section.into(),
                    entries: Vec::new(),
                });
            } else {
                let (key, v) = line
                    .split_once('=')
                    .ok_or("INI contents contain unmeasured syntax")?;
                name(key)?;
                value(v)?;
                let section = result
                    .sections
                    .last_mut()
                    .ok_or("INI key precedes section")?;
                if section
                    .entries
                    .iter()
                    .any(|(k, _)| k.eq_ignore_ascii_case(key))
                {
                    return Err("duplicate INI keys are outside the measured grammar".into());
                }
                section.entries.push((key.into(), v.into()));
            }
        }
        Ok(result)
    }
    pub(super) fn get(&self, section: &str, key: &str) -> Result<Option<&str>> {
        name(section)?;
        name(key)?;
        Ok(self
            .sections
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(section))
            .and_then(|s| s.entries.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)))
            .map(|(_, v)| v.as_str()))
    }
    pub(super) fn set(&mut self, section: &str, key: &str, v: &str) -> Result<()> {
        name(section)?;
        name(key)?;
        value(v)?;
        let index = self
            .sections
            .iter()
            .position(|s| s.name.eq_ignore_ascii_case(section))
            .unwrap_or_else(|| {
                self.sections.push(Section {
                    name: section.into(),
                    entries: Vec::new(),
                });
                self.sections.len() - 1
            });
        let entries = &mut self.sections[index].entries;
        if let Some((_, old)) = entries
            .iter_mut()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
        {
            *old = v.into();
        } else {
            entries.push((key.into(), v.into()));
        }
        Ok(())
    }
    pub(super) fn bytes(&self) -> Result<Vec<u8>> {
        let mut text = String::new();
        for s in &self.sections {
            text.push_str(&format!("[{}]\r\n", s.name));
            for (k, v) in &s.entries {
                text.push_str(&format!("{k}={v}\r\n"));
            }
        }
        if text.len() > 65536 {
            return Err("INI serialization exceeds backing budget".into());
        }
        Ok(text.into_bytes())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_native_case_and_section_order_when_updating_or_inserting() {
        let mut ini = Ini::parse(b"[First]\r\nKey=old\r\n[Other]\r\nFlag=1\r\n").unwrap();
        ini.set("FIRST", "KEY", "new").unwrap();
        ini.set("First", "Empty", "").unwrap();
        assert_eq!(
            ini.bytes().unwrap(),
            b"[First]\r\nKey=new\r\nEmpty=\r\n[Other]\r\nFlag=1\r\n"
        );
        assert_eq!(ini.get("first", "key").unwrap(), Some("new"));
        assert_eq!(ini.get("first", "empty").unwrap(), Some(""));
        assert_eq!(ini.get("absent", "key").unwrap(), None);
    }
    #[test]
    fn unknown_syntax_does_not_become_missing_values_or_get_rewritten() {
        for bytes in [
            b"[S]\nKey=1\n".as_slice(),
            b"Key=1\r\n",
            b"[S]\r\nK=1\r\nk=2\r\n",
            b"[S]\r\n[S]\r\n",
            b"[S]\r\n;comment\r\n",
            b"[S]\r\nK=\"v\"\r\n",
            b"[S]\r\nK= value\r\n",
            b"\xff\xfe",
        ] {
            assert!(Ini::parse(bytes).is_err());
        }
        assert!(name(" leading").is_err());
        assert!(value("trailing ").is_err());
        assert!(value("桜").is_err());
    }
}
