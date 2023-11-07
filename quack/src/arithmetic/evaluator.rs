// #[cfg(feature = "montgomery")]
// use crate::arithmetic::MontgomeryInteger;
// use crate::arithmetic::{ModularArithmetic, ModularInteger};

// #[cfg(feature = "libpari")]
// #[link(name = "pari", kind = "dylib")]
// extern "C" {
//     fn factor_libpari(roots: *mut u32, coeffs: *const u32, field: u32, degree: usize) -> i32;
// }

// /// Evaluate the univariate polynomial with the given coefficients using
// /// modular arithmetic, assuming all coefficients are modulo the same
// /// 32-bit prime. In the coefficient vector, the last element is the
// /// constant term in the polynomial. The number of coefficients is the
// /// degree of the polynomial. The leading coefficient is 1, and is not
// /// included in the vector.
// pub fn eval<T>(coeffs: &Vec<ModularInteger<T>>, x: <ModularInteger<T> as ModularArithmetic>::SmallModulusType) -> ModularInteger<T>
//     where ModularInteger<T>: ModularArithmetic, T: Copy {
//     let size = coeffs.len();
//     let x_mod = ModularInteger::<T>::new(x);
//     let mut result = x_mod;
//     // result = x(...(x(x(x+a0)+a1)+...))
//     // e.g., result = x(x+a0)+a1
//     for &coeff in coeffs.iter().take(size - 1) {
//         result.add_assign(coeff);
//         result.mul_assign(x_mod);
//     }
//     result.add(coeffs[size - 1])
// }

// cfg_montgomery! {
//     pub fn eval_montgomery(coeffs: &Vec<MontgomeryInteger>, x: u64) -> MontgomeryInteger {
//         let size = coeffs.len();
//         let x_mod = MontgomeryInteger::new(x);
//         let mut result = x_mod;
//         // result = x(...(x(x(x+a0)+a1)+...))
//         // e.g., result = x(x+a0)+a1
//         for &coeff in coeffs.iter().take(size - 1) {
//             result += coeff;
//             result *= x_mod;
//         }
//         result + coeffs[size - 1]
//     }
// }

// cfg_power_table! {
//     pub fn eval_precompute(coeffs: &Vec<ModularInteger<u16>>, x: u16) -> ModularInteger<u16> {
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

// cfg_libpari! {
//     /// Factors the given polynomial using modular arithmetic, assuming all
//     /// coefficients are modulo the same 32-bit prime.
//     ///
//     /// In the coefficient vector, the last element is the
//     /// constant term in the polynomial. The number of coefficients is the
//     /// degree of the polynomial. The leading coefficient is 1, and is not
//     /// included in the vector.
//     pub fn factor(coeffs: &Vec<ModularInteger<u32>>) -> Result<Vec<u32>, String> {
//         assert_ne!(coeffs.len(), 0);
//         let modulus = ModularInteger::<u32>::modulus();
//         let mut coeffs = coeffs.iter().map(|x| x.value()).collect::<Vec<_>>();
//         coeffs.insert(0, 1);
//         let mut roots: Vec<u32> = vec![0; coeffs.len() - 1];
//         if unsafe { factor_libpari(roots.as_mut_ptr(), coeffs.as_ptr(), modulus, roots.len()) } == 0
//         {
//             Ok(roots)
//         } else {
//             Err("could not factor polynomial".to_string())
//         }
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::arithmetic;

//     #[test]
//     fn test_eval_no_modulus() {
//         // f(x) = x^2 + 2*x - 3
//         // f(0) = -3
//         // f(1) = 0
//         // f(2) = 5
//         // f(3) = 12
//         let coeffs = vec![
//             ModularInteger::<u32>::new(2_),
//             ModularInteger::<u32>::new(3).neg(),
//         ];
//         assert_eq!(
//             arithmetic::eval(&coeffs, 0),
//             ModularInteger::<u32>::new(3).neg()
//         );
//         assert_eq!(arithmetic::eval(&coeffs, 1), 0);
//         assert_eq!(arithmetic::eval(&coeffs, 2), 5);
//         assert_eq!(arithmetic::eval(&coeffs, 3), 12);
//     }

//     #[cfg(feature = "power_table")]
//     #[test]
//     fn test_eval_no_modulus_precompute() {
//         // f(x) = x^2 + 2*x - 3
//         // f(0) = -3
//         // f(1) = 0
//         // f(2) = 5
//         // f(3) = 12
//         crate::quack_internal::init_pow_table();
//         let coeffs = vec![
//             ModularInteger::<u16>::new(2_),
//             ModularInteger::<u16>::new(3).neg(),
//         ];
//         assert_eq!(
//             arithmetic::eval_precompute(&coeffs, 0),
//             ModularInteger::<u16>::new(3).neg()
//         );
//         assert_eq!(arithmetic::eval_precompute(&coeffs, 1), 0);
//         assert_eq!(arithmetic::eval_precompute(&coeffs, 2), 5);
//         assert_eq!(arithmetic::eval_precompute(&coeffs, 3), 12);
//     }

//     #[test]
//     fn test_eval_with_modulus() {
//         let coeffs = vec![
//             ModularInteger::<u32>::new(2539233112),
//             ModularInteger::<u32>::new(2884903207),
//             ModularInteger::<u32>::new(3439674878),
//         ];

//         // Test zeros.
//         assert_eq!(arithmetic::eval(&coeffs, 95976998), 0);
//         assert_eq!(arithmetic::eval(&coeffs, 456975625), 0);
//         assert_eq!(arithmetic::eval(&coeffs, 1202781556), 0);

//         // Test other points.
//         assert_ne!(arithmetic::eval(&coeffs, 2315971647), 0);
//         assert_ne!(arithmetic::eval(&coeffs, 3768947911), 0);
//         assert_ne!(arithmetic::eval(&coeffs, 1649073968), 0);
//     }

//     #[cfg(feature = "libpari")]
//     #[test]
//     fn test_factor() {
//         // f(x) = x^2 + 2*x - 3
//         // f(x) = 0 when x = -3, 1
//         let coeffs = vec![
//             ModularInteger::<u32>::new(2),
//             ModularInteger::<u32>::new(3).neg(),
//         ];
//         let mut roots = arithmetic::factor(&coeffs).unwrap();
//         assert_eq!(roots.len(), 2);
//         roots.sort();
//         let modulus = ModularInteger::<u32>::modulus();
//         assert_eq!(roots, vec![1, modulus - 3]);
//     }
// }
