use crate::arithmetic::{self, MontgomeryInteger, ModularArithmetic, CoefficientVector};
use crate::PowerSumQuack;
use crate::MAX_THRESHOLD;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Multiplication by the `i`-th term corresponds to division by the integer
/// `i + 1` in modular arithmetic.
static INVERSE_TABLE_MONTGOMERY: Lazy<Vec<MontgomeryInteger>> = Lazy::new(|| {
    let mut inverse_table = Vec::new();
    let mut index = MontgomeryInteger::new_do_conversion(1);
    for _ in 0..unsafe { MAX_THRESHOLD } {
        inverse_table.push(index.inv());
        index.add_assign(MontgomeryInteger::new_do_conversion(1));
    }
    inverse_table
});

/// 64-bit power sum quACK using the Montgomery multiplication optimization.
///
/// Elements inserted into and removed from the quACK should already be in
/// Montgomery form. Any elements of type [MontgomeryInteger](MontgomeryInteger)
/// read from the quACK are also assumed to have a value in Montgomery form.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MontgomeryQuack {
    power_sums: Vec<MontgomeryInteger>,
    last_value: Option<MontgomeryInteger>,
    count: u32,
}

impl PowerSumQuack for MontgomeryQuack {
    type Element = u64;
    type ModularElement = MontgomeryInteger;

    fn new(size: usize) -> Self {
        Self {
            power_sums: (0..size).map(|_| MontgomeryInteger::new(0)).collect(),
            last_value: None,
            count: 0,
        }
    }

    fn threshold(&self) -> usize {
        self.power_sums.len()
    }

    fn count(&self) -> u32 {
        self.count
    }

    fn last_value(&self) -> Option<Self::Element>  {
        self.last_value.map(|value| value.value())
    }

    fn insert(&mut self, value: Self::Element) {
        let size = self.power_sums.len();
        let x = MontgomeryInteger::new(value);
        let mut y = x;
        for i in 0..(size - 1) {
            self.power_sums[i].add_assign(y);
            y.mul_assign(x);
        }
        self.power_sums[size - 1].add_assign(y);
        self.count = self.count.wrapping_add(1);
        self.last_value = Some(x);
    }

    fn remove(&mut self, value: Self::Element) {
        let size = self.power_sums.len();
        let x = MontgomeryInteger::new(value);
        let mut y = x;
        for i in 0..(size - 1) {
            self.power_sums[i].sub_assign(y);
            y.mul_assign(x);
        }
        self.power_sums[size - 1].sub_assign(y);
        self.count = self.count.wrapping_sub(1);
        if let Some(last_value) = self.last_value {
            if last_value.value() == value {
                self.last_value = None;
            }
        }
    }

    fn decode_with_log(&self, log: &[u64]) -> Vec<u64> {
        if self.count() == 0 {
            return log.to_vec();
        }
        assert!((self.count() as usize) <= self.threshold(), "number of elements must not exceed threshold");
        let coeffs = self.to_coeffs();
        log.iter()
            .filter(|&&x| arithmetic::eval_montgomery(&coeffs, x).value() == 0)
            .copied()
            .collect()
    }

    fn to_coeffs(&self) -> CoefficientVector<Self::ModularElement> {
        let mut coeffs = (0..self.count())
            .map(|_| MontgomeryInteger::new(0))
            .collect::<Vec<_>>();
        self.to_coeffs_preallocated(&mut coeffs);
        coeffs
    }

    fn to_coeffs_preallocated(&self, coeffs: &mut CoefficientVector<Self::ModularElement>) {
        if coeffs.is_empty() {
            return;
        }
        assert_eq!(coeffs.len(), self.count() as usize, "length of coefficient vector must be the same as the number of elements");
        coeffs[0] = self.power_sums[0].neg();
        for i in 1..coeffs.len() {
            for j in 0..i {
                coeffs[i] = coeffs[i].sub(self.power_sums[j].mul(coeffs[i - j - 1]));
            }
            coeffs[i].sub_assign(self.power_sums[i]);
            coeffs[i].mul_assign(INVERSE_TABLE_MONTGOMERY[i]);
        }
    }

