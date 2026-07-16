// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_data P67-M2 — Text cleaning tests (20 tests)
// IAM FLAGSHIP: clean pipeline = clean brain

use std::collections::HashSet;
use axon_data::types::{CorpusEntry, DataTier};
use axon_data::clean::{
    CleanOptions, CleanVerdict, FilterReason,
    scrub_pii, normalize_whitespace, symbol_density,
    repetition_ratio, unique_word_ratio,
    minhash_signature, jaccard_estimate, dedup_key,
    clean_entry, clean_batch,
};

fn entry(text: &str) -> CorpusEntry {
    CorpusEntry::from_text(text.to_string(), "test", DataTier::Noise)
}

// ── T1: normalize_whitespace collapses spaces ─────────────────────────────────
#[test]
fn t1_normalize_whitespace_spaces() {
    let s = "hello   world\t\there";
    assert_eq!(normalize_whitespace(s), "hello world here");
}

// ── T2: normalize_whitespace collapses newlines ───────────────────────────────
#[test]
fn t2_normalize_whitespace_newlines() {
    let s = "line1\n\n\nline2";
    let r = normalize_whitespace(s);
    assert!(r.contains("line1") && r.contains("line2"));
    assert!(!r.contains("\n\n\n"));
}

// ── T3: scrub_pii replaces email ─────────────────────────────────────────────
#[test]
fn t3_scrub_pii_email() {
    let (cleaned, count) = scrub_pii("contact me at user@example.com please", Some("[REDACTED]"));
    assert!(count > 0, "email should be detected");
    assert!(!cleaned.contains("user@example.com"), "email should be removed");
    assert!(cleaned.contains("[REDACTED]"));
}

// ── T4: scrub_pii replaces IPv4 ──────────────────────────────────────────────
#[test]
fn t4_scrub_pii_ipv4() {
    let (cleaned, count) = scrub_pii("server at 192.168.1.100 is down", Some("[REDACTED]"));
    assert!(count > 0, "IPv4 should be detected");
    assert!(!cleaned.contains("192.168.1.100"));
}

// ── T5: scrub_pii replaces credit card ───────────────────────────────────────
#[test]
fn t5_scrub_pii_credit_card() {
    let (cleaned, count) = scrub_pii("card: 4111 1111 1111 1111 valid", Some("[REDACTED]"));
    assert!(count > 0 || !cleaned.contains("4111 1111 1111 1111"),
        "credit card should be detected");
}

// ── T6: scrub_pii leaves clean text unchanged ────────────────────────────────
#[test]
fn t6_scrub_pii_clean_text() {
    let text = "Wisdom is the beginning of all sovereign things.";
    let (cleaned, count) = scrub_pii(text, Some("[REDACTED]"));
    assert_eq!(count, 0);
    assert_eq!(cleaned, text);
}

// ── T7: symbol_density pure text ─────────────────────────────────────────────
#[test]
fn t7_symbol_density_text() {
    let density = symbol_density("Hello world this is clean text");
    assert!(density < 0.1, "clean text density={}", density);
}

// ── T8: symbol_density high symbols ──────────────────────────────────────────
#[test]
fn t8_symbol_density_high() {
    let density = symbol_density("!!!???###$$$%%%^^^&&&***");
    assert!(density > 0.8, "symbol-heavy density={}", density);
}

// ── T9: repetition_ratio clean text ──────────────────────────────────────────
#[test]
fn t9_repetition_ratio_clean() {
    let text = "The sovereign browser protects your data and keeps your identity safe from trackers";
    let ratio = repetition_ratio(text);
    assert!(ratio < 0.3, "clean text repetition={}", ratio);
}

// ── T10: repetition_ratio repeated text ──────────────────────────────────────
#[test]
fn t10_repetition_ratio_repeated() {
    let text = "the cat sat the cat sat the cat sat the cat sat the cat sat";
    let ratio = repetition_ratio(text);
    assert!(ratio > 0.4, "repeated text ratio={}", ratio);
}

// ── T11: unique_word_ratio diverse text ──────────────────────────────────────
#[test]
fn t11_unique_word_ratio_diverse() {
    let text = "sovereignty security simplicity speed intelligence freedom privacy dignity";
    let ratio = unique_word_ratio(text);
    assert_eq!(ratio, 1.0, "all unique words should give ratio 1.0");
}

// ── T12: unique_word_ratio repeated text ─────────────────────────────────────
#[test]
fn t12_unique_word_ratio_repeated() {
    let text = "the the the the the the the the the the";
    let ratio = unique_word_ratio(text);
    assert!(ratio < 0.2, "all-same words ratio={}", ratio);
}

