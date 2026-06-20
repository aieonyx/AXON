// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// WASM module validator -- sovereign implementation.
// Clean-room: studied WebAssembly Core Spec 2.0 validation rules only.
// P61.0: validates type section, function indices, export references.
use crate::module::WasmModule;
use crate::error::{WasmError, WasmResult};

pub const MAX_FUNCTIONS: usize  = 10_000;
pub const MAX_TYPES:     usize  = 1_000;
pub const MAX_EXPORTS:   usize  = 1_000;
pub const MAX_STACK_DEPTH: usize = 1_024;

pub struct WasmValidator;

impl WasmValidator {
    pub fn validate(module: &WasmModule) -> WasmResult<()> {
        Self::validate_limits(module)?;
        Self::validate_function_types(module)?;
        Self::validate_exports(module)?;
        Ok(())
    }

    fn validate_limits(module: &WasmModule) -> WasmResult<()> {
        if module.types.len() > MAX_TYPES {
            return Err(WasmError::ValidationFailed(
                format!("too many types: {}", module.types.len())
            ));
        }
        if module.func_types.len() > MAX_FUNCTIONS {
            return Err(WasmError::ValidationFailed(
                format!("too many functions: {}", module.func_types.len())
            ));
        }
        if module.exports.len() > MAX_EXPORTS {
            return Err(WasmError::ValidationFailed(
                format!("too many exports: {}", module.exports.len())
            ));
        }
        Ok(())
    }

    fn validate_function_types(module: &WasmModule) -> WasmResult<()> {
        for &type_idx in &module.func_types {
            if type_idx as usize >= module.types.len() {
                return Err(WasmError::ValidationFailed(
                    format!("function references undefined type: {}", type_idx)
                ));
            }
        }
        if module.bodies.len() != module.func_types.len() {
            return Err(WasmError::ValidationFailed(format!(
                "function count mismatch: {} types vs {} bodies",
                module.func_types.len(), module.bodies.len()
            )));
        }
        Ok(())
    }

    fn validate_exports(module: &WasmModule) -> WasmResult<()> {
        for export in &module.exports {
            // kind 0 = function export
            if export.kind == 0 {
                if export.index as usize >= module.func_types.len() {
                    return Err(WasmError::ValidationFailed(
                        format!("export '{}' references undefined function: {}",
                            export.name, export.index)
                    ));
                }
            }
        }
        Ok(())
    }
}
