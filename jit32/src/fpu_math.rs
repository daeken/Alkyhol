//! x87 arithmetic with explicit precision control. No ARM f64 approximation.
use crate::fpu::State;
use rustc_apfloat::{
    Float, FloatConvert, Round, Status, StatusAnd,
    ieee::{IeeeFloat, Semantics, X87DoubleExtended as X},
};
struct P24;
impl Semantics for P24 {
    const BITS: usize = 39;
    const EXP_BITS: usize = 15;
    const PRECISION: usize = 24;
}
struct P53;
impl Semantics for P53 {
    const BITS: usize = 68;
    const EXP_BITS: usize = 15;
    const PRECISION: usize = 53;
}
fn calculate<F: Float>(a: F, b: F, operation: u64, round: Round) -> StatusAnd<F> {
    match operation {
        0 => a.add_r(b, round),
        1 => a.mul_r(b, round),
        2 => a.sub_r(b, round),
        3 => a.div_r(b, round),
        _ => unreachable!(),
    }
}
fn reduce<S: Semantics>(value: X, round: Round) -> StatusAnd<X> {
    let result: StatusAnd<IeeeFloat<S>> = value.convert_r(round, &mut false);
    let back: X = result.value.convert(&mut false).value;
    result.status.and(back)
}
// Wide exponents keep the directed 64-bit bounds finite even when a final
// x87 result overflows or underflows. No host floating-point arithmetic.
struct Wide64;
impl Semantics for Wide64 {
    const BITS: usize = 81;
    const EXP_BITS: usize = 17;
    const PRECISION: usize = 64;
}
struct Wide111;
impl Semantics for Wide111 {
    const BITS: usize = 128;
    const EXP_BITS: usize = 17;
    const PRECISION: usize = 111;
}
type W64 = IeeeFloat<Wide64>;
type W111 = IeeeFloat<Wide111>;
fn reduce_wide<S: Semantics>(value: W111, round: Round) -> StatusAnd<X> {
    let reduced: StatusAnd<IeeeFloat<S>> = value.convert_r(round, &mut false);
    reduced.status.and(reduced.value.convert(&mut false).value)
}
pub fn arithmetic(
    state: &mut State,
    a: u128,
    b: u128,
    operation: u64,
) -> Result<u128, &'static str> {
    let (a, b) = (X::from_bits(a), X::from_bits(b));
    let pc = state.control >> 8 & 3;
    if pc == 1 {
        return Err("reserved x87 precision control");
    }
    state.status &= !0x200;
    // x87 chooses a quiet NaN before signaling NaNs, then the larger
    // significand (positive sign breaks ties). It quiets sNaNs and sets IE.
    // NaN handling takes priority over the denormal-operand exception.
    if a.is_nan() || b.is_nan() {
        if a.is_signaling() || b.is_signaling() {
            state.status |= 1;
        }
        let selected = [a, b]
            .into_iter()
            .filter(|v| v.is_nan())
            .max_by_key(|v| (!v.is_signaling(), v.to_bits() as u64, !v.is_negative()))
            .unwrap();
        return Ok(selected.to_bits() | (1 << 62));
    }
    let round = super::fpu_ops::round(state);
    let result = calculate(a, b, operation, round);
    if result.value.is_nan() {
        super::fpu_ops::flags(state, result.status);
        return Ok(0xffffc000000000000000); // x87 masked invalid indefinite.
    }
    if !result.status.contains(Status::DIV_BY_ZERO) && (a.is_denormal() || b.is_denormal()) {
        state.status |= 2;
    }
    if !a.is_finite() || !b.is_finite() || result.status.contains(Status::DIV_BY_ZERO) {
        super::fpu_ops::flags(state, result.status);
        return Ok(result.value.to_bits());
    }
    let (value, mut status, down) = if pc == 3 {
        let mut status = result.status;
        if result.value.abs() == X::largest() && status.contains(Status::INEXACT) {
            let wa: W64 = a.convert(&mut false).value;
            let wb: W64 = b.convert(&mut false).value;
            if calculate(wa, wb, operation, round).value.abs()
                >= W64::from_u128(1).value.scalbn(16384)
            {
                status |= Status::OVERFLOW;
            }
        }
        (
            result.value,
            status,
            calculate(a, b, operation, Round::TowardZero).value,
        )
    } else {
        let a: W64 = a.convert(&mut false).value;
        let b: W64 = b.convert(&mut false).value;
        let low = calculate(a, b, operation, Round::TowardNegative);
        let high = calculate(a, b, operation, Round::TowardPositive);
        let lo: W111 = low.value.convert(&mut false).value;
        let hi: W111 = high.value.convert(&mut false).value;
        // Every 24/53-bit rounding boundary lies on the 64-bit grid. The
        // exact midpoint of adjacent bounds is a valid representative of
        // every value strictly between them, including subnormal boundaries.
        let representative = if low.value.is_zero() && high.value.is_zero() {
            result.value.convert(&mut false).value
        } else if low.value == high.value {
            lo
        } else {
            lo.scalbn(-1)
                .add_r(hi.scalbn(-1), Round::NearestTiesToEven)
                .value
        };
        let mut reduced = match pc {
            0 => reduce_wide::<P24>(representative, round),
            _ => reduce_wide::<P53>(representative, round),
        };
        // APFloat's directed saturation reports PE without OE. x87 also
        // reports OE when the unbounded rounded exponent exceeds its range.
        if representative.abs() >= W111::from_u128(1).value.scalbn(16384) {
            reduced.status |= Status::OVERFLOW;
        }
        let truncated: X = calculate(a, b, operation, Round::TowardZero)
            .value
            .convert_r(Round::TowardZero, &mut false)
            .value;
        let down = if pc == 0 {
            reduce::<P24>(truncated, Round::TowardZero).value
        } else {
            reduce::<P53>(truncated, Round::TowardZero).value
        };
        (
            reduced.value,
            reduced.status | (low.status & Status::INEXACT),
            down,
        )
    };
    if pc == 3 && status.contains(Status::INEXACT) {
        let wa: W64 = a.convert(&mut false).value;
        let wb: W64 = b.convert(&mut false).value;
        let magnitude = calculate(wa, wb, operation, round).value.abs();
        // Detect tininess after unbounded precision rounding but before
        // denormal rounding. The latter can promote a tiny 64-bit result
        // to the smallest normal; APFloat then omits x87's UE flag.
        if !magnitude.is_zero() && magnitude < W64::from_u128(1).value.scalbn(-16382) {
            status |= Status::UNDERFLOW;
        }
    }
    if status.contains(Status::INEXACT) && value.abs() > down.abs() {
        state.status |= 0x200;
    }
    super::fpu_ops::flags(state, status);
    Ok(value.to_bits())
}

