// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// WASM value types and function types.
// Clean-room: studied WebAssembly Core Spec 2.0 only. No code copied.
use crate::error::{WasmError, WasmResult};

#[derive(Debug, Clone, PartialEq)]
pub enum ValType {
    I32,
    I64,
    F32,
    F64,
}

impl ValType {
    pub fn from_byte(b: u8) -> WasmResult<Self> {
        match b {
            0x7f => Ok(ValType::I32),
            0x7e => Ok(ValType::I64),
            0x7d => Ok(ValType::F32),
            0x7c => Ok(ValType::F64),
            _    => Err(WasmError::InvalidType(b)),
        }
    }
    pub fn to_byte(&self) -> u8 {
        match self {
            ValType::I32 => 0x7f,
            ValType::I64 => 0x7e,
            ValType::F32 => 0x7d,
            ValType::F64 => 0x7c,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            ValType::I32 => "i32",
            ValType::I64 => "i64",
            ValType::F32 => "f32",
            ValType::F64 => "f64",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncType {
    pub params:  Vec<ValType>,
    pub results: Vec<ValType>,
}

impl FuncType {
    pub fn new(params: Vec<ValType>, results: Vec<ValType>) -> Self {
        FuncType { params, results }
    }
    pub fn void() -> Self { FuncType { params: vec![], results: vec![] } }
    pub fn unary(param: ValType, result: ValType) -> Self {
        FuncType { params: vec![param], results: vec![result] }
    }
    pub fn binary(a: ValType, b: ValType, result: ValType) -> Self {
        FuncType { params: vec![a, b], results: vec![result] }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl WasmValue {
    pub fn val_type(&self) -> ValType {
        match self {
            WasmValue::I32(_) => ValType::I32,
            WasmValue::I64(_) => ValType::I64,
            WasmValue::F32(_) => ValType::F32,
            WasmValue::F64(_) => ValType::F64,
        }
    }
    pub fn as_i32(&self) -> WasmResult<i32> {
        match self {
            WasmValue::I32(v) => Ok(*v),
            _ => Err(WasmError::TypeMismatch {
                expected: "i32".into(), got: self.val_type().name().into()
            }),
        }
    }
    pub fn as_i64(&self) -> WasmResult<i64> {
        match self {
            WasmValue::I64(v) => Ok(*v),
            _ => Err(WasmError::TypeMismatch {
                expected: "i64".into(), got: self.val_type().name().into()
            }),
        }
    }
    pub fn as_f32(&self) -> WasmResult<f32> {
        match self {
            WasmValue::F32(v) => Ok(*v),
            _ => Err(WasmError::TypeMismatch {
                expected: "f32".into(), got: self.val_type().name().into()
            }),
        }
    }
    pub fn as_f64(&self) -> WasmResult<f64> {
        match self {
            WasmValue::F64(v) => Ok(*v),
            _ => Err(WasmError::TypeMismatch {
                expected: "f64".into(), got: self.val_type().name().into()
            }),
        }
    }
    pub fn default_for(t: &ValType) -> Self {
        match t {
            ValType::I32 => WasmValue::I32(0),
            ValType::I64 => WasmValue::I64(0),
            ValType::F32 => WasmValue::F32(0.0),
            ValType::F64 => WasmValue::F64(0.0),
        }
    }
}
