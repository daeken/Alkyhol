//! Directed-rounding intervals for the CRT's bounded binary64 atan result.
//! Machin's identity supplies pi; the alternating-series tail is enclosed.
//! An ambiguous final rounding fails explicitly instead of returning a guess.
use rustc_apfloat::{
    Float, FloatConvert, Round,
    ieee::{Double, Quad, X87DoubleExtended as X},
};
#[derive(Clone, Copy)]
pub(crate) struct Interval {
    pub lo: Quad,
    pub hi: Quad,
}
impl Interval {
    pub(crate) fn integer(n: u128) -> Self {
        let q = Quad::from_u128(n).value;
        Self { lo: q, hi: q }
    }
    pub(crate) fn add(self, b: Self) -> Self {
        Self {
            lo: self.lo.add_r(b.lo, Round::TowardNegative).value,
            hi: self.hi.add_r(b.hi, Round::TowardPositive).value,
        }
    }
    pub(crate) fn sub(self, b: Self) -> Self {
        Self {
            lo: self.lo.sub_r(b.hi, Round::TowardNegative).value,
            hi: self.hi.sub_r(b.lo, Round::TowardPositive).value,
        }
    }
    fn combine(self, b: Self, divide: bool) -> Self {
        let mut low = Quad::INFINITY;
        let mut high = -Quad::INFINITY;
        for a in [self.lo, self.hi] {
            for b in [b.lo, b.hi] {
                let l = if divide {
                    a.div_r(b, Round::TowardNegative).value
                } else {
                    a.mul_r(b, Round::TowardNegative).value
                };
                let h = if divide {
                    a.div_r(b, Round::TowardPositive).value
                } else {
                    a.mul_r(b, Round::TowardPositive).value
                };
                if l < low {
                    low = l;
                }
                if h > high {
                    high = h;
                }
            }
        }
        Self { lo: low, hi: high }
    }
    pub(crate) fn mul(self, b: Self) -> Self {
        self.combine(b, false)
    }
    pub(crate) fn div(self, b: Self) -> Self {
        assert!(b.lo > Quad::ZERO || b.hi < Quad::ZERO);
        self.combine(b, true)
    }
    pub(crate) fn negate(self) -> Self {
        Self {
            lo: -self.hi,
            hi: -self.lo,
        }
    }
}
fn series(x: Interval) -> Interval {
    let square = x.mul(x);
    let mut power = x;
    let mut sum = Interval::integer(0);
    // All callers establish |x|<=1/2. The alternating remainder is bounded
    // by the first omitted term; outward operations retain their own error.
    for n in 0..128u128 {
        let term = power.div(Interval::integer(2 * n + 1));
        sum = if n & 1 == 0 {
            sum.add(term)
        } else {
            sum.sub(term)
        };
        power = power.mul(square);
    }
    let tail = power.div(Interval::integer(257));
    let bound = if tail.lo.abs() > tail.hi.abs() {
        tail.lo.abs()
    } else {
        tail.hi.abs()
    };
    sum.add(Interval {
        lo: -bound,
        hi: bound,
    })
}
pub(crate) fn pi_four() -> Interval {
    static VALUE: std::sync::OnceLock<Interval> = std::sync::OnceLock::new();
    *VALUE.get_or_init(|| {
        series(Interval::integer(1).div(Interval::integer(5)))
            .mul(Interval::integer(4))
            .sub(series(Interval::integer(1).div(Interval::integer(239))))
    })
}
/// Certified binary64 rounding within the finite gate's operand domain.
pub fn atan(value: X) -> Result<(Double, bool), &'static str> {
    if value.is_zero() {
        return Ok((
            Double::from_bits(if value.is_negative() { 1 << 63 } else { 0 }),
            false,
        ));
    }
    let positive: Quad = value.abs().convert(&mut false).value;
    let mut x = Interval {
        lo: positive,
        hi: positive,
    };
    let one = Interval::integer(1);
    let inverted = positive > one.hi;
    if inverted {
        x = one.div(x);
    }
    let half = one.div(Interval::integer(2));
    let mut result = if x.hi > half.hi {
        pi_four().add(series(x.sub(one).div(x.add(one))))
    } else {
        series(x)
    };
    if inverted {
        result = pi_four().mul(Interval::integer(2)).sub(result);
    }
    let a: Double = result.lo.convert(&mut false).value;
    let b: Double = result.hi.convert(&mut false).value;
    if a.to_bits() != b.to_bits() {
        return Err("atan interval needs more precision to certify binary64 rounding");
    }
    let rounded: Quad = a.convert(&mut false).value;
    let up = if rounded > result.hi || rounded == positive {
        true
    } else if rounded < result.lo {
        false
    } else {
        return Err("atan interval cannot certify rounding direction");
    };
    if value.is_negative() {
        result = result.negate();
        let a: Double = result.lo.convert(&mut false).value;
        let b: Double = result.hi.convert(&mut false).value;
        if a.to_bits() != b.to_bits() {
            return Err("negative atan interval ambiguous");
        }
        return Ok((a, up));
    }
    Ok((a, up))
}
