//! Efficient multiplicative modular inverses.
use crate::arithmetic::{ModularArithmetic, ModularInteger};
use once_cell::sync::Lazy;

cfg_montgomery! {
    use crate::arithmetic::MontgomeryInteger;
}

/// The maximum number of multiplicative modular inverses that will be lazily
/// computed.
pub(crate) static mut MAX_THRESHOLD: usize = 20;

/// The multiplicative modular inverses of the integers up to this threshold
/// are lazily precomputed, for more efficient divison. This function MUST be
/// called before modifying any quACKs with a threshold greater than the default
/// threshold of `20`, and immediately precomputes the tables.
///
/// If this function is not called, the code may panic when trying to access a
/// modular inverse that is out of range. This function should also be called
/// if the known maximum threshold is less than the default, to improve cache
/// performance.
pub fn global_config_set_max_power_sum_threshold(threshold: usize) {
    unsafe {
        MAX_THRESHOLD = threshold;
    }

    // precompute all tables
    let _ = INVERSE_TABLE_U32[0];
    // cfg_montgomery! {
    //     let _ = INVERSE_TABLE_U64[0];
    //     let _ = INVERSE_TABLE_MONTGOMERY[0];
    // }
    // cfg_power_table! {
    //     let _ = INVERSE_TABLE_U16[0];
    //     let _ = POWER_TABLE[0];
    // }
}

/// Multiplication by the `i`-th term corresponds to division by the integer
/// `i + 1` in the field modulo the largest 32-bit prime.
pub static INVERSE_TABLE_U32: Lazy<Vec<ModularInteger<u32>>> = Lazy::new(|| {
    let mut inverse_table = Vec::new();
    let mut index = ModularInteger::new(1);
    for _ in 0..unsafe { MAX_THRESHOLD } {
        inverse_table.push(index.inv());
        index.add_assign(ModularInteger::new(1));
    }
    inverse_table
});

cfg_montgomery! {
    /// Multiplication by the `i`-th term corresponds to division by the integer
    /// `i + 1` in the field modulo the largest 64-bit prime.
    pub static INVERSE_TABLE_U64: Lazy<Vec<ModularInteger<u64>>> = Lazy::new(|| {
        let mut inverse_table = Vec::new();
        let mut index = ModularInteger::new(1);
        for _ in 0..unsafe { MAX_THRESHOLD } {
            inverse_table.push(index.inv());
            index.add_assign(ModularInteger::new(1));
        }
        inverse_table
    });

    /// Multiplication by the `i`-th term corresponds to division by the integer
    /// `i + 1` in the field modulo the largest 64-bit prime, using Montgomery
    /// modular multiplication.
    pub static INVERSE_TABLE_MONTGOMERY: Lazy<Vec<MontgomeryInteger>> = Lazy::new(|| {
        let mut inverse_table = Vec::new();
        let mut index = MontgomeryInteger::new_do_conversion(1);
        for _ in 0..unsafe { MAX_THRESHOLD } {
            inverse_table.push(index.inv());
            index.add_assign(MontgomeryInteger::new_do_conversion(1));
        }
        inverse_table
    });
}

cfg_power_table! {
    /// Multiplication by the `i`-th term corresponds to division by the integer
    /// `i + 1` in the field modulo the largest 16-bit prime.
    pub static INVERSE_TABLE_U16: Lazy<Vec<ModularInteger<u16>>> = Lazy::new(|| {
        let mut inverse_table = Vec::new();
        let mut index = ModularInteger::new(1);
        for _ in 0..unsafe { MAX_THRESHOLD } {
            inverse_table.push(index.inv());
            index.add_assign(ModularInteger::new(1));
        }
        inverse_table
    });

    pub static POWER_TABLE: Lazy<Vec<Vec<ModularInteger<u16>>>> = Lazy::new(|| {
        const NUM_U16S: usize = 1 << 16;
        let threshold: usize = unsafe { MAX_THRESHOLD + 1 };
        let mut power_table = vec![vec![ModularInteger::new(0); threshold]; NUM_U16S];
        for (x, row) in power_table.iter_mut().enumerate().take(NUM_U16S) {
            let x_mi = ModularInteger::new(x as u16);
            let mut xpow = ModularInteger::new(1);
            for cell in row.iter_mut().take(threshold) {
                *cell = xpow;
                xpow.mul_assign(x_mi);
            }
        }
        power_table
    });
}
