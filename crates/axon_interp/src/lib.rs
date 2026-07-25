// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_interp P71.5 — Sovereign .ax script interpreter
//
// Upstreamed from aiXos PL-59.1 (aixos-shell/src/axon_interp.rs).
// This is the single source of truth. aiXos depends on this crate.
//
// Design: no_std compatible, no heap allocation, fixed-size buffers.
// Runs on bare-metal aiXos Phoenix (AArch64) and Linux host.
//
// Supported .ax subset:
//   print "text"        — print string literal
//   print varname       — print variable value
//   let x = N          — assign integer literal
//   let x = y          — assign from variable
//   let x = y + z      — integer addition
//   let x = y - z      — integer subtraction
//   let x = y * z      — integer multiplication
//   awp <payload>       — send AWP frame (optional transport callback)
//   // comment          — ignored
//   fn / return         — parsed but not executed (flat scripts only)
//
// Conformance oracle: axon_interp output == AArch64 native codegen output (P71 target)

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

pub const MAX_LINES:    usize = 64;   // doubled from PL-59.1 (was 32)
pub const MAX_VARS:     usize = 16;   // doubled (was 8)
pub const VAR_NAME_LEN: usize = 32;   // doubled (was 16)
pub const OUTPUT_BUF:   usize = 2048; // 4x (was 512)

// ── Variable slot ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct AxVar {
    pub name: [u8; VAR_NAME_LEN],
    pub name_len: usize,
    pub value: i64,
}

impl AxVar {
    pub const fn empty() -> Self {
        AxVar { name: [0u8; VAR_NAME_LEN], name_len: 0, value: 0 }
    }
}

// ── Interpreter state ─────────────────────────────────────────────────────────

pub struct AxInterp {
    pub vars: [AxVar; MAX_VARS],
    pub var_count: usize,
}

impl AxInterp {
    pub const fn new() -> Self {
        Self {
            vars: [AxVar::empty(); MAX_VARS],
            var_count: 0,
        }
    }

    pub fn reset(&mut self) {
        self.vars = [AxVar::empty(); MAX_VARS];
        self.var_count = 0;
    }
}

// ── Result ────────────────────────────────────────────────────────────────────

pub struct AxResult {
    pub output: [u8; OUTPUT_BUF],
    pub output_len: usize,
    pub lines_executed: usize,
    pub error: bool,
    pub error_line: usize,
    pub error_msg: [u8; 64],
    pub error_msg_len: usize,
}

impl AxResult {
    pub fn new() -> Self {
        AxResult {
            output: [0u8; OUTPUT_BUF],
            output_len: 0,
            lines_executed: 0,
            error: false,
            error_line: 0,
            error_msg: [0u8; 64],
            error_msg_len: 0,
        }
    }

    pub fn push_str(&mut self, s: &[u8]) {
        let mut i = 0;
        while i < s.len() && self.output_len < OUTPUT_BUF - 1 {
            self.output[self.output_len] = s[i];
            self.output_len += 1;
            i += 1;
        }
    }

    pub fn push_newline(&mut self) {
        if self.output_len < OUTPUT_BUF - 1 {
            self.output[self.output_len] = b'\n';
            self.output_len += 1;
        }
    }

    pub fn push_i64(&mut self, mut n: i64) {
        if n < 0 {
            self.push_str(b"-");
            n = n.wrapping_neg();
        }
        let mut tmp = [0u8; 20];
        let mut len = 0usize;
        if n == 0 { self.push_str(b"0"); return; }
        while n > 0 {
            tmp[len] = b'0' + (n % 10) as u8;
            len += 1;
            n /= 10;
        }
        let mut i = len;
        while i > 0 {
            i -= 1;
            if self.output_len < OUTPUT_BUF - 1 {
                self.output[self.output_len] = tmp[i];
                self.output_len += 1;
            }
        }
    }

    pub fn set_error(&mut self, line_num: usize, msg: &[u8]) {
        self.error = true;
        self.error_line = line_num;
        let len = msg.len().min(63);
        self.error_msg[..len].copy_from_slice(&msg[..len]);
        self.error_msg_len = len;
    }

