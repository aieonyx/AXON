// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// ed25519.rs -- RFC 8032 Ed25519 sign/verify. P57.1: full curve arithmetic.
// Keygen: SHA-512(seed) -> clamp -> basepoint_mul -> public key
// Sign:   deterministic nonce, RFC 8032 ss5.1.6
// Verify: S*B == R + k*A, RFC 8032 ss5.1.7

use crate::sha512::sha512;
use crate::curve::scalar_mul::{scalar_mul, basepoint_mul, clamp_scalar, reduce_scalar_64};
use crate::curve::point::ExtendedPoint;

#[derive(Debug, Clone)]
pub struct Ed25519PublicKey { bytes: [u8; 32] }

impl Ed25519PublicKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self { Ed25519PublicKey { bytes } }
    pub fn as_bytes(&self) -> &[u8] { &self.bytes }
    pub fn to_bytes(&self) -> [u8; 32] { self.bytes }

    pub fn verify(&self, msg: &[u8], sig: &[u8; 64]) -> bool {
        let r_bytes: [u8; 32] = sig[..32].try_into().unwrap();
        let r_point = match ExtendedPoint::from_bytes(&r_bytes) { Some(p) => p, None => return false };
        let a_point = match ExtendedPoint::from_bytes(&self.bytes) { Some(p) => p, None => return false };
        let s_bytes: [u8; 32] = sig[32..].try_into().unwrap();
        if s_bytes[31] & 0xe0 != 0 { return false; }
        let k_hash   = sha512_three(&r_bytes, &self.bytes, msg);
        let k_scalar = reduce_scalar_64(&k_hash);
        let sb  = basepoint_mul(&s_bytes);
        let ka  = scalar_mul(&a_point, &k_scalar);
        let rka = r_point + ka;
        sb.to_bytes() == rka.to_bytes()
    }
}

#[derive(Debug)]
pub struct Ed25519KeyPair { seed: [u8; 32], public_key: Ed25519PublicKey }

impl Ed25519KeyPair {
    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        fill_random(&mut seed);
        Self::from_seed(seed)
    }

    pub fn from_seed(seed: [u8; 32]) -> Self {
        let h = sha512(&seed);
        let mut scalar = [0u8; 32];
        scalar.copy_from_slice(&h[..32]);
        clamp_scalar(&mut scalar);
        let pub_bytes = basepoint_mul(&scalar).to_bytes();
        Ed25519KeyPair { seed, public_key: Ed25519PublicKey::from_bytes(pub_bytes) }
    }

    pub fn public_key(&self) -> Ed25519PublicKey { self.public_key.clone() }
    pub fn seed(&self) -> &[u8; 32] { &self.seed }

    pub fn sign(&self, msg: &[u8]) -> [u8; 64] {
        let h = sha512(&self.seed);
        let mut scalar = [0u8; 32];
        scalar.copy_from_slice(&h[..32]);
        clamp_scalar(&mut scalar);
        let prefix  = &h[32..];
        let r_hash  = sha512_two(prefix, msg);
        let r_scalar = reduce_scalar_64(&r_hash);
        let r_bytes = basepoint_mul(&r_scalar).to_bytes();
        let pk_bytes = self.public_key.to_bytes();
        let k_hash  = sha512_three(&r_bytes, &pk_bytes, msg);
        let k_scalar = reduce_scalar_64(&k_hash);
        let s_scalar = scalar_add_mul_mod_l(&r_scalar, &k_scalar, &scalar);
        let mut sig = [0u8; 64];
        sig[..32].copy_from_slice(&r_bytes);
        sig[32..].copy_from_slice(&s_scalar);
        sig
    }
}

fn sha512_two(a: &[u8], b: &[u8]) -> [u8; 64] {
    let mut buf = Vec::with_capacity(a.len() + b.len());
    buf.extend_from_slice(a); buf.extend_from_slice(b);
    sha512(&buf)
}

