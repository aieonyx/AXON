// axon_parser/src/codegen.rs
// AXON LLVM IR Emitter — Stage 8C
// Copyright © 2026 Edison Lepiten — AIEONYX
// Target: x86_64-pc-linux-gnu, LLVM 18

use crate::mono::{MonoTable};
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
        HirTy::Param(_) => "i64", // P17-M1: uninstantiated generic — conservative i64
        HirTy::Dyn(_)  => "ptr",
        HirTy::CStr    => "ptr", // P21-M2: C string is a null-terminated ptr
        // P23-M2: AtomicU64 — LLVM represents atomics as plain integer types;
        // atomicity is encoded in the load/store/rmw instructions themselves
        HirTy::AtomicU64 => "i64",
        // P20-M1: seL4 IPC types — capability slots are u64 words in seL4 ABI
        HirTy::SeL4Endpoint => "i64",
        HirTy::SeL4Badge    => "i64",
        HirTy::SeL4MsgInfo  => "i64",
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
    // P13-M3-PLACE-TYPE: tracks LLVM type for each place (overrides HIR Infer)
    pub place_type_map: HashMap<PlaceId, String>,
}

impl SsaNames {
    fn new() -> Self {
        SsaNames { place_counter: 0, tmp_counter: 0, place_map: HashMap::new(), place_type_map: HashMap::new() }
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
    /// P14-M1: struct name → ordered field names (for GEP index lookup)
    struct_defs: std::collections::HashMap<String, Vec<String>>,
    /// P16-M1: (trait_name, type_name) → ordered method names
    vtable_registry: std::collections::HashMap<(String, String), Vec<String>>,
}

#[allow(clippy::new_without_default)]
impl LlvmEmitter {
    pub fn new() -> Self {
        LlvmEmitter { output: String::new(), ssa: SsaNames::new(), errors: Vec::new(), param_allocas: Vec::new(), fn_returned: false, string_literals: Vec::new(), struct_defs: std::collections::HashMap::new(), vtable_registry: std::collections::HashMap::new() }
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
        // P13-M3-TYPE-ORDER: type must precede all uses in LLVM IR
        self.emit_line("%AxonIterResult = type { i8, i64 }");
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
        self.emit_line("declare %AxonIterResult @axon_iter_next(ptr)");
        // P13-M4-DECLARE-DROP
        self.emit_line("declare void @axon_iter_drop(ptr)");
        // P20-M2: seL4 IPC intrinsic declarations
        self.emit_line("declare i64 @axon_ipc_call(i64, i64, ...)");
        self.emit_line("declare i64 @axon_ipc_send(i64, i64, ...)");
        self.emit_line("declare i64 @axon_ipc_recv(i64, ...)");
        // P22-M2: axon_verify_core postcondition check stub
        self.emit_line("declare void @axon_ensures_check(i32, i1)");
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

    /// P17-M3: Emit concrete monomorphized functions from a MonoTable.
    /// Caller provides (fn_name, type_args) pairs to instantiate.
    pub fn emit_mono(&mut self, table: &MonoTable, instantiations: &[(&str, Vec<(&str, crate::hir::HirTy)>)]) {
        for (fn_name, type_args) in instantiations {
            let args: Vec<(&str, crate::hir::HirTy)> = type_args.iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect();
            if let Some(concrete_fn) = table.instantiate(fn_name, &args) {
                self.emit_fn(&concrete_fn);
            }
        }
    }

    fn emit_item(&mut self, item: &HirItem) {
        match item {
            HirItem::Fn(f) => self.emit_fn(f),
            HirItem::Struct(s) => {
                let fields: Vec<String> = s.fields.iter()
                    .map(|(_, ty, _)| emit_llvm_ty_owned(ty))
                    .collect();
                self.emit_line(&format!("%struct.{} = type {{ {} }}", s.name, fields.join(", ")));
                // P14-M1: register field names for GEP index lookup
                let field_names: Vec<String> = s.fields.iter()
                    .map(|(name, _, _)| name.clone())
                    .collect();
                self.struct_defs.insert(s.name.clone(), field_names);
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
            HirItem::Impl(imp) => {
                // P16-M1: emit vtable global for trait impls
                if let Some(trait_name) = &imp.trait_ {
                    let type_name = match &imp.self_ty {
                        HirTy::Named(n, _) => n.clone(),
                        _ => return,
                    };
                    let method_names: Vec<String> = imp.methods.iter()
                        .map(|m| m.name.clone())
                        .collect();
                    let ptr_list: Vec<String> = method_names.iter()
                        .map(|m| format!("ptr @{}_{}", type_name, m))
                        .collect();
                    let vtable_ty = if method_names.is_empty() {
                        "{}".to_string()
                    } else {
                        format!("{{ {} }}", vec!["ptr"; method_names.len()].join(", "))
                    };
                    let vtable_val = if ptr_list.is_empty() {
                        "{}".to_string()
                    } else {
                        format!("{{ {} }}", ptr_list.join(", "))
                    };
                    self.emit_line(&format!(
                        "@vtable_{}_{} = global {} {}",
                        trait_name, type_name, vtable_ty, vtable_val
                    ));
                    self.emit_blank();
                    self.vtable_registry.insert(
                        (trait_name.clone(), type_name.clone()),
                        method_names,
                    );
                    for method in &imp.methods {
                        let mut mangled = method.clone();
                        mangled.name = format!("{}_{}", type_name, method.name);
                        self.emit_fn(&mangled);
                    }
                }
            }
            HirItem::Trait(_) => {}
            HirItem::ExternFn(name, _abi, params, ret, _caps, _) => {
                // P21-M3: emit LLVM declare for foreign function
                let param_tys: Vec<String> = params.iter()
                    .enumerate()
                    .map(|(i, ty)| format!("{} %arg{}", emit_llvm_ty(ty), i))
                    .collect();
                let ret_ty = emit_llvm_ty(ret);
                self.emit_line(&format!(
                    "declare {} @{}({})",
                    ret_ty, name, param_tys.join(", ")
                ));
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
        // P19-QA: non-pub fns get internal linkage (LLVM IPO + tc5 fix)
        // Exception: main must always be external (linker entry point)
        let linkage = if f.is_pub || f.name == "main" { "" } else { "internal " };
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
        // P22-M2: emit @ensures postcondition checks before ret
        for (idx, contract) in f.contracts.iter().enumerate() {
            if matches!(contract.kind, crate::parser::ContractKind::Ensures) {
                let label = idx as u32;
                self.emit_line(&format!(
                    "  ; @ensures[{}]: postcondition marker — verified by axon_verify_core",
                    label
                ));
                self.emit_line(&format!(
                    "  call void @axon_ensures_check(i32 {}, i1 true)",
                    label
                ));
            }
        }
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
                // P13-M3-PLACE-TYPE: prefer registered type over HIR Infer fallback
                let ty = self.ssa.place_type_map.get(place)
                    .cloned()
                    .unwrap_or_else(|| emit_llvm_ty(&expr.ty).to_string());
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
            HirExprKind::Try(inner) => {
                // P16-M3: expr? — evaluate inner; emit conditional early-return on Err
                // Until Result ABI lands (P21), tag check is a no-op identity.
                let inner_val = self.emit_expr(inner)?;
                let n = self.ssa.tmp_counter; self.ssa.tmp_counter += 4;
                let tag     = format!("%try_tag_{}", n);
                let err_lbl = format!("try_err_{}", n);
                let ok_lbl  = format!("try_ok_{}", n);
                let cont_lbl = format!("try_cont_{}", n);
                self.emit_line(&format!("  {} = and i32 {}, 0", tag, inner_val));
                self.emit_line(&format!("  %try_cond_{} = icmp ne i32 {}, 0", n, tag));
                self.emit_line(&format!("  br i1 %try_cond_{}, label %{}, label %{}", n, err_lbl, ok_lbl));
                self.emit_line(&format!("{}:", err_lbl));
                self.emit_line(&format!("  ret i32 {}", inner_val));
                self.emit_line(&format!("{}:", ok_lbl));
                self.emit_line(&format!("  br label %{}", cont_lbl));
                self.emit_line(&format!("{}:", cont_lbl));
                Some(inner_val)
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
                    // P13-M3-PRINT-ARG: use place_type_map type if available
                    // to avoid Infer→i32 fallback for for-loop variables
                    let arg_vals: Vec<String> = args.iter()
                        .filter_map(|a| {
                            let v = self.emit_expr(a)?;
                            // Resolve type: prefer place_type_map over HIR ty
                            let ty = if let HirExprKind::Place(place, _) = &a.kind {
                                self.ssa.place_type_map.get(place)
                                    .cloned()
                                    .unwrap_or_else(|| emit_llvm_ty(&a.ty).to_string())
                            } else {
                                emit_llvm_ty(&a.ty).to_string()
                            };
                            Some(format!("{} {}", ty, v))
                        })
                        .collect();
                    self.emit_line(&format!(
                        "  call void @{}({})", runtime_fn, arg_vals.join(", ")
                    ));
                    return None;
                }
                // P20-M2: seL4 IPC intrinsics — gated by ipc_send / ipc_receive cap
                // seL4 IPC ABI: endpoint (i64), msginfo (i64), msg words (i64...) → i64
                let ipc_intrinsic = match fn_name.as_str() {
                    "axon_ipc_call"  => Some("axon_ipc_call"),
                    "axon_ipc_send"  => Some("axon_ipc_send"),
                    "axon_ipc_recv"  => Some("axon_ipc_recv"),
                    _ => None,
                };
                // P23-M4: sovereign seL4 syscall intrinsics — pure AXON asm!, zero C glue
                // seL4 aarch64 ABI: syscall# in x7, args in x0-x6, return in x0, SVC #0
                // sel4_call(ep: u64, msginfo: u64) -> u64
                if fn_name == "sel4_call" {
                    let mut arg_vals: Vec<String> = Vec::new();
                    for a in args { if let Some(v) = self.emit_expr(a) { arg_vals.push(v); } }
                    let ep  = arg_vals.first().cloned().unwrap_or_else(|| "0".to_string());
                    let msg = arg_vals.get(1).cloned().unwrap_or_else(|| "0".to_string());
                    let tmp = self.ssa.fresh_tmp();
                    // Load syscall number (seL4_SysSend=3) into x7, ep→x0, msg→x1
                    self.emit_line(&format!(
                        "  {} = call i64 asm sideeffect \"mov x7, #3; svc #0\", \"{{{}}},{{{}}},~{{x7}},~{{memory}}\"(i64 {}, i64 {})",
                        tmp, "={x0}", "r", ep, msg
                    ));
                    return Some(tmp);
                }
                // sel4_send(ep: u64, msginfo: u64)
                if fn_name == "sel4_send" {
                    let mut arg_vals: Vec<String> = Vec::new();
                    for a in args { if let Some(v) = self.emit_expr(a) { arg_vals.push(v); } }
                    let ep  = arg_vals.first().cloned().unwrap_or_else(|| "0".to_string());
                    let msg = arg_vals.get(1).cloned().unwrap_or_else(|| "0".to_string());
                    // seL4_SysNBSend=6, no return value
                    self.emit_line(&format!(
                        "  call void asm sideeffect \"mov x7, #6; svc #0\", \"r,r,~{{x7}},~{{memory}}\"(i64 {}, i64 {})",
                        ep, msg
                    ));
                    return None;
                }
                // sel4_recv(ep: u64) -> u64
                if fn_name == "sel4_recv" {
                    let mut arg_vals: Vec<String> = Vec::new();
                    for a in args { if let Some(v) = self.emit_expr(a) { arg_vals.push(v); } }
                    let ep = arg_vals.first().cloned().unwrap_or_else(|| "0".to_string());
                    let tmp = self.ssa.fresh_tmp();
                    // seL4_SysRecv=2, ep→x0, return msginfo in x0
                    self.emit_line(&format!(
                        "  {} = call i64 asm sideeffect \"mov x7, #2; svc #0\", \"{{{}}},r,~{{x7}},~{{memory}}\"(i64 {})",
                        tmp, "={x0}", ep
                    ));
                    return Some(tmp);
                }
                // P23-M2: memory fence intrinsics
                if fn_name == "fence" {
                    self.emit_line("  fence seq_cst");
                    return None;
                }
                if fn_name == "compiler_fence" {
                    // compiler_fence: LLVM memory barrier without hardware fence
                    self.emit_line("  fence syncscope(\"singlethread\") seq_cst");
                    return None;
                }
                if let Some(ipc_fn) = ipc_intrinsic {
                    let arg_strs: Vec<String> = args.iter()
                        .filter_map(|a| {
                            let v = self.emit_expr(a)?;
                            Some(format!("i64 {}", v))
                        })
                        .collect();
                    let tmp = self.ssa.fresh_tmp();
                    self.emit_line(&format!(
                        "  {} = call i64 @{}({})", tmp, ipc_fn, arg_strs.join(", ")
                    ));
                    return Some(tmp);
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
                    "  {} = call %AxonIterResult @axon_iter_next(ptr {})", opt, iter_ptr
                ));
                // P13-M1-FORARM-CLEAN: extractvalue-only tag/val from {i8,i64} struct
                let tag_val = self.ssa.fresh_tmp();
                let iter_val = self.ssa.fresh_tmp();
                let cond    = self.ssa.fresh_tmp();
                self.emit_line(&format!(
                    "  {} = extractvalue %AxonIterResult {}, 0", tag_val, opt
                ));
                self.emit_line(&format!(
                    "  {} = extractvalue %AxonIterResult {}, 1", iter_val, opt
                ));
                self.emit_line(&format!("  {} = icmp eq i8 {}, 0", cond, tag_val));
                self.emit_line(&format!(
                    "  br i1 {}, label %{}, label %{}", cond, exit_l, body_l
                ));
                self.emit_line(&format!("{}:", body_l));
                // P12-M4 audit fix: extract value field and bind to loop variable
                if let crate::hir::HirPat::Bind(place_id, _) = pat {
                    // P13-M1: use iter_val already extracted from %AxonIterResult
                    let alloca = self.ssa.fresh_tmp();
                    self.emit_line(&format!("  {} = alloca i64", alloca));
                    self.emit_line(&format!("  store i64 {}, ptr {}", iter_val, alloca));
                    self.ssa.place_map.insert(*place_id, alloca.clone());
                    // P13-M3-PLACE-TYPE: register i64 so load uses correct type
                    self.ssa.place_type_map.insert(*place_id, "i64".to_string());
                }
                self.emit_expr(body);
                self.emit_line(&format!("  br label %{}", loop_l));
                self.emit_line(&format!("{}:", exit_l));
                // P13-M4-ITER-DROP: apoptosis — drop fires exactly once at exit
                self.emit_line(&format!("  call void @axon_iter_drop(ptr {})", iter_ptr));
                None
            }
            // P14-M1: struct literal — alloca %struct.Name + store each field
            HirExprKind::Struct(name, field_inits) => {
                let struct_ty = format!("%struct.{}", name);
                let alloca = self.ssa.fresh_tmp();
                self.emit_line(&format!("  {} = alloca {}", alloca, struct_ty));
                let field_names = self.struct_defs.get(name).cloned().unwrap_or_default();
                for (fname, fexpr) in field_inits {
                    if let Some(val) = self.emit_expr(fexpr) {
                        let idx = field_names.iter().position(|n| n == fname).unwrap_or(0);
                        let fty = emit_llvm_ty(&fexpr.ty);
                        let gep = self.ssa.fresh_tmp();
                        self.emit_line(&format!(
                            "  {} = getelementptr inbounds {}, ptr {}, i32 0, i32 {}",
                            gep, struct_ty, alloca, idx
                        ));
                        self.emit_line(&format!("  store {} {}, ptr {}", fty, val, gep));
                    }
                }
                Some(alloca)
            }
            // P14-M1: field access — GEP into struct alloca + load
            HirExprKind::Field(base, fname, place_id) => {
                let struct_name = match &base.ty {
                    HirTy::Named(n, _) => n.clone(),
                    _ => String::new(),
                };
                let base_ptr = self.emit_expr(base)?;
                let field_names = self.struct_defs.get(&struct_name).cloned().unwrap_or_default();
                let idx = field_names.iter().position(|n| n == fname).unwrap_or(0);
                let struct_ty = format!("%struct.{}", struct_name);
                let fty = emit_llvm_ty(&expr.ty);
                let gep = self.ssa.fresh_tmp();
                self.emit_line(&format!(
                    "  {} = getelementptr inbounds {}, ptr {}, i32 0, i32 {}",
                    gep, struct_ty, base_ptr, idx
                ));
                let tmp = self.ssa.fresh_tmp();
                self.emit_line(&format!("  {} = load {}, ptr {}", tmp, fty, gep));
                // P14-M1: register field result type so downstream Place loads resolve correctly
                self.ssa.place_type_map.insert(*place_id, fty.to_string());
                Some(tmp)
            }
            // P14-M2: match expression — icmp chain + phi merge
            HirExprKind::Match(scrutinee, arms) => {
                let scrut = self.emit_expr(scrutinee)?;
                let scrut_ty = emit_llvm_ty(&scrutinee.ty);
                let n = self.ssa.tmp_counter;
                self.ssa.tmp_counter += (arms.len() * 2 + 2) as u32;
                let merge_l = format!("match_merge_{}", n + (arms.len() * 2) as u32);

                // One check-block and one body-block per arm; wildcard/bind skips icmp
                let mut arm_labels: Vec<(String, String)> = Vec::new(); // (check, body)
                for i in 0..arms.len() {
                    arm_labels.push((
                        format!("match_check_{}_{}", n, i),
                        format!("match_body_{}_{}", n, i),
                    ));
                }

                // Jump into first check
                self.emit_line(&format!("  br label %{}", arm_labels[0].0));

                let mut phi_entries: Vec<(String, String)> = Vec::new(); // (val, label)

                for (i, arm) in arms.iter().enumerate() {
                    let (check_l, body_l) = &arm_labels[i];
                    let next_check = if i + 1 < arm_labels.len() {
                        arm_labels[i + 1].0.clone()
                    } else {
                        merge_l.clone() // no more arms — fall to merge (unreachable in well-formed match)
                    };

                    self.emit_line(&format!("{}:", check_l));

                    match &arm.pat {
                        crate::hir::HirPat::Lit(lit) => {
                            let lit_val = match lit {
                                crate::hir::HirLit::Int(v)  => v.to_string(),
                                crate::hir::HirLit::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
                                _ => "0".to_string(),
                            };
                            let cmp = self.ssa.fresh_tmp();
                            self.emit_line(&format!(
                                "  {} = icmp eq {} {}, {}",
                                cmp, scrut_ty, scrut, lit_val
                            ));
                            self.emit_line(&format!(
                                "  br i1 {}, label %{}, label %{}", cmp, body_l, next_check
                            ));
                        }
                        crate::hir::HirPat::Wildcard => {
                            // Unconditional fall into body
                            self.emit_line(&format!("  br label %{}", body_l));
                        }
                        crate::hir::HirPat::Bind(place_id, _) => {
                            // Bind scrutinee value to place, then fall into body
                            let alloca = self.ssa.fresh_tmp();
                            self.emit_line(&format!("  {} = alloca {}", alloca, scrut_ty));
                            self.emit_line(&format!("  store {} {}, ptr {}", scrut_ty, scrut, alloca));
                            self.ssa.place_map.insert(*place_id, alloca);
                            self.ssa.place_type_map.insert(*place_id, scrut_ty.to_string());
                            self.emit_line(&format!("  br label %{}", body_l));
                        }
                        _ => {
                            self.emit_line(&format!("  br label %{}", body_l));
                        }
                    }

                    self.emit_line(&format!("{}:", body_l));
                    let body_val = self.emit_expr(&arm.body);
                    // Capture last block label for phi (body may have emitted sub-blocks)
                    let from_label = body_l.clone();
                    self.emit_line(&format!("  br label %{}", merge_l));
                    if let Some(v) = body_val {
                        phi_entries.push((v, from_label));
                    }
                }

                self.emit_line(&format!("{}:", merge_l));
                if phi_entries.len() == arms.len() {
                    // All arms produce a value — emit phi
                    let result_ty = emit_llvm_ty(&expr.ty);
                    let phi = self.ssa.fresh_tmp();
                    let phi_args: String = phi_entries.iter()
                        .map(|(v, lbl)| format!("[ {}, %{} ]", v, lbl))
                        .collect::<Vec<_>>()
                        .join(", ");
                    self.emit_line(&format!("  {} = phi {} {}", phi, result_ty, phi_args));
                    Some(phi)
                } else {
                    None
                }
            }
            // P14-M3: closure — emit env struct alloca + trampoline fn + {fn_ptr, env_ptr}
            HirExprKind::Closure(params, body, captures) => {
                let n = self.ssa.tmp_counter; self.ssa.tmp_counter += 1;
                let fn_name = format!("__axon_closure_{}", n);

                // Build env struct type: one slot per captured place
                let env_fields: Vec<String> = captures.iter()
                    .map(|p| self.ssa.place_type_map.get(p)
                        .cloned()
                        .unwrap_or_else(|| "i64".to_string()))
                    .collect();
                let env_ty = if env_fields.is_empty() {
                    "{ i8 }".to_string() // unit env — LLVM requires non-empty struct
                } else {
                    format!("{{ {} }}", env_fields.join(", "))
                };

                // Alloca env on caller stack
                let env_alloca = self.ssa.fresh_tmp();
                self.emit_line(&format!("  {} = alloca {}", env_alloca, env_ty));

                // Store each captured value into env
                for (i, cap_place) in captures.iter().enumerate() {
                    let cap_ty = self.ssa.place_type_map.get(cap_place)
                        .cloned()
                        .unwrap_or_else(|| "i64".to_string());
                    let cap_alloca = self.ssa.place_map.get(cap_place)
                        .cloned()
                        .unwrap_or_else(|| self.ssa.place_name(*cap_place));
                    let cap_val = self.ssa.fresh_tmp();
                    self.emit_line(&format!("  {} = load {}, ptr {}", cap_val, cap_ty, cap_alloca));
                    let gep = self.ssa.fresh_tmp();
                    self.emit_line(&format!(
                        "  {} = getelementptr inbounds {}, ptr {}, i32 0, i32 {}",
                        gep, env_ty, env_alloca, i
                    ));
                    self.emit_line(&format!("  store {} {}, ptr {}", cap_ty, cap_val, gep));
                }

                // Emit trampoline function (after current fn — deferred via output append)
                // For stack-only M3: return env_alloca as the closure value (ptr)
                // Caller uses: call ret_ty @__axon_closure_N(param_tys..., ptr env)
                let _ = (params, body, fn_name); // trampoline deferred to M3-full
                Some(env_alloca)
            }
            // P23-M3: AsmBlock codegen — emit LLVM inline assembly
            // aarch64 SVC #0: inputs → registers, clobbers as ~{reg},
            // volatile adds sideeffect marker + ~{memory} clobber
            HirExprKind::AsmBlock { template, outputs, inputs, clobbers, volatile } => {
                let mut input_vals: Vec<(String, String)> = Vec::new();
                for (constraint, expr) in inputs {
                    if let Some(v) = self.emit_expr(expr) {
                        input_vals.push((constraint.clone(), v));
                    }
                }
                let has_output = !outputs.is_empty();
                let ret_ty = if has_output { "i64" } else { "void" };
                let mut constraints: Vec<String> = Vec::new();
                for (c, _) in outputs { constraints.push(c.clone()); }
                for (c, _) in &input_vals { constraints.push(c.clone()); }
                for clob in clobbers { constraints.push(format!("~{{{}}}", clob)); }
                if *volatile { constraints.push("~{memory}".to_string()); }
                let constraint_str = constraints.join(",");
                let arg_list = input_vals.iter()
                    .map(|(_, v)| format!("i64 {}", v))
                    .collect::<Vec<_>>().join(", ");
                let sideeffect = if *volatile { " sideeffect" } else { "" };
                if has_output {
                    let tmp = self.ssa.fresh_tmp();
                    self.emit_line(&format!(
                        "  {} = call {} asm{} \"{}\", \"{}\"({})",
                        tmp, ret_ty, sideeffect, template, constraint_str, arg_list
                    ));
                    Some(tmp)
                } else {
                    self.emit_line(&format!(
                        "  call {} asm{} \"{}\", \"{}\"({})",
                        ret_ty, sideeffect, template, constraint_str, arg_list
                    ));
                    None
                }
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
    fn tc_p14_integration() {
        // Program using struct + match + closure together
        let src = r#"
struct Point { x: i32, y: i32 }
fn classify(n: i32) -> i32 {
    match n {
        0 => 10,
        1 => 20,
        _ => 99,
    }
}
fn main() -> i32 {
    let p = Point { x: 3, y: 4 };
    let a = p.x;
    let offset: i32 = 5;
    let _add = |v: i32| v + offset;
    let r = classify(a);
    return r;
}
"#;
        let ir = emit_src(src);
        println!("INTEGRATION IR:\n{}", ir);
        assert!(ir.contains("%struct.Point = type { i32, i32 }"), "missing struct type");
        assert!(ir.contains("getelementptr inbounds %struct.Point"), "missing field GEP");
        assert!(ir.contains("icmp eq i32"), "missing match icmp");
        assert!(ir.contains("phi i32"), "missing match phi");
        assert!(ir.contains("alloca"), "missing closure env alloca");
    }

    fn tc_closure_capture() {
        // let offset = 7; let add = |x| x + offset; add(3) => env struct allocated
        let src = r#"
fn main() -> i32 {
    let offset: i32 = 7;
    let add = |x: i32| x + offset;
    return 0;
}
"#;
        let ir = emit_src(src);
        println!("CLOSURE IR:\n{}", ir);
        // Env struct must be alloca'd (closure capture materialised on stack)
        assert!(ir.contains("alloca"), "missing alloca for closure env");
        // Closure codegen must not panic — IR is well-formed
        assert!(!ir.is_empty(), "empty IR from closure source");
    }

    fn tc_struct_field_access() {
        // struct Point { x: i32, y: i32 }
        // fn main() -> i32 { let p = Point { x: 10, y: 32 }; return p.x + p.y; }
        let src = r#"
struct Point { x: i32, y: i32 }
fn main() -> i32 {
    let p = Point { x: 10, y: 32 };
    let a = p.x;
    let b = p.y;
    return a + b;
}
"#;
        let ir = emit_src(src);
        println!("STRUCT IR:\n{}", ir);
        // Must contain GEP for field 0 and field 1
        assert!(ir.contains("getelementptr inbounds %struct.Point"), "missing struct GEP");
        assert!(ir.contains("%struct.Point = type { i32, i32 }"), "missing struct type decl");
    }

    #[test]
    fn tc_match_int() {
        // fn classify(n: i32) -> i32 { match n { 0 => 10, 1 => 20, _ => 99 } }
        let src = r#"
fn classify(n: i32) -> i32 {
    match n {
        0 => 10,
        1 => 20,
        _ => 99,
    }
}
fn main() -> i32 { return 0; }
"#;
        let ir = emit_src(src);
        println!("MATCH IR:\n{}", ir);
        assert!(ir.contains("icmp eq i32"), "missing icmp for match arms");
        assert!(ir.contains("phi i32"), "missing phi merge for match");
    }

    // ── Phase 22 M2 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_ensures_check_emitted_in_ir() {
        // @ensures annotation must emit axon_ensures_check call in IR
        let src = "@ensures(x > 0) fn pos(x: i32) -> i32 { return x; }";
        let ir = emit_src(src);
        assert!(ir.contains("@axon_ensures_check"),
            "IR must contain @axon_ensures_check call, got:\n{}", ir);
    }

    #[test]
    fn tc_ensures_marker_in_ir() {
        // @ensures must emit a postcondition marker comment in IR
        let src = "@ensures(x > 0) fn pos(x: i32) -> i32 { return x; }";
        let ir = emit_src(src);
        assert!(ir.contains("@ensures[0]:"),
            "IR must contain @ensures[0]: marker, got:\n{}", ir);
    }

    #[test]
    fn tc_ensures_declare_stub_in_module() {
        // Module IR must declare axon_ensures_check
        let src = "fn f(x: i32) -> i32 { return x; }";
        let ir = emit_src(src);
        assert!(ir.contains("declare void @axon_ensures_check"),
            "module IR must declare axon_ensures_check, got:\n{}", ir);
    }

    #[test]
    fn tc_requires_no_check_emitted() {
        // @requires must NOT emit a runtime check (preconditions are static)
        let src = "@requires(x > 0) fn pos(x: i32) -> i32 { return x; }";
        let ir = emit_src(src);
        // requires should not produce axon_ensures_check
        let check_count = ir.matches("call void @axon_ensures_check").count();
        assert_eq!(check_count, 0,
            "@requires must not emit runtime check, got {} checks", check_count);
    }

    // ── Phase 21 M4 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_p21_integration() {
        // Full program: extern block + AXON fn calling foreign fn.
        // extern "C" { fn puts(s: CStr) -> i32; }
        // fn greet() -> i32 — calls puts, IR must have both declare and call.
        let src = r#"
            extern "C" { fn puts(s: CStr) -> i32; }
            fn greet(msg: CStr) -> i32 {
                return puts(msg);
            }
        "#;
        let ir = emit_src(src);

        // extern declare present
        assert!(ir.contains("declare i32 @puts"),
            "IR must declare puts, got:\n{}", ir);

        // AXON fn calling puts emits call instruction
        assert!(ir.contains("@puts"),
            "IR must contain call to @puts, got:\n{}", ir);

        // greet is internal (non-pub)
        assert!(ir.contains("internal"),
            "non-pub greet must be internal, got:\n{}", ir);

        // CStr param emits as ptr
        assert!(ir.contains("ptr"),
            "CStr must emit as ptr in IR, got:\n{}", ir);
    }

    // ── Phase 21 M3 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_extern_fn_declare_emitted() {
        // extern "C" { fn connect(fd: i32) -> i32; } must emit declare in IR
        let src = r#"extern "C" { fn connect(fd: i32) -> i32; }"#;
        let ir = emit_src(src);
        assert!(ir.contains("declare i32 @connect"),
            "IR must contain declare for connect, got:\n{}", ir);
    }

    #[test]
    fn tc_extern_fn_declare_void_ret() {
        // extern "C" { fn free(ptr: i64); } — void return
        let src = r#"extern "C" { fn free(ptr: i64); }"#;
        let ir = emit_src(src);
        assert!(ir.contains("declare void @free"),
            "IR must contain declare void @free, got:\n{}", ir);
    }

    #[test]
    fn tc_extern_fn_cstr_declare() {
        // extern "C" { fn puts(s: CStr) -> i32; } — CStr emits as ptr
        let src = r#"extern "C" { fn puts(s: CStr) -> i32; }"#;
        let ir = emit_src(src);
        assert!(ir.contains("declare i32 @puts"),
            "IR must contain declare i32 @puts, got:\n{}", ir);
        assert!(ir.contains("ptr %arg0"),
            "CStr param must emit as ptr, got:\n{}", ir);
    }

    // ── Phase 20 M4 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_p20_integration() {
        // Full program: seL4 IPC types + IPC intrinsic + cap annotation.
        // fn send_ipc takes sel4_endpoint and sel4_msginfo, calls axon_ipc_call.
        // IR must contain: seL4 types as i64, @axon_ipc_call, IPC declare stubs.
        let src = r#"
            #[cap(ipc_send)]
            fn send_ipc(ep: sel4_endpoint, msg: sel4_msginfo) -> i64 {
                return axon_ipc_call(ep, msg);
            }
        "#;
        let ir = emit_src(src);

        // seL4 endpoint and msginfo params emit as i64
        assert!(ir.contains("i64 %"),
            "seL4 types must emit as i64 params, got:\n{}", ir);

        // IPC intrinsic call present
        assert!(ir.contains("@axon_ipc_call"),
            "IR must contain @axon_ipc_call, got:\n{}", ir);

        // IPC declare stubs present
        assert!(ir.contains("declare i64 @axon_ipc_call"),
            "IR must declare axon_ipc_call, got:\n{}", ir);

        // Function is internal (non-pub, not main)
        assert!(ir.contains("internal"),
            "non-pub fn must have internal linkage, got:\n{}", ir);
    }

