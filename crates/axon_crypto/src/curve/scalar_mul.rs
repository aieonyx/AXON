// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// curve/scalar_mul.rs -- Constant-time scalar multiplication for Ed25519.
// Algorithm: double-and-add from high bit with conditional_select for ct safety.
// @constant_time: all 255 bits processed unconditionally.
// References: RFC 8032 ss5.1.4, SUPERCOP ref10 scalar reduction.

use crate::curve::point::{ExtendedPoint, IDENTITY};

pub fn scalar_mul(point: &ExtendedPoint, scalar: &[u8; 32]) -> ExtendedPoint {
    let mut r = IDENTITY;
    for i in (0..32).rev() {
        let byte = scalar[i];
        let start_bit: i32 = if i == 31 { 6 } else { 7 };
        for bit in (0..=start_bit).rev() {
            r = r.double();
            let b = ((byte >> bit) & 1) as u64;
            let addend = ExtendedPoint::conditional_select(&IDENTITY, point, b);
            r = r + addend;
        }
    }
    r
}

pub fn basepoint_mul(scalar: &[u8; 32]) -> ExtendedPoint {
    use crate::curve::point::BASEPOINT;
    scalar_mul(&BASEPOINT, scalar)
}

#[inline]
pub fn clamp_scalar(scalar: &mut [u8; 32]) {
    scalar[0]  &= 248;
    scalar[31] &= 127;
    scalar[31] |= 64;
}

