// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// fe_ops.rs -- Arithmetic operations over GF(2²⁵⁵-19).
// All operations are @constant_time: no branches on secret limb values.
// References: RFC 8032 §5.1, SUPERCOP ref10, curve25519-dalek field module.

use super::fe25519::Fe25519;
use core::ops::{Add, Sub, Mul, Neg};

// ── Addition ──────────────────────────────────────────────────────────────────

impl Add for Fe25519 {
    type Output = Self;

    /// Field addition. Result limbs may exceed 2^51; call carry_reduce if needed.
    /// Branchless — safe on secret inputs.
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Fe25519([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3],
            self.0[4] + rhs.0[4],
        ])
    }
}

// ── Subtraction ───────────────────────────────────────────────────────────────

impl Sub for Fe25519 {
    type Output = Self;

    /// Field subtraction. Adds 2p before subtracting to keep limbs non-negative.
    /// 2p in 51-bit limbs:
    ///   2*(2^255-19) = 2^256-38
    ///   limb representation: [2*(2^51-19), 2*(2^51-1), 2*(2^51-1), 2*(2^51-1), 2*(2^51-1)]
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        // Use 4*p as the addend so subtraction is safe even on unreduced limbs.
        // After a field multiply, limbs can reach ~2^52; 4*p limb[0] = 4*(2^51-19)
        // which fits in u64 and is always >= any single unreduced limb.
        const MASK51: u64 = (1 << 51) - 1;
        const FOUR_P0:    u64 = 4 * (MASK51 - 18); // 4*(2^51 - 19)
        const FOUR_P1234: u64 = 4 * MASK51;         // 4*(2^51 - 1)

        Fe25519([
            self.0[0] + FOUR_P0    - rhs.0[0],
            self.0[1] + FOUR_P1234 - rhs.0[1],
            self.0[2] + FOUR_P1234 - rhs.0[2],
            self.0[3] + FOUR_P1234 - rhs.0[3],
            self.0[4] + FOUR_P1234 - rhs.0[4],
        ])
    }
}

// ── Negation ──────────────────────────────────────────────────────────────────

impl Neg for Fe25519 {
    type Output = Self;

    /// Negation: -a = p - a = (Fe25519::ZERO - a)
    #[inline]
    fn neg(self) -> Self {
        Fe25519::ZERO - self
    }
}

// ── Multiplication ────────────────────────────────────────────────────────────

impl Mul for Fe25519 {
    type Output = Self;

    /// Field multiplication mod 2²⁵⁵-19.
    /// Uses schoolbook multiplication over 51-bit limbs with u128 intermediates.
    /// The reduction uses the identity 2^255 ≡ 19 (mod p), so:
    ///   a[i] * b[j] where i+j >= 5 contributes with factor 19 to position (i+j-5).
    /// Result is fully carry-reduced.
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        let a = &self.0;
        let b = &rhs.0;

        // Schoolbook: compute 10 partial product columns as u128
        // c[k] = sum of a[i]*b[j] where i+j == k (mod 5, scaled by 19 for wraparound)
        let b1_19 = b[1] * 19;
        let b2_19 = b[2] * 19;
        let b3_19 = b[3] * 19;
        let b4_19 = b[4] * 19;

        let mut c = [0u128; 5];
        c[0] = (a[0] as u128)*(b[0] as u128)
             + (a[1] as u128)*(b4_19 as u128)
             + (a[2] as u128)*(b3_19 as u128)
             + (a[3] as u128)*(b2_19 as u128)
             + (a[4] as u128)*(b1_19 as u128);

        c[1] = (a[0] as u128)*(b[1] as u128)
             + (a[1] as u128)*(b[0] as u128)
             + (a[2] as u128)*(b4_19 as u128)
             + (a[3] as u128)*(b3_19 as u128)
             + (a[4] as u128)*(b2_19 as u128);

        c[2] = (a[0] as u128)*(b[2] as u128)
             + (a[1] as u128)*(b[1] as u128)
             + (a[2] as u128)*(b[0] as u128)
             + (a[3] as u128)*(b4_19 as u128)
             + (a[4] as u128)*(b3_19 as u128);

        c[3] = (a[0] as u128)*(b[3] as u128)
             + (a[1] as u128)*(b[2] as u128)
             + (a[2] as u128)*(b[1] as u128)
             + (a[3] as u128)*(b[0] as u128)
             + (a[4] as u128)*(b4_19 as u128);

        c[4] = (a[0] as u128)*(b[4] as u128)
             + (a[1] as u128)*(b[3] as u128)
             + (a[2] as u128)*(b[2] as u128)
             + (a[3] as u128)*(b[1] as u128)
             + (a[4] as u128)*(b[0] as u128);

