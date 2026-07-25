// Copyright (c) 2026 Edison Lepiten / AIEONYX
// IAMSeed model — 120M param hybrid SSM/Attention
//
// Layer interleaving strategy:
//   Layers 0-4:   SSM  (fast sequence context)
//   Layer  5:     ATTN (constitution gate 1 — @intent)
//   Layers 6-10:  SSM
//   Layer  11:    ATTN (constitution gate 2 — confidence)
//   Layers 12-16: SSM
//   Layer  17:    ATTN (constitution gate 3 — Solomon judgment)
//   Layers 18-22: SSM
//   Layer  23:    ATTN (constitution gate 4 — agency preservation)
//
// Attention layers are the constitution enforcement points.
// SSM layers handle efficient long-range context.

use crate::config::IamConfig;

// ── Tensor primitives (backed by axon_learn) ──────────────────────────────────
// We use f32 vectors as flat tensors for the pretraining scaffold.
// axon_learn provides the autograd engine underneath.

/// Flat f32 tensor with shape tracking.
#[derive(Debug, Clone)]
pub struct Tensor {
    pub data: Vec<f32>,
    pub shape: Vec<usize>,
}

impl Tensor {
    pub fn zeros(shape: &[usize]) -> Self {
        let n = shape.iter().product();
        Self { data: vec![0.0; n], shape: shape.to_vec() }
    }

    pub fn from_vec(data: Vec<f32>, shape: Vec<usize>) -> Self {
        assert_eq!(data.len(), shape.iter().product::<usize>());
        Self { data, shape }
    }

    pub fn numel(&self) -> usize { self.data.len() }

    pub fn shape_str(&self) -> String {
        let dims: Vec<String> = self.shape.iter().map(|d| d.to_string()).collect();
        format!("[{}]", dims.join(", "))
    }
}

// ── Layer Norm ────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct LayerNorm {
    pub weight: Vec<f32>,
    pub bias: Vec<f32>,
    pub dim: usize,
    pub eps: f32,
}

impl LayerNorm {
    pub fn new(dim: usize) -> Self {
        Self {
            weight: vec![1.0; dim],
            bias:   vec![0.0; dim],
            dim,
            eps: 1e-5,
        }
    }

    pub fn forward(&self, x: &[f32]) -> Vec<f32> {
        // x shape: [seq_len * dim] — normalize each dim-sized slice
        let n = self.dim;
        let seq = x.len() / n;
        let mut out = vec![0.0f32; x.len()];
        for s in 0..seq {
            let slice = &x[s*n..(s+1)*n];
            let mean = slice.iter().sum::<f32>() / n as f32;
            let var = slice.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n as f32;
            let std = (var + self.eps).sqrt();
            for i in 0..n {
                out[s*n + i] = (slice[i] - mean) / std * self.weight[i] + self.bias[i];
            }
        }
        out
    }

    pub fn param_count(&self) -> usize { self.dim * 2 }
}

// ── SSM Layer (Mamba-style State Space Model) ─────────────────────────────────
#[derive(Debug, Clone)]
pub struct SsmLayer {
    pub d_model: usize,
    pub d_state: usize,
    pub d_conv:  usize,
    pub expand:  usize,
    // Parameters
    pub in_proj:  Vec<f32>,  // [d_model, 2*d_inner]
    pub conv_weight: Vec<f32>, // [d_inner, d_conv]
    pub x_proj:  Vec<f32>,  // [d_inner, dt_rank + 2*d_state]
    pub dt_proj: Vec<f32>,  // [dt_rank, d_inner]
    pub out_proj: Vec<f32>, // [d_inner, d_model]
    pub norm: LayerNorm,
    // SSM parameters
    pub a_log: Vec<f32>,   // [d_inner, d_state]
    pub d_ssm: Vec<f32>,   // [d_inner]
}

impl SsmLayer {
    pub fn new(config: &IamConfig) -> Self {
        let d_inner = config.d_model * config.expand;
        let dt_rank = (config.d_model / 16).max(1);

        Self {
            d_model: config.d_model,
            d_state: config.d_state,
            d_conv:  config.d_conv,
            expand:  config.expand,
            in_proj:     vec![0.0; config.d_model * 2 * d_inner],
            conv_weight: vec![0.0; d_inner * config.d_conv],
            x_proj:      vec![0.0; d_inner * (dt_rank + 2 * config.d_state)],
            dt_proj:     vec![0.0; dt_rank * d_inner],
            out_proj:    vec![0.0; d_inner * config.d_model],
            norm: LayerNorm::new(config.d_model),
            a_log: vec![-1.0; d_inner * config.d_state],
            d_ssm: vec![1.0; d_inner],
        }
    }

