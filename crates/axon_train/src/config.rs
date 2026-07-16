// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_train P68-M1 — Training configuration + LR scheduler + loss tracker

use serde::{Deserialize, Serialize};

/// Full IAM training configuration.
/// Matches IAM Founding Spec §10 (Phase Plan) and AIEONYX-SPEC-IAM-IMPL-v1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    // ── Model ────────────────────────────────────────────────────────────────
    /// Model parameter count (target: 120M for IAMSeed)
    pub param_count: usize,
    /// Vocabulary size (must match axon_data tokenizer)
    pub vocab_size: usize,
    /// Sequence length (must match axon_data SEQ_LEN = 4096)
    pub seq_len: usize,
    /// Embedding dimension
    pub d_model: usize,
    /// Number of SSM layers
    pub n_ssm_layers: usize,
    /// Number of attention layers
    pub n_attn_layers: usize,

    // ── Training ─────────────────────────────────────────────────────────────
    /// Batch size (sequences per step)
    pub batch_size: usize,
    /// Total training steps
    pub max_steps: usize,
    /// Initial learning rate
    pub learning_rate: f32,
    /// Gradient clipping norm
    pub grad_clip: f32,
    /// Adam beta1
    pub adam_beta1: f32,
    /// Adam beta2
    pub adam_beta2: f32,
    /// Adam epsilon
    pub adam_epsilon: f32,

    // ── Scheduling ───────────────────────────────────────────────────────────
    /// Warmup steps (linear LR warmup)
    pub warmup_steps: usize,
    /// LR decay schedule ("cosine" | "linear" | "constant")
    pub lr_schedule: String,

    // ── Checkpointing ────────────────────────────────────────────────────────
    /// Save checkpoint every N steps
    pub checkpoint_every: usize,
    /// Auto-checkpoint every N wall-clock seconds (night-shift friendly)
    pub checkpoint_every_secs: u64,
    /// Maximum checkpoints to keep
    pub max_checkpoints: usize,

    // ── Evaluation ───────────────────────────────────────────────────────────
    /// Run evaluation every N steps
    pub eval_every: usize,
    /// Number of eval batches
    pub eval_batches: usize,

    // ── Corpus ───────────────────────────────────────────────────────────────
    /// Corpus name (must match .axd registry)
    pub corpus_name: String,
    /// Training stage ("seed" | "knowledge" | "domain")
    pub stage: String,
}

impl TrainingConfig {
    /// IAMSeed configuration — 120M params, Stage 0, Ryzen 7 only.
    pub fn iam_seed() -> Self {
        Self {
            param_count:     120_000_000,
            vocab_size:      8_000,
            seq_len:         4_096,
            d_model:         512,
            n_ssm_layers:    20,
            n_attn_layers:   4,
            batch_size:      2,
            max_steps:       100_000,
            learning_rate:   3e-4,
            grad_clip:       1.0,
            adam_beta1:      0.9,
            adam_beta2:      0.999,
            adam_epsilon:    1e-8,
            warmup_steps:    1_000,
            lr_schedule:     "cosine".into(),
            checkpoint_every: 1_000,
            checkpoint_every_secs: 1_800, // 30 min — night-shift friendly
            max_checkpoints: 5,
            eval_every:      500,
            eval_batches:    10,
            corpus_name:     "iam-seed-v0.1".into(),
            stage:           "seed".into(),
        }
    }

