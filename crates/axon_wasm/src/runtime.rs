// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// WASM stack-machine interpreter -- sovereign implementation.
// Clean-room: studied WebAssembly Core Spec 2.0 execution semantics only.
// P61.0: MVP instruction set -- i32 arithmetic, local get/set, call, return.
use crate::module::{WasmModule, read_leb128_u32};
use crate::types::{WasmValue, ValType};
use crate::error::{WasmError, WasmResult};
use crate::validator::{WasmValidator, MAX_STACK_DEPTH};

// WASM MVP opcodes
const OP_UNREACHABLE:  u8 = 0x00;
const OP_NOP:          u8 = 0x01;
const OP_RETURN:       u8 = 0x0f;
const OP_CALL:         u8 = 0x10;
const OP_DROP:         u8 = 0x1a;
const OP_LOCAL_GET:    u8 = 0x20;
const OP_LOCAL_SET:    u8 = 0x21;
const OP_LOCAL_TEE:    u8 = 0x22;
const OP_I32_CONST:    u8 = 0x41;
const OP_I64_CONST:    u8 = 0x42;
const OP_I32_EQZ:      u8 = 0x45;
const OP_I32_EQ:       u8 = 0x46;
const OP_I32_NE:       u8 = 0x47;
const OP_I32_LT_S:     u8 = 0x48;
const OP_I32_GT_S:     u8 = 0x4a;
const OP_I32_LE_S:     u8 = 0x4c;
const OP_I32_GE_S:     u8 = 0x4e;
const OP_I32_ADD:      u8 = 0x6a;
const OP_I32_SUB:      u8 = 0x6b;
const OP_I32_MUL:      u8 = 0x6c;
const OP_I32_DIV_S:    u8 = 0x6d;
const OP_I32_REM_S:    u8 = 0x6f;
const OP_I32_AND:      u8 = 0x71;
const OP_I32_OR:       u8 = 0x72;
const OP_I32_XOR:      u8 = 0x73;
const OP_END:          u8 = 0x0b;

pub struct WasmRuntime {
    module: WasmModule,
    memory: Vec<u8>,
}

impl WasmRuntime {
    pub fn instantiate(bytes: &[u8]) -> WasmResult<Self> {
        let module = WasmModule::parse(bytes)?;
        WasmValidator::validate(&module)?;
        Ok(WasmRuntime { module, memory: vec![0u8; 65536] })
    }

    pub fn from_module(module: WasmModule) -> WasmResult<Self> {
        WasmValidator::validate(&module)?;
        Ok(WasmRuntime { module, memory: vec![0u8; 65536] })
    }

    pub fn call_by_name(&mut self, name: &str, args: Vec<WasmValue>) -> WasmResult<Vec<WasmValue>> {
        let export = self.module.find_export(name)
            .ok_or_else(|| WasmError::UndefinedFunction(0))?;
        if export.kind != 0 {
            return Err(WasmError::ValidationFailed(format!("{} is not a function", name)));
        }
        let func_idx = export.index;
        self.call_function(func_idx, args)
    }

