// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// curve/point.rs -- Extended twisted Edwards point for Ed25519.
// Curve equation: -x^2 + y^2 = 1 + d*x^2*y^2  (a=-1, d=-121665/121666 mod p)
// Extended coordinates: (X:Y:Z:T) where x=X/Z, y=Y/Z, x*y=T/Z.
// @constant_time: no secret-dependent branches anywhere in this file.
// References: RFC 8032 §5.1, Hisil et al. 2008.

use crate::field::Fe25519;

pub const BASEPOINT: ExtendedPoint = ExtendedPoint {
    X: Fe25519([1738742601995546, 1146398526822698, 2070867633025821, 562264141797630,  587772402128613]),
    Y: Fe25519([1801439850948184, 1351079888211148,  450359962737049, 900719925474099, 1801439850948198]),
    Z: Fe25519::ONE,
    T: Fe25519([1841354044333475,   16398895984059,  755974180946558, 900171276175154, 1821297809914039]),
};

pub const IDENTITY: ExtendedPoint = ExtendedPoint {
    X: Fe25519::ZERO,
    Y: Fe25519::ONE,
    Z: Fe25519::ONE,
    T: Fe25519::ZERO,
};

#[derive(Clone, Copy, Debug)]
#[allow(non_snake_case)]
pub struct ExtendedPoint {
    pub(crate) X: Fe25519,
    pub(crate) Y: Fe25519,
    pub(crate) Z: Fe25519,
    pub(crate) T: Fe25519,
}

impl ExtendedPoint {
    pub fn identity() -> Self { IDENTITY }
    pub fn basepoint() -> Self { BASEPOINT }

    pub fn is_identity(&self) -> bool {
        let x_zero = self.X.is_zero();
        let yz_eq  = (self.Y - self.Z).is_zero();
        x_zero && yz_eq
    }

    pub fn negate(&self) -> Self {
        ExtendedPoint { X: -self.X, Y: self.Y, Z: self.Z, T: -self.T }
    }

    pub fn conditional_select(a: &Self, b: &Self, choice: u64) -> Self {
        ExtendedPoint {
            X: Fe25519::conditional_select(&a.X, &b.X, choice),
            Y: Fe25519::conditional_select(&a.Y, &b.Y, choice),
            Z: Fe25519::conditional_select(&a.Z, &b.Z, choice),
            T: Fe25519::conditional_select(&a.T, &b.T, choice),
        }
    }

    pub fn to_affine(&self) -> (Fe25519, Fe25519) {
        let z_inv = self.Z.invert_fermat();
        let x = (self.X * z_inv).reduce_canonical();
        let y = (self.Y * z_inv).reduce_canonical();
        (x, y)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let (x, y) = self.to_affine();
        let mut out = y.to_bytes();
        let x_sign = x.is_negative() as u8;
        out[31] |= x_sign << 7;
        out
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        let mut y_bytes = *bytes;
        let x_sign = (y_bytes[31] >> 7) & 1;
        y_bytes[31] &= 0x7f;
        let y  = Fe25519::from_bytes(&y_bytes);
        let y2 = y.square();
        let u  = y2 - Fe25519::ONE;
        let v  = (Fe25519::D * y2).carry_reduce() + Fe25519::ONE;
        let (was_square, mut x) = Fe25519::sqrt_ratio_i(u, v);
        if was_square == 0 { return None; }
        let x_is_neg = x.is_negative();
        let neg_x = -x;
        x = Fe25519::conditional_select(&x, &neg_x, x_is_neg ^ (x_sign as u64));
        if x.is_zero() && x_sign == 1 { return None; }
        let t = x * y;
        Some(ExtendedPoint { X: x, Y: y, Z: Fe25519::ONE, T: t })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_identity() {
        assert!(IDENTITY.is_identity());
    }

    #[test]
    fn basepoint_not_identity() {
        assert!(!BASEPOINT.is_identity());
    }

    #[test]
    fn basepoint_encode_decode_roundtrip() {
        let encoded = BASEPOINT.to_bytes();
        let decoded = ExtendedPoint::from_bytes(&encoded).expect("basepoint decode failed");
        let (x1, y1) = BASEPOINT.to_affine();
        let (x2, y2) = decoded.to_affine();
        assert_eq!(x1.ct_eq(&x2), 1);
        assert_eq!(y1.ct_eq(&y2), 1);
    }

    #[test]
    fn basepoint_encoded_matches_rfc8032() {
        let encoded = BASEPOINT.to_bytes();
        let expected = [
            0x58u8, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
        ];
        assert_eq!(encoded, expected);
    }

    #[test]
    fn negate_then_check() {
        assert!(!BASEPOINT.negate().is_identity());
    }

    #[test]
    fn conditional_select_choice_0() {
        let r = ExtendedPoint::conditional_select(&IDENTITY, &BASEPOINT, 0);
        assert!(r.is_identity());
    }

    #[test]
    fn conditional_select_choice_1() {
        let r = ExtendedPoint::conditional_select(&IDENTITY, &BASEPOINT, 1);
        assert!(!r.is_identity());
    }
}
