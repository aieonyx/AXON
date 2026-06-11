// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_learn::optim — Optimizers
// SGD (with optional momentum), Adam
// Operate on DynTensor<f32> parameter + gradient pairs.

use alloc::vec::Vec;
use axon_tensor::DynTensor;
use axon_tensor::ops::TensorOps;

// ------------------------------------------------------------------
// SGD — Stochastic Gradient Descent with optional momentum
// w = w - lr * grad                          (no momentum)
// v = momentum * v + grad                    (with momentum)
// w = w - lr * v
// ------------------------------------------------------------------

/// SGD optimizer state.
pub struct Sgd {
    pub lr:       f32,
    pub momentum: f32,
    /// Velocity buffers — one per parameter group.
    velocities: Vec<Vec<f32>>,
}

impl Sgd {
    /// Create SGD with learning rate and optional momentum (0.0 = no momentum).
    pub fn new(lr: f32, momentum: f32) -> Self {
        Self { lr, momentum, velocities: Vec::new() }
    }

    /// Apply one SGD step to a list of (param, grad) pairs.
    /// Initializes velocity buffers on first call.
    pub fn step(&mut self, params: &mut [(&mut DynTensor<f32>, &DynTensor<f32>)]) {
        // Ensure velocity buffers exist
        while self.velocities.len() < params.len() {
            self.velocities.push(Vec::new());
        }

        for (idx, (param, grad)) in params.iter_mut().enumerate() {
            let n = param.numel();
            // Initialize velocity to zeros if first step
            if self.velocities[idx].len() != n {
                self.velocities[idx] = alloc::vec![0.0f32; n];
            }

            if self.momentum == 0.0 {
                // Pure SGD
                for i in 0..n {
                    let new_val = param.get_flat(i) - self.lr * grad.get_flat(i);
                    param.set_flat(i, new_val);
                }
            } else {
                // SGD with momentum
                for i in 0..n {
                    self.velocities[idx][i] =
                        self.momentum * self.velocities[idx][i] + grad.get_flat(i);
                    let new_val = param.get_flat(i)
                        - self.lr * self.velocities[idx][i];
                    param.set_flat(i, new_val);
                }
            }
        }
    }

    /// Zero all velocity buffers.
    pub fn zero_velocities(&mut self) {
        for v in &mut self.velocities {
            for x in v.iter_mut() { *x = 0.0; }
        }
    }
}

// ------------------------------------------------------------------
// Adam — Adaptive Moment Estimation
// m = β1 * m + (1-β1) * grad         (first moment)
// v = β2 * v + (1-β2) * grad²        (second moment)
// m̂ = m / (1-β1^t)                   (bias correction)
// v̂ = v / (1-β2^t)                   (bias correction)
// w = w - lr * m̂ / (√v̂ + ε)
// ------------------------------------------------------------------

/// Adam optimizer state.
pub struct Adam {
    pub lr:      f32,
    pub beta1:   f32,
    pub beta2:   f32,
    pub epsilon: f32,
    step:        usize,
    m:           Vec<Vec<f32>>,   // first moment
    v:           Vec<Vec<f32>>,   // second moment
}

impl Adam {
    /// Create Adam with standard defaults: lr=0.001, β1=0.9, β2=0.999, ε=1e-8
    pub fn new(lr: f32) -> Self {
        Self {
            lr,
            beta1:   0.9,
            beta2:   0.999,
            epsilon: 1e-8,
            step:    0,
            m:       Vec::new(),
            v:       Vec::new(),
        }
    }

    /// Create Adam with custom hyperparameters.
    pub fn with_params(lr: f32, beta1: f32, beta2: f32, epsilon: f32) -> Self {
        Self { lr, beta1, beta2, epsilon, step: 0, m: Vec::new(), v: Vec::new() }
    }

    /// Apply one Adam step to a list of (param, grad) pairs.
    pub fn step(&mut self, params: &mut [(&mut DynTensor<f32>, &DynTensor<f32>)]) {
        self.step += 1;
        let t = self.step as f32;

        // Ensure moment buffers exist
        while self.m.len() < params.len() {
            self.m.push(Vec::new());
            self.v.push(Vec::new());
        }

        // Bias correction factors
        let bc1 = 1.0 - libm::powf(self.beta1, t);
        let bc2 = 1.0 - libm::powf(self.beta2, t);

        for (idx, (param, grad)) in params.iter_mut().enumerate() {
            let n = param.numel();
            if self.m[idx].len() != n {
                self.m[idx] = alloc::vec![0.0f32; n];
                self.v[idx] = alloc::vec![0.0f32; n];
            }

            for i in 0..n {
                let g = grad.get_flat(i);

                // Update moments
                self.m[idx][i] = self.beta1 * self.m[idx][i] + (1.0 - self.beta1) * g;
                self.v[idx][i] = self.beta2 * self.v[idx][i] + (1.0 - self.beta2) * g * g;

                // Bias-corrected estimates
                let m_hat = self.m[idx][i] / bc1;
                let v_hat = self.v[idx][i] / bc2;

                // Parameter update
                let new_val = param.get_flat(i)
                    - self.lr * m_hat / (libm::sqrtf(v_hat) + self.epsilon);
                param.set_flat(i, new_val);
            }
        }
    }

    /// Reset step counter and moment buffers (e.g. for a new training run).
    pub fn reset(&mut self) {
        self.step = 0;
        for m in &mut self.m { for x in m.iter_mut() { *x = 0.0; } }
        for v in &mut self.v { for x in v.iter_mut() { *x = 0.0; } }
    }

    /// Current step count.
    pub fn current_step(&self) -> usize { self.step }
}
