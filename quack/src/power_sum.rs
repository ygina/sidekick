use crate::arithmetic::{self, CoefficientVector, ModularArithmetic, ModularInteger};
use crate::precompute::INVERSE_TABLE_U32;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

cfg_power_table! {
    use crate::precompute::INVERSE_TABLE_U16;
}

cfg_montgomery! {
    use crate::precompute::INVERSE_TABLE_U64;
}

/// 32-bit power sum quACK.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerSumQuackU32 {
    power_sums: Vec<ModularInteger<u32>>,
    last_value: Option<ModularInteger<u32>>,
    count: u32,
}

/// A quACK represented by a threshold number of power sums.
///
/// The power sum quACK is useful for decoding a set difference of elements
/// when the number of elements in the set difference is comparatively small
/// to the number of elements in either set. It is also efficient to insert
/// elements in the power sum quACK. The tradeoff is that it becomes impossible
/// to decode the quACK when the number of elements in the quACK exceeds a
/// pre-determined threshold. The number of bytes needed to transmit the quACK
/// over the wire is proportional to this threshold.
///
/// The underlying representation of a power sum quACK is a `threshold` number
/// of power sums. If `X` is the multiset of elements in the quACK, then the
/// `i`-th power sum is just the sum of `x^i` for all `x` in `X`.
pub trait PowerSumQuack {
    /// The type of element that can be inserted in the quACK.
    type Element;

    /// The modular version of the elements in the quACK.
    type ModularElement;

    /// Creates a new power sum quACK that can decode at most `threshold`
    /// number of elements.
    fn new(threshold: usize) -> Self;

    /// The maximum number of elements that can be decoded by the quACK.
    fn threshold(&self) -> usize;

    /// The number of elements represented by the quACK.
    fn count(&self) -> u32;

    /// The last element inserted in the quACK, if known.
    ///
    /// If `None`, either there are no elements in the quACK, or a previous last
    /// element was removed and the actual last element is unknown.
    fn last_value(&self) -> Option<Self::Element>;

    /// Insert an element in the quACK.
    fn insert(&mut self, value: Self::Element);

    /// Remove an element in the quACK. Does not validate that the element
    /// had actually been inserted in the quACK.
    fn remove(&mut self, value: Self::Element);

    /// Decode the elements in the log that in the quACK.
    ///
    /// This method evaluates the polynomial derived from the power sums in the
    /// quACK at each of the candidate roots in the log, returning the roots.
    /// If a root appears more than once in the log, it will appear the same
    /// number of times in the returned roots. Note that the decoding method
    /// does not consider the root multiplicity in the polynomial. If the log is
    /// incomplete, there will be fewer roots returned than the actual number of
    /// elements represented by the quACK.
    fn decode_with_log(&self, log: &[Self::Element]) -> Vec<Self::Element>;

    /// Convert the `n` modular power sums that represent the elements in the
    /// quACK to a degree-`n` polynomial in the same field. The polynomial is
    /// represented by a vector of coefficients, and the coefficients are
    /// calculated using [Newton's identities](https://en.wikipedia.org/wiki/Newton%27s_identities).
    ///
    /// # Examples
    ///
    /// ```
    /// use quack::{PowerSumQuack, PowerSumQuackU32};
    /// use quack::arithmetic::{ModularInteger, ModularArithmetic};
    ///
    /// const THRESHOLD: usize = 20;
    /// const ROOT1: u32 = 10;
    /// const ROOT2: u32 = 12;
    ///
    /// fn main() {
    ///     // Polynomial with degree 1
    ///     let mut quack = PowerSumQuackU32::new(THRESHOLD);
    ///     quack.insert(ROOT1);
    ///     let coeffs = quack.to_coeffs();  // x - 10
    ///     assert_eq!(coeffs.len(), 1);
    ///     assert_eq!(coeffs, vec![ModularInteger::<u32>::new(ROOT1).neg()]);
    ///
    ///     // Polynomial with degree 2
    ///     quack.insert(ROOT2);
    ///     let coeffs = quack.to_coeffs();  // x^2 - 24x + 120
    ///     let mut quack1 = PowerSumQuackU32::new(THRESHOLD);
    ///     assert_eq!(coeffs.len(), 2);
    ///     assert_eq!(coeffs, vec![
    ///         ModularInteger::<u32>::new(ROOT1 + ROOT2).neg(),
    ///         ModularInteger::<u32>::new(ROOT1 * ROOT2),
    ///     ]);
    /// }
    /// ```
    fn to_coeffs(&self) -> CoefficientVector<Self::ModularElement>;

