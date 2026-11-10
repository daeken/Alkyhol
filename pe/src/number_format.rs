//! Bounded decimal-string arithmetic for the reached custom NUMBERFMTW profile.
//! No host locale, binary floating point, or machine-word integer conversion.
pub(super) struct Format<'a> {
    pub digits: u32,
    pub leading_zero: u32,
    pub grouping: u32,
    pub decimal: &'a [u16],
    pub thousands: &'a [u16],
    pub negative_order: u32,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Error {
    Invalid,
    Unsupported(&'static str),
}
pub(super) fn format(value: &[u16], f: &Format<'_>) -> Result<Vec<u16>, Error> {
    if f.digits > 9 || f.leading_zero > 1 || f.negative_order > 4 || f.decimal.is_empty() {
        return Err(Error::Invalid);
    }
    if value.len() > 4096 || f.decimal.len() > 4 || f.thousands.len() > 4 {
        return Err(Error::Unsupported(
            "number-format string length outside bounded profile",
        ));
    }
    if f.decimal
        .iter()
        .chain(f.thousands)
        .any(|c| (48..=57).contains(c))
    {
        return Err(Error::Unsupported(
            "number-format digit-bearing separators are unmeasured",
        ));
    }
    let (first_group, following_group) = match f.grouping {
        0 => (0, 0),
        n @ 1..=9 => (n as usize, n as usize),
        10 => (1, 0),
        32 => (3, 2),
        _ => {
            return Err(Error::Unsupported(
                "number-format grouping outside bounded profile",
            ));
        }
    };
    let negative = value.first() == Some(&45);
    let mut parts = value[usize::from(negative)..].split(|&c| c == 46);
    let whole = parts.next().unwrap_or_default();
    let fraction = parts.next().unwrap_or_default();
    if parts.next().is_some() || whole.iter().chain(fraction).any(|c| !(48..=57).contains(c)) {
        return Err(Error::Invalid);
    }
    // Native Windows accepts empty, minus-only and point-only strings as zero.
    let start = whole.iter().position(|&c| c != 48).unwrap_or(whole.len());
    let mut integer = if start == whole.len() {
        vec![48]
    } else {
        whole[start..].to_vec()
    };
    let n = f.digits as usize;
    let mut decimals = vec![48; n];
    let kept = n.min(fraction.len());
    decimals[..kept].copy_from_slice(&fraction[..kept]);
    if fraction.get(n).is_some_and(|&c| c >= 53) {
        let mut carry = true;
        for c in decimals.iter_mut().rev().chain(integer.iter_mut().rev()) {
            if *c == 57 {
                *c = 48;
            } else {
                *c += 1;
                carry = false;
                break;
            }
        }
        if carry {
            integer.insert(0, 49);
        }
    }
    let negative = negative && integer.iter().chain(&decimals).any(|&c| c != 48);
    let mut result = Vec::new();
    if integer != [48] || f.leading_zero != 0 || n == 0 {
        let mut chunks = Vec::new();
        let mut end = integer.len();
        let mut group = first_group;
        while group != 0 && end > group {
            chunks.push(&integer[end - group..end]);
            end -= group;
            group = following_group;
        }
        chunks.push(&integer[..end]);
        for (i, chunk) in chunks.iter().rev().enumerate() {
            if i != 0 {
                result.extend_from_slice(f.thousands);
            }
            result.extend_from_slice(chunk);
        }
    }
    if n != 0 {
        result.extend_from_slice(f.decimal);
        result.extend_from_slice(&decimals);
    }
    if negative {
        match f.negative_order {
            0 => {
                result.insert(0, 40);
                result.push(41);
            }
            1 => result.insert(0, 45),
            2 => {
                result.insert(0, 32);
                result.insert(0, 45);
            }
            3 => result.push(45),
            4 => result.extend_from_slice(&[32, 45]),
            _ => unreachable!(),
        }
    }
    result.push(0);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(value: &str, digits: u32, grouping: u32, zero: u32, order: u32) -> String {
        let f = Format {
            digits,
            leading_zero: zero,
            grouping,
            decimal: &[46],
            thousands: &[44],
            negative_order: order,
        };
        let v = format(&value.encode_utf16().collect::<Vec<_>>(), &f).unwrap();
        assert_eq!(v.last(), Some(&0));
        String::from_utf16(&v[..v.len() - 1]).unwrap()
    }
    #[test]
    fn arbitrary_precision_rounding_carries_and_suppresses_rounded_negative_zero() {
        assert_eq!(
            run("123456789012345678901234567890.5", 0, 3, 1, 1),
            "123,456,789,012,345,678,901,234,567,891"
        );
        assert_eq!(run("-00099.9995", 3, 0, 1, 1), "-100.000");
        assert_eq!(run("-0.0049", 2, 3, 1, 1), "0.00");
        assert_eq!(run("-0.005", 2, 3, 1, 1), "-0.01");
        assert_eq!(run("0000123.004500", 9, 3, 1, 1), "123.004500000");
    }
    #[test]
    fn grouping_and_negative_modes_retain_their_distinct_layouts() {
        for (order, expected) in ["(.50)", "-.50", "- .50", ".50-", ".50 -"]
            .iter()
            .enumerate()
        {
            assert_eq!(run("-.5", 2, 3, 0, order as u32), *expected);
        }
        assert_eq!(run("123456789", 0, 32, 1, 1), "12,34,56,789");
        assert_eq!(run("123456789", 0, 10, 1, 1), "12345678,9");
        assert_eq!(run("0", 0, 0, 0, 0), "0");
        for v in ["", "-", ".", "-."] {
            assert_eq!(run(v, 2, 3, 1, 1), "0.00");
        }
    }
    #[test]
    fn invalid_numeric_syntax_and_profile_exclusions_are_distinct() {
        let mut f = Format {
            digits: 2,
            leading_zero: 1,
            grouping: 3,
            decimal: &[46],
            thousands: &[44],
            negative_order: 1,
        };
        for value in ["+1", " 1", "1 ", "1,2", "1e2", "1.2.3", "--1", "１２"] {
            assert_eq!(
                format(&value.encode_utf16().collect::<Vec<_>>(), &f),
                Err(Error::Invalid)
            );
        }
        f.digits = 10;
        assert_eq!(format(&[49], &f), Err(Error::Invalid));
        f.digits = 2;
        f.grouping = 123;
        assert!(matches!(format(&[49], &f), Err(Error::Unsupported(_))));
        f.grouping = 3;
        assert!(matches!(
            format(&vec![49; 4097], &f),
            Err(Error::Unsupported(_))
        ));
    }
}
