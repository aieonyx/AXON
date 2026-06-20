// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// curve/mod.rs -- Twisted Edwards curve point arithmetic module.
// M2 of P57.1: ExtendedPoint type, unified addition, doubling, encode/decode.

pub mod point;
pub mod point_ops;
pub mod scalar_mul;

pub use point::ExtendedPoint;
pub use scalar_mul::{scalar_mul, basepoint_mul, clamp_scalar, reduce_scalar_64};
pub use point::{BASEPOINT, IDENTITY};