    pub fn call_function(&mut self, func_idx: u32, args: Vec<WasmValue>) -> WasmResult<Vec<WasmValue>> {
        let fi = func_idx as usize;
        if fi >= self.module.func_types.len() {
            return Err(WasmError::UndefinedFunction(func_idx));
        }
        let type_idx = self.module.func_types[fi] as usize;
        let func_type = self.module.types[type_idx].clone();
        let body = self.module.bodies[fi].clone();

        let mut locals: Vec<WasmValue> = args;
        for local_type in &body.locals {
            locals.push(WasmValue::default_for(local_type));
        }

        let mut stack: Vec<WasmValue> = Vec::new();
        let mut pc = 0usize;
        let code = &body.code;

        while pc < code.len() {
            let op = code[pc]; pc += 1;
            if stack.len() > MAX_STACK_DEPTH {
                return Err(WasmError::StackOverflow);
            }
            match op {
                OP_UNREACHABLE => return Err(WasmError::TrapUnreachable),
                OP_NOP => {}
                OP_END | OP_RETURN => break,
                OP_DROP => { stack.pop().ok_or(WasmError::StackUnderflow)?; }
                OP_LOCAL_GET => {
                    let (idx, n) = read_leb128_u32(code, pc)?; pc += n;
                    let val = locals.get(idx as usize)
                        .ok_or(WasmError::UndefinedFunction(idx))?
                        .clone();
                    stack.push(val);
                }
                OP_LOCAL_SET => {
                    let (idx, n) = read_leb128_u32(code, pc)?; pc += n;
                    let val = stack.pop().ok_or(WasmError::StackUnderflow)?;
                    if idx as usize >= locals.len() {
                        return Err(WasmError::UndefinedFunction(idx));
                    }
                    locals[idx as usize] = val;
                }
                OP_LOCAL_TEE => {
                    let (idx, n) = read_leb128_u32(code, pc)?; pc += n;
                    let val = stack.last().ok_or(WasmError::StackUnderflow)?.clone();
                    if idx as usize >= locals.len() {
                        return Err(WasmError::UndefinedFunction(idx));
                    }
                    locals[idx as usize] = val;
                }
                OP_I32_CONST => {
                    let (v, n) = read_leb128_u32(code, pc)?; pc += n;
                    stack.push(WasmValue::I32(v as i32));
                }
                OP_I64_CONST => {
                    let (v, n) = read_leb128_u32(code, pc)?; pc += n;
                    stack.push(WasmValue::I64(v as i64));
                }
                OP_I32_EQZ => {
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    stack.push(WasmValue::I32(if a == 0 { 1 } else { 0 }));
                }
                OP_I32_EQ => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    stack.push(WasmValue::I32(if a == b { 1 } else { 0 }));
                }
                OP_I32_NE => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    stack.push(WasmValue::I32(if a != b { 1 } else { 0 }));
                }
                OP_I32_LT_S => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    stack.push(WasmValue::I32(if a < b { 1 } else { 0 }));
                }
                OP_I32_GT_S => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    stack.push(WasmValue::I32(if a > b { 1 } else { 0 }));
                }
                OP_I32_LE_S => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    stack.push(WasmValue::I32(if a <= b { 1 } else { 0 }));
                }
                OP_I32_GE_S => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    stack.push(WasmValue::I32(if a >= b { 1 } else { 0 }));
                }
                OP_I32_ADD => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    stack.push(WasmValue::I32(a.wrapping_add(b)));
                }
                OP_I32_SUB => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    stack.push(WasmValue::I32(a.wrapping_sub(b)));
                }
                OP_I32_MUL => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    stack.push(WasmValue::I32(a.wrapping_mul(b)));
                }
                OP_I32_DIV_S => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    if b == 0 { return Err(WasmError::TrapUnreachable); }
                    stack.push(WasmValue::I32(a.wrapping_div(b)));
                }
                OP_I32_REM_S => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    if b == 0 { return Err(WasmError::TrapUnreachable); }
                    stack.push(WasmValue::I32(a.wrapping_rem(b)));
                }
                OP_I32_AND => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    stack.push(WasmValue::I32(a & b));
                }
                OP_I32_OR => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    stack.push(WasmValue::I32(a | b));
                }
                OP_I32_XOR => {
                    let b = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    let a = stack.pop().ok_or(WasmError::StackUnderflow)?.as_i32()?;
                    stack.push(WasmValue::I32(a ^ b));
                }
                OP_CALL => {
                    let (idx, n) = read_leb128_u32(code, pc)?; pc += n;
                    let fi = idx as usize;
                    if fi >= self.module.func_types.len() {
                        return Err(WasmError::UndefinedFunction(idx));
                    }
                    let type_idx = self.module.func_types[fi] as usize;
                    let callee_type = self.module.types[type_idx].clone();
                    let mut call_args = Vec::new();
                    for _ in 0..callee_type.params.len() {
                        call_args.push(stack.pop().ok_or(WasmError::StackUnderflow)?);
                    }
                    call_args.reverse();
                    let results = self.call_function(idx, call_args)?;
                    stack.extend(results);
                }
                other => return Err(WasmError::InvalidInstruction(other)),
            }
        }

        let result_count = func_type.results.len();
        if stack.len() < result_count {
            return Err(WasmError::StackUnderflow);
        }
        let results = stack.split_off(stack.len() - result_count);
        Ok(results)
    }

    pub fn memory(&self) -> &[u8] { &self.memory }
    pub fn module(&self) -> &WasmModule { &self.module }
}