fn sha512_three(a: &[u8], b: &[u8], c: &[u8]) -> [u8; 64] {
    let mut buf = Vec::with_capacity(a.len() + b.len() + c.len());
    buf.extend_from_slice(a); buf.extend_from_slice(b); buf.extend_from_slice(c);
    sha512(&buf)
}

fn scalar_add_mul_mod_l(r: &[u8; 32], k: &[u8; 32], a: &[u8; 32]) -> [u8; 32] {
    let load3 = |b: &[u8], i: usize| -> i128 {
        (b[i] as i128)|((b[i+1] as i128)<<8)|((b[i+2] as i128)<<16) };
    let load4 = |b: &[u8], i: usize| -> i128 {
        (b[i] as i128)|((b[i+1] as i128)<<8)|((b[i+2] as i128)<<16)|((b[i+3] as i128)<<24) };
    let r0:  i128 = 2097151&load3(r,0);  let r1:  i128 = 2097151&(load4(r,2)>>5);
    let r2:  i128 = 2097151&(load3(r,5)>>2); let r3:  i128 = 2097151&(load4(r,7)>>7);
    let r4:  i128 = 2097151&(load4(r,10)>>4); let r5:  i128 = 2097151&(load3(r,13)>>1);
    let r6:  i128 = 2097151&(load4(r,15)>>6); let r7:  i128 = 2097151&(load3(r,18)>>3);
    let r8:  i128 = 2097151&load3(r,21); let r9:  i128 = 2097151&(load4(r,23)>>5);
    let r10: i128 = 2097151&(load3(r,26)>>2); let r11: i128 = load4(r,28)>>7;
    let k0:  i128 = 2097151&load3(k,0);  let k1:  i128 = 2097151&(load4(k,2)>>5);
    let k2:  i128 = 2097151&(load3(k,5)>>2); let k3:  i128 = 2097151&(load4(k,7)>>7);
    let k4:  i128 = 2097151&(load4(k,10)>>4); let k5:  i128 = 2097151&(load3(k,13)>>1);
    let k6:  i128 = 2097151&(load4(k,15)>>6); let k7:  i128 = 2097151&(load3(k,18)>>3);
    let k8:  i128 = 2097151&load3(k,21); let k9:  i128 = 2097151&(load4(k,23)>>5);
    let k10: i128 = 2097151&(load3(k,26)>>2); let k11: i128 = load4(k,28)>>7;
    let a0:  i128 = 2097151&load3(a,0);  let a1:  i128 = 2097151&(load4(a,2)>>5);
    let a2:  i128 = 2097151&(load3(a,5)>>2); let a3:  i128 = 2097151&(load4(a,7)>>7);
    let a4:  i128 = 2097151&(load4(a,10)>>4); let a5:  i128 = 2097151&(load3(a,13)>>1);
    let a6:  i128 = 2097151&(load4(a,15)>>6); let a7:  i128 = 2097151&(load3(a,18)>>3);
    let a8:  i128 = 2097151&load3(a,21); let a9:  i128 = 2097151&(load4(a,23)>>5);
    let a10: i128 = 2097151&(load3(a,26)>>2); let a11: i128 = load4(a,28)>>7;
    let mut s0:  i128 = r0 +k0*a0;
    let mut s1:  i128 = r1 +k0*a1 +k1*a0;
    let mut s2:  i128 = r2 +k0*a2 +k1*a1 +k2*a0;
    let mut s3:  i128 = r3 +k0*a3 +k1*a2 +k2*a1 +k3*a0;
    let mut s4:  i128 = r4 +k0*a4 +k1*a3 +k2*a2 +k3*a1 +k4*a0;
    let mut s5:  i128 = r5 +k0*a5 +k1*a4 +k2*a3 +k3*a2 +k4*a1 +k5*a0;
    let mut s6:  i128 = r6 +k0*a6 +k1*a5 +k2*a4 +k3*a3 +k4*a2 +k5*a1 +k6*a0;
    let mut s7:  i128 = r7 +k0*a7 +k1*a6 +k2*a5 +k3*a4 +k4*a3 +k5*a2 +k6*a1 +k7*a0;
    let mut s8:  i128 = r8 +k0*a8 +k1*a7 +k2*a6 +k3*a5 +k4*a4 +k5*a3 +k6*a2 +k7*a1 +k8*a0;
    let mut s9:  i128 = r9 +k0*a9 +k1*a8 +k2*a7 +k3*a6 +k4*a5 +k5*a4 +k6*a3 +k7*a2 +k8*a1 +k9*a0;
    let mut s10: i128 = r10+k0*a10+k1*a9 +k2*a8 +k3*a7 +k4*a6 +k5*a5 +k6*a4 +k7*a3 +k8*a2 +k9*a1 +k10*a0;
    let mut s11: i128 = r11+k0*a11+k1*a10+k2*a9 +k3*a8 +k4*a7 +k5*a6 +k6*a5 +k7*a4 +k8*a3 +k9*a2 +k10*a1+k11*a0;
    let mut s12: i128 =     k1*a11+k2*a10+k3*a9 +k4*a8 +k5*a7 +k6*a6 +k7*a5 +k8*a4 +k9*a3 +k10*a2+k11*a1;
    let mut s13: i128 =     k2*a11+k3*a10+k4*a9 +k5*a8 +k6*a7 +k7*a6 +k8*a5 +k9*a4 +k10*a3+k11*a2;
    let mut s14: i128 =     k3*a11+k4*a10+k5*a9 +k6*a8 +k7*a7 +k8*a6 +k9*a5 +k10*a4+k11*a3;
    let mut s15: i128 =     k4*a11+k5*a10+k6*a9 +k7*a8 +k8*a7 +k9*a6 +k10*a5+k11*a4;
    let mut s16: i128 =     k5*a11+k6*a10+k7*a9 +k8*a8 +k9*a7 +k10*a6+k11*a5;
    let mut s17: i128 =     k6*a11+k7*a10+k8*a9 +k9*a8 +k10*a7+k11*a6;
    let mut s18: i128 =     k7*a11+k8*a10+k9*a9 +k10*a8+k11*a7;
    let mut s19: i128 =     k8*a11+k9*a10+k10*a9+k11*a8;
    let mut s20: i128 =     k9*a11+k10*a10+k11*a9;
    let mut s21: i128 =     k10*a11+k11*a10;
    let mut s22: i128 =     k11*a11;
    let s23: i128 = 0;
    const MU0: i128 =  666643;
    const MU1: i128 =  470296;
    const MU2: i128 =  654183;
    const MU3: i128 = -997805;
    const MU4: i128 =  136657;
    const MU5: i128 = -683901;
    s11+=s23*MU0; s12+=s23*MU1; s13+=s23*MU2; s14+=s23*MU3; s15+=s23*MU4; s16+=s23*MU5;
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


fn fill_random(buf: &mut [u8; 32]) {
    use std::fs::File; use std::io::Read;
    File::open("/dev/urandom").unwrap().read_exact(buf).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hex(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
    fn hex32(s:&str)->[u8;32]{hex(s).try_into().unwrap()}
    fn hex64(s:&str)->[u8;64]{hex(s).try_into().unwrap()}

    const V1_SEED:&str="9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae3d55";
    const V1_PK:&str  ="700e2ce7c4b674427eab27ba820bcf6f0faebe68e09fe8564292114e41dc6a41";
    const V1_SIG:&str ="37b4bd5f28b61f55dc9673ae2895baceb863d9cf51780d040f98ad8cdc896cf5be46be655a863525da0959f7f373611585e437e28ec971b7bd206ff9bd26e803";
    const V2_SEED:&str="4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4d0bd6f3";
    const V2_MSG:&str ="72";
    const V2_PK:&str  ="d3edf21c525dc8227e660a03629b29f43377183878131d7dcf19334725fb441a";
    const V2_SIG:&str ="02fa4f037164373c8b6185332d38f1759db0e81722ecc28944c2a46477dcfdb1c8e388379f97210b120ffbf403c611029ed5442a2f8abcff159d527f92e4720d";
    const V3_SEED:&str="c5aa8df43f9f837bedb7442f31dcb7b166d38535076f094b85ce3a2e0b4458f7";
    const V3_MSG:&str ="af82";
    const V3_PK:&str  ="fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025";
    const V3_SIG:&str ="6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290ae67f760984dc6594a7c15e9716ed28dc027beceea1ec40a";

    #[test] fn keygen_v1(){assert_eq!(Ed25519KeyPair::from_seed(hex32(V1_SEED)).public_key().to_bytes(),hex32(V1_PK));}
    #[test] fn keygen_v2(){assert_eq!(Ed25519KeyPair::from_seed(hex32(V2_SEED)).public_key().to_bytes(),hex32(V2_PK));}
    #[test] fn keygen_v3_rfc8032(){assert_eq!(Ed25519KeyPair::from_seed(hex32(V3_SEED)).public_key().to_bytes(),hex32(V3_PK));}
    #[test] fn keygen_deterministic(){let s=hex32(V1_SEED);assert_eq!(Ed25519KeyPair::from_seed(s).public_key().to_bytes(),Ed25519KeyPair::from_seed(s).public_key().to_bytes());}
    #[test] fn sign_v1(){assert_eq!(Ed25519KeyPair::from_seed(hex32(V1_SEED)).sign(b""),hex64(V1_SIG));}
    #[test] fn sign_v2(){assert_eq!(Ed25519KeyPair::from_seed(hex32(V2_SEED)).sign(&hex(V2_MSG)),hex64(V2_SIG));}
    #[test] fn sign_v3_rfc8032(){assert_eq!(Ed25519KeyPair::from_seed(hex32(V3_SEED)).sign(&hex(V3_MSG)),hex64(V3_SIG));}
    #[test] fn sign_deterministic(){let kp=Ed25519KeyPair::from_seed(hex32(V1_SEED));assert_eq!(kp.sign(b"t"),kp.sign(b"t"));}
    #[test] fn sign_different_msgs(){let kp=Ed25519KeyPair::from_seed(hex32(V1_SEED));assert_ne!(kp.sign(b"a"),kp.sign(b"b"));}
    #[test] fn verify_v1(){assert!(Ed25519PublicKey::from_bytes(hex32(V1_PK)).verify(b"",&hex64(V1_SIG)));}
    #[test] fn verify_v2(){assert!(Ed25519PublicKey::from_bytes(hex32(V2_PK)).verify(&hex(V2_MSG),&hex64(V2_SIG)));}
    #[test] fn verify_v3_rfc8032(){assert!(Ed25519PublicKey::from_bytes(hex32(V3_PK)).verify(&hex(V3_MSG),&hex64(V3_SIG)));}
    #[test] fn verify_roundtrip(){let kp=Ed25519KeyPair::from_seed(hex32(V1_SEED));let s=kp.sign(b"sovereign");assert!(kp.public_key().verify(b"sovereign",&s));}
    #[test] fn verify_wrong_msg(){assert!(!Ed25519PublicKey::from_bytes(hex32(V3_PK)).verify(b"wrong",&hex64(V3_SIG)));}
    #[test] fn verify_tampered(){let kp=Ed25519KeyPair::from_seed(hex32(V1_SEED));let mut s=kp.sign(b"t");s[0]^=1;assert!(!kp.public_key().verify(b"t",&s));}
    #[test] fn verify_wrong_key(){let k1=Ed25519KeyPair::from_seed(hex32(V1_SEED));let k2=Ed25519KeyPair::from_seed(hex32(V2_SEED));let s=k1.sign(b"x");assert!(!k2.public_key().verify(b"x",&s));}
    #[test] fn generate_roundtrip(){let kp=Ed25519KeyPair::generate();let s=kp.sign(b"AIEONYX");assert!(kp.public_key().verify(b"AIEONYX",&s));}
}
