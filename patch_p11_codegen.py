#!/usr/bin/env python3
import sys

PATH = "/home/edisonbl/axon/axon_parser/src/codegen.rs"

with open(PATH) as f: src = f.read()
original = src
changes = []

# PATCH 1: emit_llvm_ty_owned
OLD1 = 'pub fn emit_llvm_ty_owned(ty: &HirTy) -> String {\n    emit_llvm_ty(ty).to_string()\n}'
NEW1 = '''pub fn emit_llvm_ty_owned(ty: &HirTy) -> String {
    match ty {
        HirTy::Array(elem, n) => format!("[{} x {}]", n, emit_llvm_ty_owned(elem)),
        _ => emit_llvm_ty(ty).to_string(),
    }
}'''
if OLD1 in src: src = src.replace(OLD1, NEW1); changes.append('P1 OK')
else: changes.append('P1 MISSING')

# PATCH 2: emit_stmt Let owned ty
OLD2 = '                let llty = emit_llvm_ty(ty);'
NEW2 = '                let llty = emit_llvm_ty_owned(ty);'
if OLD2 in src: src = src.replace(OLD2, NEW2); changes.append('P2 OK')
else: changes.append('P2 MISSING')

# PATCH 3: Array literal codegen
OLD3 = '''            HirExprKind::Array(exprs) => {
                for e in exprs { self.emit_expr(e); }
                Some("null".to_string())
            }'''
NEW3 = '''            HirExprKind::Array(exprs) => {
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
            }'''
if OLD3 in src: src = src.replace(OLD3, NEW3); changes.append('P3 OK')
else: changes.append('P3 MISSING')

# PATCH 4: Index codegen inserted before Ref arm
OLD4 = '''            HirExprKind::Ref(_is_mut, place, _) => {
                Some(self.ssa.place_name(*place))
            }'''
NEW4 = '''            HirExprKind::Index(obj, idx, _) => {
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
            }'''
if OLD4 in src: src = src.replace(OLD4, NEW4); changes.append('P4 OK')
else: changes.append('P4 MISSING')

# PATCH 5: bounds_check declaration
OLD5 = '        self.emit_line("declare ptr @axon_string_to_lowercase(ptr)");' 
NEW5 = '        self.emit_line("declare ptr @axon_string_to_lowercase(ptr)");\n        self.emit_line("declare void @axon_bounds_check(i64, i64)");'
if OLD5 in src: src = src.replace(OLD5, NEW5); changes.append('P5 OK')
else: changes.append('P5 MISSING')

print('File:', PATH)
print('Modified:', src != original)
for c in changes: print(' ', c)
if src == original: print('ERROR: no changes'); sys.exit(1)
with open(PATH, 'w') as f: f.write(src)
print('Written OK')
