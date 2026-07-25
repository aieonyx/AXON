// Copyright (c) 2026 Edison Lepiten / AIEONYX
// IAM-M2 — Sovereign Tokenizer Training Binary
// Corpus and output paths: ~/Documents/IAM/

use std::{env, fs, path::PathBuf};
use axon_data::tokenizer::{BpeTrainer, serialize_vocab};
use axon_data::shard::sovereign_hash_hex;

fn main() {
    let args: Vec<String> = env::args().collect();
    let home = env::var("HOME").unwrap_or_default();

    let mut corpus_dir: Option<PathBuf> = None;
    let mut jsonl_path: Option<PathBuf> = None;
    let mut vocab_size: usize = 8000;
    let mut output = PathBuf::from(
        format!("{}/Documents/IAM/corpus/iam_seed.axvocab", home)
    );

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--corpus"     => { corpus_dir = Some(PathBuf::from(&args[i+1])); i += 2; }
            "--jsonl"      => { jsonl_path = Some(PathBuf::from(&args[i+1])); i += 2; }
            "--vocab-size" => { vocab_size = args[i+1].parse().unwrap_or(8000); i += 2; }
            "--output"     => { output = PathBuf::from(&args[i+1]); i += 2; }
            _ => { i += 1; }
        }
    }

    println!("=== IAM-M2 Tokenizer Training ===");
    println!("Target vocab size: {}", vocab_size);
    println!("Output:            {}", output.display());
    println!();

    let mut corpus = String::new();

    // Load text files from corpus/clean/
    if let Some(ref dir) = corpus_dir {
        println!("Loading corpus from: {}", dir.display());
        let mut entries: Vec<_> = fs::read_dir(dir)
            .expect("cannot read corpus dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "txt").unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let path = entry.path();
            let fname = path.file_name().unwrap().to_string_lossy().to_string();
            if let Ok(text) = fs::read_to_string(&path) {
                println!("  {:<40} {} words", fname, text.split_whitespace().count());
                corpus.push_str(&text);
                corpus.push('\n');
            }
        }
    }

    // Load JSONL pairs (adds domain vocabulary coverage)
    if let Some(ref jsonl) = jsonl_path {
        println!("Loading JSONL from: {}", jsonl.display());
        if let Ok(content) = fs::read_to_string(jsonl) {
            let mut count = 0usize;
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() { continue; }
                if let Ok(obj) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(q) = obj["q"].as_str() {
                        corpus.push_str(q); corpus.push('\n');
                    }
                    if let Some(a) = obj["a"].as_str() {
                        let text = if let Some(p) = a.find('|') { &a[p+1..] } else { a };
                        corpus.push_str(text); corpus.push('\n');
                    }
                    count += 1;
                }
            }
            println!("  {} pairs loaded", count);
        }
    }

    let total_words = corpus.split_whitespace().count();
    println!();
    println!("Total corpus: {} words (~{} estimated tokens)", total_words, corpus.len() / 4);
    println!();
    println!("Training BPE tokenizer...");
    println!("(This may take 2-5 minutes on full corpus)");
    println!();

    let trainer = BpeTrainer::new(vocab_size);
    let vocab = trainer.train(&corpus);

    println!("Training complete.");
    println!("  Vocab size:    {}", vocab.size());
    println!("  Merge rules:   {}", vocab.merge_count());
    println!("  Special tokens: {}", vocab.special_tokens.len());
    println!();

    // Serialize and save
    let serialized = serialize_vocab(&vocab);
    let hash = sovereign_hash_hex(serialized.as_bytes());

    if let Some(p) = output.parent() { fs::create_dir_all(p).ok(); }
    fs::write(&output, &serialized).expect("cannot write .axvocab");
    let hash_path = output.with_extension("axvocab.hash");
    fs::write(&hash_path, &hash).expect("cannot write hash");

    println!("=== IAM-M2 COMPLETE ===");
    println!();
    println!("Output:          {}", output.display());
    println!("tokenizer_hash:  {}", hash);
    println!("Hash file:       {}", hash_path.display());
    println!();
    println!("LOCK THIS HASH into axon_train TrainingConfig::iam_seed()");
    println!("before starting IAM-M3 pretraining.");
    println!();
    println!("Next: IAM-M3 — IAMSeed pretraining");
    println!();
    println!("Epoch declaration: Wisdom is the Beginning.");
}
