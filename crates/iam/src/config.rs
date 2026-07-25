// Copyright (c) 2026 Edison Lepiten / AIEONYX
// IAM model configuration — IAMSeed 120M

/// IAMSeed model configuration.
/// Matches IAM Founding Spec §10 — 120M params, hybrid SSM/attention.
#[derive(Debug, Clone)]
pub struct IamConfig {
    // ── Vocabulary ───────────────────────────────────────────────────────────
    /// Vocabulary size — must match iam_seed.axvocab (IAM-M2 output)
    pub vocab_size:    usize,
    /// Sequence length
    pub seq_len:       usize,

    // ── Model dimensions ─────────────────────────────────────────────────────
    /// Embedding dimension (d_model)
    pub d_model:       usize,
    /// Feed-forward hidden dimension (4 * d_model)
    pub d_ff:          usize,
    /// Number of attention heads
    pub n_heads:       usize,
    /// Head dimension (d_model / n_heads)
    pub d_head:        usize,

    // ── Layer counts ─────────────────────────────────────────────────────────
    /// SSM layers — efficient long-range sequence modeling
    pub n_ssm_layers:  usize,
    /// Attention layers — precise reasoning + constitution enforcement
    pub n_attn_layers: usize,
    /// Total layers
    pub n_layers:      usize,

    // ── SSM parameters ───────────────────────────────────────────────────────
    /// SSM state dimension
    pub d_state:       usize,
    /// SSM convolution width
    pub d_conv:        usize,
    /// SSM expansion factor
    pub expand:        usize,

    // ── Regularization ───────────────────────────────────────────────────────
    pub dropout:       f32,

    // ── Special token IDs (must match axon_data::tokenizer) ──────────────────
    pub pad_id:        u32,
    pub bos_id:        u32,
    pub eos_id:        u32,
    pub unk_id:        u32,
    pub iam_id:        u32,
    pub intent_id:     u32,
    pub conf_g_id:     u32,
    pub conf_y_id:     u32,
    pub conf_r_id:     u32,
    pub unknown_id:    u32,
}

impl IamConfig {
    /// IAMSeed — 120M parameter configuration.
    /// Architecture derived from IAM Founding Spec §10 + AIEONYX-SPEC-IAM-v1.0.
    pub fn iam_seed() -> Self {
        let d_model = 512;
        let n_heads = 8;
        Self {
            vocab_size:    8_000,
            seq_len:       4_096,
            d_model,
            d_ff:          d_model * 4,      // 2048
            n_heads,
            d_head:        d_model / n_heads, // 64
            n_ssm_layers:  20,
            n_attn_layers: 4,
            n_layers:      24,               // 20 SSM + 4 attn
            d_state:       16,
            d_conv:        4,
            expand:        2,
            dropout:       0.0,              // no dropout during pretraining
            // Special token IDs — locked at IAM-M2
            pad_id:        0,
            bos_id:        1,
            eos_id:        2,
            unk_id:        3,
            iam_id:        4,
            intent_id:     5,
            conf_g_id:     7,
            conf_y_id:     8,
            conf_r_id:     9,
            unknown_id:    10,
        }
    }

    /// Estimate parameter count for this configuration.
    pub fn param_count(&self) -> usize {
        // Token embedding
        let embed = self.vocab_size * self.d_model;

        // SSM layer params (simplified Mamba-style)
        // Each SSM layer: in_proj(2*d_model), conv(d_conv*d_model),
        //   x_proj(d_state*2+1), dt_proj(d_model), out_proj(d_model)
        let ssm_per = self.d_model * (2 * self.d_model)     // in_proj
            + self.d_conv * self.d_model                     // conv
            + self.d_model * (self.d_state * 2 + 1)         // x_proj
            + self.d_model * self.d_model                    // dt_proj
            + self.d_model * self.d_model;                   // out_proj
        let ssm_total = ssm_per * self.n_ssm_layers;

        // Attention layer params
        // Q,K,V projections + output projection + FFN
        let attn_per = 4 * self.d_model * self.d_model      // Q,K,V,O
            + 2 * self.d_model * self.d_ff                  // FFN up+down
            + self.d_ff;                                     // FFN bias
        let attn_total = attn_per * self.n_attn_layers;

        // Layer norms (2 per layer * 2 params each)
        let ln_total = self.n_layers * 2 * self.d_model * 2;

        // Output head
        let lm_head = self.d_model * self.vocab_size;

        embed + ssm_total + attn_total + ln_total + lm_head
    }

    /// Validate configuration consistency.
    pub fn validate(&self) -> Result<(), String> {
        if self.d_model % self.n_heads != 0 {
            return Err(format!("d_model {} not divisible by n_heads {}", self.d_model, self.n_heads));
        }
        if self.n_ssm_layers + self.n_attn_layers != self.n_layers {
            return Err(format!("n_ssm_layers + n_attn_layers != n_layers"));
        }
        if self.vocab_size == 0 {
            return Err("vocab_size cannot be 0".into());
        }
        Ok(())
    }
}
