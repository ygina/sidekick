use crate::arithmetic::{self, ModularArithmetic, ModularInteger, CoefficientVector};
use crate::precompute::{INVERSE_TABLE_U16, POWER_TABLE};
use crate::PowerSumQuack;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// 16-bit power sum quACK using the precomputation optimization.
///
/// The optimization precomputes the first few powers of integers in the 16-bit
/// prime field. The number of powers computed can be set by the
/// [global_config_set_max_power_sum_threshold](fn.global_config_set_max_power_sum_threshold.html)
/// function. The optimization improves the performance of insertion, removal,
/// and decoding by avoiding the need to compute powers on the fly. Precomputing
/// powers becomes less feasible in terms of memory and less cache-friendly at
/// larger bit widths.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerTableQuack {
    power_sums: Vec<ModularInteger<u16>>,
    last_value: Option<ModularInteger<u16>>,
    count: u32,
}

impl PowerSumQuack for PowerTableQuack {
    type Element = u16;
    type ModularElement = ModularInteger<u16>;

    fn new(threshold: usize) -> Self {
        Self {
            power_sums: (0..threshold).map(|_| ModularInteger::new(0)).collect(),
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

    fn last_value(&self) -> Option<Self::Element> {
        self.last_value.map(|value| value.value())
    }

    fn insert(&mut self, value: Self::Element) {
        let size = self.power_sums.len();
        let x = ModularInteger::new(value);
        for i in 0..size {
            self.power_sums[i].add_assign(
                POWER_TABLE[x.value() as usize][i + 1]);
        }
        self.count = self.count.wrapping_add(1);
        self.last_value = Some(x);
    }

    fn remove(&mut self, value: Self::Element) {
        let size = self.power_sums.len();
        let x = ModularInteger::<Self::Element>::new(value);
        for i in 0..size {
            self.power_sums[i].sub_assign(
                POWER_TABLE[x.value() as usize][i + 1]);
        }
        self.count = self.count.wrapping_sub(1);
        if let Some(last_value) = self.last_value {
            if last_value.value() == value {
                self.last_value = None;
            }
        }
    }

    fn decode_with_log(&self, log: &[Self::Element]) -> Vec<Self::Element> {
        if self.count() == 0 {
            return log.to_vec();
        }
        let coeffs = self.to_coeffs();
        log.iter()
            .filter(|&&x| arithmetic::eval_precompute(&coeffs, x).value() == 0)
            .copied()
            .collect()
    }

    fn to_coeffs(&self) -> CoefficientVector<Self::ModularElement> {
        let mut coeffs = (0..self.count())
            .map(|_| ModularInteger::new(0))
            .collect::<Vec<_>>();
        self.to_coeffs_preallocated(&mut coeffs);
        coeffs
    }

    fn to_coeffs_preallocated(&self, coeffs: &mut CoefficientVector<Self::ModularElement>) {
        if coeffs.is_empty() {
            return;
        }
        coeffs[0] = self.power_sums[0].neg();
        for i in 1..coeffs.len() {
            for j in 0..i {
                coeffs[i] = coeffs[i].sub(self.power_sums[j].mul(coeffs[i - j - 1]));
            }
            coeffs[i].sub_assign(self.power_sums[i]);
            coeffs[i].mul_assign(INVERSE_TABLE_U16[i]);
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
    fn test_quack_constructor_u16() {
        let quack = PowerTableQuack::new(THRESHOLD);
        assert_eq!(quack.threshold(), THRESHOLD);
        assert_eq!(quack.count(), 0);
        assert_eq!(quack.last_value(), None);
    }

    #[test]
    fn test_quack_insert_and_remove_u16() {
        let mut quack = PowerTableQuack::new(THRESHOLD);
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
    fn test_quack_to_coeffs_empty_u16() {
        let quack = PowerTableQuack::new(THRESHOLD);
        assert_eq!(
            quack.to_coeffs(),
            CoefficientVector::<ModularInteger<u16>>::new()
        );
        let mut coeffs = vec![];
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, CoefficientVector::<ModularInteger<u16>>::new());
    }

    #[test]
    fn test_quack_to_coeffs_small_u16() {
        const R1: u16 = 1;
        const R2: u16 = 2;

        let mut quack = PowerTableQuack::new(THRESHOLD);
        quack.insert(R1);
        quack.insert(R2);
        let expected = vec![
            ModularInteger::<u16>::new(R1 + R2).neg().value(),
            ModularInteger::<u16>::new(R1 * R2).value(),
        ]; // x^2 - 3x + 2

        assert_eq!(quack.to_coeffs(), expected);
        let mut coeffs = (0..quack.count()).map(|_| ModularInteger::new(0)).collect();
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, expected);
    }

    #[test]
    fn test_quack_to_coeffs_big_u16() {
        const R1: u32 = 36167;
        const R2: u32 = 23330;
        const R3: u32 = 22343;
        let modulus = ModularInteger::<u16>::modulus_big();

        let mut quack = PowerTableQuack::new(THRESHOLD);
        quack.insert(R1 as u16);
        quack.insert(R2 as u16);
        quack.insert(R3 as u16);
        let expected = vec![
            ModularInteger::<u16>::new(((R1 + R2 + R3) % modulus) as u16)
                .neg()
                .value(),
            ModularInteger::<u16>::new(((R1 * R2 % modulus + R2 * R3 + R1 * R3) % modulus) as u16)
                .value(),
            ModularInteger::<u16>::new(((((R1 * R2) % modulus) * R3) % modulus) as u16)
                .neg()
                .value(),
        ];

        assert_eq!(quack.to_coeffs(), expected);
        let mut coeffs = (0..quack.count()).map(|_| ModularInteger::new(0)).collect();
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, expected);
    }

    #[test]
    fn test_decode_empty_u16() {
        let quack = PowerTableQuack::new(THRESHOLD);
        assert_eq!(quack.decode_with_log(&[]), vec![]);
        assert_eq!(quack.decode_with_log(&[1]), vec![1]);
    }

    #[test]
    fn test_insert_and_decode_u16() {
        const R1: u16 = 36167;
        const R2: u16 = 23330;
        const R3: u16 = 22343;
        const R4: u16 = 44875;
        const R5: u16 = 9187;

        let mut quack = PowerTableQuack::new(THRESHOLD);
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
        assert_eq!(quack.decode_with_log(&[]), vec![]);
        assert_eq!(quack.decode_with_log(&[R1, R2, R4]), vec![R1, R2]);
    }

    #[test]
    fn test_remove_and_decode_u16() {
        const R1: u16 = 36167;
        const R2: u16 = 23330;
        const R3: u16 = 22343;
        const R4: u16 = 44875;
        const R5: u16 = 9187;

        let mut quack = PowerTableQuack::new(THRESHOLD);
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
    fn test_decode_with_multiplicity_u16() {
        const R1: u16 = 10;
        const R2: u16 = 20;

        let mut quack = PowerTableQuack::new(THRESHOLD);
        quack.insert(R1);
        quack.insert(R1);

        assert_eq!(quack.decode_with_log(&[R1, R1]), vec![R1, R1]);
        assert_eq!(quack.decode_with_log(&[R1]), vec![R1]);
        assert_eq!(quack.decode_with_log(&[R1, R1, R1]), vec![R1, R1, R1]); // even though one R1 is not in the quACK
        assert_eq!(quack.decode_with_log(&[R1, R1, R2]), vec![R1, R1]);
        assert_eq!(quack.decode_with_log(&[R2, R1, R2]), vec![R1]);
    }

    #[test]
    fn test_subtract_quacks_with_zero_difference_u16() {
        let mut q1 = PowerTableQuack::new(THRESHOLD);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        q1.insert(4);
        q1.insert(5);

        let quack = q1.clone().sub(q1);
        assert_eq!(quack.threshold(), THRESHOLD);
        assert_eq!(quack.count(), 0);
        assert_eq!(quack.last_value(), None);
        assert_eq!(
            quack.to_coeffs(),
            CoefficientVector::<ModularInteger<u16>>::new()
        );
    }

    #[test]
    fn test_subtract_quacks_with_nonzero_difference_u16() {
        let mut q1 = PowerTableQuack::new(THRESHOLD);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        q1.insert(4);
        q1.insert(5);

        let mut q2 = PowerTableQuack::new(THRESHOLD);
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
