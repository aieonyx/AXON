// ============================================================
// axon_llvm — types.rs
// AXON type → LLVM IR type mapping
// Copyright © 2026 Edison Lepiten — AIEONYX
// ============================================================

use axon_parser::ast::{Type, PrimitiveType};
use crate::ir::LlvmType;

pub fn llvm_type(ty: &Type) -> LlvmType {
    match ty {
        Type::Primitive(p, _)  => primitive_llvm_type(p),
        Type::Unit(_)          => LlvmType::I64,
        Type::Option(inner, _) => { let _ = inner; LlvmType::I64 }
        Type::Result(ok, _, _) => llvm_type(ok),
        Type::List(_, _)       => LlvmType::Ptr,
        Type::Named(_)         => LlvmType::I64,
        _                      => LlvmType::I64,
    }
}

pub fn primitive_llvm_type(p: &PrimitiveType) -> LlvmType {
    match p {
        PrimitiveType::Int     => LlvmType::I64,
        PrimitiveType::Int64   => LlvmType::I64,
        PrimitiveType::Int32   => LlvmType::I32,
        PrimitiveType::Int8    => LlvmType::I8,
        PrimitiveType::UInt    => LlvmType::I64,
        PrimitiveType::UInt64  => LlvmType::I64,
        PrimitiveType::UInt32  => LlvmType::I32,
        PrimitiveType::UInt8   => LlvmType::I8,
        PrimitiveType::Float   => LlvmType::F64,
        PrimitiveType::Float32 => LlvmType::F32,
        PrimitiveType::Bool    => LlvmType::I1,
        PrimitiveType::Char    => LlvmType::I32,
        PrimitiveType::Str     => LlvmType::Ptr,
        PrimitiveType::Bytes   => LlvmType::Ptr,
    }
}