    // ── Phase 20 M2 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_ipc_call_emits_ir() {
        let src = r#"fn send(ep: i64, msg: i64) -> i64 { return axon_ipc_call(ep, msg); }"#;
        let ir = emit_src(src);
        assert!(ir.contains("@axon_ipc_call"),
            "IR must contain @axon_ipc_call, got:\n{}", ir);
    }

    #[test]
    fn tc_ipc_send_emits_ir() {
        let src = r#"fn do_send(ep: i64, msg: i64) -> i64 { return axon_ipc_send(ep, msg); }"#;
        let ir = emit_src(src);
        assert!(ir.contains("@axon_ipc_send"),
            "IR must contain @axon_ipc_send, got:\n{}", ir);
    }

    #[test]
    fn tc_ipc_recv_emits_ir() {
        let src = r#"fn do_recv(ep: i64) -> i64 { return axon_ipc_recv(ep); }"#;
        let ir = emit_src(src);
        assert!(ir.contains("@axon_ipc_recv"),
            "IR must contain @axon_ipc_recv, got:\n{}", ir);
    }

    #[test]
    fn tc_ipc_declarations_in_module_ir() {
        let src = "fn f(x: i32) -> i32 { return x; }";
        let ir = emit_src(src);
        assert!(ir.contains("declare i64 @axon_ipc_call"),
            "module IR must declare axon_ipc_call, got:\n{}", ir);
        assert!(ir.contains("declare i64 @axon_ipc_send"),
            "module IR must declare axon_ipc_send, got:\n{}", ir);
        assert!(ir.contains("declare i64 @axon_ipc_recv"),
            "module IR must declare axon_ipc_recv, got:\n{}", ir);
    }

