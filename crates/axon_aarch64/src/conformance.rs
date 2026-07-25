// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_aarch64 P71 — Conformance oracle
//
// Acceptance criterion: axon_interp output == compiled native output.
// This module provides the host-side conformance check:
//   1. Run script through axon_interp (reference interpreter)
//   2. Compile to IR and verify IR structure matches expected behavior
//   3. (Full binary conformance requires qemu-aarch64 — tested separately)

use axon_interp::exec;
use crate::ir::{compile_to_ir, AxIr};

/// Conformance result
#[derive(Debug, Clone, PartialEq)]
pub struct ConformanceResult {
    pub script: Vec<u8>,
    pub interp_output: Vec<u8>,
    pub ir_nodes: usize,
    pub ir_has_program_start: bool,
    pub ir_has_program_end: bool,
    pub interp_error: bool,
    pub compile_error: Option<String>,
}

impl ConformanceResult {
    /// Whether IR and interpreter both succeeded
    pub fn is_conformant(&self) -> bool {
        !self.interp_error
            && self.compile_error.is_none()
            && self.ir_has_program_start
            && self.ir_has_program_end
    }
}

/// Run conformance check on a .ax script.
/// Returns interp output + IR structure for comparison.
pub fn check_conformance(script: &[u8]) -> ConformanceResult {
    // Run interpreter (reference)
    let interp_result = exec(script, 0, None);
    let interp_output = interp_result.output[..interp_result.output_len].to_vec();

    // Compile to IR
    let (ir_nodes, ir_has_start, ir_has_end, compile_error) =
        match compile_to_ir(script) {
            Ok(ir) => {
                let has_start = ir.iter().any(|n| matches!(n, AxIr::ProgramStart));
                let has_end = ir.iter().any(|n| matches!(n, AxIr::ProgramEnd));
                (ir.len(), has_start, has_end, None)
            }
            Err(e) => (0, false, false, Some(e.to_string()))
        };

    ConformanceResult {
        script: script.to_vec(),
        interp_output,
        ir_nodes,
        ir_has_program_start: ir_has_start,
        ir_has_program_end: ir_has_end,
        interp_error: interp_result.error,
        compile_error,
    }
}

/// Verify that a script's IR contains expected instruction types
pub fn ir_contains(script: &[u8], check: impl Fn(&AxIr) -> bool) -> bool {
    compile_to_ir(script)
        .map(|ir| ir.iter().any(check))
        .unwrap_or(false)
}
