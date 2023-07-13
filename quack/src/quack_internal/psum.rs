use std::ops::{Sub, SubAssign, MulAssign, AddAssign};
use std::fmt::{Debug, Display};
use crate::arithmetic::{
    ModularArithmetic,
    ModularInteger,
    MonicPolynomialEvaluator,
};
use crate::Quack;
use serde::{Serialize, Deserialize};
use log::{debug, info, trace};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerSumQuack<T> where ModularInteger<T>: ModularArithmetic<T> {
    // https://serde.rs/attr-skip-serializing.html
    #[serde(skip)]
    inverse_table: Vec<ModularInteger<T>>,
    power_sums: Vec<ModularInteger<T>>,
    last_value: ModularInteger<T>,
    count: u32,
}

impl<T> Quack<T> for PowerSumQuack<T>
where T: Debug + Display + Default + PartialOrd + Sub<Output = T> + Copy,
ModularInteger<T>: ModularArithmetic<T> + AddAssign + MulAssign + SubAssign {
    fn new(size: usize) -> Self {
        debug!("new quACK of size {}", size);

        // The i-th term corresponds to dividing by i+1 in modular arithemtic.
        let mut inverse_table = Vec::new();
        let mut index = ModularInteger::one();
        for _ in 0..size {
            inverse_table.push(index.inv());
            index += ModularInteger::one();
        }
        Self {
            inverse_table,
            power_sums: (0..size).map(|_| ModularInteger::zero()).collect(),
            last_value: ModularInteger::zero(),
            count: 0,
        }
    }

    fn insert(&mut self, value: T) {
        trace!("insert {}", value);
        let size = self.power_sums.len();
        let x = ModularInteger::<T>::new(value);
        let mut y = x;
        for i in 0..(size-1) {
            self.power_sums[i] += y;
            y *= x;
        }
        self.power_sums[size - 1] += y;
        // TODO: handle count overflow
        self.count += 1;
        self.last_value = x;
    }

    fn remove(&mut self, value: T) {
        trace!("remove {}", value);
        let size = self.power_sums.len();
        let x = ModularInteger::<T>::new(value);
        let mut y = x;
        for i in 0..(size-1) {
            self.power_sums[i] -= y;
            y *= x;
        }
        self.power_sums[size - 1] -= y;
        // TODO: handle count overflow
        self.count -= 1;
    }

    fn last_value(&self) -> T {
        self.last_value.value()
    }

    fn threshold(&self) -> usize {
        self.power_sums.len()
    }

    fn count(&self) -> u32 {
        self.count
    }

    /// Returns the missing identifiers from the log. Note that if there are
    /// collisions in the log of multiple identifiers, they will all appear.
    /// If the log is incomplete, there will be fewer than the number missing.
    fn decode_with_log(&self, log: &Vec<T>) -> Vec<T> {
        let num_packets = log.len();
        let num_missing = self.count();
        info!("decoding quACK: num_packets={}, num_missing={}",
            num_packets, num_missing);
        if num_missing == 0 {
            return vec![];
        }
        let coeffs = self.to_coeffs();
        trace!("coeffs = {:?}", coeffs);
        let missing: Vec<T> = log.iter()
            .filter(|&&x| {
                MonicPolynomialEvaluator::eval(&coeffs, x).is_zero()
            })
            .map(|&x| x)
            .collect();
        info!("found {}/{} missing packets", missing.len(), num_missing);
        debug!("missing = {:?}", missing);
        missing
    }

    /// Convert n power sums to n polynomial coefficients (not including the
    /// leading 1 coefficient) using Newton's identities.
    fn to_coeffs(&self) -> Vec<ModularInteger<T>> {
        let mut coeffs = (0..self.count())
            .map(|_| ModularInteger::zero())
            .collect::<Vec<_>>();
        self.to_coeffs_preallocated(&mut coeffs);
        coeffs
    }

    /// Convert n power sums to n polynomial coefficients (not including the
    /// leading 1 coefficient) using Newton's identities. Writes coefficients
    /// into a pre-allocated buffer.
    fn to_coeffs_preallocated(
        &self,
        coeffs: &mut Vec<ModularInteger<T>>,
    ) {
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

impl<T> SubAssign for PowerSumQuack<T> where
ModularInteger<T>: ModularArithmetic<T> + SubAssign + Copy {
    fn sub_assign(&mut self, rhs: Self) {
        assert_eq!(self.power_sums.len(), rhs.power_sums.len(),
            "expected subtracted quacks to have the same number of sums");
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

impl<T> Sub for PowerSumQuack<T> where
PowerSumQuack<T>: SubAssign, ModularInteger<T>: ModularArithmetic<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result -= rhs;
        result
    }
}

#[cfg(feature = "libpari")]
impl PowerSumQuack<u32> {
    /// Returns the missing identifiers by factorization of the difference
    /// quack. Returns None if unable to factor.
    pub fn decode_by_factorization(&self) -> Option<Vec<u32>> {
        if self.count == 0 {
            return Some(vec![]);
        }
        let coeffs = self.to_coeffs();
        match MonicPolynomialEvaluator::factor(&coeffs) {
            Ok(roots) => Some(roots),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_quack_constructor() {
        let size = 3;
        let quack = PowerSumQuack::<u32>::new(size);
        assert_eq!(quack.count, 0);
        assert_eq!(quack.power_sums.len(), size);
        for i in 0..size {
            assert_eq!(quack.power_sums[i], 0);
        }
    }

    #[test]
    fn test_quack_insert_no_modulus() {
        let mut quack = PowerSumQuack::<u32>::new(3);
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
        let mut quack = PowerSumQuack::<u32>::new(5);
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
        let mut quack = PowerSumQuack::<u32>::new(5);
        quack.insert(3616712547);
        quack.insert(2333013068);
        quack.insert(2234311686);
        quack.insert(2462729946);
        quack.insert(670144905);
        let mut coeffs = (0..5).map(|_| ModularInteger::<u32>::zero()).collect();
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs.len(), 5);
        assert_eq!(coeffs, vec![
            1567989721, 1613776244, 517289688, 17842621, 3562381446,
        ]);
    }

    #[test]
    #[should_panic]
    fn test_quack_sub_with_underflow() {
        let mut q1 = PowerSumQuack::<u32>::new(3);
        q1.insert(1);
        q1.insert(2);
        let mut q2 = PowerSumQuack::<u32>::new(3);
        q2.insert(1);
        q2.insert(2);
        q2.insert(3);
        let _ = q1 - q2;
    }

    #[test]
    #[should_panic]
    fn test_quack_sub_with_diff_thresholds() {
        let mut q1 = PowerSumQuack::<u32>::new(3);
        q1.insert(1);
        q1.insert(2);
        let mut q2 = PowerSumQuack::<u32>::new(2);
        q2.insert(1);
        q2.insert(2);
        let _ = q1 - q2;
    }

    #[test]
    fn test_quack_sub_num_missing_eq_threshold() {
        let mut coeffs = (0..3).map(|_| ModularInteger::<u32>::zero()).collect();
        let mut q1 = PowerSumQuack::<u32>::new(3);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        q1.insert(4);
        q1.insert(5);

        let quack = q1.clone() - q1.clone();
        assert_eq!(quack.count, 0);
        assert_eq!(quack.power_sums, vec![0, 0, 0]);
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, vec![0, 0, 0]);
    }

    #[test]
    fn test_quack_sub_num_missing_lt_threshold() {
        let mut coeffs = (0..3).map(|_| ModularInteger::<u32>::zero()).collect();
        let mut q1 = PowerSumQuack::<u32>::new(3);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        q1.insert(4);
        q1.insert(5);
        let mut q2 = PowerSumQuack::<u32>::new(3);
        q2.insert(1);
        q2.insert(2);
        q2.insert(3);

        // Missing 2 with threshold 3
        let quack = q1 - q2;
        assert_eq!(quack.count, 2);
        assert_eq!(quack.power_sums, vec![9, 41, 189]);
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, vec![4294967282, 20, 0]);
    }

    #[test]
    #[ignore]
    fn test_quack_serialize() {
        let mut quack = PowerSumQuack::<u32>::new(10);
        let bytes = bincode::serialize(&quack).unwrap();
        // expected length is 4*10+2 = 42 bytes (ten u32 sums and a u16 count)
        // TODO: extra 8 bytes from bincode
        assert_eq!(bytes.len(), 42);
        assert_eq!(&bytes[..], &[0; 42], "no data yet");
        quack.insert(1);
        quack.insert(2);
        quack.insert(3);
        let bytes = bincode::serialize(&quack).unwrap();
        assert_eq!(bytes.len(), 42);
        assert_ne!(&bytes[..], &[0; 42]);
    }

    #[test]
    fn test_quack_deserialize_empty() {
        let q1 = PowerSumQuack::<u32>::new(10);
        let bytes = bincode::serialize(&q1).unwrap();
        let q2: PowerSumQuack<u32> = bincode::deserialize(&bytes).unwrap();
        assert_eq!(q1.count, q2.count);
        assert_eq!(q1.power_sums, q2.power_sums);
    }

    #[test]
    fn test_quack_deserialize_with_data() {
        let mut q1 = PowerSumQuack::<u32>::new(10);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        let bytes = bincode::serialize(&q1).unwrap();
        let q2: PowerSumQuack<u32> = bincode::deserialize(&bytes).unwrap();
        assert_eq!(q1.count, q2.count);
        assert_eq!(q1.power_sums, q2.power_sums);
    }

    #[test]
    fn test_decode_log_empty_quack() {
        let quack = PowerSumQuack::<u32>::new(10);
        let log = vec![1, 2, 3];
        let result = quack.decode_with_log(&log);
        assert!(result.is_empty());
    }

    #[test]
    fn test_quack_decode_log() {
        let log = vec![1, 2, 3, 4, 5, 6];
        let mut q1 = PowerSumQuack::<u32>::new(3);
        for x in &log {
            q1.insert(*x);
        }
        let mut q2 = PowerSumQuack::<u32>::new(3);
        q2.insert(1);
        q2.insert(3);
        q2.insert(4);

        // Check the result
        let quack = q1 - q2;
        let mut result = quack.decode_with_log(&log);
        assert_eq!(result.len(), 3);
        result.sort();
        assert_eq!(result, vec![2, 5, 6]);
    }

    #[test]
    fn test_quack_decode_log_with_collisions() {
        let log = vec![1, 2, 2, 3, 4, 5, 6];
        let mut q1 = PowerSumQuack::<u32>::new(4);
        for x in &log {
            q1.insert(*x);
        }
        let mut q2 = PowerSumQuack::<u32>::new(4);
        q2.insert(1);
        q2.insert(3);
        q2.insert(4);

        // Check the result
        let quack = q1 - q2;
        let mut result = quack.decode_with_log(&log);
        assert_eq!(result.len(), 4);
        result.sort();
        assert_eq!(result, vec![2, 2, 5, 6]);
    }

    #[test]
    fn test_quack_decode_log_incomplete() {
        let log = vec![1, 2, 3, 4, 5, 6];
        let mut q1 = PowerSumQuack::<u32>::new(3);
        for x in &log {
            q1.insert(*x);
        }
        let mut q2 = PowerSumQuack::<u32>::new(3);
        q2.insert(1);
        q2.insert(3);
        q2.insert(4);

        // Check the result
        let quack = q1 - q2;
        let mut result = quack.decode_with_log(&log[2..].to_vec());
        assert_eq!(result.len(), 2);
        result.sort();
        assert_eq!(result, vec![5, 6]);
    }

    #[ignore]
    #[cfg(feature = "libpari")]
    #[test]
    fn test_decode_factor_empty_quack() {
        let quack = PowerSumQuack::<u32>::new(10);
        let result = quack.decode_by_factorization();
        assert!(result.is_some());
        assert!(result.unwrap().is_empty());
    }

    #[ignore]
    #[cfg(feature = "libpari")]
    #[test]
    fn test_quack_decode_factor() {
        let log = vec![1, 2, 3, 4, 5, 6];
        let mut q1 = PowerSumQuack::<u32>::new(3);
        for x in &log {
            q1.insert(*x);
        }
        let mut q2 = PowerSumQuack::<u32>::new(3);
        q2.insert(1);
        q2.insert(3);
        q2.insert(4);

        // Check the result
        let quack = q1 - q2;
        let result = quack.decode_by_factorization();
        assert!(result.is_some());
        let mut result = result.unwrap();
        assert_eq!(result.len(), 3);
        result.sort();
        assert_eq!(result, vec![2, 5, 6]);
    }

    #[ignore]
    #[cfg(feature = "libpari")]
    #[test]
    fn test_quack_decode_cant_factor() {
        let log = vec![1, 2, 3, 4, 5, 6];
        let mut q1 = PowerSumQuack::<u32>::new(3);
        for x in &log {
            q1.insert(*x);
        }
        let mut q2 = PowerSumQuack::<u32>::new(3);
        q2.insert(1);
        q2.insert(3);
        q2.insert(4);
        q2.power_sums[0] += ModularInteger::new(1);  // mess up the power sums

        // Check the result
        let quack = q1 - q2;
        let mut result = quack.decode_by_factorization();
        assert!(result.is_none());
    }
}