    fn sub_assign(&mut self, rhs: Self) {
        assert_eq!(
            self.threshold(),
            rhs.threshold(),
            "expected subtracted quacks to have the same threshold"
        );
        for (i, sum) in self.power_sums.iter_mut().enumerate() {
            sum.sub_assign(rhs.power_sums[i]);
        }
        self.count = self.count.wrapping_sub(rhs.count);
        self.last_value = None;
    }

    fn sub(self, rhs: Self) -> Self {
        let mut result = self;
        result.sub_assign(rhs);
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const THRESHOLD: usize = 3;

    #[test]
    fn test_quack_constructor() {
        let quack = MontgomeryQuack::new(THRESHOLD);
        assert_eq!(quack.threshold(), THRESHOLD);
        assert_eq!(quack.count(), 0);
        assert_eq!(quack.last_value(), None);
    }

    #[test]
    fn test_quack_insert_and_remove() {
        let mut quack = MontgomeryQuack::new(THRESHOLD);
        quack.insert(10);
        assert_eq!(quack.count(), 1);
        assert_eq!(quack.last_value(), Some(10));
        quack.insert(20);
        quack.insert(30);
        assert_eq!(quack.count(), 3);
        assert_eq!(quack.last_value(), Some(30));
        quack.remove(10);
        assert_eq!(quack.count(), 2);
        assert_eq!(quack.last_value(), Some(30));
        quack.remove(30);
        assert_eq!(quack.count(), 1);
        assert_eq!(quack.last_value(), None);
    }

    #[test]
    fn test_quack_to_coeffs_empty() {
        let quack = MontgomeryQuack::new(THRESHOLD);
        assert_eq!(quack.to_coeffs(), CoefficientVector::<MontgomeryInteger>::new());
        let mut coeffs = vec![];
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, CoefficientVector::<MontgomeryInteger>::new());
    }

