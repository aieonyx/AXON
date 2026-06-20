// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// curve/scalar_mul.rs -- Constant-time scalar multiplication for Ed25519.
// Algorithm: double-and-add from high bit with conditional_select.
// @constant_time: all 255 bits processed unconditionally.
// References: RFC 8032 ss5.1.4, SUPERCOP ref10 scalar reduction.

use crate::curve::point::{ExtendedPoint, IDENTITY};

pub fn reduce_scalar_64(s: &[u8; 64]) -> [u8; 32] {
    let load3 = |b: &[u8], i: usize| -> i128 {
        (b[i] as i128)|((b[i+1] as i128)<<8)|((b[i+2] as i128)<<16) };
    let load4 = |b: &[u8], i: usize| -> i128 {
        (b[i] as i128)|((b[i+1] as i128)<<8)|((b[i+2] as i128)<<16)|((b[i+3] as i128)<<24) };
    let mut s0:  i128 = 2097151 & load3(s, 0);
    let mut s1:  i128 = 2097151 & (load4(s, 2) >> 5);
    let mut s2:  i128 = 2097151 & (load3(s, 5) >> 2);
    let mut s3:  i128 = 2097151 & (load4(s, 7) >> 7);
    let mut s4:  i128 = 2097151 & (load4(s, 10) >> 4);
    let mut s5:  i128 = 2097151 & (load3(s, 13) >> 1);
    let mut s6:  i128 = 2097151 & (load4(s, 15) >> 6);
    let mut s7:  i128 = 2097151 & (load3(s, 18) >> 3);
    let mut s8:  i128 = 2097151 & load3(s, 21);
    let mut s9:  i128 = 2097151 & (load4(s, 23) >> 5);
    let mut s10: i128 = 2097151 & (load3(s, 26) >> 2);
    let mut s11: i128 = 2097151 & (load4(s, 28) >> 7);
    let mut s12: i128 = 2097151 & (load4(s, 31) >> 4);
    let mut s13: i128 = 2097151 & (load3(s, 34) >> 1);
    let mut s14: i128 = 2097151 & (load4(s, 36) >> 6);
    let mut s15: i128 = 2097151 & (load3(s, 39) >> 3);
    let mut s16: i128 = 2097151 & load3(s, 42);
    let mut s17: i128 = 2097151 & (load4(s, 44) >> 5);
    let mut s18: i128 = 2097151 & (load3(s, 47) >> 2);
    let mut s19: i128 = 2097151 & (load4(s, 49) >> 7);
    let mut s20: i128 = 2097151 & (load4(s, 52) >> 4);
    let mut s21: i128 = 2097151 & (load3(s, 55) >> 1);
    let mut s22: i128 = 2097151 & (load4(s, 57) >> 6);
    let mut s23: i128 =            load4(s, 60) >> 3;
    const MU0: i128 =  666643;
    const MU1: i128 =  470296;
    const MU2: i128 =  654183;
    const MU3: i128 = -997805;
    const MU4: i128 =  136657;
    const MU5: i128 = -683901;
    s11+=s23*MU0; s12+=s23*MU1; s13+=s23*MU2; s14+=s23*MU3; s15+=s23*MU4; s16+=s23*MU5; s23=0;
    s10+=s22*MU0; s11+=s22*MU1; s12+=s22*MU2; s13+=s22*MU3; s14+=s22*MU4; s15+=s22*MU5; s22=0;
    s9 +=s21*MU0; s10+=s21*MU1; s11+=s21*MU2; s12+=s21*MU3; s13+=s21*MU4; s14+=s21*MU5; s21=0;
    s8 +=s20*MU0; s9 +=s20*MU1; s10+=s20*MU2; s11+=s20*MU3; s12+=s20*MU4; s13+=s20*MU5; s20=0;
    s7 +=s19*MU0; s8 +=s19*MU1; s9 +=s19*MU2; s10+=s19*MU3; s11+=s19*MU4; s12+=s19*MU5; s19=0;
    s6 +=s18*MU0; s7 +=s18*MU1; s8 +=s18*MU2; s9 +=s18*MU3; s10+=s18*MU4; s11+=s18*MU5; s18=0;
    s5 +=s17*MU0; s6 +=s17*MU1; s7 +=s17*MU2; s8 +=s17*MU3; s9 +=s17*MU4; s10+=s17*MU5; s17=0;
    s4 +=s16*MU0; s5 +=s16*MU1; s6 +=s16*MU2; s7 +=s16*MU3; s8 +=s16*MU4; s9 +=s16*MU5; s16=0;
    s3 +=s15*MU0; s4 +=s15*MU1; s5 +=s15*MU2; s6 +=s15*MU3; s7 +=s15*MU4; s8 +=s15*MU5; s15=0;
    s2 +=s14*MU0; s3 +=s14*MU1; s4 +=s14*MU2; s5 +=s14*MU3; s6 +=s14*MU4; s7 +=s14*MU5; s14=0;
    s1 +=s13*MU0; s2 +=s13*MU1; s3 +=s13*MU2; s4 +=s13*MU3; s5 +=s13*MU4; s6 +=s13*MU5; s13=0;
    s0 +=s12*MU0; s1 +=s12*MU1; s2 +=s12*MU2; s3 +=s12*MU3; s4 +=s12*MU4; s5 +=s12*MU5; s12=0;
    s1 +=s0 >>21; s0 &=0x1fffff; s2 +=s1 >>21; s1 &=0x1fffff;
    s3 +=s2 >>21; s2 &=0x1fffff; s4 +=s3 >>21; s3 &=0x1fffff;
    s5 +=s4 >>21; s4 &=0x1fffff; s6 +=s5 >>21; s5 &=0x1fffff;
    s7 +=s6 >>21; s6 &=0x1fffff; s8 +=s7 >>21; s7 &=0x1fffff;
    s9 +=s8 >>21; s8 &=0x1fffff; s10+=s9 >>21; s9 &=0x1fffff;
    s11+=s10>>21; s10&=0x1fffff; s12+=s11>>21; s11&=0x1fffff;
    s0 +=s12*MU0; s1 +=s12*MU1; s2 +=s12*MU2;
    s3 +=s12*MU3; s4 +=s12*MU4; s5 +=s12*MU5; s12=0;
    s1 +=s0 >>21; s0 &=0x1fffff; s2 +=s1 >>21; s1 &=0x1fffff;
    s3 +=s2 >>21; s2 &=0x1fffff; s4 +=s3 >>21; s3 &=0x1fffff;
    s5 +=s4 >>21; s4 &=0x1fffff; s6 +=s5 >>21; s5 &=0x1fffff;
    s7 +=s6 >>21; s6 &=0x1fffff; s8 +=s7 >>21; s7 &=0x1fffff;
    s9 +=s8 >>21; s8 &=0x1fffff; s10+=s9 >>21; s9 &=0x1fffff;
    s11+=s10>>21; s10&=0x1fffff;
    let mut out = [0u8; 32];
    out[0]  =  (s0)                       as u8;
    out[1]  =  (s0  >>  8)                as u8;
    out[2]  = ((s0  >> 16) | (s1  << 5))  as u8;
    out[3]  =  (s1  >>  3)                as u8;
    out[4]  =  (s1  >> 11)                as u8;
    out[5]  = ((s1  >> 19) | (s2  << 2))  as u8;
    out[6]  =  (s2  >>  6)                as u8;
    out[7]  = ((s2  >> 14) | (s3  << 7))  as u8;
    out[8]  =  (s3  >>  1)                as u8;
    out[9]  =  (s3  >>  9)                as u8;
    out[10] = ((s3  >> 17) | (s4  << 4))  as u8;
    out[11] =  (s4  >>  4)                as u8;
    out[12] =  (s4  >> 12)                as u8;
    out[13] = ((s4  >> 20) | (s5  << 1))  as u8;
    out[14] =  (s5  >>  7)                as u8;
    out[15] = ((s5  >> 15) | (s6  << 6))  as u8;
    out[16] =  (s6  >>  2)                as u8;
    out[17] =  (s6  >> 10)                as u8;
    out[18] = ((s6  >> 18) | (s7  << 3))  as u8;
    out[19] =  (s7  >>  5)                as u8;
    out[20] =  (s7  >> 13)                as u8;
    out[21] =  (s8)                       as u8;
    out[22] =  (s8  >>  8)                as u8;
    out[23] = ((s8  >> 16) | (s9  << 5))  as u8;
    out[24] =  (s9  >>  3)                as u8;
    out[25] =  (s9  >> 11)                as u8;
    out[26] = ((s9  >> 19) | (s10 << 2))  as u8;
    out[27] =  (s10 >>  6)                as u8;
    out[28] = ((s10 >> 14) | (s11 << 7))  as u8;
    out[29] =  (s11 >>  1)                as u8;
    out[30] =  (s11 >>  9)                as u8;
    out[31] =  (s11 >> 17)                as u8;
    out
}


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

    #[test] fn scalar_mul_zero() { assert!(scalar_mul(&BASEPOINT,&[0u8;32]).is_identity()); }
    #[test] fn scalar_mul_one() { assert!(affine_eq(&scalar_mul(&BASEPOINT,&s64(1)),&BASEPOINT)); }
    #[test] fn scalar_mul_two() { assert!(affine_eq(&scalar_mul(&BASEPOINT,&s64(2)),&BASEPOINT.double())); }
    #[test] fn scalar_mul_identity_point() { assert!(scalar_mul(&IDENTITY,&s64(12345)).is_identity()); }
    #[test] fn scalar_mul_additive() {
        assert!(affine_eq(&scalar_mul(&BASEPOINT,&s64(3)),&(BASEPOINT+BASEPOINT+BASEPOINT)));
    }
    #[test] fn scalar_mul_five() {
        assert!(affine_eq(&scalar_mul(&BASEPOINT,&s64(5)),&(BASEPOINT.double().double()+BASEPOINT)));
    }
    #[test] fn scalar_mul_large() {
        let r=scalar_mul(&BASEPOINT,&s64(100));
        let b64=BASEPOINT.double().double().double().double().double().double();
        let b32=BASEPOINT.double().double().double().double().double();
        let b4=BASEPOINT.double().double();
        assert!(affine_eq(&r,&(b64+b32+b4)));
    }
    #[test] fn clamp_clears_low_bits() { let mut s=[0xffu8;32]; clamp_scalar(&mut s); assert_eq!(s[0]&7,0); }
    #[test] fn clamp_sets_bit_254() { let mut s=[0u8;32]; clamp_scalar(&mut s); assert_eq!((s[31]>>6)&1,1); }
    #[test] fn clamp_clears_bit_255() { let mut s=[0xffu8;32]; clamp_scalar(&mut s); assert_eq!(s[31]>>7,0); }
    #[test] fn reduce_scalar_zero() { assert_eq!(reduce_scalar_64(&[0u8;64]),[0u8;32]); }
    #[test] fn reduce_scalar_one() {
        let mut i=[0u8;64]; i[0]=1;
        let r=reduce_scalar_64(&i);
        assert_eq!(r[0],1);
        for j in 1..32{assert_eq!(r[j],0);}
    }
    #[test] fn basepoint_mul_matches_scalar_mul() {
        let s=s64(42);
        assert!(affine_eq(&scalar_mul(&BASEPOINT,&s),&basepoint_mul(&s)));
    }
    #[test]
    fn reduce_scalar_known_vector() {
        let r_hash: [u8; 64] = [
            0x9b,0x20,0xc0,0xeb,0xa3,0x25,0x69,0x10,0xc3,0xce,0xf2,0x8d,0xb5,0xf1,0x02,0x0b,
            0x5f,0x89,0x46,0xc7,0x18,0xf0,0x11,0xce,0x3c,0xa3,0x6a,0x50,0xda,0x17,0x3a,0x02,
            0x35,0xdb,0x91,0x33,0x93,0x19,0x57,0x5a,0xd6,0x67,0x23,0x89,0xe4,0xc6,0x53,0x13,
            0xe2,0x15,0x65,0x9c,0x21,0x6c,0x17,0x18,0x78,0xac,0x28,0xea,0xa5,0x5f,0x1a,0xd2,
        ];
        let expected: [u8; 32] = [
            0x96,0x82,0xd8,0x39,0xce,0x3c,0xb1,0x8b,0x5c,0x88,0x9a,0x0a,0x83,0x05,0x61,0x12,
            0xc0,0x4b,0x33,0x04,0x24,0x7d,0x51,0x53,0x88,0x28,0xd4,0x49,0x1f,0x95,0xe0,0x05,
        ];
        assert_eq!(reduce_scalar_64(&r_hash), expected);
    }
    #[test]
    fn basepoint_mul_known_vector() {
        let r_scalar: [u8; 32] = [
            0x96,0x82,0xd8,0x39,0xce,0x3c,0xb1,0x8b,0x5c,0x88,0x9a,0x0a,0x83,0x05,0x61,0x12,
            0xc0,0x4b,0x33,0x04,0x24,0x7d,0x51,0x53,0x88,0x28,0xd4,0x49,0x1f,0x95,0xe0,0x05,
        ];
        let expected_R: [u8; 32] = [
            0x37,0xb4,0xbd,0x5f,0x28,0xb6,0x1f,0x55,0xdc,0x96,0x73,0xae,0x28,0x95,0xba,0xce,
            0xb8,0x63,0xd9,0xcf,0x51,0x78,0x0d,0x04,0x0f,0x98,0xad,0x8c,0xdc,0x89,0x6c,0xf5,
        ];
        assert_eq!(basepoint_mul(&r_scalar).to_bytes(), expected_R);
    }
}
