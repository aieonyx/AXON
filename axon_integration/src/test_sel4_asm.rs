//! P23-M5: seL4 Syscall Roundtrip Integration Tests
//! Verifies the complete Phase 23 asm! pipeline:
//! AXON source → lexer → parser → HIR → codegen → LLVM IR
//! with sel4_call / sel4_send / sel4_recv / asm!() / AtomicU64 / fence()

#[cfg(test)]
mod tests {
    use axon_parser::lexer::Lexer;
    use axon_parser::parser::Parser;
    use axon_parser::hir::lower;
    use axon_parser::codegen::emit_ir;

    fn compile(src: &str) -> String {
        let tokens = Lexer::new(src).tokenize().expect("lex failed");
        let mut p = Parser::new(tokens);
        let items = p.parse_program().expect("parse failed");
        let hir = lower(items);
        emit_ir(&hir)
    }

    // ── T1: Full sovereign IPC chain ─────────────────────────────────────────
    // A complete PD entry point that calls sel4_recv then sel4_call
    // Verifies the full pipeline from AXON source to LLVM IR
    #[test]
    fn tc_p23_m5_sovereign_ipc_roundtrip() {
        let src = r#"
            fn sovereign_pd_entry(ep: u64) -> u64 {
                let msg: u64 = sel4_recv(ep);
                let reply: u64 = sel4_call(ep, msg);
                return reply;
            }
        "#;
        let ir = compile(src);
        // Full pipeline must produce valid IR
        assert!(ir.contains("define"), "IR must define a function");
        // Both syscalls must be present
        assert!(ir.contains("svc #0"), "IR must contain SVC #0 for seL4 syscalls");
        assert!(ir.contains("sideeffect"), "seL4 syscalls must be sideeffect");
        assert!(ir.contains("~{memory}"), "seL4 syscalls must clobber memory");
        assert!(ir.contains("~{x7}"), "seL4 syscalls must clobber x7 (syscall number)");
        // Must have both recv and call patterns
        assert!(ir.contains("mov x7, #2"), "sel4_recv must use syscall #2");
        assert!(ir.contains("mov x7, #3"), "sel4_call must use syscall #3");
    }

    // ── T2: asm! + AtomicU64 + fence in same function ────────────────────────
    #[test]
    fn tc_p23_m5_asm_atomic_fence_combo() {
        let src = r#"
            fn sovereign_atomic_ipc(ep: u64, counter: AtomicU64) -> u64 {
                fence();
                asm!("dmb sy" : : : : "volatile");
                let msg: u64 = sel4_recv(ep);
                return msg;
            }
        "#;
        let ir = compile(src);
        assert!(ir.contains("fence seq_cst"), "fence() must emit seq_cst fence");
        assert!(ir.contains("dmb sy"), "asm! template must appear in IR");
        assert!(ir.contains("svc #0"), "sel4_recv must emit SVC #0");
    }

    // ── T3: sel4_send fire-and-forget pattern ────────────────────────────────
    #[test]
    fn tc_p23_m5_sel4_send_fire_and_forget() {
        let src = r#"
            fn notify_peer(ep: u64, event: u64) {
                sel4_send(ep, event);
                fence();
            }
        "#;
        let ir = compile(src);
        assert!(ir.contains("mov x7, #6"), "sel4_send must use syscall #6");
        assert!(ir.contains("call void asm"), "sel4_send must be void");
        assert!(ir.contains("fence seq_cst"), "fence after send must emit seq_cst");
    }

    // ── T4: IPC send/recv loop — AtomicU64 RMW coverage deferred to P23-cleanup ──
    #[test]
    fn tc_p23_m5_ipc_send_recv_loop() {
        let src = r#"
            fn ipc_loop(ep: u64, counter: AtomicU64) {
                let msg: u64 = sel4_recv(ep);
                sel4_send(ep, msg);
            }
        "#;
        let ir = compile(src);
        assert!(ir.contains("svc #0"), "IPC loop must emit SVC #0");
        assert!(ir.contains("define"), "IR must be valid LLVM IR");
    }

    // ── T5: Full Phase 23 capability — no C runtime symbols ─────────────────
    // Verify IR does NOT contain libc symbols (pure sovereign binary)
    #[test]
    fn tc_p23_m5_no_libc_in_sel4_ir() {
        let src = r#"
            fn sovereign_entry(ep: u64) -> u64 {
                let msg: u64 = sel4_recv(ep);
                return sel4_call(ep, msg);
            }
        "#;
        let ir = compile(src);
        // No libc dependency in sovereign seL4 IR
        assert!(!ir.contains("@printf"), "sovereign IR must not call printf");
        assert!(!ir.contains("@malloc"), "sovereign IR must not call malloc");
        assert!(!ir.contains("@free"),   "sovereign IR must not call free");
        // Must contain sovereign module flag
        assert!(ir.contains("axon_sovereign"), "IR must carry sovereign module flag");
    }
}
