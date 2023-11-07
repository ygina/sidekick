use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

/// An element in the finite field with integers modulo a prime.
///
/// Uses `T`, an unsigned integer type, as the underlying representation. The
/// field is integers modulo the largest prime that fits in `T`. The crate
/// contains implementations for `u16`, `u32`, and `u64`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModularInteger<T> {
    pub(crate) value: T,
}

/// Arithmetic operations and other properties of the modular integer field.
pub trait ModularArithmetic {
    /// The smallest unsigned integer type that fits elements in the field.
    type SmallModulusType;

    /// The next largest unsigned integer type that fits elements in the field.
    type BigModulusType;

    /// Creates a new element in the finite field.
    ///
    /// The value of the new element is taken modulo a prime.
    fn new(n: Self::SmallModulusType) -> Self;

    /// The modulus prime of the finite field.
    fn modulus() -> Self::SmallModulusType;

    /// The modulus prime of the finite field, as a larger bit-width integer.
    fn modulus_big() -> Self::BigModulusType;

    /// The integer value of the element, where `0 <= value < MODULUS`.
    ///
    /// `MODULUS` is the largest prime integer that fits in `T`.
    fn value(&self) -> Self::SmallModulusType;

    /// Performs the `+=` operation in the finite field.
    fn add_assign(&mut self, rhs: Self);

    /// Performs the `-=` operation in the finite field.
    fn sub_assign(&mut self, rhs: Self);

    /// Performs the `*=` operation in the finite field.
    fn mul_assign(&mut self, rhs: Self);

    /// Raises the element to the `power`-th power in the finite field.
    fn pow(self, power: Self::SmallModulusType) -> Self;

    /// Performs the unary `-` operation in the finite field.
    fn neg(self) -> Self;

    /// The modular multiplicative inverse of the element.
    ///
    /// An element multiplied by its inverse is equal to one.
    fn inv(self) -> Self;

    /// Performs the `+` operation in the finite field.
    fn add(self, rhs: Self) -> Self
    where
        Self: Sized,
    {
        let mut result = self;
        result.add_assign(rhs);
        result
    }

    /// Performs the `-` operation in the finite field.
    fn sub(self, rhs: Self) -> Self
    where
        Self: Sized,
    {
        let mut result = self;
        result.sub_assign(rhs);
        result
    }

    /// Performs the `*` operation in the finite field.
    fn mul(self, rhs: Self) -> Self
    where
        Self: Sized,
    {
        let mut result = self;
        result.mul_assign(rhs);
        result
    }
}

impl<T: PartialEq> PartialEq<T> for ModularInteger<T> {
    fn eq(&self, other: &T) -> bool {
        self.value == *other
    }
}

////////////////////////////////////////////////////////////////////////////////

impl ModularArithmetic for ModularInteger<u32> {
    type SmallModulusType = u32;
    type BigModulusType = u64;

    fn new(n: Self::SmallModulusType) -> Self {
        if n >= Self::modulus() {
            Self {
                value: n - Self::modulus(),
            }
        } else {
            Self { value: n }
        }
    }

    /// The modulus prime of the finite field, `4_294_967_291`, as a `u32`.
    fn modulus() -> Self::SmallModulusType {
        4_294_967_291
    }

    /// The modulus prime of the finite field, `4_294_967_291`, as a `u64`.
    fn modulus_big() -> Self::BigModulusType {
        4_294_967_291
    }

    fn value(&self) -> Self::SmallModulusType {
        self.value
    }

    fn add_assign(&mut self, rhs: Self) {
        let sum: u64 = (self.value as u64) + (rhs.value as u64);
        self.value = if sum >= Self::modulus_big() {
            (sum - Self::modulus_big()) as u32
        } else {
            sum as u32
        };
    }

