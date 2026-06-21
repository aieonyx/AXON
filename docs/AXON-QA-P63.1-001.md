# AXON-QA-P63.1-001 — Transformer Attention QA Audit
<!-- Copyright (c) 2026 Edison Lepiten / AIEONYX -->
<!-- INTERNAL ONLY — never referenced in public commit messages -->

## Phase
P63.1 — Transformer Attention in pure AXON (.ax source)

## Deliverables
- `crates/axon_ai_runtime/axon/attention.ax` — pure AXON Q/K/V attention
- `crates/axon_ai_runtime/src/attention_ffi.rs` — C-ABI FFI bridge
- `crates/axon_ai_runtime/tests/p63_1_attention_tests.rs` — 20 tests

## Test Matrix
| T1  | alloc/free lifecycle                            |
| T2  | set/get f32 roundtrip                           |
| T3  | dot product (1*4+2*5+3*6=32)                    |
| T4  | dot of zero vector                              |
| T5  | softmax sums to 1.0                             |
| T6  | softmax uniform → uniform output               |
| T7  | softmax numerically stable (large values)       |
| T8  | matmul identity matrix                          |
| T9  | matmul rectangular [2x3]*[3x2]=[2x2]           |
| T10 | scale_inplace                                   |
| T11 | add_inplace                                     |
| T12 | relu_inplace                                    |
| T13 | scores shape + all-finite                       |
| T14 | softmax per row, each sums to 1                 |
| T15 | uniform weights → mean of V rows               |
| T16 | E2E (seq=2, d_k=2): finite + positive           |
| T17 | full softmax row sum + weight ordering          |
| T18 | causal mask: upper-triangle → ~0, rows sum to 1|
| T19 | manual verify Q=K=I, V=[[1,2],[3,4]]           |
| T20 | null pointer guard on all FFI functions         |

## 3P Doctrine gate
- P1 Purpose: +Intelligence hardened — sovereign attention in .ax source
- P2 Pattern (internal): engine-mechanical — Q/K/V parallel projection heads
- P3 Practice: Law of Complementarity — FFI bridges existing axon_ai_runtime ops

## Post Doctrine 5-check
- [ ] Attribution Scrub
- [ ] Internal Language Scrub
- [ ] Spec Confidentiality
- [ ] Clean Commit
- [ ] Copyright: "Copyright (c) 2026 Edison Lepiten / AIEONYX"

## Deferred
ai_free_f32 length-tracked dealloc → P63.2 scope
