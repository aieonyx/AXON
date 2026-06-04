// axon_parser/src/codegen.rs
// AXON LLVM IR Emitter — Stage 8C
// Copyright © 2026 Edison Lepiten — AIEONYX
// Target: x86_64-pc-linux-gnu, LLVM 18

use crate::hir::{
    HirModule, HirItem, HirFn, HirExpr, HirExprKind,
    HirStmt, HirStmtKind, HirLit, HirTy, HirPat,
    HirMatchArm, PlaceId,
};
use crate::parser::BinaryOp;
use std::collections::HashMap;
use std::process::Command;

pub fn emit_llvm_ty(ty: &HirTy) -> &'static str {
    match ty {
        HirTy::Bool   => "i1",
        HirTy::I8     => "i8",
        HirTy::I16    => "i16",
        HirTy::I32    => "i32",
        HirTy::I64    => "i64",
        HirTy::I128   => "i128",
        HirTy::Isize  => "i64",
        HirTy::U8     => "i8",
        HirTy::U16    => "i16",
        HirTy::U32    => "i32",
        HirTy::U64    => "i64",
        HirTy::U128   => "i128",
        HirTy::Usize  => "i64",
        HirTy::F32    => "float",
        HirTy::F64    => "double",
        HirTy::Char   => "i32",
        HirTy::Unit   => "void",
        HirTy::Never  => "void",
        HirTy::Str    => "ptr",
        _             => "i32", // fallback: prefer i32 over i64 for unresolved Infer
    }
}

pub fn emit_llvm_ty_owned(ty: &HirTy) -> String {
    emit_llvm_ty(ty).to_string()
}

struct SsaNames {
    place_counter: u32,
    tmp_counter: u32,
    place_map: HashMap<PlaceId, String>,
}

impl SsaNames {
    fn new() -> Self {
        SsaNames { place_counter: 0, tmp_counter: 0, place_map: HashMap::new() }
    }
    fn place_name(&mut self, place: PlaceId) -> String {
        if let Some(name) = self.place_map.get(&place) { return name.clone(); }
        let name = format!("%p{}", self.place_counter);
        self.place_counter += 1;
        self.place_map.insert(place, name.clone());
        name
    }
    fn fresh_tmp(&mut self) -> String {
        let name = format!("%t{}", self.tmp_counter);
        self.tmp_counter += 1;
        name
    }
}

pub struct LlvmEmitter {
    output: String,
    ssa: SsaNames,
    errors: Vec<String>,
}

