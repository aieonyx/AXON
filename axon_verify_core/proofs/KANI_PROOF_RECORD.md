# axon_verify_core Kani Proof Record
Tool: Kani v0.67.0 | Solver: Z3 v4.8.12 | Date: 2026-06-06
Result: 22 harnesses verified, 41 checks, 0 failures
VERIFICATION: SUCCESSFUL
The LLM is not in the Trusted Computing Base.

## Phase 22 M3 additions (EnsuresContract harnesses)

| Harness | Proves |
|---|---|
| empty_contract_has_no_witnesses | empty() always creates witness_count == 0 |
| add_witness_increments_count | add_witness increments count correctly |
| all_witnesses_valid_empty_is_false | empty contract fails validation |
| all_witnesses_valid_one_invalid_is_false | one invalid witness fails all |
| add_witness_capacity_limit | cannot exceed 8-witness capacity |

## Phase 22 M1 (prior record)
Tool: Kani v0.67.0 | Solver: Z3 v4.8.12 | Date: 2026-05-26
Result: 17 harnesses verified, 31 checks, 0 failures
VERIFICATION: SUCCESSFUL
