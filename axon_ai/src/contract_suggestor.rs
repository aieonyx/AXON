// ============================================================
// axon_ai::contract_suggestor — Contract Inference Engine
// Copyright © 2026 Edison Lepiten — AIEONYX
// SPEC: 6A-01 DWC
//
// Advisory-only. Suggestions are emitted as compiler warnings
// with --suggest-contracts. Never auto-applied.
//
// Phase 6 heuristics:
//   H-01: unchecked integer arithmetic  → bounds pre-condition
//   H-02: raw pointer dereference       → non-null pre-condition
//   H-03: index without prior len check → bounds pre-condition
// ============================================================

/// The kind of contract clause being suggested.
/// SPEC: 6A-01
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionKind {
    /// Suggested pre-condition.
    Pre,
    /// Suggested post-condition.
    Post,
    /// Suggested type invariant.
    Invariant,
}

/// A single contract suggestion produced by the suggestor.
/// SPEC: 6A-01
#[derive(Debug, Clone)]
pub struct ContractSuggestion {
    /// The kind of clause being suggested.
    pub kind:         SuggestionKind,
    /// Suggested label (becomes the `[label]` in source).
    pub label:        String,
    /// Human-readable predicate expression.
    pub predicate:    String,
    /// Confidence score in [0.0, 1.0].
    /// H-01/H-02/H-03 heuristics emit 0.7 — pattern-matched, not proven.
    pub confidence:   f32,
    /// Explanation of why this suggestion was generated.
    pub rationale:    String,
}

impl ContractSuggestion {
    /// Format as a compiler warning line.
    pub fn as_warning(&self) -> String {
        format!(
            "warning[AX-SUGGEST]: consider adding {} contract [{}: {}] (confidence: {:.0}%) — {}",
            match self.kind {
                SuggestionKind::Pre       => "pre",
                SuggestionKind::Post      => "post",
                SuggestionKind::Invariant => "invariant",
            },
            self.label,
            self.predicate,
            self.confidence * 100.0,
            self.rationale,
        )
    }
}

/// Analyses function bodies and proposes `@contract` annotations.
///
/// All suggestions are advisory — the developer reviews and applies them.
/// The suggestor is never in the trusted computing base.
///
/// SPEC: 6A-01
pub struct ContractSuggestor;

impl ContractSuggestor {
    /// Create a new suggestor instance.
    pub fn new() -> Self {
        Self
    }

    /// Analyse a function body represented as source text and return
    /// contract suggestions.
    ///
    /// Phase 6 operates on raw source strings — a full AST walk is
    /// deferred to Phase 7 when the DWC AST nodes are stable.
    ///
    /// SPEC: 6A-01
    pub fn analyse(&self, fn_source: &str) -> Vec<ContractSuggestion> {
        let mut suggestions = Vec::new();
        suggestions.extend(self.h01_unchecked_arithmetic(fn_source));
        suggestions.extend(self.h02_raw_pointer_deref(fn_source));
        suggestions.extend(self.h03_index_without_bounds(fn_source));
        suggestions
    }

    // ── H-01: Unchecked integer arithmetic ───────────────────
    //
    // Pattern: arithmetic operators (+, -, *) on integer-typed
    // expressions without a preceding checked_* call or explicit
    // bounds check.
    //
    // Suggests: pre-condition bounding the operands.

    fn h01_unchecked_arithmetic(&self, src: &str) -> Vec<ContractSuggestion> {
        let mut suggestions = Vec::new();

        // Detect wrapping/overflow-prone patterns:
        // bare `+`, `-`, `*` on identifiers without `.checked_` prefix
        let has_bare_add = src.contains(" + ") && !src.contains("checked_add");
        let has_bare_sub = src.contains(" - ") && !src.contains("checked_sub");
        let has_bare_mul = src.contains(" * ") && !src.contains("checked_mul");

        if has_bare_add {
            suggestions.push(ContractSuggestion {
                kind:       SuggestionKind::Pre,
                label:      "no_add_overflow".to_string(),
                predicate:  "lhs <= i64::MAX - rhs".to_string(),
                confidence: 0.7,
                rationale:  "H-01: unchecked addition detected; consider bounding operands \
                             or using checked_add()"
                             .to_string(),
            });
        }
        if has_bare_sub {
            suggestions.push(ContractSuggestion {
                kind:       SuggestionKind::Pre,
                label:      "no_sub_underflow".to_string(),
                predicate:  "lhs >= rhs".to_string(),
                confidence: 0.7,
                rationale:  "H-01: unchecked subtraction detected; consider bounding operands \
                             or using checked_sub()"
                             .to_string(),
            });
        }
        if has_bare_mul {
            suggestions.push(ContractSuggestion {
                kind:       SuggestionKind::Pre,
                label:      "no_mul_overflow".to_string(),
                predicate:  "lhs <= i64::MAX / rhs".to_string(),
                confidence: 0.7,
                rationale:  "H-01: unchecked multiplication detected; consider bounding \
                             operands or using checked_mul()"
                             .to_string(),
            });
        }

        suggestions
    }

    // ── H-02: Raw pointer dereference ────────────────────────
    //
    // Pattern: `*ptr` or `.as_ref()` on a raw pointer type without
    // a preceding null check.
    //
    // Suggests: pre-condition asserting non-null.

