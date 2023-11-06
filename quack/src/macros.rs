#![allow(unused_macros)]

/// Enables code for Montgomery multiplication.
/// Use this macro instead of `cfg(montgomery)` to generate docs properly.
macro_rules! cfg_montgomery {
    ($($item:item)*) => {
        $(
            #[cfg(any(montgomery, doc))]
            #[doc(cfg(montgomery))]
            $item
        )*
    }
}

/// Enables code for the power table optimization.
/// Use this macro instead of `cfg(power_table)` to generate docs properly.
macro_rules! cfg_power_table {
    ($($item:item)*) => {
        $(
            #[cfg(any(power_table, doc))]
            #[doc(cfg(power_table))]
            $item
        )*
    }
}

/// Enables code for factoring to solve polynomials.
/// Use this macro instead of `cfg(libpari)` to generate docs properly.
macro_rules! cfg_libpari {
    ($($item:item)*) => {
        $(
            #[cfg(any(libpari, doc))]
            #[doc(cfg(libpari))]
            $item
        )*
    }
}
