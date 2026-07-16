// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_train P68-M2 — Training loop core
//
// IAM FLAGSHIP: this is the engine that trains the sovereign brain.
//
// Loop: sampler → pack → forward (checkpointed) →
//       loss (CE + load_balance + router_z) →
//       backward → grad clip → constitutional zero-out →
//       Adam step → checkpoint gate → eval gate

use crate::config::{TrainingConfig, LrScheduler, LossTracker};
use crate::checkpoint::{CheckpointState, ParameterTensor};

// ── Training step result ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StepResult {
    pub step: usize,
    pub loss: f32,
    pub loss_ce: f32,
    pub loss_load_balance: f32,
    pub loss_router_z: f32,
    pub learning_rate: f32,
    pub grad_norm: f32,
    pub tokens_processed: usize,
}

// ── Constitutional zero-out ───────────────────────────────────────────────────
// IAM spec: constitutional parameters must not drift.
// PARAM MASK: any gradient targeting constitutional weights is zeroed.
// In production: this is enforced by comparing parameter names against
// the locked constitution manifest.

pub fn constitutional_zeroout(
    gradients: &mut Vec<f32>,
    param_mask: &[bool],
) {
    for (grad, &is_constitutional) in gradients.iter_mut().zip(param_mask.iter()) {
        if is_constitutional {
            *grad = 0.0;
        }
    }
}

// ── Gradient clipping ─────────────────────────────────────────────────────────

pub fn clip_gradients(gradients: &mut Vec<f32>, max_norm: f32) -> f32 {
    let norm: f32 = gradients.iter().map(|g| g * g).sum::<f32>().sqrt();
    if norm > max_norm {
        let scale = max_norm / norm;
        for g in gradients.iter_mut() { *g *= scale; }
    }
    norm
}

// ── Loss components ───────────────────────────────────────────────────────────

/// Compute cross-entropy loss on a flat logit/target pair.
/// In production: operates on full vocabulary logits.
pub fn compute_ce_loss(logits: &[f32], targets: &[usize], vocab_size: usize) -> f32 {
    if logits.is_empty() || targets.is_empty() { return 0.0; }
    let mut total = 0.0f32;
    let chunk_size = vocab_size;
    for (i, &target) in targets.iter().enumerate() {
        let start = i * chunk_size;
        let end = (start + chunk_size).min(logits.len());
        if start >= logits.len() { break; }
        let chunk = &logits[start..end];
        // Stable softmax
        let max_val = chunk.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exp_sum: f32 = chunk.iter().map(|&x| (x - max_val).exp()).sum();
        let target_idx = target.min(chunk.len().saturating_sub(1));
        let log_prob = (chunk[target_idx] - max_val) - exp_sum.ln();
        total -= log_prob;
    }
    total / targets.len().max(1) as f32
}

/// Load balance loss (MoE regularization) — penalizes expert imbalance.
/// λ = 0.01 per IAM spec.
pub fn compute_load_balance_loss(expert_usage: &[f32]) -> f32 {
    if expert_usage.is_empty() { return 0.0; }
    let n = expert_usage.len() as f32;
    let mean = expert_usage.iter().sum::<f32>() / n;
    let variance = expert_usage.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / n;
    0.01 * variance
}

/// Router z-loss (encourages clean routing decisions).
/// λ = 1e-3 per IAM spec.
pub fn compute_router_z_loss(router_logits: &[f32]) -> f32 {
    if router_logits.is_empty() { return 0.0; }
    let logsumexp: f32 = router_logits.iter()
        .map(|&x| x.exp())
        .sum::<f32>()
        .ln();
    1e-3 * logsumexp.powi(2)
}

/// Combined IAM training loss: CE + load_balance + router_z
pub fn combined_loss(ce: f32, lb: f32, rz: f32) -> f32 {
    ce + lb + rz
}

// ── Trainer state ─────────────────────────────────────────────────────────────

