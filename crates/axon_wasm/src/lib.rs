// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_wasm -- ECHO sovereign WebAssembly runtime.
// P61.0: binary parser, validator, MVP stack-machine interpreter.
// P61.1: JIT compilation via axon_gpu compute shaders.
pub mod error;
pub mod module;
pub mod runtime;
pub mod types;
pub mod validator;
pub use error::{WasmError, WasmResult};
pub use module::{WasmModule, Export, FuncBody, WASM_MAGIC, WASM_VERSION};
pub use runtime::WasmRuntime;
pub use types::{ValType, FuncType, WasmValue};
pub use validator::WasmValidator;

pub mod jit;