    fn sub_assign(&mut self, rhs: Self) {
        let neg_rhs: u64 = Self::modulus_big() - (rhs.value as u64);
        let diff: u64 = (self.value as u64) + neg_rhs;
        self.value = if diff >= Self::modulus_big() {
            (diff - Self::modulus_big()) as u32
        } else {
            diff as u32
        };
    }

    fn mul_assign(&mut self, rhs: Self) {
        let prod: u64 = (self.value as u64) * (rhs.value as u64);
        self.value = (prod % Self::modulus_big()) as u32;
    }

    fn pow(self, power: Self::SmallModulusType) -> Self {
        if power == 0 {
            Self::new(1)
        } else if power == 1 {
            self
        } else {
            let mut result = self.pow(power >> 1);
            result.mul_assign(result);
            if power & 1 == 1 {
                result.mul_assign(self);
            }
            result
        }
    }

    fn neg(self) -> Self {
        if self.value() == 0 {
            self
        } else {
            Self {
                value: Self::modulus() - self.value,
            }
        }
    }

    fn inv(self) -> Self {
        // n * inv(n) = n^(MODULUS-1) = 1 (mod MODULUS)
        self.pow(Self::modulus() - Self::new(2).value())
    }
}

////////////////////////////////////////////////////////////////////////////////

impl ModularArithmetic for ModularInteger<u64> {
    type SmallModulusType = u64;
    type BigModulusType = u128;

    fn new(n: Self::SmallModulusType) -> Self {
        if n >= Self::modulus() {
            Self {
                value: n - Self::modulus(),
            }
        } else {
            Self { value: n }
        }
    }

    /// The modulus prime of the finite field, `9_223_372_036_854_775_783`,
    /// as a `u64`.
    fn modulus() -> Self::SmallModulusType {
        9_223_372_036_854_775_783
    }

    /// The modulus prime of the finite field, `9_223_372_036_854_775_783`,
    /// as a `u128`.
    fn modulus_big() -> Self::BigModulusType {
        9_223_372_036_854_775_783
    }

    fn value(&self) -> Self::SmallModulusType {
        self.value
    }

    fn add_assign(&mut self, rhs: Self) {
        let sum: u128 = (self.value as u128) + (rhs.value as u128);
        self.value = if sum >= Self::modulus_big() {
            (sum - Self::modulus_big()) as u64
        } else {
            sum as u64
        };
    }

    fn sub_assign(&mut self, rhs: Self) {
        let neg_rhs: u128 = Self::modulus_big() - (rhs.value as u128);
        let diff: u128 = (self.value as u128) + neg_rhs;
        self.value = if diff >= Self::modulus_big() {
            (diff - Self::modulus_big()) as u64
        } else {
            diff as u64
        };
    }

    fn mul_assign(&mut self, rhs: Self) {
        let prod: u128 = (self.value as u128) * (rhs.value as u128);
        self.value = (prod % Self::modulus_big()) as u64;
    }

    fn pow(self, power: Self::SmallModulusType) -> Self {
        if power == 0 {
            Self::new(1)
        } else if power == 1 {
            self
        } else {
            let mut result = self.pow(power >> 1);
            result.mul_assign(result);
            if power & 1 == 1 {
                result.mul_assign(self);
            }
            result
        }
    }

    fn neg(self) -> Self {
        if self.value() == 0 {
            self
        } else {
            Self {
                value: Self::modulus() - self.value,
            }
        }
    }

    fn inv(self) -> Self {
        // n * inv(n) = n^(MODULUS-1) = 1 (mod MODULUS)
        self.pow(Self::modulus() - Self::new(2).value())
    }
}

////////////////////////////////////////////////////////////////////////////////

impl ModularArithmetic for ModularInteger<u16> {
    type SmallModulusType = u16;
    type BigModulusType = u32;

    fn new(n: Self::SmallModulusType) -> Self {
        if n >= Self::modulus() {
            Self {
                value: n - Self::modulus(),
            }
        } else {
            Self { value: n }
        }
    }

