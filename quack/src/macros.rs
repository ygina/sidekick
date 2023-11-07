#![allow(unused_macros)]

/// Enables code for Montgomery multiplication.
/// Use this macro instead of `cfg(montgomery)` to generate docs properly.
macro_rules! cfg_montgomery {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "montgomery", doc))]
            #[doc(cfg(feature = "montgomery"))]
            $item
        )*
    }
}

/// Enables code for the power table optimization.
/// Use this macro instead of `cfg(power_table)` to generate docs properly.
macro_rules! cfg_power_table {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "power_table", doc))]
            #[doc(cfg(feature = "power_table"))]
            $item
        )*
    }
}

/// Enables code for factoring to solve polynomials.
/// Use this macro instead of `cfg(libpari)` to generate docs properly.
macro_rules! cfg_libpari {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "libpari", doc))]
            #[doc(cfg(feature = "libpari"))]
            $item
        )*
    }
}

/// Enables code for strawmen quACKs.
/// Use this macro instead of `cfg(strawmen)` to generate docs properly.
macro_rules! cfg_strawmen {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "strawmen", doc))]
            #[doc(cfg(feature = "strawmen"))]
            $item
        )*
    }
}
