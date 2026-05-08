// ============================================================
// axon_llvm — ir.rs
// LLVM IR text emitter
// Copyright © 2026 Edison Lepiten — AIEONYX
//
// Emits LLVM IR (text format) directly as strings.
// The resulting .ll files are compiled with:
//   llc-18 -filetype=obj program.ll -o program.o
//   clang-18 program.o -o program
//
// LLVM IR reference: https://llvm.org/docs/LangRef.html
// ============================================================

use std::fmt::Write;

/// Tracks a local SSA value name (e.g. %add_1, %cond_2)
#[derive(Debug, Clone)]
pub struct Value(pub String);

impl Value {
    pub fn named(name: &str) -> Self { Value(format!("%{}", name)) }
    pub fn temp(id: usize, hint: &str) -> Self { Value(format!("%{}_{}", hint, id)) }
    pub fn global(name: &str) -> Self { Value(format!("@{}", name)) }
    pub fn int_const(n: i64) -> Self { Value(n.to_string()) }
    pub fn zero() -> Self { Value("0".to_string()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// LLVM IR types
#[derive(Debug, Clone, PartialEq)]
pub enum LlvmType {
    I1,            // bool
    I8,            // i8
    I32,           // i32
    I64,           // i64
    F32,           // float
    F64,           // double
    Ptr,           // ptr (opaque pointer, LLVM 15+)
    Void,
    Struct(Vec<LlvmType>),
    Array(Box<LlvmType>, usize),
}

impl LlvmType {
    pub fn as_str(&self) -> String {
        match self {
            LlvmType::I1           => "i1".to_string(),
            LlvmType::I8           => "i8".to_string(),
            LlvmType::I32          => "i32".to_string(),
            LlvmType::I64          => "i64".to_string(),
            LlvmType::F32          => "float".to_string(),
            LlvmType::F64          => "double".to_string(),
            LlvmType::Ptr          => "ptr".to_string(),
            LlvmType::Void         => "void".to_string(),
            LlvmType::Struct(fields) => {
                let fs = fields.iter().map(|f| f.as_str()).collect::<Vec<_>>().join(", ");
                format!("{{ {} }}", fs)
            }
            LlvmType::Array(elem, n) => format!("[{} x {}]", n, elem.as_str()),
        }
    }
}

/// A basic block with a label and instructions
pub struct BasicBlock {
    pub label: String,
    pub instrs: Vec<String>,
}

impl BasicBlock {
    pub fn new(label: &str) -> Self {
        BasicBlock { label: label.to_string(), instrs: Vec::new() }
    }

    pub fn push(&mut self, instr: impl Into<String>) {
        self.instrs.push(instr.into());
    }

    pub fn is_terminated(&self) -> bool {
        self.instrs.last()
            .map(|i| i.trim_start().starts_with("ret ")
                   || i.trim_start().starts_with("br ")
                   || i.trim_start().starts_with("switch ")
                   || i.trim_start().starts_with("unreachable"))
            .unwrap_or(false)
    }
}

/// A function definition
pub struct LlvmFunction {
    pub name       : String,
    pub params     : Vec<(LlvmType, String)>,
    pub ret_type   : LlvmType,
    pub blocks     : Vec<BasicBlock>,
    pub is_declare : bool, // external declaration (no body)
}

impl LlvmFunction {
    pub fn new(
        name: &str,
        params: Vec<(LlvmType, String)>,
        ret_type: LlvmType,
    ) -> Self {
        LlvmFunction {
            name: name.to_string(),
            params, ret_type,
            blocks: vec![BasicBlock::new("entry")],
            is_declare: false,
        }
    }

    pub fn current_block(&mut self) -> &mut BasicBlock {
        self.blocks.last_mut().expect("function must have at least one block")
    }

    pub fn add_block(&mut self, label: &str) {
        self.blocks.push(BasicBlock::new(label));
    }

    pub fn set_active_block(&mut self, label: &str) {
        if let Some(idx) = self.blocks.iter().position(|b| b.label == label) {
            // Move to end to make it "current"
            let block = self.blocks.remove(idx);
            self.blocks.push(block);
        }
    }

    pub fn emit(&self) -> String {
        let mut out = String::new();
        if self.is_declare {
            let params = self.params.iter()
                .map(|(ty, _)| ty.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(out, "declare {} @{}({})",
                self.ret_type.as_str(), self.name, params).unwrap();
            return out;
        }

        let params = self.params.iter()
            .map(|(ty, name)| format!("{} %{}", ty.as_str(), name))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(out, "define {} @{}({}) {{",
            self.ret_type.as_str(), self.name, params).unwrap();

        for block in &self.blocks {
            writeln!(out, "{}:", block.label).unwrap();
            for instr in &block.instrs {
                writeln!(out, "  {}", instr).unwrap();
            }
        }
        writeln!(out, "}}").unwrap();
        out
    }
}

/// A complete LLVM module
pub struct LlvmModule {
    pub name       : String,
    pub functions  : Vec<LlvmFunction>,
    pub globals    : Vec<String>,
    pub type_defs  : Vec<String>,
}

impl LlvmModule {
    pub fn new(name: &str) -> Self {
        LlvmModule {
            name      : name.to_string(),
            functions : Vec::new(),
            globals   : Vec::new(),
            type_defs : Vec::new(),
        }
    }

    pub fn add_function(&mut self, func: LlvmFunction) {
        self.functions.push(func);
    }

    pub fn emit(&self) -> String {
        let mut out = String::new();
        writeln!(out, "; ============================================================").unwrap();
        writeln!(out, "; AXON LLVM IR — module {}", self.name).unwrap();
        writeln!(out, "; Generated by AXON Native Backend — Phase 4").unwrap();
        writeln!(out, "; Copyright © 2026 Edison Lepiten — AIEONYX").unwrap();
        writeln!(out, "; ============================================================").unwrap();
        writeln!(out).unwrap();
        writeln!(out, "target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128\"").unwrap();
        writeln!(out, "target triple = \"x86_64-unknown-linux-gnu\"").unwrap();
        writeln!(out).unwrap();

        for td in &self.type_defs {
            writeln!(out, "{}", td).unwrap();
        }
        if !self.type_defs.is_empty() { writeln!(out).unwrap(); }

        for g in &self.globals {
            writeln!(out, "{}", g).unwrap();
        }
        if !self.globals.is_empty() { writeln!(out).unwrap(); }

        for func in &self.functions {
            out.push_str(&func.emit());
            writeln!(out).unwrap();
        }

        out
    }
}

/// SSA value counter for unique names
pub struct Counter {
    n: usize,
}

impl Counter {
    pub fn new() -> Self { Counter { n: 0 } }

    pub fn next(&mut self, hint: &str) -> Value {
        let v = Value::temp(self.n, hint);
        self.n += 1;
        v
    }
}
