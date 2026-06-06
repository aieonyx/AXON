// axon_parser/src/axon_std.rs
// AXON Standard Library Core — Stage 8D
// Copyright © 2026 Edison Lepiten — AIEONYX
//
// Core types available to all AXON programs:
//   AxonVec<T>      — growable array
//   AxonOption<T>   — optional value
//   AxonResult<T,E> — fallible value
//   AxonString      — owned UTF-8 string
//   AxonSlice<T>    — borrowed slice
//
// Drop implementations respect 7E-5 invariant:
// DropElaborated only emitted when needs_drop() is true.
//
// Integration: these types map to HirTy::Named("Vec", [...]) etc.
// The codegen (8C) emits LLVM IR for their operations.

use crate::hir::HirTy;

// ============================================================
// TYPE REGISTRY
// ============================================================
// Maps stdlib type names to their HirTy representations.
// Used by type inference (8B) to resolve named types.

#[derive(Debug, Clone, PartialEq)]
pub enum StdType {
    Vec(Box<HirTy>),
    Option(Box<HirTy>),
    Result(Box<HirTy>, Box<HirTy>),
    String,
    Slice(Box<HirTy>),
    BoxT(Box<HirTy>),
    // P12-M1: iterator type
    Iterator(Box<HirTy>),
}

impl StdType {
    pub fn to_hir_ty(&self) -> HirTy {
        match self {
            StdType::Vec(t)       => HirTy::Named("Vec".into(), vec![*t.clone()]),
            StdType::Option(t)    => HirTy::Named("Option".into(), vec![*t.clone()]),
            StdType::Result(t, e) => HirTy::Named("Result".into(), vec![*t.clone(), *e.clone()]),
            StdType::String       => HirTy::Named("String".into(), vec![]),
            StdType::Slice(t)     => HirTy::Slice(t.clone()),
            StdType::BoxT(t)      => HirTy::Named("Box".into(), vec![*t.clone()]),
            // P12-M1
            StdType::Iterator(t) => HirTy::Named("AxonIterator".into(), vec![*t.clone()]),
        }
    }

    pub fn needs_drop(&self) -> bool {
        // 7E-5: all stdlib heap types need drop
        matches!(self, StdType::Vec(_) | StdType::String | StdType::BoxT(_) | StdType::Result(_, _) | StdType::Iterator(_))
    }

    pub fn is_copy(&self) -> bool {
        matches!(self, StdType::Option(_) | StdType::Slice(_))
    }
}

// ============================================================
// AXON VEC<T>
// ============================================================

/// Runtime layout of AxonVec<T> — matches LLVM struct layout.
/// { ptr: i64, len: i64, cap: i64 }
#[derive(Debug, Clone)]
pub struct AxonVec<T> {
    data: Vec<T>,
}

#[allow(clippy::new_without_default)]
impl<T> AxonVec<T> {
    pub fn new() -> Self {
        AxonVec { data: Vec::new() }
    }

    pub fn with_capacity(cap: usize) -> Self {
        AxonVec { data: Vec::with_capacity(cap) }
    }

    pub fn push(&mut self, val: T) {
        self.data.push(val);
    }

    pub fn pop(&mut self) -> AxonOption<T> {
        match self.data.pop() {
            Some(v) => AxonOption::Some(v),
            None    => AxonOption::None,
        }
    }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn cap(&self) -> usize { self.data.capacity() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }

    pub fn get(&self, idx: usize) -> AxonOption<&T> {
        match self.data.get(idx) {
            Some(v) => AxonOption::Some(v),
            None    => AxonOption::None,
        }
    }

    pub fn get_mut(&mut self, idx: usize) -> AxonOption<&mut T> {
        match self.data.get_mut(idx) {
            Some(v) => AxonOption::Some(v),
            None    => AxonOption::None,
        }
    }

    pub fn clear(&mut self) { self.data.clear(); }

    pub fn iter(&self) -> std::slice::Iter<'_, T> { self.data.iter() }
}

impl<T: Clone> AxonVec<T> {
    pub fn extend_from_slice(&mut self, slice: &[T]) {
        self.data.extend_from_slice(slice);
    }
}