/// Exact integer-radicand square root, rounded once at the selected x87
/// precision. Even the 64-bit significand fits a 128-bit squared radicand.
pub fn sqrt(state: &mut State, bits: u128) -> Result<u128, &'static str> {
    let sign = bits >> 79 != 0;
    let exponent = ((bits >> 64) & 0x7fff) as i32;
    let mut sig = bits as u64;
    if sig == 0 && exponent == 0 {
        state.status &= !0x200;
        return Ok(bits);
    }
    if sign || exponent == 0x7fff {
        return Err("nonfinite/negative x87 sqrt profile not implemented");
    }
    let mut exponent = if exponent == 0 {
        state.status |= 2;
        -16382
    } else {
        exponent - 16383
    };
    let shift = sig.leading_zeros();
    sig <<= shift;
    exponent -= shift as i32;
    let radicand = (sig as u128) << (63 + (exponent & 1));
    let root = radicand.isqrt();
    let remainder = radicand - root * root;
    let p = match state.control >> 8 & 3 {
        0 => 24,
        2 => 53,
        3 => 64,
        _ => return Err("reserved x87 precision control"),
    };
    let drop = 64 - p;
    let mut retained = root >> drop;
    let lost = root & ((1u128 << drop) - 1);
    let inexact = lost != 0 || remainder != 0;
    let increment = match super::fpu_ops::round(state) {
        Round::NearestTiesToEven => {
            if drop == 0 {
                remainder > root
            } else {
                let half = 1u128 << (drop - 1);
                lost > half || (lost == half && (remainder != 0 || retained & 1 != 0))
            }
        }
        Round::TowardPositive => inexact,
        _ => false,
    };
    if increment {
        retained += 1;
    }
    let mut output_exp = exponent.div_euclid(2) + 16383;
    if retained == 1u128 << p {
        retained >>= 1;
        output_exp += 1;
    }
    state.status &= !0x200;
    if inexact {
        state.status |= 0x20;
        if increment {
            state.status |= 0x200;
        }
    }
    Ok(((output_exp as u128) << 64) | (retained << drop))
}

