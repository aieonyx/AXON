// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! seL4 IRQ dispatch layer — interrupt registration, handler dispatch, IRQ caps.
//!
//! seL4 IRQ model:
//!   - Hardware IRQs are routed to Protection Domains via seL4 notification objects.
//!   - A PD registers an IRQ number → receives an IrqHandler wrapping the IRQ cap.
//!   - When the IRQ fires, seL4 signals the notification → PD wakes via sel4_wait.
//!   - After handling, the PD must acknowledge the IRQ via irq_ack().
//!
//! On aarch64-seL4: real SVC #0 syscalls via inline asm.
//! On host (x86_64): safe stubs for unit testing.

use crate::types::{Cap, Badge};
use crate::ipc::{sel4_wait, sel4_poll};

// ── seL4 IRQ syscall numbers (aarch64) ───────────────────────────────────────
// Source: seL4 manual — IRQControl and IRQHandler invocations.
#[allow(dead_code)]
mod irq_sys {
    /// IRQControl_Get — obtain IRQ handler cap for a given IRQ number.
    pub const IRQ_CONTROL_GET:    u64 = 1;
    /// IRQHandler_Ack — acknowledge IRQ after handling.
    pub const IRQ_HANDLER_ACK:    u64 = 0;
    /// IRQHandler_SetNotification — bind IRQ to a notification object.
    pub const IRQ_HANDLER_SET_NTF: u64 = 1;
    /// IRQHandler_Clear — unbind and release IRQ handler.
    pub const IRQ_HANDLER_CLEAR:  u64 = 2;
}

/// An IRQ handler — wraps a seL4 IRQ capability slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IrqHandler {
    /// seL4 capability slot for this IRQ handler.
    pub cap: Cap,
    /// Hardware IRQ number this handler covers.
    pub irq_num: u32,
    /// Notification object bound to this IRQ.
    pub notification: Cap,
}

/// IRQ dispatch error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrqError {
    /// IRQ number out of range or not available.
    InvalidIrq(u32),
    /// seL4 syscall returned non-zero error code.
    SeL4Error(u64),
    /// Notification binding failed.
    BindFailed,
}

/// IRQ dispatch result.
pub type IrqResult<T> = Result<T, IrqError>;

/// Register an IRQ number and bind it to a notification object.
///
/// On aarch64-seL4: invokes IRQControl_Get to obtain the IRQ handler cap,
/// then IRQHandler_SetNotification to bind it to the notification.
/// On host: returns a stub IrqHandler for testing.
///
/// # Arguments
/// * `irq_control` — capability to the IRQControl object (slot 1 in root CNode).
/// * `irq_num`     — hardware IRQ number to register.
/// * `dest_slot`   — CNode slot to receive the new IRQ handler cap.
/// * `notification`— notification object cap to bind the IRQ to.
pub fn irq_register(
    _irq_control: Cap,
    irq_num: u32,
    dest_slot: Cap,
    notification: Cap,
) -> IrqResult<IrqHandler> {
    #[cfg(target_arch = "aarch64")]
    {
        // Step 1: IRQControl_Get — carve out IRQ handler cap
        let err = irq_control_get(irq_control, irq_num as u64, dest_slot);
        if err != 0 { return Err(IrqError::SeL4Error(err)); }
        // Step 2: IRQHandler_SetNotification — bind to notification
        let err = irq_handler_set_notification(dest_slot, notification);
        if err != 0 { return Err(IrqError::BindFailed); }
    }
    Ok(IrqHandler { cap: dest_slot, irq_num, notification })
}

/// Acknowledge an IRQ after handling — re-arms the interrupt line.
///
/// Must be called after every IRQ handling cycle or the IRQ will not fire again.
pub fn irq_ack(_handler: &IrqHandler) -> IrqResult<()> {
    #[cfg(target_arch = "aarch64")]
    {
        let err = irq_handler_ack(handler.cap);
        if err != 0 { return Err(IrqError::SeL4Error(err)); }
    }
    Ok(())
}

