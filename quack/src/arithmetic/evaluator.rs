use crate::arithmetic::ModularInteger;

pub struct MonicPolynomialEvaluator {
}

impl MonicPolynomialEvaluator {
    /// Evaluate the univariate polynomial with the given coefficients using
    /// modular arithmetic, assuming all coefficients are modulo the same
    /// 32-bit prime. In the coefficient vector, the last element is the
    /// constant term in the polynomial. The leading coefficient is 1, and is
    /// not included in the vector.
    pub fn eval(coeffs: &Vec<ModularInteger>, x: u32) -> ModularInteger {
        let size = coeffs.len();
        let x_mod = ModularInteger::new(x);
        let mut result = x_mod;
        // result = x(...(x(x(x+a0)+a1)+...))
        // e.g., result = x(x+a0)+a1
        for i in 0..(size - 1) {
            result += coeffs[i];
            result *= x_mod;
        }
        result + coeffs[size - 1]
    }
}