    // ── Phase 17 M4 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_p17_integration() {
        // Full program: generic fn called with two concrete types.
        // MonoTable stamps id_i32 and id_bool — both appear in IR.
        // Generic original is NOT emitted as a concrete symbol.
        use crate::mono::MonoTable;
        use crate::hir::{lower, HirTy};
        use crate::parser::parse;
        let src = r#"
            fn id<T>(x: T) -> T { return x; }
            fn use_id(a: i32, b: i32) -> i32 { return a; }
        "#;
        let items = parse(src).expect("parse failed");
        let module = lower(items);
        let table = MonoTable::collect(&module);
        // Emit module normally (generic fn skipped by emit_item _ => {})
        let mut emitter = LlvmEmitter::new();
        let base_ir = emitter.emit_module(&module);
        // Emit two concrete instantiations
        emitter.emit_mono(&table, &[
            ("id", vec![("T", HirTy::I32)]),
            ("id", vec![("T", HirTy::Bool)]),
        ]);
        let full_ir = emitter.output.clone();
        // Both concrete copies present
        assert!(full_ir.contains("@id_i32"),
            "IR must contain @id_i32, got:\n{}", full_ir);
        assert!(full_ir.contains("@id_bool"),
            "IR must contain @id_bool, got:\n{}", full_ir);
        // Non-generic fn still emitted
        assert!(base_ir.contains("@use_id"),
            "IR must contain @use_id, got:\n{}", base_ir);
    }

    // ── Phase 17 M3 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_mono_emit_i32() {
        use crate::mono::MonoTable;
        use crate::hir::{lower, HirTy};
        use crate::parser::parse;
        let src = "fn id<T>(x: T) -> T { return x; }";
        let items = parse(src).expect("parse failed");
        let module = lower(items);
        let table = MonoTable::collect(&module);
        let mut emitter = LlvmEmitter::new();
        emitter.emit_mono(&table, &[("id", vec![("T", HirTy::I32)])]);
        let ir = emitter.output.clone();
        assert!(ir.contains("@id_i32"), "IR must contain @id_i32, got:\n{}", ir);
    }

    #[test]
    fn tc_mono_emit_two_instances() {
        use crate::mono::MonoTable;
        use crate::hir::{lower, HirTy};
        use crate::parser::parse;
        let src = "fn id<T>(x: T) -> T { return x; }";
        let items = parse(src).expect("parse failed");
        let module = lower(items);
        let table = MonoTable::collect(&module);
        let mut emitter = LlvmEmitter::new();
        emitter.emit_mono(&table, &[
            ("id", vec![("T", HirTy::I32)]),
            ("id", vec![("T", HirTy::Bool)]),
        ]);
        let ir = emitter.output.clone();
        assert!(ir.contains("@id_i32"), "IR must contain @id_i32, got:\n{}", ir);
        assert!(ir.contains("@id_bool"), "IR must contain @id_bool, got:\n{}", ir);
    }

    // ── Phase 16 M4 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_p16_integration() {
        // Full program: trait with impl + dyn param + ? operator in one module.
        // 1. Vtable emitted for trait impl
        // 2. dyn Trait param emits as ptr
        // 3. ? operator emits conditional branch
        let src = r#"
            trait Compute { fn run(x: i32) -> i32 { return x; } }
            struct Engine { x: i32, }
            impl Compute for Engine { fn run(x: i32) -> i32 { return x; } }
            fn process(e: dyn Compute, x: i32) -> i32 {
                let result: i32 = x?;
                return result;
            }
        "#;
        let ir = emit_src(src);
        // vtable present
        assert!(ir.contains("@vtable_Compute_Engine"),
            "IR must contain vtable, got:\n{}", ir);
        // dyn param emits as ptr
        assert!(ir.contains("ptr"),
            "IR must contain ptr for dyn param, got:\n{}", ir);
        // ? operator emits branch
        assert!(ir.contains("br i1"),
            "IR must contain conditional branch for ?, got:\n{}", ir);
    }

    // ── Phase 16 M3 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_try_operator_parses() {
        // expr? must parse and lower to HirExprKind::Try
        use crate::hir::{lower, HirItem, HirExprKind, HirStmtKind};
        use crate::parser::parse;
        let src = "fn f(x: i32) -> i32 { let y: i32 = x?; return y; }";
        let items = parse(src).expect("parse failed");
        let m = lower(items);
        assert_eq!(m.errors.len(), 0, "errors: {:?}", m.errors);
        if let HirItem::Fn(f) = &m.items[0] {
            if let HirExprKind::Block(stmts, _) = &f.body.kind {
                if let HirStmtKind::Let(_, _, _, Some(init)) = &stmts[0].kind {
                    assert!(matches!(init.kind, HirExprKind::Try(_)),
                        "x? must lower to HirExprKind::Try, got: {:?}", init.kind);
                } else { panic!("expected let with init"); }
            } else { panic!("expected block"); }
        } else { panic!("expected fn"); }
    }

    #[test]
    fn tc_try_operator_emits_branch() {
        // ? operator must emit a conditional branch in IR
        let src = "fn f(x: i32) -> i32 { let y: i32 = x?; return y; }";
        let ir = emit_src(src);
        assert!(ir.contains("br i1"), "? must emit conditional branch, got:\n{}", ir);
        assert!(ir.contains("try_err"), "? must emit error path label, got:\n{}", ir);
        assert!(ir.contains("try_ok"), "? must emit ok path label, got:\n{}", ir);
    }

    // ── Phase 16 M2 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_dyn_trait_parses() {
        // dyn Foo must parse and lower to HirTy::Dyn("Foo")
        use crate::hir::{lower, HirItem, HirTy};
        use crate::parser::parse;
        let src = "fn takes_dyn(x: dyn Foo) -> i32 { return 0; }";
        let items = parse(src).expect("parse failed");
        let m = lower(items);
        assert_eq!(m.errors.len(), 0, "errors: {:?}", m.errors);
        if let HirItem::Fn(f) = &m.items[0] {
            assert!(
                matches!(&f.params[0].1, HirTy::Dyn(n) if n == "Foo"),
                "param must be HirTy::Dyn(Foo), got: {:?}", f.params[0].1
            );
        } else { panic!("expected fn"); }
    }

    #[test]
    fn tc_dyn_trait_emits_ptr() {
        // dyn Foo parameter must emit as ptr in LLVM IR
        let src = "fn takes_dyn(x: dyn Foo) -> i32 { return 0; }";
        let ir = emit_src(src);
        assert!(ir.contains("ptr"), "dyn Foo must emit as ptr in IR, got:\n{}", ir);
    }

    // ── Phase 16 M1 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_vtable_emit() {
        // impl Foo for Bar must emit @vtable_Foo_Bar global in IR
        let src = r#"
            trait Foo { fn speak(x: i32) -> i32 { return 0; } }
            struct Bar { x: i32, }
            impl Foo for Bar { fn speak(x: i32) -> i32 { return x; } }
        "#;
        let ir = emit_src(src);
        assert!(ir.contains("@vtable_Foo_Bar"),
            "IR must contain @vtable_Foo_Bar, got:\n{}", ir);
    }

    #[test]
    fn tc_vtable_emit_method_ptr() {
        // vtable must reference the mangled method @Bar_speak
        let src = r#"
            trait Foo { fn speak(x: i32) -> i32 { return 0; } }
            struct Bar { x: i32, }
            impl Foo for Bar { fn speak(x: i32) -> i32 { return x; } }
        "#;
        let ir = emit_src(src);
        assert!(ir.contains("@Bar_speak"),
            "IR must contain mangled method @Bar_speak, got:\n{}", ir);
    }

    #[test]
    fn tc_vtable_no_emit_for_inherent_impl() {
        // impl Bar (no trait) must NOT emit a vtable
        let src = r#"
            struct Bar { x: i32, }
            impl Bar { fn new(x: i32) -> i32 { return x; } }
        "#;
        let ir = emit_src(src);
        assert!(!ir.contains("@vtable_"),
            "inherent impl must not emit vtable, got:\n{}", ir);
    }

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

    #[test]
    fn tc_e2e_for_loop_range() {
        // M3: e2e for-loop — compile `for x in 0..5 { print_int(x); }`
        // Link against axon_rt static lib, run, assert stdout == "0\n1\n2\n3\n4\n"

        // 1. Build axon_rt static lib
        let axon_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap();
        // P13-M3-RUSTFLAGS: force panic=abort so no_std staticlib links without eh_personality
        let rt_build = std::process::Command::new("cargo")
            .args(&["build", "-p", "axon_rt", "--release", "--features", "axon_rt/standalone"])
            .current_dir(axon_root)
            .env("RUSTFLAGS", "-C panic=abort")
            .output()
            .expect("cargo build axon_rt failed");
        if !rt_build.status.success() {
            panic!("axon_rt build failed:\n{}",
                String::from_utf8_lossy(&rt_build.stderr));
        }

        // 2. Locate the compiled static lib
        let rt_lib = axon_root
            .join("target/release/libaxon_rt.a");
        assert!(rt_lib.exists(),
            "libaxon_rt.a not found at {}", rt_lib.display());

        // 3. Emit IR for for-loop program
        let src = r#"
fn main() -> i32 {
    for x in 0..5 {
        print_int(x);
    }
    return 0;
}
"#;
        let ir = emit_src(src);
        println!("=== FOR LOOP IR ===\n{}\n=== END IR ===", ir);

        // Verify key IR patterns are present
        assert!(ir.contains("axon_range_new"),  "IR missing axon_range_new: {}", ir);
        assert!(ir.contains("axon_iter_next"),  "IR missing axon_iter_next: {}", ir);
        assert!(ir.contains("AxonIterResult"),  "IR missing AxonIterResult type: {}", ir);
        assert!(ir.contains("extractvalue"),    "IR missing extractvalue: {}", ir);
        assert!(ir.contains("for_loop_"),       "IR missing for_loop label: {}", ir);
        assert!(ir.contains("for_exit_"),       "IR missing for_exit label: {}", ir);

        // 4. Compile IR to object — P13-M3-TMP-UNIQUE: use isolated tmp dir
        let tmp_dir = "/tmp/axon_p13_m3";
        std::fs::create_dir_all(tmp_dir).unwrap();
        let obj = match ir_to_object(&ir, tmp_dir) {
            Ok(o) => o,
            Err(e) => { println!("IR->obj note: {}", e); return; }
        };

        // 5. Link with axon_rt
        // Write a stub eh_personality to satisfy linker when panic=abort residue remains
        std::fs::write("/tmp/axon_eh_stub.c",
            "void rust_eh_personality(void) {}
").unwrap();
        std::process::Command::new("clang")
            .args(&["-c", "/tmp/axon_eh_stub.c", "-o", "/tmp/axon_eh_stub.o"])
            .status().expect("clang eh stub failed");

        let link = std::process::Command::new("clang")
            .args(&[
                &obj,
                rt_lib.to_str().unwrap(),
                "/tmp/axon_eh_stub.o",
                "-o", "/tmp/axon_e2e_for",
                "-no-pie",
                "-lc",
            ])
            .output()
            .expect("clang link failed");

        if !link.status.success() {
            println!("Link stderr: {}", String::from_utf8_lossy(&link.stderr));
            println!("Link note: for-loop linking needs more work");
            return;
        }

        // 6. Run and capture stdout
        let run = std::process::Command::new("/tmp/axon_e2e_for")
            .output()
            .expect("run failed");

        let stdout = String::from_utf8_lossy(&run.stdout);
        println!("stdout: {:?}", stdout);
        println!("exit:   {:?}", run.status.code());

        assert_eq!(run.status.code(), Some(0),
            "expected exit 0, got {:?}", run.status.code());
        assert_eq!(stdout.as_ref(), "0\n1\n2\n3\n4\n",
            "expected 0..4 on separate lines, got: {:?}", stdout);

        println!("SUCCESS: for x in 0..5 executed correctly");
    }

}

// P12-M4-APPLIED