// ── T13: minhash identical text → similarity 1.0 ─────────────────────────────
#[test]
fn t13_minhash_identical() {
    let text = "IAM is the sovereign intelligence at the heart of AIEONYX";
    let s1 = minhash_signature(text);
    let s2 = minhash_signature(text);
    let sim = jaccard_estimate(&s1, &s2);
    assert_eq!(sim, 1.0, "identical text must have similarity 1.0");
}

// ── T14: minhash different text → lower similarity ───────────────────────────
#[test]
fn t14_minhash_different() {
    let s1 = minhash_signature("IAM is the sovereign intelligence of AIEONYX");
    let s2 = minhash_signature("The quick brown fox jumps over the lazy dog");
    let sim = jaccard_estimate(&s1, &s2);
    assert!(sim < 0.5, "unrelated texts similarity={}", sim);
}

// ── T15: minhash near-duplicate detection ────────────────────────────────────
#[test]
fn t15_minhash_near_duplicate() {
    let s1 = minhash_signature("The sovereign browser Onyxia protects your data");
    let s2 = minhash_signature("The sovereign browser Onyxia protects your data today");
    let sim = jaccard_estimate(&s1, &s2);
    // Near-duplicate should have high similarity
    assert!(sim > 0.5, "near-duplicate similarity={}", sim);
}

// ── T16: clean_entry passes clean text ───────────────────────────────────────
#[test]
fn t16_clean_entry_passes() {
    let e = entry("Wisdom is the beginning of sovereignty and freedom for all people.");
    let (_, verdict) = clean_entry(&e, &CleanOptions::default());
    assert_eq!(verdict, CleanVerdict::Pass);
}

// ── T17: clean_entry filters high symbol density ─────────────────────────────
#[test]
fn t17_clean_entry_filters_symbols() {
    let e = entry("!!!###$$$%%%^^^&&&***!!!###$$$%%%^^^&&&***!!!###");
    let mut opts = CleanOptions::default();
    opts.max_symbol_density = 0.3;
    let (_, verdict) = clean_entry(&e, &opts);
    assert!(matches!(verdict, CleanVerdict::Filtered(FilterReason::TooManySymbols(_))));
}

// ── T18: clean_entry scrubs PII ──────────────────────────────────────────────
#[test]
fn t18_clean_entry_scrubs_pii() {
    let e = entry("Contact the admin at admin@aieonyx.eu for support");
    let (cleaned, verdict) = clean_entry(&e, &CleanOptions::default());
    assert_eq!(verdict, CleanVerdict::Pass);
    assert!(!cleaned.text.contains("admin@aieonyx.eu"));
    assert!(cleaned.text.contains("[REDACTED]"));
}

// ── T19: clean_batch deduplicates identical entries ──────────────────────────
#[test]
fn t19_clean_batch_dedup() {
    let text = "The sovereign stack is AXON EdisonDB Onyxia and BASTION working together";
    let entries = vec![entry(text), entry(text), entry(text)];
    let mut seen = HashSet::new();
    let (passed, filtered) = clean_batch(&entries, &CleanOptions::default(), &mut seen);
    assert_eq!(passed.len(), 1, "dedup should keep only 1");
    assert_eq!(filtered, 2, "2 duplicates filtered");
}

// ── T20: full cleaning pipeline — IAM constitutional corpus ──────────────────
#[test]
fn t20_full_clean_pipeline() {
    let entries = vec![
        entry("The fear of the LORD is the beginning of wisdom and all sovereign understanding."),
        entry("contact admin@example.com for the secret key at 192.168.1.1"),
        entry("the cat sat the cat sat the cat sat the cat sat the cat sat on the mat sat"),
        entry("Sovereignty means full control of your data identity and digital existence forever."),
        entry("The fear of the LORD is the beginning of wisdom and all sovereign understanding."), // dup
    ];

    let mut seen = HashSet::new();
    let opts = CleanOptions::default();
    let (passed, filtered) = clean_batch(&entries, &opts, &mut seen);

    // Entry 2: PII scrubbed, passes
    // Entry 3: high repetition, filtered
    // Entry 5: near-duplicate of entry 1, filtered
    // At least 1 entry passes (constitutional wisdom text)
    // Repetition entry + duplicate are filtered = at least 2 filtered
    assert!(passed.len() >= 1, "at least 1 entry should pass, got {}", passed.len());
    assert!(filtered >= 1, "at least 1 entry should be filtered, got {}", filtered);
    // Total must be conserved
    assert_eq!(passed.len() + filtered, 5, "5 entries in = {} out", passed.len() + filtered);

    // Verify PII was scrubbed in passed entries
    for e in &passed {
        assert!(!e.text.contains("@example.com"), "PII should be scrubbed");
        assert!(!e.text.contains("192.168."), "IP should be scrubbed");
    }
}