pub struct Trainer {
    pub config: TrainingConfig,
    pub scheduler: LrScheduler,
    pub loss_tracker: LossTracker,
    pub step: usize,
    pub epoch: usize,
    pub corpus_position: usize,
    /// Tokenizer hash (for checkpoint integrity)
    pub tokenizer_hash: String,
}

impl Trainer {
    pub fn new(config: TrainingConfig, tokenizer_hash: &str) -> Self {
        let scheduler = LrScheduler::new(&config);
        let loss_tracker = LossTracker::new(100); // 100-step window
        Self {
            config,
            scheduler,
            loss_tracker,
            step: 0,
            epoch: 0,
            corpus_position: 0,
            tokenizer_hash: tokenizer_hash.to_string(),
        }
    }

    /// Simulate a single training step (P68-M2 core loop).
    /// In production: runs real forward/backward through the model.
    /// Here: accepts pre-computed loss values for loop orchestration testing.
    pub fn step(
        &mut self,
        loss_ce: f32,
        loss_lb: f32,
        loss_rz: f32,
        mut gradients: Vec<f32>,
        param_mask: &[bool],
        tokens_processed: usize,
    ) -> StepResult {
        // 1. Constitutional zero-out (PARAM MASK)
        if !param_mask.is_empty() && gradients.len() == param_mask.len() {
            constitutional_zeroout(&mut gradients, param_mask);
        }

        // 2. Gradient clipping
        let grad_norm = clip_gradients(&mut gradients, self.config.grad_clip);

        // 3. Combined loss
        let loss = combined_loss(loss_ce, loss_lb, loss_rz);

        // 4. Record loss
        self.loss_tracker.record(loss);

        // 5. LR step
        let lr = self.scheduler.lr();
        self.scheduler.step();
        self.step += 1;
        self.corpus_position += tokens_processed;

        StepResult {
            step: self.step,
            loss,
            loss_ce,
            loss_load_balance: loss_lb,
            loss_router_z: loss_rz,
            learning_rate: lr,
            grad_norm,
            tokens_processed,
        }
    }

    /// Whether a checkpoint should be saved at this step.
    pub fn should_checkpoint(&self) -> bool {
        self.step > 0 && self.step % self.config.checkpoint_every == 0
    }

    /// Whether evaluation should run at this step.
    pub fn should_eval(&self) -> bool {
        self.step > 0 && self.step % self.config.eval_every == 0
    }

    /// Build a checkpoint state from current trainer state.
    pub fn build_checkpoint(
        &self,
        parameters: Vec<ParameterTensor>,
    ) -> CheckpointState {
        let mut state = CheckpointState::new(
            self.step,
            self.epoch,
            self.loss_tracker.last_loss().unwrap_or(0.0),
            self.loss_tracker.best_loss,
            self.scheduler.lr(),
            self.corpus_position,
            &self.tokenizer_hash,
            &self.config.corpus_name,
            &self.config.stage,
            (self.step as u64).wrapping_mul(0x9e3779b97f4a7c15), // deterministic seed
        );
        for p in parameters {
            state.add_param(p);
        }
        state
    }

    /// Resume from a checkpoint state.
    pub fn resume_from(&mut self, state: &CheckpointState) -> Result<(), String> {
        if state.tokenizer_hash != self.tokenizer_hash {
            return Err(format!(
                "tokenizer hash mismatch: checkpoint={} current={}",
                state.tokenizer_hash, self.tokenizer_hash
            ));
        }
        self.step = state.step;
        self.epoch = state.epoch;
        self.corpus_position = state.corpus_position;
        self.scheduler.current_step = state.step;
        Ok(())
    }

    pub fn current_lr(&self) -> f32 { self.scheduler.lr() }
    pub fn avg_loss(&self) -> f32 { self.loss_tracker.avg_loss() }
    pub fn perplexity(&self) -> f32 { self.loss_tracker.perplexity() }
    pub fn is_done(&self) -> bool { self.step >= self.config.max_steps }
}
