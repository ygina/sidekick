use std::fmt;
use std::cmp::PartialEq;
use std::ops::{Add, Sub, Mul, AddAssign, SubAssign, MulAssign, Neg};
use serde::{Serialize, Deserialize};


/// Modular integer.
#[derive(Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModularInteger<T> {
    value: T,
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

impl<T: Default + Copy> ModularInteger<T> {
    pub fn zero() -> Self {
        Self::default()
    }

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_constructor() {
        assert_eq!(ModularInteger::<u32>::new(0), 0);
        assert_eq!(ModularInteger::<u32>::new(1), 1);
        assert_eq!(ModularInteger::<u32>::new(4_294_967_290), 4_294_967_290);
    }

    #[test]
    fn test_field_getters() {
        let x = ModularInteger::<u32>::new(12345);
        assert_eq!(x.value(), 12345);
        assert_eq!(ModularInteger::<u32>::modulus(), 4_294_967_291);
    }

    #[test]
    fn test_zero_constructor() {
        let zero = ModularInteger::<u32>::zero();
        assert_eq!(zero, 0);
        assert!(zero.is_zero());
    }

    #[test]
    fn test_constructor_overflow() {
        assert_eq!(ModularInteger::<u32>::new(4_294_967_291), 0);
        assert_eq!(ModularInteger::<u32>::new(4_294_967_292), 1);
    }

    #[test]
    fn test_neg() {
        assert_eq!(-ModularInteger::<u32>::zero(), 0);
        assert_eq!(-ModularInteger::<u32>::new(1), 4_294_967_290);
        assert_eq!(-ModularInteger::<u32>::new(4_294_967_290), 1);
    }

    #[test]
    fn test_add() {
        let mut x = ModularInteger::<u32>::new(0);
        let y = ModularInteger::<u32>::new(1);
        let z = ModularInteger::<u32>::new(4_294_967_290);
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
        let mut x = ModularInteger::<u32>::new(0);
        let y = ModularInteger::<u32>::new(1);
        let z = ModularInteger::<u32>::new(4_294_967_290);
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
    fn test_pow() {
        let x = ModularInteger::<u32>::new(1_000);
        assert_eq!(x.pow(0), 1);
        assert_eq!(x.pow(1), 1_000);
        assert_eq!(x.pow(2), 1_000_000);
        assert_eq!(x.pow(8), 740_208_280);
        assert_eq!(x.pow(9), 1_473_905_948);
        assert_eq!(x.pow(10), 732_167_187);
        assert_eq!(x.pow(4_294_967_289), 811_748_818);
        assert_eq!(x.pow(4_294_967_290), 1);
    }

    #[test]
    fn test_inv() {
        let x = ModularInteger::<u32>::new(2);
        let y = ModularInteger::<u32>::new(1_000);
        let z = ModularInteger::<u32>::new(4_294_967_289);
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
        assert!(debug.contains(&format!("modulus: 4294967291")));
    }
}