pub fn reduce_scalar_64(s: &[u8; 64]) -> [u8; 32] {
    let load3 = |b: &[u8], i: usize| -> i64 {
        (b[i] as i64) | ((b[i+1] as i64) << 8) | ((b[i+2] as i64) << 16)
    };
    let load4 = |b: &[u8], i: usize| -> i64 {
        (b[i] as i64) | ((b[i+1] as i64) << 8) | ((b[i+2] as i64) << 16) | ((b[i+3] as i64) << 24)
    };
    let s0  = 2097151i64 & load3(s, 0);
    let s1  = 2097151i64 & (load4(s, 2) >> 5);
    let s2  = 2097151i64 & (load3(s, 5) >> 2);
    let s3  = 2097151i64 & (load4(s, 7) >> 7);
    let s4  = 2097151i64 & (load4(s, 10) >> 4);
    let s5  = 2097151i64 & (load3(s, 13) >> 1);
    let s6  = 2097151i64 & (load4(s, 15) >> 6);
    let s7  = 2097151i64 & (load3(s, 18) >> 3);
    let s8  = 2097151i64 & load3(s, 21);
    let s9  = 2097151i64 & (load4(s, 23) >> 5);
    let s10 = 2097151i64 & (load3(s, 26) >> 2);
    let s11 = 2097151i64 & (load4(s, 28) >> 7);
    let s12 = 2097151i64 & (load4(s, 31) >> 4);
    let s13 = 2097151i64 & (load3(s, 34) >> 1);
    let s14 = 2097151i64 & (load4(s, 36) >> 6);
    let s15 = 2097151i64 & (load3(s, 39) >> 3);
    let s16 = 2097151i64 & load3(s, 42);
    let s17 = 2097151i64 & (load4(s, 44) >> 5);
    let s18 = 2097151i64 & (load3(s, 47) >> 2);
    let s19 = 2097151i64 & (load4(s, 49) >> 7);
    let s20 = 2097151i64 & (load4(s, 52) >> 4);
    let s21 = 2097151i64 & (load3(s, 55) >> 1);
    let s22 = 2097151i64 & (load4(s, 57) >> 6);
    let s23 =               load4(s, 60) >> 3;
    const MU0: i64 =  666643;
    const MU1: i64 =  470296;
    const MU2: i64 =  654183;
    const MU3: i64 = -997805;
    const MU4: i64 =  136657;
    const MU5: i64 = -683901;
    let mut r = [0i64; 12];
    r[0]  = s0  + s12*MU0;
    r[1]  = s1  + s12*MU1 + s13*MU0;
    r[2]  = s2  + s12*MU2 + s13*MU1 + s14*MU0;
    r[3]  = s3  + s12*MU3 + s13*MU2 + s14*MU1 + s15*MU0;
    r[4]  = s4  + s12*MU4 + s13*MU3 + s14*MU2 + s15*MU1 + s16*MU0;
    r[5]  = s5  + s12*MU5 + s13*MU4 + s14*MU3 + s15*MU2 + s16*MU1 + s17*MU0;
    r[6]  = s6             + s13*MU5 + s14*MU4 + s15*MU3 + s16*MU2 + s17*MU1 + s18*MU0;
    r[7]  = s7                       + s14*MU5 + s15*MU4 + s16*MU3 + s17*MU2 + s18*MU1 + s19*MU0;
    r[8]  = s8                                 + s15*MU5 + s16*MU4 + s17*MU3 + s18*MU2 + s19*MU1 + s20*MU0;
    r[9]  = s9                                            + s16*MU5 + s17*MU4 + s18*MU3 + s19*MU2 + s20*MU1 + s21*MU0;
    r[10] = s10                                                      + s17*MU5 + s18*MU4 + s19*MU3 + s20*MU2 + s21*MU1 + s22*MU0;
    r[11] = s11                                                                 + s18*MU5 + s19*MU4 + s20*MU3 + s21*MU2 + s22*MU1 + s23*MU0;
    let carry = |r: &mut [i64; 12], i: usize| { r[i+1] += r[i] >> 21; r[i] &= 0x1fffff; };
    for i in 0..11 { carry(&mut r, i); }
    r[0] += r[11]*MU0; r[1] += r[11]*MU1; r[2] += r[11]*MU2;
    r[3] += r[11]*MU3; r[4] += r[11]*MU4; r[5] += r[11]*MU5;
    r[11] = 0;
    for i in 0..10 { carry(&mut r, i); }
    let mut out = [0u8; 32];
    out[0]  =  (r[0])                          as u8;
    out[1]  =  (r[0] >> 8)                     as u8;
    out[2]  = ((r[0] >> 16) | (r[1] << 5))     as u8;
    out[3]  =  (r[1] >> 3)                     as u8;
    out[4]  =  (r[1] >> 11)                    as u8;
    out[5]  = ((r[1] >> 19) | (r[2] << 2))     as u8;
    out[6]  =  (r[2] >> 6)                     as u8;
    out[7]  = ((r[2] >> 14) | (r[3] << 7))     as u8;
    out[8]  =  (r[3] >> 1)                     as u8;
    out[9]  =  (r[3] >> 9)                     as u8;
    out[10] = ((r[3] >> 17) | (r[4] << 4))     as u8;
    out[11] =  (r[4] >> 4)                     as u8;
    out[12] =  (r[4] >> 12)                    as u8;
    out[13] = ((r[4] >> 20) | (r[5] << 1))     as u8;
    out[14] =  (r[5] >> 7)                     as u8;
    out[15] = ((r[5] >> 15) | (r[6] << 6))     as u8;
    out[16] =  (r[6] >> 2)                     as u8;
    out[17] =  (r[6] >> 10)                    as u8;
    out[18] = ((r[6] >> 18) | (r[7] << 3))     as u8;
    out[19] =  (r[7] >> 5)                     as u8;
    out[20] =  (r[7] >> 13)                    as u8;
    out[21] =  (r[8])                          as u8;
    out[22] =  (r[8] >> 8)                     as u8;
    out[23] = ((r[8] >> 16) | (r[9] << 5))     as u8;
    out[24] =  (r[9] >> 3)                     as u8;
    out[25] =  (r[9] >> 11)                    as u8;
    out[26] = ((r[9] >> 19) | (r[10] << 2))    as u8;
    out[27] =  (r[10] >> 6)                    as u8;
    out[28] = ((r[10] >> 14) | (r[11] << 7))   as u8;
    out[29] =  (r[11] >> 1)                    as u8;
    out[30] =  (r[11] >> 9)                    as u8;
    out[31] =  (r[11] >> 17)                   as u8;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::point::{BASEPOINT, IDENTITY};

    fn affine_eq(a: &ExtendedPoint, b: &ExtendedPoint) -> bool {
        let (ax, ay) = a.to_affine();
        let (bx, by) = b.to_affine();
        ax.ct_eq(&bx) == 1 && ay.ct_eq(&by) == 1
    }

    fn s64(n: u64) -> [u8; 32] {
        let mut s = [0u8; 32];
        s[..8].copy_from_slice(&n.to_le_bytes());
        s
    }

    #[test]
    fn scalar_mul_zero() {
        assert!(scalar_mul(&BASEPOINT, &[0u8;32]).is_identity());
    }

    #[test]
    fn scalar_mul_one() {
        assert!(affine_eq(&scalar_mul(&BASEPOINT, &s64(1)), &BASEPOINT));
    }

    #[test]
    fn scalar_mul_two() {
        assert!(affine_eq(&scalar_mul(&BASEPOINT, &s64(2)), &BASEPOINT.double()));
    }

    #[test]
    fn scalar_mul_identity_point() {
        assert!(scalar_mul(&IDENTITY, &s64(12345)).is_identity());
    }

    #[test]
    fn scalar_mul_additive() {
        let r = scalar_mul(&BASEPOINT, &s64(3));
        let manual = BASEPOINT + BASEPOINT + BASEPOINT;
        assert!(affine_eq(&r, &manual));
    }

    #[test]
    fn scalar_mul_five() {
        let r = scalar_mul(&BASEPOINT, &s64(5));
        let five_b = BASEPOINT.double().double() + BASEPOINT;
        assert!(affine_eq(&r, &five_b));
    }

    #[test]
    fn scalar_mul_large() {
        let r = scalar_mul(&BASEPOINT, &s64(100));
        let b64  = BASEPOINT.double().double().double().double().double().double();
        let b32  = BASEPOINT.double().double().double().double().double();
        let b4   = BASEPOINT.double().double();
        let b100 = b64 + b32 + b4;
        assert!(affine_eq(&r, &b100));
    }

    #[test]
    fn clamp_clears_low_bits() {
        let mut s = [0xffu8; 32];
        clamp_scalar(&mut s);
        assert_eq!(s[0] & 7, 0);
    }

    #[test]
    fn clamp_sets_bit_254() {
        let mut s = [0u8; 32];
        clamp_scalar(&mut s);
        assert_eq!((s[31] >> 6) & 1, 1);
    }

    #[test]
    fn clamp_clears_bit_255() {
        let mut s = [0xffu8; 32];
        clamp_scalar(&mut s);
        assert_eq!(s[31] >> 7, 0);
    }

    #[test]
    fn reduce_scalar_zero() {
        let r = reduce_scalar_64(&[0u8; 64]);
        assert_eq!(r, [0u8; 32]);
    }

    #[test]
    fn reduce_scalar_one() {
        let mut input = [0u8; 64];
        input[0] = 1;
        let r = reduce_scalar_64(&input);
        assert_eq!(r[0], 1);
        for i in 1..32 { assert_eq!(r[i], 0); }
    }

    #[test]
    fn basepoint_mul_matches_scalar_mul() {
        let s = s64(42);
        assert!(affine_eq(&scalar_mul(&BASEPOINT, &s), &basepoint_mul(&s)));
    }
}
