// axon_parser/src/verify_integration.rs
// AXON Phase 22 — Formal Compiler Verification Integration
// Copyright © 2026 Edison Lepiten — AIEONYX
//
// Bridges axon_parser HIR contracts with axon_verify_core
// verification kernel. Tests that @ensures/@requires annotations
// in AXON source correctly map to verifiable postconditions.

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use crate::hir::{lower, HirItem, HirTy};
    use crate::parser::parse;
    use axon_verify_core::{
        check_ensures, check_dwc, VerifyOutcome,
        Witness, WitnessKind,
        enforce_ibi, EnforcementResult,
        BoundaryInvariant, Contract, InvariantTier,
    };

    // ── Phase 22 M1 ──────────────────────────────────────────────────────────

    #[test]
    fn tc_verify_core_check_ensures_pass() {
        // axon_verify_core::check_ensures correctly reports Pass
        let result = check_ensures(1, true);
        assert_eq!(result, VerifyOutcome::Pass,
            "check_ensures(label, true) must return Pass");
    }

    #[test]
    fn tc_verify_core_check_ensures_fail() {
        // axon_verify_core::check_ensures correctly reports Fail
        let result = check_ensures(1, false);
        assert_eq!(result, VerifyOutcome::Fail,
            "check_ensures(label, false) must return Fail");
    }

    #[test]
    fn tc_verify_core_dwc_valid_witness() {
        // Valid witness passes DWC check
        let w = Witness { kind: WitnessKind::Postcondition, valid: true, hash: 42 };
        assert_eq!(check_dwc(&w), VerifyOutcome::Pass);
    }

    #[test]
    fn tc_verify_core_dwc_invalid_witness() {
        // Invalid witness fails DWC check
        let w = Witness { kind: WitnessKind::Postcondition, valid: false, hash: 42 };
        assert_eq!(check_dwc(&w), VerifyOutcome::Fail);
    }

    #[test]
    fn tc_verify_core_ibi_constitutional_blocks_weakening() {
        // Constitutional invariants cannot be weakened — IBI enforces this
        let inv = BoundaryInvariant { id: 1, tier: InvariantTier::Constitutional };
        let con = Contract { invariant_id: 1, weakens_invariant: true };
        assert_eq!(enforce_ibi(&inv, &con), EnforcementResult::Block,
            "Constitutional invariant must block weakening");
    }

    #[test]
    fn tc_verify_core_ibi_operational_allows_weakening() {
        // Operational invariants can be updated with new proofs
        let inv = BoundaryInvariant { id: 2, tier: InvariantTier::Operational };
        let con = Contract { invariant_id: 2, weakens_invariant: true };
        assert_eq!(enforce_ibi(&inv, &con), EnforcementResult::Allow,
            "Operational invariant must allow weakening");
    }

    #[test]
    fn tc_hir_contract_requires_lowers() {
        // @requires annotation lowers to HirContract with Requires kind
        use crate::parser::ContractKind;
        use crate::hir::ContractExpr;
        let src = "@requires(x > 0) fn pos(x: i32) -> i32 { return x; }";
        let items = parse(src).expect("parse failed");
        let m = lower(items);
        assert_eq!(m.errors.len(), 0, "errors: {:?}", m.errors);
        if let HirItem::Fn(f) = &m.items[0] {
            assert_eq!(f.contracts.len(), 1);
            assert_eq!(f.contracts[0].kind, ContractKind::Requires);
            assert!(matches!(f.contracts[0].expr,
                ContractExpr::BinOp(_, _, _)),
                "requires condition must lower to BinOp");
        } else { panic!("expected fn"); }
    }

    #[test]
    fn tc_hir_contract_ensures_lowers() {
        // @ensures annotation lowers to HirContract with Ensures kind
        use crate::parser::ContractKind;
        let src = "@ensures(result > 0) fn pos(x: i32) -> i32 { return x; }";
        let items = parse(src).expect("parse failed");
        let m = lower(items);
        assert_eq!(m.errors.len(), 0, "errors: {:?}", m.errors);
        if let HirItem::Fn(f) = &m.items[0] {
            assert_eq!(f.contracts.len(), 1);
            assert_eq!(f.contracts[0].kind, ContractKind::Ensures);
        } else { panic!("expected fn"); }
    }

    #[test]
    fn tc_verify_pipeline_ensures_to_outcome() {
        // Full pipeline: parse @ensures → lower to HirContract →
        // evaluate condition with check_ensures → get VerifyOutcome
        // Simulates: fn add returns x+y, @ensures result >= x
        // Here we verify the condition (5 >= 3) = true → Pass
        let condition_holds = 5_i32 >= 3_i32;
        let outcome = check_ensures(42, condition_holds);
        assert_eq!(outcome, VerifyOutcome::Pass,
            "postcondition (5 >= 3) must verify as Pass");
    }

    #[test]
    fn tc_verify_pipeline_violated_ensures() {
        // Simulate a violated @ensures: result > 0 but result = -1
        let result: i32 = -1;
        let condition_holds = result > 0;
        let outcome = check_ensures(99, condition_holds);
        assert_eq!(outcome, VerifyOutcome::Fail,
            "violated postcondition must verify as Fail");
    }
}
