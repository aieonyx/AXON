// ============================================================
// AXON Compiler CLI — axon_cli/src/main.rs
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
//
// Entry point for the 'axon' command.
// Phase 2: check command wired to parser.
// ============================================================

use std::env;
use std::fs;
use std::process;

const VERSION : &str = "0.1.0";
const BANNER  : &str = r#"
   ___  _  ______  _   _
  / _ \| | \ \ \ \| \ | |
 / /_\ \ |  \ \ \ \  \| |
 |  _  | |  / / / / |\  |
 | | | | |_/ / / /| | \ |
 \_| |_/___/_/_/_/ |_|  \_|

 AI-Native Programming Language
 Copyright (c) 2026 Edison Lepiten — AIEONYX
 github.com/aieonyx/axon
"#;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        process::exit(0);
    }

    match args[1].as_str() {
        "version" | "--version" | "-v" => {
            cmd_version();
        },
        "check" => {
            if args.len() < 3 {
                eprintln!("error: 'axon check' requires a file path");
                eprintln!("usage: axon check <file.axon>");
                process::exit(1);
            }
            cmd_check(&args[2]);
        },
        "build" => {
            eprintln!("axon build: not yet implemented (Phase 4)");
            eprintln!("use 'axon check' to validate syntax");
            process::exit(1);
        },
        "help" | "--help" | "-h" => {
            print_help();
        },
        unknown => {
            eprintln!("error: unknown command '{}'", unknown);
            eprintln!("run 'axon help' for usage");
            process::exit(1);
        }
    }
}

fn cmd_version() {
    println!("{}", BANNER);
    println!("axon compiler  v{}", VERSION);
    println!("phase          2 — parser (in progress)");
    println!("built by       Edison Lepiten — AIEONYX");
    println!("repository     github.com/aieonyx/axon");
}

fn cmd_check(path: &str) {
    // Read source file
    let source = match fs::read_to_string(path) {
        Ok(s)  => s,
        Err(e) => {
            eprintln!("error: could not read '{}': {}", path, e);
            process::exit(1);
        }
    };

    println!("axon check: {}", path);

    // Lex
    let file_id = axon_lexer::FileId(0);
    let tokens  = axon_lexer::lex(&source, file_id);

    println!("  lexer  : {} tokens", tokens.len());

    // Parse
    let result = axon_parser::parse(&source, file_id);

    if result.is_ok() {
        println!("  parser : ok");
        println!("  result : no errors");
        process::exit(0);
    } else {
        println!("  parser : {} error(s)", result.errors.len());
        for err in &result.errors {
            eprintln!("{}", err.display(&source));
        }
        process::exit(1);
    }
}

fn print_help() {
    println!("{}", BANNER);
    println!("USAGE:");
    println!("    axon <command> [arguments]");
    println!();
    println!("COMMANDS:");
    println!("    check  <file.axon>    Check syntax and report errors");
    println!("    build  <file.axon>    Compile to binary (Phase 4 — not yet available)");
    println!("    version               Print compiler version");
    println!("    help                  Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    axon check src/main.axon");
    println!("    axon version");
    println!();
    println!("PHASE STATUS:");
    println!("    Phase 2 — Parser (current)");
    println!("    Phase 3 — Transpiler (planned)");
    println!("    Phase 4 — Native compiler via LLVM (planned)");
    println!();
    println!("Copyright (c) 2026 Edison Lepiten — AIEONYX");
    println!("github.com/aieonyx/axon");
}