    #[test]
    fn test_quack_to_coeffs_small() {
        const R1: u64 = 1;
        const R2: u64 = 2;

        let mut quack = MontgomeryQuack::new(THRESHOLD);
        quack.insert(MontgomeryInteger::new_do_conversion(R1).value());
        quack.insert(MontgomeryInteger::new_do_conversion(R2).value());
        let expected = vec![
            MontgomeryInteger::new_do_conversion(R1 + R2).neg().value(),
            MontgomeryInteger::new_do_conversion(R1 * R2).value(),
        ]; // x^2 - 3x + 2

        assert_eq!(quack.to_coeffs(), expected);
        let mut coeffs = (0..quack.count()).map(|_| MontgomeryInteger::new(0)).collect();
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, expected);
    }

    #[test]
    fn test_quack_to_coeffs_big() {
        const R1: u128 = 3616712547361671254;
        const R2: u128 = 2333013068233301306;
        const R3: u128 = 2234311686223431168;
        let modulus = MontgomeryInteger::modulus_big();

        let mut quack = MontgomeryQuack::new(THRESHOLD);
        quack.insert(MontgomeryInteger::new_do_conversion(R1 as u64).value());
        quack.insert(MontgomeryInteger::new_do_conversion(R2 as u64).value());
        quack.insert(MontgomeryInteger::new_do_conversion(R3 as u64).value());
        let expected = vec![
            MontgomeryInteger::new_do_conversion(((R1 + R2 + R3) % modulus) as u64)
                .neg()
                .value(),
            MontgomeryInteger::new_do_conversion(((R1 * R2 % modulus + R2 * R3 + R1 * R3) % modulus) as u64)
                .value(),
            MontgomeryInteger::new_do_conversion(((((R1 * R2) % modulus) * R3) % modulus) as u64)
                .neg()
                .value(),
        ];

        assert_eq!(quack.to_coeffs(), expected);
        let mut coeffs = (0..quack.count()).map(|_| MontgomeryInteger::new(0)).collect();
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, expected);
    }

    #[test]
    fn test_decode_empty() {
        let quack = MontgomeryQuack::new(THRESHOLD);
        assert_eq!(quack.decode_with_log(&[]), Vec::<u64>::new());
        assert_eq!(quack.decode_with_log(&[1]), vec![1]);
    }

    #[test]
    fn test_insert_and_decode() {
        const R1: u64 = 3616712547361671254;
        const R2: u64 = 2333013068233301306;
        const R3: u64 = 2234311686223431168;
        const R4: u64 = 448751902448751902;
        const R5: u64 = 918748965918748965;

        let mut quack = MontgomeryQuack::new(THRESHOLD);
        quack.insert(R1);
        quack.insert(R2);
        quack.insert(R3);

        // different orderings
        // NOTE: the spec of `decode_with_log` doesn't guarantee an order but
        // here we assume the elements appear in the same order as the list.
        assert_eq!(quack.decode_with_log(&[R1, R2, R3]), vec![R1, R2, R3]);
        assert_eq!(quack.decode_with_log(&[R3, R1, R2]), vec![R3, R1, R2]);

        // one extra element in log
        assert_eq!(quack.decode_with_log(&[R1, R2, R3, R4]), vec![R1, R2, R3]);
        assert_eq!(quack.decode_with_log(&[R1, R4, R2, R3]), vec![R1, R2, R3]);
        assert_eq!(quack.decode_with_log(&[R4, R1, R2, R3]), vec![R1, R2, R3]);

        // two extra elements in log
        assert_eq!(
            quack.decode_with_log(&[R1, R5, R2, R3, R4]),
            vec![R1, R2, R3]
        );

        // not all roots are in log
        assert_eq!(quack.decode_with_log(&[R1, R2]), vec![R1, R2]);
        assert_eq!(quack.decode_with_log(&[]), Vec::<u64>::new());
        assert_eq!(quack.decode_with_log(&[R1, R2, R4]), vec![R1, R2]);
    }

    #[test]
    fn test_remove_and_decode() {
        const R1: u64 = 3616712547;
        const R2: u64 = 2333013068;
        const R3: u64 = 2234311686;
        const R4: u64 = 448751902;
        const R5: u64 = 918748965;

        let mut quack = MontgomeryQuack::new(THRESHOLD);
        quack.insert(R5);
        quack.insert(R4);
        quack.insert(R3);
        quack.insert(R2);
        quack.insert(R1);
        quack.remove(R5);
        quack.remove(R4);

        // R4 and R5 are removed, and R1,2,3 are able to be decoded.
        // NOTE: the spec of `decode_with_log` doesn't guarantee an order but
        // here we assume the elements appear in the same order as the list.
        assert_eq!(quack.decode_with_log(&[R1, R2, R3]), vec![R1, R2, R3]);
        assert_eq!(
            quack.decode_with_log(&[R1, R5, R2, R3, R4]),
            vec![R1, R2, R3]
        );
    }

    #[test]
    fn test_decode_with_multiplicity() {
        const R1: u64 = 10;
        const R2: u64 = 20;

        let mut quack = MontgomeryQuack::new(THRESHOLD);
        quack.insert(R1);
        quack.insert(R1);

        assert_eq!(quack.decode_with_log(&[R1, R1]), vec![R1, R1]);
        assert_eq!(quack.decode_with_log(&[R1]), vec![R1]);
        assert_eq!(quack.decode_with_log(&[R1, R1, R1]), vec![R1, R1, R1]); // even though one R1 is not in the quACK
        assert_eq!(quack.decode_with_log(&[R1, R1, R2]), vec![R1, R1]);
        assert_eq!(quack.decode_with_log(&[R2, R1, R2]), vec![R1]);
    }

    #[test]
    fn test_subtract_quacks_with_zero_difference() {
        let mut q1 = MontgomeryQuack::new(THRESHOLD);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        q1.insert(4);
        q1.insert(5);

        let quack = q1.clone().sub(q1);
        assert_eq!(quack.threshold(), THRESHOLD);
        assert_eq!(quack.count(), 0);
        assert_eq!(quack.last_value(), None);
        assert_eq!(quack.to_coeffs(), CoefficientVector::<MontgomeryInteger>::new());
    }

    #[test]
    fn test_subtract_quacks_with_nonzero_difference() {
        let mut q1 = MontgomeryQuack::new(THRESHOLD);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        q1.insert(4);
        q1.insert(5);

        let mut q2 = MontgomeryQuack::new(THRESHOLD);
        q2.insert(1);
        q2.insert(2);

        let quack = q1.sub(q2);
        assert_eq!(quack.threshold(), THRESHOLD);
        assert_eq!(quack.count(), 3);
        assert_eq!(quack.last_value(), None);
        assert_eq!(quack.to_coeffs().len(), 3);
        assert_eq!(quack.decode_with_log(&[1, 2, 3, 4, 5]), vec![3, 4, 5]);
    }
}
