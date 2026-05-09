// ============================================================
// AXON CLI — main.rs
// Commands: axon version | axon check | axon build | axon run
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
// ============================================================

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use axon_lexer::FileId;
use axon_ai;
use axon_llvm;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { print_usage(); std::process::exit(1); }

    match args[1].as_str() {
        "version" => cmd_version(),
        "check"   => cmd_check(&args),
        "verify"  => cmd_verify(&args),
        "build"   => cmd_build(&args),
        "run"     => cmd_run(&args),
        other => {
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
    println!("  version                         Print AXON version");
    println!("  check  <file>                   Parse and verify AXON source
  verify <file>                   Formal @ensures/@requires verification");
    println!("  build  <file>                   Transpile AXON → Rust (Phase 3)");
    println!("  build  --native <file>          Compile AXON → native binary (Phase 4)");
    println!("  build  --native --target <t> <file>  Cross-compile (arm64, aarch64-sel4)");
    println!("  run    <file>                   Build and execute AXON program");
}

fn cmd_version() {
    println!("AXON 0.3.1-phase3");
    println!("Lexer:     complete (v0.3.1)");
    println!("Parser:    complete (P2-19 passed)");
    println!("Codegen:   phase 3 (Rust transpiler)");
    println!("Runtime:   axon_rt + axon_std (P3-05)");
    println!("Backend:   planned (LLVM, Phase 4)");
    println!("AI engine: planned (Phase 5)");
}

fn cmd_verify(args: &[String]) {
    // ── axon verify <file.axon> ──────────────────────────────
    // Runs formal verification on @ensures, @requires, @effect annotations.
    // Separate from axon check (syntax) — this is semantic verification.
    if args.len() < 3 {
        eprintln!("Usage: axon verify <file.axon>");
        std::process::exit(1);
    }
    let file   = &args[2];
    let source = read_file(file);

    println!("axon verify: {}", file);

    let results = axon_ai::verify_source(&source);

    if results.is_empty() {
        println!("  No @ensures/@requires annotations found.");
        println!("axon verify: {} — nothing to verify", file);
        return;
    }

    let mut violation_count = 0;
    let mut verified_count  = 0;
    let mut unknown_count   = 0;

    for result in &results {
        match result.status {
            axon_ai::VerificationStatus::Violated => {
                violation_count += 1;
                for v in &result.violations {
                    eprintln!("
error[E411]: @ensures constraint violated");
                    eprintln!("  → fn {} declares: {}", v.function_name, v.constraint);
                    eprintln!("  → violating path: {}", v.violating_path);
                    eprintln!("  → hint: {}", v.suggestion);
                }
            }
            axon_ai::VerificationStatus::Verified => {
                verified_count += 1;
                println!("  ✓ fn {} — @ensures verified on all paths", result.function_name);
            }
            axon_ai::VerificationStatus::Unknown => {
                unknown_count += 1;
                println!("  ? fn {} — unknown (cannot fully prove on all paths)", result.function_name);
                for w in &result.warnings {
                    println!("    {}", w);
                }
            }
            axon_ai::VerificationStatus::NotVerifiable => {}
        }
    }

    println!();
    if violation_count > 0 {
        eprintln!("axon verify: {} — {} violation(s) found", file, violation_count);
        std::process::exit(1);
    } else {
        println!("axon verify: {} — OK ({} verified, {} unknown)",
            file, verified_count, unknown_count);
    }
}

fn cmd_check(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: axon check <file.axon>");
        std::process::exit(1);
    }
    let path = &args[2];
    let source = read_file(path);
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
        for err in &result.errors { eprintln!("  {:?}", err); }
        std::process::exit(1);
    }
}

fn cmd_build(args: &[String]) {
    // Parse flags: axon build [--native] [--target <t>] <file.axon>
    let mut native = false;
    let mut target_name: Option<String> = None;
    let mut file_arg: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--native" => { native = true; i += 1; }
            "--target" => {
                i += 1;
                if i < args.len() {
                    target_name = Some(args[i].clone());
                    i += 1;
                }
            }
            _ => { file_arg = Some(args[i].clone()); i += 1; }
        }
    }

    let file = match file_arg {
        Some(f) => f,
        None => {
            eprintln!("Usage: axon build [--native] [--target <t>] <file.axon>");
            std::process::exit(1);
        }
    };

    if native {
        cmd_build_native(&file, target_name.as_deref());
    } else {
        let axon_path = Path::new(&file);
        let project_dir = build_project(axon_path);
        println!("axon build: {} → {}/", file, project_dir.display());
        println!("  Run: cd {} && cargo build", project_dir.display());
    }
}

