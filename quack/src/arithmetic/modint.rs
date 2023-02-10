use std::fmt;
use std::cmp::PartialEq;
use std::ops::{Add, Sub, Mul, AddAssign, SubAssign, MulAssign, Neg};
use serde::{Serialize, Deserialize};

/// A 63-bit prime.
pub const MODULUS: u64 = 9223372036854775783;

/// 64-bit modular integer.
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModularInteger {
    value: u64,
}

impl ModularInteger {
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

impl fmt::Display for ModularInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Debug for ModularInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ModularInteger")
         .field("value", &self.value)
         .field("modulus", &MODULUS)
         .finish()
    }
}

impl PartialEq<u64> for ModularInteger {
    fn eq(&self, other: &u64) -> bool {
        self.value == *other
    }
}

impl PartialEq<ModularInteger> for u64 {
    fn eq(&self, other: &ModularInteger) -> bool {
        self == &other.value
    }
}

impl Neg for ModularInteger {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self.value == 0 {
            self
        } else {
            Self { value: MODULUS - self.value }
        }
    }
}

impl AddAssign for ModularInteger {
    fn add_assign(&mut self, rhs: Self) {
        let sum: u64 = self.value + rhs.value;
        self.value = if sum >= MODULUS {
            (sum - MODULUS) as u64
        } else {
            sum as u64
        };
    }
}

impl Add for ModularInteger {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result += rhs;
        result
    }
}

impl SubAssign for ModularInteger {
    fn sub_assign(&mut self, rhs: Self) {
        let neg_rhs: u64 = MODULUS - rhs.value;
        let diff: u64 = self.value + neg_rhs;
        self.value = if diff >= MODULUS {
            diff - MODULUS
        } else {
            diff
        };
    }
}

impl Sub for ModularInteger {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result -= rhs;
        result
    }
}

impl MulAssign for ModularInteger {
    fn mul_assign(&mut self, rhs: Self) {
        let prod: u128 = (self.value as u128) * (rhs.value as u128);
        self.value = (prod % (MODULUS as u128)) as u64;
    }
}

impl Mul for ModularInteger {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result *= rhs;
        result
    }
}

impl ModularInteger {
    pub fn pow(self, power: u64) -> Self {
        if power == 0 {
            ModularInteger::new(1)
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
        self.pow(MODULUS-2)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_constructor() {
        assert_eq!(0, ModularInteger::new(0));
        assert_eq!(1, ModularInteger::new(1));
        assert_eq!(4_294_967_290, ModularInteger::new(4_294_967_290));
    }

    #[test]
    fn test_field_getters() {
        let x = ModularInteger::new(12345);
        assert_eq!(x.value(), 12345);
        assert_eq!(x.modulus(), MODULUS);
    }

    #[test]
    fn test_zero_constructor() {
        let zero = ModularInteger::zero();
        assert_eq!(0, zero);
        assert!(zero.is_zero());
    }

    #[test]
    fn test_constructor_overflow() {
    }

    #[test]
    fn test_neg() {
        assert_eq!(0, -ModularInteger::zero());
        assert_eq!(MODULUS - 1, -ModularInteger::new(1));
        assert_eq!(1, -ModularInteger::new(MODULUS - 1));
    }

    #[test]
    fn test_add() {
        let mut x = ModularInteger::new(0);
        let y = ModularInteger::new(1);
    }

    #[test]
    fn test_sub() {
    }

    #[test]
    fn test_mul() {
    }

    #[test]
    fn test_pow() {
        let x = ModularInteger::new(1_000);
        assert_eq!(x.pow(0), 1);
    }

    #[test]
    fn test_inv() {
        let x = ModularInteger::new(2);
        let y = ModularInteger::new(1_000);
        let z = ModularInteger::new(4_294_967_289);
        assert_eq!(x * x.inv(), 1);
        assert_eq!(y * y.inv(), 1);
        assert_eq!(z * z.inv(), 1);
    }

    #[test]
    fn test_fmt() {
        let x = ModularInteger::new(12345);
        let display = format!("{}", x);
        let debug = format!("{:?}", x);
        assert_eq!(display, "12345".to_string());
        assert!(debug.contains("value: 12345"));
        assert!(debug.contains(&format!("modulus: {}", MODULUS)));
    }
}