    /// Similar to [to_coeffs](trait.PowerSumQuack.html#method.to_coeffs)
    /// but reuses the same vector allocation to return the coefficients.
    fn to_coeffs_preallocated(&self, coeffs: &mut CoefficientVector<Self::ModularElement>);

    /// Subtracts another power sum quACK from this power sum quACK.
    ///
    /// The difference between a quACK with `x` elements and a quACK with `y`
    /// elements is a quACK with `x - y` elements. Assumes the elements in the
    /// second quACK are a subset of the elements in the first quACK. Assumes
    /// the two quACKs have the same threshold. If these conditions are met,
    /// then the `x - y` elements in the difference represent the set
    /// difference, and can be decoded from the quACK as long as this number of
    /// elements does not exceed the threshold.
    ///
    /// # Examples
    ///
    /// ```
    /// use quack::{PowerSumQuack, PowerSumQuackU32};
    /// use quack::arithmetic::{ModularInteger, ModularArithmetic};
    ///
    /// const THRESHOLD: usize = 20;
    ///
    /// fn main() {
    ///     // Insert some elements in the first quACK.
    ///     let mut quack1 = PowerSumQuackU32::new(THRESHOLD);
    ///     quack1.insert(1);
    ///     quack1.insert(2);
    ///     quack1.insert(3);
    ///     quack1.insert(4);
    ///     quack1.insert(5);
    ///
    ///     // Insert a subset of the same elements in the second quACK.
    ///     let mut quack2 = PowerSumQuackU32::new(THRESHOLD);
    ///     quack2.insert(2);
    ///     quack2.insert(5);
    ///
    ///     // Subtract the second quACK from the first and decode the elements.
    ///     quack1.sub_assign(quack2);
    ///     // let mut roots = quack1.decode_with_log(&[1, 2, 3, 4, 5]);
    ///     // roots.sort();
    ///     // assert_eq!(quack1.count(), 3);
    ///     // assert_eq!(roots, vec![1, 3, 4]);
    /// }
    /// ```
    fn sub_assign(&mut self, rhs: Self);

    /// Similar to [sub_assign](trait.PowerSumQuack.html#method.sub_assign)
    /// but returns the difference as a new quACK.
    fn sub(self, rhs: Self) -> Self;
}

impl PowerSumQuack for PowerSumQuackU32 {
    type Element = u32;
    type ModularElement = ModularInteger<Self::Element>;

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
        let x = ModularInteger::new(value);
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

    fn decode_with_log(&self, log: &[Self::Element]) -> Vec<Self::Element> {
        if self.count() == 0 {
            return log.to_vec();
        }
        let coeffs = self.to_coeffs();
        log.iter()
            .filter(|&&x| arithmetic::eval(&coeffs, x).value() == 0)
            .copied()
            .collect()
    }

