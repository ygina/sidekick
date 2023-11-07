use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use crate::arithmetic::ModularArithmetic;

// N
const N: u64 = 18_446_744_073_709_551_557;
// The auxiliary modulus R is 1 << R_LOG2
const R_LOG2: u128 = 64;
// R mod N
const R_MOD_N: u64 = ((1 << R_LOG2) % (N as u128)) as u64;
// N' such that NN' = -1 mod R
const N_NEGMODINV_R: u128 = 14_694_863_923_124_558_067;


/// A 64-bit finite field element in [Montgomery form](https://en.wikipedia.org/wiki/Montgomery_modular_multiplication).
///
/// The Montgomery modular multiplication algorithm uses the Montgomery forms of
/// `a` and `b` to efficiently compute the Montgomery form of `ab mod N`. The
/// efficiency comes from avoiding expensive division operations.
///
/// The auxiliary modulus `R` must be a positive integer such that
/// `gcd(R, N) = 1`. Division and reduction modulo `R` should be inexpensive,
/// and `R > N` to be useful for modular multiplication. The implementation
/// uses `N = 18446744073709551557`, the largest 64-bit prime, and
/// `R = 1 << 64`, a co-prime power of two with efficient division and modulus.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MontgomeryInteger {
    value: u64,
}

impl MontgomeryInteger where MontgomeryInteger: ModularArithmetic {
    /// Create a new Montgomery integer, doing the conversion from the original
    /// integer to the integer in Montgomery form.
    ///
    /// The _Montgomery form_ of the residue class `a_bar` with respect to `R`
    /// is `aR mod N`. For example, suppose that `N = 17` and that `R = 100`.
    /// The Montgomery forms of `3` and `15` are `300 mod 17 = 11` and
    /// `1500 mod 17 = 4`.
    pub fn new_do_conversion(n: u64) -> Self {
        let product = ((R_MOD_N as u128) * (n as u128)) % (N as u128);
        Self { value: product as u64 }
    }
}

impl ModularArithmetic for MontgomeryInteger {
    type SmallModulusType = u64;
    type BigModulusType = u128;

    /// Creates a new Montgomery integer, assuming the provided integer is
    /// already in Montgomery form `n = a*R mod N`.
    fn new(n: u64) -> Self {
        if n >= Self::modulus() {
            Self { value: n - Self::modulus() }
        } else {
            Self { value: n }
        }
    }

    /// The original prime modulus, `18_446_744_073_709_551_557`, as a `u64`.
    /// This is the largest unsigned 64-bit prime.
    fn modulus() -> Self::SmallModulusType {
        N
    }

    /// The original prime modulus, `18_446_744_073_709_551_557`, as a `u128`.
    /// This is the largest unsigned 64-bit prime.
    fn modulus_big() -> Self::BigModulusType {
        N as u128
    }

    fn value(&self) -> Self::SmallModulusType {
        self.value
    }

    /// Performs the `+=` operation in the finite field.
    ///
    /// Addition in Montgomery form is the same as ordinary modular addition
    /// because of the distributive law: `aR + bR = (a + b)R`.
    fn add_assign(&mut self, rhs: Self) {
        let sum: u128 = (self.value as u128) + (rhs.value as u128);
        self.value = if sum >= Self::modulus_big() {
            (sum - Self::modulus_big()) as u64
        } else {
            sum as u64
        };
    }

    /// Performs the `-=` operation in the finite field.
    ///
    /// Subtraction in Montgomery form is the same as ordinary modular
    /// subtraction because of the distributive law: `aR - bR = (a - b)R`.
    fn sub_assign(&mut self, rhs: Self) {
        let neg_rhs: u64 = Self::modulus() - rhs.value;
        let diff: u128 = (self.value as u128) + (neg_rhs as u128);
        self.value = if diff >= Self::modulus_big() {
            (diff - Self::modulus_big()) as u64
        } else {
            diff as u64
        };
    }

