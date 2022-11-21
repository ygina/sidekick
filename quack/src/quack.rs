use std::ops::{Sub, SubAssign};
use crate::arithmetic::ModularInteger;

/// The i-th term corresponds to dividing by i+1 in modular arithemtic.
fn modular_inverse_table(size: usize) -> Vec<ModularInteger> {
    (0..(size as u32)).map(|i| ModularInteger::new(i+1).inv()).collect()
}

#[derive(Clone, Debug)]
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
        assert_eq!(self.power_sums.len(), rhs.power_sums.len(),
            "expected subtracted quacks to have the same number of sums");
        assert!(self.count >= rhs.count, "subtract count with overflow");
        let size = self.power_sums.len();
        for i in 0..size {
            self.power_sums[i] -= rhs.power_sums[i];
        }
        self.count -= rhs.count;
    }
}

impl Sub for Quack {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result -= rhs;
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_quack_constructor() {
        let size = 3;
        let quack = Quack::new(size);
        assert_eq!(quack.count, 0);
        assert_eq!(quack.power_sums.len(), size);
        for i in 0..size {
            assert_eq!(quack.power_sums[i], 0);
        }
    }

    #[test]
    fn test_quack_insert_no_modulus() {
        let mut quack = Quack::new(3);
        quack.insert(1);
        assert_eq!(quack.count, 1);
        assert_eq!(quack.power_sums, vec![1, 1, 1]);
        quack.insert(2);
        assert_eq!(quack.count, 2);
        assert_eq!(quack.power_sums, vec![3, 5, 9]);
        quack.insert(3);
        assert_eq!(quack.count, 3);
        assert_eq!(quack.power_sums, vec![6, 14, 36]);
    }

    #[test]
    fn test_quack_insert_with_modulus() {
        let mut quack = Quack::new(5);
        quack.insert(1143971604);
        quack.insert(734067013);
        quack.insert(130412990);
        quack.insert(2072080394);
        quack.insert(748120679);
        assert_eq!(quack.count, 5);
        assert_eq!(quack.power_sums, vec![
            533685389, 1847039354, 2727275532, 1272499396, 2347942976,
        ]);
    }

    #[test]
    fn test_quack_to_polynomial_coefficients() {
        let mut quack = Quack::new(5);
        quack.insert(3616712547);
        quack.insert(2333013068);
        quack.insert(2234311686);
        quack.insert(2462729946);
        quack.insert(670144905);
        let mut coeffs = (0..5).map(|_| ModularInteger::zero()).collect();
        quack.to_polynomial_coefficients(&mut coeffs);
        assert_eq!(coeffs.len(), 5);
        assert_eq!(coeffs, vec![
            1567989721, 1613776244, 517289688, 17842621, 3562381446,
        ]);
    }

    #[test]
    #[should_panic]
    fn test_quack_sub_with_underflow() {
        let mut q1 = Quack::new(3);
        q1.insert(1);
        q1.insert(2);
        let mut q2 = Quack::new(3);
        q2.insert(1);
        q2.insert(2);
        q2.insert(3);
        let _ = q1 - q2;
    }

    #[test]
    #[should_panic]
    fn test_quack_sub_with_diff_thresholds() {
        let mut q1 = Quack::new(3);
        q1.insert(1);
        q1.insert(2);
        let mut q2 = Quack::new(2);
        q2.insert(1);
        q2.insert(2);
        let _ = q1 - q2;
    }

    #[test]
    fn test_quack_sub_num_missing_eq_threshold() {
        let mut coeffs = (0..3).map(|_| ModularInteger::zero()).collect();
        let mut q1 = Quack::new(3);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        q1.insert(4);
        q1.insert(5);

        let quack = q1.clone() - q1.clone();
        assert_eq!(quack.count, 0);
        assert_eq!(quack.power_sums, vec![0, 0, 0]);
        quack.to_polynomial_coefficients(&mut coeffs);
        assert_eq!(coeffs, vec![0, 0, 0]);
    }

    #[test]
    fn test_quack_sub_num_missing_lt_threshold() {
        let mut coeffs = (0..3).map(|_| ModularInteger::zero()).collect();
        let mut q1 = Quack::new(3);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        q1.insert(4);
        q1.insert(5);
        let mut q2 = Quack::new(3);
        q2.insert(1);
        q2.insert(2);
        q2.insert(3);

        // Missing 2 with threshold 3
        let quack = q1 - q2;
        assert_eq!(quack.count, 2);
        assert_eq!(quack.power_sums, vec![9, 41, 189]);
        quack.to_polynomial_coefficients(&mut coeffs);
        assert_eq!(coeffs, vec![4294967282, 20, 0]);
    }
}
