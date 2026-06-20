// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Secret<T> — cryptographic data with zeroize-on-drop.
// Mirrors secret.ax (sovereign source of truth).
// Cannot be cloned without explicit declassify().
// Cannot be printed via Display/Debug (shows "Secret<*>").
// Zeroized to zero bytes on drop.

use std::fmt;

/// A secret value that is zeroized on drop.
/// T must be: Default (for zeroing) + sized.
pub struct Secret<T: Zeroize> {
    inner: T,
}

/// Trait for types that can be securely zeroed.
pub trait Zeroize {
    fn zeroize(&mut self);
}

impl Zeroize for Vec<u8> {
    fn zeroize(&mut self) {
        for b in self.iter_mut() { *b = 0; }
        self.clear();
    }
}

impl Zeroize for [u8; 32] {
    fn zeroize(&mut self) { self.iter_mut().for_each(|b| *b = 0); }
}

impl Zeroize for [u8; 64] {
    fn zeroize(&mut self) { self.iter_mut().for_each(|b| *b = 0); }
}

impl Zeroize for String {
    fn zeroize(&mut self) {
        // Safety: overwrite bytes then clear
        unsafe {
            let v = self.as_bytes_mut();
            for b in v.iter_mut() { *b = 0; }
        }
        self.clear();
    }
}

impl Zeroize for i64 {
    fn zeroize(&mut self) { *self = 0; }
}

impl<T: Zeroize> Secret<T> {
    /// Wrap a value as a Secret.
    pub fn new(inner: T) -> Self { Secret { inner } }

    /// Declassify: expose the inner value for explicit use.
    /// Caller must ensure the value is not logged or transmitted.
    pub fn declassify(&self) -> &T { &self.inner }

    /// Declassify mutably.
    pub fn declassify_mut(&mut self) -> &mut T { &mut self.inner }
}

/// Drop zeroizes the inner value.
impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

/// Debug never reveals the inner value.
impl<T: Zeroize> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret<*>")
    }
}

/// Display never reveals the inner value.
impl<T: Zeroize> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret<*>")
    }
}

