// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_aarch64 P71 — AArch64 IR (intermediate representation)
//
// Simplified IR for the .ax subset supported by axon_interp:
//   - Integer literals and variables (i64)
//   - Arithmetic: add, sub, mul
//   - Print (syscall-backed on Linux, UART-backed on aiXos)
//   - AWP send (stub)

/// AArch64 target profile
#[derive(Debug, Clone, PartialEq)]
pub enum Aarch64Target {
    /// Bare-metal aiXos Phoenix — no OS, UART output
    Freestanding,
    /// Linux AArch64 — syscall output (for conformance testing on host)
    LinuxAarch64,
}

impl Aarch64Target {
    pub fn triple(&self) -> &'static str {
        match self {
            Self::Freestanding => "aarch64-unknown-none-elf",
            Self::LinuxAarch64 => "aarch64-unknown-linux-gnu",
        }
    }

    pub fn is_freestanding(&self) -> bool {
        matches!(self, Self::Freestanding)
    }

    /// Output primitive: how does print work on this target?
    pub fn print_impl(&self) -> PrintImpl {
        match self {
            Self::Freestanding => PrintImpl::Uart,
            Self::LinuxAarch64 => PrintImpl::Syscall,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrintImpl {
    /// UART MMIO write (aiXos bare-metal)
    Uart,
    /// Linux write(1, buf, len) syscall
    Syscall,
}

/// AArch64 register alias
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reg {
    X0, X1, X2, X3, X4, X5, X6, X7,
    X8, X9, X10, X11, X12, X13, X14, X15,
    X16, X17, X18, X19, X20, X21, X22, X23,
    X24, X25, X26, X27, X28, X29, X30,
    SP, XZR,
}

impl Reg {
    pub fn name(&self) -> &'static str {
        match self {
            Self::X0  => "x0",  Self::X1  => "x1",  Self::X2  => "x2",
            Self::X3  => "x3",  Self::X4  => "x4",  Self::X5  => "x5",
            Self::X6  => "x6",  Self::X7  => "x7",  Self::X8  => "x8",
            Self::X9  => "x9",  Self::X10 => "x10", Self::X11 => "x11",
            Self::X12 => "x12", Self::X13 => "x13", Self::X14 => "x14",
            Self::X15 => "x15", Self::X16 => "x16", Self::X17 => "x17",
            Self::X18 => "x18", Self::X19 => "x19", Self::X20 => "x20",
            Self::X21 => "x21", Self::X22 => "x22", Self::X23 => "x23",
            Self::X24 => "x24", Self::X25 => "x25", Self::X26 => "x26",
            Self::X27 => "x27", Self::X28 => "x28", Self::X29 => "x29",
            Self::X30 => "x30", Self::SP  => "sp",  Self::XZR => "xzr",
        }
    }

    /// Caller-saved registers (x0-x15, x16-x17 scratch)
    pub fn is_caller_saved(&self) -> bool {
        matches!(self, Self::X0|Self::X1|Self::X2|Self::X3|
            Self::X4|Self::X5|Self::X6|Self::X7|Self::X8|
            Self::X9|Self::X10|Self::X11|Self::X12|Self::X13|
            Self::X14|Self::X15|Self::X16|Self::X17)
    }
}

/// Minimal IR for .ax script compilation
#[derive(Debug, Clone)]
pub enum AxIr {
    /// Load integer immediate into variable slot
    LoadImm { var: String, value: i64 },
    /// Load from variable into variable
    LoadVar { dst: String, src: String },
    /// Binary operation: dst = lhs op rhs
    BinOp { dst: String, lhs: String, op: BinOpKind, rhs: String },
    /// Print string literal
    PrintStr { text: String },
    /// Print variable (integer)
    PrintVar { var: String },
    /// AWP send (stub — no-op in freestanding v0)
    AwpSend { payload: String },
    /// Function entry (label)
    FnEntry { name: String },
    /// Function exit (ret)
    FnExit,
    /// Program entry point
    ProgramStart,
    /// Program exit (infinite loop on freestanding, exit(0) on Linux)
    ProgramEnd,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOpKind { Add, Sub, Mul }

impl BinOpKind {
    pub fn as_str(&self) -> &'static str {
        match self { Self::Add => "add", Self::Sub => "sub", Self::Mul => "mul" }
    }
}

/// Compile .ax script bytes to AxIr sequence.
/// Uses the same parsing logic as axon_interp for conformance.
pub fn compile_to_ir(script: &[u8]) -> Result<Vec<AxIr>, CompileError> {
    use axon_interp::{trim, starts_with};

    let mut ir = Vec::new();
    ir.push(AxIr::ProgramStart);

    let mut line_start = 0usize;
    let mut line_num = 0usize;

    while line_start <= script.len() {
        let mut line_end = line_start;
        while line_end < script.len() && script[line_end] != b'\n' { line_end += 1; }
        let line = trim(&script[line_start..line_end]);

        if !line.is_empty() && !starts_with(line, b"//") && !starts_with(line, b"#") {
            compile_line(line, &mut ir, line_num)?;
        }

        if line_end >= script.len() { break; }
        line_start = line_end + 1;
        line_num += 1;
    }

    ir.push(AxIr::ProgramEnd);
    Ok(ir)
}

fn compile_line(line: &[u8], ir: &mut Vec<AxIr>, line_num: usize) -> Result<(), CompileError> {
    use axon_interp::{trim, starts_with};

    // print "text"
    if starts_with(line, b"print ") {
        let arg = trim(&line[6..]);
        if arg.len() >= 2 && arg[0] == b'"' && arg[arg.len()-1] == b'"' {
            let text = String::from_utf8_lossy(&arg[1..arg.len()-1]).to_string();
            ir.push(AxIr::PrintStr { text });
        } else {
            let var = String::from_utf8_lossy(arg).to_string();
            ir.push(AxIr::PrintVar { var });
        }
        return Ok(());
    }

    // let x = expr
    if starts_with(line, b"let ") {
        let rest = trim(&line[4..]);
        let eq = rest.iter().position(|&b| b == b'=')
            .ok_or(CompileError::MissingSyntax(line_num, "let: missing '='"))?;
        let name = String::from_utf8_lossy(trim(&rest[..eq])).to_string();
        let expr = trim(&rest[eq+1..]);

        // Try literal
        if let Some(n) = axon_interp::parse_i64(expr) {
            ir.push(AxIr::LoadImm { var: name, value: n });
            return Ok(());
        }

        // Try binary op
        for op_byte in [b'+', b'-', b'*'] {
            if let Some(pos) = find_op_byte(expr, op_byte) {
                let lhs = String::from_utf8_lossy(trim(&expr[..pos])).to_string();
                let rhs = String::from_utf8_lossy(trim(&expr[pos+1..])).to_string();
                let op = match op_byte {
                    b'+' => BinOpKind::Add,
                    b'-' => BinOpKind::Sub,
                    b'*' => BinOpKind::Mul,
                    _    => unreachable!(),
                };
                ir.push(AxIr::BinOp { dst: name, lhs, op, rhs });
                return Ok(());
            }
        }

        // Variable copy
        let src = String::from_utf8_lossy(expr).to_string();
        ir.push(AxIr::LoadVar { dst: name, src });
        return Ok(());
    }

    // awp
    if starts_with(line, b"awp ") {
        let payload = String::from_utf8_lossy(trim(&line[4..])).to_string();
        ir.push(AxIr::AwpSend { payload });
        return Ok(());
    }

    // fn / return — skip
    if starts_with(line, b"fn ") || starts_with(line, b"return ") {
        return Ok(());
    }

    Ok(()) // unknown lines silently skipped (matches interp behaviour)
}

fn find_op_byte(expr: &[u8], op: u8) -> Option<usize> {
    let mut i = expr.len();
    while i > 0 {
        i -= 1;
        if expr[i] == op && i > 0 { return Some(i); }
    }
    None
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    MissingSyntax(usize, &'static str),
    UnsupportedSyntax(usize, String),
    InvalidUtf8,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::MissingSyntax(l, s)    => write!(f, "line {}: {}", l, s),
            Self::UnsupportedSyntax(l,s) => write!(f, "line {}: unsupported: {}", l, s),
            Self::InvalidUtf8            => write!(f, "invalid UTF-8 in script"),
        }
    }
}
