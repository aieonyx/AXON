// ============================================================
// axon_ai — lib.rs
// AXON AI Assistance Engine — Phase 5
// Copyright © 2026 Edison Lepiten — AIEONYX
//
// ARCHITECTURE PRINCIPLE (established by peer review, 2026):
//
//   "AI as assistant. Formal methods as gate."
//
//   The LLM proposes formal specifications.
//   The developer reviews and approves them.
//   The deterministic verifier enforces them.
//   The LLM is never in the Trusted Computing Base.
//
// Pipeline:
//   @ai.intent("always returns non-negative")    ← NL description
//       ↓
//   IntentTranslator::translate()                ← LLM proposes (advisory)
//       ↓
//   FormalSpec { ensures: [ResultNonNegative] }  ← proposed spec
//       ↓
//   Developer reviews / adds @ensures annotation ← human approval
//       ↓
//   ConstraintVerifier::verify(fn, spec)         ← deterministic gate
//       ↓
//   VerificationResult { Verified | Violated }   ← compile-time result
// ============================================================

pub mod spec;
pub mod translator;
pub mod verifier;
pub mod error;

pub use spec::{FormalSpec, Constraint, Effect, ModuleIntent};
pub use translator::IntentTranslator;
pub use verifier::{ConstraintVerifier, VerificationResult, VerificationStatus, AbstractValue};
pub use error::{AiError, ConstraintViolation};

use axon_lexer::FileId;
use axon_parser::ast::FnDecl;

// ── High-level API ────────────────────────────────────────────

/// Full pipeline: NL intent → formal spec proposal (advisory)
/// This does NOT gate compilation. It returns a proposal for review.
pub fn propose_spec(intent_nl: &str) -> Result<FormalSpec, AiError> {
    let mut translator = IntentTranslator::new();
    translator.translate(intent_nl)
}

/// Verify a function against a formal spec.
/// This IS the gate. Deterministic. No LLM calls.
pub fn verify_fn(func: &FnDecl, spec: &FormalSpec) -> VerificationResult {
    ConstraintVerifier::verify(func, spec)
}

/// Full pipeline: parse AXON source and verify all @ai.intent functions.
pub fn verify_source(source: &str) -> Vec<VerificationResult> {
    let raw    = axon_lexer::lex(source, FileId(1));
    let tokens = axon_lexer::inject_indentation(raw);
    let mut p  = axon_parser::parser::Parser::new(tokens, source, FileId(1));
    let program = p.parse_program();

    let mut results = Vec::new();
    for item in &program.items {
        if let axon_parser::ast::TopLevelItem::Fn(func) = item {
            let intent_nl = func.decorators.iter()
                .find(|d| d.name.iter().any(|n| n.name.as_str() == "ai"))
                .and_then(|d| d.args.first())
                .and_then(|a| {
                    if let axon_parser::ast::Expr::Lit(
                        axon_parser::ast::Literal::Str(s, _)) = &a.value {
                        Some(s.clone())
                    } else { None }
                });

            if let Some(nl) = intent_nl {
                let mut translator = IntentTranslator::new();
                if let Ok(spec) = translator.translate(&nl) {
                    results.push(ConstraintVerifier::verify(func, &spec));
                }
            }
        }
    }
    results
}

// ── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{FormalSpec, Constraint};
    use axon_lexer::FileId;

    /// Parse AXON source correctly using the real lexer pipeline.
    /// Uses: lex → inject_indentation → Parser → parse_program
    fn parse_axon(source: &str) -> axon_parser::ParseResult {
        let raw    = axon_lexer::lex(source, FileId(1));
        let tokens = axon_lexer::inject_indentation(raw);
        let mut p  = axon_parser::parser::Parser::new(tokens, source, FileId(1));
        let program = p.parse_program();
        axon_parser::ParseResult {
            program,
            errors: p.into_errors(),
        }
    }

    /// Find first Fn declaration from a parsed program
    fn first_fn(result: &axon_parser::ParseResult) -> Option<&axon_parser::ast::FnDecl> {
        result.program.items.iter()
            .find_map(|i| if let axon_parser::ast::TopLevelItem::Fn(f) = i {
                Some(f)
            } else { None })
    }

    // ── FormalSpec DSL tests ──────────────────────────────────

    #[test]
    fn test_formal_spec_construction() {
        let spec = FormalSpec::new("always returns non-negative")
            .with_ensures(Constraint::ResultNonNegative)
            .with_effect(Effect::Pure)
            .approved();

        assert_eq!(spec.ensures.len(), 1);
        assert_eq!(spec.effects.len(), 1);
        assert!(spec.approved);
        assert!(spec.is_verifiable());
    }

    #[test]
    fn test_constraint_descriptions() {
        assert_eq!(Constraint::ResultNonNegative.description(), "result >= 0");
        assert_eq!(Constraint::ResultPositive.description(),    "result > 0");
        assert_eq!(Constraint::ResultNonNull.description(),     "result != null");
        assert_eq!(Constraint::ResultAtLeast(5).description(),  "result >= 5");
        assert_eq!(Constraint::NoHeapAllocation.description(),  "no heap allocation");
    }

    #[test]
    fn test_abstract_value_join() {
        let a = AbstractValue::Const(5);
        let b = AbstractValue::Const(10);
        assert_eq!(AbstractValue::join(&a, &b), AbstractValue::Range(5, 10));

        let r1 = AbstractValue::Range(0, 5);
        let r2 = AbstractValue::Range(3, 8);
        assert_eq!(AbstractValue::join(&r1, &r2), AbstractValue::Range(0, 8));
    }

    // ── Rule-based fallback tests ─────────────────────────────

    #[test]
    fn test_rule_based_fallback_non_negative() {
        let translator = IntentTranslator::new();
        let spec = translator.rule_based_fallback("always returns non-negative");
        assert!(spec.ensures.contains(&Constraint::ResultNonNegative));
    }

    #[test]
    fn test_rule_based_fallback_pure() {
        let translator = IntentTranslator::new();
        let spec = translator.rule_based_fallback("pure function, no side effects");
        assert!(spec.effects.contains(&Effect::Pure));
    }

    #[test]
    fn test_rule_based_fallback_readonly() {
        let translator = IntentTranslator::new();
        let spec = translator.rule_based_fallback("only reads data, never writes system state");
        assert!(spec.effects.contains(&Effect::ReadOnly));
    }

    #[test]
    fn test_rule_based_fallback_no_allocation() {
        let translator = IntentTranslator::new();
        let spec = translator.rule_based_fallback("does not allocate heap memory");
        assert!(spec.ensures.contains(&Constraint::NoHeapAllocation));
        assert!(spec.effects.contains(&Effect::NoAllocate));
    }

    #[test]
    fn test_intent_translator_falls_back_gracefully() {
        // Without Ollama running, translator falls back to rule-based
        // This is NOT an error — compilation continues
        let mut translator = IntentTranslator::new();
        let result = translator.translate("always returns non-negative");
        assert!(result.is_ok());
        let spec = result.unwrap();
        assert!(spec.ensures.contains(&Constraint::ResultNonNegative));
    }

    // ── Verifier tests (require real lexer on Edison's machine) ──

    #[test]
    fn test_verifier_detects_violation_abs() {
        // abs(x) = x — returns unknown (Top) value for unknown x
        // Verifier cannot prove this is always non-negative
        let src = concat!(
            "fn abs(x : Int) -> Int:\n",
            "    return x\n",
        );
        let result = parse_axon(src);
        assert!(result.errors.is_empty(), "parse errors: {:?}", result.errors);
        let func = first_fn(&result).expect("fn abs not found");

        let spec = FormalSpec::new("always returns non-negative")
            .with_ensures(Constraint::ResultNonNegative);
        let vresult = ConstraintVerifier::verify(func, &spec);

        // x is Top (unknown) — verifier cannot certify this as Verified
        assert_ne!(vresult.status, VerificationStatus::Verified,
            "Must not certify abs(x)=x as provably non-negative (x could be negative)");
    }

    #[test]
    fn test_verifier_accepts_correct_abs() {
        // Correct abs: returns 0 for negatives, x for non-negatives
        let src = concat!(
            "fn abs(x : Int) -> Int:\n",
            "    if x < 0:\n",
            "        return 0\n",
            "    return x\n",
        );
        let result = parse_axon(src);
        assert!(result.errors.is_empty(), "parse errors: {:?}", result.errors);
        let func = first_fn(&result).expect("fn abs not found");

        let spec = FormalSpec::new("always returns non-negative")
            .with_ensures(Constraint::ResultNonNegative);
        let vresult = ConstraintVerifier::verify(func, &spec);

        // return 0 = Const(0) ≥ 0 ✓  |  return x = Top (unknown but no definite violation)
        assert_ne!(vresult.status, VerificationStatus::Violated,
            "Verifier must not report violation for correct abs");
    }

    #[test]
    fn test_verifier_catches_definite_violation() {
        // 0 - 1 = Const(-1) which is provably < 0 → definite violation
        let src = concat!(
            "fn bad(x : Int) -> Int:\n",
            "    return 0 - 1\n",
        );
        let result = parse_axon(src);
        assert!(result.errors.is_empty(), "parse errors: {:?}", result.errors);
        let func = first_fn(&result).expect("fn bad not found");

        let spec = FormalSpec::new("always returns non-negative")
            .with_ensures(Constraint::ResultNonNegative);
        let vresult = ConstraintVerifier::verify(func, &spec);

        assert_eq!(vresult.status, VerificationStatus::Violated,
            "Verifier must catch 0-1 = Const(-1) as non-negative violation");
        assert!(!vresult.violations.is_empty());
    }

    #[test]
    fn test_verifier_accepts_literal_zero() {
        // return 0 = Const(0), 0 ≥ 0 → not violated
        let src = concat!(
            "fn zero(x : Int) -> Int:\n",
            "    return 0\n",
        );
        let result = parse_axon(src);
        assert!(result.errors.is_empty(), "parse errors: {:?}", result.errors);
        let func = first_fn(&result).expect("fn zero not found");

        let spec = FormalSpec::new("always non-negative")
            .with_ensures(Constraint::ResultNonNegative);
        let vresult = ConstraintVerifier::verify(func, &spec);

        assert_ne!(vresult.status, VerificationStatus::Violated,
            "return 0 must not violate non-negative constraint");
    }
}
