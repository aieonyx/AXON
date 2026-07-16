// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_data P67-M1 — Corpus ingestion tests (20 tests)
// IAM FLAGSHIP: these tests validate the mouth of the training pipeline.

use axon_data::types::{CorpusEntry, CorpusError, DataTier, CorpusStats};
use axon_data::ingest::{ingest_jsonl, ingest_text, compute_stats, validate_entry, IngestOptions};

// ── T1: DataTier from_str roundtrip ──────────────────────────────────────────
#[test]
fn t1_datatier_roundtrip() {
    for (s, t) in [("critical", DataTier::Critical), ("personal", DataTier::Personal), ("noise", DataTier::Noise)] {
        assert_eq!(DataTier::from_str(s).unwrap(), t);
        assert_eq!(t.as_str(), s);
    }
    assert!(DataTier::from_str("unknown").is_none());
}

// ── T2: DataTier training weights ────────────────────────────────────────────
#[test]
fn t2_training_weights() {
    assert!(DataTier::Critical.training_weight() > DataTier::Personal.training_weight());
    assert!(DataTier::Personal.training_weight() > DataTier::Noise.training_weight());
    assert_eq!(DataTier::Critical.training_weight(), 3.0);
}

// ── T3: CorpusEntry from_text ─────────────────────────────────────────────────
#[test]
fn t3_corpus_entry_from_text() {
    let e = CorpusEntry::from_text("Wisdom is the beginning.".into(), "kjv", DataTier::Critical);
    assert!(!e.is_pair);
    assert_eq!(e.tier, DataTier::Critical);
    assert_eq!(e.source, "kjv");
    assert!(e.is_constitutional());
    assert!(e.char_count > 0);
}

// ── T4: CorpusEntry from_pair ─────────────────────────────────────────────────
#[test]
fn t4_corpus_entry_from_pair() {
    let e = CorpusEntry::from_pair(
        "What is sovereignty?".into(),
        "Full control of your digital existence.".into(),
        "iam-corpus", DataTier::Noise, vec!["sovereignty".into()],
    );
    assert!(e.is_pair);
    assert!(!e.is_constitutional());
    assert_eq!(e.tags.len(), 1);
}

// ── T5: estimated_tokens rough approximation ─────────────────────────────────
#[test]
fn t5_estimated_tokens() {
    let e = CorpusEntry::from_text("a".repeat(400), "test", DataTier::Noise);
    // 400 chars / 4 = 100 tokens
    assert_eq!(e.estimated_tokens(), 100);
}

// ── T6: ingest_jsonl valid pairs ──────────────────────────────────────────────
#[test]
fn t6_ingest_jsonl_valid() {
    let jsonl = r#"{"q":"What is IAM?","a":"Intelligent Assistant to Man.","tier":"noise","source":"iam-doc","tags":["iam"]}
{"q":"What is sovereignty?","a":"Full control.","tier":"critical","source":"constitution"}"#;
    let opts = IngestOptions::default();
    let (entries, errors) = ingest_jsonl(jsonl, "test", &opts);
    assert_eq!(entries.len(), 2);
    assert!(errors.is_empty());
    assert_eq!(entries[0].tier, DataTier::Noise);
    assert_eq!(entries[1].tier, DataTier::Critical);
}

// ── T7: ingest_jsonl skips blank lines and comments ──────────────────────────
#[test]
fn t7_ingest_jsonl_skip_blank() {
    let jsonl = "// comment\n\n{\"q\":\"hello\",\"a\":\"world\",\"tier\":\"noise\",\"source\":\"s\"}";
    let (entries, errors) = ingest_jsonl(jsonl, "test", &IngestOptions::default());
    assert_eq!(entries.len(), 1);
    assert!(errors.is_empty());
}

// ── T8: ingest_jsonl invalid JSON produces error ──────────────────────────────
#[test]
fn t8_ingest_jsonl_invalid_json() {
    let jsonl = "not json at all";
    let (entries, errors) = ingest_jsonl(jsonl, "test", &IngestOptions::default());
    assert!(entries.is_empty());
    assert!(!errors.is_empty());
    assert!(matches!(errors[0], CorpusError::InvalidJsonl { .. }));
}

// ── T9: ingest_jsonl invalid tier produces error ──────────────────────────────
#[test]
fn t9_ingest_jsonl_invalid_tier() {
    let jsonl = r#"{"q":"test","a":"answer","tier":"superuser","source":"s"}"#;
    let (entries, errors) = ingest_jsonl(jsonl, "test", &IngestOptions::default());
    assert!(entries.is_empty());
    assert!(matches!(errors[0], CorpusError::InvalidTier(_)));
}

// ── T10: ingest_jsonl skips empty response by default ────────────────────────
#[test]
fn t10_ingest_jsonl_skip_empty_response() {
    let jsonl = r#"{"q":"test","a":"","tier":"noise","source":"s"}"#;
    let mut opts = IngestOptions::default();
    opts.skip_empty_response = true;
    let (entries, _) = ingest_jsonl(jsonl, "test", &opts);
    assert!(entries.is_empty());
}

// ── T11: ingest_jsonl keeps empty response when disabled ──────────────────────
#[test]
fn t11_ingest_jsonl_keep_empty_response() {
    let jsonl = r#"{"q":"Sovereign text","a":"","tier":"critical","source":"kjv"}"#;
    let mut opts = IngestOptions::default();
    opts.skip_empty_response = false;
    let (entries, _) = ingest_jsonl(jsonl, "test", &opts);
    assert_eq!(entries.len(), 1);
}

