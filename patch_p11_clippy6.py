#!/usr/bin/env python3
changes = []

# ── axon_std.rs ──────────────────────────────────────────────────────────
STD = "/home/edisonbl/axon/axon_parser/src/axon_std.rs"
with open(STD) as f: s = f.read()
orig = s
# Move allow from struct to impl for AxonVec
s = s.replace('#[allow(clippy::new_without_default)]\npub struct AxonVec<T> {', 'pub struct AxonVec<T> {')
s = s.replace('impl<T> AxonVec<T> {', '#[allow(clippy::new_without_default)]\nimpl<T> AxonVec<T> {')
changes.append('S1: AxonVec allow on impl')
# Move allow from struct to impl for AxonString
s = s.replace('#[allow(clippy::new_without_default, clippy::should_implement_trait)]\npub struct AxonString {', 'pub struct AxonString {')
s = s.replace('impl AxonString {', '#[allow(clippy::new_without_default, clippy::should_implement_trait)]\nimpl AxonString {')
changes.append('S2: AxonString allow on impl')
# Fix Iter lifetime
s = s.replace('pub fn iter(&self) -> std::slice::Iter<T> {', 'pub fn iter(&self) -> std::slice::Iter<\'_, T> {')
changes.append('S3: Iter lifetime fixed')
if s != orig:
    with open(STD, 'w') as f: f.write(s)
    print('axon_std.rs written OK')
else: print('axon_std.rs: NO CHANGES')

# ── infer.rs ─────────────────────────────────────────────────────────────
INFER = "/home/edisonbl/axon/axon_parser/src/infer.rs"
with open(INFER) as f: s = f.read()
orig = s
for struct_name, impl_line in [
    ('Unifier', 'impl Unifier {'),
    ('TypeEnv', 'impl TypeEnv {'),
    ('ConstraintGen', 'impl ConstraintGen {'),
]:
    s = s.replace(f'#[allow(clippy::new_without_default)]\npub struct {struct_name} {{', f'pub struct {struct_name} {{')
    s = s.replace(impl_line, f'#[allow(clippy::new_without_default)]\n{impl_line}')
    changes.append(f'I: {struct_name} allow on impl')
if s != orig:
    with open(INFER, 'w') as f: f.write(s)
    print('infer.rs written OK')
else: print('infer.rs: NO CHANGES')

# ── codegen.rs ───────────────────────────────────────────────────────────
CG = "/home/edisonbl/axon/axon_parser/src/codegen.rs"
with open(CG) as f: s = f.read()
orig = s
# Move allow from struct to impl for LlvmEmitter
s = s.replace('#[allow(dead_code, clippy::new_without_default)]\npub struct LlvmEmitter {', '#[allow(dead_code)]\npub struct LlvmEmitter {')
s = s.replace('impl LlvmEmitter {', '#[allow(clippy::new_without_default)]\nimpl LlvmEmitter {')
changes.append('C1: LlvmEmitter allow on impl')
# Fix useless format: &format!("-march=nvptx64") -> "-march=nvptx64"
s = s.replace('&format!("-march=nvptx64")', '"-march=nvptx64"')
changes.append('C2: useless format removed')
# Fix borrowed &String args: &obj_path -> obj_path.as_str(), &ll_path -> ll_path.as_str()
s = s.replace('"-filetype=obj", "-o", &obj_path, &ll_path', '"-filetype=obj", "-o", obj_path.as_str(), ll_path.as_str()')
changes.append('C3: &String -> .as_str()')
if s != orig:
    with open(CG, 'w') as f: f.write(s)
    print('codegen.rs written OK')
else: print('codegen.rs: NO CHANGES')

# ── profile.rs ───────────────────────────────────────────────────────────
PROF = "/home/edisonbl/axon/axon_parser/src/profile.rs"
with open(PROF) as f: s = f.read()
orig = s
# Move allow from struct to impl for Capability and Profile
s = s.replace('#[allow(clippy::should_implement_trait)]\nimpl Capability {', 'impl Capability {')
s = s.replace('#[allow(clippy::should_implement_trait)]\nimpl Profile {', 'impl Profile {')
# Allow on specific fn from_str instead
s = s.replace('impl Capability {\n    pub fn from_str(', 'impl Capability {\n    #[allow(clippy::should_implement_trait)]\n    pub fn from_str(')
s = s.replace('impl Profile {\n    pub fn from_str(', 'impl Profile {\n    #[allow(clippy::should_implement_trait)]\n    pub fn from_str(')
changes.append('PR1: from_str allow on fn')
# CompilerArgs::default -> new, and add Default trait impl
s = s.replace('#[allow(clippy::new_without_default)]\npub struct CompilerArgs {', 'pub struct CompilerArgs {')
s = s.replace('impl CompilerArgs {\n    pub fn default() -> Self {', 'impl Default for CompilerArgs {\n    fn default() -> Self {')
changes.append('PR2: CompilerArgs Default trait impl')
if s != orig:
    with open(PROF, 'w') as f: f.write(s)
    print('profile.rs written OK')
else: print('profile.rs: NO CHANGES')

print('--- changes ---')
for c in changes: print(' ', c)
