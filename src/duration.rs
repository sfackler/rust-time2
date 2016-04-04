use std::time::Duration;

const NANOS_PER_SEC: u64 = 1_000_000_000;
const MILLIS_PER_SEC: u64 = 1_000;
const NANOS_PER_MILLI: u64 = NANOS_PER_SEC / MILLIS_PER_SEC;

/// Extensions to the `Duration` type.
pub trait DurationExt {
    /// Returns the number of whole milliseconds contained in this `Duration`.
    ///
    /// Returns `None` if the value exceeds the capacity of a `u64`.
    fn as_millis(&self) -> Option<u64>;

    /// Multiplies this `Duration`.
    ///
    /// Like the `Mul` implementation for `Duration`, except that it takes a
    /// `u64` instead of a `u32.
    fn mul_u64(&self, rhs: u64) -> Self;

    /// Divides this `Duration`.
    ///
    /// Like the `Div` implementation for `Duration`, except that it takes a
    /// `u64` instead of a `u32`.
    fn div_u64(&self, rhs: u64) -> Self;
}

impl DurationExt for Duration {
    fn as_millis(&self) -> Option<u64> {
        self.as_secs()
            .checked_mul(MILLIS_PER_SEC)
            .and_then(|m| m.checked_add(self.subsec_nanos() as u64 / NANOS_PER_MILLI))
    }

    fn mul_u64(&self, rhs: u64) -> Duration {
        // for nanos, treat rhs as (NANOS_PER_SEC * a + b), where b < NANOS_PER_SEC
        let a = rhs / NANOS_PER_SEC;
        let b = rhs % NANOS_PER_SEC;
        let total_nanos = self.subsec_nanos() as u64 * b; // can't overflow
        let nanos = (total_nanos % NANOS_PER_SEC as u64) as u32;

        let secs = self.as_secs()
                       .checked_mul(rhs)
                       .and_then(|s| s.checked_add(total_nanos / NANOS_PER_SEC))
                       .and_then(|s| s.checked_add(self.subsec_nanos() as u64 * a))
                       .expect("overflow when multiplying duration");
        debug_assert!(nanos < NANOS_PER_SEC as u32);
        Duration::new(secs, nanos)
    }

    fn div_u64(&self, rhs: u64) -> Duration {
        let secs = self.as_secs() / rhs;
        let carry = self.as_secs() - secs * rhs;
        let extra_nanos = mul_div(carry, NANOS_PER_SEC, rhs)
                              .expect("overflow when dividing duration");
        let nanos = (self.subsec_nanos() as u64 / rhs + extra_nanos) as u32;
        debug_assert!(nanos < NANOS_PER_SEC as u32);
        Duration::new(secs, nanos)
    }
}

fn mul_div(a: u64, b: u64, c: u64) -> Option<u64> {
    let m0 = a.wrapping_mul(b);
    let m1 = mul_u64_hi(a, b);
    div_u128(m1, m0, c)
}

// hacker's delight 2nd ed 8-2
fn mul_u64_hi(u: u64, v: u64) -> u64 {
    let u0 = u & 0xFFFF_FFFF;
    let u1 = u >> 32;
    let v0 = v & 0xFFFF_FFFF;
    let v1 = v >> 32;
    let w0 = u0 * v0;
    let t = u1 * v0 + (w0 >> 32);
    let w1 = t & 0xFFFF_FFFF;
    let w2 = t >> 32;
    let w1 = u0 * v1 + w1;
    u1 * v1 + w2 + (w1 >> 32)
}

//hacker's delight 2nd ed 9-3
fn div_u128(u1: u64, u0: u64, v: u64) -> Option<u64> {
    let b = 0x1_0000_0000;

    if u1 >= v {
        return None;
    }

    let s = v.leading_zeros();
    let v = v << s;
    let vn0 = v & 0xFFFF_FFFF;
    let vn1 = v >> 32;

    let un32 = (u1 << s) | (u0.wrapping_shr(64 - s)) & (-(s as i64) >> 63) as u64;
    let un10 = u0 << s;

    let un1 = un10 >> 32;
    let un0 = un10 & 0xFFFF_FFFF;

    let mut q1 = un32 / vn1;
    let mut rhat = un32 - q1 * vn1;

    while q1 >= b || q1 * vn0 > b * rhat + un1 {
        q1 -= 1;
        rhat += vn1;
        if rhat >= b {
            break;
        }
    }

    let un21 = un32 * b + un1 - q1 * v;

    let mut q0 = un21 / vn1;
    rhat = un21 - q0 * vn1;

    while q0 >= b || q0 * vn0 > b * rhat + un0 {
        q0 -= 1;
        rhat += vn1;
        if rhat >= b {
            break;
        }
    }

    Some(q1 * b + q0)
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;

    #[test]
    fn as_millis() {
        assert_eq!(Some(1100), Duration::new(1, 100_000_999).as_millis());
        assert_eq!(None, Duration::from_secs(1 << 60).as_millis());
    }

    #[test]
    fn mul_u64() {
        assert_eq!(Duration::new(0, 1).mul_u64(2), Duration::new(0, 2));
        assert_eq!(Duration::new(1, 1).mul_u64(3), Duration::new(3, 3));
        assert_eq!(Duration::new(0, 500_000_001).mul_u64(4), Duration::new(2, 4));
        assert_eq!(Duration::new(0, 500_000_001).mul_u64(4000),
                   Duration::new(2000, 4000));
        assert_eq!(Duration::new(0, 500_000_000).mul_u64(1 << 63),
                   Duration::new(1 << 62, 0));
    }

    #[test]
    fn div_u64() {
        assert_eq!(Duration::new(0, 1).div_u64(2), Duration::new(0, 0));
        assert_eq!(Duration::new(1, 1).div_u64(3), Duration::new(0, 333_333_333));
        assert_eq!(Duration::new(99, 999_999_000).div_u64(100),
                   Duration::new(0, 999_999_990));
        assert_eq!(Duration::new(1 << 62, 0).div_u64(1 << 63),
                   Duration::new(0, 500_000_000));
    }
}
