// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Money<T> — decimal-safe fixed-point arithmetic.
// Mirrors money.ax (sovereign source of truth).
// No f32/f64 permitted. All amounts stored as i64 cents at given precision.
// Overflow panics — never wraps, never silently corrupts.

/// Fixed-point monetary value.
/// `precision` = number of decimal places (e.g. 2 for EUR cents).
/// `amount` = value * 10^precision (e.g. 19.99 EUR = 1999 with precision 2).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money {
    pub currency:  &'static str,
    pub precision: u8,
    pub amount:    i64,  // fixed-point: real_value * 10^precision
}

impl Money {
    /// Create a Money value from a whole + fractional part.
    /// e.g. Money::new("EUR", 2, 19, 99) = 19.99 EUR
    pub fn new(currency: &'static str, precision: u8, whole: i64, frac: i64) -> Self {
        let scale = 10_i64.pow(precision as u32);
        let amount = whole.checked_mul(scale)
            .and_then(|w| w.checked_add(frac))
            .expect("Money::new: overflow");
        Money { currency, precision, amount }
    }

    /// Create from raw fixed-point amount.
    pub fn from_raw(currency: &'static str, precision: u8, amount: i64) -> Self {
        Money { currency, precision, amount }
    }

    /// Add two Money values. Panics on overflow or currency mismatch.
    pub fn add(&self, other: &Money) -> Money {
        assert_eq!(self.currency, other.currency, "Money::add: currency mismatch");
        assert_eq!(self.precision, other.precision, "Money::add: precision mismatch");
        Money {
            currency:  self.currency,
            precision: self.precision,
            amount:    self.amount.checked_add(other.amount)
                .expect("Money::add: overflow"),
        }
    }

    /// Subtract two Money values. Panics on overflow or currency mismatch.
    pub fn sub(&self, other: &Money) -> Money {
        assert_eq!(self.currency, other.currency, "Money::sub: currency mismatch");
        assert_eq!(self.precision, other.precision, "Money::sub: precision mismatch");
        Money {
            currency:  self.currency,
            precision: self.precision,
            amount:    self.amount.checked_sub(other.amount)
                .expect("Money::sub: overflow"),
        }
    }

    /// Multiply by an integer scalar. Panics on overflow.
    pub fn mul_scalar(&self, scalar: i64) -> Money {
        Money {
            currency:  self.currency,
            precision: self.precision,
            amount:    self.amount.checked_mul(scalar)
                .expect("Money::mul_scalar: overflow"),
        }
    }

    /// Returns true if this value is zero.
    pub fn is_zero(&self) -> bool { self.amount == 0 }

    /// Returns true if this value is negative.
    pub fn is_negative(&self) -> bool { self.amount < 0 }

    /// Format as decimal string e.g. "19.99 EUR"
    pub fn display(&self) -> String {
        let scale  = 10_i64.pow(self.precision as u32);
        let whole  = self.amount / scale;
        let frac   = (self.amount % scale).abs();
        format!("{}.{:0>width$} {}", whole, frac,
            self.currency, width = self.precision as usize)
    }
}

/// Double-entry balance check: sum of debits must equal sum of credits.
/// Returns Ok(()) if balanced, Err with discrepancy amount if not.
pub fn assert_balanced(debits: &[Money], credits: &[Money]) -> Result<(), i64> {
    let debit_sum: i64  = debits.iter().map(|m| m.amount).sum();
    let credit_sum: i64 = credits.iter().map(|m| m.amount).sum();
    if debit_sum == credit_sum { Ok(()) }
    else { Err(debit_sum - credit_sum) }
}
