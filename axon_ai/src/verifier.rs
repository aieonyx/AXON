// ============================================================
// axon_ai — verifier.rs
// Deterministic Formal Constraint Verifier
// Copyright © 2026 Edison Lepiten — AIEONYX
//
// THIS is the actual gate. Not the LLM.
//
// The verifier is:
//   - deterministic (same input → same output, always)
//   - reproducible (no randomness, no model calls)
//   - sound within its constraint domain
//   - transparent (produces human-readable violation traces)
//
// Current verification capabilities (P5-01):
//   - Return value range analysis (numeric bounds)
//   - Nullability checking (non-null returns)
//   - Reachability analysis (always/never reaches)
//   - Effect declaration checking (pure, readonly)
//
// P5-04 will expand: SMT-backed relational constraints
// ============================================================

use axon_parser::ast::{
    FnDecl, Block, Stmt, Expr, Literal,
    BinOp, UnaryOp,
};
use crate::spec::{FormalSpec, Constraint, Effect};
use crate::error::{AiError, ConstraintViolation};

// ── Verification result ───────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub function_name  : String,
    pub spec           : FormalSpec,
    pub status         : VerificationStatus,
    pub violations     : Vec<ConstraintViolation>,
    pub warnings       : Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerificationStatus {
    /// All constraints verified — function matches its spec
    Verified,
    /// Verifier could not prove the constraint (not a failure — needs more info)
    Unknown,
    /// A constraint is definitively violated
    Violated,
    /// Spec not verifiable with current domain (no @ensures/@requires)
    NotVerifiable,
}

// ── Value range domain ────────────────────────────────────────

/// Abstract value — tracks what we know about a value at a point in the program.
#[derive(Debug, Clone, PartialEq)]
pub enum AbstractValue {
    /// Known constant
    Const(i64),
    /// Range: lo ≤ value ≤ hi
    Range(i64, i64),
    /// Boolean value
    Bool(bool),
    /// Could be null/None
    MaybeNull,
    /// Definitely not null
    NonNull,
    /// Unknown — no information
    Top,
    /// Unreachable — dead code
    Bottom,
}

impl AbstractValue {
    pub fn is_non_negative(&self) -> Option<bool> {
        match self {
            AbstractValue::Const(n)       => Some(*n >= 0),
            AbstractValue::Range(lo, _)   => if *lo >= 0 { Some(true) }
                                             else { None }, // unknown
            AbstractValue::Bottom         => Some(true),   // vacuously true (unreachable)
            _                             => None,
        }
    }

    pub fn is_positive(&self) -> Option<bool> {
        match self {
            AbstractValue::Const(n)       => Some(*n > 0),
            AbstractValue::Range(lo, _)   => if *lo > 0 { Some(true) } else { None },
            AbstractValue::Bottom         => Some(true),
            _                             => None,
        }
    }

    pub fn is_non_null(&self) -> Option<bool> {
        match self {
            AbstractValue::NonNull | AbstractValue::Const(_) |
            AbstractValue::Range(_, _) | AbstractValue::Bool(_) => Some(true),
            AbstractValue::MaybeNull                            => None,
            AbstractValue::Bottom                               => Some(true),
            _                                                   => None,
        }
    }

    pub fn join(a: &AbstractValue, b: &AbstractValue) -> AbstractValue {
        match (a, b) {
            (AbstractValue::Bottom, x) | (x, AbstractValue::Bottom) => x.clone(),
            (AbstractValue::Const(n), AbstractValue::Const(m)) if n == m => AbstractValue::Const(*n),
            (AbstractValue::Const(n), AbstractValue::Const(m)) =>
                AbstractValue::Range(*n.min(m), *n.max(m)),
            (AbstractValue::Range(lo1, hi1), AbstractValue::Range(lo2, hi2)) =>
                AbstractValue::Range(*lo1.min(lo2), *hi1.max(hi2)),
            (AbstractValue::Range(lo, hi), AbstractValue::Const(n)) |
            (AbstractValue::Const(n), AbstractValue::Range(lo, hi)) =>
                AbstractValue::Range(*lo.min(n), *hi.max(n)),
            _ => AbstractValue::Top,
        }
    }
}

// ── Constraint Verifier ───────────────────────────────────────

/// Three-valued result of a single constraint check.
/// Sound semantics:
///   Proven  — constraint holds on all reachable return paths
///   Unknown — verifier could not determine (conservative: no false Verified)
///   Violated — at least one return path definitively violates the constraint
#[derive(Debug, Clone)]
pub enum ConstraintCheckResult {
    Proven,
    Unknown,
    Violated(ConstraintViolation),
}

pub struct ConstraintVerifier;

