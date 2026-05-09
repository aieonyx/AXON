// ============================================================
// axon_ai — constraint_parser.rs
// Decorator → FormalSpec extractor (P5-02)
// Copyright © 2026 Edison Lepiten — AIEONYX
//
// Reads AXON AST decorator nodes and produces FormalSpec.
//
// Supported decorator syntax:
//   @ai.intent("NL description")    → intent_nl (for AI translation)
//   @ensures(result >= 0)           → Constraint::ResultNonNegative
//   @ensures(result > 0)            → Constraint::ResultPositive
//   @ensures(result >= N)           → Constraint::ResultAtLeast(N)
//   @ensures(result <= N)           → Constraint::ResultAtMost(N)
//   @ensures(result == N)           → Constraint::ResultEquals(N)
//   @ensures(result != null)        → Constraint::ResultNonNull
//   @ensures(no_allocation)         → Constraint::NoHeapAllocation
//   @ensures(no_io)                 → Constraint::NoIO
//   @requires(param >= 0)           → Constraint precondition
//   @effect(pure)                   → Effect::Pure
//   @effect(readonly)               → Effect::ReadOnly
//   @effect(no_allocate)            → Effect::NoAllocate
//   @effect(writes_audit_log)       → Effect::WritesAuditLog
// ============================================================

use axon_parser::ast::{
    Decorator, DecoratorArg,
    Expr, Literal, BinOp, UnaryOp,
};
use crate::spec::{FormalSpec, Constraint, Effect};

// ── Public API ────────────────────────────────────────────────

/// Extract a FormalSpec from a function's decorator list.
/// This is the bridge between AXON syntax and the formal verifier.
pub fn extract_spec(decorators: &[Decorator]) -> FormalSpec {
    let mut spec = FormalSpec::new("");

    for dec in decorators {
        let name = decorator_name(dec);
        match name.as_str() {
            // @ai.intent("...") — natural language description
            "ai.intent" | "ai" => {
                if let Some(nl) = first_string_arg(dec) {
                    spec.intent_nl = nl;
                }
            }

            // @ensures(constraint_expr) — postcondition
            "ensures" => {
                for arg in &dec.args {
                    if let Some(c) = expr_to_constraint(&arg.value) {
                        spec.ensures.push(c);
                    }
                }
            }

            // @requires(constraint_expr) — precondition
            "requires" => {
                for arg in &dec.args {
                    if let Some(c) = expr_to_constraint(&arg.value) {
                        spec.requires.push(c);
                    }
                }
            }

            // @effect(effect_name) — side effect declaration
            "effect" => {
                for arg in &dec.args {
                    if let Some(e) = expr_to_effect(&arg.value) {
                        spec.effects.push(e);
                    }
                }
            }

            // @verifier(strict) — enable strict mode (P5-08)
            "verifier" => {}

            _ => {} // unknown decorator — ignore
        }
    }

    // A spec that came from decorators is considered developer-approved
    if spec.is_verifiable() {
        spec.approved = true;
        spec.ai_confidence = 1.0; // formal annotation, not AI-generated
    }

    spec
}

/// Check if a function has any verification-relevant decorators
pub fn has_formal_spec(decorators: &[Decorator]) -> bool {
    decorators.iter().any(|d| {
        let name = decorator_name(d);
        matches!(name.as_str(), "ensures" | "requires" | "effect" | "ai.intent" | "ai")
    })
}

// ── Constraint expression parser ──────────────────────────────

/// Parse a decorator argument expression into a Constraint.
///
/// Handles:
///   result >= 0     → ResultNonNegative
///   result > 0      → ResultPositive
///   result >= N     → ResultAtLeast(N)
///   result <= N     → ResultAtMost(N)
///   result == N     → ResultEquals(N)
///   result != null  → ResultNonNull
///   no_allocation   → NoHeapAllocation
///   no_io           → NoIO
///   pure_inputs     → PureInputs
pub fn expr_to_constraint(expr: &Expr) -> Option<Constraint> {
    match expr {
        // Binary comparison: result OP literal
        Expr::BinOp(b) => {
            // BinOpExpr.lhs and .rhs are Expr (not Box)
            match (&b.op, lhs_name(&b.lhs).as_deref(), &b.rhs) {

                // result >= 0  →  ResultNonNegative
                (BinOp::GtEq, Some("result"), Expr::Lit(Literal::Int(0, _))) =>
                    Some(Constraint::ResultNonNegative),

                // result > 0  →  ResultPositive
                (BinOp::Gt, Some("result"), Expr::Lit(Literal::Int(0, _))) =>
                    Some(Constraint::ResultPositive),

                // result >= N  →  ResultAtLeast(N)
                (BinOp::GtEq, Some("result"), Expr::Lit(Literal::Int(n, _))) =>
                    Some(Constraint::ResultAtLeast(*n)),

                // result <= N  →  ResultAtMost(N)
                (BinOp::LtEq, Some("result"), Expr::Lit(Literal::Int(n, _))) =>
                    Some(Constraint::ResultAtMost(*n)),

                // result == N  →  ResultEquals(N)
                (BinOp::Eq, Some("result"), Expr::Lit(Literal::Int(n, _))) =>
                    Some(Constraint::ResultEquals(*n)),

                // result != null  →  ResultNonNull
                (BinOp::NotEq, Some("result"), Expr::Lit(Literal::None(_))) |
                (BinOp::NotEq, Some("result"), Expr::Ident(_)) =>
                    Some(Constraint::ResultNonNull),

                // 0 <= result  →  ResultNonNegative (reversed)
                (BinOp::LtEq, _, _) => {
                    if let (Some("result"), Expr::Lit(Literal::Int(0, _))) =
                        (lhs_name(&b.rhs).as_deref(), &b.lhs) {
                        Some(Constraint::ResultNonNegative)
                    } else { None }
                }

                _ => None,
            }
        }

        // Identifier shortcuts: no_allocation, no_io, pure_inputs
        Expr::Ident(id) => match id.name.as_str() {
            "no_allocation" | "no_heap" | "no_allocate" =>
                Some(Constraint::NoHeapAllocation),
            "no_io"                                     =>
                Some(Constraint::NoIO),
            "pure_inputs"                               =>
                Some(Constraint::PureInputs),
            _ => Some(Constraint::Custom(id.name.clone())),
        },

        // String literal: parse as constraint DSL
        // This is the primary form: @ensures("result >= 0")
        Expr::Lit(Literal::Str(s, _)) =>
            parse_constraint_string(s),

        _ => None,
    }
}

