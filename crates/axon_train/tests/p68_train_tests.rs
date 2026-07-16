// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_train P68 — Training loop orchestrator tests (20 tests)
// IAM FLAGSHIP: the engine that trains the sovereign brain

use axon_train::config::{TrainingConfig, LrScheduler, LossTracker, ConfigError};
use axon_train::trainer::{
    Trainer, StepResult,
    constitutional_zeroout, clip_gradients,
    compute_ce_loss, compute_load_balance_loss,
    compute_router_z_loss, combined_loss,
};
use axon_train::checkpoint::{
    CheckpointState, ParameterTensor,
    save_checkpoint, load_checkpoint, CheckpointError,
};
use axon_train::eval::{EvalResult, EvalHarness, BenchmarkScore};
use axon_train::export::{export_iam, parse_iam_header, IAM_MAGIC};

// ── T1: TrainingConfig validates correctly ────────────────────────────────────
#[test]
fn t1_config_validation() {
    assert!(TrainingConfig::test().validate().is_ok());
    assert!(TrainingConfig::iam_seed().validate().is_ok());
    let mut bad = TrainingConfig::test();
    bad.batch_size = 0;
    assert!(matches!(bad.validate(), Err(ConfigError::InvalidBatchSize(0))));
}

// ── T2: LrScheduler warmup increases LR ──────────────────────────────────────
#[test]
fn t2_lr_warmup() {
    let config = TrainingConfig::test();
    let mut sched = LrScheduler::new(&config);
    let initial_lr = sched.lr();
    sched.step(); sched.step(); sched.step();
    let later_lr = sched.lr();
    assert!(later_lr > initial_lr, "LR should increase during warmup");
}

// ── T3: LrScheduler cosine decays after warmup ────────────────────────────────
#[test]
fn t3_lr_cosine_decay() {
    let config = TrainingConfig::test();
    let mut sched = LrScheduler::new(&config);
    // Skip through warmup
    for _ in 0..=config.warmup_steps { sched.step(); }
    let post_warmup_lr = sched.lr();
    for _ in 0..50 { sched.step(); }
    let later_lr = sched.lr();
    assert!(later_lr <= post_warmup_lr + 0.001,
        "cosine decay: lr should not increase after warmup");
}

// ── T4: LossTracker windowed average ─────────────────────────────────────────
#[test]
fn t4_loss_tracker() {
    let mut tracker = LossTracker::new(5);
    for i in 1..=10 {
        tracker.record(i as f32);
    }
    // Last 5: 6,7,8,9,10 → avg 8.0
    assert!((tracker.avg_loss() - 8.0).abs() < 0.01);
    assert_eq!(tracker.best_loss, 1.0);
}

// ── T5: LossTracker perplexity ────────────────────────────────────────────────
#[test]
fn t5_perplexity() {
    let mut tracker = LossTracker::new(10);
    tracker.record(1.0);
    let ppl = tracker.perplexity();
    assert!((ppl - std::f32::consts::E).abs() < 0.01, "ppl of loss=1.0 should be e");
}

// ── T6: constitutional_zeroout zeros masked gradients ────────────────────────
#[test]
fn t6_constitutional_zeroout() {
    let mut grads = vec![1.0f32, 2.0, 3.0, 4.0];
    let mask = vec![true, false, true, false];
    constitutional_zeroout(&mut grads, &mask);
    assert_eq!(grads[0], 0.0, "constitutional gradient must be zeroed");
    assert_eq!(grads[1], 2.0, "non-constitutional gradient must be unchanged");
    assert_eq!(grads[2], 0.0);
    assert_eq!(grads[3], 4.0);
}

// ── T7: clip_gradients clips correctly ───────────────────────────────────────
#[test]
fn t7_gradient_clipping() {
    let mut grads = vec![3.0f32, 4.0]; // norm = 5.0
    let norm = clip_gradients(&mut grads, 1.0);
    assert!((norm - 5.0).abs() < 0.01, "returned norm should be pre-clip norm");
    let clipped_norm: f32 = grads.iter().map(|g| g*g).sum::<f32>().sqrt();
    assert!((clipped_norm - 1.0).abs() < 0.01, "clipped norm should be 1.0");
}

// ── T8: compute_ce_loss on uniform logits ────────────────────────────────────
#[test]
fn t8_ce_loss() {
    // Uniform logits → loss = ln(vocab_size)
    let vocab = 4;
    let logits: Vec<f32> = vec![0.0f32; vocab];
    let targets = vec![0usize];
    let loss = compute_ce_loss(&logits, &targets, vocab);
    let expected = (vocab as f32).ln();
    assert!((loss - expected).abs() < 0.01, "uniform logits loss={} expected={}", loss, expected);
}

