// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_data P67-M3 — BPE Tokenizer tests (20 tests)
// IAM FLAGSHIP: the vocabulary IAM thinks in

use axon_data::tokenizer::{
    BpeVocab, BpeTrainer, BpeEncoder,
    serialize_vocab, deserialize_vocab,
    PAD_ID, BOS_ID, EOS_ID, UNK_ID, IAM_ID,
    INTENT_ID, CONF_G_ID, CONF_Y_ID, CONF_R_ID,
    UNKNOWN_ID, SPECIAL_TOKEN_COUNT, BASE_VOCAB_SIZE,
    DEFAULT_VOCAB_SIZE, SPECIAL_TOKENS,
};

// ── T1: BpeVocab base size correct ────────────────────────────────────────────
#[test]
fn t1_vocab_base_size() {
    let vocab = BpeVocab::new();
    // 32 reserved + 256 base bytes = 288
    assert_eq!(vocab.size() as usize, SPECIAL_TOKEN_COUNT + BASE_VOCAB_SIZE);
}

// ── T2: Special tokens all registered ────────────────────────────────────────
#[test]
fn t2_special_tokens_registered() {
    let vocab = BpeVocab::new();
    for &(name, id) in SPECIAL_TOKENS {
        assert_eq!(vocab.special_id(name), Some(id),
            "special token {} not found", name);
    }
}

// ── T3: IAM special token IDs are correct ────────────────────────────────────
#[test]
fn t3_iam_token_ids() {
    assert_eq!(PAD_ID,     0);
    assert_eq!(BOS_ID,     1);
    assert_eq!(EOS_ID,     2);
    assert_eq!(UNK_ID,     3);
    assert_eq!(IAM_ID,     4);
    assert_eq!(INTENT_ID,  5);
    assert_eq!(CONF_G_ID,  7);
    assert_eq!(CONF_Y_ID,  8);
    assert_eq!(CONF_R_ID,  9);
    assert_eq!(UNKNOWN_ID, 10);
}

// ── T4: Special tokens are below SPECIAL_TOKEN_COUNT ─────────────────────────
#[test]
fn t4_special_tokens_in_range() {
    for &(_, id) in SPECIAL_TOKENS {
        assert!(id < SPECIAL_TOKEN_COUNT as u32,
            "special token {} must be < {}", id, SPECIAL_TOKEN_COUNT);
    }
}

// ── T5: is_special correctly identifies special tokens ───────────────────────
#[test]
fn t5_is_special() {
    let vocab = BpeVocab::new();
    assert!(vocab.is_special(PAD_ID));
    assert!(vocab.is_special(CONF_R_ID));
    assert!(!vocab.is_special(SPECIAL_TOKEN_COUNT as u32)); // first base byte
    assert!(!vocab.is_special(SPECIAL_TOKEN_COUNT as u32 + 65)); // 'A'
}

// ── T6: Base byte tokens correct ─────────────────────────────────────────────
#[test]
fn t6_base_byte_tokens() {
    let vocab = BpeVocab::new();
    // Byte 'A' (65) should be at ID SPECIAL_TOKEN_COUNT + 65
    let a_id = SPECIAL_TOKEN_COUNT as u32 + 65;
    assert_eq!(vocab.token_bytes(a_id), Some(&[65u8][..]));
}

// ── T7: add_merge grows vocabulary ───────────────────────────────────────────
#[test]
fn t7_add_merge() {
    let mut vocab = BpeVocab::new();
    let base = vocab.size();
    let id_a = SPECIAL_TOKEN_COUNT as u32 + 65; // 'A'
    let id_b = SPECIAL_TOKEN_COUNT as u32 + 66; // 'B'
    let new_id = vocab.add_merge(id_a, id_b);
    assert_eq!(new_id, base);
    assert_eq!(vocab.size(), base + 1);
    assert_eq!(vocab.merge_count(), 1);
    // Merged token should be "AB"
    assert_eq!(vocab.token_bytes(new_id), Some(&[65u8, 66u8][..]));
}

