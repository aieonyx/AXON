#!/usr/bin/env python3
changes = []

# ── infer.rs ─────────────────────────────────────────────────────────────
INFER = "/home/edisonbl/axon/axon_parser/src/infer.rs"
with open(INFER) as f: inf = f.read()
inf_orig = inf

# I2: NodeId unused
inf = inf.replace('    PlaceId, NodeId,', '    PlaceId,')
changes.append('I2: NodeId removed')

# I3: Error(e) unused e
inf = inf.replace('        InfTy::Error(e)     => HirTy::Error,', '        InfTy::Error(_e)    => HirTy::Error,')
changes.append('I3: Error e -> _e')

# I4: l1 l2 unused in Ref unification
inf = inf.replace('            (InfTy::Ref(m1,l1,t1), InfTy::Ref(m2,l2,t2)) => {', '            (InfTy::Ref(m1,_l1,t1), InfTy::Ref(m2,_l2,t2)) => {')
changes.append('I4: Ref l1,l2 -> _l1,_l2')

if inf != inf_orig:
    with open(INFER, 'w') as f: f.write(inf)
    print('infer.rs written OK')
else: print('infer.rs: NO CHANGES')

# ── hir.rs ───────────────────────────────────────────────────────────────
HIR = "/home/edisonbl/axon/axon_parser/src/hir.rs"
with open(HIR) as f: hir = f.read()
hir_orig = hir

# H6: span unused in lower_stmt
hir = hir.replace('        let span = Span::new(0, 0);', '        let _span = Span::new(0, 0);')
changes.append('H6: span -> _span')

# H7: item unused in Stmt::Item
hir = hir.replace('            Stmt::Item(item) => {', '            Stmt::Item(_item) => {')
changes.append('H7: item -> _item')

if hir != hir_orig:
    with open(HIR, 'w') as f: f.write(hir)
    print('hir.rs written OK')
else: print('hir.rs: NO CHANGES')

print('--- changes ---')
for c in changes: print(' ', c)
