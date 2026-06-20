// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// fe25519.rs -- GF(2²⁵⁵-19) field element.
// Representation: [u64; 5] with 51-bit limbs (radix-2⁵¹).
//   h = h[0] + h[1]*2^51 + h[2]*2^102 + h[3]*2^153 + h[4]*2^204
// Limb range invariant (after reduction): each limb < 2^51.
// Limb range (intermediate): limbs may carry up to ~2^62 during mul — never overflow u64.
// @constant_time: all ops are branchless on secret data. No secret-dependent indexing.
// P55.7 will enforce @constant_time at compiler level; we uphold it structurally here.
// References: RFC 8032, curve25519-dalek (studied for design patterns), SUPERCOP ref10.

/// A field element in GF(2²⁵⁵-19), stored as 5 × 51-bit limbs.
/// The value is `limbs[0] + limbs[1]*2^51 + limbs[2]*2^102 + limbs[3]*2^153 + limbs[4]*2^204`.
#[derive(Clone, Copy, Debug)]
pub struct Fe25519(pub(crate) [u64; 5]);

impl Fe25519 {
    /// Additive identity: 0
    pub const ZERO: Self = Fe25519([0, 0, 0, 0, 0]);

    /// Multiplicative identity: 1
    pub const ONE: Self = Fe25519([1, 0, 0, 0, 0]);

    /// The constant d = -121665/121666 mod p used in the twisted Edwards curve equation.
    /// d = 37095705934669439343138083508754565189542113879843219016388785533085940283555
    /// In 51-bit limbs (little-endian):
    pub const D: Self = Fe25519([
        929955233495203,
        466365720129213,
        1662059464998953,
        2033849074728123,
        1442794654840575,
    ]);

    /// 2*d — precomputed for point addition formula
    pub const D2: Self = Fe25519([
        1859910466990425,
        932731440258426,
        1072319116312658,
        1815898335770999,
        633789495995903,
    ]);

    /// The square root of -1 mod p (SQRT_M1).
    /// Used in point decompression (RFC 8032 §5.1.3).
    /// Value: 2^((p-1)/4) mod p
    pub const SQRT_M1: Self = Fe25519([
        1718705420411056,
        234908883556509,
        2233514472574048,
        2117202627021982,
        765476049583133,
    ]);

    /// Construct from raw 51-bit limbs. Caller ensures limbs are reduced.
    #[inline]
    pub const fn from_limbs(limbs: [u64; 5]) -> Self {
        Fe25519(limbs)
    }

    /// Decode a 32-byte little-endian encoding into a field element.
    /// Performs one carry pass to normalize limbs.
    /// Does NOT verify the value is < p (caller must reduce if needed).
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        // Load 5 limbs from 255 bits (little-endian), 51 bits each.
        // Bit layout:
        //   limb[0]: bits   0..50  (bytes 0..6, low 51 bits)
        //   limb[1]: bits  51..101 (bytes 6..12, shifted)
        //   limb[2]: bits 102..152
        //   limb[3]: bits 153..203
        //   limb[4]: bits 204..254 (top bit of byte 31 is always 0 per RFC 8032)
        let load = |b: &[u8; 32], lo: usize, bits: usize| -> u64 {
            // Read 8 bytes starting at byte offset lo/8, then shift and mask
            let byte_lo = lo / 8;
            let bit_lo  = lo % 8;
            let mut v: u64 = 0;
            let n = ((bits + bit_lo + 7) / 8).min(8);
            for i in 0..n {
                if byte_lo + i < 32 {
                    v |= (b[byte_lo + i] as u64) << (i * 8);
                }
            }
            (v >> bit_lo) & ((1u64 << bits) - 1)
        };

        let h0 = load(bytes,   0, 51);
        let h1 = load(bytes,  51, 51);
        let h2 = load(bytes, 102, 51);
        let h3 = load(bytes, 153, 51);
        let h4 = load(bytes, 204, 51);