// ── T8: BPE trainer on minimal corpus ────────────────────────────────────────
#[test]
fn t8_trainer_minimal() {
    let corpus = "wisdom wisdom wisdom sovereignty sovereignty";
    let trainer = BpeTrainer::new(300); // small target
    let vocab = trainer.train(corpus);
    // Should have more tokens than base (merges happened)
    assert!(vocab.size() > (SPECIAL_TOKEN_COUNT + BASE_VOCAB_SIZE) as u32,
        "trainer should produce merges, vocab size={}", vocab.size());
    assert!(vocab.merge_count() > 0);
}

// ── T9: Trainer is deterministic ─────────────────────────────────────────────
#[test]
fn t9_trainer_deterministic() {
    let corpus = "the fear of the lord is the beginning of wisdom and all sovereign things";
    let v1 = BpeTrainer::new(320).train(corpus);
    let v2 = BpeTrainer::new(320).train(corpus);
    assert_eq!(v1.merge_count(), v2.merge_count(), "same corpus must produce same merge count");
    for (m1, m2) in v1.merges.iter().zip(v2.merges.iter()) {
        assert_eq!(m1.left,   m2.left);
        assert_eq!(m1.right,  m2.right);
        assert_eq!(m1.result, m2.result);
    }
}

// ── T10: Encoder encodes plain text ──────────────────────────────────────────
#[test]
fn t10_encoder_plain_text() {
    let vocab = BpeVocab::new(); // no merges
    let encoder = BpeEncoder::new(&vocab);
    let ids = encoder.encode("hi");
    // 'h' = 104, 'i' = 105
    assert!(ids.contains(&(SPECIAL_TOKEN_COUNT as u32 + 104)));
    assert!(ids.contains(&(SPECIAL_TOKEN_COUNT as u32 + 105)));
}

// ── T11: Encoder handles special tokens ──────────────────────────────────────
#[test]
fn t11_encoder_special_tokens() {
    let vocab = BpeVocab::new();
    let encoder = BpeEncoder::new(&vocab);
    let ids = encoder.encode("<conf_g>verified fact<conf_g>");
    assert!(ids.contains(&CONF_G_ID), "conf_g token must appear");
}

// ── T12: Encoder handles IAM identity marker ──────────────────────────────────
#[test]
fn t12_encoder_iam_token() {
    let vocab = BpeVocab::new();
    let encoder = BpeEncoder::new(&vocab);
    let ids = encoder.encode("<iam>I am here</iam>");
    assert!(ids.contains(&IAM_ID), "<iam> token must appear");
}

// ── T13: Confidence tier tokens all encodable ────────────────────────────────
#[test]
fn t13_confidence_tokens_encodable() {
    let vocab = BpeVocab::new();
    let encoder = BpeEncoder::new(&vocab);
    let text = "<conf_g>fact<conf_y>maybe<conf_r>unknown";
    let ids = encoder.encode(text);
    assert!(ids.contains(&CONF_G_ID), "conf_g missing");
    assert!(ids.contains(&CONF_Y_ID), "conf_y missing");
    assert!(ids.contains(&CONF_R_ID), "conf_r missing");
}

// ── T14: Intent boundary tokens encodable ────────────────────────────────────
#[test]
fn t14_intent_tokens_encodable() {
    let vocab = BpeVocab::new();
    let encoder = BpeEncoder::new(&vocab);
    let ids = encoder.encode("<intent>presence_unclear</intent>");
    assert!(ids.contains(&INTENT_ID), "<intent> missing");
}

// ── T15: Decode roundtrip (no merges) ────────────────────────────────────────
#[test]
fn t15_decode_roundtrip() {
    let vocab = BpeVocab::new();
    let encoder = BpeEncoder::new(&vocab);
    let original = "sovereign";
    let ids = encoder.encode(original);
    let decoded = encoder.decode(&ids);
    assert!(decoded.contains("sovereign"), "decoded='{}' should contain 'sovereign'", decoded);
}

