// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// SafeInt — bounds-checked integer.
// Mirrors safeint.ax (sovereign source of truth).
// Panics on overflow or out-of-range assignment.
// Never wraps. Never silently truncates.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SafeInt {
    pub lo:    i64,
    pub hi:    i64,
    pub value: i64,
}

impl SafeInt {
    /// Create a SafeInt with given bounds and initial value.
    /// Panics if value is out of [lo, hi].
    pub fn new(lo: i64, hi: i64, value: i64) -> Self {
        assert!(lo <= hi, "SafeInt::new: lo > hi");
        assert!(value >= lo && value <= hi,
            "SafeInt::new: value {} out of range [{}, {}]", value, lo, hi);
        SafeInt { lo, hi, value }
    }

    /// Add checked within bounds. Panics on overflow or range violation.
    pub fn add(&self, rhs: i64) -> SafeInt {
        let result = self.value.checked_add(rhs)
            .expect("SafeInt::add: arithmetic overflow");
        assert!(result >= self.lo && result <= self.hi,
            "SafeInt::add: result {} out of range [{}, {}]", result, self.lo, self.hi);
        SafeInt { lo: self.lo, hi: self.hi, value: result }
    }

    /// Subtract checked within bounds. Panics on overflow or range violation.
    pub fn sub(&self, rhs: i64) -> SafeInt {
        let result = self.value.checked_sub(rhs)
            .expect("SafeInt::sub: arithmetic overflow");
        assert!(result >= self.lo && result <= self.hi,
            "SafeInt::sub: result {} out of range [{}, {}]", result, self.lo, self.hi);
        SafeInt { lo: self.lo, hi: self.hi, value: result }
    }

    /// Multiply checked within bounds. Panics on overflow or range violation.
    pub fn mul(&self, rhs: i64) -> SafeInt {
        let result = self.value.checked_mul(rhs)
            .expect("SafeInt::mul: arithmetic overflow");
        assert!(result >= self.lo && result <= self.hi,
            "SafeInt::mul: result {} out of range [{}, {}]", result, self.lo, self.hi);
        SafeInt { lo: self.lo, hi: self.hi, value: result }
    }

    /// Set a new value within bounds. Panics if out of range.
    pub fn set(&mut self, value: i64) {
        assert!(value >= self.lo && value <= self.hi,
            "SafeInt::set: value {} out of range [{}, {}]", value, self.lo, self.hi);
        self.value = value;
    }

    pub fn get(&self) -> i64 { self.value }
}
