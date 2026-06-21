// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// curve/point_ops.rs -- Unified addition and doubling for Ed25519.
// Formula: Hisil et al. 2008 §3.1. Unified -- handles all cases, no branches.

use crate::field::Fe25519;
use crate::curve::point::ExtendedPoint;
use core::ops::Add;

impl Add for ExtendedPoint {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        let (x1, y1, z1, t1) = (self.X, self.Y, self.Z, self.T);
        let (x2, y2, z2, t2) = (rhs.X,  rhs.Y,  rhs.Z,  rhs.T);
        let a = (y1 - x1) * (y2 - x2);
        let b = (y1 + x1) * (y2 + x2);
        let c = t1 * Fe25519::D2 * t2;
        let d = z1 * z2;
        let d = d + d;
        let e = b - a;
        let f = d - c;
        let g = d + c;
        let h = b + a;
        ExtendedPoint { X: e * f, Y: g * h, Z: f * g, T: e * h }
    }
}

impl ExtendedPoint {
    pub fn double(&self) -> Self {
        let x1 = self.X;
        let y1 = self.Y;
        let z1 = self.Z;
        let a  = x1.square();
        let b  = y1.square();
        let c  = z1.square();
        let c  = c + c;
        let h  = a + b;
        let xy = x1 + y1;
        let e  = h - xy.square();
        let g  = a - b;
        let f  = c + g;
        ExtendedPoint { X: e * f, Y: g * h, Z: f * g, T: e * h }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::point::{BASEPOINT, IDENTITY};

    fn scalar_mul_naive(p: ExtendedPoint, n: u64) -> ExtendedPoint {
        let mut r = IDENTITY;
        let mut q = p;
        let mut k = n;
        while k > 0 {
            if k & 1 == 1 { r = r + q; }
            q = q.double();
            k >>= 1;
        }
        r
    }

    fn affine_eq(a: &ExtendedPoint, b: &ExtendedPoint) -> bool {
        let (ax, ay) = a.to_affine();
        let (bx, by) = b.to_affine();
        ax.ct_eq(&bx) == 1 && ay.ct_eq(&by) == 1
    }

    #[test]
    fn add_identity_right() {
        assert!(affine_eq(&(BASEPOINT + IDENTITY), &BASEPOINT));
    }

    #[test]
    fn add_identity_left() {
        assert!(affine_eq(&(IDENTITY + BASEPOINT), &BASEPOINT));
    }

    #[test]
    fn add_identity_self() {
        assert!((IDENTITY + IDENTITY).is_identity());
    }

    #[test]
    fn add_commutative() {
        let two_b = BASEPOINT.double();
        assert!(affine_eq(&(BASEPOINT + two_b), &(two_b + BASEPOINT)));
    }

    #[test]
    fn add_neg_is_identity() {
        assert!((BASEPOINT + BASEPOINT.negate()).is_identity());
    }

    #[test]
    fn add_vs_double() {
        assert!(affine_eq(&(BASEPOINT + BASEPOINT), &BASEPOINT.double()));
    }

    #[test]
    fn add_associative() {
        let b2 = BASEPOINT.double();
        let b3 = b2 + BASEPOINT;
        assert!(affine_eq(&((BASEPOINT + b2) + b3), &(BASEPOINT + (b2 + b3))));
    }

    #[test]
    fn double_identity() {
        assert!(IDENTITY.double().is_identity());
    }

    #[test]
    fn double_matches_naive_2b() {
        assert!(affine_eq(&BASEPOINT.double(), &scalar_mul_naive(BASEPOINT, 2)));
    }

    #[test]
    fn double_double_matches_4b() {
        assert!(affine_eq(&BASEPOINT.double().double(), &scalar_mul_naive(BASEPOINT, 4)));
    }

    #[test]
    fn scalar_8b_not_identity() {
        assert!(!scalar_mul_naive(BASEPOINT, 8).is_identity());
    }

    #[test]
    fn scalar_add_consistency() {
        let three_b_naive = scalar_mul_naive(BASEPOINT, 3);
        let three_b_add   = BASEPOINT + BASEPOINT + BASEPOINT;
        assert!(affine_eq(&three_b_naive, &three_b_add));
    }

    #[test]
    fn double_then_add_consistency() {
        let five_b_naive = scalar_mul_naive(BASEPOINT, 5);
        let five_b = BASEPOINT.double().double() + BASEPOINT;
        assert!(affine_eq(&five_b_naive, &five_b));
    }

    #[test]
    fn two_b_y_coordinate() {
        let (_, y) = BASEPOINT.double().to_affine();
        let known: [u8; 32] = [
            0xc9, 0xa3, 0xf8, 0x6a, 0xae, 0x46, 0x5f, 0x0e,
            0x56, 0x51, 0x38, 0x64, 0x51, 0x0f, 0x39, 0x97,
            0x56, 0x1f, 0xa2, 0xc9, 0xe8, 0x5e, 0xa2, 0x1d,
            0xc2, 0x29, 0x23, 0x09, 0xf3, 0xcd, 0x60, 0x22,
        ];
        assert_eq!(y.to_bytes(), known);
    }
}
