// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "backend-posix")]
pub mod posix;

#[cfg(feature = "backend-sel4")]
pub mod sel4;

#[cfg(feature = "backend-posix")]
pub use posix as active;

#[cfg(feature = "backend-sel4")]
pub use sel4 as active;
