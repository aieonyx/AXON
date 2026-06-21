// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// WASM binary module parser -- sovereign implementation.
// Clean-room: studied WebAssembly Core Spec 2.0 binary format only.
// P61.0: parses type, function, export, code sections.
use crate::types::{ValType, FuncType};
use crate::error::{WasmError, WasmResult};

pub const WASM_MAGIC:   [u8; 4] = [0x00, 0x61, 0x73, 0x6d];
pub const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

pub const SECTION_TYPE:     u8 = 1;
pub const SECTION_IMPORT:   u8 = 2;
pub const SECTION_FUNCTION: u8 = 3;
pub const SECTION_EXPORT:   u8 = 7;
pub const SECTION_CODE:     u8 = 10;

#[derive(Debug, Clone)]
pub struct Export {
    pub name:  String,
    pub kind:  u8,
    pub index: u32,
}

#[derive(Debug, Clone)]
pub struct FuncBody {
    pub locals: Vec<ValType>,
    pub code:   Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct WasmModule {
    pub types:     Vec<FuncType>,
    pub func_types: Vec<u32>,
    pub exports:   Vec<Export>,
    pub bodies:    Vec<FuncBody>,
}

impl WasmModule {
    pub fn parse(bytes: &[u8]) -> WasmResult<Self> {
        if bytes.len() < 8 {
            return Err(WasmError::ParseError("module too short".into()));
        }
        if bytes[0..4] != WASM_MAGIC {
            return Err(WasmError::InvalidMagic);
        }
        if bytes[4..8] != WASM_VERSION {
            return Err(WasmError::InvalidVersion);
        }
        let mut module = WasmModule {
            types: vec![], func_types: vec![], exports: vec![], bodies: vec![],
        };
        let mut pos = 8usize;
        while pos < bytes.len() {
            if pos >= bytes.len() { break; }
            let section_id = bytes[pos]; pos += 1;
            let (size, n) = read_leb128_u32(bytes, pos)?;
            pos += n;
            let section_end = pos + size as usize;
            if section_end > bytes.len() {
                return Err(WasmError::ParseError("section extends beyond module".into()));
            }
            match section_id {
                SECTION_TYPE     => module.parse_type_section(&bytes[pos..section_end])?,
                SECTION_FUNCTION => module.parse_function_section(&bytes[pos..section_end])?,
                SECTION_EXPORT   => module.parse_export_section(&bytes[pos..section_end])?,
                SECTION_CODE     => module.parse_code_section(&bytes[pos..section_end])?,
                _                => {} // skip unknown sections
            }
            pos = section_end;
        }
        Ok(module)
    }

    fn parse_type_section(&mut self, data: &[u8]) -> WasmResult<()> {
        let mut pos = 0;
        let (count, n) = read_leb128_u32(data, pos)?; pos += n;
        for _ in 0..count {
            if pos >= data.len() || data[pos] != 0x60 {
                return Err(WasmError::ParseError("expected func type marker 0x60".into()));
            }
            pos += 1;
            let (param_count, n) = read_leb128_u32(data, pos)?; pos += n;
            let mut params = vec![];
            for _ in 0..param_count {
                params.push(ValType::from_byte(data[pos])?); pos += 1;
            }
            let (result_count, n) = read_leb128_u32(data, pos)?; pos += n;
            let mut results = vec![];
            for _ in 0..result_count {
                results.push(ValType::from_byte(data[pos])?); pos += 1;
            }
            self.types.push(FuncType { params, results });
        }
        Ok(())
    }

    fn parse_function_section(&mut self, data: &[u8]) -> WasmResult<()> {
        let mut pos = 0;
        let (count, n) = read_leb128_u32(data, pos)?; pos += n;
        for _ in 0..count {
            let (idx, n) = read_leb128_u32(data, pos)?; pos += n;
            self.func_types.push(idx);
        }
        Ok(())
    }

    fn parse_export_section(&mut self, data: &[u8]) -> WasmResult<()> {
        let mut pos = 0;
        let (count, n) = read_leb128_u32(data, pos)?; pos += n;
        for _ in 0..count {
            let (name_len, n) = read_leb128_u32(data, pos)?; pos += n;
            let name = String::from_utf8_lossy(&data[pos..pos+name_len as usize]).to_string();
            pos += name_len as usize;
            let kind = data[pos]; pos += 1;
            let (index, n) = read_leb128_u32(data, pos)?; pos += n;
            self.exports.push(Export { name, kind, index });
        }
        Ok(())
    }

    fn parse_code_section(&mut self, data: &[u8]) -> WasmResult<()> {
        let mut pos = 0;
        let (count, n) = read_leb128_u32(data, pos)?; pos += n;
        for _ in 0..count {
            let (body_size, n) = read_leb128_u32(data, pos)?; pos += n;
            let body_end = pos + body_size as usize;
            let mut bpos = pos;
            let (local_count, n) = read_leb128_u32(data, bpos)?; bpos += n;
            let mut locals = vec![];
            for _ in 0..local_count {
                let (n_locals, n) = read_leb128_u32(data, bpos)?; bpos += n;
                let vt = ValType::from_byte(data[bpos])?; bpos += 1;
                for _ in 0..n_locals { locals.push(vt.clone()); }
            }
            let code = data[bpos..body_end].to_vec();
            self.bodies.push(FuncBody { locals, code });
            pos = body_end;
        }
        Ok(())
    }

    pub fn find_export(&self, name: &str) -> Option<&Export> {
        self.exports.iter().find(|e| e.name == name)
    }

    pub fn func_count(&self) -> usize { self.func_types.len() }
}

pub fn read_leb128_u32(data: &[u8], pos: usize) -> WasmResult<(u32, usize)> {
    let mut result = 0u32;
    let mut shift  = 0u32;
    let mut i      = pos;
    loop {
        if i >= data.len() {
            return Err(WasmError::ParseError("unexpected end in LEB128".into()));
        }
        let byte = data[i] as u32; i += 1;
        result |= (byte & 0x7f) << shift;
        if byte & 0x80 == 0 { break; }
        shift += 7;
        if shift >= 32 { return Err(WasmError::ParseError("LEB128 overflow".into())); }
    }
    Ok((result, i - pos))
}

// P61.1: WasmFunc — function body for JIT compilation
#[derive(Debug, Clone)]
pub struct WasmFunc {
    pub type_idx: u32,
    pub locals: Vec<crate::types::ValType>,
    /// Raw WASM bytecode body (expression, including 0x0B end marker)
    pub body: Vec<u8>,
}
