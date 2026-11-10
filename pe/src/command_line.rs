use crate::Result;

/// Encode valid Unicode argv using Windows CRT/CommandLineToArgvW quoting.
/// argv[0] uses the distinct executable-name rule; quotes there are invalid.
/// The returned UTF-16 includes its NUL and obeys the 32767-unit process limit.
pub fn windows_command_line(argv: &[String]) -> Result<Vec<u16>> {
    let executable = argv.first().ok_or("missing guest executable name")?;
    if executable.is_empty() || executable.contains(['"', '\0']) {
        return Err("invalid guest executable name".into());
    }
    let mut command = format!("\"{executable}\"");
    for argument in &argv[1..] {
        if argument.contains('\0') {
            return Err("NUL in guest argument".into());
        }
        command.push_str(" \"");
        let mut slashes = 0;
        for ch in argument.chars() {
            if ch == '\\' {
                slashes += 1;
                continue;
            }
            command.extend(std::iter::repeat_n(
                '\\',
                if ch == '"' { slashes * 2 + 1 } else { slashes },
            ));
            slashes = 0;
            command.push(ch);
        }
        command.extend(std::iter::repeat_n('\\', slashes * 2));
        command.push('"');
    }
    let mut wide: Vec<u16> = command.encode_utf16().collect();
    wide.push(0);
    if wide.len() > 32767 {
        return Err("Windows command line exceeds 32767 UTF-16 units".into());
    }
    Ok(wide)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quotes_empty_space_quote_backslash_and_surrogate_arguments() {
        let args = [r"C:\a b\x.exe", "", "a b", "x\"y", "tail\\", "λ🌸"];
        let actual = windows_command_line(&args.map(String::from)).unwrap();
        let expected = "\"C:\\a b\\x.exe\" \"\" \"a b\" \"x\\\"y\" \"tail\\\\\" \"λ🌸\"\0";
        assert_eq!(actual, expected.encode_utf16().collect::<Vec<_>>());
    }
    #[test]
    fn ambiguous_or_unrepresentable_commands_are_rejected() {
        assert!(windows_command_line(&[]).is_err());
        for argv in [vec![""], vec!["a\"b"], vec!["a\0b"], vec!["app", "a\0b"]] {
            assert!(
                windows_command_line(&argv.into_iter().map(String::from).collect::<Vec<_>>())
                    .is_err()
            );
        }
        assert!(windows_command_line(&["x".into(), "a".repeat(32759)]).is_ok());
        assert!(windows_command_line(&["x".into(), "🌸".repeat(16384)]).is_err());
    }
}

// Deliberately bounded inverse of our canonical Windows command encoding.
// Other valid Shell32 command-line spellings require separate native evidence.
#[cfg(any(
    test,
    all(
        target_os = "macos",
        any(target_arch = "x86_64", feature = "native-runtime32")
    )
))]
pub(crate) fn canonical_arguments(text: &str) -> Result<Vec<String>> {
    let Some(rest) = text.strip_prefix('"') else {
        return Err("noncanonical executable quoting".into());
    };
    let end = rest.find('"').ok_or("unterminated executable name")?;
    let mut arguments = vec![rest[..end].to_owned()];
    let mut rest = &rest[end + 1..];
    while !rest.is_empty() {
        rest = rest
            .strip_prefix(" \"")
            .ok_or("noncanonical argument separator")?;
        let mut chars = rest.char_indices().peekable();
        let mut value = String::new();
        let mut closed = None;
        while let Some((position, ch)) = chars.next() {
            if ch == '"' {
                closed = Some(position + 1);
                break;
            }
            if ch != '\\' {
                value.push(ch);
                continue;
            }
            let mut slashes = 1;
            while chars.peek().is_some_and(|&(_, ch)| ch == '\\') {
                chars.next();
                slashes += 1;
            }
            if chars.peek().is_some_and(|&(_, ch)| ch == '"') {
                value.extend(std::iter::repeat_n('\\', slashes / 2));
                let (position, _) = chars.next().unwrap();
                if slashes % 2 == 0 {
                    closed = Some(position + 1);
                    break;
                } else {
                    value.push('"');
                }
            } else {
                value.extend(std::iter::repeat_n('\\', slashes));
            }
        }
        rest = &rest[closed.ok_or("unterminated argument")?..];
        arguments.push(value);
    }
    let encoded = windows_command_line(&arguments)?;
    if encoded[..encoded.len() - 1] != text.encode_utf16().collect::<Vec<_>>() {
        return Err("noncanonical command line".into());
    }
    Ok(arguments)
}
#[cfg(test)]
mod canonical_tests {
    use super::*;
    #[test]
    fn canonical_inverse_recovers_unicode_empty_quotes_and_every_backslash_parity() {
        for slashes in 0..12 {
            let args = vec![
                "/a b/λ🌸.exe".into(),
                String::new(),
                " ".into(),
                format!("a{}\"b{}", "\\".repeat(slashes), "\\".repeat(slashes)),
                "λ🌸\t\n".into(),
            ];
            let encoded = windows_command_line(&args).unwrap();
            let text = String::from_utf16(&encoded[..encoded.len() - 1]).unwrap();
            assert_eq!(canonical_arguments(&text).unwrap(), args);
        }
    }
    #[test]
    fn unsupported_spellings_nuls_and_truncation_are_not_silently_reinterpreted() {
        for text in [
            "",
            "app arg",
            "\"\"",
            " \"app\"",
            "\"app\" ",
            "\"app\"  \"a\"",
            "\"app\" \"x",
            "\"app\" \"x\"junk",
            "\"app\" \"a\0b\"",
        ] {
            assert!(canonical_arguments(text).is_err(), "{text:?}");
        }
    }
}
