// ============================================================
// AXON Codegen — types.rs
// AXON type → Rust type mapping
// Copyright © 2026 Edison Lepiten — AIEONYX
// ============================================================

use axon_parser::ast::{Type, PrimitiveType};

/// Map an AXON Type to its Rust equivalent.
pub fn rust_type(ty: &Type) -> String {
    match ty {
        Type::Primitive(p, _) => primitive_type(p),
        Type::Unit(_)         => "()".into(),
        Type::Option(inner,_) => format!("Option<{}>", rust_type(inner)),
        Type::Result(ok,err,_)=> format!("Result<{}, {}>", rust_type(ok), rust_type(err)),
        Type::List(inner,_)   => format!("Vec<{}>", rust_type(inner)),
        Type::Cap(inner,_)    => rust_type(inner), // capabilities are transparent in Rust
        Type::Named(parts)    => {
            parts.iter().map(|i| i.name.as_str()).collect::<Vec<_>>().join("::")
        }
        Type::Generic(name,args,_) => {
            let args_str = args.iter()
                .map(rust_type)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", name.name, args_str)
        }
        Type::Ref(inner, _)    => format!("&{}",     rust_type(inner)),
        Type::MutRef(inner, _) => format!("&mut {}", rust_type(inner)),
        _ => "/* unsupported type */".into(),
    }
}

fn primitive_type(p: &PrimitiveType) -> String {
    match p {
        PrimitiveType::Int    => "i64".into(),
        PrimitiveType::Int32  => "i32".into(),
        PrimitiveType::Int64  => "i64".into(),
        PrimitiveType::Int8   => "i8".into(),
        PrimitiveType::UInt   => "u64".into(),
        PrimitiveType::UInt32 => "u32".into(),
        PrimitiveType::UInt64 => "u64".into(),
        PrimitiveType::UInt8  => "u8".into(),
        PrimitiveType::Float  => "f64".into(),
        PrimitiveType::Float32=> "f32".into(),
        PrimitiveType::Bool   => "bool".into(),
        PrimitiveType::Char   => "char".into(),
        PrimitiveType::Str    => "String".into(),
        PrimitiveType::Bytes  => "Vec<u8>".into(),
    }
}
