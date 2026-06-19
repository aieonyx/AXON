// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// LLVM IR emission bridge — mirrors emit.ax exactly.
// Single-pass linear emission: one HIR node → one IR sequence.
// Excised at P55 bootstrap when axonc compiles emit.ax natively.

use axon_hir::{HirBinOp, HirExpr, HirFn, HirProgram, HirStmt, HirUnaryOp};
use axon_infer::{infer_program, InferredFn, InferredProgram, Ty};
use axon_std_string::AxString;
use crate::error::{CodegenError, CodegenResult};
use crate::ir::ty_str;

// ── Emission context ──────────────────────────────────────────────────────────

struct EmitCtx {
    buf:      String,
    reg:      usize,
    /// Param name → Ty (accessed as %name directly — no load needed)
    params:   Vec<(String, Ty)>,
    /// Let binding: (var_name, alloca_reg, Ty)
    allocas:  Vec<(String, String, Ty)>,
    /// Function name → return Ty
    fn_rets:  Vec<(String, Ty)>,
}

impl EmitCtx {
    fn new() -> Self {
        EmitCtx {
            buf:     String::new(),
            reg:     0,
            params:  Vec::new(),
            allocas: Vec::new(),
            fn_rets: Vec::new(),
        }
    }

    fn fresh_reg(&mut self) -> String {
        let r = format!("%{}", self.reg);
        self.reg += 1;
        r
    }

    fn line(&mut self, s: &str) {
        self.buf.push_str(s);
        self.buf.push('\n');
    }

    fn lookup_param(&self, name: &str) -> Option<Ty> {
        self.params.iter().rev()
            .find(|(n, _)| n == name)
            .map(|(_, ty)| ty.clone())
    }

    fn lookup_alloca(&self, name: &str) -> Option<(String, Ty)> {
        self.allocas.iter().rev()
            .find(|(n, _, _)| n == name)
            .map(|(_, reg, ty)| (reg.clone(), ty.clone()))
    }

    fn lookup_fn_ret(&self, name: &str) -> Ty {
        self.fn_rets.iter()
            .find(|(n, _)| n == name)
            .map(|(_, ty)| ty.clone())
            .unwrap_or(Ty::Unknown)
    }
}

// ── Public entry points ───────────────────────────────────────────────────────

/// Emit LLVM IR from an AXON source string.
pub fn codegen_source(source: &str) -> CodegenResult<String> {
    let ast = axon_parse::parse(source).map_err(|e| {
        CodegenError::PipelineError(AxString::ax_from_str(&format!("parse: {}", e)))
    })?;
    let hir = axon_hir::lower_program(&ast).map_err(|e| {
        CodegenError::PipelineError(AxString::ax_from_str(&format!("hir: {}", e)))
    })?;
    let typed = infer_program(&hir).map_err(|e| {
        CodegenError::PipelineError(AxString::ax_from_str(&format!("infer: {}", e)))
    })?;
    emit_module(&hir, &typed)
}

/// Emit LLVM IR from a HirProgram + InferredProgram.
pub fn emit_module(hir: &HirProgram, typed: &InferredProgram) -> CodegenResult<String> {
    let mut ctx = EmitCtx::new();

    // Collect function return types for call emission
    for f in &typed.fns {
        ctx.fn_rets.push((f.name.as_str().to_string(), f.ret_ty.clone()));
    }

    // Module header
    ctx.line("; AXONYX sovereign codegen output");
    ctx.line("; Copyright (c) 2026 Edison Lepiten / AIEONYX");
    ctx.line("");

    // Emit each function
    for (hir_fn, inf_fn) in hir.fns.iter().zip(typed.fns.iter()) {
        emit_fn(&mut ctx, hir_fn, inf_fn)?;
    }

    Ok(ctx.buf)
}

// ── Function emission ─────────────────────────────────────────────────────────

fn emit_fn(ctx: &mut EmitCtx, f: &HirFn, inf: &InferredFn) -> CodegenResult<()> {
    // Reset per-function state
    ctx.reg = 0;
    ctx.params.clear();
    ctx.allocas.clear();

    // Build parameter list and register param types
    let params_str: String = f.params.iter()
        .map(|p| {
            let ty = axon_infer::ty_from_hir(&p.ty);
            ctx.params.push((p.name.as_str().to_string(), ty.clone()));
            format!("{} %{}", ty_str(&ty), p.name.as_str())
        })
        .collect::<Vec<_>>()
        .join(", ");

    let ret_str = ty_str(&inf.ret_ty);
    ctx.line(&format!("define {} @{}({}) {{", ret_str, f.name.as_str(), params_str));
    ctx.line("entry:");

    for stmt in &f.body {
        emit_stmt(ctx, stmt)?;
    }

    ctx.line("}");
    ctx.line("");
    Ok(())
}

// ── Statement emission ────────────────────────────────────────────────────────

fn emit_stmt(ctx: &mut EmitCtx, stmt: &HirStmt) -> CodegenResult<()> {
    match stmt {
        HirStmt::Let { name, value, .. } => {
            let (val, ty) = emit_expr(ctx, value)?;
            let ir_ty     = ty_str(&ty);
            let slot      = format!("%{}_slot", name.as_str());
            ctx.line(&format!("  {} = alloca {}", slot, ir_ty));
            ctx.line(&format!("  store {} {}, {}* {}", ir_ty, val, ir_ty, slot));
            ctx.allocas.push((name.as_str().to_string(), slot, ty));
            Ok(())
        }
        HirStmt::Return(expr) => {
            let (val, ty) = emit_expr(ctx, expr)?;
            if ty == Ty::Nil {
                ctx.line("  ret void");
            } else {
                ctx.line(&format!("  ret {} {}", ty_str(&ty), val));
            }
            Ok(())
        }
        HirStmt::ExprStmt(expr) => {
            emit_expr(ctx, expr)?;
            Ok(())
        }
    }
}

