import itertools

def argmax(l):
    return max(range(len(l)), key=lambda i: l[i])

def prod(l):
    product = 1
    for x in l: product *= x
    return product

import math

def generalized_crt(congruences, moduli):
    for i, j in itertools.combinations(range(len(congruences)), 2):
        gcd = math.gcd(moduli[i], moduli[j])
        if congruences[i] % gcd != congruences[j] % gcd:
            return False
    # If that check passes, then there is a unique lifting that we can get from
    # the CRT.
    moduli_factored = list(map(factor, moduli))
    coprime_moduli = [1 for _ in moduli]
    for factors in zip(*moduli_factored):
        # factors[i][0] = prime, factors[i][1] = power
        prime = factors[0][0]
        assert all(factor[0] == prime for factor in factors)
        # This is the one contributing to the LCM
        max_i = argmax([factor[1] for factor in factors])
        coprime_moduli[max_i] *= prime**factors[max_i][1]
    return crt(congruences, coprime_moduli)

def crt(congruences, moduli):
    # First inverse moduli[i] wrt prod_{j != i} moduli[j]
    product = prod(moduli)
    solution = 0
    for i, (congruence, modulus) in enumerate(zip(congruences, moduli)):
        others = product // modulus
        inverse, _ = invert(modulus, others)
        solution = (solution + (congruence * inverse)) % product
    return solution

# solves x == a mod mod_a, x == b mod mod_b
def generalized_crt_old(congruences, moduli):
    factor_mod_a, factor_mod_b = factor(mod_a), factor(mod_b)
    gcd, factor_gcd = gcd_from_factors(factor_mod_a, factor_mod_b)
    print("GCD of", mod_a, mod_b, "is", gcd)
    assert a % gcd == b % gcd
    coprime_mod_a, coprime_mod_b = 1, 1
    for a_factor, b_factor in zip(factor_mod_a, factor_mod_b):
        assert a_factor[0] == b_factor[0]
        if a_factor[1] == b_factor[1] == 0: continue
        if a_factor[1] >= b_factor[1]:
            coprime_mod_a *= (a_factor[0]**a_factor[1])
        else:
            coprime_mod_b *= (b_factor[0]**b_factor[1])
    a_inv_mod_b, b_inv_mod_a = invert(coprime_mod_a, coprime_mod_b)
    return ((a * b_inv_mod_a) + (b * a_inv_mod_b)) % (coprime_mod_a * coprime_mod_b)

def gcd_from_factors(a, b):
    gcd = 1
    factors = []
    for af, bf in zip(a, b):
        mf = af if af[1] <= bf[1] else bf
        factors.append(mf)
        gcd *= mf[0]**mf[1]
    return gcd, factors

def prime_sieve(hi):
    is_prime = [True for _ in range(hi)]
    primes = []
    for i in range(2, hi):
        if not is_prime[i]: continue
        primes.append(i)
        for j in range(i, hi, i):
            is_prime[j] = False
    return primes
PRIMES = prime_sieve(101)

def prime_power(prime, number):
    for n in itertools.count():
        if number % prime**n != 0:
            return n - 1

def factor(num):
    factors = []
    check = 1
    for prime in PRIMES:
        factors.append((prime, prime_power(prime, num)))
        check *= prime**prime_power(prime, num)
    assert check == num, "set a bigger max prime"
    return factors

# From wiki
def bezout(a, b):
    old_r, r = a, b
    old_s, s = 1, 0
    old_t, t = 0, 1

    while r != 0:
        quotient = old_r // r
        old_r, r = r, old_r - (quotient * r)
        old_s, s = s, old_s - (quotient * s)
        old_t, t = t, old_t - (quotient * t)

    return old_s, old_t

def invert(a, b): # inverse of a mod b and vice versa
    bezout_a, bezout_b = bezout(a, b)
    # bezout_a*a + bezout_b*b = gcd(a, b), so if a, b coprime then
    # bezout_a = a^{-1} mod b and same for b
    return (bezout_a % b), (bezout_b % a)
