# AXON Phase 7H — LLVM IR Type + Instruction Mapping + Emission

**Tests:** 82 (80 Phase 7 gate + 2 SSA collision tests)  
**Crates:** `axon_codegen/src/emit/`, `axon_codegen/src/phase7.rs`  
**Status:** CLOSED

---

## Overview

Phase 7H specifies the LLVM IR emission layer for AXON. It targets LLVM 15+ (opaque pointer model). Local variables are stack-allocated via `alloca`; SSA temporaries use a per-block monotonic counter. The full pipeline driver `run_phase7()` orchestrates all Phase 7 passes.

---

## LLVM Type System (`axon_codegen/src/emit/types.rs`)

### LlvmType

```rust
pub enum LlvmType {
    I1,                                              // bool
    Int(u32),                                        // i8, i16, i32, i64, i128
    Float(FloatWidth),                               // f32, f64
    Void,                                            // ()
    Ptr,                                             // ptr (LLVM 15+ opaque)
    Struct { name: Symbol, fields: Vec<LlvmType> },  // %struct.Name
    Array  { elem: Box<LlvmType>, len: usize },      // [N x T]
    Function { params: Vec<LlvmType>, ret: Box<LlvmType>, variadic: bool },
}
```

### emit_type() — AXON Type → LlvmType

| AXON Type | LLVM Type |
|-----------|-----------|
| `Primitive("i32")` | `i32` |
| `Primitive("bool")` | `i1` |
| `Primitive("char")` | `i32` |
| `Primitive("()")` | `void` |
| `Primitive("usize"/"isize")` | `i64` (64-bit target) |
| `Reference(_, _)` | `ptr` (opaque) |
| `Pointer(_, _)` | `ptr` (opaque) |
| `Named(name, _)` → known struct | `%struct.name { fields... }` |
| `Named(name, _)` → unknown | `ptr` |
| `Tuple(elems)` | `%struct.tuple { elem_types... }` |
| `Array(elem, len)` | `[len x elem_type]` |
| `Dyn(_)` | `ptr` (fat pointer via opaque ptr model) |

### dyn Trait Fat Pointer (Phase 8 production model)

In Phase 7, `dyn Trait` maps to `ptr` (simplified). Production Phase 8 will emit:
```llvm
%DynObject = type { ptr, ptr }   ; { data_ptr, vtable_ptr }
```
Full GEP-based vtable dispatch will be implemented in Phase 8C.

---

## LLVM Value Types (`axon_codegen/src/emit/ir.rs`)

```rust
pub enum LlvmValue {
    Local(u32),          // %N — unnamed SSA register (used internally)
    Named(Symbol),       // %name — named SSA register (places: %pN, temps: %tN)
    ConstInt(i64),       // integer literal
    ConstBool(bool),     // i1 literal (0 or 1)
    Null,                // null pointer
    Global(Symbol),      // @symbol — function or global reference
}
```

---

## LLVM Instructions (`axon_codegen/src/emit/ir.rs`)

| Instruction | LLVM IR Form |
|-------------|-------------|
| `Alloca { result, ty }` | `%result = alloca T` |
| `Load { result, ty, ptr }` | `%result = load T, ptr %ptr` |
| `Store { ty, val, ptr }` | `store T %val, ptr %ptr` |
| `GetElementPtr { result, ty, ptr, indices }` | `%result = getelementptr T, ptr %ptr, i32 0, i32 N` |
| `Call { result, ret_ty, callee, args }` | `%result = call ret_ty @callee(args...)` |
| `IndirectCall { result, fn_ty, fn_ptr, args }` | `%result = call ret_ty %fn_ptr(args...)` |
| `Br(label)` | `br label %label` |
| `CondBr { cond, true_bb, false_bb }` | `br i1 %cond, label %true, label %false` |
| `Ret { ty, val }` | `ret void` / `ret T %val` |
| `Label(sym)` | `sym:` |
| `Comment(text)` | `; text` |
| `ICmp { result, op, ty, lhs, rhs }` | `%result = icmp op T %lhs, %rhs` |
| `Select { result, cond, ty, true_val, false_val }` | `%result = select i1 %cond, T %true, T %false` |

---

## CfgStatement → LLVM Emission Rules

| CfgStatement | LLVM IR Emitted |
|-------------|-----------------|
| `StorageLive { p }` | `%pN = alloca T` |
| `StorageDead { p }` | *(nothing)* |
| `BorrowExpires { _ }` | *(nothing)* |
| `Assign { lhs, rhs }` | `%tN = load T, ptr %pRhs; store T %tN, ptr %pLhs` |
| `Borrow { result, lender }` | `store ptr %pLender, ptr %pResult` |
| `DropElaborated { p }` | `call void @TypeName_drop_glue(ptr %pN)` |
| `SetDropFlag { flag, val }` | `store i1 val, ptr %flagN` |
| `DynCall { site }` | `; DynCall: method_name` + `IndirectCall` |

**DropElaborated invariant:** Only inserted when `place_needs_drop()` is true (7E-5 drop elaboration). The drop glue function is always defined for types that need dropping.

