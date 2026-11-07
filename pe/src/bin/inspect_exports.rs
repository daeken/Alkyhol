//! Read-only DLL metadata observer. Never maps or enters the input image.
use alkyhol_pe::{ExportTarget, Image};
use std::fmt::Write;
fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c < ' ' || c == '\u{7f}' => {
                write!(&mut out, "\\u{:04x}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 {
        return Err("usage: inspect_exports DLL".into());
    }
    let bytes = std::fs::read(&args[0]).map_err(|e| e.to_string())?;
    let table = Image::parse_dll(&bytes)?
        .exports()?
        .ok_or("DLL has no export directory")?;
    let entries = table
        .entries
        .into_iter()
        .map(|entry| {
            let names = entry
                .names
                .iter()
                .map(|n| quote(n))
                .collect::<Vec<_>>()
                .join(",");
            let target = match entry.target {
                ExportTarget::Address { rva } => format!("{{\"rva\":{rva}}}"),
                ExportTarget::Forwarder(s) => format!("{{\"forwarder\":{}}}", quote(&s)),
            };
            format!(
                "{{\"ordinal\":{},\"names\":[{}],\"target\":{}}}",
                entry.ordinal, names, target
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{{\"dll_name\":{},\"ordinal_base\":{},\"entries\":[{}]}}",
        quote(&table.dll_name),
        table.ordinal_base,
        entries
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn names_and_forwarders_are_json_escaped_not_rust_debug_strings() {
        assert_eq!(
            super::quote("a\"\\\n\t\u{7f}"),
            "\"a\\\"\\\\\\u000a\\u0009\\u007f\""
        );
    }
}
