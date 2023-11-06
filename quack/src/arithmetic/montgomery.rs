use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A 63-bit prime.
pub const MODULUS: u64 = 18446744073709551557;
pub const R_LOG2: u64 = 64;
pub const NEG_MODULUS_INV: u64 = 14694863923124558067;

// from wiki https://en.wikipedia.org/wiki/Montgomery_modular_multiplication
fn montgomery_redc(x: u128) -> u64 {
    // Overflow here is OK because we're modding by a small power of two
    let m: u64 = (((x as u64) as u128) * (NEG_MODULUS_INV as u128)) as u64;
    let extra_bit = x.overflowing_add((m as u128) * (MODULUS as u128)).1;
    let sum: u128 = x.overflowing_add((m as u128) * (MODULUS as u128)).0;
    let t: u64 = (sum >> (R_LOG2 as u128)) as u64;
    if extra_bit {
        return t.overflowing_sub(MODULUS).0;
    }
    if t < MODULUS {
        t
    } else {
        t - MODULUS
    }
}

fn montgomery_multiply(x: u64, y: u64) -> u64 {
    montgomery_redc((x as u128) * (y as u128))
}

/// 64-bit modular integer.
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MontgomeryInteger {
    value: u64,
}

impl MontgomeryInteger {
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn new(n: u64) -> Self {
        if n >= MODULUS {
            Self { value: n - MODULUS }
        } else {
            Self { value: n }
        }
    }

    pub fn new_do_conversion(n: u64) -> Self {
        let r_mod_modulus: u64 = (((1_u128) << (64_u128)) % (MODULUS as u128)) as u64;
        MontgomeryInteger::new((((r_mod_modulus as u128) * (n as u128)) % (MODULUS as u128)) as u64)
    }

    pub fn value(&self) -> u64 {
        self.value
    }

    pub fn modulus(&self) -> u64 {
        MODULUS
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0
    }
}

impl fmt::Display for MontgomeryInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Debug for MontgomeryInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MontgomeryInteger")
            .field("value", &self.value)
            .field("modulus", &MODULUS)
            .finish()
    }
}

impl PartialEq<u64> for MontgomeryInteger {
    fn eq(&self, other: &u64) -> bool {
        self.value == *other
    }
}

impl PartialEq<MontgomeryInteger> for u64 {
    fn eq(&self, other: &MontgomeryInteger) -> bool {
        self == &other.value
    }
}

impl Neg for MontgomeryInteger {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self.value == 0 {
            self
        } else {
            Self {
                value: MODULUS - self.value,
            }
        }
    }
}

impl AddAssign for MontgomeryInteger {
    fn add_assign(&mut self, rhs: Self) {
        // NOTE: we didn't need to consider overflow here for 63-bit, but we do for both 32 and
        // 64-bit.
        let sum: u128 = (self.value as u128) + (rhs.value as u128);
        self.value = if sum >= (MODULUS as u128) {
            (sum - (MODULUS as u128)) as u64
        } else {
            sum as u64
        };
    }
}

impl Add for MontgomeryInteger {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result += rhs;
        result
    }
}

impl SubAssign for MontgomeryInteger {
    fn sub_assign(&mut self, rhs: Self) {
        let neg_rhs: u64 = MODULUS - rhs.value;
        let diff: u128 = (self.value as u128) + (neg_rhs as u128);
        self.value = if diff >= (MODULUS as u128) {
            (diff - (MODULUS as u128)) as u64
        } else {
            diff as u64
        };
    }
}

impl Sub for MontgomeryInteger {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result -= rhs;
        result
    }
}

impl MulAssign for MontgomeryInteger {
    fn mul_assign(&mut self, rhs: Self) {
        self.value = montgomery_multiply(self.value, rhs.value);
    }
}

impl Mul for MontgomeryInteger {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result *= rhs;
        result
    }
}

impl MontgomeryInteger {
    pub fn pow(self, power: u64) -> Self {
        if power == 0 {
            MontgomeryInteger::new(1)
        } else if power == 1 {
            self
        } else {
            let mut result = self.pow(power >> 1);
            result *= result;
            if power & 1 == 1 {
                result *= self;
            }
            result
        }
    }

    /// n * inv(n) = n^(MODULUS-1) = 1 (mod MODULUS)
    pub fn inv(self) -> Self {
        self.pow(MODULUS - 2)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_constructor() {
        assert_eq!(0, MontgomeryInteger::new(0));
        assert_eq!(1, MontgomeryInteger::new(1));
        assert_eq!(4_294_967_290, MontgomeryInteger::new(4_294_967_290));
        assert_eq!(0, MontgomeryInteger::new(0));
        assert_ne!(1, MontgomeryInteger::new_do_conversion(1));
        assert_ne!(
            4_294_967_290,
            MontgomeryInteger::new_do_conversion(4_294_967_290)
        );
    }
}
