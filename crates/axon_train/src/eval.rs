// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_train P68-M4 — Evaluation harness
//
// Runs every N steps. Computes:
//   - Validation loss (cross-entropy on held-out batch)
//   - Perplexity
//   - Constitutional compliance score (Articles 0-5 fire rate)
//   - Knowledge boundary hit rate (how often IAM says "I don't know")

use serde::{Deserialize, Serialize};

/// Evaluation result for one eval run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    pub step: usize,
    pub val_loss: f32,
    pub perplexity: f32,
    /// How often Article 2 fired (honest uncertainty)
    pub honesty_rate: f32,
    /// How often constitutional articles were invoked
    pub constitution_compliance: f32,
    /// Graduation suite scores (per benchmark)
    pub benchmark_scores: Vec<BenchmarkScore>,
    /// Whether this is the best eval so far
    pub is_best: bool,
}

impl EvalResult {
    pub fn new(step: usize, val_loss: f32) -> Self {
        Self {
            step,
            val_loss,
            perplexity: val_loss.exp(),
            honesty_rate: 0.0,
            constitution_compliance: 0.0,
            benchmark_scores: vec![],
            is_best: false,
        }
    }

    pub fn with_honesty(mut self, rate: f32) -> Self {
        self.honesty_rate = rate;
        self
    }

    pub fn with_compliance(mut self, rate: f32) -> Self {
        self.constitution_compliance = rate;
        self
    }

    pub fn add_benchmark(mut self, score: BenchmarkScore) -> Self {
        self.benchmark_scores.push(score);
        self
    }

    pub fn mark_best(mut self) -> Self {
        self.is_best = true;
        self
    }
}

/// Score on a named benchmark suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkScore {
    pub name: String,
    pub score: f32,
    pub max_score: f32,
    pub passed: bool,
}

impl BenchmarkScore {
    pub fn new(name: &str, score: f32, max_score: f32, threshold: f32) -> Self {
        Self {
            name: name.to_string(),
            score,
            max_score,
            passed: score >= threshold,
        }
    }

    pub fn pct(&self) -> f32 {
        if self.max_score == 0.0 { return 0.0; }
        self.score / self.max_score * 100.0
    }
}

/// Evaluation harness — tracks eval history and best results.
pub struct EvalHarness {
    pub history: Vec<EvalResult>,
    pub best_val_loss: f32,
    pub best_step: usize,
}

impl EvalHarness {
    pub fn new() -> Self {
        Self { history: vec![], best_val_loss: f32::MAX, best_step: 0 }
    }

    /// Record an eval result. Returns true if this is the best so far.
    pub fn record(&mut self, mut result: EvalResult) -> bool {
        let is_best = result.val_loss < self.best_val_loss;
        if is_best {
            self.best_val_loss = result.val_loss;
            self.best_step = result.step;
            result = result.mark_best();
        }
        self.history.push(result);
        is_best
    }

    /// Compute simple validation loss from a set of per-token losses.
    pub fn compute_val_loss(token_losses: &[f32]) -> f32 {
        if token_losses.is_empty() { return 0.0; }
        token_losses.iter().sum::<f32>() / token_losses.len() as f32
    }

    pub fn eval_count(&self) -> usize { self.history.len() }
    pub fn last_result(&self) -> Option<&EvalResult> { self.history.last() }

    /// Whether validation loss is trending down.
    pub fn is_converging(&self) -> bool {
        if self.history.len() < 3 { return true; }
        let n = self.history.len();
        let recent = self.history[n-3..].iter().map(|r| r.val_loss).collect::<Vec<_>>();
        recent[2] < recent[0]
    }
}

impl Default for EvalHarness { fn default() -> Self { Self::new() } }