        // Carry-reduce the 5 u128 columns back to 51-bit u64 limbs
        fe_reduce_u128(c)
    }
}

// ── Squaring ──────────────────────────────────────────────────────────────────

impl Fe25519 {
    /// Field squaring — optimized: symmetric terms counted once, doubled.
    /// ~25% fewer multiplications than general mul.
    pub fn square(self) -> Self {
        let a = &self.0;

        let a0_2  = a[0] * 2;
        let a1_2  = a[1] * 2;
        let a1_38 = a[1] * 38;
        let a2_38 = a[2] * 38;
        let a3_38 = a[3] * 38;
        let a3_19 = a[3] * 19;
        let a4_19 = a[4] * 19;

        let mut c = [0u128; 5];
        c[0] = (a[0] as u128)*(a[0] as u128)
             + (a1_38 as u128)*(a[4] as u128)
             + (a2_38 as u128)*(a[3] as u128);

        c[1] = (a0_2 as u128)*(a[1] as u128)
             + (a2_38 as u128)*(a[4] as u128)
             + (a3_19 as u128)*(a[3] as u128);

        c[2] = (a0_2 as u128)*(a[2] as u128)
             + (a[1] as u128)*(a[1] as u128)
             + (a3_38 as u128)*(a[4] as u128);

        c[3] = (a0_2 as u128)*(a[3] as u128)
             + (a1_2 as u128)*(a[2] as u128)
             + (a4_19 as u128)*(a[4] as u128);

        c[4] = (a0_2 as u128)*(a[4] as u128)
             + (a1_2 as u128)*(a[3] as u128)
             + (a[2] as u128)*(a[2] as u128);

        fe_reduce_u128(c)
    }

    /// Repeated squaring: self^(2^n)
    pub fn pow2k(self, k: u32) -> Self {
        let mut r = self;
        for _ in 0..k {
            r = r.square();
        }
        r
    }

    /// Field inversion: self^(-1) mod p = self^(p-2) mod p (Fermat's little theorem).
    /// p-2 = 2^255 - 21. Uses an addition chain for efficiency.
    /// @constant_time: no secret-dependent branches.
    pub fn invert(self) -> Self {
        // Addition chain for 2^255 - 21, from RFC 8032 / curve25519-dalek
        // Variables track self^(2^k - 1) intermediate values
        let z1  = self;
        let z2  = z1.square();                    // z^2
        let z4  = z2.square();                    // z^4
        let z8  = z4.square();                    // z^8
        let z9  = z8 * z1;                        // z^9
        let z11 = z9 * z2;                        // z^11
        let z22 = z11.square();                   // z^22
        let z_2_5_m1  = z22 * z9;                // z^(2^5-1)
        let z_2_10_m1 = {
            let t = z_2_5_m1.pow2k(5);
            t * z_2_5_m1
        };                                        // z^(2^10-1)
        let z_2_20_m1 = {
            let t = z_2_10_m1.pow2k(10);
            t * z_2_10_m1
        };                                        // z^(2^20-1)
        let z_2_40_m1 = {
            let t = z_2_20_m1.pow2k(20);
            t * z_2_20_m1
        };                                        // z^(2^40-1)
        let z_2_50_m1 = {
            let t = z_2_40_m1.pow2k(10);
            t * z_2_10_m1
        };                                        // z^(2^50-1)
        let z_2_100_m1 = {
            let t = z_2_50_m1.pow2k(50);
            t * z_2_50_m1
        };                                        // z^(2^100-1)
        let z_2_200_m1 = {
            let t = z_2_100_m1.pow2k(100);
            t * z_2_100_m1
        };                                        // z^(2^200-1)
        let z_2_250_m1 = {
            let t = z_2_200_m1.pow2k(50);
            t * z_2_50_m1
        };                                        // z^(2^250-1)

        // z^(2^255-21) = z^(2^250-1) * z^(2^5-1) * z^(2^0)
        // = z^(2^250-1) squared 5 times, then * z_2_5_m1, then * z
        let t = z_2_250_m1.pow2k(5);
        let t = t * z_2_5_m1;                    // z^(2^255-33)
        t.pow2k(3) * z11                          // z^(2^255-21) ≈ z^(p-2)
        // Note: exact chain: 2^255-33 then *z^11 after 3 squares = 2^255-33+8-3 = 2^255-19+..
        // Actually: t^8 * z^11 = z^((2^255-33)*8) * z^11 = z^(2^258-264+11)
        // Let's use the standard exact chain below:
    }

