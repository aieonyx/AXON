# AXON Compiler Verification Report
**Project:** AXON Compiler — axon_verify_core  
**Author:** Edison Lepiten / AIEONYX  
**Date:** 2026-06-06  
**NLNet Grant:** AXON Compiler (submitted 2026-05-21)  
**Review Window:** August–October 2026  

---

## Executive Summary

The AXON compiler formal verification kernel (`axon_verify_core`) has been
formally verified using Kani bounded model checking. This report documents
the verification scope, methodology, results, and integration with the
AXON compiler pipeline.

**Verification result: SUCCESSFUL — 22 harnesses verified, 0 failures (2026-06-06)**  
**Bug fixed by Kani: all_witnesses_valid() empty contract case — proof-driven fix.**

---

## Verification Scope

### Trusted Computing Base (TCB)

`axon_verify_core` is the innermost ring of the AXON security model.

Constitutional rules:
1. Zero external dependencies — depends only on `core`
2. Every public function verified by Kani bounded model checking
3. Proofs committed alongside source in `proofs/`
4. The LLM is NEVER in the Trusted Computing Base
5. Any modification requires a new proof before merge
6. `#![no_std]` — no heap allocation in the TCB
7. `#![deny(unsafe_code)]` — no unsafe code, ever

### Verified Components

| Module | Functions | Harnesses | Properties Proved |
|---|---|---|---|
| `checker.rs` | `check_ensures`, `check_dwc`, `check_qcc` | 10 | Determinism, correctness, no panics |
| `enforcer.rs` | `enforce_ibi`, `validate_witness` | 7 | Constitutional invariant protection |
| `contract.rs` | `EnsuresContract`, `Witness`, `Contract` | 5 | Capacity limits, validity, correctness |

---

## Verification Methodology

### Tool: Kani v0.67.0

Kani is a bounded model checker for Rust that uses the CBMC backend
with Z3 as the SMT solver. It exhaustively verifies properties over
all possible inputs within a bounded domain.

### Properties Verified

**`check_ensures`:**
- For all `label: u32` and `ok: bool`:
  - `check_ensures(label, true)` → always `Pass`
  - `check_ensures(label, false)` → always `Fail`
  - Deterministic: same inputs always produce same output
  - No panics for any input

**`enforce_ibi` (Immutability-by-Inference):**
- Constitutional invariants ALWAYS block weakening (for all IDs)
- Non-weakening changes to constitutional invariants ALWAYS pass
- Operational invariants permit weakening
- Advisory invariants permit weakening
- Contracts targeting different invariant IDs always pass

**`validate_witness` (Dynamic Witness Contract):**
- Empty contracts always fail validation
- Single valid witness passes
- Single invalid witness fails
- Hash field is irrelevant to outcome

**`EnsuresContract`:**
- `empty()` always creates zero-witness contract
- `add_witness` correctly increments count
- Capacity limit of 8 witnesses is enforced
- `all_witnesses_valid` correctly aggregates witness validity

---

## Integration with AXON Compiler Pipeline

### Phase 22 Deliverables

1. **Integration tests** (Phase 22 M1): `axon_parser` imports
   `axon_verify_core` as a dev-dependency. 10 integration tests
   verify that HIR contract annotations (`@requires`, `@ensures`)
   correctly map to `axon_verify_core` types.

2. **Postcondition pipeline** (Phase 22 M2): The AXON compiler emits
   `axon_ensures_check` calls in LLVM IR for every `@ensures`-annotated
   function. These markers enable runtime postcondition verification
   and serve as proof obligations for the formal verifier.

3. **Kani harnesses** (Phase 22 M3): 5 new harnesses added for
   `EnsuresContract` — covering capacity limits, witness validity
   aggregation, and empty contract rejection.

### Contract Flow
AXON source (@ensures annotation)
↓ parse
Item::Fn with contracts: Vec<Contract>
↓ lower
HirFn with contracts: Vec<HirContract>
↓ codegen
LLVM IR: call void @axon_ensures_check(i32 label, i1 condition)
↓ runtime
axon_verify_core::check_ensures(label, condition) → VerifyOutcome
---

## Proof Record

See `proofs/KANI_PROOF_RECORD.md` for the complete proof record.

| Phase | Harnesses | Checks | Failures | Date |
|---|---|---|---|---|
| Phase 22 M1 (baseline) | 17 | 31 | 0 | 2026-05-26 |
| Phase 22 M3 (verified) | 22 | 31 | 0 | 2026-06-06 |

---

## Constitutional Invariant Protection

The IBI (Immutability-by-Inference) enforcement in `enforcer.rs` provides
a formally verified guarantee:

> **No change can weaken a Constitutional invariant.**
> This holds for ALL possible invariant IDs and ALL possible contracts,
> as proved by the `enforce_ibi_constitutional_block` Kani harness.

This is the core security property of the AXON constitutional model —
once a Constitutional invariant is established, no code change can
silently remove it.

---

## Sovereign Alignment

This verification kernel embodies the AXON mission:

> "We are not users. We are not accounts. We are not products. We are people."

The TCB serves the sovereign individual — not the system's convenience.
The LLM is explicitly excluded from the TCB. Every security property
is proved, not assumed.

---

*Copyright © 2026 Edison Lepiten / AIEONYX*  
*Licensed under Apache 2.0*
