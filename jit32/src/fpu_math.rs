//! x87 arithmetic with explicit precision control. No ARM f64 approximation.
use crate::fpu::State;
use rustc_apfloat::{
    Float, FloatConvert, Round, Status, StatusAnd,
    ieee::{IeeeFloat, Quad, Semantics, X87DoubleExtended as X},
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
fn reduce_quad<S: Semantics>(value: Quad, round: Round) -> StatusAnd<X> {
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
    if !a.is_finite() || !b.is_finite() || a.is_denormal() || b.is_denormal() {
        return Err("nonfinite/denormal x87 arithmetic profile not implemented");
    }
    let round = super::fpu_ops::round(state);
    let result = calculate(a, b, operation, round);
    if result
        .status
        .intersects(Status::INVALID_OP | Status::DIV_BY_ZERO | Status::OVERFLOW | Status::UNDERFLOW)
    {
        return Err("exceptional x87 arithmetic profile not implemented");
    }
    let pc = state.control >> 8 & 3;
    // Directed 64-bit bounds are equal or adjacent. Every 24/53-bit
    // rounding boundary (including nearest ties) lies on that 64-bit grid.
    // Thus an exact 65-bit midpoint rounds like every real value strictly
    // between inexact bounds. Quad stores that representative exactly; this
    // prevents double rounding without approximating x87 with binary64.
    let representative: Quad = if pc != 3 && result.status.contains(Status::INEXACT) {
        let low = calculate(a, b, operation, Round::TowardNegative).value;
        let high = calculate(a, b, operation, Round::TowardPositive).value;
        if !low.is_finite() || !high.is_finite() {
            return Err("x87 arithmetic overflow boundary is not implemented");
        }
        let low: Quad = low.convert(&mut false).value;
        let high: Quad = high.convert(&mut false).value;
        low.scalbn(-1)
            .add_r(high.scalbn(-1), Round::NearestTiesToEven)
            .value
    } else {
        result.value.convert(&mut false).value
    };
    let reduced = match pc {
        0 => reduce_quad::<P24>(representative, round),
        2 => reduce_quad::<P53>(representative, round),
        3 => Status::OK.and(result.value),
        _ => return Err("reserved x87 precision control"),
    };
    let status = result.status | reduced.status;
    state.status &= !0x200;
    if status.contains(Status::INEXACT) {
        let truncated = calculate(a, b, operation, Round::TowardZero).value;
        let down = match pc {
            0 => reduce::<P24>(truncated, Round::TowardZero).value,
            2 => reduce::<P53>(truncated, Round::TowardZero).value,
            _ => truncated,
        };
        if reduced.value.abs() > down.abs() {
            state.status |= 0x200;
        }
    }
    super::fpu_ops::flags(state, status);
    Ok(reduced.value.to_bits())
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