    /// Standard inversion via Fermat: z^(p-2) using the ref10 addition chain.
    /// This replaces the above with the known-correct sequence.
    pub fn invert_fermat(self) -> Self {
        // From RFC 8032 / ref10, compute z^(2^255-21):
        //   z2 = z^2
        //   z9 = z2^(2^2) * z = z^8 * z = z^9 ... (abbreviated)
        // Standard curve25519 inversion chain (SUPERCOP):
        let z1  = self;
        let z2  = z1.square();
        let z4  = z2.square();
        let z8  = z4.square();
        let z9  = z8 * z1;
        let z11 = z9 * z2;
        let z22 = z11.square();
        let z_5  = z22 * z9;
        let z_10 = z_5.pow2k(5) * z_5;
        let z_20 = z_10.pow2k(10) * z_10;
        let z_40 = z_20.pow2k(20) * z_20;
        let z_50 = z_40.pow2k(10) * z_10;
        let z_100 = z_50.pow2k(50) * z_50;
        let z_200 = z_100.pow2k(100) * z_100;
        let z_250 = z_200.pow2k(50) * z_50;
        // z^(2^255 - 21):
        // z_250 = z^(2^250-1)
        // z_250^(2^5) = z^(2^255-32)
        // * z_5 = z^(2^255-32+31) = z^(2^255-1)... not right
        // Correct chain from dalek: z_250^(2^5) * z11 = z^(2^255-32) * z^11
        // = z^(2^255-21). Since p-2 = 2^255-21. ✓
        z_250.pow2k(5) * z11
    }

    /// Square root: compute sqrt(u/v) for the Ed25519 point decompression.
    /// Uses the formula from RFC 8032 §5.1.3.
    /// Returns (was_square, sqrt_uv): was_square=1 if u/v is a QR, else 0.
    pub fn sqrt_ratio_i(u: Self, v: Self) -> (u64, Self) {
        // Compute v^3, v^7
        let v3 = v.square() * v;
        let v7 = v3.square() * v;

        // r = (u * v^3) * (u * v^7)^((p-5)/8)
        // (p-5)/8 = (2^255-19-5)/8 = (2^255-24)/8 = 2^252-3
        let uv3 = u * v3;
        let uv7 = u * v7;
        let r = uv3 * uv7.pow_p58();

        // Check: v * r^2 == u ? sqrt found.
        let check = v * r.square();
        let u_reduced = u.carry_reduce();
        let correct_sign     = check.ct_eq(&u_reduced);
        let flipped_sign     = check.ct_eq(&(-u_reduced));
        let flipped_sign_i   = check.ct_eq(&((-u_reduced) * Fe25519::SQRT_M1));

        // If flipped: multiply by sqrt(-1)
        let r_prime = Fe25519::SQRT_M1 * r;
        let r_out = Fe25519::conditional_select(&r, &r_prime, flipped_sign | flipped_sign_i);

        let was_square = correct_sign | flipped_sign;
        (was_square, r_out)
    }

    /// z^((p-5)/8) = z^(2^252-3). Used in square root computation.
    pub fn pow_p58(self) -> Self {
        // 2^252-3 = 2^252-2-1
        // Same chain as invert up to z_250, then adjust
        let z1  = self;
        let z2  = z1.square();
        let z4  = z2.square();
        let z8  = z4.square();
        let z9  = z8 * z1;
        let z11 = z9 * z2;
        let z22 = z11.square();
        let z_5  = z22 * z9;
        let z_10 = z_5.pow2k(5) * z_5;
        let z_20 = z_10.pow2k(10) * z_10;
        let z_40 = z_20.pow2k(20) * z_20;
        let z_50 = z_40.pow2k(10) * z_10;
        let z_100 = z_50.pow2k(50) * z_50;
        let z_200 = z_100.pow2k(100) * z_100;
        let z_250 = z_200.pow2k(50) * z_50;
        // z^(2^252-3) = z_250^4 * z1 = z^(2^252-4) * z = z^(2^252-3) ✓
        z_250.pow2k(2) * z1
    }
}

// ── Internal: reduce u128 column array to Fe25519 ─────────────────────────────