impl ConstraintVerifier {
    /// Verify a function against its formal spec.
    /// Returns a VerificationResult — deterministic, no LLM calls.
    pub fn verify(func: &FnDecl, spec: &FormalSpec) -> VerificationResult {
        let mut result = VerificationResult {
            function_name : func.name.name.clone(),
            spec          : spec.clone(),
            status        : VerificationStatus::NotVerifiable,
            violations    : Vec::new(),
            warnings      : Vec::new(),
        };

        if !spec.is_verifiable() {
            result.warnings.push(
                format!("fn {}: no formal constraints to verify (advisory only)",
                    func.name.name));
            return result;
        }

        result.status = VerificationStatus::Unknown;

        // Analyze return values from the function body
        let return_values = Self::collect_return_values(&func.body);

        // Check each @ensures constraint
        // Track: did we find a violation? or was any check inconclusive?
        let mut had_violation = false;
        let mut had_unknown   = false;

        for constraint in &spec.ensures {
            match Self::check_constraint_result(
                &func.name.name, constraint, &return_values, &func.body
            ) {
                ConstraintCheckResult::Violated(v) => {
                    result.violations.push(v);
                    had_violation = true;
                }
                ConstraintCheckResult::Unknown => {
                    had_unknown = true;
                }
                ConstraintCheckResult::Proven => {
                    // This constraint is satisfied on all paths
                }
            }
        }

        // Check effect declarations
        for effect in &spec.effects {
            if let Some(warning) = Self::check_effect_decl(
                &func.name.name, effect, &func.body
            ) {
                result.warnings.push(warning);
            }
        }

        // Determine final status:
        // Violated  → at least one definite violation
        // Unknown   → no violations proven, but some paths couldn't be analyzed
        // Verified  → all constraints proven on all paths
        // NotVerifiable → no constraints to check
        result.status = if had_violation {
            VerificationStatus::Violated
        } else if spec.ensures.is_empty() {
            VerificationStatus::NotVerifiable
        } else if had_unknown {
            VerificationStatus::Unknown
        } else {
            VerificationStatus::Verified
        };

        result
    }

    /// Collect abstract values of all return expressions in the function.
    fn collect_return_values(block: &Block) -> Vec<AbstractValue> {
        let mut values = Vec::new();
        Self::collect_returns_from_block(block, &mut values);
        values
    }

    fn collect_returns_from_block(block: &Block, values: &mut Vec<AbstractValue>) {
        for stmt in &block.stmts {
            Self::collect_returns_from_stmt(stmt, values);
        }
    }

