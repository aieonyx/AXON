#!/usr/bin/env python3
import sys

changes = []

# ── hir.rs ───────────────────────────────────────────────────────────────
HIR = "/home/edisonbl/axon/axon_parser/src/hir.rs"
with open(HIR) as f: hir = f.read()
hir_orig = hir

# H1: remove unused Contract import
hir = hir.replace('FnSig, Contract, ContractKind,', 'FnSig, ContractKind,')
changes.append('H1: Contract import removed')

# H2: unreachable wildcard arm at end of lower_expr match
hir = hir.replace(
    '            _ => HirExprKind::Lit(HirLit::Unit),',
    '            #[allow(unreachable_patterns)]\n            _ => HirExprKind::Lit(HirLit::Unit),'
)
changes.append('H2: unreachable pattern suppressed')

# H3: unused variable lhs in Assign
hir = hir.replace(
    'Expr::Assign(lhs, rhs, _) => {',
    'Expr::Assign(_lhs, rhs, _) => {'
)
changes.append('H3: Assign lhs -> _lhs')

# H4: unused variables op and lhs in AssignOp
hir = hir.replace(
    'Expr::AssignOp(op, lhs, rhs, _) => {',
    'Expr::AssignOp(_op, _lhs, rhs, _) => {'
)
changes.append('H4: AssignOp op,lhs -> _op,_lhs')

# H5: unused variable inner in Ref
hir = hir.replace(
    'Expr::Ref(is_mut, inner, _) => {',
    'Expr::Ref(is_mut, _inner, _) => {'
)
changes.append('H5: Ref inner -> _inner')

if hir != hir_orig:
    with open(HIR, 'w') as f: f.write(hir)
    print('hir.rs written OK')
else:
    print('hir.rs: NO CHANGES')

# ── infer.rs ─────────────────────────────────────────────────────────────
INFER = "/home/edisonbl/axon/axon_parser/src/infer.rs"
with open(INFER) as f: inf = f.read()
inf_orig = inf

# I1: remove unused imports
inf = inf.replace(
    'HirModule, HirItem, HirFn, HirStruct, HirEnum,',
    'HirModule, HirItem, HirFn,'
)
inf = inf.replace(
    '    HirExpr, HirExprKind, HirStmt, HirStmtKind,\n    HirPat, HirMatchArm, HirLit, HirTy, HirImpl,\n    PlaceId, BorrowId, MoveState, NodeId,',
    '    HirExpr, HirExprKind, HirStmt, HirStmtKind,\n    HirLit, HirTy,\n    PlaceId, NodeId,'
)
changes.append('I1: infer.rs unused imports removed')

if inf != inf_orig:
    with open(INFER, 'w') as f: f.write(inf)
    print('infer.rs written OK')
else:
    print('infer.rs: NO CHANGES')

# ── codegen.rs ───────────────────────────────────────────────────────────
CG = "/home/edisonbl/axon/axon_parser/src/codegen.rs"
with open(CG) as f: cg = f.read()
cg_orig = cg

# C1: remove HirPat and HirMatchArm from codegen imports
cg = cg.replace(
    '    HirStmt, HirStmtKind, HirLit, HirTy, HirPat,\n    HirMatchArm, PlaceId,',
    '    HirStmt, HirStmtKind, HirLit, HirTy,\n    PlaceId,'
)
changes.append('C1: codegen.rs unused imports removed')

if cg != cg_orig:
    with open(CG, 'w') as f: f.write(cg)
    print('codegen.rs written OK')
else:
    print('codegen.rs: NO CHANGES')

print('--- changes ---')
for c in changes: print(' ', c)
