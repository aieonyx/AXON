// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_wasm error types.
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum WasmError {
    InvalidMagic,
    InvalidVersion,
    InvalidSection(u8),
    InvalidType(u8),
    InvalidInstruction(u8),
    StackUnderflow,
    StackOverflow,
    TypeMismatch { expected: String, got: String },
    UndefinedFunction(u32),
    UndefinedMemory,
    MemoryOutOfBounds { addr: u32, size: u32 },
    TrapUnreachable,
    ValidationFailed(String),
    ParseError(String),
}

impl fmt::Display for WasmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WasmError::InvalidMagic              => write!(f, "invalid WASM magic bytes"),
            WasmError::InvalidVersion            => write!(f, "invalid WASM version"),
            WasmError::InvalidSection(s)         => write!(f, "invalid section id: {}", s),
            WasmError::InvalidType(t)            => write!(f, "invalid type: 0x{:02x}", t),
            WasmError::InvalidInstruction(i)     => write!(f, "invalid instruction: 0x{:02x}", i),
            WasmError::StackUnderflow            => write!(f, "stack underflow"),
            WasmError::StackOverflow             => write!(f, "stack overflow"),
            WasmError::TypeMismatch { expected, got } =>
                write!(f, "type mismatch: expected {}, got {}", expected, got),
            WasmError::UndefinedFunction(i)      => write!(f, "undefined function: {}", i),
            WasmError::UndefinedMemory           => write!(f, "undefined memory"),
            WasmError::MemoryOutOfBounds { addr, size } =>
                write!(f, "memory out of bounds: addr={} size={}", addr, size),
            WasmError::TrapUnreachable           => write!(f, "unreachable trap"),
            WasmError::ValidationFailed(s)       => write!(f, "validation failed: {}", s),
            WasmError::ParseError(s)             => write!(f, "parse error: {}", s),
        }
    }
}

pub type WasmResult<T> = Result<T, WasmError>;