// ── Effect expression parser ──────────────────────────────────

/// Parse a decorator argument expression into an Effect.
pub fn expr_to_effect(expr: &Expr) -> Option<Effect> {
    match expr {
        Expr::Ident(id) => match id.name.as_str() {
            "pure"             => Some(Effect::Pure),
            "readonly"         => Some(Effect::ReadOnly),
            "writes_audit_log" => Some(Effect::WritesAuditLog),
            "may_allocate"     => Some(Effect::MayAllocate),
            "no_allocate"      => Some(Effect::NoAllocate),
            s                  => Some(Effect::Custom(s.to_string())),
        },
        Expr::Lit(Literal::Str(s, _)) => {
            match s.as_str() {
                "pure"             => Some(Effect::Pure),
                "readonly"         => Some(Effect::ReadOnly),
                "writes_audit_log" => Some(Effect::WritesAuditLog),
                s                  => Some(Effect::Custom(s.to_string())),
            }
        }
        _ => None,
    }
}

// ── String constraint parser ─────────────────────────────────

/// Parse a constraint string like "result >= 0" into a Constraint.
/// This is the primary form used in @ensures("...") decorators.
///
/// Supported patterns:
///   "result >= 0"    → ResultNonNegative
///   "result > 0"     → ResultPositive
///   "result >= N"    → ResultAtLeast(N)
///   "result <= N"    → ResultAtMost(N)
///   "result == N"    → ResultEquals(N)
///   "result != null" → ResultNonNull
///   "no_allocation"  → NoHeapAllocation
///   "no_io"          → NoIO
///   "pure_inputs"    → PureInputs
pub fn parse_constraint_string(s: &str) -> Option<Constraint> {
    let s = s.trim();
    match s {
        "result >= 0" | "result >= 0i64" | "result_non_negative" | "non_negative" =>
            Some(Constraint::ResultNonNegative),
        "result > 0"  | "result_positive" | "positive" =>
            Some(Constraint::ResultPositive),
        "result != null" | "result_non_null" | "non_null" | "not_null" =>
            Some(Constraint::ResultNonNull),
        "no_allocation" | "no_heap" | "no_allocate" | "no heap allocation" =>
            Some(Constraint::NoHeapAllocation),
        "no_io" | "no io" | "no I/O" =>
            Some(Constraint::NoIO),
        "pure_inputs" | "pure inputs" =>
            Some(Constraint::PureInputs),
        s => {
            // Try "result >= N" pattern
            if let Some(rest) = s.strip_prefix("result >= ") {
                if let Ok(n) = rest.trim().parse::<i64>() {
                    return Some(if n == 0 { Constraint::ResultNonNegative }
                                else { Constraint::ResultAtLeast(n) });
                }
            }
            // Try "result > N"
            if let Some(rest) = s.strip_prefix("result > ") {
                if let Ok(n) = rest.trim().parse::<i64>() {
                    return Some(if n == 0 { Constraint::ResultPositive }
                                else { Constraint::ResultAtLeast(n + 1) });
                }
            }
            // Try "result <= N"
            if let Some(rest) = s.strip_prefix("result <= ") {
                if let Ok(n) = rest.trim().parse::<i64>() {
                    return Some(Constraint::ResultAtMost(n));
                }
            }
            // Try "result == N"
            if let Some(rest) = s.strip_prefix("result == ") {
                if let Ok(n) = rest.trim().parse::<i64>() {
                    return Some(Constraint::ResultEquals(n));
                }
            }
            // Try "result >= N" variant with spaces
            if s.contains("result") && s.contains(">=") {
                let parts: Vec<&str> = s.split(">=").collect();
                if parts.len() == 2 {
                    if let Ok(n) = parts[1].trim().parse::<i64>() {
                        return Some(if n == 0 { Constraint::ResultNonNegative }
                                    else { Constraint::ResultAtLeast(n) });
                    }
                }
            }
            // Fall back to custom
            Some(Constraint::Custom(s.to_string()))
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────

/// Get the full decorator name, handling dotted paths like "ai.intent"
fn decorator_name(dec: &Decorator) -> String {
    dec.name.iter()
        .map(|i| i.name.as_str())
        .collect::<Vec<_>>()
        .join(".")
}

/// Get the first string literal argument from a decorator
fn first_string_arg(dec: &Decorator) -> Option<String> {
    dec.args.first().and_then(|a| {
        if let Expr::Lit(Literal::Str(s, _)) = &a.value {
            Some(s.clone())
        } else { None }
    })
}

/// Get the name of an identifier expression, if it is one
fn lhs_name(expr: &Expr) -> Option<String> {
    if let Expr::Ident(id) = expr { Some(id.name.clone()) } else { None }
}