    /// The modulus prime of the finite field, `65521`, as a `u16`.
    fn modulus() -> Self::SmallModulusType {
        65521
    }

    /// The modulus prime of the finite field, `65521`, as a `u32`.
    fn modulus_big() -> Self::BigModulusType {
        65521
    }

    fn value(&self) -> Self::SmallModulusType {
        self.value
    }

    fn add_assign(&mut self, rhs: Self) {
        let sum: u32 = (self.value as u32) + (rhs.value as u32);
        self.value = if sum >= Self::modulus_big() {
            (sum - Self::modulus_big()) as u16
        } else {
            sum as u16
        };
    }

    fn sub_assign(&mut self, rhs: Self) {
        let neg_rhs: u32 = Self::modulus_big() - (rhs.value as u32);
        let diff: u32 = (self.value as u32) + neg_rhs;
        self.value = if diff >= Self::modulus_big() {
            (diff - Self::modulus_big()) as u16
        } else {
            diff as u16
        };
    }

    fn mul_assign(&mut self, rhs: Self) {
        let prod: u32 = (self.value as u32) * (rhs.value as u32);
        self.value = (prod % Self::modulus_big()) as u16;
    }

    fn pow(self, power: Self::SmallModulusType) -> Self {
        if power == 0 {
            Self::new(1)
        } else if power == 1 {
            self
        } else {
            let mut result = self.pow(power >> 1);
            result.mul_assign(result);
            if power & 1 == 1 {
                result.mul_assign(self);
            }
            result
        }
    }

    fn neg(self) -> Self {
        if self.value() == 0 {
            self
        } else {
            Self {
                value: Self::modulus() - self.value,
            }
        }
    }