    /// Minimal config for unit testing.
    pub fn test() -> Self {
        Self {
            param_count:     1_000,
            vocab_size:      288, // base vocab only
            seq_len:         64,
            d_model:         32,
            n_ssm_layers:    2,
            n_attn_layers:   1,
            batch_size:      2,
            max_steps:       100,
            learning_rate:   1e-3,
            grad_clip:       1.0,
            adam_beta1:      0.9,
            adam_beta2:      0.999,
            adam_epsilon:    1e-8,
            warmup_steps:    10,
            lr_schedule:     "cosine".into(),
            checkpoint_every: 10,
            checkpoint_every_secs: 3600,
            max_checkpoints: 3,
            eval_every:      10,
            eval_batches:    2,
            corpus_name:     "test-corpus".into(),
            stage:           "seed".into(),
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.batch_size == 0 {
            return Err(ConfigError::InvalidBatchSize(0));
        }
        if self.learning_rate <= 0.0 || self.learning_rate > 1.0 {
            return Err(ConfigError::InvalidLearningRate(self.learning_rate));
        }
        if self.grad_clip <= 0.0 {
            return Err(ConfigError::InvalidGradClip(self.grad_clip));
        }
        if self.vocab_size == 0 {
            return Err(ConfigError::InvalidVocabSize(0));
        }
        if !["cosine", "linear", "constant"].contains(&self.lr_schedule.as_str()) {
            return Err(ConfigError::InvalidSchedule(self.lr_schedule.clone()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    InvalidBatchSize(usize),
    InvalidLearningRate(f32),
    InvalidGradClip(f32),
    InvalidVocabSize(usize),
    InvalidSchedule(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidBatchSize(n)   => write!(f, "invalid batch_size: {}", n),
            Self::InvalidLearningRate(r)=> write!(f, "invalid learning_rate: {}", r),
            Self::InvalidGradClip(c)    => write!(f, "invalid grad_clip: {}", c),
            Self::InvalidVocabSize(n)   => write!(f, "invalid vocab_size: {}", n),
            Self::InvalidSchedule(s)    => write!(f, "invalid lr_schedule: {}", s),
        }
    }
}

// ── LR Scheduler ─────────────────────────────────────────────────────────────

/// Learning rate scheduler with warmup.
#[derive(Debug, Clone)]
pub struct LrScheduler {
    pub base_lr: f32,
    pub warmup_steps: usize,
    pub max_steps: usize,
    pub schedule: ScheduleType,
    pub current_step: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScheduleType {
    Cosine,
    Linear,
    Constant,
}

impl LrScheduler {
    pub fn new(config: &TrainingConfig) -> Self {
        let schedule = match config.lr_schedule.as_str() {
            "cosine"   => ScheduleType::Cosine,
            "linear"   => ScheduleType::Linear,
            _          => ScheduleType::Constant,
        };
        Self {
            base_lr: config.learning_rate,
            warmup_steps: config.warmup_steps,
            max_steps: config.max_steps,
            schedule,
            current_step: 0,
        }
    }

    /// Compute current learning rate.
    pub fn lr(&self) -> f32 {
        let step = self.current_step as f32;
        let warmup = self.warmup_steps as f32;
        let max = self.max_steps as f32;

        // Linear warmup
        if self.current_step < self.warmup_steps {
            return self.base_lr * (step + 1.0) / warmup.max(1.0);
        }

        // Post-warmup decay
        let progress = (step - warmup) / (max - warmup).max(1.0);
        let progress = progress.clamp(0.0, 1.0);

        match self.schedule {
            ScheduleType::Cosine => {
                // Cosine decay to 10% of base_lr
                let cos = (std::f32::consts::PI * progress).cos();
                self.base_lr * (0.1 + 0.9 * (1.0 + cos) / 2.0)
            }
            ScheduleType::Linear => {
                self.base_lr * (1.0 - 0.9 * progress)
            }
            ScheduleType::Constant => self.base_lr,
        }
    }

    pub fn step(&mut self) { self.current_step += 1; }
    pub fn reset(&mut self) { self.current_step = 0; }
}

// ── Loss tracker ──────────────────────────────────────────────────────────────

/// Tracks training loss with windowed averaging.
#[derive(Debug, Clone)]
pub struct LossTracker {
    pub window: usize,
    history: Vec<f32>,
    pub step: usize,
    pub best_loss: f32,
    pub best_step: usize,
}

impl LossTracker {
    pub fn new(window: usize) -> Self {
        Self {
            window,
            history: Vec::new(),
            step: 0,
            best_loss: f32::MAX,
            best_step: 0,
        }
    }

    pub fn record(&mut self, loss: f32) {
        self.history.push(loss);
        if self.history.len() > self.window {
            self.history.remove(0);
        }
        if loss < self.best_loss {
            self.best_loss = loss;
            self.best_step = self.step;
        }
        self.step += 1;
    }

    /// Windowed average loss.
    pub fn avg_loss(&self) -> f32 {
        if self.history.is_empty() { return 0.0; }
        self.history.iter().sum::<f32>() / self.history.len() as f32
    }

    /// Perplexity = exp(avg_loss).
    pub fn perplexity(&self) -> f32 {
        self.avg_loss().exp()
    }

    /// Whether loss is improving (last value < window average).
    pub fn is_improving(&self) -> bool {
        if self.history.len() < 2 { return true; }
        let last = *self.history.last().unwrap();
        let prev_avg = self.history[..self.history.len()-1].iter().sum::<f32>()
            / (self.history.len() - 1) as f32;
        last < prev_avg
    }

    pub fn last_loss(&self) -> Option<f32> { self.history.last().copied() }
    pub fn recorded_steps(&self) -> usize { self.history.len() }
}