    /// Convert the `n` modular power sums that represent the elements in the
    /// quACK to a degree-`n` polynomial in the same field. The polynomial is
    /// represented by a vector of coefficients, and the coefficients are
    /// calculated using [Newton's identities](https://en.wikipedia.org/wiki/Newton%27s_identities).
    ///
    /// # Examples
    ///
    /// ```
    /// use quack::{PowerSumQuack, PowerSumQuackU32};
    /// use quack::arithmetic::{ModularInteger, ModularArithmetic};
    ///
    /// const THRESHOLD: usize = 20;
    /// const ROOT1: u32 = 10;
    /// const ROOT2: u32 = 12;
    ///
    /// fn main() {
    ///     // Polynomial with degree 1
    ///     let mut quack = PowerSumQuackU32::new(THRESHOLD);
    ///     quack.insert(ROOT1);
    ///     let coeffs = quack.to_coeffs();  // x - 10
    ///     assert_eq!(coeffs.len(), 1);
    ///     assert_eq!(coeffs, vec![ModularInteger::<u32>::new(ROOT1).neg()]);
    ///
    ///     // Polynomial with degree 2
    ///     quack.insert(ROOT2);
    ///     let coeffs = quack.to_coeffs();  // x^2 - 24x + 120
    ///     let mut quack1 = PowerSumQuackU32::new(THRESHOLD);
    ///     assert_eq!(coeffs.len(), 2);
    ///     assert_eq!(coeffs, vec![
    ///         ModularInteger::<u32>::new(ROOT1 + ROOT2).neg(),
    ///         ModularInteger::<u32>::new(ROOT1 * ROOT2),
    ///     ]);
    /// }
    /// ```
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
            coeffs[i].mul_assign(INVERSE_TABLE_U32[i]);
        }
    }

    /// Subtracts another power sum quACK from this power sum quACK.
    ///
    /// The difference between a quACK with `x` elements and a quACK with `y`
    /// elements is a quACK with `x - y` elements. Assumes the elements in the
    /// second quACK are a subset of the elements in the first quACK. Assumes
    /// the two quACKs have the same threshold. If these conditions are met,
    /// then the `x - y` elements in the difference represent the set
    /// difference, and can be decoded from the quACK as long as this number of
    /// elements does not exceed the threshold.
    ///
    /// # Examples
    ///
    /// ```
    /// use quack::{PowerSumQuack, PowerSumQuackU32};
    /// use quack::arithmetic::{ModularInteger, ModularArithmetic};
    ///
    /// const THRESHOLD: usize = 20;
    ///
    /// fn main() {
    ///     // Insert some elements in the first quACK.
    ///     let mut quack1 = PowerSumQuackU32::new(THRESHOLD);
    ///     quack1.insert(1);
    ///     quack1.insert(2);
    ///     quack1.insert(3);
    ///     quack1.insert(4);
    ///     quack1.insert(5);
    ///
    ///     // Insert a subset of the same elements in the second quACK.
    ///     let mut quack2 = PowerSumQuackU32::new(THRESHOLD);
    ///     quack2.insert(2);
    ///     quack2.insert(5);
    ///
    ///     // Subtract the second quACK from the first and decode the elements.
    ///     quack1.sub_assign(quack2);
    ///     // let mut roots = quack1.decode_with_log(&[1, 2, 3, 4, 5]);
    ///     // roots.sort();
    ///     // assert_eq!(roots, vec![1, 3, 4]);
    /// }
    /// ```
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

cfg_libpari! {
    impl PowerSumQuackU32 {
        /// Decode the elements in the quACK by factorization.
        ///
        /// Returns `n` integer roots from the degree-`n` polynomial represented
        /// by the `n` elements in the quACK. Returns None if unable to factor,
        /// or if any of the roots are not real.
        pub fn decode_by_factorization(&self) -> Option<Vec<u32>> {
            if self.count == 0 {
                return Some(vec![]);
            }
            let coeffs = self.to_coeffs();
            match arithmetic::factor(&coeffs) {
                Ok(roots) => Some(roots),
                Err(_) => None,
            }
        }
    }
}

