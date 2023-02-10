use std::ops::{Sub, SubAssign};

use crate::Quack;
use crate::Identifier;

pub struct MontgomeryQuack {
    count: u16,
}

impl Quack for MontgomeryQuack {
    fn new(threshold: usize) -> Self {
        unimplemented!()
    }

    fn insert(&mut self, value: Identifier) {
        unimplemented!()
    }

    fn remove(&mut self, value: Identifier) {
        unimplemented!()
    }

    fn threshold(&self) -> usize {
        unimplemented!()
    }

    fn count(&self) -> u16 {
        unimplemented!()
    }
}

impl SubAssign for MontgomeryQuack {
    fn sub_assign(&mut self, rhs: Self) {
        unimplemented!()
    }
}

impl Sub for MontgomeryQuack {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self;
        result -= rhs;
        result
    }
}