**DynCall production note:** Phase 7 uses a simplified emission. Phase 8 will use:
```llvm
%vtable = load ptr, ptr %dyn_obj.vtable_slot
%fn_ptr = getelementptr inbounds %VtableTy, ptr %vtable, i32 0, i32 <method_idx>
%fn = load ptr, ptr %fn_ptr
call ret_ty %fn(args...)
```

### Terminator → LLVM

| Terminator | LLVM IR |
|-----------|---------|
| `Return { None }` | `ret void` |
| `Return { Some(p) }` | `%tN = load T, ptr %pN; ret T %tN` |
| `Goto { target }` | `br label %bbN` |
| `SwitchBool { disc, tt, ft }` | `%tN = load i1, ptr %pDisc; br i1 %tN, label %bbTT, label %bbFT` |
| `Unreachable` | `; unreachable` |

---

## SSA Register Naming (Critical — C-001/C-002 Fix)

### EmitCtx

```rust
struct EmitCtx {
    next_tmp: u32,
}

impl EmitCtx {
    fn new() -> Self { EmitCtx { next_tmp: 0 } }

    fn fresh_tmp(&mut self) -> LlvmValue {
        let n = self.next_tmp;
        self.next_tmp += 1;
        LlvmValue::Named(Symbol::intern(&format!("t{}", n)))
    }
}

fn place_alloca(place: PlaceId) -> LlvmValue {
    LlvmValue::Named(Symbol::intern(&format!("p{}", place.index())))
}
```

| Namespace | Pattern | Example | Purpose |
|-----------|---------|---------|---------|
| Place allocas | `%pN` | `%p0`, `%p1` | Stack slot pointer for place N |
| Temporaries | `%tN` | `%t0`, `%t1` | SSA temporaries, unique per block |
| Drop flags | `%flagN` | `%flag0` | Conditional drop flag |
| Block labels | `%bbN` | `%bb0`, `%bb1` | Basic block labels |

**EmitCtx is created fresh per basic block.** Temporaries are therefore unique within each block, satisfying LLVM's SSA requirement.

---

## LlvmModule Structure

```rust
pub struct LlvmModule {
    pub module_id:  Symbol,
    pub type_defs:  Vec<LlvmTypeDef>,   // %struct.Name = type { ... }
    pub functions:  Vec<LlvmFunction>,   // define ret_ty @name(params) { ... }
    pub globals:    Vec<LlvmGlobal>,     // @name = constant T value
}
```

---

## run_phase7() — Complete Pipeline Driver

```
Input:  HirModule + HashMap<DefId, FnDef> + DispatchTable
Output: Result<Phase7Result, Phase7Errors>

Pass 1: run_7e_passes() → OwnershipResult    (short-circuit on error)
Pass 2: run_7f_passes() → DynDispatchResult  (short-circuit on error)
Pass 3: run_7g_passes() → VerificationResult (short-circuit on error)
Pass 4: emit_module()   → LlvmModule         (only if all prior passes succeed)
```

### Phase7Result

```rust
pub struct Phase7Result {
    pub ownership:    OwnershipResult,
    pub dispatch:     DynDispatchResult,
    pub verification: VerificationResult,
    pub llvm_module:  LlvmModule,
}
```

---

## Contract Syntax (7H-2: axon_parser/src/contract_parser.rs)

### Grammar

```
expr     := and_expr ("||" and_expr)*
and_expr := unary  ("&&" unary)*
unary    := "!" unary | atom
atom     := "true" | "false" | ident compare_op value
           | ident "(" [ident ("," ident)*] ")"
           | "(" expr ")"
compare_op := ">=" | "<=" | "!=" | "==" | ">" | "<"   (longer tokens first)
value    := integer | "true" | "false" | "null" | ident
```

### parse_contract_spec()

```rust
pub fn parse_contract_spec(
    annotations: &[(String, String)],  // ("@requires", "x > 0"), ("@ensures", "result >= 0")
    fn_span: Span,
) -> Result<ContractSpec, Vec<ParseError>>
```

---

## CompilerError (7H-2: axon_codegen/src/error.rs)

```rust
pub enum CompilerError {
    InternalError       { message: String, span: Span },
    UnresolvedSymbol    { symbol: Symbol, span: Span },
    UnmonomorphizedType { type_name: Symbol, span: Span },
}

impl CompilerError {
    pub fn ice(message: impl Into<String>, span: Span) -> Self { ... }
}
```

All 5 `.expect()`/`panic!()` ICE paths in `axon_codegen/src/ownership/dispatch.rs` were replaced with `Err(CompilerError::ice(...))`.

---

## Exit Conditions Verified (82 tests)

- Type emission: all AXON types → correct LlvmType (10 tests)
- Instruction emission: alloca/load/store/call round-trip (10 tests)
- SSA uniqueness: multi-statement block → unique %tN registers (2 critical tests)
- Place/tmp namespace: %p0 ≠ %t0 (collision test)
- Ownership pipeline integration (10 tests)
- Dispatch pipeline integration (10 tests)
- Verification pipeline integration (10 tests)
- Contract parser (10 tests)
- Full run_phase7() pipeline (10 tests)
- Phase 7 regression (10 tests)

---

*Phase 7H — CLOSED*
