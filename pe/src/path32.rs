//! Lexical full-path family, using real process-owned Z: current directory.
use super::{LoadedImage32, bulk32, directory32, startup32, unsupported};
use crate::Result;
fn resolve(current: &str, input: &str) -> Result<String> {
    if input.is_empty() || input.len() > 32767 || input.starts_with("\\\\") {
        return Err("full-path input is outside the ordinary drive profile".into());
    }
    let input = input.replace('/', "\\");
    let combined = if input.as_bytes().get(1) == Some(&b':') {
        if !input[..1].eq_ignore_ascii_case("Z") {
            return Err("full-path query uses an unconfigured drive".into());
        }
        if input.as_bytes().get(2) == Some(&b'\\') {
            format!("Z:{}", &input[2..])
        } else {
            format!("{}\\{}", current.trim_end_matches('\\'), &input[2..])
        }
    } else if input.starts_with('\\') {
        format!("Z:{input}")
    } else {
        format!("{}\\{input}", current.trim_end_matches('\\'))
    };
    let mut parts = Vec::new();
    for part in combined[3..].split('\\') {
        match part {
            "" | "." => (),
            ".." => {
                parts.pop();
            }
            v => {
                if v.ends_with(['.', ' ']) || v.chars().any(|c| c < ' ' || "<>:\"|".contains(c)) {
                    return Err("full-path component normalization is unmeasured".into());
                }
                let stem = v.split('.').next().unwrap().to_ascii_uppercase();
                if ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"].contains(&stem.as_str())
                    || ["COM", "LPT"].iter().any(|p| {
                        stem.strip_prefix(p).is_some_and(|s| {
                            matches!(s, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
                        })
                    })
                {
                    return Err("full-path DOS device names are unsupported".into());
                }
                parts.push(v);
            }
        }
    }
    let mut full = format!("Z:\\{}", parts.join("\\"));
    if input.ends_with('\\') && !full.ends_with('\\') {
        full.push('\\');
    }
    if full.encode_utf16().count() >= 32767 {
        return Err("full-path result exceeds the UTF-16 bound".into());
    }
    Ok(full)
}
pub(super) fn full(image: &LoadedImage32<'_>, args: &[u32; 8], ansi: bool) -> u32 {
    let text = if ansi {
        bulk32::ascii(image, args[0])
    } else {
        let units = startup32::read_units::<u16>(image, args[0], -1)
            .unwrap_or_else(|| unsupported("full-path input outside checked UTF-16 storage"));
        String::from_utf16(&units[..units.len() - 1])
            .unwrap_or_else(|_| unsupported("full-path input is invalid UTF-16"))
    };
    if image.cpu.diagnostic.is_some() {
        eprintln!("PE32 diagnostic full path: input={text:?}; ansi={ansi}");
    }
    let current = directory32::current(image).unwrap_or_else(|e| unsupported(&e));
    let path = resolve(&current, &text).unwrap_or_else(|e| unsupported(&e));
    if ansi && !path.is_ascii() {
        unsupported("ANSI full-path output requires ASCII spelling");
    }
    let units: Vec<u16> = path.encode_utf16().collect();
    let count = if ansi { path.len() } else { units.len() };
    if args[1] as usize <= count {
        if args[1] > 0 {
            if !(args[2] as usize).is_multiple_of(if ansi { 1 } else { 2 })
                || !image.permits(args[2] as usize, if ansi { 1 } else { 2 }, true)
            {
                unsupported("full-path short buffer outside writable guest storage");
            }
            unsafe {
                if ansi {
                    (args[2] as *mut u8).write(0);
                } else {
                    (args[2] as *mut u16).write(0);
                }
            }
        }
        return count as u32 + 1;
    }
    let width = if ansi { 1 } else { 2 };
    if !(args[2] as usize).is_multiple_of(width)
        || !image.permits(args[2] as usize, (count + 1) * width, true)
    {
        unsupported("full-path output outside checked writable guest storage");
    }
    if args[3] != 0 && (!args[3].is_multiple_of(4) || !image.permits(args[3] as usize, 4, true)) {
        unsupported("full-path file-part pointer outside writable guest storage");
    }
    let part = if path.ends_with('\\') {
        0
    } else {
        let start = path.rfind('\\').unwrap() + 1;
        let offset = if ansi {
            start
        } else {
            path[..start].encode_utf16().count()
        };
        args[2]
            .checked_add((offset * width) as u32)
            .unwrap_or_else(|| unsupported("full-path file-part address overflow"))
    };
    unsafe {
        if ansi {
            std::ptr::copy_nonoverlapping(path.as_ptr(), args[2] as *mut u8, count);
            (args[2] as *mut u8).add(count).write(0);
        } else {
            std::ptr::copy_nonoverlapping(units.as_ptr(), args[2] as *mut u16, count);
            (args[2] as *mut u16).add(count).write(0);
        }
        if args[3] != 0 {
            (args[3] as *mut u32).write(part);
        }
    }
    count as u32
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolves_lexically_without_inventing_file_existence() {
        let cwd = "Z:\\guest\\root";
        for (input, expected) in [
            ("missing.ini", "Z:\\guest\\root\\missing.ini"),
            (".\\a\\..\\file", "Z:\\guest\\root\\file"),
            ("..\\file", "Z:\\guest\\file"),
            ("\\rooted", "Z:\\rooted"),
            ("z:relative", "Z:\\guest\\root\\relative"),
            ("Z:\\..\\file", "Z:\\file"),
            ("folder\\", "Z:\\guest\\root\\folder\\"),
            ("*.txt", "Z:\\guest\\root\\*.txt"),
            ("桜.ini", "Z:\\guest\\root\\桜.ini"),
        ] {
            assert_eq!(resolve(cwd, input).unwrap(), expected);
        }
    }
    #[test]
    fn rejects_unknown_namespaces_and_unmeasured_trimming() {
        for path in [
            "",
            "C:\\file",
            "\\\\server\\share",
            "name. ",
            "NUL",
            "COM1.txt",
        ] {
            assert!(resolve("Z:\\guest", path).is_err());
        }
    }
}
