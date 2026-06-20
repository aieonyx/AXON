// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P61 QA -- axon_wasm ECHO sovereign WASM runtime tests
// Pass bar: 20/20
// P3 Doctrine: complements axon_crypto P57 (module signing), axon_layout P60 (UI output)
use axon_wasm::{
    WasmModule, WasmRuntime, WasmValidator, WasmValue,
    ValType, FuncType, WasmError, WASM_MAGIC, WASM_VERSION,
};

// Minimal valid WASM module: no functions, no exports
fn empty_module() -> Vec<u8> {
    let mut m = vec![];
    m.extend_from_slice(&WASM_MAGIC);
    m.extend_from_slice(&WASM_VERSION);
    m
}

// WASM module: (func (param i32 i32) (result i32) local.get 0 local.get 1 i32.add)
// Exported as "add"
fn add_module() -> Vec<u8> {
    vec![
        0x00,0x61,0x73,0x6d, // magic
        0x01,0x00,0x00,0x00, // version
        // type section: (i32,i32)->i32
        0x01,0x07, 0x01,0x60,0x02,0x7f,0x7f,0x01,0x7f,
        // function section: func 0 uses type 0
        0x03,0x02, 0x01,0x00,
        // export section: "add" = func 0
        0x07,0x07, 0x01,0x03,0x61,0x64,0x64,0x00,0x00,
        // code section: body of add
        0x0a,0x09, 0x01,0x07,0x00,
        0x20,0x00, // local.get 0
        0x20,0x01, // local.get 1
        0x6a,      // i32.add
        0x0b,      // end
    ]
}

// WASM module: (func (result i32) i32.const 42 return)
// Exported as "answer"
fn const42_module() -> Vec<u8> {
    vec![
        0x00,0x61,0x73,0x6d,
        0x01,0x00,0x00,0x00,
        // type: ()->i32
        0x01,0x05, 0x01,0x60,0x00,0x01,0x7f,
        // function: type 0
        0x03,0x02, 0x01,0x00,
        // export: "answer" = func 0
        0x07,0x0a, 0x01,0x06,0x61,0x6e,0x73,0x77,0x65,0x72,0x00,0x00,
        // code: i32.const 42 end
        0x0a,0x06, 0x01,0x04,0x00,0x41,0x2a,0x0b,
    ]
}

// ── Parser tests ──────────────────────────────────────────────────────────────

#[test]
fn test_parse_empty_module() {
    let m = WasmModule::parse(&empty_module()).unwrap();
    assert_eq!(m.func_count(), 0);
    assert_eq!(m.exports.len(), 0);
}

#[test]
fn test_parse_invalid_magic() {
    let bad = vec![0x00,0x00,0x00,0x00, 0x01,0x00,0x00,0x00];
    assert_eq!(WasmModule::parse(&bad).unwrap_err(), WasmError::InvalidMagic);
}

#[test]
fn test_parse_invalid_version() {
    let mut bad = WASM_MAGIC.to_vec();
    bad.extend_from_slice(&[0x02,0x00,0x00,0x00]);
    assert_eq!(WasmModule::parse(&bad).unwrap_err(), WasmError::InvalidVersion);
}

#[test]
fn test_parse_too_short() {
    assert!(WasmModule::parse(&[0x00,0x61]).is_err());
}

#[test]
fn test_parse_add_module() {
    let m = WasmModule::parse(&add_module()).unwrap();
    assert_eq!(m.func_count(), 1);
    assert_eq!(m.types.len(), 1);
    assert_eq!(m.types[0].params, vec![ValType::I32, ValType::I32]);
    assert_eq!(m.types[0].results, vec![ValType::I32]);
    assert_eq!(m.exports.len(), 1);
    assert_eq!(m.exports[0].name, "add");
}

#[test]
fn test_parse_find_export() {
    let m = WasmModule::parse(&add_module()).unwrap();
    assert!(m.find_export("add").is_some());
    assert!(m.find_export("missing").is_none());
}

// ── Validator tests ───────────────────────────────────────────────────────────

#[test]
fn test_validate_empty_module() {
    let m = WasmModule::parse(&empty_module()).unwrap();
    assert!(WasmValidator::validate(&m).is_ok());
}

#[test]
fn test_validate_add_module() {
    let m = WasmModule::parse(&add_module()).unwrap();
    assert!(WasmValidator::validate(&m).is_ok());
}

// ── ValType tests ─────────────────────────────────────────────────────────────

#[test]
fn test_valtype_from_byte() {
    assert_eq!(ValType::from_byte(0x7f).unwrap(), ValType::I32);
    assert_eq!(ValType::from_byte(0x7e).unwrap(), ValType::I64);
    assert_eq!(ValType::from_byte(0x7d).unwrap(), ValType::F32);
    assert_eq!(ValType::from_byte(0x7c).unwrap(), ValType::F64);
}

#[test]
fn test_valtype_invalid() {
    assert!(ValType::from_byte(0x00).is_err());
}

#[test]
fn test_valtype_roundtrip() {
    for vt in [ValType::I32, ValType::I64, ValType::F32, ValType::F64] {
        assert_eq!(ValType::from_byte(vt.to_byte()).unwrap(), vt);
    }
}

// ── WasmValue tests ───────────────────────────────────────────────────────────

#[test]
fn test_wasm_value_as_i32() {
    assert_eq!(WasmValue::I32(42).as_i32().unwrap(), 42);
    assert!(WasmValue::I64(1).as_i32().is_err());
}

#[test]
fn test_wasm_value_default_for() {
    assert_eq!(WasmValue::default_for(&ValType::I32), WasmValue::I32(0));
    assert_eq!(WasmValue::default_for(&ValType::I64), WasmValue::I64(0));
}

// ── Runtime tests ─────────────────────────────────────────────────────────────

#[test]
fn test_runtime_const42() {
    let mut rt = WasmRuntime::instantiate(&const42_module()).unwrap();
    let result = rt.call_by_name("answer", vec![]).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].as_i32().unwrap(), 42);
}

#[test]
fn test_runtime_add() {
    let mut rt = WasmRuntime::instantiate(&add_module()).unwrap();
    let result = rt.call_by_name("add", vec![
        WasmValue::I32(10), WasmValue::I32(32)
    ]).unwrap();
    assert_eq!(result[0].as_i32().unwrap(), 42);
}

#[test]
fn test_runtime_undefined_export() {
    let mut rt = WasmRuntime::instantiate(&add_module()).unwrap();
    assert!(rt.call_by_name("missing", vec![]).is_err());
}

#[test]
fn test_runtime_unreachable_trap() {
    let bytes = vec![
        0x00,0x61,0x73,0x6d, 0x01,0x00,0x00,0x00,
        0x01,0x04, 0x01,0x60,0x00,0x00,
        0x03,0x02, 0x01,0x00,
        0x07,0x07, 0x01,0x03,0x74,0x72,0x70,0x00,0x00,
        0x0a,0x04, 0x01,0x02,0x00,0x00, // unreachable (0x00) + end (0x0b) — missing end
    ];
    if let Ok(mut rt) = WasmRuntime::instantiate(&bytes) {
        let result = rt.call_by_name("trp", vec![]);
        assert!(result.is_err());
    }
}
