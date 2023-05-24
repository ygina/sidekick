use std::fmt;
use std::cmp::PartialEq;
use std::ops::{Add, Sub, Mul, AddAssign, SubAssign, MulAssign, Neg};
use serde::{Serialize, Deserialize};


/// Modular integer.
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModularInteger<T> {
    pub(crate) value: T,
}

pub trait ModularArithmetic<T> {
    type ModulusBig;

    fn one() -> Self;
    fn two() -> Self;
    fn is_zero(&self) -> bool;
    fn pow(self, power: T) -> Self;
    /// The largest T-bit prime.
    fn modulus() -> T;
    /// The largest T-bit prime as an unsigned (2*T)-bit integer.
    fn modulus_big() -> Self::ModulusBig;
}

impl<T: fmt::Display> fmt::Display for ModularInteger<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T: fmt::Debug> fmt::Debug for ModularInteger<T> where
ModularInteger<T>: ModularArithmetic<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ModularInteger")
         .field("value", &self.value)
         .field("modulus", &Self::modulus())
         .finish()
    }
}

impl<T: PartialEq> PartialEq<T> for ModularInteger<T> {
    fn eq(&self, other: &T) -> bool {
        self.value == *other
    }
}

impl<T> Add for ModularInteger<T> where ModularInteger<T>: AddAssign {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result += rhs;
        result
    }
}

impl<T> Sub for ModularInteger<T> where ModularInteger<T>: SubAssign {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result -= rhs;
        result
    }
}

impl<T> Mul for ModularInteger<T> where ModularInteger<T>: MulAssign {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result *= rhs;
        result
    }
}

impl<T> Neg for ModularInteger<T> where
ModularInteger<T>: ModularArithmetic<T>, T: Sub<Output = T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if self.is_zero() {
            self
        } else {
            Self { value: Self::modulus() - self.value }
        }
    }
}

impl<T: Default> ModularInteger<T> {
    pub fn zero() -> Self {
        Self::default()
    }
}

impl<T: Copy> ModularInteger<T> {
    pub fn value(&self) -> T {
        self.value
    }
}

impl<T> ModularInteger<T> where
ModularInteger<T>: ModularArithmetic<T>, T: PartialOrd + Sub<Output = T> {
    /// Creates a new integer modulo the largest T-bit prime.
    pub fn new(n: T) -> Self {
        if n >= Self::modulus() {
            Self { value: n - Self::modulus() }
        } else {
            Self { value: n }
        }
    }
}