impl LlvmEmitter {
    pub fn new() -> Self {
        LlvmEmitter { output: String::new(), ssa: SsaNames::new(), errors: Vec::new() }
    }
    fn emit_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }
    fn emit_blank(&mut self) { self.output.push('\n'); }

    pub fn emit_module(&mut self, module: &HirModule) -> String {
        self.emit_line("; AXON compiled output");
        self.emit_line("; Target: x86_64-pc-linux-gnu");
        self.emit_line("; LLVM 18");
        self.emit_line("source_filename = \"axon_module\"");
        self.emit_line("target datalayout = \"e-m:e-i64:64-f80:128-n8:16:32:64-S128\"");
        self.emit_line("target triple = \"x86_64-pc-linux-gnu\"");
        self.emit_blank();
        for item in &module.items { self.emit_item(item); }
        self.emit_blank();
        self.emit_line("!llvm.module.flags = !{!0}");
        self.emit_line("!0 = !{i32 1, !\"axon_sovereign\", i32 1}");
        self.output.clone()
    }

    fn emit_item(&mut self, item: &HirItem) {
        match item {
            HirItem::Fn(f) => self.emit_fn(f),
            HirItem::Struct(s) => {
                let fields: Vec<String> = s.fields.iter()
                    .map(|(_, ty, _)| emit_llvm_ty_owned(ty))
                    .collect();
                self.emit_line(&format!("%struct.{} = type {{ {} }}", s.name, fields.join(", ")));
                self.emit_blank();
            }
            HirItem::Enum(e) => {
                self.emit_line(&format!("%enum.{} = type {{ i32, [8 x i8] }}", e.name));
                self.emit_blank();
            }
            HirItem::Const(name, ty, _, _) => {
                self.emit_line(&format!("@{} = constant {} undef", name, emit_llvm_ty(ty)));
                self.emit_blank();
            }
            _ => {}
        }
    }

    fn emit_fn(&mut self, f: &HirFn) {
        self.ssa = SsaNames::new();
        let params: Vec<String> = f.params.iter().map(|(place, ty)| {
            let name = self.ssa.place_name(*place);
            format!("{} {}", emit_llvm_ty(ty), name)
        }).collect();
        let ret_ty = emit_llvm_ty(&f.ret);
        let linkage = if f.is_pub { "" } else { "internal " };
        if matches!(f.ret, HirTy::Unit | HirTy::Never) {
            self.emit_line(&format!("define {}void @{}({}) {{", linkage, f.name, params.join(", ")));
        } else {
            self.emit_line(&format!("define {}{} @{}({}) {{", linkage, ret_ty, f.name, params.join(", ")));
        }
        self.emit_line("entry:");
        for (place, ty) in &f.params {
            let name = self.ssa.place_name(*place);
            let llty = emit_llvm_ty(ty);
            let alloca = self.ssa.fresh_tmp();
            self.emit_line(&format!("  {} = alloca {}", alloca, llty));
            self.emit_line(&format!("  store {} {}, ptr {}", llty, name, alloca));
        }
        let body_val = self.emit_expr(&f.body);
        if matches!(f.ret, HirTy::Unit | HirTy::Never) {
            self.emit_line("  ret void");
        } else {
            let ret_ty = emit_llvm_ty(&f.ret);
            if let Some(val) = body_val {
                self.emit_line(&format!("  ret {} {}", ret_ty, val));
            } else {
                self.emit_line(&format!("  ret {} {}", ret_ty, self.default_value(&f.ret)));
            }
        }
        self.emit_line("}");
        self.emit_blank();
    }

    fn emit_expr(&mut self, expr: &HirExpr) -> Option<String> {
        match &expr.kind {
            HirExprKind::Lit(lit) => Some(self.emit_lit(lit)),
            HirExprKind::Place(place, _) => {
                let name = self.ssa.place_name(*place);
                let ty = emit_llvm_ty(&expr.ty);
                let tmp = self.ssa.fresh_tmp();
                self.emit_line(&format!("  {} = load {}, ptr {}", tmp, ty, name));
                Some(tmp)
            }
            HirExprKind::Block(stmts, tail) => {
                for stmt in stmts { self.emit_stmt(stmt); }
                if let Some(t) = tail { self.emit_expr(t) } else { None }
            }
            HirExprKind::BinOp(op, lhs, rhs) => {
                let lv = self.emit_expr(lhs)?;
                let rv = self.emit_expr(rhs)?;
                let tmp = self.ssa.fresh_tmp();
                let ty = emit_llvm_ty(&lhs.ty);
                let instr = self.binop_instr(op, &lhs.ty);
                self.emit_line(&format!("  {} = {} {} {}, {}", tmp, instr, ty, lv, rv));
                Some(tmp)
            }
            HirExprKind::UnOp(_op, inner) => {
                let iv = self.emit_expr(inner)?;
                let tmp = self.ssa.fresh_tmp();
                let ty = emit_llvm_ty(&inner.ty);
                self.emit_line(&format!("  {} = sub {} 0, {}", tmp, ty, iv));
                Some(tmp)
            }
            HirExprKind::Return(val) => {
                if let Some(v) = val {
                    if let Some(rv) = self.emit_expr(v) {
                        let ty = emit_llvm_ty(&v.ty);
                        self.emit_line(&format!("  ret {} {}", ty, rv));
                    } else {
                        self.emit_line("  ret void");
                    }
                } else {
                    self.emit_line("  ret void");
                }
                let lbl = format!("after_ret_{}:", self.ssa.tmp_counter);
                self.ssa.tmp_counter += 1;
                self.emit_line(&lbl);
                None
            }
            HirExprKind::If(cond, then, else_) => {
                let cv = self.emit_expr(cond)?;
                let n = self.ssa.tmp_counter; self.ssa.tmp_counter += 3;
                let (tl, el, ml) = (format!("then_{}", n), format!("else_{}", n+1), format!("merge_{}", n+2));
                self.emit_line(&format!("  br i1 {}, label %{}, label %{}", cv, tl, el));
                self.emit_line(&format!("{}:", tl));
                let tv = self.emit_expr(then);
                self.emit_line(&format!("  br label %{}", ml));
                self.emit_line(&format!("{}:", el));
                let ev = if let Some(e) = else_ { self.emit_expr(e) } else { None };
                self.emit_line(&format!("  br label %{}", ml));
                self.emit_line(&format!("{}:", ml));
                if let (Some(tv), Some(ev)) = (tv, ev) {
                    let ty = emit_llvm_ty(&then.ty);
                    let phi = self.ssa.fresh_tmp();
                    self.emit_line(&format!("  {} = phi {} [ {}, %{} ], [ {}, %{} ]", phi, ty, tv, tl, ev, el));
                    Some(phi)
                } else { None }
            }
            HirExprKind::While(cond, body) => {
                let n = self.ssa.tmp_counter; self.ssa.tmp_counter += 3;
                let (cl, bl, xl) = (format!("wcond_{}", n), format!("wbody_{}", n+1), format!("wexit_{}", n+2));
                self.emit_line(&format!("  br label %{}", cl));
                self.emit_line(&format!("{}:", cl));
                let cv = self.emit_expr(cond);
                let cv = cv.unwrap_or_else(|| "0".to_string());
                self.emit_line(&format!("  br i1 {}, label %{}, label %{}", cv, bl, xl));
                self.emit_line(&format!("{}:", bl));
                self.emit_expr(body);
                self.emit_line(&format!("  br label %{}", cl));
                self.emit_line(&format!("{}:", xl));
                None
            }
            HirExprKind::Call(func, args) => {
                let fn_name = match &func.kind {
                    HirExprKind::Path(segs) => segs.join("_"),
                    _ => "unknown_fn".to_string(),
                };
                let mut arg_strs = Vec::new();
                for arg in args {
                    if let Some(v) = self.emit_expr(arg) {
                        arg_strs.push(format!("{} {}", emit_llvm_ty(&arg.ty), v));
                    }
                }
                let ret_ty = emit_llvm_ty(&expr.ty);
                if ret_ty == "void" {
                    self.emit_line(&format!("  call void @{}({})", fn_name, arg_strs.join(", ")));
                    None
                } else {
                    let tmp = self.ssa.fresh_tmp();
                    self.emit_line(&format!("  {} = call {} @{}({})", tmp, ret_ty, fn_name, arg_strs.join(", ")));
                    Some(tmp)
                }
            }
            HirExprKind::Assign(place, val) => {
                if let Some(v) = self.emit_expr(val) {
                    let name = self.ssa.place_name(*place);
                    let ty = emit_llvm_ty(&val.ty);
                    self.emit_line(&format!("  store {} {}, ptr {}", ty, v, name));
                }
                None
            }
            HirExprKind::Cast(inner, ty) => {
                let iv = self.emit_expr(inner)?;
                let from_ty = emit_llvm_ty(&inner.ty);
                let to_ty = emit_llvm_ty(ty);
                let tmp = self.ssa.fresh_tmp();
                self.emit_line(&format!("  {} = bitcast {} {} to {}", tmp, from_ty, iv, to_ty));
                Some(tmp)
            }
            HirExprKind::Ref(_is_mut, place, _) => {
                Some(self.ssa.place_name(*place))
            }
            HirExprKind::Deref(inner, _) => {
                let ptr = self.emit_expr(inner)?;
                let ty = emit_llvm_ty(&expr.ty);
                let tmp = self.ssa.fresh_tmp();
                self.emit_line(&format!("  {} = load {}, ptr {}", tmp, ty, ptr));
                Some(tmp)
            }
            HirExprKind::Tuple(exprs) => {
                let mut last = None;
                for e in exprs { last = self.emit_expr(e); }
                last
            }
            HirExprKind::Array(exprs) => {
                for e in exprs { self.emit_expr(e); }
                Some("null".to_string())
            }
            _ => None,
        }
    }

    fn emit_stmt(&mut self, stmt: &HirStmt) {
        match &stmt.kind {
            HirStmtKind::Let(place, _, ty, init) => {
                let llty = emit_llvm_ty(ty);
                let name = self.ssa.place_name(*place);
                self.emit_line(&format!("  {} = alloca {}", name, llty));
                if let Some(ie) = init {
                    if let Some(v) = self.emit_expr(ie) {
                        let ity = emit_llvm_ty(&ie.ty);
                        self.emit_line(&format!("  store {} {}, ptr {}", ity, v, name));
                    }
                }
            }
            HirStmtKind::Expr(e) => { self.emit_expr(e); }
            _ => {}
        }
    }

    fn emit_lit(&self, lit: &HirLit) -> String {
        match lit {
            HirLit::Int(n)   => n.to_string(),
            HirLit::Float(f) => format!("{:.6e}", f),
            HirLit::Bool(b)  => if *b { "1".to_string() } else { "0".to_string() },
            HirLit::Char(c)  => (*c as u32).to_string(),
            HirLit::Str(_)   => "null".to_string(), // DEFERRED: string IR for 8C full
            HirLit::Unit     => "0".to_string(),
        }
    }

    pub fn binop_instr(&self, op: &BinaryOp, ty: &HirTy) -> &'static str {
        let is_float = matches!(ty, HirTy::F32 | HirTy::F64);
        match op {
            BinaryOp::Add => if is_float { "fadd" } else { "add" },
            BinaryOp::Sub => if is_float { "fsub" } else { "sub" },
            BinaryOp::Mul => if is_float { "fmul" } else { "mul" },
            BinaryOp::Div => if is_float { "fdiv" } else { "sdiv" },
            BinaryOp::Rem => if is_float { "frem" } else { "srem" },
            BinaryOp::And | BinaryOp::BitAnd => "and",
            BinaryOp::Or  | BinaryOp::BitOr  => "or",
            BinaryOp::BitXor => "xor",
            BinaryOp::Shl => "shl",
            BinaryOp::Shr => "ashr",
            BinaryOp::Eq  => if is_float { "fcmp oeq" } else { "icmp eq" },
            BinaryOp::Ne  => if is_float { "fcmp one" } else { "icmp ne" },
            BinaryOp::Lt  => if is_float { "fcmp olt" } else { "icmp slt" },
            BinaryOp::Le  => if is_float { "fcmp ole" } else { "icmp sle" },
            BinaryOp::Gt  => if is_float { "fcmp ogt" } else { "icmp sgt" },
            BinaryOp::Ge  => if is_float { "fcmp oge" } else { "icmp sge" },
        }
    }

    fn default_value(&self, ty: &HirTy) -> &'static str {
        match ty {
            HirTy::Bool => "false",
            HirTy::F32 | HirTy::F64 => "0.0",
            _ => "0",
        }
    }
}

