// AXON Aegis Monitor — test harness
// Provides stub implementations until axon_std is built (P3-07)

mod monitor;
use monitor::{ThreatLevel, Signal, classify};

fn main() {
    // Test the classify function directly
    let s0 = Signal { severity: 0, message: String::from("all clear"), layer: 0 };
    let s1 = Signal { severity: 1, message: String::from("advisory detected"), layer: 2 };
    let s2 = Signal { severity: 9, message: String::from("critical breach"), layer: 5 };

    println!("severity 0 → {:?}", classify(s0));
    println!("severity 1 → {:?}", classify(s1));
    println!("severity 9 → {:?}", classify(s2));
}
