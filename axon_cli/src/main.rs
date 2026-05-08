// ============================================================
// AXON CLI — main.rs
// Commands: axon version | axon check | axon build
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
// ============================================================

use std::env;
use std::fs;
use std::path::Path;
use axon_lexer::FileId;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "version" => cmd_version(),
        "check"   => cmd_check(&args),
        "build"   => cmd_build(&args),
        other     => {
            eprintln!("axon: unknown command '{}'", other);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("AXON — AI-Native Sovereign Systems Programming Language");
    println!("Copyright © 2026 Edison Lepiten — AIEONYX");
    println!();
    println!("Usage: axon <command> [file]");
    println!();
    println!("Commands:");
    println!("  version         Print AXON version");
    println!("  check  <file>   Parse and verify an AXON source file");
    println!("  build  <file>   Transpile AXON source to Rust");
}

fn cmd_version() {
    println!("AXON 0.3.1-phase3");
    println!("Lexer:     complete (v0.3.1)");
    println!("Parser:    complete (P2-19 passed)");
    println!("Codegen:   phase 3 (Rust transpiler)");
    println!("Backend:   planned (LLVM, Phase 4)");
    println!("AI engine: planned (Phase 5)");
}

fn cmd_check(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: axon check <file.axon>");
        std::process::exit(1);
    }
    let path = &args[2];
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("axon: cannot read '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let result = axon_parser::parse(&source, FileId(1));
    if result.errors.is_empty() {
        println!("axon check: {} — OK", path);
        if let Some(m) = &result.program.module {
            let p = m.path.iter().map(|i| i.name.as_str()).collect::<Vec<_>>().join(".");
            println!("  module:  {}", p);
        }
        println!("  imports: {}", result.program.imports.len());
        println!("  items:   {}", result.program.items.len());
    } else {
        eprintln!("axon check: {} — {} error(s)", path, result.errors.len());
        for err in &result.errors {
            eprintln!("  {:?}", err);
        }
        std::process::exit(1);
    }
}

fn cmd_build(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: axon build <file.axon>");
        std::process::exit(1);
    }
    let path = &args[2];
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("axon: cannot read '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    match axon_codegen::codegen(&source) {
        Ok(rust_source) => {
            // Write to <file>.rs
            let out_path = Path::new(path)
                .with_extension("rs")
                .to_string_lossy()
                .to_string();
            match fs::write(&out_path, &rust_source) {
                Ok(_) => {
                    println!("axon build: {} → {}", path, out_path);
                    println!("  Rust source written. Run: rustc {}", out_path);
                }
                Err(e) => {
                    eprintln!("axon: cannot write '{}': {}", out_path, e);
                    // Print to stdout as fallback
                    print!("{}", rust_source);
                }
            }
        }
        Err(e) => {
            eprintln!("axon build failed:\n{}", e);
            std::process::exit(1);
        }
    }
}