    pub fn as_str(&self) -> &[u8] {
        &self.output[..self.output_len]
    }

    pub fn error_str(&self) -> &[u8] {
        &self.error_msg[..self.error_msg_len]
    }
}

// ── Parser helpers ─────────────────────────────────────────────────────────────

pub fn trim(s: &[u8]) -> &[u8] {
    let mut start = 0;
    while start < s.len() && (s[start] == b' ' || s[start] == b'\t') { start += 1; }
    let mut end = s.len();
    while end > start && (s[end-1] == b' ' || s[end-1] == b'\t'
        || s[end-1] == b'\r' || s[end-1] == b'\n') { end -= 1; }
    &s[start..end]
}

pub fn starts_with(s: &[u8], prefix: &[u8]) -> bool {
    s.len() >= prefix.len() && &s[..prefix.len()] == prefix
}

pub fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x == y)
}

pub fn parse_i64(s: &[u8]) -> Option<i64> {
    let s = trim(s);
    if s.is_empty() { return None; }
    let (neg, digits) = if s[0] == b'-' { (true, &s[1..]) } else { (false, s) };
    if digits.is_empty() { return None; }
    let mut val: i64 = 0;
    let mut i = 0;
    while i < digits.len() {
        if digits[i] < b'0' || digits[i] > b'9' { return None; }
        val = val.wrapping_mul(10).wrapping_add((digits[i] - b'0') as i64);
        i += 1;
    }
    Some(if neg { val.wrapping_neg() } else { val })
}

// ── Variable operations ───────────────────────────────────────────────────────

pub fn find_var(vars: &[AxVar], name: &[u8]) -> Option<i64> {
    let nlen = name.len();
    for v in vars {
        if v.name_len == nlen && &v.name[..nlen] == name {
            return Some(v.value);
        }
    }
    None
}

pub fn set_var(vars: &mut [AxVar; MAX_VARS], name: &[u8], value: i64) -> bool {
    let nlen = name.len().min(VAR_NAME_LEN);
    // update existing
    for v in vars.iter_mut() {
        if v.name_len == nlen && v.name[..nlen] == name[..nlen] {
            v.value = value;
            return true;
        }
    }
    // new slot
    for v in vars.iter_mut() {
        if v.name_len == 0 {
            v.name_len = nlen;
            v.name[..nlen].copy_from_slice(&name[..nlen]);
            v.value = value;
            return true;
        }
    }
    false // out of variable slots
}

// ── Expression evaluator ──────────────────────────────────────────────────────

/// Evaluate a simple expression: literal, variable, or binary op (a op b).
pub fn eval_expr(expr: &[u8], vars: &[AxVar]) -> Option<i64> {
    let expr = trim(expr);

    // Try literal first
    if let Some(n) = parse_i64(expr) { return Some(n); }

    // Try binary op: find operator
    for (op, op_byte) in [(b'+', true), (b'-', false), (b'*', false)] {
        if let Some(pos) = find_op(expr, op) {
            let lhs = trim(&expr[..pos]);
            let rhs = trim(&expr[pos+1..]);
            let l = eval_expr(lhs, vars)?;
            let r = eval_expr(rhs, vars)?;
            return Some(match op {
                b'+' => l.wrapping_add(r),
                b'-' => l.wrapping_sub(r),
                b'*' => l.wrapping_mul(r),
                _    => return None,
            });
        }
        let _ = op_byte;
    }

    // Try variable
    find_var(vars, expr)
}

fn find_op(expr: &[u8], op: u8) -> Option<usize> {
    // Find last occurrence (right-associative for now) outside quotes
    let mut i = expr.len();
    while i > 0 {
        i -= 1;
        if expr[i] == op {
            // Don't split negative literal at start
            if i == 0 { continue; }
            return Some(i);
        }
    }
    None
}

// ── Line executor ─────────────────────────────────────────────────────────────

