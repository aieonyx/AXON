#!/usr/bin/env python3
changes = []

# ── codegen.rs: AxonVec runtime externs + MethodCall dispatch ────────────
CG = "/home/edisonbl/axon/axon_parser/src/codegen.rs"
with open(CG) as f: s = f.read()
orig = s

# C1: add AxonVec extern declarations after axon_slice_len
old1 = '        self.emit_line("declare i64 @axon_slice_len(ptr)");'
new1 = '''        self.emit_line("declare i64 @axon_slice_len(ptr)");
        // P11-M3: AxonVec runtime externs
        self.emit_line("declare ptr @axon_vec_new()");
        self.emit_line("declare void @axon_vec_push(ptr, ptr)");
        self.emit_line("declare ptr @axon_vec_pop(ptr)");
        self.emit_line("declare i64 @axon_vec_len(ptr)");
        self.emit_line("declare i1  @axon_vec_is_empty(ptr)");
        self.emit_line("declare ptr @axon_vec_get(ptr, i64)");'''
if old1 in s: s = s.replace(old1, new1); changes.append('C1: AxonVec externs declared')
else: changes.append('C1 MISSING')

# C2: AxonVec MethodCall dispatch — insert inside MethodCall arm before String dispatch
old2 = '                // String method dispatch — existing runtime externs'
new2 = '''                // P11-M3: AxonVec method dispatch
                if matches!(&recv.ty, HirTy::Named(n, _) if n == "AxonVec") {
                    if let Some(vec_ptr) = self.emit_expr(recv) {
                        match method.as_str() {
                            \"len\" => {
                                let tmp = self.ssa.fresh_tmp();
                                self.emit_line(&format!(\"  {} = call i64 @axon_vec_len(ptr {})\", tmp, vec_ptr));
                                return Some(tmp);
                            }
                            \"is_empty\" => {
                                let tmp = self.ssa.fresh_tmp();
                                self.emit_line(&format!(\"  {} = call i1 @axon_vec_is_empty(ptr {})\", tmp, vec_ptr));
                                return Some(tmp);
                            }
                            \"push\" => {
                                if let Some(arg) = args.first() {
                                    if let Some(av) = self.emit_expr(arg) {
                                        let elem_alloca = self.ssa.fresh_tmp();
                                        let ety = emit_llvm_ty(&arg.ty);
                                        self.emit_line(&format!(\"  {} = alloca {}\", elem_alloca, ety));
                                        self.emit_line(&format!(\"  store {} {}, ptr {}\", ety, av, elem_alloca));
                                        self.emit_line(&format!(\"  call void @axon_vec_push(ptr {}, ptr {})\", vec_ptr, elem_alloca));
                                    }
                                }
                                return None;
                            }
                            \"pop\" => {
                                let tmp = self.ssa.fresh_tmp();
                                self.emit_line(&format!(\"  {} = call ptr @axon_vec_pop(ptr {})\", tmp, vec_ptr));
                                return Some(tmp);
                            }
                            \"get\" => {
                                if let Some(arg) = args.first() {
                                    if let Some(iv) = self.emit_expr(arg) {
                                        let tmp = self.ssa.fresh_tmp();
                                        self.emit_line(&format!(\"  {} = call ptr @axon_vec_get(ptr {}, i64 {})\", tmp, vec_ptr, iv));
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
                // String method dispatch — existing runtime externs'''
if old2 in s: s = s.replace(old2, new2); changes.append('C2: AxonVec MethodCall dispatch')
else: changes.append('C2 MISSING')

# C3: AxonVec::new() Call dispatch — route Path('AxonVec','new') to @axon_vec_new
old3 = '                let sovereign_print = match fn_name.as_str() {'
new3 = '''                // P11-M3: AxonVec::new() constructor
                if fn_name == "AxonVec_new" || fn_name == "AxonVec::new" {
                    let tmp = self.ssa.fresh_tmp();
                    self.emit_line(&format!("  {} = call ptr @axon_vec_new()", tmp));
                    return Some(tmp);
                }
                let sovereign_print = match fn_name.as_str() {'''
if old3 in s: s = s.replace(old3, new3); changes.append('C3: AxonVec::new() Call dispatch')
else: changes.append('C3 MISSING')

if s != orig:
    with open(CG, 'w') as f: f.write(s)
    print('codegen.rs written OK')
else: print('codegen.rs: NO CHANGES')

# ── infer.rs: AxonVec method return types ────────────────────────────────
INFER = "/home/edisonbl/axon/axon_parser/src/infer.rs"
with open(INFER) as f: s = f.read()
orig = s
old4 = '''                // P11-M2: slice method return types
                if matches!(recv_ty, InfTy::Slice(_)) {'
new4 = '''                // P11-M3: AxonVec method return types
                if matches!(&recv_ty, InfTy::Named(n, _) if n == "AxonVec") {
                    return match method.as_str() {
                        \"len\"      => InfTy::Usize,
                        \"is_empty\" => InfTy::Bool,
                        \"push\"     => InfTy::Unit,
                        \"pop\"      => self.fresh_var(),
                        \"get\"      => self.fresh_var(),
                        _           => self.fresh_var(),
                    };
                }
                // P11-M2: slice method return types
                if matches!(recv_ty, InfTy::Slice(_)) {'
if old4 in s: s = s.replace(old4, new4); changes.append('I1: AxonVec inference')
else: changes.append('I1 MISSING')
if s != orig:
    with open(INFER, 'w') as f: f.write(s)
    print('infer.rs written OK')
else: print('infer.rs: NO CHANGES')

# ── hir.rs: alloc_heap auto-infer for AxonVec ────────────────────────────
HIR = "/home/edisonbl/axon/axon_parser/src/hir.rs"
with open(HIR) as f: s = f.read()
orig = s
old5 = '        HirTy::Named(_, ts) => ts.iter().any(hir_ty_contains_string),'
new5 = '''        HirTy::Named(n, ts) => {
            // P11-M3: AxonVec always requires alloc_heap
            if n == "AxonVec" { return true; }
            ts.iter().any(hir_ty_contains_string)
        }'''
if old5 in s: s = s.replace(old5, new5); changes.append('H1: AxonVec alloc_heap inference')
else: changes.append('H1 MISSING')
if s != orig:
    with open(HIR, 'w') as f: f.write(s)
    print('hir.rs written OK')
else: print('hir.rs: NO CHANGES')

print('--- changes ---')
for c in changes: print(' ', c)
