use crate::arithmetic::{MonicPolynomialEvaluator, MontgomeryInteger};
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::{Sub, SubAssign};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MontgomeryQuack {
    // https://serde.rs/attr-skip-serializing.html
    #[serde(skip)]
    inverse_table: Vec<MontgomeryInteger>,
    power_sums: Vec<MontgomeryInteger>,
    last_value: MontgomeryInteger,
    count: u32,
}

impl MontgomeryQuack {
    pub fn new(size: usize) -> Self {
        debug!("new quACK of size {}", size);

        // The i-th term corresponds to dividing by i+1 in modular arithemtic.
        let inverse_table = (0..(size as u64))
            .map(|i| MontgomeryInteger::new_do_conversion(i + 1).inv())
            .collect();
        Self {
            inverse_table,
            power_sums: (0..size).map(|_| MontgomeryInteger::zero()).collect(),
            last_value: MontgomeryInteger::zero(),
            count: 0,
        }
    }

    pub fn insert(&mut self, value: u64) {
        trace!("insert {}", value);
        let size = self.power_sums.len();
        let x = MontgomeryInteger::new(value);
        let mut y = x;
        for i in 0..(size - 1) {
            self.power_sums[i] += y;
            y *= x;
        }
        self.power_sums[size - 1] += y;
        // TODO: handle count overflow
        self.count += 1;
        self.last_value = x;
    }

    pub fn remove(&mut self, value: u64) {
        trace!("remove {}", value);
        let size = self.power_sums.len();
        let x = MontgomeryInteger::new(value);
        let mut y = x;
        for i in 0..(size - 1) {
            self.power_sums[i] -= y;
            y *= x;
        }
        self.power_sums[size - 1] -= y;
        // TODO: handle count overflow
        self.count -= 1;
    }

    pub fn last_value(&self) -> u64 {
        self.last_value.value()
    }

    pub fn threshold(&self) -> usize {
        self.power_sums.len()
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    /// Returns the missing identifiers from the log. Note that if there are
    /// collisions in the log of multiple identifiers, they will all appear.
    /// If the log is incomplete, there will be fewer than the number missing.
    pub fn decode_with_log(&self, log: &Vec<u64>) -> Vec<u64> {
        let num_packets = log.len();
        let num_missing = self.count();
        info!(
            "decoding quACK: num_packets={}, num_missing={}",
            num_packets, num_missing
        );
        if num_missing == 0 {
            return vec![];
        }
        let coeffs = self.to_coeffs();
        trace!("coeffs = {:?}", coeffs);
        let missing: Vec<u64> = log
            .iter()
            .filter(|&&x| MonicPolynomialEvaluator::eval_montgomery(&coeffs, x).is_zero())
            .map(|&x| x)
            .collect();
        info!("found {}/{} missing packets", missing.len(), num_missing);
        debug!("missing = {:?}", missing);
        missing
    }

    /// Convert n power sums to n polynomial coefficients (not including the
    /// leading 1 coefficient) using Newton's identities.
    pub fn to_coeffs(&self) -> Vec<MontgomeryInteger> {
        let mut coeffs = (0..self.count())
            .map(|_| MontgomeryInteger::zero())
            .collect::<Vec<_>>();
        self.to_coeffs_preallocated(&mut coeffs);
        coeffs
    }

    /// Convert n power sums to n polynomial coefficients (not including the
    /// leading 1 coefficient) using Newton's identities. Writes coefficients
    /// into a pre-allocated buffer.
    pub fn to_coeffs_preallocated(&self, coeffs: &mut Vec<MontgomeryInteger>) {
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

impl SubAssign for MontgomeryQuack
where
    MontgomeryInteger: SubAssign + Copy,
{
    fn sub_assign(&mut self, rhs: Self) {
        assert_eq!(
            self.power_sums.len(),
            rhs.power_sums.len(),
            "expected subtracted quacks to have the same number of sums"
        );
        // TODO: actually, subtraction with underflow should be allowed in case
        // the count overflowed in the original quACK.
        assert!(self.count >= rhs.count, "subtract count with overflow");
        let size = self.power_sums.len();
        for i in 0..size {
            self.power_sums[i] -= rhs.power_sums[i];
        }
        self.count -= rhs.count;
        self.last_value = rhs.last_value;
    }
}

impl Sub for MontgomeryQuack
where
    MontgomeryQuack: SubAssign,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result -= rhs;
        result
    }
}