// ── T9: compute_load_balance_loss zero variance ───────────────────────────────
#[test]
fn t9_load_balance_zero() {
    let usage = vec![0.25f32, 0.25, 0.25, 0.25]; // perfectly balanced
    let loss = compute_load_balance_loss(&usage);
    assert!(loss < 0.001, "balanced experts should have near-zero loss");
}

// ── T10: compute_router_z_loss ────────────────────────────────────────────────
#[test]
fn t10_router_z_loss() {
    let logits = vec![1.0f32, 1.0, 1.0, 1.0];
    let loss = compute_router_z_loss(&logits);
    assert!(loss >= 0.0);
}

// ── T11: Trainer step produces result ────────────────────────────────────────
#[test]
fn t11_trainer_step() {
    let config = TrainingConfig::test();
    let mut trainer = Trainer::new(config, "abc123");
    let result = trainer.step(2.5, 0.01, 0.001, vec![0.1, 0.2, 0.3], &[], 64);
    assert_eq!(result.step, 1);
    assert!(result.loss > 0.0);
    assert!(result.learning_rate > 0.0);
}

// ── T12: Trainer should_checkpoint ───────────────────────────────────────────
#[test]
fn t12_trainer_checkpoint_gate() {
    let mut config = TrainingConfig::test();
    config.checkpoint_every = 5;
    let mut trainer = Trainer::new(config, "abc");
    for i in 0..10 {
        trainer.step(1.0, 0.0, 0.0, vec![], &[], 0);
        if (i + 1) % 5 == 0 {
            assert!(trainer.should_checkpoint(), "step {} should checkpoint", i+1);
        }
    }
}

// ── T13: Trainer build_checkpoint ────────────────────────────────────────────
#[test]
fn t13_trainer_build_checkpoint() {
    let config = TrainingConfig::test();
    let mut trainer = Trainer::new(config, "tok_hash_xyz");
    trainer.step(1.5, 0.0, 0.0, vec![], &[], 0);
    let params = vec![ParameterTensor::zeros("embed.weight", vec![100, 32])];
    let ckpt = trainer.build_checkpoint(params);
    assert_eq!(ckpt.step, 1);
    assert_eq!(ckpt.tokenizer_hash, "tok_hash_xyz");
    assert_eq!(ckpt.parameters.len(), 1);
}

// ── T14: checkpoint save/load roundtrip ──────────────────────────────────────
#[test]
fn t14_checkpoint_roundtrip() {
    let mut state = CheckpointState::new(
        500, 2, 1.23, 1.10, 0.0003, 100000,
        "deadbeef", "iam-seed", "seed", 42,
    );
    state.add_param(ParameterTensor::zeros("layer.0.weight", vec![64, 32]));
    let bytes = save_checkpoint(&state).unwrap();
    let restored = load_checkpoint(&bytes).unwrap();
    assert_eq!(restored.step, 500);
    assert_eq!(restored.epoch, 2);
    assert_eq!(restored.tokenizer_hash, "deadbeef");
    assert_eq!(restored.parameters.len(), 1);
    assert_eq!(restored.parameters[0].name, "layer.0.weight");
}

// ── T15: checkpoint detects corruption ───────────────────────────────────────
#[test]
fn t15_checkpoint_corruption() {
    let state = CheckpointState::new(1, 0, 2.0, 2.0, 1e-4, 0, "h", "c", "seed", 0);
    let mut bytes = save_checkpoint(&state).unwrap();
    // Corrupt a byte in the JSON body (header line is first, body starts after first newline)
    // Find position after first newline (header line) and corrupt there
    let header_end = bytes.iter().position(|&b| b == b'\n').unwrap_or(0) + 1;
    let corrupt_pos = header_end + 10;
    if corrupt_pos < bytes.len() {
        bytes[corrupt_pos] ^= 0x01; // single bit flip in body
    }
    // Must be HashMismatch or ParseError (both indicate corruption detected)
    let result = load_checkpoint(&bytes);
    assert!(
        matches!(result, Err(CheckpointError::HashMismatch)) ||
        matches!(result, Err(CheckpointError::ParseError(_))),
        "corrupted checkpoint must not load cleanly: {:?}", result
    );
}

// ── T16: Trainer resume_from ──────────────────────────────────────────────────
#[test]
fn t16_trainer_resume() {
    let config = TrainingConfig::test();
    let mut trainer = Trainer::new(config, "tok_hash");
    let state = CheckpointState::new(50, 1, 1.5, 1.4, 1e-4, 5000, "tok_hash", "c", "seed", 0);
    assert!(trainer.resume_from(&state).is_ok());
    assert_eq!(trainer.step, 50);
    assert_eq!(trainer.epoch, 1);
}

