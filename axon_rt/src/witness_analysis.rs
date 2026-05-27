// ============================================================
// axon_rt::witness_analysis — WitnessAnalyser trait
// Copyright © 2026 Edison Lepiten — AIEONYX
// SPEC: 6A-01 DWC
//
// Trait implemented by consumers of the witness stream.
//
// Phase 6: stub definition only.
// The Aegis AI Sentinel will implement this trait and subscribe
// to the WitnessStore drain channel in Phase 7.
// ============================================================

use crate::witness::WitnessRecord;

/// Implemented by any component that consumes the witness stream.
///
/// Phase 6: stub. The Aegis AI Sentinel implements this in Phase 7.
/// SPEC: 6A-01
pub trait WitnessAnalyser: Send + Sync {
    /// Called for each individual record as it is produced.
    /// Must be non-blocking — implementors must not call back into
    /// `axon_rt::store()` from this method (deadlock risk in Phase 7).
    fn on_record(&self, record: &WitnessRecord);

    /// Called when a batch of records is flushed from the store.
    /// Provides bulk processing for efficient Aegis ingestion.
    fn on_flush(&self, records: &[WitnessRecord]);
}

/// A no-op analyser used during testing and when no Aegis channel
/// is configured.
/// SPEC: 6A-01
pub struct NullAnalyser;

impl WitnessAnalyser for NullAnalyser {
    fn on_record(&self, _record: &WitnessRecord) {}
    fn on_flush(&self, _records: &[WitnessRecord]) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::witness::{
        ContractId, SourceLocation, Verdict, WitnessKind, WitnessRecord,
    };

    fn dummy_record() -> WitnessRecord {
        WitnessRecord {
            contract_id: ContractId::from_hash(1),
            kind:        WitnessKind::Pre,
            verdict:     Verdict::Pass,
            call_site:   SourceLocation { file: "t.ax", line: 1, column: 1 },
            timestamp:   0,
        }
    }

    #[test]
    fn test_null_analyser_does_not_panic() {
        let a = NullAnalyser;
        let r = dummy_record();
        a.on_record(&r);
        a.on_flush(&[]);
    }
}