    fn inv(self) -> Self {
        // n * inv(n) = n^(MODULUS-1) = 1 (mod MODULUS)
        self.pow(Self::modulus() - Self::new(2).value())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use once_cell::sync::Lazy;

    static U16_MODULUS: Lazy<u16> = Lazy::new(|| ModularInteger::<u16>::modulus());
    static U32_MODULUS: Lazy<u32> = Lazy::new(|| ModularInteger::<u32>::modulus());
    static U64_MODULUS: Lazy<u64> = Lazy::new(|| ModularInteger::<u64>::modulus());

    #[test]
    fn test_zero_constructor() {
        assert_eq!(ModularInteger::<u16>::new(0), 0);
        assert_eq!(ModularInteger::<u32>::new(0), 0);
        assert_eq!(ModularInteger::<u64>::new(0), 0);
        assert_eq!(ModularInteger::<u16>::new(*U16_MODULUS), 0);
        assert_eq!(ModularInteger::<u32>::new(*U32_MODULUS), 0);
        assert_eq!(ModularInteger::<u64>::new(*U64_MODULUS), 0);
    }

    #[test]
    fn test_nonzero_constructor() {
        assert_eq!(
            ModularInteger::<u16>::new(*U16_MODULUS - 1),
            *U16_MODULUS - 1
        );
        assert_eq!(
            ModularInteger::<u32>::new(*U32_MODULUS - 1),
            *U32_MODULUS - 1
        );
        assert_eq!(
            ModularInteger::<u64>::new(*U64_MODULUS - 1),
            *U64_MODULUS - 1
        );
        assert_eq!(ModularInteger::<u16>::new(*U16_MODULUS + 1), 1);
        assert_eq!(ModularInteger::<u32>::new(*U32_MODULUS + 1), 1);
        assert_eq!(ModularInteger::<u64>::new(*U64_MODULUS + 1), 1);
    }

    #[test]
    fn test_modulus_big() {
        assert_eq!(ModularInteger::<u16>::modulus_big(), *U16_MODULUS as u32);
        assert_eq!(ModularInteger::<u32>::modulus_big(), *U32_MODULUS as u64);
        assert_eq!(ModularInteger::<u64>::modulus_big(), *U64_MODULUS as u128);
    }

    #[test]
    fn test_equals_operation() {
        assert_eq!(
            ModularInteger::<u16>::new(*U16_MODULUS),
            ModularInteger::<u16>::new(0)
        );
        assert_eq!(
            ModularInteger::<u32>::new(*U32_MODULUS),
            ModularInteger::<u32>::new(0)
        );
        assert_eq!(
            ModularInteger::<u64>::new(*U64_MODULUS),
            ModularInteger::<u64>::new(0)
        );
    }

    #[test]
    fn test_neg() {
        assert_eq!(ModularInteger::<u16>::new(0).neg(), 0);
        assert_eq!(ModularInteger::<u16>::new(1).neg(), *U16_MODULUS - 1);
        assert_eq!(ModularInteger::<u16>::new(*U16_MODULUS - 1).neg(), 1);
        assert_eq!(ModularInteger::<u32>::new(0).neg(), 0);
        assert_eq!(ModularInteger::<u32>::new(1).neg(), *U32_MODULUS - 1);
        assert_eq!(ModularInteger::<u32>::new(*U32_MODULUS - 1).neg(), 1);
        assert_eq!(ModularInteger::<u64>::new(0).neg(), 0);
        assert_eq!(ModularInteger::<u64>::new(1).neg(), *U64_MODULUS - 1);
        assert_eq!(ModularInteger::<u64>::new(*U64_MODULUS - 1).neg(), 1);
    }

    #[test]
    fn test_add_u16() {
        let x = ModularInteger::<u16>::new(1);
        let y = ModularInteger::<u16>::new(2);
        let z = ModularInteger::<u16>::new(*U16_MODULUS - 1);
        assert_eq!(x.add(x), 2);
        assert_eq!(x.add(y), 3);
        assert_eq!(x.add(z), 0);
        assert_eq!(z.add(x), 0);
        assert_eq!(y.add(z), 1);
        assert_eq!(z.add(y), 1);
    }

    #[test]
    fn test_add_u32() {
        let x = ModularInteger::<u32>::new(1);
        let y = ModularInteger::<u32>::new(2);
        let z = ModularInteger::<u32>::new(*U32_MODULUS - 1);
        assert_eq!(x.add(x), 2);
        assert_eq!(x.add(y), 3);
        assert_eq!(x.add(z), 0);
        assert_eq!(z.add(x), 0);
        assert_eq!(y.add(z), 1);
        assert_eq!(z.add(y), 1);
    }

    #[test]
    fn test_add_u64() {
        let x = ModularInteger::<u64>::new(1);
        let y = ModularInteger::<u64>::new(2);
        let z = ModularInteger::<u64>::new(*U64_MODULUS - 1);
        assert_eq!(x.add(x), 2);
        assert_eq!(x.add(y), 3);
        assert_eq!(x.add(z), 0);
        assert_eq!(z.add(x), 0);
        assert_eq!(y.add(z), 1);
        assert_eq!(z.add(y), 1);
    }

    #[test]
    fn test_sub_u16() {
        let x = ModularInteger::<u16>::new(0);
        let y = ModularInteger::<u16>::new(1);
        let z = ModularInteger::<u16>::new(*U16_MODULUS - 1);
        assert_eq!(x.sub(y), *U16_MODULUS - 1);
        assert_eq!(x.sub(z), 1);
        assert_eq!(y.sub(x), 1);
        assert_eq!(y.sub(z), 2);
        assert_eq!(z.sub(x), *U16_MODULUS - 1);
        assert_eq!(z.sub(y), *U16_MODULUS - 2);
    }

    #[test]
    fn test_sub_u32() {
        let x = ModularInteger::<u32>::new(0);
        let y = ModularInteger::<u32>::new(1);
        let z = ModularInteger::<u32>::new(*U32_MODULUS - 1);
        assert_eq!(x.sub(y), *U32_MODULUS - 1);
        assert_eq!(x.sub(z), 1);
        assert_eq!(y.sub(x), 1);
        assert_eq!(y.sub(z), 2);
        assert_eq!(z.sub(x), *U32_MODULUS - 1);
        assert_eq!(z.sub(y), *U32_MODULUS - 2);
    }

    #[test]
    fn test_sub_u64() {
        let x = ModularInteger::<u64>::new(0);
        let y = ModularInteger::<u64>::new(1);
        let z = ModularInteger::<u64>::new(*U64_MODULUS - 1);
        assert_eq!(x.sub(y), *U64_MODULUS - 1);
        assert_eq!(x.sub(z), 1);
        assert_eq!(y.sub(x), 1);
        assert_eq!(y.sub(z), 2);
        assert_eq!(z.sub(x), *U64_MODULUS - 1);
        assert_eq!(z.sub(y), *U64_MODULUS - 2);
    }

    #[test]
    fn test_mul_u32() {
        let x = ModularInteger::<u32>::new(1_000);
        let y = ModularInteger::<u32>::new(4_294_968);
        assert_eq!(x.mul(x), 1_000_000);
        assert_eq!(x.mul(y), 709);
        assert_eq!(y.mul(y), 4_160_573_470);
    }

    #[test]
    fn test_mul_u64() {
        let x = ModularInteger::<u64>::new(1_000);
        let y = ModularInteger::<u64>::new(9223372036854776);
        assert_eq!(x.mul(x), 1_000_000);
        assert_eq!(x.mul(y), 217);
        assert_eq!(y.mul(y), 7159338172331303494);
    }

    #[test]
    fn test_mul_u16() {
        let x = ModularInteger::<u16>::new(10);
        let y = ModularInteger::<u16>::new(6553);
        assert_eq!(x.mul(x), 100);
        assert_eq!(x.mul(y), 9);
        assert_eq!(y.mul(y), 25554);
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
        assert_eq!(x.pow(*U32_MODULUS - 2), 811_748_818);
        assert_eq!(x.pow(*U32_MODULUS - 1), 1);
    }

    #[test]
    fn test_pow_u64() {
        let x = ModularInteger::<u64>::new(1_000);
        assert_eq!(x.pow(0), 1);
        assert_eq!(x.pow(1), 1_000);
        assert_eq!(x.pow(2), 1_000_000);
        assert_eq!(x.pow(3), 1_000_000_000);
        assert_eq!(x.pow(*U64_MODULUS - 1), 1);
    }

    #[test]
    fn test_pow_u16() {
        let x = ModularInteger::<u16>::new(1_000);
        assert_eq!(x.pow(0), 1);
        assert_eq!(x.pow(1), 1000);
        assert_eq!(x.pow(2), 17185);
        assert_eq!(x.pow(3), 18498);
        assert_eq!(x.pow(*U16_MODULUS - 1), 1);
    }

    #[test]
    fn test_inv() {
        let x = ModularInteger::<u32>::new(2);
        let y = ModularInteger::<u32>::new(1_000);
        let z = ModularInteger::<u32>::new(*U32_MODULUS - 2);
        assert_eq!(x.mul(x.inv()), 1);
        assert_eq!(y.mul(y.inv()), 1);
        assert_eq!(z.mul(z.inv()), 1);

        let x = ModularInteger::<u64>::new(2);
        let y = ModularInteger::<u64>::new(1_000);
        let z = ModularInteger::<u64>::new(*U64_MODULUS - 2);
        assert_eq!(x.mul(x.inv()), 1);
        assert_eq!(y.mul(y.inv()), 1);
        assert_eq!(z.mul(z.inv()), 1);

        let x = ModularInteger::<u16>::new(2);
        let y = ModularInteger::<u16>::new(1_000);
        let z = ModularInteger::<u16>::new(*U16_MODULUS - 2);
        assert_eq!(x.mul(x.inv()), 1);
        assert_eq!(y.mul(y.inv()), 1);
        assert_eq!(z.mul(z.inv()), 1);
    }
}
