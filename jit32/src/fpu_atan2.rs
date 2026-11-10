//! Bounded mathematical FPATAN profile, independent of host floating point.
//! Outward Quad intervals certify nearest 64-significand rounding. This does
//! not reproduce a particular CPU's internal polynomial or detailed C1/PE behavior.
use crate::fpu_atan::{Interval as I, pi_four, series};
use rustc_apfloat::{
    Float, FloatConvert,
    ieee::{Quad, X87DoubleExtended as X},
};
pub(crate) struct Angle {
    pub value: X,
    pub rounded_up: bool,
    pub inexact: bool,
}
fn normal_or_zero(bits: u128) -> bool {
    let exp = (bits >> 64) & 0x7fff;
    let sig = bits as u64;
    bits >> 80 == 0 && ((exp == 0 && sig == 0) || (exp != 0 && exp != 0x7fff && sig >> 63 == 1))
}
fn interval(value: X) -> I {
    let q: Quad = value.convert(&mut false).value;
    I { lo: q, hi: q }
}
// If atan(r) is indistinguishable from r at Quad precision, compare the
// exact 128-bit product q*x with y before rounding their small difference.
// r^3/3-r^5/5 < r-atan(r) < r^3/3 certifies the remaining direction.
fn ratio_direction(y_bits: u128, x_bits: u128, ratio: I, q: X) -> Option<bool> {
    if ratio.hi > I::integer(1).hi.scalbn(-1) {
        return None;
    }
    let exp = |b: u128| ((b >> 64 & 0x7fff) as i32) - 16383;
    let q_bits = q.to_bits();
    let shift = exp(y_bits) - exp(q_bits) - exp(x_bits) + 63;
    if !(0..=64).contains(&shift) {
        return None;
    }
    let y_sig = y_bits as u64 as u128;
    if y_sig > (u128::MAX >> shift) {
        return None;
    }
    let scaled = y_sig << shift;
    let product = (q_bits as u64 as u128) * (x_bits as u64 as u128);
    if product >= scaled {
        return Some(true);
    }
    let delta = scaled - product;
    if delta.leading_zeros() < 15 {
        return None;
    } // I::integer is exact to 113 bits.
    let d = I::integer(delta).div(I::integer(x_bits as u64 as u128));
    let d = I {
        lo: d.lo.scalbn(exp(q_bits) - 63),
        hi: d.hi.scalbn(exp(q_bits) - 63),
    };
    let cube = ratio.mul(ratio).mul(ratio);
    let upper = cube.div(I::integer(3));
    let lower = upper.sub(cube.mul(ratio).mul(ratio).div(I::integer(5)));
    if d.lo > upper.hi {
        Some(false)
    } else if d.hi < lower.lo {
        Some(true)
    } else {
        None
    }
}
fn rounded(value: I, direct_ratio: Option<(u128, u128, I)>) -> Result<Angle, &'static str> {
    let low: X = value.lo.convert(&mut false).value;
    let high: X = value.hi.convert(&mut false).value;
    if low.to_bits() != high.to_bits() {
        return Err("FPATAN interval cannot certify x87 rounding");
    }
    let q: Quad = low.abs().convert(&mut false).value;
    let (lo, hi) = if value.hi < Quad::ZERO {
        (-value.hi, -value.lo)
    } else if value.lo > Quad::ZERO {
        (value.lo, value.hi)
    } else {
        return Err("FPATAN interval cannot certify sign");
    };
    let rounded_up = if q >= hi {
        true
    } else if q <= lo {
        false
    } else if let Some((y, x, r)) = direct_ratio {
        ratio_direction(y, x, r, low.abs())
            .ok_or("FPATAN interval cannot certify rounding direction")?
    } else {
        return Err("FPATAN interval cannot certify rounding direction");
    };
    Ok(Angle {
        value: low,
        rounded_up,
        inexact: true,
    })
}
pub(crate) fn atan2(y_bits: u128, x_bits: u128) -> Result<Angle, &'static str> {
    if !normal_or_zero(x_bits) || !normal_or_zero(y_bits) {
        return Err("FPATAN nonfinite/subnormal/noncanonical operand is outside the profile");
    }
    let (x, y) = (X::from_bits(x_bits), X::from_bits(y_bits));
    if y.is_zero() && !x.is_negative() {
        return Ok(Angle {
            value: y,
            rounded_up: false,
            inexact: false,
        });
    }
    let pi = pi_four().mul(I::integer(4));
    let half_pi = pi_four().mul(I::integer(2));
    let mut direct_ratio = None;
    let mut angle = if y.is_zero() {
        pi
    } else if x.is_zero() {
        half_pi
    } else {
        let gap = ((x_bits >> 64 & 0x7fff) as i32 - (y_bits >> 64 & 0x7fff) as i32).abs();
        if gap > 1024 {
            return Err("FPATAN exponent separation exceeds the certified profile");
        }
        let swapped = y.abs() > x.abs();
        let (small, big) = if swapped {
            (x.abs(), y.abs())
        } else {
            (y.abs(), x.abs())
        };
        let ratio = interval(small).div(interval(big));
        let one = I::integer(1);
        let mut reduced = if ratio.hi > one.hi.scalbn(-1) {
            pi_four().add(series(ratio.sub(one).div(ratio.add(one))))
        } else {
            series(ratio)
        };
        if swapped {
            reduced = half_pi.sub(reduced);
        }
        if x.is_negative() {
            reduced = pi.sub(reduced);
        }
        if !swapped && !x.is_negative() {
            direct_ratio = Some((y_bits, x_bits, ratio));
        }
        reduced
    };
    if y.is_negative() {
        angle = angle.negate();
    }
    rounded(angle, direct_ratio)
}
