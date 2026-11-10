//! Bounded software trig for reached game code. Directed Quad intervals certify
//! mathematical 64-significand rounding, not Intel's proprietary approximation.
use crate::fpu_atan::{Interval as I, pi_four};
use rustc_apfloat::{
    Float, FloatConvert, Round,
    ieee::{Quad, X87DoubleExtended as X},
};
fn extend(sum: I, term: I) -> I {
    let bound = if term.lo.abs() > term.hi.abs() {
        term.lo.abs()
    } else {
        term.hi.abs()
    };
    sum.add(I {
        lo: -bound,
        hi: bound,
    })
}
fn reduced(x: I) -> (I, I) {
    let square = x.mul(x);
    let mut sin = x;
    let mut cos = I::integer(1);
    let mut st = x;
    let mut ct = cos;
    for n in 1..=48u128 {
        st = st.mul(square).div(I::integer(2 * n * (2 * n + 1)));
        ct = ct.mul(square).div(I::integer((2 * n - 1) * (2 * n)));
        if n & 1 != 0 {
            sin = sin.sub(st);
            cos = cos.sub(ct);
        } else {
            sin = sin.add(st);
            cos = cos.add(ct);
        }
    }
    st = st.mul(square).div(I::integer(98 * 99));
    ct = ct.mul(square).div(I::integer(97 * 98));
    (extend(sin, st), extend(cos, ct))
}
fn rounded(value: I) -> Result<(X, bool), &'static str> {
    let a: X = value.lo.convert(&mut false).value;
    let b: X = value.hi.convert(&mut false).value;
    if a.to_bits() != b.to_bits() {
        return Err("trig interval needs more precision for x87 rounding");
    }
    let q: Quad = a.abs().convert(&mut false).value;
    let (low, high) = if value.hi <= Quad::ZERO {
        (-value.hi, -value.lo)
    } else if value.lo >= Quad::ZERO {
        (value.lo, value.hi)
    } else {
        return Err("trig interval cannot certify sign");
    };
    let up = if q >= high {
        true
    } else if q <= low {
        false
    } else {
        return Err("trig interval cannot certify rounding direction");
    };
    Ok((a, up))
}
type RoundedPair = ((X, bool), (X, bool));
pub(crate) fn sin_cos(value: X) -> Result<RoundedPair, &'static str> {
    if value.is_zero() {
        return Ok(((value, false), (X::from_u128(1).value, false)));
    }
    if !value.is_finite() || value.is_denormal() {
        return Err("nonfinite/subnormal x87 trig profile not implemented");
    }
    let q: Quad = value.abs().convert(&mut false).value;
    if q > Quad::from_u128(1 << 20).value {
        return Err("x87 trig outside the certified argument domain");
    }
    let x = I { lo: q, hi: q };
    let half = pi_four().mul(I::integer(2));
    let ratio = x.div(half);
    let a = ratio
        .lo
        .to_i128_r(32, Round::NearestTiesToEven, &mut false)
        .value;
    let b = ratio
        .hi
        .to_i128_r(32, Round::NearestTiesToEven, &mut false)
        .value;
    if a != b {
        return Err("trig argument reduction needs more precision");
    }
    let r = x.sub(half.mul(I::integer(a as u128)));
    if r.lo.abs() > Quad::from_u128(1).value || r.hi.abs() > Quad::from_u128(1).value {
        return Err("trig reduced argument outside series domain");
    }
    let (s, c) = reduced(r);
    let (mut s, c) = match a & 3 {
        0 => (s, c),
        1 => (c, s.negate()),
        2 => (s.negate(), c.negate()),
        _ => (c.negate(), s),
    };
    let mut c = c;
    let one = Quad::from_u128(1).value;
    for interval in [&mut s, &mut c] {
        if interval.lo < -one {
            interval.lo = -one;
        }
        if interval.hi > one {
            interval.hi = one;
        }
    }
    // Strict analytic bounds also certify the direction when the correction
    // is much smaller than a Quad ulp, e.g. sin(tiny) rounds back to tiny.
    if q <= one && s.hi > q {
        s.hi = q;
    }
    if value.is_negative() {
        s = s.negate();
    }
    Ok((rounded(s)?, rounded(c)?))
}
