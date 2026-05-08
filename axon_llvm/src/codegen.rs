// ============================================================
// axon_llvm — codegen.rs
// AXON AST → LLVM IR text
// Copyright © 2026 Edison Lepiten — AIEONYX
// ============================================================

use std::collections::HashMap;
use axon_parser::ast::{
    Program, TopLevelItem,
    FnDecl, TaskDecl,
    Block, Stmt,
    Expr, Literal,
    BinOp, UnaryOp,
    Pattern,
};
use crate::ir::{LlvmModule, LlvmFunction, LlvmType, Value, Counter};
use crate::types::llvm_type;
use crate::error::LlvmCodegenError;

pub struct LlvmCodegen {
    pub module  : LlvmModule,
    counter     : Counter,
    variables   : HashMap<String, (Value, LlvmType)>,
}

impl LlvmCodegen {
    pub fn new(module_name: &str) -> Self {
        LlvmCodegen {
            module   : LlvmModule::new(module_name),
            counter  : Counter::new(),
            variables: HashMap::new(),
        }
    }

    pub fn emit_program(
        &mut self, program: &Program,
    ) -> Result<String, LlvmCodegenError> {
        for item in &program.items {
            match item {
                TopLevelItem::Fn(f)   => { self.emit_fn(f)?; }
                TopLevelItem::Task(t) => { self.emit_task(t)?; }
                _ => {}
            }
        }
        Ok(self.module.emit())
    }

    // ── Function emission ─────────────────────────────────────

    fn emit_fn(&mut self, f: &FnDecl) -> Result<(), LlvmCodegenError> {
        let params: Vec<(LlvmType, String)> = f.params.iter()
            .map(|p| (llvm_type(&p.ty), p.name.name.clone()))
            .collect();
        let ret_ty = f.ret_type.as_ref()
            .map(|t| llvm_type(t))
            .unwrap_or(LlvmType::I64);

        let mut func = LlvmFunction::new(&f.name.name, params, ret_ty.clone());

        // Bind parameters to variables
        self.variables.clear();
        for p in &f.params {
            let ty = llvm_type(&p.ty);
            self.variables.insert(
                p.name.name.clone(),
                (Value::named(&p.name.name), ty)
            );
        }

        self.emit_block_into(&f.body, &mut func)?;

        // Ensure terminator
        if !func.current_block().is_terminated() {
            let zero = default_value(&ret_ty);
            func.current_block().push(format!("ret {} {}", ret_ty.as_str(), zero));
        }

        self.module.add_function(func);
        Ok(())
    }

    fn emit_task(&mut self, t: &TaskDecl) -> Result<(), LlvmCodegenError> {
        let params: Vec<(LlvmType, String)> = t.params.iter()
            .map(|p| (llvm_type(&p.ty), p.name.name.clone()))
            .collect();
        let ret_ty = LlvmType::I64;
        let mut func = LlvmFunction::new(&t.name.name, params, ret_ty.clone());

        self.variables.clear();
        for p in &t.params {
            let ty = llvm_type(&p.ty);
            self.variables.insert(
                p.name.name.clone(),
                (Value::named(&p.name.name), ty)
            );
        }

        self.emit_block_into(&t.body, &mut func)?;

        if !func.current_block().is_terminated() {
            func.current_block().push("ret i64 0");
        }
        self.module.add_function(func);
        Ok(())
    }

    // ── Block / statement emission ────────────────────────────

    fn emit_block_into(
        &mut self,
        block: &Block,
        func: &mut LlvmFunction,
    ) -> Result<(), LlvmCodegenError> {
        for stmt in &block.stmts {
            self.emit_stmt(stmt, func)?;
            if func.current_block().is_terminated() { break; }
        }
        Ok(())
    }

