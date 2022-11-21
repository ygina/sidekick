use std::fmt;
use std::ops::{Add, Sub, Mul, AddAssign, SubAssign, MulAssign, Neg};

/// The largest 32-bit prime.
pub const MODULUS: u32 = 4_294_967_291;
/// The largest 32-bit prime as an unsigned 64-bit integer.
pub const MODULUS_U64: u64 = 4_294_967_291;

/// 32-bit modular integer.
#[derive(Copy, Clone, Default)]
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

    pub fn value(&self) -> u32 {
        self.value
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
        let prod: u64 = (self.value as u64) * (rhs.value as u64);
        self.value = (prod % MODULUS_U64) as u32;
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
        assert_eq!(0, ModularInteger::new(0).value());
        assert_eq!(1, ModularInteger::new(1).value());
        assert_eq!(4_294_967_290, ModularInteger::new(4_294_967_290).value());
    }

    #[test]
    fn test_zero_constructor() {
        let zero = ModularInteger::zero();
        assert_eq!(0, zero.value());
        assert!(zero.is_zero());
    }

    #[test]
    fn test_constructor_overflow() {
        assert_eq!(0, ModularInteger::new(4_294_967_291).value());
        assert_eq!(1, ModularInteger::new(4_294_967_292).value());
    }

    #[test]
    fn test_neg() {
        assert_eq!(0, (-ModularInteger::zero()).value());
        assert_eq!(4_294_967_290, (-ModularInteger::new(1)).value());
        assert_eq!(1, (-ModularInteger::new(4_294_967_290)).value());
    }

    #[test]
    fn test_add() {
        let mut x = ModularInteger::new(0);
        let y = ModularInteger::new(1);
        let z = ModularInteger::new(4_294_967_290);
        x += y;
        assert_eq!(x.value(), 1);
        assert_eq!(y.value(), 1);
        x += z;
        assert_eq!(x.value(), 0);
        assert_eq!(z.value(), 4_294_967_290);
        x += y;
        x += y;
        x += z;
        assert_eq!(x.value(), 1);
        assert_eq!((z + y).value(), 0);
        assert_eq!((z + z).value(), 4_294_967_289);
    }

    #[test]
    fn test_sub() {
        let mut x = ModularInteger::new(0);
        let y = ModularInteger::new(1);
        let z = ModularInteger::new(4_294_967_290);
        x -= y;
        assert_eq!(x.value(), 4_294_967_290);
        assert_eq!(y.value(), 1);
        x -= z;
        assert_eq!(x.value(), 0);
        assert_eq!(z.value(), 4_294_967_290);
        x -= y;
        x -= y;
        x -= z;
        assert_eq!(x.value(), 4_294_967_290);
        assert_eq!((z - y).value(), 4_294_967_289);
        assert_eq!((z - z).value(), 0);
    }

    #[test]
    fn test_mul() {
        let mut x = ModularInteger::new(1_000);
        let mut y = ModularInteger::new(4_294_968);
        assert_eq!((x * x).value(), 1_000_000);
        assert_eq!((x * y).value(), 709);
        assert_eq!((y * y).value(), 4_160_573_470);
        x *= y;
        assert_eq!(x.value(), 709);
        y *= y;
        assert_eq!(y.value(), 4_160_573_470);
    }

    #[test]
    fn test_pow() {
        let x = ModularInteger::new(1_000);
        assert_eq!(x.pow(0).value(), 1);
        assert_eq!(x.pow(1).value(), 1_000);
        assert_eq!(x.pow(2).value(), 1_000_000);
        assert_eq!(x.pow(8).value(), 740_208_280);
        assert_eq!(x.pow(9).value(), 1_473_905_948);
        assert_eq!(x.pow(10).value(), 732_167_187);
        assert_eq!(x.pow(4_294_967_289).value(), 811_748_818);
        assert_eq!(x.pow(4_294_967_290).value(), 1);
    }

    #[test]
    fn test_inv() {
        let x = ModularInteger::new(2);
        let y = ModularInteger::new(1_000);
        let z = ModularInteger::new(4_294_967_289);
        assert_eq!((x * x.inv()).value(), 1);
        assert_eq!((y * y.inv()).value(), 1);
        assert_eq!((z * z.inv()).value(), 1);
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
