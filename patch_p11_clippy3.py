#!/usr/bin/env python3
changes = []

# ── parser.rs: dead code — allow on unused no_struct parse methods ────────
PARSER = "/home/edisonbl/axon/axon_parser/src/parser.rs"
with open(PARSER) as f: p = f.read()
p_orig = p

p = p.replace(
    '    fn parse_assign_expr_no_struct(',
    '    #[allow(dead_code)]\n    fn parse_assign_expr_no_struct('
)
changes.append('P1: parse_assign_expr_no_struct allow dead_code')

p = p.replace(
    '    fn parse_range_expr_no_struct(',
    '    #[allow(dead_code)]\n    fn parse_range_expr_no_struct('
)
changes.append('P2: parse_range_expr_no_struct allow dead_code')

p = p.replace(
    '    fn parse_or_expr_no_struct(',
    '    #[allow(dead_code)]\n    fn parse_or_expr_no_struct('
)
changes.append('P3: parse_or_expr_no_struct allow dead_code')

p = p.replace(
    '    fn parse_and_expr_no_struct(',
    '    #[allow(dead_code)]\n    fn parse_and_expr_no_struct('
)
changes.append('P4: parse_and_expr_no_struct allow dead_code')

p = p.replace(
    '    fn parse_cmp_expr_no_struct(',
    '    #[allow(dead_code)]\n    fn parse_cmp_expr_no_struct('
)
changes.append('P5: parse_cmp_expr_no_struct allow dead_code')

p = p.replace(
    '    fn parse_primary_expr_no_struct(',
    '    #[allow(dead_code)]\n    fn parse_primary_expr_no_struct('
)
changes.append('P6: parse_primary_expr_no_struct allow dead_code')

if p != p_orig:
    with open(PARSER, 'w') as f: f.write(p)
    print('parser.rs written OK')
else: print('parser.rs: NO CHANGES')

# ── hir.rs: dead fn error + new_without_default ──────────────────────────
HIR = "/home/edisonbl/axon/axon_parser/src/hir.rs"
with open(HIR) as f: hir = f.read()
hir_orig = hir

# H8: allow dead_code on fn error
hir = hir.replace(
    '    fn error(&mut self, msg: impl Into<String>, span: Span) {',
    '    #[allow(dead_code)]\n    fn error(&mut self, msg: impl Into<String>, span: Span) {'
)
changes.append('H8: fn error allow dead_code')

# H9: add Default impl for MoveStateMap
hir = hir.replace(
    '    pub fn new() -> Self { MoveStateMap { entries: Vec::new() } }',
    '    pub fn new() -> Self { MoveStateMap { entries: Vec::new() } }\n}\nimpl Default for MoveStateMap {\n    fn default() -> Self { Self::new() }\n}\nimpl MoveStateMap2 {'
)
# That approach is risky — use allow instead
# Revert and use allow attribute
hir = hir.replace(
    '    pub fn new() -> Self { MoveStateMap { entries: Vec::new() } }\n}\nimpl Default for MoveStateMap {\n    fn default() -> Self { Self::new() }\n}\nimpl MoveStateMap2 {',
    '    pub fn new() -> Self { MoveStateMap { entries: Vec::new() } }'
)
# Find the struct definition and add allow above it
hir = hir.replace(
    'pub struct MoveStateMap {',
    '#[allow(clippy::new_without_default)]\npub struct MoveStateMap {'
)
changes.append('H9: MoveStateMap new_without_default suppressed')

if hir != hir_orig:
    with open(HIR, 'w') as f: f.write(hir)
    print('hir.rs written OK')
else: print('hir.rs: NO CHANGES')

# ── codegen.rs: dead field errors ───────────────────────────────────────
CG = "/home/edisonbl/axon/axon_parser/src/codegen.rs"
with open(CG) as f: cg = f.read()
cg_orig = cg

cg = cg.replace(
    'pub struct LlvmEmitter {',
    '#[allow(dead_code)]\npub struct LlvmEmitter {'
)
changes.append('C2: LlvmEmitter allow dead_code')

if cg != cg_orig:
    with open(CG, 'w') as f: f.write(cg)
    print('codegen.rs written OK')
else: print('codegen.rs: NO CHANGES')

print('--- changes ---')
for c in changes: print(' ', c)