// ── T16: Serialize/deserialize .axvocab ──────────────────────────────────────
#[test]
fn t16_axvocab_roundtrip() {
    let corpus = "wisdom sovereignty wisdom the beginning the";
    let vocab = BpeTrainer::new(300).train(corpus);
    let serialized = serialize_vocab(&vocab);
    assert!(serialized.starts_with("#axvocab v1"), "header missing");
    let vocab2 = deserialize_vocab(&serialized).unwrap();
    assert_eq!(vocab.merge_count(), vocab2.merge_count(),
        "merge count must survive roundtrip");
}

// ── T17: .axvocab contains special token declarations ────────────────────────
#[test]
fn t17_axvocab_contains_specials() {
    let vocab = BpeVocab::new();
    let s = serialize_vocab(&vocab);
    assert!(s.contains("special <iam>"), ".axvocab must declare <iam>");
    assert!(s.contains("special <conf_g>"), ".axvocab must declare <conf_g>");
    assert!(s.contains("special <conf_r>"), ".axvocab must declare <conf_r>");
    assert!(s.contains("special <intent>"), ".axvocab must declare <intent>");
}

// ── T18: Trainer respects min_frequency ──────────────────────────────────────
#[test]
fn t18_trainer_min_frequency() {
    // Word "rare" appears once, "common" appears 10 times
    let corpus = "common common common common common common common common common common rare";
    let v_strict = BpeTrainer::new(300).with_min_frequency(3).train(corpus);
    let v_lenient = BpeTrainer::new(300).with_min_frequency(1).train(corpus);
    // Lenient should produce more merges (includes rare pairs)
    assert!(v_lenient.merge_count() >= v_strict.merge_count(),
        "lenient min_freq should produce >= merges");
}

// ── T19: Default vocab size matches spec ─────────────────────────────────────
#[test]
fn t19_default_vocab_size() {
    // Spec: 8,000-16,000 vocabulary size for IAM seed
    assert!(DEFAULT_VOCAB_SIZE >= 8000 && DEFAULT_VOCAB_SIZE <= 16000,
        "DEFAULT_VOCAB_SIZE={} must be 8k-16k per spec", DEFAULT_VOCAB_SIZE);
}

// ── T20: Full IAM tokenizer pipeline ─────────────────────────────────────────
#[test]
fn t20_full_iam_pipeline() {
    // Simulate IAM Stage 0 corpus (constitutional + wisdom texts)
    let corpus = "the fear of the lord is the beginning of wisdom \
                  wisdom is sovereign truth sovereignty is the foundation \
                  help human never harm maximum capacity always \
                  i do not know would you like me to search \
                  reason from principle not from pattern";

    // Train tokenizer
    let vocab = BpeTrainer::new(350).train(corpus);
    assert!(vocab.merge_count() > 0, "corpus training must produce merges");
    assert!(vocab.special_id("<iam>") == Some(IAM_ID), "IAM token must survive training");
    assert!(vocab.special_id("<conf_g>") == Some(CONF_G_ID), "conf_g must survive training");

    // Encode a constitutional IAM response
    let response = "<conf_g>Wisdom is the beginning of sovereignty.</conf_g>";
    let encoder = BpeEncoder::new(&vocab);
    let ids = encoder.encode(response);
    assert!(!ids.is_empty(), "encoding must produce tokens");
    assert!(ids.contains(&CONF_G_ID), "conf_g must appear in encoded response");

    // Serialize and reload
    let serialized = serialize_vocab(&vocab);
    let vocab2 = deserialize_vocab(&serialized).unwrap();
    assert_eq!(vocab2.special_id("<conf_r>"), Some(CONF_R_ID),
        "conf_r must survive serialize/deserialize");

    // Verify all 16 special tokens present after roundtrip
    let encoder2 = BpeEncoder::new(&vocab2);
    let ids2 = encoder2.encode("<iam><intent><conf_g><conf_y><conf_r><unknown>");
    assert!(ids2.contains(&IAM_ID));
    assert!(ids2.contains(&INTENT_ID));
    assert!(ids2.contains(&CONF_G_ID));
    assert!(ids2.contains(&CONF_Y_ID));
    assert!(ids2.contains(&CONF_R_ID));
    assert!(ids2.contains(&UNKNOWN_ID));
}