pub fn exec_line(
    line: &[u8],
    vars: &mut [AxVar; MAX_VARS],
    result: &mut AxResult,
    awp_node_id: u64,
    awp_send: Option<fn(u64, &[u8]) -> bool>,
    line_num: usize,
) {
    if starts_with(line, b"//") || starts_with(line, b"#") { return; }

    // print "text" or print varname/expr
    if starts_with(line, b"print ") {
        let arg = trim(&line[6..]);
        if arg.len() >= 2 && arg[0] == b'"' && arg[arg.len()-1] == b'"' {
            result.push_str(&arg[1..arg.len()-1]);
            result.push_newline();
        } else if let Some(val) = eval_expr(arg, vars) {
            result.push_i64(val);
            result.push_newline();
        } else {
            result.push_str(b"undefined: ");
            result.push_str(arg);
            result.push_newline();
        }
        return;
    }

    // let x = expr
    if starts_with(line, b"let ") {
        let rest = trim(&line[4..]);
        let eq = rest.iter().position(|&b| b == b'=');
        let Some(eq) = eq else {
            result.set_error(line_num, b"let: missing '='");
            return;
        };
        let name = trim(&rest[..eq]);
        let expr = trim(&rest[eq+1..]);
        if name.is_empty() {
            result.set_error(line_num, b"let: empty variable name");
            return;
        }
        let Some(value) = eval_expr(expr, vars) else {
            result.set_error(line_num, b"let: undefined expression");
            return;
        };
        if !set_var(vars, name, value) {
            result.set_error(line_num, b"let: variable slots full");
        }
        return;
    }

    // awp <payload>
    if starts_with(line, b"awp ") {
        let payload = trim(&line[4..]);
        if let Some(send) = awp_send {
            if send(awp_node_id, payload) {
                result.push_str(b"awp: sent ");
                result.push_str(payload);
                result.push_newline();
            } else {
                result.push_str(b"awp: not live\n");
            }
        } else {
            result.push_str(b"awp: no transport\n");
        }
        return;
    }

    // fn / return — skip (flat scripts)
    if starts_with(line, b"fn ") || starts_with(line, b"return ") { return; }

    // Unknown statement
    result.push_str(b"unknown: ");
    result.push_str(line);
    result.push_newline();
}

// ── Main entry point ──────────────────────────────────────────────────────────

/// Execute a .ax script. Returns AxResult with output and error state.
/// `awp_send`: optional transport callback (node_id, payload) -> bool
pub fn exec(
    script: &[u8],
    awp_node_id: u64,
    awp_send: Option<fn(u64, &[u8]) -> bool>,
) -> AxResult {
    let mut result = AxResult::new();
    let mut vars = [AxVar::empty(); MAX_VARS];
    let mut line_start = 0usize;
    let mut line_num = 0usize;

    while line_start <= script.len() && line_num < MAX_LINES {
        let mut line_end = line_start;
        while line_end < script.len() && script[line_end] != b'\n' { line_end += 1; }
        let line = trim(&script[line_start..line_end]);
        if !line.is_empty() {
            exec_line(line, &mut vars, &mut result, awp_node_id, awp_send, line_num);
            if result.error { return result; }
            result.lines_executed += 1;
        }
        if line_end >= script.len() { break; }
        line_start = line_end + 1;
        line_num += 1;
    }
    result
}

/// Execute with a persistent interpreter state (for REPL / session continuity).
pub fn exec_with_state(
    script: &[u8],
    interp: &mut AxInterp,
    awp_node_id: u64,
    awp_send: Option<fn(u64, &[u8]) -> bool>,
) -> AxResult {
    let mut result = AxResult::new();
    let mut line_start = 0usize;
    let mut line_num = 0usize;

    while line_start <= script.len() && line_num < MAX_LINES {
        let mut line_end = line_start;
        while line_end < script.len() && script[line_end] != b'\n' { line_end += 1; }
        let line = trim(&script[line_start..line_end]);
        if !line.is_empty() {
            exec_line(line, &mut interp.vars, &mut result,
                awp_node_id, awp_send, line_num);
            if result.error { return result; }
            result.lines_executed += 1;
        }
        if line_end >= script.len() { break; }
        line_start = line_end + 1;
        line_num += 1;
    }
    result
}
