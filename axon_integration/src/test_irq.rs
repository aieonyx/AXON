// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Phase 38 integration tests — IRQ dispatch layer.

use axon_sel4::irq::{IrqHandler, IrqTable, IrqError, irq_register, irq_ack, irq_clear, irq_poll};

fn make_handler(irq_num: u32) -> IrqHandler {
    IrqHandler { cap: irq_num as u64 + 100, irq_num, notification: irq_num as u64 + 200 }
}

#[test]
fn p38_irq_register_returns_correct_handler() {
    let h = irq_register(1, 42, 15, 25).unwrap();
    assert_eq!(h.irq_num, 42);
    assert_eq!(h.cap, 15);
    assert_eq!(h.notification, 25);
}

#[test]
fn p38_irq_ack_ok_on_host() {
    let h = make_handler(10);
    assert!(irq_ack(&h).is_ok());
}

#[test]
fn p38_irq_clear_ok_on_host() {
    let h = make_handler(11);
    assert!(irq_clear(&h).is_ok());
}

#[test]
fn p38_irq_poll_zero_on_host() {
    let h = make_handler(12);
    assert_eq!(irq_poll(&h), 0);
}

#[test]
fn p38_irq_table_full_lifecycle() {
    let mut table = IrqTable::new();
    assert!(table.is_empty());
    for i in 0..16u32 {
        table.register(make_handler(i)).unwrap();
    }
    assert_eq!(table.len(), 16);
    for i in 0..16u32 {
        let h = table.find(i).unwrap();
        assert_eq!(h.irq_num, i);
    }
}

#[test]
fn p38_irq_table_dispatch_no_pending_on_host() {
    let mut table = IrqTable::new();
    for i in 0..4u32 { table.register(make_handler(i)).unwrap(); }
    let mut count = 0u32;
    table.dispatch_poll(|_, _| { count += 1; });
    assert_eq!(count, 0);
}

#[test]
fn p38_irq_error_variants_distinct() {
    assert_ne!(IrqError::InvalidIrq(1), IrqError::InvalidIrq(2));
    assert_ne!(IrqError::SeL4Error(1),  IrqError::BindFailed);
}