cfg_montgomery! {
    /// 64-bit power sum quACK.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PowerSumQuackU64 {
        power_sums: Vec<ModularInteger<u64>>,
        last_value: Option<ModularInteger<u64>>,
        count: u32,
    }

    impl PowerSumQuack for PowerSumQuackU64 {
        type Element = u64;
        type ModularElement = ModularInteger<Self::Element>;

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
            let x = ModularInteger::new(value);
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

        fn decode_with_log(&self, log: &[Self::Element]) -> Vec<Self::Element> {
            if self.count() == 0 {
                return log.to_vec();
            }
            let coeffs = self.to_coeffs();
            log.iter()
                .filter(|&&x| arithmetic::eval(&coeffs, x).value() == 0)
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
                coeffs[i].mul_assign(INVERSE_TABLE_U64[i]);
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
}

cfg_power_table! {
    /// 16-bit power sum quACK.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PowerSumQuackU16 {
        power_sums: Vec<ModularInteger<u16>>,
        last_value: Option<ModularInteger<u16>>,
        count: u32,
    }

    impl PowerSumQuack for PowerSumQuackU16 {
        type Element = u16;
        type ModularElement = ModularInteger<Self::Element>;

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
            let x = ModularInteger::new(value);
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

        fn decode_with_log(&self, log: &[Self::Element]) -> Vec<Self::Element> {
            if self.count() == 0 {
                return log.to_vec();
            }
            assert!((self.count() as usize) <= self.threshold(), "number of elements must not exceed threshold");
            let coeffs = self.to_coeffs();
            log.iter()
                .filter(|&&x| arithmetic::eval(&coeffs, x).value() == 0)
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
            assert_eq!(coeffs.len(), self.count() as usize, "length of coefficient vector must be the same as the number of elements");
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
}

#[cfg(test)]
mod test {
    use super::*;

    const THRESHOLD: usize = 3;

    #[test]
    fn test_quack_constructor_u32() {
        let quack = PowerSumQuackU32::new(THRESHOLD);
        assert_eq!(quack.threshold(), THRESHOLD);
        assert_eq!(quack.count(), 0);
        assert_eq!(quack.last_value(), None);
    }

    #[test]
    fn test_quack_insert_and_remove_u32() {
        let mut quack = PowerSumQuackU32::new(THRESHOLD);
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
    fn test_quack_to_coeffs_empty_u32() {
        let quack = PowerSumQuackU32::new(THRESHOLD);
        assert_eq!(
            quack.to_coeffs(),
            CoefficientVector::<ModularInteger<u32>>::new()
        );
        let mut coeffs = vec![];
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, CoefficientVector::<ModularInteger<u32>>::new());
    }

    #[test]
    fn test_quack_to_coeffs_small_u32() {
        const R1: u32 = 1;
        const R2: u32 = 2;

        let mut quack = PowerSumQuackU32::new(THRESHOLD);
        quack.insert(R1);
        quack.insert(R2);
        let expected = vec![
            ModularInteger::<u32>::new(R1 + R2).neg().value(),
            ModularInteger::<u32>::new(R1 * R2).value(),
        ]; // x^2 - 3x + 2

        assert_eq!(quack.to_coeffs(), expected);
        let mut coeffs = (0..quack.count()).map(|_| ModularInteger::new(0)).collect();
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, expected);
    }

    #[test]
    fn test_quack_to_coeffs_big_u32() {
        const R1: u64 = 3616712547;
        const R2: u64 = 2333013068;
        const R3: u64 = 2234311686;
        let modulus = ModularInteger::<u32>::modulus_big();

        let mut quack = PowerSumQuackU32::new(THRESHOLD);
        quack.insert(R1 as u32);
        quack.insert(R2 as u32);
        quack.insert(R3 as u32);
        let expected = vec![
            ModularInteger::<u32>::new(((R1 + R2 + R3) % modulus) as u32)
                .neg()
                .value(),
            ModularInteger::<u32>::new(((R1 * R2 % modulus + R2 * R3 + R1 * R3) % modulus) as u32)
                .value(),
            ModularInteger::<u32>::new(((((R1 * R2) % modulus) * R3) % modulus) as u32)
                .neg()
                .value(),
        ];

        assert_eq!(quack.to_coeffs(), expected);
        let mut coeffs = (0..quack.count()).map(|_| ModularInteger::new(0)).collect();
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, expected);
    }

    #[test]
    fn test_decode_empty_u32() {
        let quack = PowerSumQuackU32::new(THRESHOLD);
        assert_eq!(quack.decode_with_log(&[]), vec![]);
        assert_eq!(quack.decode_with_log(&[1]), vec![1]);
    }

    #[test]
    fn test_insert_and_decode_u32() {
        const R1: u32 = 3616712547;
        const R2: u32 = 2333013068;
        const R3: u32 = 2234311686;
        const R4: u32 = 448751902;
        const R5: u32 = 918748965;

        let mut quack = PowerSumQuackU32::new(THRESHOLD);
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
    fn test_remove_and_decode_u32() {
        const R1: u32 = 3616712547;
        const R2: u32 = 2333013068;
        const R3: u32 = 2234311686;
        const R4: u32 = 448751902;
        const R5: u32 = 918748965;

        let mut quack = PowerSumQuackU32::new(THRESHOLD);
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
    fn test_decode_with_multiplicity_u32() {
        const R1: u32 = 10;
        const R2: u32 = 20;

        let mut quack = PowerSumQuackU32::new(THRESHOLD);
        quack.insert(R1);
        quack.insert(R1);

        assert_eq!(quack.decode_with_log(&[R1, R1]), vec![R1, R1]);
        assert_eq!(quack.decode_with_log(&[R1]), vec![R1]);
        assert_eq!(quack.decode_with_log(&[R1, R1, R1]), vec![R1, R1, R1]); // even though one R1 is not in the quACK
        assert_eq!(quack.decode_with_log(&[R1, R1, R2]), vec![R1, R1]);
        assert_eq!(quack.decode_with_log(&[R2, R1, R2]), vec![R1]);
    }

    #[test]
    fn test_subtract_quacks_with_zero_difference_u32() {
        let mut q1 = PowerSumQuackU32::new(THRESHOLD);
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
            CoefficientVector::<ModularInteger<u32>>::new()
        );
    }

    #[test]
    fn test_subtract_quacks_with_nonzero_difference_u32() {
        let mut q1 = PowerSumQuackU32::new(THRESHOLD);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        q1.insert(4);
        q1.insert(5);

        let mut q2 = PowerSumQuackU32::new(THRESHOLD);
        q2.insert(1);
        q2.insert(2);

        let quack = q1.sub(q2);
        assert_eq!(quack.threshold(), THRESHOLD);
        assert_eq!(quack.count(), 3);
        assert_eq!(quack.last_value(), None);
        assert_eq!(quack.to_coeffs().len(), 3);
        assert_eq!(quack.decode_with_log(&[1, 2, 3, 4, 5]), vec![3, 4, 5]);
    }

    #[test]
    #[ignore]
    fn test_quack_serialize_u32() {
        let mut quack = PowerSumQuackU32::new(10);
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
    fn test_quack_deserialize_empty_u32() {
        let q1 = PowerSumQuackU32::new(10);
        let bytes = bincode::serialize(&q1).unwrap();
        let q2: PowerSumQuackU32 = bincode::deserialize(&bytes).unwrap();
        assert_eq!(q1.count(), q2.count());
        assert_eq!(q1.to_coeffs(), q2.to_coeffs());
    }

    #[test]
    fn test_quack_deserialize_with_data_u32() {
        let mut q1 = PowerSumQuackU32::new(10);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        let bytes = bincode::serialize(&q1).unwrap();
        let q2: PowerSumQuackU32 = bincode::deserialize(&bytes).unwrap();
        assert_eq!(q1.count(), q2.count());
        assert_eq!(q1.to_coeffs(), q2.to_coeffs());
    }

    #[ignore]
    #[cfg(feature = "libpari")]
    #[test]
    fn test_decode_factor_empty_quack_u32() {
        let quack = PowerSumQuackU32::new(10);
        let result = quack.decode_by_factorization();
        assert!(result.is_some());
        assert!(result.unwrap().is_empty());
    }

    #[ignore]
    #[cfg(feature = "libpari")]
    #[test]
    fn test_quack_decode_factor_u32() {
        let log = vec![1, 2, 3, 4, 5, 6];
        let mut q1 = PowerSumQuackU32::new(3);
        for x in &log {
            q1.insert(*x);
        }
        let mut q2 = PowerSumQuackU32::new(3);
        q2.insert(1);
        q2.insert(3);
        q2.insert(4);

        // Check the result
        let quack = q1.sub(q2);
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
    fn test_quack_decode_cant_factor_u32() {
        let log = vec![1, 2, 3, 4, 5, 6];
        let mut q1 = PowerSumQuackU32::new(3);
        for x in &log {
            q1.insert(*x);
        }
        let mut q2 = PowerSumQuackU32::new(3);
        q2.insert(1);
        q2.insert(3);
        q2.insert(4);
        q2.power_sums[0].add_assign(ModularInteger::new(1)); // mess up the power sums

        // Check the result
        let quack = q1.sub(q2);
        let result = quack.decode_by_factorization();
        assert!(result.is_none());
    }

    #[test]
    #[cfg(feature = "power_table")]
    fn test_quack_constructor_u16() {
        let quack = PowerSumQuackU16::new(THRESHOLD);
        assert_eq!(quack.threshold(), THRESHOLD);
        assert_eq!(quack.count(), 0);
        assert_eq!(quack.last_value(), None);
    }

    #[test]
    #[cfg(feature = "power_table")]
    fn test_quack_insert_and_remove_u16() {
        let mut quack = PowerSumQuackU16::new(THRESHOLD);
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
    #[cfg(feature = "power_table")]
    fn test_quack_to_coeffs_empty_u16() {
        let quack = PowerSumQuackU16::new(THRESHOLD);
        assert_eq!(
            quack.to_coeffs(),
            CoefficientVector::<ModularInteger<u16>>::new()
        );
        let mut coeffs = vec![];
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, CoefficientVector::<ModularInteger<u16>>::new());
    }

    #[test]
    #[cfg(feature = "power_table")]
    fn test_quack_to_coeffs_small_u16() {
        const R1: u16 = 1;
        const R2: u16 = 2;

        let mut quack = PowerSumQuackU16::new(THRESHOLD);
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
    #[cfg(feature = "power_table")]
    fn test_quack_to_coeffs_big_u16() {
        const R1: u32 = 36167;
        const R2: u32 = 23330;
        const R3: u32 = 22343;
        let modulus = ModularInteger::<u16>::modulus_big();

        let mut quack = PowerSumQuackU16::new(THRESHOLD);
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
    #[cfg(feature = "power_table")]
    fn test_decode_empty_u16() {
        let quack = PowerSumQuackU16::new(THRESHOLD);
        assert_eq!(quack.decode_with_log(&[]), vec![]);
        assert_eq!(quack.decode_with_log(&[1]), vec![1]);
    }

    #[test]
    #[cfg(feature = "power_table")]
    fn test_insert_and_decode_u16() {
        const R1: u16 = 36167;
        const R2: u16 = 23330;
        const R3: u16 = 22343;
        const R4: u16 = 44875;
        const R5: u16 = 9187;

        let mut quack = PowerSumQuackU16::new(THRESHOLD);
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
    #[cfg(feature = "power_table")]
    fn test_remove_and_decode_u16() {
        const R1: u16 = 36167;
        const R2: u16 = 23330;
        const R3: u16 = 22343;
        const R4: u16 = 44875;
        const R5: u16 = 9187;

        let mut quack = PowerSumQuackU16::new(THRESHOLD);
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
    #[cfg(feature = "power_table")]
    fn test_decode_with_multiplicity_u16() {
        const R1: u16 = 10;
        const R2: u16 = 20;

        let mut quack = PowerSumQuackU16::new(THRESHOLD);
        quack.insert(R1);
        quack.insert(R1);

        assert_eq!(quack.decode_with_log(&[R1, R1]), vec![R1, R1]);
        assert_eq!(quack.decode_with_log(&[R1]), vec![R1]);
        assert_eq!(quack.decode_with_log(&[R1, R1, R1]), vec![R1, R1, R1]); // even though one R1 is not in the quACK
        assert_eq!(quack.decode_with_log(&[R1, R1, R2]), vec![R1, R1]);
        assert_eq!(quack.decode_with_log(&[R2, R1, R2]), vec![R1]);
    }

    #[test]
    #[cfg(feature = "power_table")]
    fn test_subtract_quacks_with_zero_difference_u16() {
        let mut q1 = PowerSumQuackU16::new(THRESHOLD);
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
    #[cfg(feature = "power_table")]
    fn test_subtract_quacks_with_nonzero_difference_u16() {
        let mut q1 = PowerSumQuackU16::new(THRESHOLD);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        q1.insert(4);
        q1.insert(5);

        let mut q2 = PowerSumQuackU16::new(THRESHOLD);
        q2.insert(1);
        q2.insert(2);

        let quack = q1.sub(q2);
        assert_eq!(quack.threshold(), THRESHOLD);
        assert_eq!(quack.count(), 3);
        assert_eq!(quack.last_value(), None);
        assert_eq!(quack.to_coeffs().len(), 3);
        assert_eq!(quack.decode_with_log(&[1, 2, 3, 4, 5]), vec![3, 4, 5]);
    }

    #[test]
    #[cfg(feature = "montgomery")]
    fn test_quack_constructor_u64() {
        let quack = PowerSumQuackU64::new(THRESHOLD);
        assert_eq!(quack.threshold(), THRESHOLD);
        assert_eq!(quack.count(), 0);
        assert_eq!(quack.last_value(), None);
    }

    #[test]
    #[cfg(feature = "montgomery")]
    fn test_quack_insert_and_remove_u64() {
        let mut quack = PowerSumQuackU64::new(THRESHOLD);
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
    #[cfg(feature = "montgomery")]
    fn test_quack_to_coeffs_empty_u64() {
        let quack = PowerSumQuackU64::new(THRESHOLD);
        assert_eq!(
            quack.to_coeffs(),
            CoefficientVector::<ModularInteger<u64>>::new()
        );
        let mut coeffs = vec![];
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, CoefficientVector::<ModularInteger<u64>>::new());
    }

    #[test]
    #[cfg(feature = "montgomery")]
    fn test_quack_to_coeffs_small_u64() {
        const R1: u64 = 1;
        const R2: u64 = 2;

        let mut quack = PowerSumQuackU64::new(THRESHOLD);
        quack.insert(R1);
        quack.insert(R2);
        let expected = vec![
            ModularInteger::<u64>::new(R1 + R2).neg().value(),
            ModularInteger::<u64>::new(R1 * R2).value(),
        ]; // x^2 - 3x + 2

        assert_eq!(quack.to_coeffs(), expected);
        let mut coeffs = (0..quack.count()).map(|_| ModularInteger::new(0)).collect();
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, expected);
    }

    #[test]
    #[cfg(feature = "montgomery")]
    fn test_quack_to_coeffs_big_u64() {
        const R1: u128 = 3616712547361671254;
        const R2: u128 = 2333013068233301306;
        const R3: u128 = 2234311686223431168;
        let modulus = ModularInteger::<u64>::modulus_big();

        let mut quack = PowerSumQuackU64::new(THRESHOLD);
        quack.insert(R1 as u64);
        quack.insert(R2 as u64);
        quack.insert(R3 as u64);
        let expected = vec![
            ModularInteger::<u64>::new(((R1 + R2 + R3) % modulus) as u64)
                .neg()
                .value(),
            ModularInteger::<u64>::new(((R1 * R2 % modulus + R2 * R3 + R1 * R3) % modulus) as u64)
                .value(),
            ModularInteger::<u64>::new(((((R1 * R2) % modulus) * R3) % modulus) as u64)
                .neg()
                .value(),
        ];

        assert_eq!(quack.to_coeffs(), expected);
        let mut coeffs = (0..quack.count()).map(|_| ModularInteger::new(0)).collect();
        quack.to_coeffs_preallocated(&mut coeffs);
        assert_eq!(coeffs, expected);
    }

    #[test]
    #[cfg(feature = "montgomery")]
    fn test_decode_empty_u64() {
        let quack = PowerSumQuackU64::new(THRESHOLD);
        assert_eq!(quack.decode_with_log(&[]), Vec::<u64>::new());
        assert_eq!(quack.decode_with_log(&[1]), vec![1]);
    }

    #[test]
    #[cfg(feature = "montgomery")]
    fn test_insert_and_decode_u64() {
        const R1: u64 = 3616712547361671254;
        const R2: u64 = 2333013068233301306;
        const R3: u64 = 2234311686223431168;
        const R4: u64 = 448751902448751902;
        const R5: u64 = 918748965918748965;

        let mut quack = PowerSumQuackU64::new(THRESHOLD);
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
    #[cfg(feature = "montgomery")]
    fn test_remove_and_decode_u64() {
        const R1: u64 = 3616712547;
        const R2: u64 = 2333013068;
        const R3: u64 = 2234311686;
        const R4: u64 = 448751902;
        const R5: u64 = 918748965;

        let mut quack = PowerSumQuackU64::new(THRESHOLD);
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
    #[cfg(feature = "montgomery")]
    fn test_decode_with_multiplicity_u64() {
        const R1: u64 = 10;
        const R2: u64 = 20;

        let mut quack = PowerSumQuackU64::new(THRESHOLD);
        quack.insert(R1);
        quack.insert(R1);

        assert_eq!(quack.decode_with_log(&[R1, R1]), vec![R1, R1]);
        assert_eq!(quack.decode_with_log(&[R1]), vec![R1]);
        assert_eq!(quack.decode_with_log(&[R1, R1, R1]), vec![R1, R1, R1]); // even though one R1 is not in the quACK
        assert_eq!(quack.decode_with_log(&[R1, R1, R2]), vec![R1, R1]);
        assert_eq!(quack.decode_with_log(&[R2, R1, R2]), vec![R1]);
    }

    #[test]
    #[cfg(feature = "montgomery")]
    fn test_subtract_quacks_with_zero_difference_u64() {
        let mut q1 = PowerSumQuackU64::new(THRESHOLD);
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
            CoefficientVector::<ModularInteger<u64>>::new()
        );
    }

    #[test]
    #[cfg(feature = "montgomery")]
    fn test_subtract_quacks_with_nonzero_difference_u64() {
        let mut q1 = PowerSumQuackU64::new(THRESHOLD);
        q1.insert(1);
        q1.insert(2);
        q1.insert(3);
        q1.insert(4);
        q1.insert(5);

        let mut q2 = PowerSumQuackU64::new(THRESHOLD);
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
