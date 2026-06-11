// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_learn::tape — Wengert Tape Autodiff Engine
// Forward pass records ops; backward pass computes gradients.
// Operates on scalar f32 values. DynTensor paths use element-wise tape.

use alloc::vec::Vec;

/// Unique identifier for a variable on the tape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VarId(pub usize);

/// A recorded operation on the tape.
/// Stores the local gradients (partial derivatives) w.r.t. each input.
#[derive(Clone, Debug)]
struct TapeEntry {
    /// (input_var_id, local_gradient) pairs
    inputs: Vec<(usize, f32)>,
}

/// Wengert tape — records the forward computation graph.
#[derive(Debug, Default)]
pub struct Tape {
    entries: Vec<TapeEntry>,
    values:  Vec<f32>,
}

impl Tape {
    /// Create a new empty tape.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            values:  Vec::new(),
        }
    }

    /// Push a leaf variable (no inputs, e.g. a weight or input datum).
    pub fn leaf(&mut self, value: f32) -> Var {
        let id = self.entries.len();
        self.entries.push(TapeEntry { inputs: Vec::new() });
        self.values.push(value);
        Var { id: VarId(id), value }
    }

    /// Record a new variable as a function of existing variables.
    /// `inputs` = &[(parent_id, ∂result/∂parent)]
    pub fn push(&mut self, value: f32, inputs: Vec<(usize, f32)>) -> Var {
        let id = self.entries.len();
        self.entries.push(TapeEntry { inputs });
        self.values.push(value);
        Var { id: VarId(id), value }
    }

    /// Run reverse-mode autodiff from `root_id`.
    /// Returns a Vec of gradients indexed by VarId.
    pub fn backward(&self, root_id: VarId) -> Vec<f32> {
        let n = self.entries.len();
        let mut grads = alloc::vec![0.0f32; n];
        grads[root_id.0] = 1.0;

        // Traverse in reverse order (topological order guaranteed by push order)
        for i in (0..n).rev() {
            let g = grads[i];
            if g == 0.0 { continue; }
            for &(parent_id, local_grad) in &self.entries[i].inputs {
                grads[parent_id] += g * local_grad;
            }
        }
        grads
    }

    /// Number of variables recorded on the tape.
    pub fn len(&self) -> usize { self.entries.len() }

    /// True if tape has no entries.
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

/// A variable on the tape — carries its scalar value and tape position.
#[derive(Clone, Copy, Debug)]
pub struct Var {
    pub id:    VarId,
    pub value: f32,
}

// ------------------------------------------------------------------
// Differentiable operations on Var
// ------------------------------------------------------------------

impl Var {
    /// Add two variables: z = a + b, ∂z/∂a = 1, ∂z/∂b = 1
    pub fn add(self, other: Self, tape: &mut Tape) -> Self {
        tape.push(
            self.value + other.value,
            alloc::vec![(self.id.0, 1.0), (other.id.0, 1.0)],
        )
    }

    /// Subtract: z = a - b, ∂z/∂a = 1, ∂z/∂b = -1
    pub fn sub(self, other: Self, tape: &mut Tape) -> Self {
        tape.push(
            self.value - other.value,
            alloc::vec![(self.id.0, 1.0), (other.id.0, -1.0)],
        )
    }

    /// Multiply: z = a * b, ∂z/∂a = b, ∂z/∂b = a
    pub fn mul(self, other: Self, tape: &mut Tape) -> Self {
        tape.push(
            self.value * other.value,
            alloc::vec![(self.id.0, other.value), (other.id.0, self.value)],
        )
    }

    /// Square: z = a², ∂z/∂a = 2a
    pub fn square(self, tape: &mut Tape) -> Self {
        tape.push(
            self.value * self.value,
            alloc::vec![(self.id.0, 2.0 * self.value)],
        )
    }

    /// ReLU: z = max(0, a), ∂z/∂a = 1 if a > 0 else 0
    pub fn relu(self, tape: &mut Tape) -> Self {
        let val = if self.value > 0.0 { self.value } else { 0.0 };
        let grad = if self.value > 0.0 { 1.0 } else { 0.0 };
        tape.push(val, alloc::vec![(self.id.0, grad)])
    }

    /// Sigmoid: z = 1/(1+e^-a), ∂z/∂a = z*(1-z)
    pub fn sigmoid(self, tape: &mut Tape) -> Self {
        let val = 1.0 / (1.0 + libm::expf(-self.value));
        let grad = val * (1.0 - val);
        tape.push(val, alloc::vec![(self.id.0, grad)])
    }

    /// Natural log: z = ln(a), ∂z/∂a = 1/a
    /// Clamps input to avoid log(0).
    pub fn ln(self, tape: &mut Tape) -> Self {
        let clamped = self.value.max(1e-7);
        let val = libm::logf(clamped);
        let grad = 1.0 / clamped;
        tape.push(val, alloc::vec![(self.id.0, grad)])
    }

    /// Negate: z = -a, ∂z/∂a = -1
    pub fn neg(self, tape: &mut Tape) -> Self {
        tape.push(-self.value, alloc::vec![(self.id.0, -1.0)])
    }

    /// Scale by constant: z = c * a, ∂z/∂a = c
    pub fn scale(self, c: f32, tape: &mut Tape) -> Self {
        tape.push(self.value * c, alloc::vec![(self.id.0, c)])
    }
}
