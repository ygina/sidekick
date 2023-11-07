cfg_montgomery! {
    use crate::arithmetic::MontgomeryInteger;
}
use crate::arithmetic::{ModularArithmetic, ModularInteger};

/// The coefficient vector defines a univariate polynomial where the constant
/// term in the polynomial is the last element in the vector. The polynomial is
/// monic, meaning the leading coefficient is one, and this is not included in
/// the vector. The number of elements in the vector is the degree of the
/// polynomial.
pub type CoefficientVector<T> = Vec<ModularInteger<T>>;

#[cfg(feature = "libpari")]
#[link(name = "pari", kind = "dylib")]
extern "C" {
    fn factor_libpari(roots: *mut u32, coeffs: *const u32, field: u32, degree: usize) -> i32;
}

/// Evaluate the univariate polynomial at `x` in a modular finite field.
///
/// Assumes all integers are modulo the same prime.
///
/// # Returns
///
/// The polynomial evaluated at `x`, in the same field.
pub fn eval<T>(
    coeffs: &CoefficientVector<T>,
    x: <ModularInteger<T> as ModularArithmetic>::SmallModulusType,
) -> ModularInteger<T>
where
    ModularInteger<T>: ModularArithmetic,
    T: Copy,
{
    let size = coeffs.len();
    let x_mod = ModularInteger::<T>::new(x);
    let mut result = x_mod;
    // result = x(...(x(x(x+a0)+a1)+...))
    // e.g., result = x(x+a0)+a1
    for &coeff in coeffs.iter().take(size - 1) {
        result.add_assign(coeff);
        result.mul_assign(x_mod);
    }
    result.add(coeffs[size - 1])
}

cfg_montgomery! {
    /// Evaluate the univariate polynomial at `x`, assuming that `x` and the
    /// the coefficients are given in Montgomery form using the same moduli.
    ///
    /// Assumes all integers use the same prime modulus `N` and co-prime
    /// auxiliary modulus `R` in Montgomery form.
    ///
    /// # Returns
    ///
    /// The polynomial evaluated at `x`, in Montgomery form.
    pub fn eval_montgomery(
        coeffs: &Vec<MontgomeryInteger>,
        x: u64,
    ) -> MontgomeryInteger {
        let size = coeffs.len();
        let x_mod = MontgomeryInteger::new(x);
        let mut result = x_mod;
        // result = x(...(x(x(x+a0)+a1)+...))
        // e.g., result = x(x+a0)+a1
        for &coeff in coeffs.iter().take(size - 1) {
            result.add_assign(coeff);
            result.mul_assign(x_mod);
        }
        result.add(coeffs[size - 1])
    }
}

// cfg_power_table! {
//     pub fn eval_precompute(coeffs: &CoefficientVector<u16>, x: u16) -> ModularInteger<u16> {
//         let size = coeffs.len();
//         let x_modint = ModularInteger::<u16>::new(x);
//         let mut result: u64 =
//             unsafe { crate::POWER_TABLE[x_modint.value as usize][size] }.value() as u64;
//         for (i, coeff) in coeffs.iter().enumerate().take(size - 1) {
//             result += (coeff.value() as u64)
//                 * (unsafe { crate::POWER_TABLE[x_modint.value as usize][size - i - 1] }.value()
//                     as u64);
//         }
//         result += coeffs[size - 1].value() as u64;
//         ModularInteger::new((result % (ModularInteger::<u16>::modulus() as u64)) as u16)
//     }
// }