fn cmd_build_native(file: &str, target_str: Option<&str>) {
    let source = read_file(file);

    // Resolve target
    let target = match target_str {
        Some(t) => match axon_llvm::Target::from_str(t) {
            Some(t) => t,
            None => {
                eprintln!("axon: unknown target '{}'. Valid: x86_64, arm64, aarch64-sel4", t);
                std::process::exit(1);
            }
        },
        None => axon_llvm::Target::X86_64Linux,
    };

    let stem = Path::new(file)
        .file_stem().unwrap_or_default()
        .to_string_lossy().to_string();
    let output_dir  = Path::new(file).parent().unwrap_or(Path::new("."));
    let output_stem = output_dir.join(&stem).to_string_lossy().to_string();

    println!("axon build --native: {} → {} ({})",
        file, stem, target.triple());

    // Only link when source has a main entry point
    let has_main = source.contains("fn main") || source.contains("task main");
    let link = !target.is_cross() && has_main;
    match axon_llvm::compile_native(&source, &output_stem, target, link) {
        Ok(out) => {
            if let Some(bin) = &out.binary_path {
                println!("
  Binary ready: {}", bin);
                println!("  Run: {}", bin);
            } else {
                println!("
  Object ready: {}", out.obj_path);
                println!("  IR ready:     {}", out.ll_path);
            }
        }
        Err(e) => {
            eprintln!("axon build --native failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_run(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: axon run <file.axon>");
        std::process::exit(1);
    }
    let axon_path = Path::new(&args[2]);
    let project_dir = build_project(axon_path);

    println!("axon run: compiling {}...", axon_path.display());
    let status = Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(project_dir.join("Cargo.toml"))
        .status()
        .expect("failed to invoke cargo");

    std::process::exit(status.code().unwrap_or(1));
}

/// Transpile an AXON file and generate a complete Cargo project.
/// Returns the path to the generated project directory.
fn build_project(axon_path: &Path) -> PathBuf {
    let source = read_file(&axon_path.to_string_lossy());

    // Step 1: Transpile to Rust
    let rust_source = match axon_codegen::codegen(&source) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("axon: transpile failed:\n{}", e);
            std::process::exit(1);
        }
    };

    // Step 2: Create project directory
    let stem = axon_path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let project_dir = axon_path.parent()
        .unwrap_or(Path::new("."))
        .join(format!("{}_axon_out", stem));

    fs::create_dir_all(project_dir.join("src"))
        .expect("cannot create project directory");

    // Step 3: Write generated lib.rs
    let lib_path = project_dir.join("src").join("lib.rs");
    fs::write(&lib_path, &rust_source)
        .expect("cannot write lib.rs");

    // Step 4: Write main.rs that re-exports the lib
    let main_rs = format!(
        "// Generated by AXON Transpiler — axon run\n\
         // To add program logic, edit {stem}.axon\n\
         use {stem}_axon_out::*;\n\n\
         fn main() {{\n\
             println!(\"AXON program '{}' loaded.\");\n\
             println!(\"Implement main() logic in {stem}.axon\");\n\
         }}\n",
        stem
    );
    fs::write(project_dir.join("src").join("main.rs"), main_rs)
        .expect("cannot write main.rs");

    // Step 5: Write Cargo.toml for the generated project
    // Find axon workspace root to reference axon_rt and axon_std
    let axon_root = find_axon_root();
    let cargo_toml = format!(
r#"[workspace]

[package]
name    = "{stem}_axon_out"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{stem}"
path = "src/main.rs"

[lib]
name = "{stem}_axon_out"
path = "src/lib.rs"

[dependencies]
axon_rt  = {{ path = "{axon_root}/axon_rt"  }}
axon_std = {{ path = "{axon_root}/axon_std" }}

[profile.release]
opt-level = 3
"#,
        stem = stem,
        axon_root = axon_root,
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)
        .expect("cannot write Cargo.toml");

    project_dir
}

fn find_axon_root() -> String {
    let home = env::var("HOME").unwrap_or_else(|_| "/home/edisonbl".to_string());
    let candidate = PathBuf::from(&home).join("axon");
    if candidate.join("axon_rt").exists() {
        return candidate.to_string_lossy().to_string();
    }
    // Walk up from exe
    if let Ok(exe) = env::current_exe() {
        let mut dir = exe.as_path();
        for _ in 0..8 {
            if let Some(parent) = dir.parent() {
                if parent.join("axon_rt").exists() {
                    return parent.to_string_lossy().to_string();
                }
                dir = parent;
            }
        }
    }
    format!("{}/axon", home)
}

fn dirs_home() -> PathBuf {
    env::var("HOME").map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/edisonbl"))
}

fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("axon: cannot read '{}': {}", path, e);
        std::process::exit(1);
    })
}