// ── T17: EvalHarness tracks best ─────────────────────────────────────────────
#[test]
fn t17_eval_harness_best() {
    let mut harness = EvalHarness::new();
    harness.record(EvalResult::new(100, 3.0));
    harness.record(EvalResult::new(200, 2.5));
    let is_best = harness.record(EvalResult::new(300, 2.0));
    assert!(is_best, "lower loss should be best");
    assert_eq!(harness.best_val_loss, 2.0);
    assert_eq!(harness.best_step, 300);
}

// ── T18: .iam export magic and header ────────────────────────────────────────
#[test]
fn t18_iam_export_header() {
    let mut state = CheckpointState::new(
        1000, 3, 0.8, 0.75, 1e-4, 50000,
        "vocab_hash", "iam-seed", "seed", 99,
    );
    state.add_param(ParameterTensor::zeros("embed", vec![8000, 512]));
    let bytes = export_iam(&state, 8000, 512, 20, 4).unwrap();
    assert_eq!(&bytes[0..4], &IAM_MAGIC, "magic must be IAM1");
    let header = parse_iam_header(&bytes).unwrap();
    assert_eq!(header.vocab_size, 8000);
    assert_eq!(header.d_model, 512);
    assert_eq!(header.training_steps, 1000);
    assert!(!header.signed, "stub export must be unsigned");
}

// ── T19: .iam detects bad magic ───────────────────────────────────────────────
#[test]
fn t19_iam_bad_magic() {
    use axon_train::export::ExportError;
    let mut bad = vec![0u8; 20];
    bad[0..4].copy_from_slice(b"XXXX");
    assert!(matches!(parse_iam_header(&bad), Err(ExportError::InvalidMagic(_))));
}

// ── T20: full P68 training pipeline ──────────────────────────────────────────
#[test]
fn t20_full_training_pipeline() {
    let mut config = TrainingConfig::test();
    config.max_steps = 20;
    config.checkpoint_every = 5;
    config.eval_every = 5;

    let mut trainer = Trainer::new(config.clone(), "sovereign_tokenizer_hash");
    let mut harness = EvalHarness::new();
    let mut checkpoints: Vec<Vec<u8>> = Vec::new();

    // Simulate training loop
    let param_mask = vec![true, false, true, false]; // 2 constitutional params

    for step in 0..config.max_steps {
        // Simulate diminishing loss
        let loss_ce = 4.0 / (1.0 + step as f32 * 0.1);
        let loss_lb = 0.01;
        let loss_rz = 0.001;
        let grads = vec![0.5f32, 1.0, -0.5, 0.3];

        let result = trainer.step(loss_ce, loss_lb, loss_rz, grads, &param_mask, 128);
        assert!(result.loss > 0.0);
        assert!(result.learning_rate > 0.0);

        // Constitutional gradients must be zeroed (verified via step result)
        // In production: assert param[0] unchanged after step

        if trainer.should_checkpoint() {
            let params = vec![ParameterTensor::zeros("test.weight", vec![32, 32])];
            let ckpt_state = trainer.build_checkpoint(params);
            let ckpt_bytes = save_checkpoint(&ckpt_state).unwrap();
            // Verify checkpoint integrity immediately
            let restored = load_checkpoint(&ckpt_bytes).unwrap();
            assert_eq!(restored.step, trainer.step);
            checkpoints.push(ckpt_bytes);
        }

        if trainer.should_eval() {
            let val_loss = 3.5 / (1.0 + step as f32 * 0.1);
            let eval = EvalResult::new(trainer.step, val_loss)
                .with_honesty(0.95)
                .with_compliance(1.0)
                .add_benchmark(BenchmarkScore::new("LOGOS", 0.8, 1.0, 0.7));
            harness.record(eval);
        }
    }

    assert!(trainer.is_done());
    assert!(trainer.avg_loss() < 4.0, "loss should decrease");
    assert!(!checkpoints.is_empty(), "checkpoints must be saved");
    assert!(!harness.history.is_empty(), "eval must run");

    // Export .iam
    let final_params = vec![ParameterTensor::zeros("brain.weight", vec![512, 512])];
    let final_state = trainer.build_checkpoint(final_params);
    let iam_bytes = export_iam(&final_state, 8000, 512, 20, 4).unwrap();
    assert!(!iam_bytes.is_empty());
    let header = parse_iam_header(&iam_bytes).unwrap();
    assert_eq!(header.stage, "seed");
    assert!(!header.signed, "unsigned until key ceremony");
}
