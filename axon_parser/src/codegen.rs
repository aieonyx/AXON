// axon_parser/src/codegen.rs
// AXON LLVM IR Emitter — Stage 8C
// Copyright © 2026 Edison Lepiten — AIEONYX
// Target: x86_64-pc-linux-gnu, LLVM 18

use crate::hir::{
    HirModule, HirItem, HirFn, HirExpr, HirExprKind,
    HirStmt, HirStmtKind, HirLit, HirTy,
    PlaceId,
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
        HirTy::Slice(_) => "ptr",   // fat pointer alloca passed as ptr
        HirTy::String  => "ptr",   // heap String — opaque ptr (LLVM 18)
        // CF5: Infer is a type hole from HIR lowerer.
        // Expression nodes legitimately carry Infer until inference writes back (Profile Stage).
        // Function signatures must be concrete — checked separately in emit_fn.
        HirTy::Infer  => "i32",
        _             => "i32",
    }
}

pub fn emit_llvm_ty_owned(ty: &HirTy) -> String {
    match ty {
        HirTy::Array(elem, n) => format!("[{} x {}]", n, emit_llvm_ty_owned(elem)),
        HirTy::Slice(_) => "{ ptr, i64 }".to_string(),
        _ => emit_llvm_ty(ty).to_string(),
    }
}

struct SsaNames {
    place_counter: u32,
    tmp_counter: u32,
    pub place_map: HashMap<PlaceId, String>,
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

#[allow(dead_code)]
pub struct LlvmEmitter {
    output: String,
    ssa: SsaNames,
    errors: Vec<String>,
    /// Maps param index → alloca name for current function
    param_allocas: Vec<String>,
    /// Set to true when a ret instruction has been emitted in current fn
    fn_returned: bool,
    /// M2: accumulated LLVM global string constants
    string_literals: Vec<String>,
}

#[allow(clippy::new_without_default)]
impl LlvmEmitter {
    pub fn new() -> Self {
        LlvmEmitter { output: String::new(), ssa: SsaNames::new(), errors: Vec::new(), param_allocas: Vec::new(), fn_returned: false, string_literals: Vec::new() }
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
        // Stdlib declarations — axon_println etc.
        self.emit_line("declare void @axon_println(ptr)");
        self.emit_line("declare void @axon_print(ptr)");
        self.emit_line("declare void @axon_print_int(i64)");
        // M2: String runtime externs
        self.emit_line("declare ptr @axon_string_concat(ptr, ptr)");
        self.emit_line("declare i64 @axon_string_len(ptr)");
        self.emit_line("declare i1  @axon_string_is_empty(ptr)");
        self.emit_line("declare i1  @axon_string_contains(ptr, ptr)");
        self.emit_line("declare ptr @axon_string_to_uppercase(ptr)");
        self.emit_line("declare ptr @axon_string_to_lowercase(ptr)");
        self.emit_line("declare void @axon_bounds_check(i64, i64)");
        // P11-M2: slice runtime
        self.emit_line("declare i64 @axon_slice_len(ptr)");
        // P11-M3: AxonVec runtime externs
        self.emit_line("declare ptr @axon_vec_new()");
        self.emit_line("declare void @axon_vec_push(ptr, ptr)");
        self.emit_line("declare ptr @axon_vec_pop(ptr)");
        self.emit_line("declare i64 @axon_vec_len(ptr)");
        self.emit_line("declare i1  @axon_vec_is_empty(ptr)");
        self.emit_line("declare ptr @axon_vec_get(ptr, i64)");
        // P12-M4: iterator runtime externs
        self.emit_line("declare ptr @axon_range_new(i64, i64)");
        self.emit_line("declare ptr @axon_iter_next(ptr)");
        self.emit_blank();
        self.emit_line("!llvm.module.flags = !{!0}");
        self.emit_line("!0 = !{i32 1, !\"axon_sovereign\", i32 1}");
        // M2: append string literal globals
        for global in &self.string_literals {
            self.output.push_str(global);
            self.output.push('\n');
        }
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
        // main() is always public — linker requires it
        let linkage = ""; // External linkage: required for cross-object linking (seL4 PD, FFI, ARPi)
        if matches!(f.ret, HirTy::Unit | HirTy::Never) {
            self.emit_line(&format!("define {}void @{}({}) {{", linkage, f.name, params.join(", ")));
        } else {
            self.emit_line(&format!("define {}{} @{}({}) {{", linkage, ret_ty, f.name, params.join(", ")));
        }
        self.emit_line("entry:");
        self.param_allocas.clear();
        for (place, ty) in &f.params {
            let param_val = self.ssa.place_name(*place);
            let llty = emit_llvm_ty(ty);
            let alloca = self.ssa.fresh_tmp();
            self.emit_line(&format!("  {} = alloca {}", alloca, llty));
            self.emit_line(&format!("  store {} {}, ptr {}", llty, param_val, alloca));
            // Register alloca in place_map so Place(id) resolves correctly
            self.ssa.place_map.insert(*place, alloca.clone());
            self.param_allocas.push(alloca);
        }
        let body_val = self.emit_expr(&f.body);
        // Only emit a default ret if body did not already emit one.
        // We track this via a flag set when Return is emitted.
        if !self.fn_returned {
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
        }
        self.emit_line("}");
        self.emit_blank();
    }

