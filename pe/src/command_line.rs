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
