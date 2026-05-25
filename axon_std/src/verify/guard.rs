//! DVG — Dependent Variable Guards (P6+ feature).
//!
//! Guards enforce that a dependent variable (one whose correctness
//! depends on another variable's value) is only used after its
//! dependency has been verified.

use super::check::{VerificationError, VerifyResult, fnv64};

/// A violation of a dependent variable guard.
#[derive(Debug, Clone)]
pub struct GuardViolation {
    /// The variable that was accessed without its dependency being satisfied.
    pub variable:   &'static str,
    /// The dependency that was not satisfied.
    pub dependency: &'static str,
}

impl std::fmt::Display for GuardViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "DVG violation: '{}' accessed before dependency '{}' was satisfied",
            self.variable, self.dependency
        )
    }
}

/// A guard that enforces dependency between two variables.
///
/// # P6+ DVG
///
/// In AXON, `@depends_on(x)` on a variable `y` means `y` must not
/// be used unless `x` has been verified as valid. `DependentGuard`
/// enforces this at runtime.
///
/// # Examples
///
/// ```rust
/// use axon_std::verify::guard::DependentGuard;
///
/// let mut guard = DependentGuard::new("output", "input_validated");
/// guard.satisfy_dependency(); // input was validated
/// assert!(guard.check_access().is_ok());
/// ```
#[derive(Debug)]
pub struct DependentGuard {
    variable:            &'static str,
    dependency:          &'static str,
    dependency_satisfied: bool,
}

impl DependentGuard {
    /// Create a new guard: `variable` depends on `dependency`.
    pub fn new(variable: &'static str, dependency: &'static str) -> Self {
        Self { variable, dependency, dependency_satisfied: false }
    }

    /// Mark the dependency as satisfied (e.g. after validation).
    pub fn satisfy_dependency(&mut self) {
        self.dependency_satisfied = true;
    }

    /// Check that the dependency is satisfied before accessing the variable.
    pub fn check_access(&self) -> VerifyResult<()> {
        if self.dependency_satisfied {
            Ok(())
        } else {
            Err(VerificationError {
                label: self.variable,
                description: format!(
                    "DVG: '{}' accessed before dependency '{}' was satisfied",
                    self.variable, self.dependency
                ),
                fn_hash: fnv64(self.variable.as_bytes()),
            })
        }
    }

    /// Returns true if the dependency is satisfied.
    pub fn is_satisfied(&self) -> bool { self.dependency_satisfied }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_blocks_before_satisfaction() {
        let g = DependentGuard::new("output", "input_validated");
        assert!(g.check_access().is_err());
    }

    #[test]
    fn guard_allows_after_satisfaction() {
        let mut g = DependentGuard::new("output", "input_validated");
        g.satisfy_dependency();
        assert!(g.check_access().is_ok());
    }

    #[test]
    fn guard_is_satisfied_reflects_state() {
        let mut g = DependentGuard::new("y", "x");
        assert!(!g.is_satisfied());
        g.satisfy_dependency();
        assert!(g.is_satisfied());
    }

    #[test]
    fn guard_violation_display() {
        let v = GuardViolation { variable: "y", dependency: "x" };
        let s = format!("{v}");
        assert!(s.contains("y"));
        assert!(s.contains("x"));
    }
}
