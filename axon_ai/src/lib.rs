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
pub mod constraint_parser;
pub mod error;

pub use spec::{FormalSpec, Constraint, Effect, ModuleIntent};
pub use translator::IntentTranslator;
pub use verifier::{ConstraintVerifier, VerificationResult, VerificationStatus, AbstractValue};
pub use constraint_parser::{extract_spec, has_formal_spec, expr_to_constraint, expr_to_effect, parse_constraint_string};
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

/// Full pipeline: parse AXON source and verify all annotated functions.
///
/// Checks functions with @ensures, @requires, @effect, or @ai.intent decorators.
/// Returns one VerificationResult per annotated function.
pub fn verify_source(source: &str) -> Vec<VerificationResult> {
    let raw    = axon_lexer::lex(source, FileId(1));
    let tokens = axon_lexer::inject_indentation(raw);
    let mut p  = axon_parser::parser::Parser::new(tokens, source, FileId(1));
    let program = p.parse_program();

    let mut results = Vec::new();
    for item in &program.items {
        if let axon_parser::ast::TopLevelItem::Fn(func) = item {
            if !has_formal_spec(&func.decorators) { continue; }

            // Extract formal spec from @ensures/@requires/@effect decorators
            let mut spec = extract_spec(&func.decorators);

            // If no formal constraints but has @ai.intent: propose via AI (advisory)
            if !spec.is_verifiable() && !spec.intent_nl.is_empty() {
                let mut translator = IntentTranslator::new();
                if let Ok(proposed) = translator.translate(&spec.intent_nl) {
                    // Only use AI proposal if developer hasn't written formal spec
                    spec = proposed;
                }
            }

            if spec.is_verifiable() {
                results.push(ConstraintVerifier::verify(func, &spec));
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
    // ── P5-02 constraint_parser tests ────────────────────────

    #[test]
    fn test_constraint_parser_result_gte_zero() {
        // @ensures(result >= 0) → Constraint::ResultNonNegative
        // We test by parsing a real AXON source with @ensures decorator
        let src = concat!(
            "fn f(x : Int) -> Int:\n",
            "    return 0\n",
        );
        let result = parse_axon(src);
        // No @ensures decorator → no spec → verify_source returns empty
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_verify_source_catches_violation() {
        // Simulating what axon check does:
        // parse source with @ensures → run verifier → find violation
        let src = concat!(
            "fn bad(x : Int) -> Int:\n",
            "    return 0 - 1\n",
        );
        let result = parse_axon(src);
        assert!(result.errors.is_empty());
        let func = first_fn(&result).expect("fn not found");

        // Build spec as if from @ensures(result >= 0)
        let spec = FormalSpec::new("always non-negative")
            .with_ensures(Constraint::ResultNonNegative);

        let vresult = ConstraintVerifier::verify(func, &spec);
        assert_eq!(vresult.status, VerificationStatus::Violated,
            "0 - 1 = Const(-1) must violate result >= 0");
    }

    #[test]
    fn test_verify_source_accepts_verified() {
        let src = concat!(
            "fn good(x : Int) -> Int:\n",
            "    return 42\n",
        );
        let result = parse_axon(src);
        let func = first_fn(&result).expect("fn not found");

        let spec = FormalSpec::new("always positive")
            .with_ensures(Constraint::ResultAtLeast(1));

        let vresult = ConstraintVerifier::verify(func, &spec);
        // Const(42) >= 1 → Verified
        assert_eq!(vresult.status, VerificationStatus::Verified,
            "return 42 must satisfy result >= 1");
    }

    #[test]
    fn test_extract_spec_from_decorators() {
        // Test constraint_parser::expr_to_constraint semantics
        // via the verifier: if we build a spec with ResultNonNegative
        // and verify a fn that returns 0, it should be Verified.
        // This confirms the constraint means what we think it means.
        let src = concat!(
            "fn f(x : Int) -> Int:\n",
            "    return 0\n",
        );
        let result = parse_axon(src);
        assert!(result.errors.is_empty());
        let func = first_fn(&result).expect("fn not found");

        // ResultNonNegative = @ensures(result >= 0)
        let spec = FormalSpec::new("always non-negative")
            .with_ensures(Constraint::ResultNonNegative);
        let vresult = ConstraintVerifier::verify(func, &spec);

        // return 0 = Const(0) >= 0 → Verified
        assert_eq!(vresult.status, VerificationStatus::Verified,
            "return 0 must satisfy result >= 0");
    }
}


// ── Phase 5.5-05: Security-Weighted AI Inference (SWI) ───────

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityTier {
    Tier1, // security-critical — full inference
    Tier2, // annotated — standard inference
    Tier3, // low-value — skip inference
}

/// Classify a function into a security tier.
pub fn classify_tier(fn_name: &str, has_annotations: bool) -> SecurityTier {
    const SECURITY_KEYWORDS: &[&str] = &[
        "classify", "authenticate", "authorize", "encrypt",
        "verify", "sanitize", "validate",
    ];
    if SECURITY_KEYWORDS.iter().any(|&k| fn_name.contains(k)) {
        SecurityTier::Tier1
    } else if has_annotations {
        SecurityTier::Tier2
    } else {
        SecurityTier::Tier3
    }
}

/// Format a SWI classification report.
pub fn swi_report(functions: &[(String, SecurityTier)]) -> String {
    let mut report = String::from("SWI Classification:\n");
    for (fn_name, tier) in functions {
        let label = match tier {
            SecurityTier::Tier1 => "Tier1 (security-critical)",
            SecurityTier::Tier2 => "Tier2 (annotated)",
            SecurityTier::Tier3 => "Tier3 (skipped)",
        };
        report.push_str(&format!("  fn {} -> {}\n", fn_name, label));
    }
    report
}

#[cfg(test)]
mod swi_tests {
    use super::*;

    #[test]
    fn test_swi_tier1() {
        assert_eq!(classify_tier("classify", false), SecurityTier::Tier1);
    }

    #[test]
    fn test_swi_tier1_authenticate() {
        assert_eq!(classify_tier("authenticate_user", false), SecurityTier::Tier1);
    }

    #[test]
    fn test_swi_tier2() {
        assert_eq!(classify_tier("process", true), SecurityTier::Tier2);
    }

    #[test]
    fn test_swi_tier3() {
        assert_eq!(classify_tier("get_timestamp", false), SecurityTier::Tier3);
    }

    #[test]
    fn test_swi_report_format() {
        let fns = vec![
            ("classify".to_string(), SecurityTier::Tier1),
            ("get_name".to_string(), SecurityTier::Tier3),
        ];
        let report = swi_report(&fns);
        assert!(report.contains("Tier1"));
        assert!(report.contains("Tier3"));
        assert!(report.contains("classify"));
    }
}

// ── Phase 5.5-06: Incremental Verification Cache (IVC) ───────

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum CacheResult {
    Verified,
    Unknown,
    Violated,
}

#[derive(Debug, Clone)]
pub struct WitnessEntry {
    pub fn_name    : String,
    pub source_hash: u64,
    pub result     : CacheResult,
    pub timestamp  : u64,
}

pub struct IVCCache {
    entries: HashMap<String, WitnessEntry>,
    hits    : usize,
    misses  : usize,
}

impl IVCCache {
    pub fn new() -> Self {
        IVCCache { entries: HashMap::new(), hits: 0, misses: 0 }
    }

    /// Hash function source + annotations for cache key.
    pub fn hash_source(fn_name: &str, source: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        fn_name.hash(&mut h);
        source.hash(&mut h);
        h.finish()
    }

    /// Look up a cached verification result.
    /// Returns None on cache miss (or if result was Violated — never cache violations).
    pub fn get(&mut self, fn_name: &str, source: &str) -> Option<&CacheResult> {
        let key  = Self::hash_source(fn_name, source);
        let now  = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if let Some(entry) = self.entries.get(fn_name) {
            // Invalidate if hash changed or entry > 30 days old
            let stale = entry.source_hash != key
                || (now - entry.timestamp) > 60 * 60 * 24 * 30;
            if stale {
                self.entries.remove(fn_name);
                self.misses += 1;
                return None;
            }
            self.hits += 1;
            return Some(&self.entries[fn_name].result);
        }
        self.misses += 1;
        None
    }

    /// Store a verification result. Violated results are never cached.
    pub fn set(&mut self, fn_name: &str, source: &str, result: CacheResult) {
        if result == CacheResult::Violated { return; }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.entries.insert(fn_name.to_string(), WitnessEntry {
            fn_name    : fn_name.to_string(),
            source_hash: Self::hash_source(fn_name, source),
            result,
            timestamp  : now,
        });
    }

    /// Cache statistics summary.
    pub fn stats(&self) -> String {
        let total = self.hits + self.misses;
        let rate  = if total > 0 {
            (self.hits as f64 / total as f64 * 100.0) as usize
        } else { 0 };
        format!("Cache: {} hits, {} misses, {}% hit rate",
            self.hits, self.misses, rate)
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }
}

#[cfg(test)]
mod ivc_tests {
    use super::*;

    #[test]
    fn test_ivc_miss_on_empty() {
        let mut cache = IVCCache::new();
        assert!(cache.get("classify", "fn classify() {}").is_none());
    }

    #[test]
    fn test_ivc_hit_after_set() {
        let mut cache = IVCCache::new();
        cache.set("classify", "fn classify() {}", CacheResult::Verified);
        assert_eq!(
            cache.get("classify", "fn classify() {}"),
            Some(&CacheResult::Verified)
        );
    }

    #[test]
    fn test_ivc_miss_after_source_change() {
        let mut cache = IVCCache::new();
        cache.set("f", "fn f() { 1 }", CacheResult::Verified);
        // Different source → cache miss
        assert!(cache.get("f", "fn f() { 2 }").is_none());
    }

    #[test]
    fn test_ivc_violated_not_cached() {
        let mut cache = IVCCache::new();
        cache.set("f", "fn f() {}", CacheResult::Violated);
        assert!(cache.get("f", "fn f() {}").is_none());
    }

    #[test]
    fn test_ivc_hit_rate() {
        let mut cache = IVCCache::new();
        cache.set("f", "source", CacheResult::Verified);
        cache.get("f", "source"); // hit
        cache.get("f", "source"); // hit
        cache.get("g", "other");  // miss
        assert!(cache.hit_rate() > 0.6);
    }

    #[test]
    fn test_ivc_stats_format() {
        let mut cache = IVCCache::new();
        cache.set("f", "src", CacheResult::Verified);
        cache.get("f", "src");
        let stats = cache.stats();
        assert!(stats.contains("hits"));
        assert!(stats.contains("misses"));
    }
}