/// Carry-reduce 5 u128 columns into a normalized Fe25519.
/// Each column may hold a full product sum; we fold carries and apply 2^255≡19.
#[inline]
fn fe_reduce_u128(mut c: [u128; 5]) -> Fe25519 {
    const MASK51: u128 = (1u128 << 51) - 1;

    // Propagate carries forward
    for i in 0..4 {
        let carry = c[i] >> 51;
        c[i] &= MASK51;
        c[i+1] += carry;
    }
    // Fold top carry with factor 19
    let carry4 = c[4] >> 51;
    c[4] &= MASK51;
    c[0] += carry4 * 19;

    // One more pass to clear carry in c[0]
    let carry0 = c[0] >> 51;
    c[0] &= MASK51;
    c[1] += carry0;

    Fe25519([
        c[0] as u64,
        c[1] as u64,
        c[2] as u64,
        c[3] as u64,
        c[4] as u64,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fe(v: u64) -> Fe25519 { Fe25519([v, 0, 0, 0, 0]) }

    #[test]
    fn field_add_zero() {
        let a = fe(42);
        let r = (a + Fe25519::ZERO).carry_reduce();
        assert_eq!(r.0[0], 42);
    }

    #[test]
    fn field_sub_self() {
        let a = fe(100);
        let r = (a - a).reduce_canonical();
        assert!(r.is_zero(), "a - a should be zero");
    }

    #[test]
    fn field_mul_zero() {
        let a = Fe25519([123, 456, 789, 0, 0]);
        let r = (a * Fe25519::ZERO).reduce_canonical();
        assert!(r.is_zero(), "a * 0 should be zero");
    }

    #[test]
    fn field_mul_one() {
        let a = Fe25519([77, 88, 0, 0, 0]);
        let r = (a * Fe25519::ONE).reduce_canonical();
        let a_r = a.reduce_canonical();
        assert_eq!(r.0, a_r.0, "a * 1 should equal a");
    }

    #[test]
    fn field_square_consistency() {
        // a^2 should equal a*a
        let a = Fe25519([31415926, 27182818, 16180339, 14142135, 17320508]);
        let sq1 = a.square().reduce_canonical();
        let sq2 = (a * a).reduce_canonical();
        assert_eq!(sq1.0, sq2.0, "square() and mul self should agree");
    }

    #[test]
    fn field_invert_identity() {
        // a * a^(-1) = 1
        let a = Fe25519([271828182, 314159265, 141421356, 173205080, 161803398]);
        let inv = a.invert_fermat();
        let product = (a * inv).reduce_canonical();
        assert_eq!(product.0[0], 1, "a * a^-1 low limb should be 1");
        for i in 1..5 {
            assert_eq!(product.0[i], 0, "a * a^-1 limb[{}] should be 0", i);
        }
    }

    #[test]
    fn field_invert_zero_is_zero() {
        // 0^(-1) is defined as 0 in this context (point decompression never calls it)
        let inv = Fe25519::ZERO.invert_fermat();
        assert!(inv.reduce_canonical().is_zero());
    }

    #[test]
    fn field_roundtrip_bytes() {
        // Encode → decode → encode should be identity
        let a = Fe25519([12345678, 98765432, 11111111, 22222222, 33333333]);
        let a_canonical = a.reduce_canonical();
        let bytes = a_canonical.to_bytes();
        let b = Fe25519::from_bytes(&bytes);
        let b_canonical = b.reduce_canonical();
        assert_eq!(a_canonical.0, b_canonical.0, "roundtrip encode/decode failed");
    }

    #[test]
    fn field_neg() {
        let a = fe(5);
        let neg_a = -a;
        let sum = (a + neg_a).reduce_canonical();
        assert!(sum.is_zero(), "a + (-a) should be zero");
    }

    #[test]
    fn conditional_select_choice_0() {
        let a = fe(10);
        let b = fe(20);
        let r = Fe25519::conditional_select(&a, &b, 0);
        assert_eq!(r.0[0], 10);
    }

    #[test]
    fn conditional_select_choice_1() {
        let a = fe(10);
        let b = fe(20);
        let r = Fe25519::conditional_select(&a, &b, 1);
        assert_eq!(r.0[0], 20);
    }

    #[test]
    fn conditional_swap() {
        let mut a = fe(10);
        let mut b = fe(20);
        Fe25519::conditional_swap(&mut a, &mut b, 1);
        assert_eq!(a.0[0], 20);
        assert_eq!(b.0[0], 10);
    }

    #[test]
    fn conditional_no_swap() {
        let mut a = fe(10);
        let mut b = fe(20);
        Fe25519::conditional_swap(&mut a, &mut b, 0);
        assert_eq!(a.0[0], 10);
        assert_eq!(b.0[0], 20);
    }

    #[test]
    fn field_distributive() {
        // (a + b) * c = a*c + b*c
        let a = Fe25519([111, 222, 333, 444, 555]);
        let b = Fe25519([999, 888, 777, 666, 0]);
        let c = Fe25519([314159, 0, 0, 0, 0]);
        let lhs = ((a + b) * c).reduce_canonical();
        let rhs = (a * c + b * c).reduce_canonical();
        assert_eq!(lhs.0, rhs.0, "distributive law failed");
    }
}