    /// Performs the `*=` operation in the finite field.
    ///
    /// Multiplication in Montgomery form is seemingly more complicated. The
    /// usual product of `aR` and `bR` does not represent the product of `a` and
    /// `b` because it has an extra factor of `R`:
    /// `(aR mod N)(bR mod N) mod N = (abR)R mod N`.
    ///
    /// Removing the extra factor of `R` can be done by multiplying by an
    /// integer `R'` such that `RR' = 1 mod N`, that is, the modular inverse of
    /// `R mod N`. [Montgomery reduction](https://en.wikipedia.org/wiki/Montgomery_modular_multiplication#The_REDC_algorithm),
    /// also known as REDC, is an algorithm that simultaneously computes the
    /// product by `R'` and reduces modulo `N` more quickly than the naive
    /// method. REDC focuses on making the number more divisible by `R`.
    fn mul_assign(&mut self, rhs: Self) {
        let x = (self.value as u128) * (rhs.value as u128);  // T
        let m: u64 = (((x as u64) as u128) * N_NEGMODINV_R) as u64;  // cast as u64 to mod R
        let extra_bit = x.overflowing_add((m as u128) * MontgomeryInteger::modulus_big()).1;
        let sum: u128 = x.overflowing_add((m as u128) * MontgomeryInteger::modulus_big()).0;
        let t: u64 = (sum >> R_LOG2) as u64;
        self.value = if extra_bit {
            t.overflowing_sub(MontgomeryInteger::modulus()).0
        } else if t < MontgomeryInteger::modulus() {
            t
        } else {
            t - MontgomeryInteger::modulus()
        };
    }

    fn pow(self, power: u64) -> Self {
        if power == 0 {
            MontgomeryInteger::new(R_MOD_N)  // a^0*R mod N = R mod N
        } else if power == 1 {
            self  // a^1*R mod N = aR mod N (same as in normal pow)
        } else {
            // same as in normal pow
            let mut result = self.pow(power >> 1);
            result.mul_assign(result);
            if power & 1 == 1 {
                result.mul_assign(self);
            }
            result
        }
    }

    fn neg(self) -> Self {
        // same as in normal modular negation: `-a*R mod N = -aR mod N`
        if self.value == 0 {
            self
        } else {
            Self { value: Self::modulus() - self.value }
        }
    }

