# QuACK

The _quACK_ is a data structure for being able to refer to and efficiently
acknowledge a set of opaque packets seen by a network intermediary.

## Overview

This crate contains the recommended 32-bit power sum quACK implementation and,
if feature-enabled, strawmen and power sum quACKs in different bit widths with
various optimizations.

* Build: `cargo +nightly build --release`
* Test: `cargo +nightly test --all-features`
* Documentation: `cargo +nightly doc --all-features`

The _power sum quACK_ is useful for decoding a set difference of elements when
the number of elements in the set difference is comparatively small to the
number of elements in either set. It is also efficient to insert elements in the
power sum quACK. The tradeoff is that it becomes impossible to decode the quACK
when the number of elements in the quACK exceeds a pre-determined threshold. The
number of bytes needed to transmit the quACK over the wire is proportional to
this threshold.

The underlying representation of a power sum quACK is a threshold number of
power sums. If `X` is the multiset of elements in the quACK, then the `i`-th
power sum is just the sum of `x^i` for all `x` in `X`.

See the [API docs](target/doc/quack/) for more info.

## Example

All power sum quACKS implement the same [PowerSumQuack]() trait and can be used
interchangeably in the following example:

```rust
use quack::{PowerSumQuack, PowerSumQuackU32};

// The threshold is the maximum number of elements that can be decoded.
const THRESHOLD: usize = 10;

fn main () {
    // Set the maximum threshold for lazy performance optimizations.
    quack::global_config_set_max_power_sum_threshold(THRESHOLD);

    // Insert some elements in the first quACK.
    let mut q1 = PowerSumQuackU32::new(THRESHOLD);
    q1.insert(1);
    q1.insert(2);
    q1.insert(3);
    q1.insert(4);
    q1.insert(5);

    // Insert a subset of the same elements in the second quACK.
    let mut q2 = PowerSumQuackU32::new(THRESHOLD);
    q2.insert(2);
    q2.insert(5);

    // Subtract the second quACK from the first and decode the elements.
    q1.sub_assign(q2);
    let mut roots = q1.decode_with_log(&[1, 2, 3, 4, 5]);
    roots.sort();
    assert_eq!(roots, vec![1, 3, 4]);
}
```

## Benchmark

There are three benchmarks for the various quACK and strawman implementations:

* `benchmark_construct`: Benchmark the time it takes to construct and serialize
a quACK(s) representing `n` received packets, and if applicable, a threshold
number of missing packets `t`.
* `benchmark_decode`: Benchmark the time it takes to decode the `m` missing
elements in a received quACK(s), given the `n` sent packets.
* `benchmark_construct_multi`: Benchmark the time it takes to construct and
serialize a quACK(s) when multiplexing quACKs across multiple connections using
a hash table.
