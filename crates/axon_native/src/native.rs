// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// x86_64 native code emission bridge — mirrors native.ax exactly.
// System V AMD64 ABI. Stack-machine expression evaluation.
// Excised at P55 bootstrap when axonc compiles native.ax natively.
//
// Closes DEFER-P53-001: if/else with conditional jumps implemented.

use axon_hir::{HirBinOp, HirExpr, HirFn, HirProgram, HirStmt, HirUnaryOp};
use axon_infer::{infer_program, InferredFn, InferredProgram};
use axon_std_string::AxString;
use crate::error::{NativeError, NativeResult};
use crate::x86::*;

// ── Emission context ──────────────────────────────────────────────────────────

struct NativeCtx {
    code:        Vec<u8>,
    /// Param name → reg_id (rdi=7, rsi=6, rdx=2)
    param_regs:  Vec<(String, u8)>,
    /// Let binding name → stack slot index
    local_slots: Vec<(String, usize)>,
    /// Function name → start offset in code
    fn_offsets:  Vec<(String, usize)>,
    /// Pending call relocations: (rel32_field_offset, fn_name)
    relocs:      Vec<(usize, String)>,
}

impl NativeCtx {
    fn new() -> Self {
        NativeCtx {
            code:        Vec::new(),
            param_regs:  Vec::new(),
            local_slots: Vec::new(),
            fn_offsets:  Vec::new(),
            relocs:      Vec::new(),
        }
    }