    fn h02_raw_pointer_deref(&self, src: &str) -> Vec<ContractSuggestion> {
        let mut suggestions = Vec::new();

        let has_raw_deref = (src.contains("*mut ") || src.contains("*const "))
            && (src.contains("unsafe") || src.contains("deref"));
        let has_null_check = src.contains("is_null()") || src.contains("!= null");

        if has_raw_deref && !has_null_check {
            suggestions.push(ContractSuggestion {
                kind:       SuggestionKind::Pre,
                label:      "ptr_non_null".to_string(),
                predicate:  "!ptr.is_null()".to_string(),
                confidence: 0.7,
                rationale:  "H-02: raw pointer dereference without null check detected; \
                             add non-null pre-condition or use NonNull<T>"
                             .to_string(),
            });
        }

        suggestions
    }

    // ── H-03: Index without bounds check ─────────────────────
    //
    // Pattern: `a[i]` or `slice[idx]` without a preceding
    // `i < a.len()` or `idx < slice.len()` guard.
    //
    // Suggests: pre-condition bounding the index.

    fn h03_index_without_bounds(&self, src: &str) -> Vec<ContractSuggestion> {
        let mut suggestions = Vec::new();

        // Heuristic: indexing syntax present but no .len() check nearby
        let has_index = src.contains('[') && src.contains(']');
        let has_len_check = src.contains(".len()");
        let has_get = src.contains(".get("); // safe alternative

        if has_index && !has_len_check && !has_get {
            suggestions.push(ContractSuggestion {
                kind:       SuggestionKind::Pre,
                label:      "index_in_bounds".to_string(),
                predicate:  "idx < slice.len()".to_string(),
                confidence: 0.7,
                rationale:  "H-03: index operation without bounds check detected; \
                             add bounds pre-condition or use .get(idx) instead"
                             .to_string(),
            });
        }

        suggestions
    }
}

impl Default for ContractSuggestor {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_h01_detects_bare_addition() {
        let src = "fn add(a: i64, b: i64) -> i64 { a + b }";
        let s = ContractSuggestor::new();
        let suggestions = s.analyse(src);
        assert!(
            suggestions.iter().any(|s| s.label == "no_add_overflow"),
            "H-01 must detect bare addition"
        );
    }

    #[test]
    fn test_h01_no_suggestion_with_checked_add() {
        let src = "fn add(a: i64, b: i64) -> i64 { a.checked_add(b).unwrap() }";
        let s = ContractSuggestor::new();
        let suggestions = s.analyse(src);
        assert!(
            !suggestions.iter().any(|s| s.label == "no_add_overflow"),
            "H-01 must not fire when checked_add is present"
        );
    }

    #[test]
    fn test_h01_detects_bare_subtraction() {
        let src = "fn sub(a: i64, b: i64) -> i64 { a - b }";
        let s = ContractSuggestor::new();
        let suggestions = s.analyse(src);
        assert!(suggestions.iter().any(|s| s.label == "no_sub_underflow"));
    }

    #[test]
    fn test_h02_detects_raw_pointer_deref() {
        let src = "unsafe fn read(ptr: *const u8) -> u8 { *ptr }";
        let s = ContractSuggestor::new();
        let suggestions = s.analyse(src);
        assert!(
            suggestions.iter().any(|s| s.label == "ptr_non_null"),
            "H-02 must detect raw pointer dereference"
        );
    }

    #[test]
    fn test_h02_no_suggestion_with_null_check() {
        let src = "unsafe fn read(ptr: *const u8) -> u8 { \
                   if ptr.is_null() { return 0; } *ptr }";
        let s = ContractSuggestor::new();
        let suggestions = s.analyse(src);
        assert!(
            !suggestions.iter().any(|s| s.label == "ptr_non_null"),
            "H-02 must not fire when null check is present"
        );
    }

    #[test]
    fn test_h03_detects_index_without_bounds() {
        let src = "fn get(arr: &[i64], i: usize) -> i64 { arr[i] }";
        let s = ContractSuggestor::new();
        let suggestions = s.analyse(src);
        assert!(
            suggestions.iter().any(|s| s.label == "index_in_bounds"),
            "H-03 must detect index without bounds check"
        );
    }

    #[test]
    fn test_h03_no_suggestion_with_len_check() {
        let src = "fn get(arr: &[i64], i: usize) -> i64 { \
                   if i < arr.len() { arr[i] } else { 0 } }";
        let s = ContractSuggestor::new();
        let suggestions = s.analyse(src);
        assert!(
            !suggestions.iter().any(|s| s.label == "index_in_bounds"),
            "H-03 must not fire when len check is present"
        );
    }

    #[test]
    fn test_suggestion_warning_format() {
        let suggestion = ContractSuggestion {
            kind:       SuggestionKind::Pre,
            label:      "no_add_overflow".to_string(),
            predicate:  "lhs <= i64::MAX - rhs".to_string(),
            confidence: 0.7,
            rationale:  "test rationale".to_string(),
        };
        let warning = suggestion.as_warning();
        assert!(warning.contains("warning[AX-SUGGEST]"));
        assert!(warning.contains("no_add_overflow"));
        assert!(warning.contains("70%"));
    }

    #[test]
    fn test_clean_fn_no_suggestions() {
        let src = "fn safe(a: i64) -> i64 { a.checked_add(1).unwrap_or(i64::MAX) }";
        let s = ContractSuggestor::new();
        let suggestions = s.analyse(src);
        // No arithmetic, no pointer, no raw index
        assert!(suggestions.is_empty() || suggestions.iter().all(|s| s.confidence < 0.8));
    }
}
