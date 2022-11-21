use std::ops::SubAssign;
use crate::arithmetic::ModularInteger;

/// The i-th term corresponds to dividing by i+1 in modular arithemtic.
fn modular_inverse_table(size: usize) -> Vec<ModularInteger> {
    (0..(size as u32)).map(|i| ModularInteger::new(i+1).inv()).collect()
}

pub struct Quack {
    inverse_table: Vec<ModularInteger>,
    pub power_sums: Vec<ModularInteger>,
    pub count: u16,
}

impl Quack {
    pub fn new(size: usize) -> Self {
        Self {
            inverse_table: modular_inverse_table(size),
            power_sums: (0..size).map(|_| ModularInteger::zero()).collect(),
            count: 0,
        }
    }

    pub fn insert(&mut self, value: u32) {
        let size = self.power_sums.len();
        let x = ModularInteger::new(value);
        let mut y = x;
        for i in 0..(size-1) {
            self.power_sums[i] += y;
            y *= x;
        }
        self.power_sums[size - 1] += y;
        self.count += 1;
    }

    /// Convert n power sums to n polynomial coefficients (not including the
    /// leading 1 coefficient) using Newton's identities.
    pub fn to_polynomial_coefficients(self, coeffs: &mut Vec<ModularInteger>) {
        let size = coeffs.len();
        coeffs[0] = -self.power_sums[0];
        for i in 1..size {
            for j in 0..i {
                coeffs[i] = coeffs[i] - self.power_sums[j] * coeffs[i - j - 1];
            }
            coeffs[i] -= self.power_sums[i];
            coeffs[i] *= self.inverse_table[i];
        }
    }
}

impl SubAssign for Quack {
    fn sub_assign(&mut self, rhs: Self) {
        let size = self.power_sums.len();
        for i in 0..size {
            self.power_sums[i] -= rhs.power_sums[i];
        }
        self.count -= rhs.count;
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_power_sum_accumulator() {

    }
}