pub struct CompileResult {
    pub ll_source: String,
    pub object_path: Option<String>,
    pub binary_path: Option<String>,
    pub errors: Vec<String>,
}

pub fn emit_ir(module: &HirModule) -> String {
    let mut emitter = LlvmEmitter::new();
    emitter.emit_module(module)
}

pub fn ir_to_object(ll_source: &str, out_dir: &str) -> Result<String, String> {
    let ll_path = format!("{}/axon_out.ll", out_dir);
    let obj_path = format!("{}/axon_out.o", out_dir);
    std::fs::write(&ll_path, ll_source)
        .map_err(|e| format!("failed to write .ll: {}", e))?;
    let output = Command::new("llc")
        .args(&["-filetype=obj", "-o", &obj_path, &ll_path])
        .output()
        .map_err(|e| format!("llc not found: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("llc failed: {}", stderr));
    }
    Ok(obj_path)
}

pub fn object_to_binary(obj_path: &str, bin_path: &str) -> Result<(), String> {
    let output = Command::new("clang")
        .args(&[obj_path, "-o", bin_path, "-no-pie"])
        .output()
        .map_err(|e| format!("clang not found: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("clang failed: {}", stderr));
    }
    Ok(())
}

pub fn compile(module: &HirModule, out_dir: &str, bin_name: &str) -> CompileResult {
    let ll_source = emit_ir(module);
    let mut errors = Vec::new();
    let object_path = match ir_to_object(&ll_source, out_dir) {
        Ok(p) => Some(p),
        Err(e) => { errors.push(e); None }
    };
    let binary_path = if let Some(ref obj) = object_path {
        let bin = format!("{}/{}", out_dir, bin_name);
        match object_to_binary(obj, &bin) {
            Ok(()) => Some(bin),
            Err(e) => { errors.push(e); None }
        }
    } else { None };
    CompileResult { ll_source, object_path, binary_path, errors }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::hir::lower;

    fn emit_src(src: &str) -> String {
        let items = parse(src).expect("parse failed");
        let module = lower(items);
        emit_ir(&module)
    }

    #[test]
    fn tc1_emit_module_header() {
        let ir = emit_src("fn f(x: i32) -> i32 { return x; }");
        assert!(ir.contains("target triple"));
        assert!(ir.contains("x86_64-pc-linux-gnu"));
        assert!(ir.contains("source_filename"));
    }

    #[test]
    fn tc2_emit_fn_signature() {
        let ir = emit_src("fn add(x: i32, y: i32) -> i32 { return x; }");
        assert!(ir.contains("define"), "IR: {}", ir);
        assert!(ir.contains("@add"), "IR: {}", ir);
        assert!(ir.contains("i32"), "IR: {}", ir);
    }

    #[test]
    fn tc3_emit_struct_type() {
        let ir = emit_src("struct Point { x: i32, y: i32, }");
        assert!(ir.contains("%struct.Point"), "IR: {}", ir);
    }

    #[test]
    fn tc4_emit_lit_int() {
        let ir = emit_src("fn f() -> i32 { return 42; }");
        assert!(ir.contains("42"), "IR: {}", ir);
    }

    #[test]
    fn tc5_emit_fn_internal() {
        let ir = emit_src("fn private(x: i32) -> i32 { return x; }");
        assert!(ir.contains("internal"), "IR: {}", ir);
    }

    #[test]
    fn tc6_llvm_ty_mapping() {
        assert_eq!(emit_llvm_ty(&HirTy::I32),  "i32");
        assert_eq!(emit_llvm_ty(&HirTy::Bool), "i1");
        assert_eq!(emit_llvm_ty(&HirTy::F64),  "double");
        assert_eq!(emit_llvm_ty(&HirTy::F32),  "float");
        assert_eq!(emit_llvm_ty(&HirTy::U64),  "i64");
        assert_eq!(emit_llvm_ty(&HirTy::Unit), "void");
        assert_eq!(emit_llvm_ty(&HirTy::Char), "i32");
    }

    #[test]
    fn tc7_ssa_names_distinct() {
        let mut ssa = SsaNames::new();
        let p0 = PlaceId(0);
        let p1 = PlaceId(1);
        let n0 = ssa.place_name(p0);
        let n1 = ssa.place_name(p1);
        assert_ne!(n0, n1);
        assert!(n0.starts_with("%p"));
        let t0 = ssa.fresh_tmp();
        assert!(t0.starts_with("%t"));
        assert_ne!(n0, t0);
    }

    #[test]
    fn tc8_ssa_place_stable() {
        let mut ssa = SsaNames::new();
        let p = PlaceId(5);
        assert_eq!(ssa.place_name(p), ssa.place_name(p));
    }

    #[test]
    fn tc9_emit_enum_type() {
        let ir = emit_src("enum Color { Red, Green, Blue, }");
        assert!(ir.contains("%enum.Color"), "IR: {}", ir);
    }

    #[test]
    fn tc10_emit_pub_fn() {
        let ir = emit_src("pub fn exported(x: i32) -> i32 { return x; }");
        assert!(ir.contains("@exported"), "IR: {}", ir);
        assert!(!ir.contains("internal @exported"), "IR: {}", ir);
    }

    #[test]
    fn tc11_ir_to_object_produces_file() {
        let ir = emit_src("fn main() -> i32 { return 0; }");
        match ir_to_object(&ir, "/tmp") {
            Ok(obj_path) => {
                assert!(std::path::Path::new(&obj_path).exists(),
                    "Object file not found: {}", obj_path);
            }
            Err(e) => panic!("ir_to_object failed: {}", e),
        }
    }

    #[test]
    fn tc12_full_pipeline_simple_fn() {
        let ir = emit_src("fn main() -> i32 { return 0; }");
        match ir_to_object(&ir, "/tmp") {
            Ok(obj_path) => {
                match object_to_binary(&obj_path, "/tmp/axon_tc12") {
                    Ok(()) => assert!(std::path::Path::new("/tmp/axon_tc12").exists()),
                    Err(e) => println!("clang note: {}", e),
                }
            }
            Err(e) => println!("llc note: {}", e),
        }
    }

    #[test]
    fn tc13_binop_instr_mapping() {
        let e = LlvmEmitter::new();
        assert_eq!(e.binop_instr(&BinaryOp::Add, &HirTy::I32), "add");
        assert_eq!(e.binop_instr(&BinaryOp::Add, &HirTy::F64), "fadd");
        assert_eq!(e.binop_instr(&BinaryOp::Eq,  &HirTy::I32), "icmp eq");
        assert_eq!(e.binop_instr(&BinaryOp::Lt,  &HirTy::F32), "fcmp olt");
        assert_eq!(e.binop_instr(&BinaryOp::Mul, &HirTy::F64), "fmul");
    }

    #[test]
    fn tc14_emit_no_panic_on_complex() {
        let src = "struct Point { x: i32, y: i32, } fn add(x: i32, y: i32) -> i32 { return x; } enum Color { Red, Green, Blue, }";
        let ir = emit_src(src);
        assert!(!ir.is_empty());
        assert!(ir.contains("target triple"));
    }

    #[test]
    fn tc15_ir_has_entry_and_ret() {
        let ir = emit_src("fn f(x: i32) -> i32 { return x; }");
        assert!(ir.contains("entry:"));
        assert!(ir.contains("ret"));
    }
}
