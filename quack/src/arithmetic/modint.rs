use std::fmt;
use std::cmp::PartialEq;
use std::ops::{Add, Sub, Mul, AddAssign, SubAssign, MulAssign, Neg};
use serde::{Serialize, Deserialize};

/// The largest 32-bit prime.
pub const MODULUS: u32 = 4_294_967_291;
/// The largest 32-bit prime as an unsigned 64-bit integer.
pub const MODULUS_U64: u64 = 4_294_967_291;
pub const R_INV: u32 = 3435973833;
pub const R_LOG2: u32 = 32;
pub const NEG_MODULUS_INV: u32 = 3435973837;

// from wiki https://en.wikipedia.org/wiki/Montgomery_modular_multiplication
fn montgomery_redc(x: u64) -> u32 {
    // Overflow here is OK because we're modding by a small power of two
    let m: u32 = (((x as u32) as u64) * (NEG_MODULUS_INV as u64)) as u32;
    let extra_bit = x.overflowing_add((m as u64) * (MODULUS as u64)).1;
    let sum: u64 = x.overflowing_add((m as u64) * (MODULUS as u64)).0;
    let t: u32 = (sum >> (R_LOG2 as u64)) as u32;
    if extra_bit {
        return t.overflowing_sub(MODULUS).0;
    }
    if t < MODULUS {
        return t;
    }
    return t - MODULUS;
}

fn montgomery_multiply(x: u32, y: u32) -> u32 {
    return montgomery_redc((x as u64) * (y as u64));
}

/// 32-bit modular integer.
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModularInteger {
    value: u32,
}

impl ModularInteger {
    pub fn zero() -> Self {
        Self::default()
    }

    /// Creates a new integer modulo the largest 32-bit prime.
    pub fn new(n: u32) -> Self {
        if n >= MODULUS {
            Self { value: n - MODULUS }
        } else {
            Self { value: n }
        }
    }


    pub fn new_do_conversion(n: u32) -> Self {
        let R_mod_MODULUS: u32 = (((1 as u64) << (32 as u64)) % (MODULUS as u64)) as u32;
        return ModularInteger::new((((R_mod_MODULUS as u64) * (n as u64)) % (MODULUS as u64)) as u32);
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn modulus(&self) -> u32 {
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

impl PartialEq<u32> for ModularInteger {
    fn eq(&self, other: &u32) -> bool {
        self.value == *other
    }
}

impl PartialEq<ModularInteger> for u32 {
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
        let sum: u64 = (self.value as u64) + (rhs.value as u64);
        self.value = if sum >= MODULUS_U64 {
            (sum - MODULUS_U64) as u32
        } else {
            sum as u32
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
        let neg_rhs: u64 = MODULUS_U64 - (rhs.value as u64);
        let diff: u64 = (self.value as u64) + neg_rhs;
        self.value = if diff >= MODULUS_U64 {
            (diff - MODULUS_U64) as u32
        } else {
            diff as u32
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
        self.value = montgomery_multiply(self.value, rhs.value);
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
    pub fn pow(self, power: u32) -> Self {
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
        assert_eq!(0, ModularInteger::new(4_294_967_291));
        assert_eq!(1, ModularInteger::new(4_294_967_292));
    }

    #[test]
    fn test_neg() {
        assert_eq!(0, -ModularInteger::zero());
        assert_eq!(4_294_967_290, -ModularInteger::new(1));
        assert_eq!(1, -ModularInteger::new(4_294_967_290));
    }

    #[test]
    fn test_add() {
        let mut x = ModularInteger::new(0);
        let y = ModularInteger::new(1);
        let z = ModularInteger::new(4_294_967_290);
        x += y;
        assert_eq!(x, 1);
        assert_eq!(y, 1);
        x += z;
        assert_eq!(x, 0);
        assert_eq!(z, 4_294_967_290);
        x += y;
        x += y;
        x += z;
        assert_eq!(x, 1);
        assert_eq!(z + y, 0);
        assert_eq!(z + z, 4_294_967_289);
    }

    #[test]
    fn test_sub() {
        let mut x = ModularInteger::new(0);
        let y = ModularInteger::new(1);
        let z = ModularInteger::new(4_294_967_290);
        x -= y;
        assert_eq!(x, 4_294_967_290);
        assert_eq!(y, 1);
        x -= z;
        assert_eq!(x, 0);
        assert_eq!(z, 4_294_967_290);
        x -= y;
        x -= y;
        x -= z;
        assert_eq!(x, 4_294_967_290);
        assert_eq!(z - y, 4_294_967_289);
        assert_eq!(z - z, 0);
    }

    #[test]
    fn test_mul() {
        let mut x = ModularInteger::new(1_000);
        let mut y = ModularInteger::new(4_294_968);
        // assert_eq!(x * x, 1_000_000);
        // assert_eq!(x * y, 709);
        // assert_eq!(y * y, 4_160_573_470);
        // x *= y;
        // assert_eq!(x, 709);
        // y *= y;
        // assert_eq!(y, 4_160_573_470);
    }

    #[test]
    fn test_pow() {
        let x = ModularInteger::new(1_000);
        assert_eq!(x.pow(0), 1);
        assert_eq!(x.pow(1), 1_000);
        // assert_eq!(x.pow(2), 1_000_000);
        // assert_eq!(x.pow(8), 740_208_280);
        // assert_eq!(x.pow(9), 1_473_905_948);
        // assert_eq!(x.pow(10), 732_167_187);
        // assert_eq!(x.pow(4_294_967_289), 811_748_818);
        // assert_eq!(x.pow(4_294_967_290), 1);
    }

    #[test]
    fn test_inv() {
        let x = ModularInteger::new(2);
        let y = ModularInteger::new(1_000);
        let z = ModularInteger::new(4_294_967_289);
        let one = ModularInteger::new_do_conversion(1);
        assert_eq!(x * x.inv(), one);
        assert_eq!(y * y.inv(), one);
        assert_eq!(z * z.inv(), one);
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
