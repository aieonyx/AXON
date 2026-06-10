// axon_cli/src/main.rs
// AXON Compiler CLI — Phase 8
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
//
// Commands:
//   axon version
//   axon build [--profile <p>] [-o <out>] <file.axon>
//   axon check <file.axon>
//   axon run <file.axon>

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { print_usage(); std::process::exit(1); }

    match args[1].as_str() {
        "version" => cmd_version(),
        "check"   => cmd_check(&args),
        "build"   => cmd_build(&args),
        "run"     => cmd_run(&args),
        other => {
            eprintln!("axon: unknown command '{}'. Try: version, check, build, run", other);
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("AXON — Sovereign Systems Programming Language");
    println!("Copyright © 2026 Edison Lepiten — AIEONYX");
    println!();
    println!("Usage: axon <command> [options] [file]");
    println!();
    println!("Commands:");
    println!("  version                                Print AXON version");
    println!("  check  <file.axon>                     Parse and type-check");
    println!("  build  [--profile <p>] [-o <out>] <file.axon>  Compile to binary");
    println!("  run    <file.axon>                     Build and run");
    println!();
    println!("Profiles:");
    println!("  seL4-strict       Maximum isolation (BASTION production)");
    println!("  sovereign-offline No network, local sovereign node (default)");
    println!("  mesh-node         Controlled network, mesh participant");
    println!("  dev-mode          All capabilities (development only)");
}

fn cmd_version() {
    println!("AXON 0.8.0-phase8");
    println!("Lexer:      complete");
    println!("Parser:     complete");
    println!("HIR:        complete");
    println!("Inference:  complete (HM)");
    println!("Codegen:    complete (LLVM 18)");
    println!("Stdlib:     complete (Vec, Option, Result, String)");
    println!("Profiles:   complete (seL4-strict, sovereign-offline, mesh-node, dev-mode)");
    println!("Target:     x86_64-pc-linux-gnu");
    println!("Copyright:  2026 Edison Lepiten — AIEONYX");
}

fn cmd_check(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: axon check <file.axon>");
        std::process::exit(1);
    }
    let file = &args[2];
    let source = read_file(file);

    match axon_parser::parser::parse(&source) {
        Ok(items) => {
            let module = axon_parser::hir::lower(items);
            if module.errors.is_empty() {
                println!("axon check: {} — OK", file);
                println!("  items: {}", module.items.len());
            } else {
                eprintln!("axon check: {} — {} HIR error(s)", file, module.errors.len());
                for e in &module.errors { eprintln!("  error: {}", e.msg); }
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("axon check: {} — parse error: {}", file, e);
            std::process::exit(1);
        }
    }
}

fn cmd_build(args: &[String]) {
    use axon_parser::profile::{Profile, check_profile, enforce_profile};
    use axon_parser::axon_manifest::parse_manifest;
    use axon_parser::parser::parse;
    use axon_parser::hir::lower;
    use axon_parser::codegen::{emit_ir, ir_to_object, object_to_binary, ir_to_ptx, ir_to_sel4, sel4_abi_check};

    let mut profile_str: Option<String> = None;
    let mut output: Option<String> = None;
    let mut file_arg: Option<String> = None;
    let mut emit_ir_flag = false;
    let mut target: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--profile" | "-p" => {
                i += 1;
                if i < args.len() { profile_str = Some(args[i].clone()); i += 1; }
            }
            "--output" | "-o" => {
                i += 1;
                if i < args.len() { output = Some(args[i].clone()); i += 1; }
            }
            "--target" | "-t" => {
                i += 1;
                if i < args.len() { target = Some(args[i].clone()); i += 1; }
            }
            "--emit-ir" => { emit_ir_flag = true; i += 1; }
            _ => { file_arg = Some(args[i].clone()); i += 1; }
        }
    }

    // P28: auto-detect axon.toml if no file arg given
    let (file, manifest_profile, manifest_target) = if file_arg.is_none() && std::path::Path::new("axon.toml").exists() {
        let toml_src = std::fs::read_to_string("axon.toml").unwrap_or_default();
        match parse_manifest(&toml_src) {
            Ok(m) => {
                println!("axon build: using axon.toml [{}]", m.name);
                let mp = if profile_str.is_none() { Some(m.profile.clone()) } else { None };
                let mt = if target.is_none() { Some(m.target.clone()) } else { None };
                (m.entry, mp, mt)
            }
            Err(e) => {
                eprintln!("axon: axon.toml error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        match file_arg {
            Some(f) => (f, None, None),
            None => {
                eprintln!("Usage: axon build [--profile <p>] [-o <out>] <file.axon>");
                std::process::exit(1);
            }
        }
    };
    // Apply manifest defaults if not overridden by CLI flags
    if profile_str.is_none() { if let Some(p) = manifest_profile { profile_str = Some(p); } }
    if target.is_none() { if let Some(t) = manifest_target { target = Some(t); } }

    // Resolve profile (default: sovereign-offline)
    let profile = match profile_str.as_deref() {
        Some(p) => match Profile::from_str(p) {
            Some(prof) => prof,
            None => {
                eprintln!("axon: unknown profile '{}'. Valid: seL4-strict, sovereign-offline, mesh-node, dev-mode", p);
                std::process::exit(1);
            }
        },
        None => Profile::SovereignOffline,
    };

    println!("axon build: {} [profile: {}]", file, profile.name());

    // Read and parse
    let source = read_file(&file);
    let items = match parse(&source) {
        Ok(items) => items,
        Err(e) => { eprintln!("axon: parse error: {}", e); std::process::exit(1); }
    };

    // Lower to HIR
    let module = lower(items);
    if !module.errors.is_empty() {
        for e in &module.errors { eprintln!("axon: error: {}", e.msg); }
        std::process::exit(1);
    }

    // Profile enforcement — violations are fatal (SEC3)
    let violations = check_profile(&module, profile);
    enforce_profile(&violations);

    // Emit LLVM IR
    let ir = emit_ir(&module);
    if emit_ir_flag {
        println!("{}", ir);
    }

    // Resolve output path
    let stem = Path::new(&file)
        .file_stem().unwrap_or_default()
        .to_string_lossy().to_string();
    let bin_path = output.unwrap_or_else(|| stem.clone());

    // GPU target: emit PTX instead of native binary
    if let Some(ref tgt) = target {
        if tgt == "nvptx64" || tgt.starts_with("sm_") {
            let sm = if tgt.starts_with("sm_") { tgt.trim_start_matches("sm_") }
                     else { "75" }; // T4 default
            println!("axon: targeting NVIDIA GPU sm_{} (nvptx64)", sm);
            match ir_to_ptx(&ir, "/tmp", sm) {
                Ok(ptx_path) => {
                    let out = format!("{}.ptx", bin_path);
                    std::fs::copy(&ptx_path, &out).ok();
                    println!("axon: PTX ready: {}", out);
                    println!("axon: validate: ptxas -arch=sm_{} {}", sm, out);
                    println!("axon: run on GPU via PyCUDA or cuLaunch");
                }
                Err(e) => { eprintln!("axon: PTX error: {}", e); std::process::exit(1); }
            }
            return;
        }
    }

    // seL4 target: compile to aarch64-unknown-none-elf
        if let Some(ref tgt) = target {
            if tgt == "aarch64-sel4" {
                println!("axon: targeting seL4 (aarch64-unknown-none-elf)");
                match ir_to_sel4(&ir, "/tmp") {
                    Ok(obj_path) => {
                        let out = format!("{}.o", bin_path);
                        std::fs::copy(&obj_path, &out).unwrap_or_else(|e| {
                            eprintln!("axon: copy failed: {}", e);
                            std::process::exit(1);
                        });
                        println!("axon: seL4 object ready: {}", out);
                        match sel4_abi_check(&out) {
                            Ok(()) => {
                                println!("axon: seL4 ABI check PASSED");
                                println!("axon: profile seL4-strict enforced");
                                println!("axon: binary ready for BASTION signing");
                            }
                            Err(v) => {
                                eprintln!("axon: seL4 ABI VIOLATIONS:
{}", v);
                                std::process::exit(1);
                            }
                        }
                    }
                    Err(e) => { eprintln!("axon: seL4 compile error: {}", e); std::process::exit(1); }
                }
                return;
            }
        }

        // CPU target: compile → object → binary
    let obj = match ir_to_object(&ir, "/tmp") {
        Ok(p) => p,
        Err(e) => { eprintln!("axon: compile error: {}", e); std::process::exit(1); }
    };
    match object_to_binary(&obj, &bin_path) {
        Ok(()) => {
            println!("axon: binary ready: {}", bin_path);
            println!("axon: run with: ./{}", bin_path);
        }
        Err(e) => { eprintln!("axon: link error: {}", e); std::process::exit(1); }
    }
}

fn cmd_run(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: axon run <file.axon>");
        std::process::exit(1);
    }
    let file = &args[2].clone();
    // Build first
    let build_args = vec![
        "axon".to_string(),
        "build".to_string(),
        file.clone(),
    ];
    cmd_build(&build_args);
    // Run the binary
    let stem = Path::new(file)
        .file_stem().unwrap_or_default()
        .to_string_lossy().to_string();
    let status = std::process::Command::new(format!("./{}", stem))
        .status()
        .unwrap_or_else(|e| { eprintln!("axon: run error: {}", e); std::process::exit(1); });
    std::process::exit(status.code().unwrap_or(1));
}

fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("axon: cannot read '{}': {}", path, e);
        std::process::exit(1);
    })
}
