// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_data P67-M4 — Batch iterator tests (20 tests)

use axon_data::batch::{
    TrainingSequence, TokenizedEntry, MixtureSampler,
    BatchIterator, pack_sequences, SEQ_LEN,
};
use axon_data::tokenizer::{BOS_ID, EOS_ID, PAD_ID};
use axon_data::types::DataTier;

fn entry(tokens: Vec<u32>, tier: DataTier) -> TokenizedEntry {
    TokenizedEntry::new(tokens, tier, "test")
}
fn tok(n: usize) -> Vec<u32> { (100..100 + n as u32).collect() }

#[test]
fn t1_seq_len() { assert_eq!(SEQ_LEN, 4096); }

#[test]
fn t2_empty_sequence_padded() {
    let seq = TrainingSequence::empty();
    assert_eq!(seq.tokens.len(), SEQ_LEN);
    assert!(seq.tokens.iter().all(|&t| t == PAD_ID));
    assert_eq!(seq.real_token_count, 0);
}

#[test]
fn t3_pack_bos_eos() {
    let entries = vec![entry(tok(10), DataTier::Noise)];
    let seqs = pack_sequences(&entries);
    assert!(!seqs.is_empty());
    assert_eq!(seqs[0].tokens[0], BOS_ID);
    assert_eq!(seqs[0].tokens[11], EOS_ID);
}

#[test]
fn t4_pack_padded_to_seq_len() {
    let entries = vec![entry(tok(10), DataTier::Noise)];
    let seqs = pack_sequences(&entries);
    assert_eq!(seqs[0].tokens.len(), SEQ_LEN);
}

#[test]
fn t5_attention_mask_real_tokens() {
    let entries = vec![entry(tok(10), DataTier::Noise)];
    let seqs = pack_sequences(&entries);
    assert_eq!(seqs[0].real_token_count, 12); // 10 + BOS + EOS
    for i in 0..12 { assert_eq!(seqs[0].attention_mask[i], 1); }
    for i in 12..SEQ_LEN { assert_eq!(seqs[0].attention_mask[i], 0); }
}

#[test]
fn t6_doc_boundaries() {
    let entries = vec![entry(tok(10), DataTier::Noise), entry(tok(10), DataTier::Noise)];
    let seqs = pack_sequences(&entries);
    assert_eq!(seqs[0].doc_boundaries.len(), 2);
    assert_eq!(seqs[0].doc_boundaries[0], 0);
}

#[test]
fn t7_long_entry_truncated() {
    let seqs = pack_sequences(&[entry(tok(SEQ_LEN + 100), DataTier::Noise)]);
    assert!(!seqs.is_empty());
    assert_eq!(seqs[0].tokens.len(), SEQ_LEN);
}

#[test]
fn t8_multiple_sequences() {
    let entries: Vec<_> = (0..100).map(|_| entry(tok(100), DataTier::Noise)).collect();
    let seqs = pack_sequences(&entries);
    assert!(seqs.len() >= 2);
}

#[test]
fn t9_critical_tier_propagates() {
    let entries = vec![entry(tok(10), DataTier::Critical), entry(tok(10), DataTier::Noise)];
    let seqs = pack_sequences(&entries);
    assert_eq!(seqs[0].tier, DataTier::Critical);
}

#[test]
fn t10_utilization() {
    let entries = vec![entry(tok(10), DataTier::Noise)];
    let seqs = pack_sequences(&entries);
    let util = seqs[0].utilization();
    assert!(util > 0.0 && util < 0.01);
}

#[test]
fn t11_mixture_sampler_split() {
    let entries = vec![
        entry(tok(5), DataTier::Critical),
        entry(tok(5), DataTier::Personal),
        entry(tok(5), DataTier::Noise),
        entry(tok(5), DataTier::Noise),
    ];
    let sampler = MixtureSampler::new(entries);
    assert_eq!(sampler.critical_count(), 1);
    assert_eq!(sampler.personal_count(), 1);
    assert_eq!(sampler.noise_count(), 2);
}

#[test]
fn t12_mixture_epoch_size() {
    let entries = vec![
        entry(tok(5), DataTier::Critical),
        entry(tok(5), DataTier::Personal),
        entry(tok(5), DataTier::Noise),
    ];
    let sampler = MixtureSampler::new(entries);
    assert_eq!(sampler.epoch_size(), 6);
}