    fn inv(self) -> Self {
        // n * inv(n) = n^(N-1) = 1 (mod N)
        // (aR mod N)(inv(aR mod N)) = R mod N
        // inv(aR mod N) = a^-1 mod N = (aR)^-1 * R mod N
        self.pow(Self::modulus() - 2).mul(Self::new(R_MOD_N))
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::arithmetic::ModularInteger;

    fn from_montgomery_form(x: MontgomeryInteger) -> u64 {
        let r_modinv_n = ModularInteger::<u64>::new(R_MOD_N).inv().value();
        let product = (x.value() as u128) * (r_modinv_n as u128);
        (product % (N as u128)) as u64
    }

    #[test]
    fn test_zero_constructor() {
        assert_eq!(MontgomeryInteger::new(0), 0);
        assert_eq!(MontgomeryInteger::new(N), 0);
        assert_eq!(MontgomeryInteger::new_do_conversion(0), 0);
        assert_eq!(MontgomeryInteger::new_do_conversion(N), 0);
        assert_eq!(from_montgomery_form(MontgomeryInteger::new(0)), 0);
    }

    #[test]
    fn test_nonzero_constructor() {
        assert_eq!(MontgomeryInteger::new(N - 1), N - 1);
        assert_eq!(MontgomeryInteger::new(N + 1), 1);

        // conversion works properly
        assert_eq!(MontgomeryInteger::new_do_conversion(1), R_MOD_N);
        assert_eq!(MontgomeryInteger::new_do_conversion(2), 2 * R_MOD_N);
        let expected = (((N - 1) as u128) * (1 << R_LOG2)) % (N as u128);
        assert_eq!(MontgomeryInteger::new_do_conversion(N - 1), expected as u64);
        assert_eq!(MontgomeryInteger::new_do_conversion(N + 1), R_MOD_N);
        assert_eq!(MontgomeryInteger::new_do_conversion(N + 2), 2 * R_MOD_N);

        // and in reverse
        assert_eq!(from_montgomery_form(MontgomeryInteger::new(R_MOD_N)), 1);
        assert_eq!(from_montgomery_form(MontgomeryInteger::new(2 * R_MOD_N)), 2);
        assert_eq!(from_montgomery_form(MontgomeryInteger::new(expected as u64)), N - 1);
    }

    #[test]
    fn test_constants() {
        assert_eq!(MontgomeryInteger::modulus(), N);
        assert_eq!(MontgomeryInteger::modulus_big(), N as u128);
        assert_eq!(MontgomeryInteger::modulus(), ModularInteger::<u64>::modulus());
        const R: u128 = 1 << R_LOG2;
        assert_eq!(((N as u128) * N_NEGMODINV_R) % R, R - 1);
    }

    #[test]
    fn test_equals() {
        assert_eq!(MontgomeryInteger::new(0), MontgomeryInteger::new(0));
        assert_eq!(MontgomeryInteger::new(0), MontgomeryInteger::new(N));
        assert_eq!(MontgomeryInteger::new(1), MontgomeryInteger::new(N+1));
        assert_eq!(MontgomeryInteger::new(1000), MontgomeryInteger::new(1000));
    }

    #[test]
    fn test_neg() {
        assert_eq!(MontgomeryInteger::new(0).neg(), MontgomeryInteger::new(0));
        assert_eq!(MontgomeryInteger::new(1).neg(), MontgomeryInteger::new(N - 1));
        assert_eq!(MontgomeryInteger::new(N - 1).neg(), MontgomeryInteger::new(1));
    }

    #[test]
    fn test_add() {
        let x = MontgomeryInteger::new(1);
        let y = MontgomeryInteger::new(2);
        let z = MontgomeryInteger::new(N - 1);
        assert_eq!(x.add(x), MontgomeryInteger::new(2));
        assert_eq!(x.add(y), MontgomeryInteger::new(3));
        assert_eq!(x.add(z), MontgomeryInteger::new(0));
        assert_eq!(z.add(x), MontgomeryInteger::new(0));
        assert_eq!(y.add(z), MontgomeryInteger::new(1));
        assert_eq!(z.add(y), MontgomeryInteger::new(1));
    }

    #[test]
    fn test_sub() {
        let x = MontgomeryInteger::new(0);
        let y = MontgomeryInteger::new(1);
        let z = MontgomeryInteger::new(N - 1);
        assert_eq!(x.sub(y), MontgomeryInteger::new(N - 1));
        assert_eq!(x.sub(z), MontgomeryInteger::new(1));
        assert_eq!(y.sub(x), MontgomeryInteger::new(1));
        assert_eq!(y.sub(z), MontgomeryInteger::new(2));
        assert_eq!(z.sub(x), MontgomeryInteger::new(N - 1));
        assert_eq!(z.sub(y), MontgomeryInteger::new(N - 2));
    }

    #[ignore]
    #[test]
    fn test_mul() {
        let x = 10_000;
        let y = 9223372036854776;
        let mod_x = MontgomeryInteger::new(x);
        let mod_y = MontgomeryInteger::new(y);
        assert_eq!(mod_x.mul(mod_x), MontgomeryInteger::new(x * x));
        assert_eq!(
            mod_x.mul(mod_y),
            MontgomeryInteger::new(((x as u128 * y as u128) % (N as u128)) as u64)
        );
        assert_eq!(
            mod_y.mul(mod_y),
            MontgomeryInteger::new(((y as u128 * y as u128) % (N as u128)) as u64)
        );
    }

    #[ignore]
    #[test]
    fn test_pow() {
        let x = MontgomeryInteger::new(1_000);
        assert_eq!(x.pow(0), MontgomeryInteger::new(1));
        assert_eq!(x.pow(1), MontgomeryInteger::new(1_000));
        assert_eq!(x.pow(2), MontgomeryInteger::new(1_000_000));
        assert_eq!(x.pow(3), MontgomeryInteger::new(1_000_000_000));
        assert_eq!(x.pow(N - 1), MontgomeryInteger::new(1));
    }

    #[ignore]
    #[test]
    fn test_inv() {
        let x = MontgomeryInteger::new(2);
        let y = MontgomeryInteger::new(1_000);
        let z = MontgomeryInteger::new(N - 2);
        let one = MontgomeryInteger::new(1);
        assert_eq!(x.mul(x.inv()), one);
        assert_eq!(y.mul(y.inv()), one);
        assert_eq!(z.mul(z.inv()), one);
    }
}