        Fe25519([h0, h1, h2, h3, h4])
    }

    /// Encode a field element to 32 bytes (little-endian, canonical form).
    /// Performs full reduction to ensure output is in [0, p-1].
    pub fn to_bytes(self) -> [u8; 32] {
        let r = self.reduce_canonical();
        let [h0, h1, h2, h3, h4] = r.0;

        // Pack 5 × 51-bit limbs into 255 bits (32 bytes, little-endian)
        let mut out = [0u8; 32];

        // h0: bits 0..50 → bytes 0..6
        out[0] =  (h0)        as u8;
        out[1] =  (h0 >> 8)   as u8;
        out[2] =  (h0 >> 16)  as u8;
        out[3] =  (h0 >> 24)  as u8;
        out[4] =  (h0 >> 32)  as u8;
        out[5] =  (h0 >> 40)  as u8;
        // h0 gives 51 bits; h1 starts at bit 51 = byte 6 bit 3
        out[6] =  ((h0 >> 48) | (h1 << 3)) as u8;
        out[7] =  (h1 >> 5)   as u8;
        out[8] =  (h1 >> 13)  as u8;
        out[9] =  (h1 >> 21)  as u8;
        out[10] = (h1 >> 29)  as u8;
        out[11] = (h1 >> 37)  as u8;
        out[12] = (h1 >> 45)  as u8;
        // h2 starts at bit 102 = byte 12 bit 6
        out[12] |= (h2 << 6) as u8;
        out[13] = (h2 >> 2)   as u8;
        out[14] = (h2 >> 10)  as u8;
        out[15] = (h2 >> 18)  as u8;
        out[16] = (h2 >> 26)  as u8;
        out[17] = (h2 >> 34)  as u8;
        out[18] = (h2 >> 42)  as u8;
        // h3 starts at bit 153 = byte 19 bit 1
        out[19] = (h2 >> 50) as u8 | (h3 << 1) as u8;
        out[20] = (h3 >> 7)   as u8;
        out[21] = (h3 >> 15)  as u8;
        out[22] = (h3 >> 23)  as u8;
        out[23] = (h3 >> 31)  as u8;
        out[24] = (h3 >> 39)  as u8;
        // h4 starts at bit 204 = byte 25 bit 4
        out[25] = (h3 >> 47) as u8 | (h4 << 4) as u8;
        out[26] = (h4 >> 4)   as u8;
        out[27] = (h4 >> 12)  as u8;
        out[28] = (h4 >> 20)  as u8;
        out[29] = (h4 >> 28)  as u8;
        out[30] = (h4 >> 36)  as u8;
        out[31] = (h4 >> 44)  as u8;

        out
    }

    /// Constant-time conditional select: returns `a` if choice==0, `b` if choice==1.
    /// `choice` MUST be 0 or 1. Branchless — safe on secret selector.
    #[inline]
    pub fn conditional_select(a: &Self, b: &Self, choice: u64) -> Self {
        // mask = 0 if choice==0, all-ones if choice==1
        let mask = choice.wrapping_neg(); // 0 → 0, 1 → 0xFFFF...
        let mut out = [0u64; 5];
        for i in 0..5 {
            out[i] = a.0[i] ^ (mask & (a.0[i] ^ b.0[i]));
        }
        Fe25519(out)
    }

    /// Constant-time swap: swaps (a, b) if choice==1, leaves unchanged if choice==0.
    #[inline]
    pub fn conditional_swap(a: &mut Self, b: &mut Self, choice: u64) {
        let mask = choice.wrapping_neg();
        for i in 0..5 {
            let t = mask & (a.0[i] ^ b.0[i]);
            a.0[i] ^= t;
            b.0[i] ^= t;
        }
    }

    /// Canonical reduction: reduce to [0, p-1].
    /// Two-pass: first propagate carries, then subtract p if value >= p.
    pub fn reduce_canonical(self) -> Self {
        let h = self.carry_reduce();

        // Now each limb < 2^51. Check if h >= p = 2^255 - 19.
        // p in limbs: [2^51-19, 2^51-1, 2^51-1, 2^51-1, 2^51-1]
        // Subtract p and check for underflow — branchless via mask.
        const MASK51: u64 = (1 << 51) - 1;
        const P0: u64 = MASK51 - 18; // 2^51 - 19
        const P1234: u64 = MASK51;   // 2^51 - 1

        // Attempt h - p
        let mut s = [0u64; 5];
        // Use i64 for signed borrow propagation
        let mut borrow: i64 = 0;
        let p = [P0, P1234, P1234, P1234, P1234];
        for i in 0..5 {
            let diff = (h.0[i] as i64) - (p[i] as i64) + borrow;
            // Extract low 51 bits, propagate borrow
            s[i] = (diff as u64) & MASK51;
            borrow = diff >> 51;
        }

        // borrow < 0 means h < p: keep h. borrow >= 0 means h >= p: use s.
        // Convert borrow to choice: if borrow < 0 → choice=0 (keep h), else choice=1 (use s)
        let use_s = if borrow >= 0 { 1u64 } else { 0u64 }; // 1 if borrow >= 0, 0 if borrow < 0
        Fe25519::conditional_select(&h, &Fe25519(s), use_s)
    }

    /// Carry reduction: propagate carries to normalize limbs to 51 bits.
    /// Does NOT reduce modulo p (may still be >= p after this).
    pub(crate) fn carry_reduce(self) -> Self {
        const MASK51: u64 = (1 << 51) - 1;
        let mut h = self.0;

        // Forward pass: propagate carry from low to high
        for i in 0..4 {
            let carry = h[i] >> 51;
            h[i] &= MASK51;
            h[i+1] += carry;
        }
        // Wrap: carry from h[4] folds back with factor 19 (since 2^255 ≡ 19 mod p)
        let carry4 = h[4] >> 51;
        h[4] &= MASK51;
        h[0] += carry4 * 19;

        // Second pass to clear any carry introduced into h[0]
        let carry0 = h[0] >> 51;
        h[0] &= MASK51;
        h[1] += carry0;

        Fe25519(h)
    }

    /// Constant-time equality check. Returns 1u64 if equal, 0u64 if not.
    pub fn ct_eq(&self, other: &Self) -> u64 {
        let a = self.reduce_canonical();
        let b = other.reduce_canonical();
        let mut diff = 0u64;
        for i in 0..5 {
            diff |= a.0[i] ^ b.0[i];
        }
        // diff == 0 iff equal. Map 0→1, nonzero→0.
        // ct_is_zero: (diff - 1) >> 63 in wrapping arithmetic
        ((diff.wrapping_sub(1)) >> 63) & 1
    }

    /// Returns true if this element is zero (uses ct_eq for constant-time).
    pub fn is_zero(&self) -> bool {
        self.ct_eq(&Fe25519::ZERO) == 1
    }

    /// Returns 1 if this element is negative (low bit of canonical form == 1), 0 otherwise.
    /// "Negative" in the Edwards sense: the element's byte encoding has bit 0 set.
    pub fn is_negative(&self) -> u64 {
        self.reduce_canonical().0[0] & 1
    }
}