impl<T> Drop for AxonVec<T> {
    fn drop(&mut self) {
        // 7E-5: DropElaborated — Vec<T> always needs drop
        // data is dropped by inner Vec<T>
    }
}

// ============================================================
// AXON OPTION<T>
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum AxonOption<T> {
    Some(T),
    None,
}

impl<T> AxonOption<T> {
    pub fn is_some(&self) -> bool { matches!(self, AxonOption::Some(_)) }
    pub fn is_none(&self) -> bool { matches!(self, AxonOption::None) }

    pub fn unwrap(self) -> T {
        match self {
            AxonOption::Some(v) => v,
            AxonOption::None    => panic!("AxonOption::unwrap called on None"),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            AxonOption::Some(v) => v,
            AxonOption::None    => default,
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> AxonOption<U> {
        match self {
            AxonOption::Some(v) => AxonOption::Some(f(v)),
            AxonOption::None    => AxonOption::None,
        }
    }

    pub fn and_then<U, F: FnOnce(T) -> AxonOption<U>>(self, f: F) -> AxonOption<U> {
        match self {
            AxonOption::Some(v) => f(v),
            AxonOption::None    => AxonOption::None,
        }
    }

    pub fn ok_or<E>(self, err: E) -> AxonResult<T, E> {
        match self {
            AxonOption::Some(v) => AxonResult::Ok(v),
            AxonOption::None    => AxonResult::Err(err),
        }
    }
}

// ============================================================
// AXON RESULT<T, E>
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum AxonResult<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> AxonResult<T, E> {
    pub fn is_ok(&self)  -> bool { matches!(self, AxonResult::Ok(_)) }
    pub fn is_err(&self) -> bool { matches!(self, AxonResult::Err(_)) }

    pub fn unwrap(self) -> T where E: std::fmt::Debug {
        match self {
            AxonResult::Ok(v)  => v,
            AxonResult::Err(e) => panic!("AxonResult::unwrap called on Err: {:?}", e),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            AxonResult::Ok(v)  => v,
            AxonResult::Err(_) => default,
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> AxonResult<U, E> {
        match self {
            AxonResult::Ok(v)  => AxonResult::Ok(f(v)),
            AxonResult::Err(e) => AxonResult::Err(e),
        }
    }

    pub fn map_err<F2, F: FnOnce(E) -> F2>(self, f: F) -> AxonResult<T, F2> {
        match self {
            AxonResult::Ok(v)  => AxonResult::Ok(v),
            AxonResult::Err(e) => AxonResult::Err(f(e)),
        }
    }

    pub fn ok(self) -> AxonOption<T> {
        match self {
            AxonResult::Ok(v)  => AxonOption::Some(v),
            AxonResult::Err(_) => AxonOption::None,
        }
    }
}

// ============================================================
// AXON STRING
// ============================================================

/// Sovereign UTF-8 string type.
/// Runtime layout: { ptr: i64, len: i64, cap: i64 }
#[derive(Debug, Clone, PartialEq)]
pub struct AxonString {
    data: String,
}

#[allow(clippy::new_without_default, clippy::should_implement_trait)]
impl AxonString {
    pub fn new() -> Self { AxonString { data: String::new() } }

    pub fn from_str(s: &str) -> Self { AxonString { data: s.to_string() } }

    pub fn push_str(&mut self, s: &str) { self.data.push_str(s); }

    pub fn push_char(&mut self, c: char) { self.data.push(c); }

    pub fn len(&self) -> usize { self.data.len() }

    pub fn is_empty(&self) -> bool { self.data.is_empty() }

    pub fn as_str(&self) -> &str { &self.data }

    pub fn contains(&self, pat: &str) -> bool { self.data.contains(pat) }

    pub fn starts_with(&self, pat: &str) -> bool { self.data.starts_with(pat) }

    pub fn ends_with(&self, pat: &str) -> bool { self.data.ends_with(pat) }

    pub fn to_uppercase(&self) -> AxonString {
        AxonString { data: self.data.to_uppercase() }
    }

    pub fn to_lowercase(&self) -> AxonString {
        AxonString { data: self.data.to_lowercase() }
    }

    pub fn trim(&self) -> &str { self.data.trim() }
}

impl Drop for AxonString {
    fn drop(&mut self) {
        // 7E-5: DropElaborated — String always needs drop
    }
}

impl std::fmt::Display for AxonString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

// ============================================================
// BASIC I/O
// ============================================================

/// Print to stdout — sovereign print primitive.
pub fn axon_print(s: &str) {
    print!("{}", s);
}

/// Print line to stdout.
pub fn axon_println(s: &str) {
    println!("{}", s);
}

/// Print integer to stdout.
pub fn axon_print_int(n: i64) {
    print!("{}", n);
}

/// Read line from stdin.
pub fn axon_read_line() -> AxonResult<AxonString, AxonString> {
    let mut buf = String::new();
    match std::io::stdin().read_line(&mut buf) {
        Ok(_)  => AxonResult::Ok(AxonString { data: buf.trim_end().to_string() }),
        Err(e) => AxonResult::Err(AxonString::from_str(&e.to_string())),
    }
}

// ============================================================
// LLVM IR SIGNATURES
// ============================================================
// These are the LLVM IR declarations for stdlib functions.
// Emitted at the top of every compiled module.

pub fn stdlib_ir_declarations() -> &'static str {
    "; === AXON stdlib declarations ===\n\
declare void @axon_print(ptr)\n\
declare void @axon_println(ptr)\n\
declare void @axon_print_int(i64)\n\
; P12-M1: iterator protocol\n\
declare ptr @axon_iter_next(ptr)\n\
; === end stdlib ===\n"
}

// ============================================================
// STDLIB TYPE LOOKUP
// ============================================================

/// Look up a stdlib type by name.
/// Used by type inference to resolve named types.
pub fn lookup_stdlib_type(name: &str, args: Vec<HirTy>) -> Option<StdType> {
    match name {
        "Vec" => {
            let inner = args.into_iter().next().unwrap_or(HirTy::Infer);
            Some(StdType::Vec(Box::new(inner)))
        }
        "Option" => {
            let inner = args.into_iter().next().unwrap_or(HirTy::Infer);
            Some(StdType::Option(Box::new(inner)))
        }
        "Result" => {
            let mut it = args.into_iter();
            let t = it.next().unwrap_or(HirTy::Infer);
            let e = it.next().unwrap_or(HirTy::Infer);
            Some(StdType::Result(Box::new(t), Box::new(e)))
        }
        "String" => Some(StdType::String),
        "Box"    => {
            let inner = args.into_iter().next().unwrap_or(HirTy::Infer);
            Some(StdType::BoxT(Box::new(inner)))
        }
        // P12-M1
        "AxonIterator" | "Iterator" => {
            let inner = args.into_iter().next().unwrap_or(HirTy::Infer);
            Some(StdType::Iterator(Box::new(inner)))
        }
        _ => None,
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn td1_vec_push_pop() {
        let mut v: AxonVec<i32> = AxonVec::new();
        v.push(1); v.push(2); v.push(3);
        assert_eq!(v.len(), 3);
        assert_eq!(v.pop(), AxonOption::Some(3));
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn td2_vec_get() {
        let mut v: AxonVec<i32> = AxonVec::new();
        v.push(42);
        assert_eq!(v.get(0), AxonOption::Some(&42));
        assert_eq!(v.get(1), AxonOption::None);
    }

    #[test]
    fn td3_vec_empty() {
        let mut v: AxonVec<i32> = AxonVec::new();
        assert!(v.is_empty());
        assert_eq!(v.pop(), AxonOption::None);
    }

    #[test]
    fn td4_option_map() {
        let x = AxonOption::Some(5i32);
        let y = x.map(|v| v * 2);
        assert_eq!(y, AxonOption::Some(10));
        let n: AxonOption<i32> = AxonOption::None;
        assert_eq!(n.map(|v| v * 2), AxonOption::None);
    }

    #[test]
    fn td5_option_ok_or() {
        let x = AxonOption::Some(1i32);
        assert_eq!(x.ok_or("err"), AxonResult::Ok(1));
        let n: AxonOption<i32> = AxonOption::None;
        assert_eq!(n.ok_or("err"), AxonResult::Err("err"));
    }

    #[test]
    fn td6_result_map() {
        let r: AxonResult<i32, &str> = AxonResult::Ok(5);
        assert_eq!(r.map(|v| v * 2), AxonResult::Ok(10));
        let e: AxonResult<i32, &str> = AxonResult::Err("fail");
        assert_eq!(e.map(|v| v * 2), AxonResult::Err("fail"));
    }

    #[test]
    fn td7_result_ok() {
        let r: AxonResult<i32, &str> = AxonResult::Ok(42);
        assert_eq!(r.ok(), AxonOption::Some(42));
        let e: AxonResult<i32, &str> = AxonResult::Err("fail");
        assert_eq!(e.ok(), AxonOption::None);
    }

    #[test]
    fn td8_string_ops() {
        let mut s = AxonString::new();
        s.push_str("hello");
        s.push_char(' ');
        s.push_str("world");
        assert_eq!(s.as_str(), "hello world");
        assert_eq!(s.len(), 11);
        assert!(s.contains("world"));
        assert!(s.starts_with("hello"));
        assert!(s.ends_with("world"));
    }

    #[test]
    fn td9_string_case() {
        let s = AxonString::from_str("Hello");
        assert_eq!(s.to_uppercase().as_str(), "HELLO");
        assert_eq!(s.to_lowercase().as_str(), "hello");
    }

    #[test]
    fn td10_stdlib_type_lookup_vec() {
        let st = lookup_stdlib_type("Vec", vec![crate::hir::HirTy::I32]);
        assert!(st.is_some());
        let st = st.unwrap();
        assert!(st.needs_drop());
        assert!(!st.is_copy());
    }

    #[test]
    fn td11_stdlib_type_lookup_option() {
        let st = lookup_stdlib_type("Option", vec![crate::hir::HirTy::I32]);
        assert!(st.is_some());
        let st = st.unwrap();
        assert!(!st.needs_drop());
        assert!(st.is_copy());
    }

    #[test]
    fn td12_stdlib_type_lookup_result() {
        let st = lookup_stdlib_type("Result", vec![crate::hir::HirTy::I32, crate::hir::HirTy::Str]);
        assert!(st.is_some());
        assert!(st.unwrap().needs_drop());
    }

    #[test]
    fn td13_stdlib_unknown_returns_none() {
        assert!(lookup_stdlib_type("Unknown", vec![]).is_none());
    }

    #[test]
    fn td14_std_type_to_hir_ty() {
        let st = StdType::Vec(Box::new(crate::hir::HirTy::I32));
        let hty = st.to_hir_ty();
        assert_eq!(hty, crate::hir::HirTy::Named("Vec".into(), vec![crate::hir::HirTy::I32]));
    }

    #[test]
    fn td15_axon_print_does_not_panic() {
        // Just confirm I/O primitives don't panic
        axon_print("");
        axon_print_int(42);
    }
    #[test]
    fn td16_iterator_type_lookup() {
        let st = lookup_stdlib_type("AxonIterator", vec![crate::hir::HirTy::I64]);
        assert!(st.is_some());
        let st = st.unwrap();
        assert!(st.needs_drop());
        let hty = st.to_hir_ty();
        assert_eq!(hty, crate::hir::HirTy::Named("AxonIterator".into(), vec![crate::hir::HirTy::I64]));
    }

    #[test]
    fn td17_iterator_alias_lookup() {
        // "Iterator" is an accepted alias for "AxonIterator"
        let st = lookup_stdlib_type("Iterator", vec![crate::hir::HirTy::Bool]);
        assert!(st.is_some());
    }

    #[test]
    fn td18_iterator_no_inner_defaults_to_infer() {
        let st = lookup_stdlib_type("AxonIterator", vec![]);
        assert!(st.is_some());
        let hty = st.unwrap().to_hir_ty();
        assert_eq!(hty, crate::hir::HirTy::Named("AxonIterator".into(), vec![crate::hir::HirTy::Infer]));
    }

}

// P12-M1-APPLIED