/// Block until the IRQ fires. Returns the notification badge.
///
/// Calls sel4_wait on the bound notification object.
/// The caller must call irq_ack() after processing.
pub fn irq_wait(handler: &IrqHandler) -> Badge {
    sel4_wait(handler.notification)
}

/// Non-blocking IRQ check. Returns badge if pending, 0 if not.
pub fn irq_poll(handler: &IrqHandler) -> Badge {
    sel4_poll(handler.notification)
}

/// Unbind and release an IRQ handler cap.
pub fn irq_clear(_handler: &IrqHandler) -> IrqResult<()> {
    #[cfg(target_arch = "aarch64")]
    {
        let err = irq_handler_clear(handler.cap);
        if err != 0 { return Err(IrqError::SeL4Error(err)); }
    }
    Ok(())
}

// ── aarch64 seL4 IRQ syscall wrappers ────────────────────────────────────────

/// IRQControl_Get — obtain IRQ handler cap for irq_num into dest_slot.
#[cfg(target_arch = "aarch64")]
fn irq_control_get(irq_control: Cap, irq_num: u64, dest_slot: Cap) -> u64 {
    unsafe {
        let result: u64;
        core::arch::asm!(
            "mov x7, {label}",
            "svc #0",
            label = const irq_sys::IRQ_CONTROL_GET,
            inout("x0") irq_control => result,
            in("x1") irq_num,
            in("x2") dest_slot,
            lateout("x7") _,
            options(nostack),
        );
        result
    }
}

/// IRQHandler_Ack — acknowledge IRQ, re-arm interrupt line.
#[cfg(target_arch = "aarch64")]
fn irq_handler_ack(handler_cap: Cap) -> u64 {
    unsafe {
        let result: u64;
        core::arch::asm!(
            "mov x7, {label}",
            "svc #0",
            label = const irq_sys::IRQ_HANDLER_ACK,
            inout("x0") handler_cap => result,
            lateout("x7") _,
            options(nostack),
        );
        result
    }
}

/// IRQHandler_SetNotification — bind IRQ handler to notification object.
#[cfg(target_arch = "aarch64")]
fn irq_handler_set_notification(handler_cap: Cap, notification: Cap) -> u64 {
    unsafe {
        let result: u64;
        core::arch::asm!(
            "mov x7, {label}",
            "svc #0",
            label = const irq_sys::IRQ_HANDLER_SET_NTF,
            inout("x0") handler_cap => result,
            in("x1") notification,
            lateout("x7") _,
            options(nostack),
        );
        result
    }
}

/// IRQHandler_Clear — unbind and release IRQ handler.
#[cfg(target_arch = "aarch64")]
fn irq_handler_clear(handler_cap: Cap) -> u64 {
    unsafe {
        let result: u64;
        core::arch::asm!(
            "mov x7, {label}",
            "svc #0",
            label = const irq_sys::IRQ_HANDLER_CLEAR,
            inout("x0") handler_cap => result,
            lateout("x7") _,
            options(nostack),
        );
        result
    }
}

// ── IRQ dispatch table ────────────────────────────────────────────────────────

/// Maximum number of registered IRQ handlers.
pub const MAX_IRQ_HANDLERS: usize = 64;

/// A simple static IRQ dispatch table.
/// Maps IRQ numbers to registered handlers.
pub struct IrqTable {
    handlers: [Option<IrqHandler>; MAX_IRQ_HANDLERS],
    count: usize,
}

impl IrqTable {
    pub const fn new() -> Self {
        Self {
            handlers: [None; MAX_IRQ_HANDLERS],
            count: 0,
        }
    }

    /// Register a handler in the table.
    pub fn register(&mut self, handler: IrqHandler) -> IrqResult<()> {
        if self.count >= MAX_IRQ_HANDLERS {
            return Err(IrqError::InvalidIrq(handler.irq_num));
        }
        self.handlers[self.count] = Some(handler);
        self.count += 1;
        Ok(())
    }