    fn emit_stmt(
        &mut self,
        stmt: &Stmt,
        func: &mut LlvmFunction,
    ) -> Result<(), LlvmCodegenError> {
        match stmt {
            Stmt::Pass(_) => {}

            Stmt::Let(s) => {
                let (val, ty) = self.emit_expr_into(&s.init, func)?;
                self.variables.insert(s.name.name.clone(), (val, ty));
            }

            Stmt::Mut(s) => {
                let (val, ty) = self.emit_expr_into(&s.init, func)?;
                self.variables.insert(s.name.name.clone(), (val, ty));
            }

            Stmt::Return(s) => {
                if let Some(expr) = &s.value {
                    let (val, ty) = self.emit_expr_into(expr, func)?;
                    func.current_block().push(
                        format!("ret {} {}", ty.as_str(), val));
                } else {
                    func.current_block().push("ret i64 0");
                }
            }

            Stmt::If(s) => {
                let (cond_val, _) = self.emit_expr_into(&s.condition, func)?;
                let then_lbl  = self.counter.next("then").0.trim_start_matches('%').to_string();
                let merge_lbl = self.counter.next("merge").0.trim_start_matches('%').to_string();
                let else_lbl  = if s.else_block.is_some() {
                    self.counter.next("else").0.trim_start_matches('%').to_string()
                } else { merge_lbl.clone() };

                // Convert to i1 if needed
                let cond_i1  = self.to_bool(cond_val, func);
                func.current_block().push(
                    format!("br i1 {}, label %{}, label %{}", cond_i1, then_lbl, else_lbl));

                func.add_block(&then_lbl);
                self.emit_block_into(&s.then_block, func)?;
                if !func.current_block().is_terminated() {
                    func.current_block().push(format!("br label %{}", merge_lbl));
                }

                if let Some(else_block) = &s.else_block {
                    if else_lbl != merge_lbl {
                        func.add_block(&else_lbl);
                        self.emit_block_into(else_block, func)?;
                        if !func.current_block().is_terminated() {
                            func.current_block().push(format!("br label %{}", merge_lbl));
                        }
                    }
                }

                func.add_block(&merge_lbl);
            }

            Stmt::While(s) => {
                let cond_lbl = self.counter.next("while_cond").0.trim_start_matches('%').to_string();
                let body_lbl = self.counter.next("while_body").0.trim_start_matches('%').to_string();
                let exit_lbl = self.counter.next("while_exit").0.trim_start_matches('%').to_string();

                func.current_block().push(format!("br label %{}", cond_lbl));
                func.add_block(&cond_lbl);
                let (cond_val, _) = self.emit_expr_into(&s.condition, func)?;
                let cond_i1 = self.to_bool(cond_val, func);
                func.current_block().push(
                    format!("br i1 {}, label %{}, label %{}", cond_i1, body_lbl, exit_lbl));

                func.add_block(&body_lbl);
                self.emit_block_into(&s.body, func)?;
                if !func.current_block().is_terminated() {
                    func.current_block().push(format!("br label %{}", cond_lbl));
                }
                func.add_block(&exit_lbl);
            }

            Stmt::Match(s) => {
                let (subject_val, subject_ty) = self.emit_expr_into(&s.subject, func)?;
                let merge_lbl = self.counter.next("match_merge").0.trim_start_matches('%').to_string();

                // Build arm blocks
                let mut arm_labels: Vec<String> = Vec::new();
                let mut default_lbl = merge_lbl.clone();

                for (i, arm) in s.arms.iter().enumerate() {
                    let lbl = format!("match_arm_{}", i);
                    arm_labels.push(lbl.clone());
                    if matches!(arm.pattern,
                        Pattern::Wildcard(_) | Pattern::Binding(_)) {
                        default_lbl = lbl;
                    }
                }

                // Build switch instruction
                let mut switch_cases = Vec::new();
                for (arm, lbl) in s.arms.iter().zip(&arm_labels) {
                    if let Pattern::Literal(Literal::Int(n, _)) = &arm.pattern {
                        switch_cases.push(format!(
                            "{} {}, label %{}", subject_ty.as_str(), n, lbl));
                    }
                }

                let switch_str = if switch_cases.is_empty() {
                    format!("br label %{}", default_lbl)
                } else {
                    format!("switch {} {}, label %{} [\n    {}\n  ]",
                        subject_ty.as_str(), subject_val, default_lbl,
                        switch_cases.join("\n    "))
                };
                func.current_block().push(switch_str);

                // Emit each arm
                for (arm, lbl) in s.arms.iter().zip(&arm_labels) {
                    func.add_block(lbl);
                    // Bind pattern variables
                    if let Pattern::Binding(id) = &arm.pattern {
                        self.variables.insert(
                            id.name.clone(),
                            (subject_val.clone(), subject_ty.clone())
                        );
                    }
                    // Emit arm body
                    let (body_val, body_ty) = self.emit_expr_into(&arm.body, func)?;
                    if !func.current_block().is_terminated() {
                        // For return expressions — already emitted
                        // For value expressions — br to merge
                        let _ = (body_val, body_ty);
                        func.current_block().push(format!("br label %{}", merge_lbl));
                    }
                }

                func.add_block(&merge_lbl);
            }

            Stmt::Expr(s) => { self.emit_expr_into(&s.expr, func)?; }

            _ => {}
        }
        Ok(())
    }

    // ── Expression emission ───────────────────────────────────