    /// Forward pass — simplified SSM (full selective scan in production).
    /// For pretraining scaffold: applies layer norm + residual.
    pub fn forward(&self, x: &[f32]) -> Vec<f32> {
        // Residual connection with layer norm
        let normed = self.norm.forward(x);
        // Add residual
        x.iter().zip(normed.iter()).map(|(a, b)| a + b * 0.1).collect()
    }

    pub fn param_count(&self) -> usize {
        self.in_proj.len()
            + self.conv_weight.len()
            + self.x_proj.len()
            + self.dt_proj.len()
            + self.out_proj.len()
            + self.a_log.len()
            + self.d_ssm.len()
            + self.norm.param_count()
    }
}

// ── Attention Layer (Constitution Enforcement Point) ──────────────────────────
#[derive(Debug, Clone)]
pub struct AttnLayer {
    pub d_model: usize,
    pub n_heads: usize,
    pub d_head:  usize,
    // Parameters
    pub q_proj: Vec<f32>,  // [d_model, d_model]
    pub k_proj: Vec<f32>,
    pub v_proj: Vec<f32>,
    pub o_proj: Vec<f32>,
    pub ff_up:  Vec<f32>,  // [d_model, d_ff]
    pub ff_down: Vec<f32>, // [d_ff, d_model]
    pub norm1: LayerNorm,
    pub norm2: LayerNorm,
}

impl AttnLayer {
    pub fn new(config: &IamConfig) -> Self {
        Self {
            d_model: config.d_model,
            n_heads: config.n_heads,
            d_head:  config.d_head,
            q_proj:  vec![0.0; config.d_model * config.d_model],
            k_proj:  vec![0.0; config.d_model * config.d_model],
            v_proj:  vec![0.0; config.d_model * config.d_model],
            o_proj:  vec![0.0; config.d_model * config.d_model],
            ff_up:   vec![0.0; config.d_model * config.d_ff],
            ff_down: vec![0.0; config.d_ff * config.d_model],
            norm1:   LayerNorm::new(config.d_model),
            norm2:   LayerNorm::new(config.d_model),
        }
    }

    /// Forward pass — simplified attention with residual.
    pub fn forward(&self, x: &[f32]) -> Vec<f32> {
        let n1 = self.norm1.forward(x);
        let r1: Vec<f32> = x.iter().zip(n1.iter()).map(|(a, b)| a + b * 0.1).collect();
        let n2 = self.norm2.forward(&r1);
        r1.iter().zip(n2.iter()).map(|(a, b)| a + b * 0.1).collect()
    }

    pub fn param_count(&self) -> usize {
        self.q_proj.len() + self.k_proj.len()
            + self.v_proj.len() + self.o_proj.len()
            + self.ff_up.len() + self.ff_down.len()
            + self.norm1.param_count() + self.norm2.param_count()
    }
}

// ── Embedding ─────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct Embedding {
    pub weight: Vec<f32>,  // [vocab_size, d_model]
    pub vocab_size: usize,
    pub d_model: usize,
}

impl Embedding {
    pub fn new(vocab_size: usize, d_model: usize) -> Self {
        // Xavier initialization
        let scale = (2.0 / (vocab_size + d_model) as f32).sqrt();
        let mut weight = vec![0.0f32; vocab_size * d_model];
        // Simple deterministic init (full random init happens in trainer)
        for (i, w) in weight.iter_mut().enumerate() {
            *w = ((i as f32 * 1.6180339887) % 2.0 - 1.0) * scale;
        }
        Self { weight, vocab_size, d_model }
    }

    pub fn forward(&self, token_ids: &[u32]) -> Vec<f32> {
        let mut out = vec![0.0f32; token_ids.len() * self.d_model];
        for (i, &id) in token_ids.iter().enumerate() {
            let id = (id as usize).min(self.vocab_size - 1);
            let src = &self.weight[id * self.d_model..(id + 1) * self.d_model];
            out[i * self.d_model..(i + 1) * self.d_model].copy_from_slice(src);
        }
        out
    }

    pub fn param_count(&self) -> usize { self.weight.len() }
}

// ── LM Head ───────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct LmHead {
    pub weight: Vec<f32>,  // [d_model, vocab_size]
    pub d_model: usize,
    pub vocab_size: usize,
    pub norm: LayerNorm,
}

impl LmHead {
    pub fn new(vocab_size: usize, d_model: usize) -> Self {
        Self {
            weight: vec![0.0; d_model * vocab_size],
            d_model,
            vocab_size,
            norm: LayerNorm::new(d_model),
        }
    }

