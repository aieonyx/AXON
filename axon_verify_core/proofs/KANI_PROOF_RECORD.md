# axon_verify_core Kani Proof Record

## Latest verified Kani run
Tool: Kani v0.67.0 | Solver: Z3 v4.8.12 | Date: 2026-06-06
Result: 22 harnesses verified, 31 checks, 0 failures
VERIFICATION: SUCCESSFUL
The LLM is not in the Trusted Computing Base.

## Bug fixed during Phase 22 Kani run
`all_witnesses_valid()` incorrectly returned `true` for empty contracts.
Fixed: added early return `false` when `witness_count == 0`.
Kani harness `all_witnesses_valid_empty_is_false` detected this — proof-driven bug fix.

## Harness inventory

### checker.rs (10 harnesses)
| Harness | Proves |
|---|---|
| check_ensures_pass | check_ensures(any, true) always returns Pass |
| check_ensures_fail | check_ensures(any, false) always returns Fail |
| check_ensures_deterministic | same inputs always produce same output |
| check_dwc_valid | valid witness always passes DWC |
| check_dwc_invalid | invalid witness always fails DWC |
| check_dwc_hash_irrelevant | hash field does not affect outcome |
| check_qcc_sufficient | two valid witnesses satisfy quorum of 2 |
| check_qcc_insufficient | one valid witness does not satisfy quorum of 2 |
| check_qcc_zero_required | quorum of 0 always passes |
| (additional) | see checker.rs |

### enforcer.rs (7 harnesses)
| Harness | Proves |
|---|---|
| enforce_ibi_constitutional_block | Constitutional invariants ALWAYS block weakening (all u32 IDs) |
| enforce_ibi_constitutional_allow_non_weakening | Non-weakening changes always pass |
| enforce_ibi_operational_allows_weakening | Operational invariants permit weakening |
| enforce_ibi_advisory_allows_weakening | Advisory invariants permit weakening |
| enforce_ibi_different_id_allows | Different invariant IDs always pass |
| validate_witness_empty_contract | empty contract always rejected |
| validate_witness_single_valid | single valid witness accepted |
| validate_witness_single_invalid | single invalid witness rejected |

### contract.rs (5 harnesses) — Phase 22 M3
| Harness | Proves |
|---|---|
| empty_contract_has_no_witnesses | empty() creates zero-witness contract |
| add_witness_increments_count | add_witness increments count (any hash) |
| all_witnesses_valid_empty_is_false | empty contract fails validation |
| all_witnesses_valid_one_invalid_is_false | one invalid witness fails all |
| add_witness_capacity_limit | 8-witness capacity limit enforced |

## Prior record
Tool: Kani v0.67.0 | Solver: Z3 v4.8.12 | Date: 2026-05-26
Result: 17 harnesses verified, 31 checks, 0 failures