    /// Look up handler by IRQ number.
    pub fn find(&self, irq_num: u32) -> Option<&IrqHandler> {
        self.handlers[..self.count]
            .iter()
            .filter_map(|h| h.as_ref())
            .find(|h| h.irq_num == irq_num)
    }

    /// Dispatch: poll all registered handlers, call closure for each pending IRQ.
    pub fn dispatch_poll<F: FnMut(&IrqHandler, Badge)>(&self, mut f: F) {
        self.handlers[..self.count]
            .iter()
            .flatten()
            .for_each(|h| {
                let badge = irq_poll(h);
                if badge != 0 { f(h, badge); }
            });
    }

    /// Number of registered handlers.
    pub fn len(&self) -> usize { self.count }

    /// True if no handlers registered.
    pub fn is_empty(&self) -> bool { self.count == 0 }
}

impl Default for IrqTable {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_handler(irq_num: u32) -> IrqHandler {
        IrqHandler { cap: irq_num as u64 + 100, irq_num, notification: irq_num as u64 + 200 }
    }

    #[test]
    fn tp38_irq_register_stub() {
        // On host: irq_register always succeeds (no seL4 kernel)
        let h = irq_register(1, 32, 10, 20).unwrap();
        assert_eq!(h.irq_num, 32);
        assert_eq!(h.cap, 10);
        assert_eq!(h.notification, 20);
    }

    #[test]
    fn tp38_irq_ack_stub() {
        let h = make_handler(33);
        assert!(irq_ack(&h).is_ok());
    }

    #[test]
    fn tp38_irq_clear_stub() {
        let h = make_handler(34);
        assert!(irq_clear(&h).is_ok());
    }

    #[test]
    fn tp38_irq_poll_returns_zero_on_host() {
        let h = make_handler(35);
        assert_eq!(irq_poll(&h), 0);
    }

    #[test]
    fn tp38_irq_table_register_and_find() {
        let mut table = IrqTable::new();
        let h = make_handler(32);
        table.register(h).unwrap();
        let found = table.find(32).unwrap();
        assert_eq!(found.irq_num, 32);
        assert_eq!(found.cap, 132);
    }

    #[test]
    fn tp38_irq_table_find_missing() {
        let table = IrqTable::new();
        assert!(table.find(99).is_none());
    }

    #[test]
    fn tp38_irq_table_multiple_handlers() {
        let mut table = IrqTable::new();
        for i in 0..8u32 {
            table.register(make_handler(i)).unwrap();
        }
        assert_eq!(table.len(), 8);
        for i in 0..8u32 {
            assert!(table.find(i).is_some());
        }
    }

    #[test]
    fn tp38_irq_table_dispatch_poll_no_pending() {
        let mut table = IrqTable::new();
        table.register(make_handler(40)).unwrap();
        let mut called = false;
        table.dispatch_poll(|_, _| { called = true; });
        // On host sel4_poll always returns 0 — no dispatch expected
        assert!(!called);
    }

    #[test]
    fn tp38_irq_error_display() {
        let e = IrqError::InvalidIrq(99);
        assert_eq!(e, IrqError::InvalidIrq(99));
        let e2 = IrqError::SeL4Error(7);
        assert_eq!(e2, IrqError::SeL4Error(7));
    }

    #[test]
    fn tp38_irq_table_is_empty() {
        let table = IrqTable::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn tp38_irq_syscall_numbers_stable() {
        assert_eq!(irq_sys::IRQ_CONTROL_GET,     1);
        assert_eq!(irq_sys::IRQ_HANDLER_ACK,     0);
        assert_eq!(irq_sys::IRQ_HANDLER_SET_NTF, 1);
        assert_eq!(irq_sys::IRQ_HANDLER_CLEAR,   2);
    }
}
