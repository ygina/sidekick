const N: u64            = 9223372036854775783;
// const R: u64            = 9223372036854775808;
// const R_INV: u64         = 1106804644422573094;
const NEG_N_INV: u64      = 1106804644422573097;
const R_SQ_MOD_N: u64      = 625;
const R_LOG2: u64       = 63;
const R_MOD_MASK: u64     = (1 << 63) - 1;

// Helper to do 64 x 64 - > 128 multiplication
fn multiply_64(x: u64, y: u64) -> u128 {
    return (x as u128) * (y as u128);
}

// Montgomery form multiplication, addition (these are cheap!)
// from wiki https://en.wikipedia.org/wiki/Montgomery_modular_multiplication
fn montgomery_redc(x: u128) -> u64 {
    // Overflow here is OK because we're modding by a small power of two
    let m: u64 = (((x & (R_MOD_MASK as u128)) * (NEG_N_INV as u128)) as u64) & R_MOD_MASK;
    // // (x + (m * N)) is 63 bit + 126 bits => 127 bits, then downshift
    // // so should all work out
    let t: u64 = (((x as u128) + multiply_64(m, N)) >> (R_LOG2 as u128)) as u64;
    if t < N {
        return t;
    }
    return t - N;
}

fn montgomery_redc_64(x: u64) -> u64 {
    // Overflow here is OK because we're modding by a small power of two
    let m: u64 = (x & R_MOD_MASK).overflowing_mul(NEG_N_INV).0 & R_MOD_MASK;
    // // (x + (m * N)) is 63 bit + 126 bits => 127 bits, then downshift
    // // so should all work out
    let t: u64 = (((x as u128) + multiply_64(m, N)) >> (R_LOG2 as u128)) as u64;
    if t < N {
        return t;
    }
    return t - N;
}

// Montgomery form I/O
fn to_montgomery_form(x: u64) -> u64 {
    // optimization pointed out by AOzdemir!
    // return multiply_64(x, R) % (uint128_t)N;
    return montgomery_redc(multiply_64(x, R_SQ_MOD_N));
}

fn from_montgomery_form(x: u64) -> u64 {
    // optimization pointed out by aozdemir!
    // return multiply_64(x, R_INV) % (uint128_t)N;
    return montgomery_redc_64(x);
}

fn montgomery_multiply(x: u64, y: u64) -> u64 {
    return montgomery_redc(multiply_64(x, y));
}

fn montgomery_add(x: u64, y: u64) -> u64 {
    let sum: u64 = x + y;
    if sum >= N {
        return sum - N;
    }
    return sum;
}

const N_PACKETS: usize  = 10000;
const N_SUMS: usize     = 20;
static mut PACKETS: [u64; N_PACKETS] = [0; N_PACKETS];
static mut SUMS: [u64; N_SUMS] = [0; N_SUMS];

// After computing sums with the naive approach, we'll copy them here to
// cross-check against later and ensure our Montgomery form is computing the
// right thing.
static mut SUMS_TO_CHECK: [u64; N_SUMS] = [0; N_SUMS];

fn main() {
    // Fill random packet values.
    unsafe { // rust thinks we have multiple threads????? oh well
        for i in 0..N_PACKETS {
            PACKETS[i] = rand::random::<u64>() % N;
        }

        const N_TRIALS: usize = 100;

        println!("Running withOUT montgomery...");
        // https://doc.rust-lang.org/std/time/struct.SystemTime.html
        let start_time = std::time::SystemTime::now();
        for _run in 0..N_TRIALS {
            SUMS.fill(0);
            for p in 0..N_PACKETS {
                let packet: u64 = PACKETS[p];
                let mut power: u64 = packet;
                for i in 0..N_SUMS {
                    SUMS[i] = SUMS[i].overflowing_add(power).0;
                    if SUMS[i] > N {
                        SUMS[i] = SUMS[i].overflowing_sub(N).0;
                    }
                    if i == N_SUMS {
                        break;
                    }
                    power = (multiply_64(power, packet) % (N as u128)) as u64;
                }
            }
        }
        println!("WITHOUT time: {}", (start_time.elapsed().unwrap().as_millis() as f64) / 1000.);

        for i in 0..N_SUMS {
            SUMS_TO_CHECK[i] = SUMS[i];
        }

        println!("Running WITH montgomery...");
        // https://doc.rust-lang.org/std/time/struct.SystemTime.html
        let start_time = std::time::SystemTime::now();
        for _run in 0..N_TRIALS {
            SUMS.fill(0);
            for p in 0..N_PACKETS {
                let packet: u64 = to_montgomery_form(PACKETS[p]);
                let mut power: u64 = packet;
                for i in 0..N_SUMS {
                    SUMS[i] = montgomery_add(SUMS[i], power);
                    if i == N_SUMS {
                        break;
                    }
                    power = montgomery_multiply(power, packet);
                }
            }
        }
        println!("WITH time: {}", (start_time.elapsed().unwrap().as_millis() as f64) / 1000.);

        // Check that Montgomery gave us the same results
        println!("Checking both give same power sums...");
        for i in 0..N_SUMS {
            assert!(from_montgomery_form(SUMS[i]) == SUMS_TO_CHECK[i]);
        }

        println!("Running WITH montgomery DIRECTLY...");
        // https://doc.rust-lang.org/std/time/struct.SystemTime.html
        let start_time = std::time::SystemTime::now();
        for _run in 0..N_TRIALS {
            SUMS.fill(0);
            for p in 0..N_PACKETS {
                let packet: u64 = PACKETS[p];
                let mut power: u64 = packet;
                for i in 0..N_SUMS {
                    SUMS[i] = montgomery_add(SUMS[i], power);
                    if i == N_SUMS {
                        break;
                    }
                    power = montgomery_multiply(power, packet);
                }
            }
        }
        println!("WITH DIRECTLY time: {}", (start_time.elapsed().unwrap().as_millis() as f64) / 1000.);
    }
}