// ── Expression emission ───────────────────────────────────────────────────────

/// Emit instructions for an expression. Returns (value_string, Ty).
/// value_string is either a constant ("42") or a register ("%0").
fn emit_expr(ctx: &mut EmitCtx, expr: &HirExpr) -> CodegenResult<(String, Ty)> {
    match expr {
        HirExpr::IntLit(n)    => Ok((n.to_string(), Ty::I32)),
        HirExpr::FloatLit(f)  => Ok((format!("{}", f), Ty::F64)),
        HirExpr::BoolLit(b)   => Ok((if *b { "1" } else { "0" }.to_string(), Ty::Bool)),
        HirExpr::StringLit(_) => Ok(("null".to_string(), Ty::Str)),
        HirExpr::Nil          => Ok(("".to_string(), Ty::Nil)),

        HirExpr::Var(name) => {
            // Params: direct register value, no load needed
            if let Some(ty) = ctx.lookup_param(name.as_str()) {
                return Ok((format!("%{}", name.as_str()), ty));
            }
            // Let bindings: load from alloca slot
            if let Some((slot, ty)) = ctx.lookup_alloca(name.as_str()) {
                let ir_ty    = ty_str(&ty);
                let load_reg = ctx.fresh_reg();
                ctx.line(&format!("  {} = load {}, {}* {}", load_reg, ir_ty, ir_ty, slot));
                return Ok((load_reg, ty));
            }
            Err(CodegenError::UndefinedName(AxString::ax_from_str(name.as_str())))
        }

        HirExpr::BinOp { op, lhs, rhs } => {
            let (lv, lty) = emit_expr(ctx, lhs.as_ref())?;
            let (rv, _)   = emit_expr(ctx, rhs.as_ref())?;
            let result    = ctx.fresh_reg();
            let ir_ty     = ty_str(&lty);

            let (instr, result_ty) = match op {
                HirBinOp::Add => (format!("add {} {}, {}", ir_ty, lv, rv),  lty),
                HirBinOp::Sub => (format!("sub {} {}, {}", ir_ty, lv, rv),  lty),
                HirBinOp::Mul => (format!("mul {} {}, {}", ir_ty, lv, rv),  lty),
                HirBinOp::Div => (format!("sdiv {} {}, {}", ir_ty, lv, rv), lty),
                HirBinOp::Mod => (format!("srem {} {}, {}", ir_ty, lv, rv), lty),
                HirBinOp::Eq  => (format!("icmp eq {} {}, {}",  ir_ty, lv, rv), Ty::Bool),
                HirBinOp::Ne  => (format!("icmp ne {} {}, {}",  ir_ty, lv, rv), Ty::Bool),
                HirBinOp::Lt  => (format!("icmp slt {} {}, {}", ir_ty, lv, rv), Ty::Bool),
                HirBinOp::Le  => (format!("icmp sle {} {}, {}", ir_ty, lv, rv), Ty::Bool),
                HirBinOp::Gt  => (format!("icmp sgt {} {}, {}", ir_ty, lv, rv), Ty::Bool),
                HirBinOp::Ge  => (format!("icmp sge {} {}, {}", ir_ty, lv, rv), Ty::Bool),
                HirBinOp::And => (format!("and i1 {}, {}", lv, rv), Ty::Bool),
                HirBinOp::Or  => (format!("or i1 {}, {}",  lv, rv), Ty::Bool),
            };

            ctx.line(&format!("  {} = {}", result, instr));
            Ok((result, result_ty))
        }

        HirExpr::UnaryOp { op, expr } => {
            let (val, ty) = emit_expr(ctx, expr.as_ref())?;
            let result    = ctx.fresh_reg();
            let instr = match op {
                HirUnaryOp::Neg => format!("sub {} 0, {}", ty_str(&ty), val),
                HirUnaryOp::Not => format!("xor i1 {}, 1", val),
            };
            let result_ty = match op {
                HirUnaryOp::Not => Ty::Bool,
                HirUnaryOp::Neg => ty,
            };
            ctx.line(&format!("  {} = {}", result, instr));
            Ok((result, result_ty))
        }

        HirExpr::Call { name, args } => {
            let mut arg_strs: Vec<String> = Vec::new();
            for arg in args {
                let (val, ty) = emit_expr(ctx, arg)?;
                arg_strs.push(format!("{} {}", ty_str(&ty), val));
            }
            let ret_ty   = ctx.lookup_fn_ret(name.as_str());
            let args_str = arg_strs.join(", ");

            if ret_ty == Ty::Nil {
                ctx.line(&format!("  call void @{}({})", name.as_str(), args_str));
                Ok(("".to_string(), Ty::Nil))
            } else {
                let result = ctx.fresh_reg();
                ctx.line(&format!("  {} = call {} @{}({})",
                    result, ty_str(&ret_ty), name.as_str(), args_str));
                Ok((result, ret_ty))
            }
        }

        HirExpr::If { cond, then, else_ } => {
            // DEFER-P53-001: full phi-node if codegen arrives at P54.
            // At P53: emit condition and then-block only (linear approximation).
            let (_cond_val, _) = emit_expr(ctx, cond.as_ref())?;
            for stmt in then {
                emit_stmt(ctx, stmt)?;
            }
            if let Some(else_stmts) = else_ {
                for stmt in else_stmts {
                    emit_stmt(ctx, stmt)?;
                }
            }
            Ok(("".to_string(), Ty::Nil))
        }
    }
}