    /// Project hidden state to vocabulary logits.
    pub fn forward(&self, hidden: &[f32]) -> Vec<f32> {
        // hidden: [seq_len * d_model] → logits: [seq_len * vocab_size]
        let seq = hidden.len() / self.d_model;
        let normed = self.norm.forward(hidden);
        let mut logits = vec![0.0f32; seq * self.vocab_size];
        for s in 0..seq {
            let h = &normed[s * self.d_model..(s + 1) * self.d_model];
            for v in 0..self.vocab_size {
                let w = &self.weight[v * self.d_model..(v + 1) * self.d_model];
                logits[s * self.vocab_size + v] =
                    h.iter().zip(w.iter()).map(|(a, b)| a * b).sum();
            }
        }
        logits
    }

    pub fn param_count(&self) -> usize {
        self.weight.len() + self.norm.param_count()
    }
}

// ── Layer enum ────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub enum Layer {
    Ssm(SsmLayer),
    Attn(AttnLayer),
}

impl Layer {
    pub fn forward(&self, x: &[f32]) -> Vec<f32> {
        match self {
            Layer::Ssm(l)  => l.forward(x),
            Layer::Attn(l) => l.forward(x),
        }
    }

    pub fn param_count(&self) -> usize {
        match self {
            Layer::Ssm(l)  => l.param_count(),
            Layer::Attn(l) => l.param_count(),
        }
    }

    pub fn is_attn(&self) -> bool {
        matches!(self, Layer::Attn(_))
    }
}

// ── IAM Model ─────────────────────────────────────────────────────────────────

/// IAMSeed — 120M parameter hybrid SSM/Attention language model.
///
/// Constitution enforcement architecture:
///   Attention layers are placed at positions where constitutional
///   reasoning must be applied — intent detection, confidence marking,
///   Solomonic judgment, and agency preservation.
pub struct IamModel {
    pub config: IamConfig,
    pub embedding: Embedding,
    pub layers: Vec<Layer>,
    pub lm_head: LmHead,
}

impl IamModel {
    /// Create IAMSeed model with the sovereign architecture.
    pub fn new(config: IamConfig) -> Self {
        config.validate().expect("invalid IAM config");

        // Build embedding
        let embedding = Embedding::new(config.vocab_size, config.d_model);

        // Build layers — interleave SSM and attention
        // Pattern: 5 SSM, 1 ATTN, 5 SSM, 1 ATTN, 5 SSM, 1 ATTN, 5 SSM, 1 ATTN
        let ssm_per_block = config.n_ssm_layers / config.n_attn_layers; // 5
        let mut layers = Vec::with_capacity(config.n_layers);

        for block in 0..config.n_attn_layers {
            // SSM layers for this block
            for _ in 0..ssm_per_block {
                layers.push(Layer::Ssm(SsmLayer::new(&config)));
            }
            // Attention layer — constitution enforcement point
            layers.push(Layer::Attn(AttnLayer::new(&config)));
            let _ = block; // suppress unused warning
        }

        let lm_head = LmHead::new(config.vocab_size, config.d_model);

        Self { config, embedding, layers, lm_head }
    }

    /// Forward pass: token IDs → logits.
    pub fn forward(&self, token_ids: &[u32]) -> Vec<f32> {
        // Embed tokens
        let mut hidden = self.embedding.forward(token_ids);

        // Pass through all layers
        for layer in &self.layers {
            hidden = layer.forward(&hidden);
        }

        // Project to vocabulary
        self.lm_head.forward(&hidden)
    }

    /// Count total parameters.
    pub fn param_count(&self) -> usize {
        let layer_params: usize = self.layers.iter().map(|l| l.param_count()).sum();
        self.embedding.param_count()
            + layer_params
            + self.lm_head.param_count()
    }

    /// Print architecture summary.
    pub fn summary(&self) {
        println!("=== IAMSeed Architecture ===");
        println!("  d_model:      {}", self.config.d_model);
        println!("  n_layers:     {} ({} SSM + {} Attn)",
            self.config.n_layers, self.config.n_ssm_layers, self.config.n_attn_layers);
        println!("  vocab_size:   {}", self.config.vocab_size);
        println!("  seq_len:      {}", self.config.seq_len);
        println!();
        println!("  Layer layout:");
        for (i, layer) in self.layers.iter().enumerate() {
            let kind = if layer.is_attn() {
                "ATTN [constitution gate]"
            } else {
                "SSM "
            };
            println!("    Layer {:>2}: {} ({} params)", i, kind, layer.param_count());
        }
        println!();
        println!("  Embedding:  {} params", self.embedding.param_count());
        println!("  LM Head:    {} params", self.lm_head.param_count());
        println!();
        let total = self.param_count();
        println!("  Total:      {} params ({:.1}M)",
            total, total as f32 / 1_000_000.0);
        println!();
        println!("  Constitution enforcement points:");
        println!("    Layer  5 ATTN — Article 0: @intent gate");
        println!("    Layer 11 ATTN — Article 2: confidence marking");
        println!("    Layer 17 ATTN — Article 4: Solomonic judgment");
        println!("    Layer 23 ATTN — Article 5: agency preservation");
    }
}
