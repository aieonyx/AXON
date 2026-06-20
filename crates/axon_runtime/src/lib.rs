// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_runtime — sovereign type runtime enforcement.
// P55.6: Money<T>, Secret<T>, SafeInt.
// P55.7: @constant_time, @sealed_memory, domain Finance CCP profile.

pub mod money;
pub mod safeint;
pub mod secret;

pub use money::{Money, assert_balanced};
pub use safeint::SafeInt;
pub use secret::{Secret, Zeroize};