impl<T> ModularInteger<T> where
ModularInteger<T>: ModularArithmetic<T>, T: Sub<Output = T> {
    /// n * inv(n) = n^(MODULUS-1) = 1 (mod MODULUS)
    pub fn inv(self) -> Self {
        self.pow(Self::modulus()-Self::two().value)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl ModularArithmetic<u32> for ModularInteger<u32> {
    type ModulusBig = u64;

    fn is_zero(&self) -> bool {
        self.value == 0
    }

    fn one() -> Self {
        Self { value: 1 }
    }

    fn two() -> Self {
        Self { value: 2 }
    }

    fn modulus() -> u32 {
        4_294_967_291
    }

    fn modulus_big() -> Self::ModulusBig {
        4_294_967_291
    }

    fn pow(self, power: u32) -> Self {
        if power == 0 {
            Self::one()
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
}

impl AddAssign for ModularInteger<u32> where
ModularInteger<u32>: ModularArithmetic<u32> {
    fn add_assign(&mut self, rhs: Self) {
        let sum: u64 = (self.value as u64) + (rhs.value as u64);
        self.value = if sum >= Self::modulus_big() {
            (sum - Self::modulus_big()) as u32
        } else {
            sum as u32
        };
    }
}

impl SubAssign for ModularInteger<u32> where
ModularInteger<u32>: ModularArithmetic<u32> {
    fn sub_assign(&mut self, rhs: Self) {
        let neg_rhs: u64 = Self::modulus_big() - (rhs.value as u64);
        let diff: u64 = (self.value as u64) + neg_rhs;
        self.value = if diff >= Self::modulus_big() {
            (diff - Self::modulus_big()) as u32
        } else {
            diff as u32
        };
    }
}

impl MulAssign for ModularInteger<u32> where
ModularInteger<u32>: ModularArithmetic<u32> {
    fn mul_assign(&mut self, rhs: Self) {
        let prod: u64 = (self.value as u64) * (rhs.value as u64);
        self.value = (prod % Self::modulus_big()) as u32;
    }
}

////////////////////////////////////////////////////////////////////////////////

impl ModularArithmetic<u64> for ModularInteger<u64> {
    type ModulusBig = u128;

    fn is_zero(&self) -> bool {
        self.value == 0
    }

    fn one() -> Self {
        Self { value: 1 }
    }

    fn two() -> Self {
        Self { value: 2 }
    }

    fn modulus() -> u64 {
        9_223_372_036_854_775_783
    }

    fn modulus_big() -> Self::ModulusBig {
        9_223_372_036_854_775_783
    }

    fn pow(self, power: u64) -> Self {
        if power == 0 {
            Self::one()
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
}

impl AddAssign for ModularInteger<u64> where
ModularInteger<u64>: ModularArithmetic<u64> {
    fn add_assign(&mut self, rhs: Self) {
        let sum: u128 = (self.value as u128) + (rhs.value as u128);
        self.value = if sum >= Self::modulus_big() {
            (sum - Self::modulus_big()) as u64
        } else {
            sum as u64
        };
    }
}

impl SubAssign for ModularInteger<u64> where
ModularInteger<u64>: ModularArithmetic<u64> {
    fn sub_assign(&mut self, rhs: Self) {
        let neg_rhs: u128 = Self::modulus_big() - (rhs.value as u128);
        let diff: u128 = (self.value as u128) + neg_rhs;
        self.value = if diff >= Self::modulus_big() {
            (diff - Self::modulus_big()) as u64
        } else {
            diff as u64
        };
    }
}

impl MulAssign for ModularInteger<u64> where
ModularInteger<u64>: ModularArithmetic<u64> {
    fn mul_assign(&mut self, rhs: Self) {
        let prod: u128 = (self.value as u128) * (rhs.value as u128);
        self.value = (prod % Self::modulus_big()) as u64;
    }
}

////////////////////////////////////////////////////////////////////////////////

impl ModularArithmetic<u16> for ModularInteger<u16> {
    type ModulusBig = u32;

    fn is_zero(&self) -> bool {
        self.value == 0
    }

    fn one() -> Self {
        Self { value: 1 }
    }

    fn two() -> Self {
        Self { value: 2 }
    }

    fn modulus() -> u16 {
        65521
    }

    fn modulus_big() -> Self::ModulusBig {
        65521
    }

    fn pow(self, power: u16) -> Self {
        if power == 0 {
            Self::one()
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
}

impl AddAssign for ModularInteger<u16> where
ModularInteger<u16>: ModularArithmetic<u16> {
    fn add_assign(&mut self, rhs: Self) {
        let sum: u32 = (self.value as u32) + (rhs.value as u32);
        self.value = if sum >= Self::modulus_big() {
            (sum - Self::modulus_big()) as u16
        } else {
            sum as u16
        };
    }
}

impl SubAssign for ModularInteger<u16> where
ModularInteger<u16>: ModularArithmetic<u16> {
    fn sub_assign(&mut self, rhs: Self) {
        let neg_rhs: u32 = Self::modulus_big() - (rhs.value as u32);
        let diff: u32 = (self.value as u32) + neg_rhs;
        self.value = if diff >= Self::modulus_big() {
            (diff - Self::modulus_big()) as u16
        } else {
            diff as u16
        };
    }
}

impl MulAssign for ModularInteger<u16> where
ModularInteger<u16>: ModularArithmetic<u16> {
    fn mul_assign(&mut self, rhs: Self) {
        let prod: u32 = (self.value as u32) * (rhs.value as u32);
        self.value = (prod % Self::modulus_big()) as u16;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_constructor_u32() {
        assert_eq!(ModularInteger::<u32>::zero(), 0);
        assert_eq!(ModularInteger::<u32>::one(), 1);
        assert_eq!(ModularInteger::<u32>::two(), 2);
        assert_eq!(ModularInteger::<u32>::new(4_294_967_291), 0);
        assert_eq!(ModularInteger::<u32>::new(4_294_967_292), 1);
    }

    #[test]
    fn test_constructor_u64() {
        assert_eq!(ModularInteger::<u64>::zero(), 0);
        assert_eq!(ModularInteger::<u64>::one(), 1);
        assert_eq!(ModularInteger::<u64>::two(), 2);
        assert_eq!(ModularInteger::<u64>::new(9_223_372_036_854_775_783), 0);
        assert_eq!(ModularInteger::<u64>::new(9_223_372_036_854_775_784), 1);
    }

    #[test]
    fn test_constructor_u16() {
        assert_eq!(ModularInteger::<u16>::zero(), 0);
        assert_eq!(ModularInteger::<u16>::one(), 1);
        assert_eq!(ModularInteger::<u16>::two(), 2);
        assert_eq!(ModularInteger::<u16>::new(65521), 0);
        assert_eq!(ModularInteger::<u16>::new(65522), 1);
    }

    #[test]
    fn test_field_getters() {
        let x = ModularInteger::<u32>::new(12345);
        assert_eq!(x.value(), 12345);
        assert_eq!(ModularInteger::<u32>::modulus(), 4294967291);
        assert_eq!(ModularInteger::<u32>::modulus_big(), 4294967291);
        let x = ModularInteger::<u64>::new(12345);
        assert_eq!(x.value(), 12345);
        assert_eq!(ModularInteger::<u64>::modulus(), 9223372036854775783);
        assert_eq!(ModularInteger::<u64>::modulus_big(), 9223372036854775783);
        let x = ModularInteger::<u16>::new(12345);
        assert_eq!(x.value(), 12345);
        assert_eq!(ModularInteger::<u16>::modulus(), 65521);
        assert_eq!(ModularInteger::<u16>::modulus_big(), 65521);
    }

    #[test]
    fn test_zero_constructor() {
        assert!(ModularInteger::<u32>::zero().is_zero());
        assert!(ModularInteger::<u64>::zero().is_zero());
        assert!(ModularInteger::<u16>::zero().is_zero());
    }

    #[test]
    fn test_neg() {
        assert_eq!(-ModularInteger::<u32>::zero(), 0);
        assert_eq!(-ModularInteger::<u32>::new(1), 4294967290);
        assert_eq!(-ModularInteger::<u32>::new(4294967290), 1);
        assert_eq!(-ModularInteger::<u64>::zero(), 0);
        assert_eq!(-ModularInteger::<u64>::new(1), 9223372036854775782);
        assert_eq!(-ModularInteger::<u64>::new(9223372036854775782), 1);
        assert_eq!(-ModularInteger::<u16>::zero(), 0);
        assert_eq!(-ModularInteger::<u16>::new(1), 65520);
        assert_eq!(-ModularInteger::<u16>::new(65520), 1);
    }

    #[test]
    fn test_add_u32() {
        let mut x = ModularInteger::<u32>::new(0);
        let y = ModularInteger::<u32>::new(1);
        let z = ModularInteger::<u32>::new(ModularInteger::<u32>::modulus()-1);
        x += y;
        assert_eq!(x, 1);
        assert_eq!(y, 1);
        x += z;
        assert_eq!(x, 0);
        assert_eq!(z, ModularInteger::<u32>::modulus()-1);
        x += y;
        x += y;
        x += z;
        assert_eq!(x, 1);
        assert_eq!(z + y, 0);
        assert_eq!(z + z, ModularInteger::<u32>::modulus()-2);
    }

    #[test]
    fn test_add_u64() {
        let mut x = ModularInteger::<u64>::new(0);
        let y = ModularInteger::<u64>::new(1);
        let z = ModularInteger::<u64>::new(ModularInteger::<u64>::modulus()-1);
        x += y;
        assert_eq!(x, 1);
        assert_eq!(y, 1);
        x += z;
        assert_eq!(x, 0);
        assert_eq!(z, ModularInteger::<u64>::modulus()-1);
        x += y;
        x += y;
        x += z;
        assert_eq!(x, 1);
        assert_eq!(z + y, 0);
        assert_eq!(z + z, ModularInteger::<u64>::modulus()-2);
    }

    #[test]
    fn test_add_u16() {
        let mut x = ModularInteger::<u16>::new(0);
        let y = ModularInteger::<u16>::new(1);
        let z = ModularInteger::<u16>::new(ModularInteger::<u16>::modulus()-1);
        x += y;
        assert_eq!(x, 1);
        assert_eq!(y, 1);
        x += z;
        assert_eq!(x, 0);
        assert_eq!(z, ModularInteger::<u16>::modulus()-1);
        x += y;
        x += y;
        x += z;
        assert_eq!(x, 1);
        assert_eq!(z + y, 0);
        assert_eq!(z + z, ModularInteger::<u16>::modulus()-2);
    }

    #[test]
    fn test_sub_u32() {
        let mut x = ModularInteger::<u32>::new(0);
        let y = ModularInteger::<u32>::new(1);
        let z = ModularInteger::<u32>::new(ModularInteger::<u32>::modulus()-1);
        x -= y;
        assert_eq!(x, ModularInteger::<u32>::modulus()-1);
        assert_eq!(y, 1);
        x -= z;
        assert_eq!(x, 0);
        assert_eq!(z, ModularInteger::<u32>::modulus()-1);
        x -= y;
        x -= y;
        x -= z;
        assert_eq!(x, ModularInteger::<u32>::modulus()-1);
        assert_eq!(z - y, ModularInteger::<u32>::modulus()-2);
        assert_eq!(z - z, 0);
    }

    #[test]
    fn test_sub_u64() {
        let mut x = ModularInteger::<u64>::new(0);
        let y = ModularInteger::<u64>::new(1);
        let z = ModularInteger::<u64>::new(ModularInteger::<u64>::modulus()-1);
        x -= y;
        assert_eq!(x, ModularInteger::<u64>::modulus()-1);
        assert_eq!(y, 1);
        x -= z;
        assert_eq!(x, 0);
        assert_eq!(z, ModularInteger::<u64>::modulus()-1);
        x -= y;
        x -= y;
        x -= z;
        assert_eq!(x, ModularInteger::<u64>::modulus()-1);
        assert_eq!(z - y, ModularInteger::<u64>::modulus()-2);
        assert_eq!(z - z, 0);
    }

    #[test]
    fn test_sub_u16() {
        let mut x = ModularInteger::<u16>::new(0);
        let y = ModularInteger::<u16>::new(1);
        let z = ModularInteger::<u16>::new(ModularInteger::<u16>::modulus()-1);
        x -= y;
        assert_eq!(x, ModularInteger::<u16>::modulus()-1);
        assert_eq!(y, 1);
        x -= z;
        assert_eq!(x, 0);
        assert_eq!(z, ModularInteger::<u16>::modulus()-1);
        x -= y;
        x -= y;
        x -= z;
        assert_eq!(x, ModularInteger::<u16>::modulus()-1);
        assert_eq!(z - y, ModularInteger::<u16>::modulus()-2);
        assert_eq!(z - z, 0);
    }

    #[test]
    fn test_mul_u32() {
        let mut x = ModularInteger::<u32>::new(1_000);
        let mut y = ModularInteger::<u32>::new(4_294_968);
        assert_eq!(x * x, 1_000_000);
        assert_eq!(x * y, 709);
        assert_eq!(y * y, 4_160_573_470);
        x *= y;
        assert_eq!(x, 709);
        y *= y;
        assert_eq!(y, 4_160_573_470);
    }

    #[test]
    fn test_mul_u64() {
        let mut x = ModularInteger::<u64>::new(1_000);
        let mut y = ModularInteger::<u64>::new(9223372036854776);
        assert_eq!(x * x, 1_000_000);
        assert_eq!(x * y, 217);
        assert_eq!(y * y, 7159338172331303494);
        x *= y;
        assert_eq!(x, 217);
        y *= y;
        assert_eq!(y, 7159338172331303494);
    }

    #[test]
    fn test_mul_u16() {
        let mut x = ModularInteger::<u16>::new(10);
        let mut y = ModularInteger::<u16>::new(6553);
        assert_eq!(x * x, 100);
        assert_eq!(x * y, 9);
        assert_eq!(y * y, 25554);
        x *= y;
        assert_eq!(x, 9);
        y *= y;
        assert_eq!(y, 25554);
    }

    #[test]
    fn test_pow_u32() {
        let x = ModularInteger::<u32>::new(1_000);
        assert_eq!(x.pow(0), 1);
        assert_eq!(x.pow(1), 1_000);
        assert_eq!(x.pow(2), 1_000_000);
        assert_eq!(x.pow(8), 740_208_280);
        assert_eq!(x.pow(9), 1_473_905_948);
        assert_eq!(x.pow(10), 732_167_187);
        assert_eq!(x.pow(ModularInteger::<u32>::modulus()-2), 811_748_818);
        assert_eq!(x.pow(ModularInteger::<u32>::modulus()-1), 1);
    }

    #[test]
    fn test_pow_u64() {
        let x = ModularInteger::<u64>::new(1_000);
        assert_eq!(x.pow(0), 1);
        assert_eq!(x.pow(1), 1_000);
        assert_eq!(x.pow(2), 1_000_000);
        assert_eq!(x.pow(3), 1_000_000_000);
        assert_eq!(x.pow(ModularInteger::<u64>::modulus()-1), 1);
    }

    #[test]
    fn test_pow_u16() {
        let x = ModularInteger::<u16>::new(1_000);
        assert_eq!(x.pow(0), 1);
        assert_eq!(x.pow(1), 1000);
        assert_eq!(x.pow(2), 17185);
        assert_eq!(x.pow(3), 18498);
        assert_eq!(x.pow(ModularInteger::<u16>::modulus()-1), 1);
    }

    #[test]
    fn test_inv() {
        let x = ModularInteger::<u32>::new(2);
        let y = ModularInteger::<u32>::new(1_000);
        let z = ModularInteger::<u32>::new(ModularInteger::<u32>::modulus()-2);
        assert_eq!(x * x.inv(), 1);
        assert_eq!(y * y.inv(), 1);
        assert_eq!(z * z.inv(), 1);

        let x = ModularInteger::<u64>::new(2);
        let y = ModularInteger::<u64>::new(1_000);
        let z = ModularInteger::<u64>::new(ModularInteger::<u64>::modulus()-2);
        assert_eq!(x * x.inv(), 1);
        assert_eq!(y * y.inv(), 1);
        assert_eq!(z * z.inv(), 1);

        let x = ModularInteger::<u16>::new(2);
        let y = ModularInteger::<u16>::new(1_000);
        let z = ModularInteger::<u16>::new(ModularInteger::<u16>::modulus()-2);
        assert_eq!(x * x.inv(), 1);
        assert_eq!(y * y.inv(), 1);
        assert_eq!(z * z.inv(), 1);
    }

    #[test]
    fn test_fmt() {
        let x = ModularInteger::<u32>::new(12345);
        let display = format!("{}", x);
        let debug = format!("{:?}", x);
        assert_eq!(display, "12345".to_string());
        assert!(debug.contains("value: 12345"));
        assert!(debug.contains("modulus: 4294967291"));

        let x = ModularInteger::<u64>::new(12345);
        let display = format!("{}", x);
        let debug = format!("{:?}", x);
        assert_eq!(display, "12345".to_string());
        assert!(debug.contains("value: 12345"));
        assert!(debug.contains("modulus: 9223372036854775783"));

        let x = ModularInteger::<u16>::new(12345);
        let display = format!("{}", x);
        let debug = format!("{:?}", x);
        assert_eq!(display, "12345".to_string());
        assert!(debug.contains("value: 12345"));
        assert!(debug.contains("modulus: 65521"));
    }
}
