#!/usr/bin/env python3
import sys
changes = []

# ── codegen.rs: Slice fat pointer support ────────────────────────────────
CG = "/home/edisonbl/axon/axon_parser/src/codegen.rs"
with open(CG) as f: s = f.read()
orig = s

# C1: emit_llvm_ty_owned — Slice -> { ptr, i64 } fat pointer
old1 = '''pub fn emit_llvm_ty_owned(ty: &HirTy) -> String {
    match ty {
        HirTy::Array(elem, n) => format!("[{} x {}]", n, emit_llvm_ty_owned(elem)),
        _ => emit_llvm_ty(ty).to_string(),
    }
}'''
new1 = '''pub fn emit_llvm_ty_owned(ty: &HirTy) -> String {
    match ty {
        HirTy::Array(elem, n) => format!("[{} x {}]", n, emit_llvm_ty_owned(elem)),
        HirTy::Slice(_) => "{ ptr, i64 }".to_string(),
        _ => emit_llvm_ty(ty).to_string(),
    }
}'''
if old1 in s: s = s.replace(old1, new1); changes.append('C1: emit_llvm_ty_owned Slice')
else: changes.append('C1 MISSING')

# C2: emit_llvm_ty — Slice -> ptr (opaque, fat ptr passed as alloca ptr)
old2 = "        HirTy::Str    => \"ptr\","
new2 = "        HirTy::Str    => \"ptr\",\n        HirTy::Slice(_) => \"ptr\",   // fat pointer alloca passed as ptr"
if old2 in s: s = s.replace(old2, new2); changes.append('C2: emit_llvm_ty Slice')
else: changes.append('C2 MISSING')

# C3: MethodCall codegen — add slice .len() dispatch before String dispatch
old3 = "            HirExprKind::MethodCall(recv, method, args) => {"
new3 = '''            HirExprKind::MethodCall(recv, method, args) => {
                // P11-M2: slice .len() — extract i64 from fat pointer field 1
                if matches!(recv.ty, HirTy::Slice(_)) && method == "len" {
                    if let Some(fat_ptr) = self.emit_expr(recv) {
                        let gep = self.ssa.fresh_tmp();
                        self.emit_line(&format!("  {} = getelementptr inbounds {{ ptr, i64 }}, ptr {}, i32 0, i32 1", gep, fat_ptr));
                        let tmp = self.ssa.fresh_tmp();
                        self.emit_line(&format!("  {} = load i64, ptr {}", tmp, gep));
                        return Some(tmp);
                    }
                    return None;
                }'''
if old3 in s: s = s.replace(old3, new3); changes.append('C3: slice .len() codegen')
else: changes.append('C3 MISSING')

# C4: Array codegen — after storing elements, also build fat pointer if coerced to slice
# We add a helper: when a let binding has Slice type and init is Array, build fat ptr
# This is done in emit_stmt Let — detect HirTy::Slice on the let type
old4 = "            HirStmtKind::Let(place, _, ty, init) => {"
new4 = '''            HirStmtKind::Let(place, _, ty, init) => {
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
                }'''
if old4 in s: s = s.replace(old4, new4); changes.append('C4: slice-from-array coercion in emit_stmt')
else: changes.append('C4 MISSING')

# C5: declare @axon_slice_len as fallback runtime (optional, for dynamic slices)
old5 = '        // P11-M4: bounds check runtime\n        self.emit_line("declare void @axon_bounds_check(i64, i64)");'
new5 = '        // P11-M4: bounds check runtime\n        self.emit_line("declare void @axon_bounds_check(i64, i64)");\n        // P11-M2: slice runtime\n        self.emit_line("declare i64 @axon_slice_len(ptr)");'
if old5 in s: s = s.replace(old5, new5); changes.append('C5: axon_slice_len declaration')
else: changes.append('C5 MISSING')

if s != orig:
    with open(CG, 'w') as f: f.write(s)
    print('codegen.rs written OK')
else: print('codegen.rs: NO CHANGES')

# ── infer.rs: Slice .len() returns Usize ─────────────────────────────────
INFER = "/home/edisonbl/axon/axon_parser/src/infer.rs"
with open(INFER) as f: s = f.read()
orig = s
old6 = '''                if matches!(recv_ty, InfTy::String) {
                    return match method.as_str() {
                        "len"          => InfTy::Usize,
                        "is_empty"     => InfTy::Bool,
                        "contains"     => InfTy::Bool,
                        "to_uppercase" => InfTy::String,
                        "to_lowercase" => InfTy::String,
                        _              => self.fresh_var(),
                    };
                }'''
new6 = '''                // P11-M2: slice method return types
                if matches!(recv_ty, InfTy::Slice(_)) {
                    return match method.as_str() {
                        "len"      => InfTy::Usize,
                        "is_empty" => InfTy::Bool,
                        _          => self.fresh_var(),
                    };
                }
                if matches!(recv_ty, InfTy::String) {
                    return match method.as_str() {
                        "len"          => InfTy::Usize,
                        "is_empty"     => InfTy::Bool,
                        "contains"     => InfTy::Bool,
                        "to_uppercase" => InfTy::String,
                        "to_lowercase" => InfTy::String,
                        _              => self.fresh_var(),
                    };
                }'''
if old6 in s: s = s.replace(old6, new6); changes.append('I1: Slice .len() inference')
else: changes.append('I1 MISSING')
if s != orig:
    with open(INFER, 'w') as f: f.write(s)
    print('infer.rs written OK')
else: print('infer.rs: NO CHANGES')

print('--- changes ---')
for c in changes: print(' ', c)
