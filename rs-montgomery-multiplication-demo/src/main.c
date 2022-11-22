#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>
#include <assert.h>
#include "timerstuff.h"

// GENERATE these with preprocess.py
// NOTE: Using N < 2^63, R = 2^63
const uint64_t N        = 9223372036854775783ull,
               R        = 9223372036854775808ull,
               RInv     = 1106804644422573094ull,
               NegNInv  = 1106804644422573097ull,
               RSqModN  = 625ull;
// Bitwise division & mod by R
const uint64_t R_log2 = 63;
const uint64_t RModMask = (1ull << 63) - 1;

// Add a 128-bit integer type
// https://stackoverflow.com/questions/16088282/is-there-a-128-bit-integer-in-gcc
#define uint128_t __uint128_t

// Helper to do 64 x 64 - > 128 multiplication
inline static uint128_t multiply_64(uint64_t x, uint64_t y) {
    return (uint128_t)x * (uint128_t)y;
}

// Montgomery form multiplication, addition (these are cheap!)
// from wiki https://en.wikipedia.org/wiki/Montgomery_modular_multiplication
static inline uint64_t montgomery_redc(const uint128_t x) {
    // Overflow here is OK because we're modding by a small power of two
    uint64_t m = ((x & RModMask) * NegNInv) & RModMask;
    // // (x + (m * N)) is 63 bit + 126 bits => 127 bits, then downshift
    // // so should all work out
    uint64_t t = (((uint128_t)x) + multiply_64(m, N)) >> (uint128_t)R_log2;
    return (t < N) ? t : (t - N);
}

static inline uint64_t montgomery_redc_64(const uint64_t x) {
    // Overflow here is OK because we're modding by a small power of two
    uint64_t m = ((x & RModMask) * NegNInv) & RModMask;
    // // (x + (m * N)) is 63 bit + 126 bits => 127 bits, then downshift
    // // so should all work out
    uint64_t t = (((uint128_t)x) + multiply_64(m, N)) >> (uint128_t)R_log2;
    return (t < N) ? t : (t - N);
}

// Montgomery form I/O (these are expensive!)
inline static uint64_t to_montgomery_form(const uint64_t x) {
    // optimization pointed out by AOzdemir!
    // return multiply_64(x, R) % (uint128_t)N;
    return montgomery_redc(multiply_64(x, RSqModN));
}

inline static uint64_t from_montgomery_form(const uint64_t x) {
    // optimization pointed out by aozdemir!
    // return multiply_64(x, RInv) % (uint128_t)N;
    return montgomery_redc_64(x);
}

static inline uint64_t montgomery_multiply(const uint64_t x, const uint64_t y) {
    return montgomery_redc(multiply_64(x, y));
}

static inline uint64_t montgomery_add(const uint64_t x, const uint64_t y) {
    uint64_t sum = x + y;
    return (sum >= N) ? (sum - N) : sum;
}

// Global variables
#define N_PACKETS   10000
uint64_t PACKETS[N_PACKETS] = {0};
#define N_SUMS      20
uint64_t SUMS[N_SUMS] = {0};

// After computing sums with the naive approach, we'll copy them here to
// cross-check against later and ensure our Montgomery form is computing the
// right thing.
uint64_t SUMS_TO_CHECK[N_SUMS] = {0};

int main() {
    // Fill random packet values.
    srand(24);
    for (int i = 0; i < N_PACKETS; i++) {
        PACKETS[i] = rand();
        PACKETS[i] = (PACKETS[i] << 32ul) | (uint64_t)rand();
        PACKETS[i] = PACKETS[i] % N;
    }

    const int N_TRIALS = 100;

    // Time the naive approach
    printf("Running withOUT montgomery...\n");
    timer_start();
    for (int run = 0; run < N_TRIALS; run++) {
        memset(SUMS, 0, N_SUMS * sizeof(SUMS[0]));
        for (int p = 0; p < N_PACKETS; p++) {
            uint64_t packet = PACKETS[p];
            uint64_t power = packet;
            for (int i = 0; i < N_SUMS; i++) {
                SUMS[i] += power;
                if (SUMS[i] > N) SUMS[i] -= N;
                if (i == N_SUMS) break;
                power = multiply_64(power, packet) % (uint128_t)N;
            }
        }
    }
    timer_print("WITHOUT");

    // Backup the values computed by the naive approach for later crosschecking
    memcpy(SUMS_TO_CHECK, SUMS, sizeof(SUMS[0]) * N_SUMS);

    // Time the Montgomery form approach
    printf("Running WITH montgomery...\n");
    timer_start();
    for (int run = 0; run < N_TRIALS; run++) {
        memset(SUMS, 0, N_SUMS * sizeof(SUMS[0]));
        for (int p = 0; p < N_PACKETS; p++) {
            uint64_t packet = to_montgomery_form(PACKETS[p]);
            uint64_t power = packet;
            for (int i = 0; i < N_SUMS; i++) {
                SUMS[i] = montgomery_add(power, SUMS[i]);
                if (i == N_SUMS) break;
                power = montgomery_multiply(power, packet);
            }
        }
    }
    timer_print("WITH");

    // Check that Montgomery gave us the same results
    printf("Checking both give same power sums...\n");
    for (int i = 0; i < N_SUMS; i++)
        assert(from_montgomery_form(SUMS[i]) == SUMS_TO_CHECK[i]);

    // Time the Montgomery form approach
    printf("Running WITH montgomery form DIRECTLY...\n");
    timer_start();
    for (int run = 0; run < N_TRIALS; run++) {
        memset(SUMS, 0, N_SUMS * sizeof(SUMS[0]));
        for (int p = 0; p < N_PACKETS; p++) {
            uint64_t packet = PACKETS[p];
            uint64_t power = packet;
            for (int i = 0; i < N_SUMS; i++) {
                SUMS[i] = montgomery_add(power, SUMS[i]);
                if (i == N_SUMS) break;
                power = montgomery_multiply(power, packet);
            }
        }
    }
    timer_print("WITH DIRECTLY");
}