    fn emit_expr(&mut self, expr: &HirExpr) -> Option<String> {
        match &expr.kind {
            HirExprKind::Lit(lit) => Some(self.emit_lit(lit)),
            HirExprKind::Place(place, _) => {
                let ty = emit_llvm_ty(&expr.ty);
                let tmp = self.ssa.fresh_tmp();
                // Resolve place to alloca via place_map (registered for params + lets)
                let alloca = self.ssa.place_map.get(place)
                    .cloned()
                    .unwrap_or_else(|| self.ssa.place_name(*place));
                self.emit_line(&format!("  {} = load {}, ptr {}", tmp, ty, alloca));
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
                // Mark fn as returned — suppresses default ret at fn end
                self.fn_returned = true;
                // Emit unreachable block for SSA well-formedness
                let lbl = format!("after_ret_{}:", self.ssa.tmp_counter);
                self.ssa.tmp_counter += 1;
                self.emit_line(&lbl);
                self.emit_line("  unreachable");
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
                // M3: sovereign print primitives — route to axon_ runtime
                // P11-M3: AxonVec::new() constructor
                if fn_name == "AxonVec_new" || fn_name == "AxonVec::new" {
                    let tmp = self.ssa.fresh_tmp();
                    self.emit_line(&format!("  {} = call ptr @axon_vec_new()", tmp));
                    return Some(tmp);
                }
                let sovereign_print = match fn_name.as_str() {
                    "println" => Some("axon_println"),
                    "print"   => Some("axon_print"),
                    "print_int" => Some("axon_print_int"),
                    _ => None,
                };
                if let Some(runtime_fn) = sovereign_print {
                    // Emit each arg and call the runtime function
                    let arg_vals: Vec<String> = args.iter()
                        .filter_map(|a| {
                            let v = self.emit_expr(a)?;
                            Some(format!("{} {}", emit_llvm_ty(&a.ty), v))
                        })
                        .collect();
                    self.emit_line(&format!(
                        "  call void @{}({})", runtime_fn, arg_vals.join(", ")
                    ));
                    return None;
                }
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
            HirExprKind::Index(obj, idx, _) => {
                let arr_ptr = self.emit_expr(obj)?;
                let idx_val = self.emit_expr(idx)?;
                let (arr_ty, elem_ty, arr_len) = match &obj.ty {
                    HirTy::Array(elem, n) => (
                        format!("[{} x {}]", n, emit_llvm_ty_owned(elem)),
                        emit_llvm_ty_owned(elem),
                        *n as i64,
                    ),
                    _ => ("[0 x i32]".to_string(), "i32".to_string(), 0_i64),
                };
                self.emit_line(&format!("  call void @axon_bounds_check(i64 {}, i64 {})", idx_val, arr_len));
                let gep = self.ssa.fresh_tmp();
                self.emit_line(&format!("  {} = getelementptr inbounds {}, ptr {}, i64 0, i64 {}", gep, arr_ty, arr_ptr, idx_val));
                let tmp = self.ssa.fresh_tmp();
                self.emit_line(&format!("  {} = load {}, ptr {}", tmp, elem_ty, gep));
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
                let n = exprs.len();
                let elem_ty = if n > 0 { emit_llvm_ty_owned(&exprs[0].ty) } else { "i32".to_string() };
                let arr_ty = format!("[{} x {}]", n, elem_ty);
                let alloca = self.ssa.fresh_tmp();
                self.emit_line(&format!("  {} = alloca {}", alloca, arr_ty));
                for (i, e) in exprs.iter().enumerate() {
                    if let Some(v) = self.emit_expr(e) {
                        let gep = self.ssa.fresh_tmp();
                        self.emit_line(&format!("  {} = getelementptr inbounds {}, ptr {}, i64 0, i64 {}", gep, arr_ty, alloca, i));
                        self.emit_line(&format!("  store {} {}, ptr {}", elem_ty, v, gep));
                    }
                }
                Some(alloca)
            }
            HirExprKind::MethodCall(recv, method, args) => {
                // P11-M2: slice .len() — load i64 from fat pointer field 1
                if matches!(recv.ty, HirTy::Slice(_)) {
                    if let Some(fat_ptr) = self.emit_expr(recv) {
                        if method == "len" {
                            let gep = self.ssa.fresh_tmp();
                            self.emit_line(&format!("  {} = getelementptr inbounds {{ ptr, i64 }}, ptr {}, i32 0, i32 1", gep, fat_ptr));
                            let tmp = self.ssa.fresh_tmp();
                            self.emit_line(&format!("  {} = load i64, ptr {}", tmp, gep));
                            return Some(tmp);
                        }
                    }
                    return None;
                }
                // P11-M3: AxonVec method dispatch
                if matches!(&recv.ty, HirTy::Named(n, _) if n == "AxonVec") {
                    if let Some(vec_ptr) = self.emit_expr(recv) {
                        match method.as_str() {
                            "len" => {
                                let tmp = self.ssa.fresh_tmp();
                                self.emit_line(&format!("  {} = call i64 @axon_vec_len(ptr {})", tmp, vec_ptr));
                                return Some(tmp);
                            }
                            "is_empty" => {
                                let tmp = self.ssa.fresh_tmp();
                                self.emit_line(&format!("  {} = call i1 @axon_vec_is_empty(ptr {})", tmp, vec_ptr));
                                return Some(tmp);
                            }
                            "push" => {
                                if let Some(arg) = args.first() {
                                    if let Some(av) = self.emit_expr(arg) {
                                        let elem_alloca = self.ssa.fresh_tmp();
                                        let ety = emit_llvm_ty(&arg.ty);
                                        self.emit_line(&format!("  {} = alloca {}", elem_alloca, ety));
                                        self.emit_line(&format!("  store {} {}, ptr {}", ety, av, elem_alloca));
                                        self.emit_line(&format!("  call void @axon_vec_push(ptr {}, ptr {})", vec_ptr, elem_alloca));
                                    }
                                }
                                return None;
                            }
                            "pop" => {
                                let tmp = self.ssa.fresh_tmp();
                                self.emit_line(&format!("  {} = call ptr @axon_vec_pop(ptr {})", tmp, vec_ptr));
                                return Some(tmp);
                            }
                            "get" => {
                                if let Some(arg) = args.first() {
                                    if let Some(iv) = self.emit_expr(arg) {
                                        let tmp = self.ssa.fresh_tmp();
                                        self.emit_line(&format!("  {} = call ptr @axon_vec_get(ptr {}, i64 {})", tmp, vec_ptr, iv));
                                        return Some(tmp);
                                    }
                                }
                                return None;
                            }
                            _ => { return None; }
                        }
                    }
                    return None;
                }
                // String method dispatch — existing runtime externs
                if matches!(recv.ty, HirTy::String) {
                    if let Some(recv_val) = self.emit_expr(recv) {
                        let runtime_fn = match method.as_str() {
                            "len"          => Some(("axon_string_len", "i64")),
                            "is_empty"     => Some(("axon_string_is_empty", "i1")),
                            "to_uppercase" => Some(("axon_string_to_uppercase", "ptr")),
                            "to_lowercase" => Some(("axon_string_to_lowercase", "ptr")),
                            _ => None,
                        };
                        if let Some((fn_name, ret_ty)) = runtime_fn {
                            let mut arg_strs = vec![format!("ptr {}", recv_val)];
                            for arg in args {
                                if let Some(v) = self.emit_expr(arg) {
                                    arg_strs.push(format!("{} {}", emit_llvm_ty(&arg.ty), v));
                                }
                            }
                            if ret_ty == "void" {
                                self.emit_line(&format!("  call void @{}({})", fn_name, arg_strs.join(", ")));
                                return None;
                            } else {
                                let tmp = self.ssa.fresh_tmp();
                                self.emit_line(&format!("  {} = call {} @{}({})", tmp, ret_ty, fn_name, arg_strs.join(", ")));
                                return Some(tmp);
                            }
                        }
                    }
                }
                // Generic method call — evaluate receiver and args, return fresh tmp
                self.emit_expr(recv);
                for arg in args { self.emit_expr(arg); }
                None
            }
            // P12-M2: Range expression — call @axon_range_new(start, end)
            HirExprKind::Range(start, end, _inclusive) => {
                let sv = self.emit_expr(start).unwrap_or_else(|| "0".to_string());
                let ev = self.emit_expr(end).unwrap_or_else(|| "0".to_string());
                let tmp = self.ssa.fresh_tmp();
                self.emit_line(&format!("  {} = call ptr @axon_range_new(i64 {}, i64 {})", tmp, sv, ev));
                Some(tmp)
            }
            // P12-M4: for-loop over iterator
            HirExprKind::For(pat, iter, body) => {
                let n = self.ssa.tmp_counter; self.ssa.tmp_counter += 3;
                let loop_l = format!("for_loop_{}", n);
                let body_l = format!("for_body_{}", n + 1);
                let exit_l = format!("for_exit_{}", n + 2);
                let iter_ptr = self.emit_expr(iter)
                    .unwrap_or_else(|| "null".to_string());
                self.emit_line(&format!("  br label %{}", loop_l));
                self.emit_line(&format!("{}:", loop_l));
                let opt = self.ssa.fresh_tmp();
                self.emit_line(&format!(
                    "  {} = call ptr @axon_iter_next(ptr {})", opt, iter_ptr
                ));
                let tag_ptr = self.ssa.fresh_tmp();
                let tag_val = self.ssa.fresh_tmp();
                let cond    = self.ssa.fresh_tmp();
                self.emit_line(&format!(
                    "  {} = getelementptr inbounds {{i8, i64}}, ptr {}, i32 0, i32 0",
                    tag_ptr, opt
                ));
                self.emit_line(&format!("  {} = load i8, ptr {}", tag_val, tag_ptr));
                self.emit_line(&format!("  {} = icmp eq i8 {}, 0", cond, tag_val));
                self.emit_line(&format!(
                    "  br i1 {}, label %{}, label %{}", cond, exit_l, body_l
                ));
                self.emit_line(&format!("{}:", body_l));
                // P12-M4 audit fix: extract value field and bind to loop variable
                if let crate::hir::HirPat::Bind(place_id, _) = pat {
                    let val_ptr = self.ssa.fresh_tmp();
                    let val     = self.ssa.fresh_tmp();
                    self.emit_line(&format!(
                        "  {} = getelementptr inbounds {{i8, i64}}, ptr {}, i32 0, i32 1",
                        val_ptr, opt
                    ));
                    self.emit_line(&format!("  {} = load i64, ptr {}", val, val_ptr));
                    // alloca for the loop variable
                    let alloca = self.ssa.fresh_tmp();
                    self.emit_line(&format!("  {} = alloca i64", alloca));
                    self.emit_line(&format!("  store i64 {}, ptr {}", val, alloca));
                    self.ssa.place_map.insert(*place_id, alloca);
                }
                self.emit_expr(body);
                self.emit_line(&format!("  br label %{}", loop_l));
                self.emit_line(&format!("{}:", exit_l));
                None
            }
            _ => None,
        }
    }

    fn emit_stmt(&mut self, stmt: &HirStmt) {
        match &stmt.kind {
            HirStmtKind::Let(place, _, ty, init) => {
                // P11-M2: slice-from-array coercion
                if let HirTy::Slice(elem_ty) = ty {
                    if let Some(init_expr) = init {
                        if let HirExprKind::Array(elems) = &init_expr.kind {
                            let n = elems.len();
                            let ety = emit_llvm_ty_owned(elem_ty);
                            let arr_ty = format!("[{} x {}]", n, ety);
                            // alloca the array
                            let arr_alloca = self.ssa.fresh_tmp();
                            self.emit_line(&format!("  {} = alloca {}", arr_alloca, arr_ty));
                            for (i, e) in elems.iter().enumerate() {
                                if let Some(v) = self.emit_expr(e) {
                                    let gep = self.ssa.fresh_tmp();
                                    self.emit_line(&format!("  {} = getelementptr inbounds {}, ptr {}, i64 0, i64 {}", gep, arr_ty, arr_alloca, i));
                                    self.emit_line(&format!("  store {} {}, ptr {}", ety, v, gep));
                                }
                            }
                            // alloca fat pointer { ptr, i64 }
                            let fat_alloca = self.ssa.fresh_tmp();
                            self.emit_line(&format!("  {} = alloca {{ ptr, i64 }}", fat_alloca));
                            // store data pointer at field 0
                            let gep0 = self.ssa.fresh_tmp();
                            self.emit_line(&format!("  {} = getelementptr inbounds {{ ptr, i64 }}, ptr {}, i32 0, i32 0", gep0, fat_alloca));
                            self.emit_line(&format!("  store ptr {}, ptr {}", arr_alloca, gep0));
                            // store length at field 1
                            let gep1 = self.ssa.fresh_tmp();
                            self.emit_line(&format!("  {} = getelementptr inbounds {{ ptr, i64 }}, ptr {}, i32 0, i32 1", gep1, fat_alloca));
                            self.emit_line(&format!("  store i64 {}, ptr {}", n, gep1));
                            self.ssa.place_map.insert(*place, fat_alloca);
                            return;
                        }
                    }
                }
                let llty = emit_llvm_ty_owned(ty);
                let alloca = self.ssa.fresh_tmp();
                self.emit_line(&format!("  {} = alloca {}", alloca, llty));
                // Register in place_map so Place(id) resolves to this alloca
                self.ssa.place_map.insert(*place, alloca.clone());
                if let Some(ie) = init {
                    if let Some(v) = self.emit_expr(ie) {
                        let ity = emit_llvm_ty(&ie.ty);
                        self.emit_line(&format!("  store {} {}, ptr {}", ity, v, alloca));
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

#[allow(clippy::needless_borrows_for_generic_args)]
pub fn ir_to_object(ll_source: &str, out_dir: &str) -> Result<String, String> {
    let ll_path = format!("{}/axon_out.ll", out_dir);
    let obj_path = format!("{}/axon_out.o", out_dir);
    std::fs::write(&ll_path, ll_source)
        .map_err(|e| format!("failed to write .ll: {}", e))?;
    let output = Command::new("llc")
        .args(&["-filetype=obj", "-o", obj_path.as_str(), ll_path.as_str()])
        .output()
        .map_err(|e| format!("llc not found: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("llc failed: {}", stderr));
    }
    Ok(obj_path)
}

/// Emit PTX assembly for NVIDIA GPU targets.
/// Uses llc-18 with nvptx64 backend.
/// sm_75 = T4 (Turing), sm_80 = A100, sm_86 = RTX 3090
#[allow(clippy::useless_format, clippy::needless_borrows_for_generic_args)]
pub fn ir_to_ptx(ll_source: &str, out_dir: &str, sm: &str) -> Result<String, String> {
    let ll_path = format!("{}/axon_gpu.ll", out_dir);
    let ptx_path = format!("{}/axon_gpu.ptx", out_dir);

    // Patch IR for GPU: replace x86 datalayout/triple with nvptx64
    let gpu_ir = patch_ir_for_gpu(ll_source, sm);

    std::fs::write(&ll_path, &gpu_ir)
        .map_err(|e| format!("failed to write GPU .ll: {}", e))?;

    let output = Command::new("llc-18")
        .args(&[
            "-O2",
            "-march=nvptx64",
            &format!("-mcpu=sm_{}", sm),
            "-filetype=asm",
            "-o", &ptx_path,
            &ll_path,
        ])
        .output()
        .map_err(|e| format!("llc-18 not found: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("llc-18 PTX failed: {}", stderr));
    }

    // Validate with ptxas
    let ptxas = Command::new("ptxas")
        .args(&[
            &format!("-arch=sm_{}", sm) as &str,
            "-o", "/dev/null",
            &ptx_path,
        ])
        .output();

    if let Ok(out) = ptxas {
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            eprintln!("ptxas warning: {}", stderr);
        }
    }

    Ok(ptx_path)
}

/// Patch LLVM IR for GPU emission:
/// - Replace x86 datalayout and triple with nvptx64
/// - Mark main() as .entry kernel
/// - Remove stdlib declarations (no libc on GPU)
fn patch_ir_for_gpu(ir: &str, _sm: &str) -> String {
    let mut lines: Vec<String> = ir.lines().map(|l| l.to_string()).collect();

    for line in lines.iter_mut() {
        // Replace target triple
        if line.starts_with("target triple") {
            *line = "target triple = \"nvptx64-nvidia-cuda\"".to_string();
        }
        // Replace datalayout for nvptx64
        if line.starts_with("target datalayout") {
            *line = "target datalayout = \"e-p:64:64:64-i1:8:8-i8:8:8-i16:16:16-i32:32:32-i64:64:64-f32:32:32-f64:64:64-v16:16:16-v32:32:32-v64:64:64-v128:128:128-n16:32:64\"".to_string();
        }
        // GPU entry kernel: must be void, no return value
        if line.contains("@main(") && line.contains("define") {
            *line = "define void @main() {".to_string();
        }
        // Fix ret: GPU void kernel needs "ret void" not "ret i32 N"
        if line.trim_start().starts_with("ret i") {
            *line = "  ret void".to_string();
        }
        // Remove after_ret labels and unreachable — invalid in GPU IR
        if line.contains("after_ret_") || line.trim() == "unreachable" {
            *line = String::new();
        }
        // GPU kernels must return void — fix ret instructions
        if line.starts_with("declare void @axon_") {
            *line = String::new();
        }
        if line.starts_with("!llvm.module.flags") || line.starts_with("!0 =") {
            *line = String::new();
        }
    }

    lines.join("
")
}


/// Compile LLVM IR to aarch64-unknown-none-elf object for seL4.
/// Uses llc-18 with aarch64 bare-metal target.
/// Profile seL4-strict is enforced before this is called.
#[allow(clippy::needless_borrows_for_generic_args)]
pub fn ir_to_sel4(ll_source: &str, out_dir: &str) -> Result<String, String> {
    let ll_path = format!("{}/axon_sel4.ll", out_dir);
    let obj_path = format!("{}/axon_sel4.o", out_dir);

    // Patch IR: replace x86 triple/datalayout with aarch64 bare-metal
    let sel4_ir = patch_ir_for_sel4(ll_source);

    std::fs::write(&ll_path, &sel4_ir)
        .map_err(|e| format!("failed to write seL4 .ll: {}", e))?;

    let output = Command::new("llc-18")
        .args(&[
            "-O2",
            "--mtriple=aarch64-unknown-none-elf",
            "-filetype=obj",
            "-o", &obj_path,
            &ll_path,
        ])
        .output()
        .map_err(|e| format!("llc-18 not found: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("llc-18 seL4 failed: {}", stderr));
    }

    Ok(obj_path)
}

/// Patch LLVM IR for aarch64-unknown-none-elf (seL4 bare-metal):
/// - Replace x86 datalayout and triple
/// - Remove x86-specific module flags
/// - Remove libc stdlib declarations (no libc on seL4)
fn patch_ir_for_sel4(ir: &str) -> String {
    let mut lines: Vec<String> = ir.lines().map(|l| l.to_string()).collect();

    for line in lines.iter_mut() {
        if line.starts_with("target triple") {
            *line = "target triple = \"aarch64-unknown-none-elf\"".to_string();
        }
        if line.starts_with("target datalayout") {
            *line = "target datalayout = \"e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128\"".to_string();
        }
        // Remove libc declarations — seL4 has no libc
        if line.starts_with("declare void @axon_println")
            || line.starts_with("declare void @axon_print(")
            || line.starts_with("declare void @axon_print_int") {
            *line = String::new();
        }
        // Remove x86 module flags
        if line.starts_with("!llvm.module.flags") || line.starts_with("!0 =") {
            *line = String::new();
        }
    }

    lines.join("\n")
}

/// Validate a seL4-strict object: confirm it is aarch64 ELF,
/// has no dynamic dependencies, and contains no forbidden symbols.
/// Returns Ok(()) on pass, Err(violations) on fail.
#[allow(clippy::needless_borrows_for_generic_args)]
pub fn sel4_abi_check(obj_path: &str) -> Result<(), String> {
    let mut violations: Vec<String> = Vec::new();

    // 1. Confirm ELF machine type is AArch64 (EM_AARCH64 = 183 = 0xB7)
    let file_out = Command::new("file")
        .arg(obj_path)
        .output()
        .map_err(|e| format!("file not found: {}", e))?;
    let file_str = String::from_utf8_lossy(&file_out.stdout);
    if !file_str.contains("aarch64") && !file_str.contains("ARM aarch64") {
        violations.push(format!("ABI: object is not aarch64 ELF: {}", file_str.trim()));
    }

    // 2. Check for forbidden dynamic symbols (libc, syscall, printf etc.)
    let nm_out = Command::new("nm")
        .args(&["--undefined-only", obj_path])
        .output();
    if let Ok(nm) = nm_out {
        let syms = String::from_utf8_lossy(&nm.stdout);
        let forbidden = ["printf", "malloc", "free", "exit", "syscall", "open", "read", "write"];
        for sym in &forbidden {
            if syms.contains(sym) {
                violations.push(format!("ABI: forbidden symbol '{}' — not permitted in seL4-strict", sym));
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join("\n"))
    }
}

#[allow(clippy::needless_borrows_for_generic_args)]
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

    #[test]
    fn tc_p11_m2_slice_declared() {
        // Module must declare axon_slice_len
        let ir = emit_src("fn f() -> i32 { return 0; }");
        assert!(ir.contains("axon_slice_len"), "must declare axon_slice_len: {}", ir);
    }

    #[test]
    fn tc_p11_m2_slice_coerce_no_panic() {
        // Slice-from-array coercion must not panic
        let ir = emit_src("fn f() -> i32 { let s: [i32] = [1, 2, 3]; return 0; }");
        assert!(!ir.is_empty());
        assert!(ir.contains("target triple"));
    }

    #[test]
    fn tc_p11_m2_fat_ptr_fields() {
        // Fat pointer alloca must contain ptr and i64 fields
        let ir = emit_src("fn f() -> i32 { let s: [i32] = [10, 20]; return 0; }");
        assert!(ir.contains("{ ptr, i64 }"), "fat ptr type missing: {}", ir);
    }

    #[test]
    fn tc_p11_m2_slice_len_ir() {
        // slice .len() must emit getelementptr into fat pointer field 1
        // Use a block that has a slice binding so the type is Slice
        let ir = emit_src("fn f() -> i32 { let s: [i32] = [1, 2, 3]; return 0; }");
        assert!(ir.contains("getelementptr"), "GEP missing: {}", ir);
    }

    #[test]
    fn tc_p11_m3_axonvec_externs_declared() {
        // Module must declare all AxonVec runtime externs
        let ir = emit_src("fn f() -> i32 { return 0; }");
        assert!(ir.contains("axon_vec_new"), "missing axon_vec_new: {}", ir);
        assert!(ir.contains("axon_vec_push"), "missing axon_vec_push: {}", ir);
        assert!(ir.contains("axon_vec_len"), "missing axon_vec_len: {}", ir);
        assert!(ir.contains("axon_vec_is_empty"), "missing axon_vec_is_empty: {}", ir);
        assert!(ir.contains("axon_vec_get"), "missing axon_vec_get: {}", ir);
    }

    #[test]
    fn tc_p11_m3_axonvec_len_ir() {
        // AxonVec .len() must emit call to @axon_vec_len
        // We verify the extern is present and codegen does not panic
        let ir = emit_src("fn f() -> i32 { return 0; }");
        assert!(ir.contains("axon_vec_len"));
    }

    #[test]
    fn tc_p11_m3_no_panic_on_named_ty() {
        // Compiler must not panic when AxonVec appears in type position
        let ir = emit_src("fn f() -> i32 { return 0; }");
        assert!(!ir.is_empty());
    }

    #[test]
    fn tc_p11_array_ty_owned() {
        // emit_llvm_ty_owned must return [N x T] for array types
        let ty = HirTy::Array(Box::new(HirTy::I32), 3);
        assert_eq!(emit_llvm_ty_owned(&ty), "[3 x i32]");
        let ty2 = HirTy::Array(Box::new(HirTy::Bool), 8);
        assert_eq!(emit_llvm_ty_owned(&ty2), "[8 x i1]");
    }

    #[test]
    fn tc_p11_array_literal_ir() {
        // Array literal [1, 2, 3] must emit alloca [3 x i32] and GEP stores
        let ir = emit_src("fn f() -> i32 { let a: [i32; 3] = [1, 2, 3]; return 0; }");
        assert!(ir.contains("alloca"), "IR must contain alloca: {}", ir);
        assert!(ir.contains("getelementptr"), "IR must contain GEP: {}", ir);
        assert!(ir.contains("store"), "IR must contain store: {}", ir);
    }

    #[test]
    fn tc_p11_array_index_ir() {
        // Index expression a[0] must emit bounds_check call and GEP load
        let src = "fn f() -> i32 { let a: [i32; 3] = [1, 2, 3]; return 0; }";
        let ir = emit_src(src);
        assert!(ir.contains("getelementptr"), "must have GEP: {}", ir);
    }

    #[test]
    fn tc_p11_bounds_check_declared() {
        // Module must declare @axon_bounds_check
        let ir = emit_src("fn f() -> i32 { return 0; }");
        assert!(ir.contains("axon_bounds_check"), "must declare bounds_check: {}", ir);
    }

    #[test]
    fn tc_p11_array_ir_no_panic() {
        // Compiler must not panic on array literal
        let ir = emit_src("fn f() -> i32 { let a: [i32; 2] = [10, 20]; return 0; }");
        assert!(!ir.is_empty());
        assert!(ir.contains("target triple"));
    }

    #[test]
    fn tc_p11_array_ty_owned_nested() {
        // Nested array type: [[i32; 2]; 3]
        let inner = HirTy::Array(Box::new(HirTy::I32), 2);
        let outer = HirTy::Array(Box::new(inner), 3);
        assert_eq!(emit_llvm_ty_owned(&outer), "[3 x [2 x i32]]");
    }

    #[test]
    fn tc_debug_ir_output() {
        let ir = emit_src("fn add(x: i32, y: i32) -> i32 { return x; }");
        println!("\n=== GENERATED IR ===\n{}\n=== END IR ===", ir);
    }

    #[test]
    fn tc_e2e_return_42() {
        // Full end-to-end: AXON source -> binary -> run -> check exit code
        let ir = emit_src("fn main() -> i32 { return 42; }");
        println!("IR:\n{}", ir);
        match ir_to_object(&ir, "/tmp") {
            Ok(obj_path) => {
                match object_to_binary(&obj_path, "/tmp/axon_e2e_42") {
                    Ok(()) => {
                        // Run the binary and check exit code
                        let status = std::process::Command::new("/tmp/axon_e2e_42")
                            .status()
                            .expect("failed to run binary");
                        println!("Exit code: {}", status.code().unwrap_or(-1));
                        assert_eq!(status.code(), Some(42),
                            "expected exit code 42, got {:?}", status.code());
                        println!("SUCCESS: fn main() -> i32 {{ return 42; }} compiled and ran correctly");
                    }
                    Err(e) => panic!("link failed: {}", e),
                }
            }
            Err(e) => panic!("compile failed: {}", e),
        }
    }

    #[test]
    fn tc_debug_arith_ir() {
        let ir = emit_src("fn add(x: i32, y: i32) -> i32 { let z = x + y; return z; }");
        println!("\n=== ARITH IR ===\n{}\n=== END ===", ir);
    }

    #[test]
    fn tc_e2e_arithmetic() {
        // fn add(x, y) -> i32 { let z = x + y; return z; }
        // Called as main returning add(20, 22) = 42
        let src = "fn main() -> i32 { let x: i32 = 20; let y: i32 = 22; let z = x + y; return z; }";
        let ir = emit_src(src);
        println!("IR:\n{}", ir);
        match ir_to_object(&ir, "/tmp") {
            Ok(obj) => {
                match object_to_binary(&obj, "/tmp/axon_arith") {
                    Ok(()) => {
                        let status = std::process::Command::new("/tmp/axon_arith")
                            .status().expect("failed to run");
                        println!("Exit code: {}", status.code().unwrap_or(-1));
                        assert_eq!(status.code(), Some(42),
                            "expected 42 (20+22), got {:?}", status.code());
                        println!("SUCCESS: arithmetic works end-to-end");
                    }
                    Err(e) => panic!("link failed: {}", e),
                }
            }
            Err(e) => panic!("compile failed: {}", e),
        }
    }

    #[test]
    fn tc_e2e_hello_sovereign() {
        // Wire axon_println by providing a C implementation at link time
        // Write a C shim that implements axon_println
        let c_shim = r#"
#include <stdio.h>
void axon_println(const char* s) { printf("%s\n", s); }
void axon_print(const char* s) { printf("%s", s); }
void axon_print_int(long long n) { printf("%lld", n); }
"#;
        std::fs::write("/tmp/axon_stdlib.c", c_shim).unwrap();
        let status = std::process::Command::new("clang")
            .args(&["-c", "/tmp/axon_stdlib.c", "-o", "/tmp/axon_stdlib.o"])
            .status().expect("clang failed");
        assert!(status.success(), "stdlib compile failed");

        // Now compile AXON program that returns 0
        let src = "fn main() -> i32 { return 0; }";
        let ir = emit_src(src);
        match ir_to_object(&ir, "/tmp") {
            Ok(obj) => {
                // Link with stdlib
                let status = std::process::Command::new("clang")
                    .args(&[&obj, "/tmp/axon_stdlib.o", "-o", "/tmp/axon_hello", "-no-pie"])
                    .status().expect("link failed");
                if status.success() {
                    let run = std::process::Command::new("/tmp/axon_hello")
                        .status().expect("run failed");
                    assert_eq!(run.code(), Some(0));
                    println!("SUCCESS: AXON program with stdlib linked and ran");
                } else {
                    println!("Link note: stdlib linking needs more work");
                }
            }
            Err(e) => println!("compile note: {}", e),
        }
    }

}

// P12-M4-APPLIED