#[cfg(test)]
mod arithmetic_reference {
    use super::*;
    #[test]
    #[ignore = "run tools/check_jit32_x87_arithmetic.py for actual x86 observations"]
    fn actual_x86_arithmetic_observations() {
        let data =
            std::fs::read(std::env::var_os("ALKYHOL_X87_ARITHMETIC_REFERENCE").unwrap()).unwrap();
        assert_eq!(data.len(), 19200 * 40);
        for (i, row) in data.chunks_exact(40).enumerate() {
            fn raw(b: &[u8]) -> u128 {
                let mut x = [0; 16];
                x[..10].copy_from_slice(b);
                u128::from_le_bytes(x)
            }
            let op = u32::from_le_bytes(row[..4].try_into().unwrap());
            let cw = u32::from_le_bytes(row[4..8].try_into().unwrap());
            let a = raw(&row[8..18]);
            let b = raw(&row[18..28]);
            let expected = raw(&row[28..38]);
            let sw = u16::from_le_bytes(row[38..40].try_into().unwrap());
            let mut state = State {
                control: cw as u16,
                ..Default::default()
            };
            let result = arithmetic(&mut state, a, b, op as u64).unwrap();
            assert_eq!(
                result, expected,
                "row={i} op={op} cw={cw:x} a={a:x} b={b:x}"
            );
            assert_eq!(
                state.status & 0x23f,
                sw & 0x23f,
                "status row={i} op={op} cw={cw:x} a={a:x} b={b:x}"
            );
        }
    }
}

#[cfg(test)]
mod special_tests {
    use crate::{
        Stop,
        memory::{READ, WRITE},
        tests::cpu,
    };
    #[test]
    fn masked_division_classes_and_status_commit_through_checked_native_code() {
        for (input, divisor, expected, status) in [
            (
                0x3fff8000000000000000u128,
                0u64,
                0x7fff8000000000000000u128,
                4,
            ),
            (0, 0, 0xffffc000000000000000, 1),
            (1, 0x3ff0000000000000, 1, 2),
            (
                0x7fff8000000000000001,
                0x3ff0000000000000,
                0x7fffc000000000000001,
                1,
            ),
            (0x3fff8000000000000000, 0x7ff0000000000000, 0, 0),
        ] {
            let mut r = cpu(&[0xdc, 0x35, 0, 0, 2, 0]); // FDIV qword [0x20000].
            r.memory
                .map(0x20000, 8, READ, &divisor.to_le_bytes())
                .unwrap();
            {
                let mut f = r.memory.fpu.borrow_mut();
                f.push(input);
                f.status |= 0x600;
            }
            r.state[16] = 0xad7;
            let before = r.state;
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            let f = r.memory.fpu.borrow();
            assert_eq!(f.registers[f.top()], expected);
            assert_eq!(f.status, 0x3c00 | status);
            assert_eq!(f.control, 0x37f);
            assert_eq!(&r.state[..17], &before[..17]);
        }
        for rights in [0, READ | WRITE] {
            let mut r = cpu(&[0xdc, 0x35, 0, 0, 2, 0]);
            if rights != 0 {
                r.memory
                    .map(0x20000, 8, rights, &0u64.to_le_bytes())
                    .unwrap();
                r.memory.fpu.borrow_mut().control &= !4;
            }
            r.memory.fpu.borrow_mut().push(0x3fff8000000000000000);
            let before = r.memory.fpu.borrow().clone();
            let stop = r.run(1);
            if rights == 0 {
                assert!(matches!(stop, Stop::Fault { .. }));
            } else {
                assert!(matches!(stop, Stop::CpuFault { .. }));
            }
            let after = r.memory.fpu.borrow();
            assert_eq!(after.status, before.status);
            assert_eq!(after.tag, before.tag);
            assert_eq!(after.registers, before.registers);
        }
    }
    #[test]
    fn sticky_flags_nan_priority_and_extended_underflow_boundaries() {
        use super::*;
        let mut s = State {
            status: 0x2470,
            ..Default::default()
        };
        assert_eq!(
            arithmetic(&mut s, 0x7fff8000000000000002, 0x7fffc000000000000001, 0).unwrap(),
            0x7fffc000000000000001
        );
        assert_eq!(s.status, 0x2471);
        let mut s = State {
            control: 0xb7f,
            ..Default::default()
        };
        assert_eq!(
            arithmetic(&mut s, 0x18000000000000000, 0x3fff8000000000000001, 3).unwrap(),
            0x18000000000000000
        );
        assert_eq!(s.status, 0x230);
        let mut s = State {
            control: 0x47f,
            ..Default::default()
        };
        assert_eq!(
            arithmetic(&mut s, 0x7ffeffffffffffffffff, 0x7ffeffffffffffffffff, 0).unwrap(),
            0x7ffeffffff0000000000
        );
        assert_eq!(s.status, 0x28);
    }
}