#[test]
fn t13_mixture_epoch_count() {
    let entries = vec![
        entry(tok(5), DataTier::Critical),
        entry(tok(5), DataTier::Personal),
        entry(tok(5), DataTier::Noise),
    ];
    let mut sampler = MixtureSampler::new(entries);
    assert_eq!(sampler.build_epoch_sequence().len(), 6);
}

#[test]
fn t14_mixture_constitutional_present() {
    let entries = vec![
        entry(vec![42], DataTier::Critical),
        entry(vec![99], DataTier::Noise),
    ];
    let mut sampler = MixtureSampler::new(entries);
    let epoch = sampler.build_epoch_sequence();
    let crit = epoch.iter().filter(|e| e.tier == DataTier::Critical).count();
    assert_eq!(crit, 3);
}

#[test]
fn t15_batch_iterator_yields() {
    let entries: Vec<_> = (0..20).map(|_| entry(tok(10), DataTier::Noise)).collect();
    let seqs = pack_sequences(&entries);
    let expected = (seqs.len() + 3) / 4;
    let mut iter = BatchIterator::new(seqs, 4);
    let mut count = 0;
    while iter.next_batch().is_some() { count += 1; }
    assert_eq!(count, expected);
}

#[test]
fn t16_batch_size() {
    let entries: Vec<_> = (0..50).map(|_| entry(tok(10), DataTier::Noise)).collect();
    let seqs = pack_sequences(&entries);
    let mut iter = BatchIterator::new(seqs, 8);
    if let Some(batch) = iter.next_batch() {
        assert!(batch.batch_size <= 8);
    }
}

#[test]
fn t17_iterator_exhausted() {
    let seqs = pack_sequences(&[entry(tok(10), DataTier::Noise)]);
    let mut iter = BatchIterator::new(seqs, 4);
    let _ = iter.next_batch();
    assert!(iter.next_batch().is_none());
}

#[test]
fn t18_iterator_reset() {
    let entries: Vec<_> = (0..10).map(|_| entry(tok(10), DataTier::Noise)).collect();
    let seqs = pack_sequences(&entries);
    let mut iter = BatchIterator::new(seqs, 4);
    while iter.next_batch().is_some() {}
    iter.reset_epoch();
    assert!(iter.has_next());
    assert_eq!(iter.epoch(), 1);
}

#[test]
fn t19_batch_total_tokens() {
    let entries = vec![entry(tok(10), DataTier::Noise), entry(tok(20), DataTier::Noise)];
    let seqs = pack_sequences(&entries);
    let mut iter = BatchIterator::new(seqs, 4);
    if let Some(batch) = iter.next_batch() {
        assert_eq!(batch.total_tokens(), 34); // (10+2) + (20+2)
    }
}

#[test]
fn t20_full_batch_pipeline() {
    let entries = vec![
        entry(tok(50), DataTier::Critical),
        entry(tok(60), DataTier::Critical),
        entry(tok(30), DataTier::Personal),
        entry(tok(40), DataTier::Noise),
        entry(tok(45), DataTier::Noise),
        entry(tok(35), DataTier::Noise),
    ];
    let mut sampler = MixtureSampler::new(entries);
    assert_eq!(sampler.epoch_size(), 11); // 2*3+1*2+3*1

    let epoch_entries = sampler.build_epoch_sequence();
    assert_eq!(epoch_entries.len(), 11);

    let crit = epoch_entries.iter().filter(|e| e.tier == DataTier::Critical).count();
    assert_eq!(crit, 6);

    let seqs = pack_sequences(&epoch_entries);
    assert!(!seqs.is_empty());
    for seq in &seqs {
        assert_eq!(seq.tokens.len(), SEQ_LEN);
        assert_eq!(seq.attention_mask.len(), SEQ_LEN);
    }

    let mut iter = BatchIterator::new(seqs, 4);
    let mut total_tokens = 0;
    while let Some(batch) = iter.next_batch() {
        total_tokens += batch.total_tokens();
        assert!(batch.avg_utilization() > 0.0);
    }
    assert!(total_tokens > 0);
}
