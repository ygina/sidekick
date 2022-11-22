import crt

# https://primes.utm.edu/lists/2small/0bit.html
N = 2**63 - 25
R = 2**63
RInv, NInv = crt.invert(R, N)
NPrime = R - NInv
RSqModN = (2**63)**2 % N

assert (R * RInv) % N == 1
assert (N * NInv) % R == 1
assert (N * NPrime) % R == (R - 1)

print(f"const N: u64\t\t= {N};")
print(f"const R: u64\t\t= {R};")
print(f"const RInv: u64\t\t= {RInv};")
print(f"const NegNInv: u64\t\t={NPrime};")
print(f"const RSqModN: u64\t\t={RSqModN};")