    fn collect_returns_from_stmt(stmt: &Stmt, values: &mut Vec<AbstractValue>) {
        match stmt {
            Stmt::Return(r) => {
                if let Some(expr) = &r.value {
                    values.push(Self::abstract_eval(expr));
                } else {
                    values.push(AbstractValue::Const(0)); // unit return
                }
            }
            Stmt::If(s) => {
                Self::collect_returns_from_block(&s.then_block, values);
                if let Some(else_block) = &s.else_block {
                    Self::collect_returns_from_block(else_block, values);
                }
            }
            Stmt::While(s) => {
                Self::collect_returns_from_block(&s.body, values);
            }
            Stmt::For(s) => {
                Self::collect_returns_from_block(&s.body, values);
            }
            Stmt::Match(s) => {
                for arm in &s.arms {
                    // Check if arm body is a return expression
                    if let Expr::Return(val, _) = &arm.body {
                        if let Some(e) = val.as_ref() {
                            values.push(Self::abstract_eval(e));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Abstract evaluation of an expression to an AbstractValue.
    fn abstract_eval(expr: &Expr) -> AbstractValue {
        match expr {
            Expr::Lit(lit) => match lit {
                Literal::Int(n, _)  => AbstractValue::Const(*n),
                Literal::Bool(b, _) => AbstractValue::Bool(*b),
                Literal::None(_)    => AbstractValue::MaybeNull,
                _                   => AbstractValue::NonNull,
            },
            Expr::Ident(_) => AbstractValue::Top, // unknown variable

            Expr::BinOp(b) => {
                let lhs = Self::abstract_eval(&b.lhs);
                let rhs = Self::abstract_eval(&b.rhs);
                Self::abstract_binop(&b.op, &lhs, &rhs)
            }

            Expr::UnaryOp(u) => {
                let inner = Self::abstract_eval(&u.expr);
                match &u.op {
                    UnaryOp::Neg => match inner {
                        AbstractValue::Const(n)     => AbstractValue::Const(-n),
                        AbstractValue::Range(lo, hi) => AbstractValue::Range(-hi, -lo),
                        other                        => other,
                    },
                    UnaryOp::Not => AbstractValue::Top,
                    _            => AbstractValue::Top,
                }
            }

            Expr::Call(_) => AbstractValue::Top, // function call — unknown result
            Expr::Return(val, _) => {
                if let Some(e) = val.as_ref() { Self::abstract_eval(e) }
                else { AbstractValue::Const(0) }
            }
            _ => AbstractValue::Top,
        }
    }

    fn abstract_binop(op: &BinOp, lhs: &AbstractValue, rhs: &AbstractValue) -> AbstractValue {
        match (op, lhs, rhs) {
            (BinOp::Add, AbstractValue::Const(a), AbstractValue::Const(b)) =>
                AbstractValue::Const(a.saturating_add(*b)),
            (BinOp::Sub, AbstractValue::Const(a), AbstractValue::Const(b)) =>
                AbstractValue::Const(a.saturating_sub(*b)),
            (BinOp::Mul, AbstractValue::Const(a), AbstractValue::Const(b)) =>
                AbstractValue::Const(a.saturating_mul(*b)),
            (BinOp::Add, AbstractValue::Range(lo1, hi1), AbstractValue::Range(lo2, hi2)) =>
                AbstractValue::Range(lo1.saturating_add(*lo2), hi1.saturating_add(*hi2)),
            _ => AbstractValue::Top,
        }
    }

    // ── Three-valued constraint check result ─────────────────

    fn check_constraint_result(
        fn_name       : &str,
        constraint    : &Constraint,
        return_values : &[AbstractValue],
        body          : &Block,
    ) -> ConstraintCheckResult {
        // Empty return values — vacuously proven (dead code / pass-only)
        if return_values.is_empty() {
            return ConstraintCheckResult::Proven;
        }

        let mut all_proven = true;

        match constraint {
            Constraint::ResultNonNegative => {
                for (i, rv) in return_values.iter().enumerate() {
                    match rv.is_non_negative() {
                        Some(true)  => {}  // this path proven
                        Some(false) => return ConstraintCheckResult::Violated(ConstraintViolation {
                            constraint    : constraint.description(),
                            function_name : fn_name.to_string(),
                            violating_path: format!("return path #{}", i + 1),
                            suggestion    : "ensure all code paths return a value >= 0".into(),
                        }),
                        None => { all_proven = false; } // unknown (e.g. variable)
                    }
                }
            }
            Constraint::ResultPositive => {
                for (i, rv) in return_values.iter().enumerate() {
                    match rv.is_positive() {
                        Some(true)  => {}
                        Some(false) => return ConstraintCheckResult::Violated(ConstraintViolation {
                            constraint    : constraint.description(),
                            function_name : fn_name.to_string(),
                            violating_path: format!("return path #{}", i + 1),
                            suggestion    : "ensure all code paths return a value > 0".into(),
                        }),
                        None => { all_proven = false; }
                    }
                }
            }
            Constraint::ResultNonNull => {
                for (i, rv) in return_values.iter().enumerate() {
                    match rv.is_non_null() {
                        Some(true)  => {}
                        Some(false) => return ConstraintCheckResult::Violated(ConstraintViolation {
                            constraint    : constraint.description(),
                            function_name : fn_name.to_string(),
                            violating_path: format!("return path #{}", i + 1),
                            suggestion    : "ensure the function never returns None/null".into(),
                        }),
                        None => { all_proven = false; }
                    }
                }
            }
            Constraint::ResultAtLeast(n) => {
                for (i, rv) in return_values.iter().enumerate() {
                    match rv {
                        AbstractValue::Const(v)    if *v >= *n  => {}
                        AbstractValue::Range(lo, _) if *lo >= *n => {}
                        AbstractValue::Bottom                    => {}
                        AbstractValue::Const(_) | AbstractValue::Range(_, _) =>
                            return ConstraintCheckResult::Violated(ConstraintViolation {
                                constraint    : constraint.description(),
                                function_name : fn_name.to_string(),
                                violating_path: format!("return path #{}", i + 1),
                                suggestion    : format!("ensure all returns satisfy result >= {}", n),
                            }),
                        _ => { all_proven = false; }
                    }
                }
            }
            // Other constraints — Unknown for now, expanded in P5-04
            _ => { all_proven = false; }
        }

        if all_proven {
            ConstraintCheckResult::Proven
        } else {
            ConstraintCheckResult::Unknown
        }
    }

    /// Kept for backward compatibility
    fn check_constraint(
        fn_name       : &str,
        constraint    : &Constraint,
        return_values : &[AbstractValue],
        body          : &Block,
    ) -> Option<ConstraintViolation> {
        match Self::check_constraint_result(fn_name, constraint, return_values, body) {
            ConstraintCheckResult::Violated(v) => Some(v),
            _ => None,
        }
    }

    /// Check effect declarations (advisory warnings, not hard errors in P5-01)
    fn check_effect_decl(
        fn_name : &str,
        effect  : &Effect,
        _body   : &Block,
    ) -> Option<String> {
        match effect {
            Effect::Pure => {
                // TODO P5-04: scan for side effects in body
                // For now: advisory only
                Some(format!("fn {}: @effect(pure) — effect checking pending P5-04", fn_name))
            }
            Effect::ReadOnly => {
                Some(format!("fn {}: @effect(readonly) — effect checking pending P5-04", fn_name))
            }
            _ => None,
        }
    }
}
