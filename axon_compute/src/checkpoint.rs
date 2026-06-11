// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_compute::checkpoint — Model Checkpoint Storage
// Serializes model weights (DynTensor<f32>) to/from flat byte buffers.
// EdisonDB integration: store checkpoint bytes as vector entries.
// Fully testable without EdisonDB — round-trip verified in tests.

use alloc::vec::Vec;
use alloc::string::String;
use axon_tensor::DynTensor;
use axon_tensor::ops::TensorOps;

// ------------------------------------------------------------------
// ModelCheckpoint — a named collection of weight tensors
// ------------------------------------------------------------------

/// A serialized model checkpoint entry: name + flat f32 data + shape.
#[derive(Clone, Debug, PartialEq)]
pub struct WeightEntry {
    pub name:  String,
    pub shape: Vec<usize>,
    pub data:  Vec<f32>,
}

/// A complete model checkpoint — collection of named weight tensors.
#[derive(Clone, Debug, Default)]
pub struct ModelCheckpoint {
    pub entries:    Vec<WeightEntry>,
    pub step:       u64,
    pub loss:       f32,
    pub version:    u32,
}

impl ModelCheckpoint {
    /// Create a new empty checkpoint.
    pub fn new(step: u64, loss: f32) -> Self {
        Self {
            entries: Vec::new(),
            step,
            loss,
            version: 1,
        }
    }

    /// Add a named weight tensor to the checkpoint.
    pub fn add_weight(&mut self, name: &str, tensor: &DynTensor<f32>) {
        self.entries.push(WeightEntry {
            name:  alloc::string::ToString::to_string(name),
            shape: tensor.shape().to_vec(),
            data:  tensor.data().to_vec(),
        });
    }

    /// Get a weight entry by name.
    pub fn get_weight(&self, name: &str) -> Option<&WeightEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Restore a named weight as a DynTensor<f32>.
    pub fn restore_tensor(&self, name: &str) -> Option<DynTensor<f32>> {
        let entry = self.get_weight(name)?;
        Some(DynTensor::from_vec(entry.shape.clone(), entry.data.clone()))
    }

    /// Number of weight entries.
    pub fn weight_count(&self) -> usize { self.entries.len() }

    /// Total number of parameters across all weights.
    pub fn total_params(&self) -> usize {
        self.entries.iter().map(|e| e.data.len()).sum()
    }
}

// ------------------------------------------------------------------
// save_checkpoint — serialize to flat byte buffer
// ------------------------------------------------------------------
// Wire format (little-endian):
// [magic: 4 bytes "ONYX"]
// [version: u32]
// [step: u64]
// [loss: f32]
// [n_entries: u32]
// For each entry:
//   [name_len: u32][name: utf8 bytes]
//   [rank: u32][shape: rank * u64]
//   [numel: u32][data: numel * f32]
// ------------------------------------------------------------------

const MAGIC: &[u8; 4] = b"ONYX";

/// Serialize a ModelCheckpoint to a byte buffer.
pub fn save_checkpoint(ckpt: &ModelCheckpoint) -> Vec<u8> {
    let mut buf = Vec::new();

    // Magic
    buf.extend_from_slice(MAGIC);

    // Version
    buf.extend_from_slice(&ckpt.version.to_le_bytes());

    // Step
    buf.extend_from_slice(&ckpt.step.to_le_bytes());

    // Loss
    buf.extend_from_slice(&ckpt.loss.to_le_bytes());

    // Number of entries
    buf.extend_from_slice(&(ckpt.entries.len() as u32).to_le_bytes());

    for entry in &ckpt.entries {
        // Name
        let name_bytes = entry.name.as_bytes();
        buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(name_bytes);

        // Shape
        buf.extend_from_slice(&(entry.shape.len() as u32).to_le_bytes());
        for &dim in &entry.shape {
            buf.extend_from_slice(&(dim as u64).to_le_bytes());
        }

        // Data
        buf.extend_from_slice(&(entry.data.len() as u32).to_le_bytes());
        for &v in &entry.data {
            buf.extend_from_slice(&v.to_le_bytes());
        }
    }

    buf
}

// ------------------------------------------------------------------
// load_checkpoint — deserialize from byte buffer
// ------------------------------------------------------------------

/// Deserialize a ModelCheckpoint from a byte buffer.
/// Returns Err with a reason string if the buffer is malformed.
pub fn load_checkpoint(buf: &[u8]) -> Result<ModelCheckpoint, &'static str> {
    let mut pos = 0usize;

    // Helper: read n bytes
    macro_rules! read_bytes {
        ($n:expr) => {{
            if pos + $n > buf.len() {
                return Err("checkpoint: unexpected end of buffer");
            }
            let slice = &buf[pos..pos + $n];
            pos += $n;
            slice
        }};
    }

    // Magic
    let magic = read_bytes!(4);
    if magic != MAGIC {
        return Err("checkpoint: invalid magic bytes");
    }

    // Version
    let version = u32::from_le_bytes(read_bytes!(4).try_into()
        .map_err(|_| "checkpoint: version parse error")?);

    // Step
    let step = u64::from_le_bytes(read_bytes!(8).try_into()
        .map_err(|_| "checkpoint: step parse error")?);

    // Loss
    let loss = f32::from_le_bytes(read_bytes!(4).try_into()
        .map_err(|_| "checkpoint: loss parse error")?);

    // Number of entries
    let n_entries = u32::from_le_bytes(read_bytes!(4).try_into()
        .map_err(|_| "checkpoint: n_entries parse error")?) as usize;

    let mut entries = Vec::with_capacity(n_entries);

    for _ in 0..n_entries {
        // Name
        let name_len = u32::from_le_bytes(read_bytes!(4).try_into()
            .map_err(|_| "checkpoint: name_len parse error")?) as usize;
        let name_bytes = read_bytes!(name_len);
        let name = alloc::str::from_utf8(name_bytes)
            .map_err(|_| "checkpoint: name utf8 error")?;

        // Shape
        let rank = u32::from_le_bytes(read_bytes!(4).try_into()
            .map_err(|_| "checkpoint: rank parse error")?) as usize;
        let mut shape = Vec::with_capacity(rank);
        for _ in 0..rank {
            let dim = u64::from_le_bytes(read_bytes!(8).try_into()
                .map_err(|_| "checkpoint: dim parse error")?) as usize;
            shape.push(dim);
        }

        // Data
        let numel = u32::from_le_bytes(read_bytes!(4).try_into()
            .map_err(|_| "checkpoint: numel parse error")?) as usize;
        let mut data = Vec::with_capacity(numel);
        for _ in 0..numel {
            let v = f32::from_le_bytes(read_bytes!(4).try_into()
                .map_err(|_| "checkpoint: f32 parse error")?);
            data.push(v);
        }

        entries.push(WeightEntry {
            name: alloc::string::ToString::to_string(name),
            shape,
            data,
        });
    }

    Ok(ModelCheckpoint { entries, step, loss, version })
}
