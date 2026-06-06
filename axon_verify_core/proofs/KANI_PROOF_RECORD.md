# axon_verify_core Kani Proof Record

## Last verified Kani run
Tool: Kani v0.67.0 | Solver: Z3 v4.8.12 | Date: 2026-05-26
Result: 17 harnesses verified, 31 checks, 0 failures
VERIFICATION: SUCCESSFUL
The LLM is not in the Trusted Computing Base.

## Phase 22 M3 additions (PENDING Kani run)
Date added: 2026-06-06
Status: Harnesses written and unit-tested. Kani re-run required before NLNet submission.

| Harness | Location | Proves |
|---|---|---|
| empty_contract_has_no_witnesses | contract.rs | empty() creates zero-witness contract |
| add_witness_increments_count | contract.rs | add_witness increments count correctly |
| all_witnesses_valid_empty_is_false | contract.rs | empty contract fails validation |
| all_witnesses_valid_one_invalid_is_false | contract.rs | one invalid fails all |
| add_witness_capacity_limit | contract.rs | 8-witness capacity limit enforced |

## To re-run Kani (required before NLNet submission)
cargo kani --harness empty_contract_has_no_witnesses -p axon_verify_core
cargo kani --harness add_witness_increments_count -p axon_verify_core
cargo kani --harness all_witnesses_valid_empty_is_false -p axon_verify_core
cargo kani --harness all_witnesses_valid_one_invalid_is_false -p axon_verify_core
cargo kani --harness add_witness_capacity_limit -p axon_verify_core

## Note
The 2026-05-26 run is the authoritative verified record.
Phase 22 harnesses are structurally correct (unit tests pass) but
require a Kani invocation to produce a formal proof certificate.