    fn emit_expr_into(
        &mut self,
        expr: &Expr,
        func: &mut LlvmFunction,
    ) -> Result<(Value, LlvmType), LlvmCodegenError> {
        match expr {
            Expr::Lit(lit) => Ok(emit_literal(lit)),

            Expr::Ident(id) => {
                if let Some((val, ty)) = self.variables.get(&id.name) {
                    Ok((val.clone(), ty.clone()))
                } else {
                    Ok((Value::int_const(0), LlvmType::I64))
                }
            }

            Expr::BinOp(b) => {
                let (lhs, lty) = self.emit_expr_into(&b.lhs, func)?;
                let (rhs, _)   = self.emit_expr_into(&b.rhs, func)?;
                let result     = self.counter.next(&format!("{:?}", b.op).to_lowercase());
                let ty_str     = lty.as_str();

                let instr = match &b.op {
                    BinOp::Add  => format!("{} = add {} {}, {}",    result, ty_str, lhs, rhs),
                    BinOp::Sub  => format!("{} = sub {} {}, {}",    result, ty_str, lhs, rhs),
                    BinOp::Mul  => format!("{} = mul {} {}, {}",    result, ty_str, lhs, rhs),
                    BinOp::Div  => format!("{} = sdiv {} {}, {}",   result, ty_str, lhs, rhs),
                    BinOp::Mod  => format!("{} = srem {} {}, {}",   result, ty_str, lhs, rhs),
                    BinOp::Eq   => format!("{} = icmp eq {} {}, {}",  result, ty_str, lhs, rhs),
                    BinOp::NotEq=> format!("{} = icmp ne {} {}, {}",  result, ty_str, lhs, rhs),
                    BinOp::Lt   => format!("{} = icmp slt {} {}, {}", result, ty_str, lhs, rhs),
                    BinOp::Gt   => format!("{} = icmp sgt {} {}, {}", result, ty_str, lhs, rhs),
                    BinOp::LtEq => format!("{} = icmp sle {} {}, {}", result, ty_str, lhs, rhs),
                    BinOp::GtEq => format!("{} = icmp sge {} {}, {}", result, ty_str, lhs, rhs),
                    BinOp::And  => format!("{} = and {} {}, {}",    result, ty_str, lhs, rhs),
                    BinOp::Or   => format!("{} = or {} {}, {}",     result, ty_str, lhs, rhs),
                    BinOp::BitAnd => format!("{} = and {} {}, {}",  result, ty_str, lhs, rhs),
                    BinOp::BitOr  => format!("{} = or {} {}, {}",   result, ty_str, lhs, rhs),
                    BinOp::BitXor => format!("{} = xor {} {}, {}",  result, ty_str, lhs, rhs),
                    BinOp::Shl    => format!("{} = shl {} {}, {}",  result, ty_str, lhs, rhs),
                    BinOp::Shr    => format!("{} = ashr {} {}, {}", result, ty_str, lhs, rhs),
                    _ => format!("; unsupported binop"),
                };
                func.current_block().push(instr);

                // Comparison ops return i1
                let res_ty = match &b.op {
                    BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt |
                    BinOp::LtEq | BinOp::GtEq => LlvmType::I1,
                    _ => lty,
                };
                Ok((result, res_ty))
            }

            Expr::UnaryOp(u) => {
                let (operand, ty) = self.emit_expr_into(&u.expr, func)?;
                let result = self.counter.next("unary");
                let instr = match &u.op {
                    UnaryOp::Neg =>
                        format!("{} = sub {} 0, {}", result, ty.as_str(), operand),
                    UnaryOp::Not =>
                        format!("{} = xor {} {}, -1", result, ty.as_str(), operand),
                    _ => format!("; unsupported unary"),
                };
                func.current_block().push(instr);
                Ok((result, ty))
            }

            Expr::Return(val, _) => {
                if let Some(e) = val.as_ref() {
                    let (rv, ty) = self.emit_expr_into(e, func)?;
                    func.current_block().push(format!("ret {} {}", ty.as_str(), rv));
                } else {
                    func.current_block().push("ret i64 0");
                }
                Ok((Value::int_const(0), LlvmType::I64))
            }

            Expr::Call(c) => {
                let result = self.counter.next("call");
                let args: Result<Vec<_>, _> = c.args.iter()
                    .map(|a| self.emit_expr_into(&a.value, func)
                        .map(|(v, ty)| format!("{} {}", ty.as_str(), v)))
                    .collect();
                let args_str = args?.join(", ");
                func.current_block().push(
                    format!("{} = call i64 @{}({})", result, c.callee.name, args_str));
                Ok((result, LlvmType::I64))
            }

            _ => Ok((Value::int_const(0), LlvmType::I64)),
        }
    }

    fn to_bool(&mut self, val: Value, func: &mut LlvmFunction) -> Value {
        // If already i1, return as-is
        // Otherwise compare != 0
        let result = self.counter.next("bool");
        func.current_block().push(
            format!("{} = icmp ne i64 {}, 0", result, val));
        result
    }
}

// ── Helper functions ──────────────────────────────────────────

fn emit_literal(lit: &Literal) -> (Value, LlvmType) {
    match lit {
        Literal::Int(n, _)   => (Value::int_const(*n), LlvmType::I64),
        Literal::Bool(b, _)  => (Value(if *b { "true".into() } else { "false".into() }), LlvmType::I1),
        Literal::Float(f, _) => (Value(format!("{:.6}", f)), LlvmType::F64),
        Literal::None(_)     => (Value::int_const(0), LlvmType::I64),
        _                    => (Value::int_const(0), LlvmType::I64),
    }
}

fn default_value(ty: &LlvmType) -> String {
    match ty {
        LlvmType::I1           => "false".to_string(),
        LlvmType::F32 |
        LlvmType::F64          => "0.0".to_string(),
        LlvmType::Ptr          => "null".to_string(),
        _                      => "0".to_string(),
    }
}