cfg_libpari! {
    /// Factor the univariate polynomial using modular arithmetic, returning
    /// the roots.
    ///
    /// Assumes all integers are modulo the same 32-bit prime.
    ///
    /// # Returns
    ///
    /// On success, a length-`n` vector of all `n` real integer roots of the
    /// degree-`n` polynomial are returned. If the polynomial cannot be
    /// factored, an error is returned instead.
    pub fn factor(coeffs: &CoefficientVector<u32>) -> Result<Vec<u32>, String> {
        assert_ne!(coeffs.len(), 0);
        let modulus = ModularInteger::<u32>::modulus();
        let mut coeffs = coeffs.iter().map(|x| x.value()).collect::<Vec<_>>();
        coeffs.insert(0, 1);
        let mut roots: Vec<u32> = vec![0; coeffs.len() - 1];
        if unsafe { factor_libpari(roots.as_mut_ptr(), coeffs.as_ptr(), modulus, roots.len()) } == 0
        {
            Ok(roots)
        } else {
            Err("could not factor polynomial".to_string())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::arithmetic;

    #[test]
    fn test_eval_no_modulus() {
        // f(x) = x^2 + 2*x - 3
        // f(0) = -3
        // f(1) = 0
        // f(2) = 5
        // f(3) = 12
        let coeffs = vec![
            ModularInteger::<u32>::new(2),
            ModularInteger::<u32>::new(3).neg(),
        ];
        assert_eq!(
            arithmetic::eval(&coeffs, 0),
            ModularInteger::<u32>::new(3).neg()
        );
        assert_eq!(arithmetic::eval(&coeffs, 1), 0);
        assert_eq!(arithmetic::eval(&coeffs, 2), 5);
        assert_eq!(arithmetic::eval(&coeffs, 3), 12);
    }

    // #[cfg(feature = "power_table")]
    // #[test]
    // fn test_eval_no_modulus_precompute() {
    //     // f(x) = x^2 + 2*x - 3
    //     // f(0) = -3
    //     // f(1) = 0
    //     // f(2) = 5
    //     // f(3) = 12
    //     crate::quack_internal::init_pow_table();
    //     let coeffs = vec![
    //         ModularInteger::<u16>::new(2),
    //         ModularInteger::<u16>::new(3).neg(),
    //     ];
    //     assert_eq!(
    //         arithmetic::eval_precompute(&coeffs, 0),
    //         ModularInteger::<u16>::new(3).neg()
    //     );
    //     assert_eq!(arithmetic::eval_precompute(&coeffs, 1), 0);
    //     assert_eq!(arithmetic::eval_precompute(&coeffs, 2), 5);
    //     assert_eq!(arithmetic::eval_precompute(&coeffs, 3), 12);
    // }

    #[test]
    fn test_eval_with_modulus() {
        let r1: u64 = 95976998;
        let r2: u64 = 456975625;
        let r3: u64 = 1202781556;
        let modulus = ModularInteger::<u32>::modulus_big();

        let coeffs = vec![
            ModularInteger::<u32>::new(((r1 + r2 + r3) % modulus) as u32).neg(),
            ModularInteger::<u32>::new(((r1 * r2 + r2 * r3 + r1 * r3) % modulus) as u32),
            ModularInteger::<u32>::new(((((r1 * r2) % modulus) * r3) % modulus) as u32).neg(),
        ];

        // Test zeros.
        assert_eq!(arithmetic::eval(&coeffs, r1 as u32), 0);
        assert_eq!(arithmetic::eval(&coeffs, r2 as u32), 0);
        assert_eq!(arithmetic::eval(&coeffs, r3 as u32), 0);

        // Test other points.
        assert_ne!(arithmetic::eval(&coeffs, (r1 as u32) + 1), 0);
        assert_ne!(arithmetic::eval(&coeffs, (r2 as u32) + 1), 0);
        assert_ne!(arithmetic::eval(&coeffs, (r3 as u32) + 1), 0);
    }

    #[cfg(feature = "libpari")]
    #[test]
    fn test_factor() {
        // NOTE: There can only be one libpari test or the tests will segfault
        // when run concurrently.

        // no modulus
        // f(x) = x^2 + 2*x - 3
        // f(x) = 0 when x = -3, 1
        let coeffs = vec![
            ModularInteger::<u32>::new(2),
            ModularInteger::<u32>::new(3).neg(),
        ];
        let mut roots = arithmetic::factor(&coeffs).unwrap();
        roots.sort();
        assert_eq!(roots.len(), 2);
        assert_eq!(roots, vec![1, ModularInteger::<u32>::modulus() - 3]);

        // with modulus
        let r1: u64 = 95976998;
        let r2: u64 = 456975625;
        let r3: u64 = 1202781556;
        let modulus = ModularInteger::<u32>::modulus_big();

        let coeffs = vec![
            ModularInteger::<u32>::new(((r1 + r2 + r3) % modulus) as u32).neg(),
            ModularInteger::<u32>::new(((r1 * r2 + r2 * r3 + r1 * r3) % modulus) as u32),
            ModularInteger::<u32>::new(((((r1 * r2) % modulus) * r3) % modulus) as u32).neg(),
        ];
        let mut roots = arithmetic::factor(&coeffs).unwrap();
        roots.sort();
        assert_eq!(roots.len(), 3);
        assert_eq!(roots, vec![r1 as u32, r2 as u32, r3 as u32]);
    }
}