// ── T12: ingest_jsonl alias fields (question/answer) ─────────────────────────
#[test]
fn t12_ingest_jsonl_aliases() {
    let jsonl = r#"{"question":"What?","answer":"This.","tier":"noise","source":"s"}"#;
    let (entries, errors) = ingest_jsonl(jsonl, "test", &IngestOptions::default());
    assert_eq!(entries.len(), 1, "errors: {:?}", errors);
    assert_eq!(entries[0].text, "What?");
}

// ── T13: ingest_text splits into chunks ───────────────────────────────────────
#[test]
fn t13_ingest_text_chunks() {
    // Two paragraphs, each ~100 chars
    let text = format!("{}\n\n{}", "A".repeat(100), "B".repeat(100));
    let mut opts = IngestOptions::default();
    opts.chunk_size = 80; // force split
    let entries = ingest_text(&text, "test", &opts);
    assert!(entries.len() >= 2, "expected at least 2 chunks, got {}", entries.len());
}

// ── T14: ingest_text single small document ────────────────────────────────────
#[test]
fn t14_ingest_text_single() {
    let text = "Wisdom is the beginning of all things.";
    let entries = ingest_text(text, "kjv", &IngestOptions::constitutional());
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].tier, DataTier::Critical);
    assert_eq!(entries[0].source, "kjv");
}

// ── T15: ingest_text filters too-short chunks ────────────────────────────────
#[test]
fn t15_ingest_text_min_chars() {
    let text = "Hi\n\nThis is a longer paragraph with enough content to pass the filter.";
    let mut opts = IngestOptions::default();
    opts.min_chars = 20;
    let entries = ingest_text(text, "test", &opts);
    // "Hi" (2 chars) should be filtered
    for e in &entries {
        assert!(e.char_count >= 20, "entry too short: {}", e.char_count);
    }
}

// ── T16: compute_stats counts correctly ──────────────────────────────────────
#[test]
fn t16_compute_stats() {
    let entries = vec![
        CorpusEntry::from_text("constitutional text".into(), "kjv", DataTier::Critical),
        CorpusEntry::from_pair("q".into(), "a".into(), "doc", DataTier::Noise, vec![]),
        CorpusEntry::from_pair("q2".into(), "a2".into(), "doc2", DataTier::Personal, vec![]),
    ];
    let stats = compute_stats(&entries);
    assert_eq!(stats.total_entries, 3);
    assert_eq!(stats.critical_entries, 1);
    assert_eq!(stats.personal_entries, 1);
    assert_eq!(stats.noise_entries, 1);
    assert_eq!(stats.pair_entries, 2);
    assert_eq!(stats.text_entries, 1);
}

// ── T17: compute_stats unique sources ────────────────────────────────────────
#[test]
fn t17_compute_stats_sources() {
    let entries = vec![
        CorpusEntry::from_text("text".into(), "kjv", DataTier::Critical),
        CorpusEntry::from_text("text".into(), "kjv", DataTier::Critical),
        CorpusEntry::from_text("text".into(), "stoic", DataTier::Noise),
    ];
    let stats = compute_stats(&entries);
    assert_eq!(stats.sources.len(), 2);
}

// ── T18: validate_entry passes valid entry ────────────────────────────────────
#[test]
fn t18_validate_entry_ok() {
    let e = CorpusEntry::from_text("Valid content here.".into(), "test", DataTier::Noise);
    assert!(validate_entry(&e, &IngestOptions::default()).is_ok());
}

// ── T19: validate_entry rejects empty text ────────────────────────────────────
#[test]
fn t19_validate_entry_empty() {
    let e = CorpusEntry::from_text("".into(), "test", DataTier::Noise);
    assert!(matches!(validate_entry(&e, &IngestOptions::default()), Err(CorpusError::EmptyEntry)));
}

// ── T20: full ingestion pipeline — mixed sources ──────────────────────────────
#[test]
fn t20_full_pipeline() {
    // Constitutional source (KJV-style)
    let kjv = "The fear of the LORD is the beginning of wisdom.\n\nA wise man will hear and increase learning.";
    let kjv_entries = ingest_text(kjv, "kjv", &IngestOptions::constitutional());

    // QA pairs (IAM training corpus)
    let jsonl = r#"{"q":"What is IAM?","a":"Intelligent Assistant to Man — sovereign AI.","tier":"noise","source":"iam-faq","tags":["iam","intro"]}
{"q":"What is the S4+i framework?","a":"Security, Sovereignty, Simplicity, Speed + Intelligence.","tier":"critical","source":"aieonyx-spec"}"#;
    let opts = IngestOptions::default();
    let (pair_entries, errors) = ingest_jsonl(jsonl, "iam-corpus", &opts);

    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
    assert!(!kjv_entries.is_empty());
    assert_eq!(pair_entries.len(), 2);

    // Merge and compute stats
    let mut all: Vec<CorpusEntry> = kjv_entries;
    all.extend(pair_entries);

    let stats = compute_stats(&all);
    assert!(stats.total_entries >= 3);
    assert!(stats.critical_entries >= 1); // constitutional + S4+i pair
    assert!(stats.total_chars > 0);
    assert!(stats.estimated_tokens > 0);

    // All constitutional entries carry weight 3x
    for e in all.iter().filter(|e| e.is_constitutional()) {
        assert_eq!(e.tier.training_weight(), 3.0);
    }
}
