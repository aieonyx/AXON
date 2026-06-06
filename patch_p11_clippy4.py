#!/usr/bin/env python3
changes = []

HIR = "/home/edisonbl/axon/axon_parser/src/hir.rs"
with open(HIR) as f: hir = f.read()
hir_orig = hir

# H9b: allow must be on impl block, not struct — place on fn new instead
hir = hir.replace(
    '#[allow(clippy::new_without_default)]\npub struct MoveStateMap {',
    'pub struct MoveStateMap {'
)
hir = hir.replace(
    '    pub fn new() -> Self { MoveStateMap { entries: Vec::new() } }',
    '    #[allow(clippy::new_without_default)]\n    pub fn new() -> Self { MoveStateMap { entries: Vec::new() } }'
)
changes.append('H9b: new_without_default allow on fn new')

# H10: redundant closure |a| hir_expr_contains_string(a) x2 in contains_string fn
hir = hir.replace(
    '|| args.iter().any(|a| hir_expr_contains_string(a)),\n        HirExprKind::MethodCall(recv, _, args) =>\n            hir_expr_contains_string(recv) || args.iter().any(|a| hir_expr_contains_string(a)),',
    '|| args.iter().any(hir_expr_contains_string),\n        HirExprKind::MethodCall(recv, _, args) =>\n            hir_expr_contains_string(recv) || args.iter().any(hir_expr_contains_string),'
)
changes.append('H10: redundant closure in contains_string')

# H11: map_or(false, ...) -> is_some_and in contains_string
hir = hir.replace(
    '}) || tail.as_ref().map_or(false, |e| hir_expr_contains_string(e))\n        }\n        HirExprKind::If(c, t, e) =>\n            hir_expr_contains_string(c) || hir_expr_contains_string(t)\n            || e.as_ref().map_or(false, |e| hir_expr_contains_string(e)),',
    '}) || tail.as_ref().is_some_and(|e| hir_expr_contains_string(e))\n        }\n        HirExprKind::If(c, t, e) =>\n            hir_expr_contains_string(c) || hir_expr_contains_string(t)\n            || e.as_ref().is_some_and(|e| hir_expr_contains_string(e)),'
)
changes.append('H11: map_or->is_some_and in contains_string')

# H12: same fixes in hir_expr_contains_index
hir = hir.replace(
    '|| args.iter().any(|a| hir_expr_contains_index(a)),\n        HirExprKind::MethodCall(recv, _, args) =>\n            hir_expr_contains_index(recv) || args.iter().any(|a| hir_expr_contains_index(a)),',
    '|| args.iter().any(hir_expr_contains_index),\n        HirExprKind::MethodCall(recv, _, args) =>\n            hir_expr_contains_index(recv) || args.iter().any(hir_expr_contains_index),'
)
changes.append('H12: redundant closure in contains_index')

hir = hir.replace(
    '}) || tail.as_ref().map_or(false, |e| hir_expr_contains_index(e))\n        }\n        HirExprKind::If(c, t, e) =>\n            hir_expr_contains_index(c) || hir_expr_contains_index(t)\n            || e.as_ref().map_or(false, |e| hir_expr_contains_index(e)),',
    '}) || tail.as_ref().is_some_and(|e| hir_expr_contains_index(e))\n        }\n        HirExprKind::If(c, t, e) =>\n            hir_expr_contains_index(c) || hir_expr_contains_index(t)\n            || e.as_ref().is_some_and(|e| hir_expr_contains_index(e)),'
)
changes.append('H13: map_or->is_some_and in contains_index')

if hir != hir_orig:
    with open(HIR, 'w') as f: f.write(hir)
    print('hir.rs written OK')
else: print('hir.rs: NO CHANGES')

print('--- changes ---')
for c in changes: print(' ', c)