    fn emit(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    fn pos(&self) -> usize { self.code.len() }

    fn patch_i32(&mut self, pos: usize, val: i32) {
        let bytes = val.to_le_bytes();
        self.code[pos..pos+4].copy_from_slice(&bytes);
    }
}

// ── Public entry points ───────────────────────────────────────────────────────

/// Compile an AXON source string to raw x86_64 machine bytes.
pub fn native_codegen_source(source: &str) -> NativeResult<Vec<u8>> {
    let ast = axon_parse::parse(source).map_err(|e| {
        NativeError::PipelineError(AxString::ax_from_str(&format!("parse: {}", e)))
    })?;
    let hir = axon_hir::lower_program(&ast).map_err(|e| {
        NativeError::PipelineError(AxString::ax_from_str(&format!("hir: {}", e)))
    })?;
    let typed = infer_program(&hir).map_err(|e| {
        NativeError::PipelineError(AxString::ax_from_str(&format!("infer: {}", e)))
    })?;
    emit_program(&hir, &typed)
}

/// Emit machine bytes for a HirProgram + InferredProgram.
pub fn emit_program(hir: &HirProgram, typed: &InferredProgram) -> NativeResult<Vec<u8>> {
    let mut ctx = NativeCtx::new();

    for (hir_fn, inf_fn) in hir.fns.iter().zip(typed.fns.iter()) {
        emit_fn(&mut ctx, hir_fn, inf_fn)?;
    }

    apply_relocations(&mut ctx);
    Ok(ctx.code)
}

// ── Function emission ─────────────────────────────────────────────────────────

fn count_locals(stmts: &[HirStmt]) -> usize {
    stmts.iter().filter(|s| matches!(s, HirStmt::Let { .. })).count()
}

fn block_returns(stmts: &[HirStmt]) -> bool {
    stmts.last().map(|s| matches!(s, HirStmt::Return(_))).unwrap_or(false)
}

fn emit_fn(ctx: &mut NativeCtx, f: &HirFn, _inf: &InferredFn) -> NativeResult<()> {
    // Record function start for call relocation
    ctx.fn_offsets.push((f.name.as_str().to_string(), ctx.pos()));

    // Pre-scan for local count (needed before prologue)
    let local_count = count_locals(&f.body);

    // Reset per-function state
    ctx.param_regs.clear();
    ctx.local_slots.clear();

    // Register params: rdi(7), rsi(6), rdx(2) for first three
    let param_reg_ids = [7u8, 6, 2];
    for (i, p) in f.params.iter().enumerate() {
        if i < param_reg_ids.len() {
            ctx.param_regs.push((p.name.as_str().to_string(), param_reg_ids[i]));
        }
    }

    // Prologue
    ctx.emit(&push_rbp());
    ctx.emit(&mov_rbp_rsp());
    if local_count > 0 {
        ctx.emit(&sub_rsp_imm8((local_count * 8) as u8));
    }

    // Body — each Return emits its own epilogue
    for stmt in &f.body {
        emit_stmt(ctx, stmt)?;
    }

    Ok(())
}

// ── Statement emission ────────────────────────────────────────────────────────

fn emit_stmt(ctx: &mut NativeCtx, stmt: &HirStmt) -> NativeResult<()> {
    match stmt {
        HirStmt::Let { name, value, .. } => {
            emit_expr(ctx, value)?;              // result → rax
            let slot = ctx.local_slots.len();
            ctx.emit(&store_rax_rbp_slot(slot)); // store rax → [rbp-slot]
            ctx.local_slots.push((name.as_str().to_string(), slot));
            Ok(())
        }
        HirStmt::Return(expr) => {
            emit_expr(ctx, expr)?;    // result → rax
            ctx.emit(&mov_rsp_rbp()); // restore stack pointer
            ctx.emit(&pop_rbp());     // restore frame pointer
            ctx.emit(&ret_byte());    // return
            Ok(())
        }
        HirStmt::ExprStmt(expr) => {
            emit_expr(ctx, expr)?;
            Ok(())
        }
        // P55.5: ephemeral — same as Let for native codegen; zeroize on drop at P55.6
        HirStmt::LetAt { name, value, .. } => {
            emit_expr(ctx, value)?;
            let slot = ctx.local_slots.len();
            ctx.emit(&push_rax());
            ctx.local_slots.push((name.clone(), slot));
            Ok(())
        }
    }
}

// ── Expression emission ───────────────────────────────────────────────────────

fn emit_expr(ctx: &mut NativeCtx, expr: &HirExpr) -> NativeResult<()> {
    match expr {
        HirExpr::IntLit(n) => {
            ctx.emit(&mov_rax_imm32(*n as i32));
            Ok(())
        }
        HirExpr::BoolLit(b) => {
            ctx.emit(&mov_rax_imm32(if *b { 1 } else { 0 }));
            Ok(())
        }
        HirExpr::FloatLit(_) => {
            // Float requires SSE2 registers — DEFER-P54-002
            // Emit 0 as placeholder
            ctx.emit(&mov_rax_imm32(0));
            Ok(())
        }
        HirExpr::Nil | HirExpr::StringLit(_) => {
            ctx.emit(&xor_rax_rax());
            Ok(())
        }
        HirExpr::Var(name) => emit_var(ctx, name),
        HirExpr::BinOp { op, lhs, rhs } => emit_binop(ctx, op, lhs, rhs),
        HirExpr::UnaryOp { op, expr } => {
            emit_expr(ctx, expr)?;
            match op {
                HirUnaryOp::Neg => ctx.emit(&neg_rax()),
                HirUnaryOp::Not => ctx.emit(&[0x48, 0x83, 0xf0, 0x01]), // xor rax, 1
            }
            Ok(())
        }
        HirExpr::Call { name, args } => emit_call(ctx, name, args),
        HirExpr::If { cond, then, else_ } => emit_if(ctx, cond, then, else_),
        // P55.5: v0.3 nodes — full native emission in P55.6/P57
        HirExpr::Pipe { lhs, .. }        => emit_expr(ctx, lhs),
        HirExpr::Morph { expr, .. }      => emit_expr(ctx, expr),
        HirExpr::CapPinCall { expr, .. } => emit_expr(ctx, expr),
        HirExpr::Temporal(_)             => { ctx.emit(&mov_rax_imm32(0)); Ok(()) }
        HirExpr::Foreach { body, .. }    => {
            for stmt in body { emit_stmt(ctx, stmt)?; }
            Ok(())
        }
        HirExpr::Yield(expr)             => emit_expr(ctx, expr),
        HirExpr::IntentBlock { body, .. } => {
            for stmt in body { emit_stmt(ctx, stmt)?; }
            Ok(())
        }
    }
}

fn emit_var(ctx: &mut NativeCtx, name: &str) -> NativeResult<()> {
    let n = name;

    // Check param registers first (direct register access)
    for (pname, reg_id) in &ctx.param_regs {
        if pname == n {
            ctx.emit(&mov_rax_param(*reg_id));
            return Ok(());
        }
    }

    // Check local stack slots
    for (lname, slot) in &ctx.local_slots {
        if lname == n {
            ctx.emit(&load_rax_rbp_slot(*slot));
            return Ok(());
        }
    }

    Err(NativeError::UndefinedName(AxString::ax_from_str(name)))
}

fn emit_binop(
    ctx: &mut NativeCtx,
    op: &HirBinOp,
    lhs: &HirExpr,
    rhs: &HirExpr,
) -> NativeResult<()> {
    // Stack-machine: emit LHS → rax, push; emit RHS → rax, pop LHS into rbx
    // After: rax = RHS, rbx = LHS
    emit_expr(ctx, lhs)?;
    ctx.emit(&push_rax()); // save LHS
    emit_expr(ctx, rhs)?;
    ctx.emit(&pop_rbx());  // restore LHS into rbx

    match op {
        HirBinOp::Add => ctx.emit(&add_rax_rbx()),  // rax = RHS + LHS ✓
        HirBinOp::Sub => {
            ctx.emit(&sub_rbx_rax()); // rbx = LHS - RHS
            ctx.emit(&mov_rax_rbx()); // rax = result
        }
        HirBinOp::Mul => ctx.emit(&imul_rax_rbx()), // rax = RHS * LHS ✓
        HirBinOp::Div | HirBinOp::Mod => {
            // DEFER-P54-001: idiv requires sign-extend rdx:rax setup
            // Simplified: return LHS (rbx) as placeholder
            ctx.emit(&mov_rax_rbx());
        }
        HirBinOp::Eq  => { ctx.emit(&cmp_rbx_rax()); ctx.emit(&sete_al());  ctx.emit(&movzx_rax_al()); }
        HirBinOp::Ne  => { ctx.emit(&cmp_rbx_rax()); ctx.emit(&setne_al()); ctx.emit(&movzx_rax_al()); }
        HirBinOp::Lt  => { ctx.emit(&cmp_rbx_rax()); ctx.emit(&setl_al());  ctx.emit(&movzx_rax_al()); }
        HirBinOp::Le  => { ctx.emit(&cmp_rbx_rax()); ctx.emit(&setle_al()); ctx.emit(&movzx_rax_al()); }
        HirBinOp::Gt  => { ctx.emit(&cmp_rbx_rax()); ctx.emit(&setg_al());  ctx.emit(&movzx_rax_al()); }
        HirBinOp::Ge  => { ctx.emit(&cmp_rbx_rax()); ctx.emit(&setge_al()); ctx.emit(&movzx_rax_al()); }
        HirBinOp::And => ctx.emit(&and_rax_rbx()),
        HirBinOp::Or  => ctx.emit(&or_rax_rbx()),
    }
    Ok(())
}

fn emit_call(ctx: &mut NativeCtx, name: &str, args: &[HirExpr]) -> NativeResult<()> {
    // Push args in reverse order, then pop into param registers in forward order.
    // This correctly handles register reuse (e.g., g(b, a) where a=rdi, b=rsi).
    for arg in args.iter().rev() {
        emit_expr(ctx, arg)?;
        ctx.emit(&push_rax());
    }
    // Pop into rdi, rsi, rdx
    let pop_ops: &[&[u8]] = &[&[0x5f], &[0x5e], &[0x5a]]; // pop rdi, rsi, rdx
    for pop_op in pop_ops.iter().take(args.len().min(3)) {
        ctx.emit(pop_op);
    }
    // Emit call with placeholder — patched by apply_relocations
    ctx.emit(&[0xe8]);
    let rel_pos = ctx.pos();
    ctx.emit(&[0x00, 0x00, 0x00, 0x00]);
    ctx.relocs.push((rel_pos, name.to_string()));
    Ok(())
}

fn emit_if(
    ctx: &mut NativeCtx,
    cond: &HirExpr,
    then: &[HirStmt],
    else_: &Option<Vec<HirStmt>>,
) -> NativeResult<()> {
    // Emit condition → rax
    emit_expr(ctx, cond)?;

    // test rax, rax (is condition zero?)
    ctx.emit(&test_rax_rax());

    // je .else_or_end (placeholder: skip then-block if condition is false)
    ctx.emit(&[0x0f, 0x84]);
    let je_rel_pos = ctx.pos();
    ctx.emit(&[0x00, 0x00, 0x00, 0x00]);

    // Emit then-block
    for stmt in then {
        emit_stmt(ctx, stmt)?;
    }

    let then_always_returns = block_returns(then);

    // If there's an else-block and then doesn't always return, emit jmp .end
    let jmp_rel_pos: Option<usize> = if else_.is_some() && !then_always_returns {
        ctx.emit(&[0xe9]);
        let pos = ctx.pos();
        ctx.emit(&[0x00, 0x00, 0x00, 0x00]);
        Some(pos)
    } else {
        None
    };

    // Patch je to current position (start of else or end)
    let je_target = ctx.pos();
    let je_rel = (je_target as i64 - (je_rel_pos + 4) as i64) as i32;
    ctx.patch_i32(je_rel_pos, je_rel);

    // Emit else-block
    if let Some(else_stmts) = else_ {
        for stmt in else_stmts {
            emit_stmt(ctx, stmt)?;
        }
    }

    // Patch jmp .end (if emitted)
    if let Some(jmp_pos) = jmp_rel_pos {
        let jmp_target = ctx.pos();
        let jmp_rel = (jmp_target as i64 - (jmp_pos + 4) as i64) as i32;
        ctx.patch_i32(jmp_pos, jmp_rel);
    }

    Ok(())
}

// ── Relocation patching ───────────────────────────────────────────────────────

fn apply_relocations(ctx: &mut NativeCtx) {
    let relocs = std::mem::take(&mut ctx.relocs);
    for (rel_pos, fn_name) in relocs {
        if let Some(target) = ctx.fn_offsets.iter()
            .find(|(n, _)| n == &fn_name)
            .map(|(_, o)| *o)
        {
            // rel32 = target - (rel_pos + 4)  [next instruction after the rel32 field]
            let rel = (target as i64 - (rel_pos + 4) as i64) as i32;
            ctx.patch_i32(rel_pos, rel);
        }
    }
}
