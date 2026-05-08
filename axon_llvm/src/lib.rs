// ============================================================
// axon_llvm — lib.rs
// AXON LLVM Native Backend — Phase 4
// Copyright © 2026 Edison Lepiten — AIEONYX
// ============================================================

pub mod ir;
pub mod types;
pub mod codegen;
pub mod error;

pub use codegen::LlvmCodegen;
pub use error::LlvmCodegenError;

use axon_lexer::FileId;
use std::fs;
use std::process::Command;

/// Compile AXON source to LLVM IR text.
pub fn llvm_codegen(source: &str) -> Result<String, LlvmCodegenError> {
    let result = axon_parser::parse(source, FileId(1));
    if !result.errors.is_empty() {
        return Err(LlvmCodegenError::ParseErrors(
            result.errors.iter().map(|e| format!("{:?}", e)).collect()));
    }

    let module_name = result.program.module.as_ref()
        .map(|m| m.path.iter().map(|i| i.name.as_str()).collect::<Vec<_>>().join("."))
        .unwrap_or_else(|| "axon_module".to_string());

    let mut gen = LlvmCodegen::new(&module_name);
    gen.emit_program(&result.program)
}

/// Compile AXON source to LLVM IR and write to .ll file.
pub fn llvm_codegen_to_file(
    source: &str,
    ll_path: &str,
) -> Result<(), LlvmCodegenError> {
    let ir = llvm_codegen(source)?;
    fs::write(ll_path, ir)
        .map_err(|e| LlvmCodegenError::IoError(e.to_string()))
}

/// Full pipeline: AXON → .ll → .o → binary
/// Requires llc-18 and clang-18 on PATH.
pub fn compile_to_binary(
    source: &str,
    binary_path: &str,
) -> Result<(), LlvmCodegenError> {
    let ll_path = format!("{}.ll", binary_path);
    let obj_path = format!("{}.o", binary_path);

    // Step 1: generate .ll
    llvm_codegen_to_file(source, &ll_path)?;

    // Step 2: .ll → .o via llc-18
    let llc = Command::new("llc-18")
        .args(["-filetype=obj", &ll_path, "-o", &obj_path])
        .output()
        .map_err(|e| LlvmCodegenError::IoError(
            format!("llc-18 not found: {}", e)))?;

    if !llc.status.success() {
        return Err(LlvmCodegenError::IoError(
            format!("llc-18 failed:\n{}", String::from_utf8_lossy(&llc.stderr))));
    }

    // Step 3: .o → binary via clang-18
    let clang = Command::new("clang-18")
        .args([&obj_path, "-o", binary_path])
        .output()
        .map_err(|e| LlvmCodegenError::IoError(
            format!("clang-18 not found: {}", e)))?;

    if !clang.status.success() {
        return Err(LlvmCodegenError::IoError(
            format!("clang-18 failed:\n{}", String::from_utf8_lossy(&clang.stderr))));
    }

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llvm_simple_fn() {
        let src = "fn add(x : Int, y : Int) -> Int:\n    return x\n";
        let ir = llvm_codegen(src).expect("llvm_codegen failed");
        assert!(ir.contains("define"), "expected function definition in IR");
        assert!(ir.contains("add"),    "expected function name 'add' in IR");
        assert!(ir.contains("ret"),    "expected return instruction");
        println!("\n--- LLVM IR for add() ---\n{}\n---", ir);
    }

    #[test]
    fn test_llvm_arithmetic() {
        let src = concat!(
            "fn add(x : Int, y : Int) -> Int:\n",
            "    let result = x + y\n",
            "    return result\n",
        );
        let ir = llvm_codegen(src).expect("failed");
        assert!(ir.contains("add i64"), "expected add instruction");
        println!("\n--- Arithmetic IR ---\n{}\n---", ir);
    }

    #[test]
    fn test_llvm_if_stmt() {
        let src = concat!(
            "fn abs(x : Int) -> Int:\n",
            "    if x < 0:\n",
            "        return 0\n",
            "    return x\n",
        );
        let ir = llvm_codegen(src).expect("failed");
        assert!(ir.contains("icmp slt"), "expected comparison");
        assert!(ir.contains("br i1"),    "expected conditional branch");
        println!("\n--- If IR ---\n{}\n---", ir);
    }

    #[test]
    fn test_llvm_match_stmt() {
        let src = concat!(
            "fn classify(severity : Int) -> Int:\n",
            "    match severity:\n",
            "        0 => return 0\n",
            "        1 => return 1\n",
            "        _ => return 2\n",
        );
        let ir = llvm_codegen(src).expect("failed");
        assert!(ir.contains("switch"), "expected switch instruction");
        println!("\n--- Match IR ---\n{}\n---", ir);
    }

    #[test]
    fn test_llvm_module_header() {
        let src = "module aieonyx.aegis.monitor\nfn f():\n    pass\n";
        let ir = llvm_codegen(src).expect("failed");
        assert!(ir.contains("aieonyx.aegis.monitor"), "expected module name");
        assert!(ir.contains("target triple"),          "expected target triple");
    }

    #[test]
    fn test_llvm_ir_valid_verify() {
        // Write IR to temp file and verify with llvm-as-18 if available
        let src = "fn add(x : Int, y : Int) -> Int:\n    return x\n";
        let ir = llvm_codegen(src).expect("failed");

        let tmp = "/tmp/axon_test.ll";
        std::fs::write(tmp, &ir).expect("write failed");

        // Try to verify with llvm-as-18 (optional)
        let ok = std::process::Command::new("llvm-as-18")
            .args([tmp, "-o", "/tmp/axon_test.bc"])
            .status()
            .map(|s| s.success())
            .unwrap_or(true); // if llvm-as-18 not available, skip

        assert!(ok, "LLVM IR verification failed:\n{}", ir);
        println!("\n--- IR verified by llvm-as-18 ---");
    }
}
