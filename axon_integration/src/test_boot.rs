// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Phase 42 — Live aarch64-seL4 boot integration test.
//!
//! Verifies the full AXON → seL4 pipeline:
//!   AXON source → aarch64 ELF → Microkit image → QEMU boot → axon_main() = 42
//!
//! Requires: qemu-system-aarch64, aarch64-linux-gnu-gcc, Microkit SDK 1.4.1
//! SDK path: ~/microkit-sdk-1.4.1

use std::process::Command;
use std::path::{Path, PathBuf};

fn sdk_path() -> String {
    std::env::var("AXON_MICROKIT_SDK")
        .unwrap_or_else(|_| format!("{}/microkit-sdk-1.4.1",
            std::env::var("HOME").unwrap_or_else(|_| "/home/edisonbl".into())))
}
fn repo_root() -> String {
    // Walk up from CARGO_MANIFEST_DIR to find repo root
    std::env::var("CARGO_MANIFEST_DIR")
        .map(|d| format!("{}", Path::new(&d).parent().unwrap().display()))
        .unwrap_or_else(|_| "/home/edisonbl/axon".into())
}
fn build_dir() -> String {
    std::env::var("AXON_P42_BUILD").unwrap_or_else(|_| "/tmp/axon_p42_boot".into())
}

const AXON_SRC: &str = r#"
fn axon_main() -> i32 {
    let x: i32 = 21;
    let y: i32 = 21;
    return x + y;
}
"#;

fn sdk_available(sdk: &str) -> bool {
    Path::new(sdk).join("bin/microkit").exists()
        && Path::new(sdk).join("board/qemu_virt_aarch64/debug/lib/libmicrokit.a").exists()
}

fn qemu_available() -> bool {
    Command::new("qemu-system-aarch64").arg("--version")
        .output().map(|o| o.status.success()).unwrap_or(false)
}

fn cross_gcc_available() -> bool {
    Command::new("aarch64-linux-gnu-gcc").arg("--version")
        .output().map(|o| o.status.success()).unwrap_or(false)
}

#[test]
#[ignore] // Run explicitly: cargo test p42_live_boot -- --ignored --nocapture
fn p42_live_aarch64_sel4_boot() {
    let sdk = sdk_path();
    let build_str = build_dir();
    let build = Path::new(&build_str);
    let root = repo_root();
    let system = format!("{}/docs/milestones/sel4-strict/axon.system", root);
    let c_harness = format!("{}/docs/milestones/sel4-strict/axon_pd.c", root);
    if !sdk_available(&sdk) { println!("SKIP: Microkit SDK not at {}", sdk); return; }
    if !qemu_available()     { println!("SKIP: qemu-system-aarch64 not found"); return; }
    if !cross_gcc_available(){ println!("SKIP: aarch64-linux-gnu-gcc not found"); return; }

    std::fs::create_dir_all(build).unwrap();
    let sdk_config = format!("{}/board/qemu_virt_aarch64/debug", sdk);

    // Step 1: Write AXON source
    let axon_src = build.join("axon_prog.axon");
    std::fs::write(&axon_src, AXON_SRC).unwrap();


    // Step 2: Compile AXON to aarch64-seL4 object
    let axon_obj = build.join("axon_prog.o");
    let r = Command::new("axon")
        .args(["build", "--profile", "seL4-strict", "--target", "aarch64-sel4",
               "-o", &build.join("axon_prog").to_string_lossy(),
               &axon_src.to_string_lossy()])
        .status().unwrap();
    assert!(r.success(), "AXON seL4 compile failed");
    assert!(axon_obj.exists(), "axon_prog.o not produced");

    // Step 3: Globalize axon_main symbol
    let axon_global = build.join("axon_prog_global.o");
    let r = Command::new("aarch64-linux-gnu-objcopy")
        .args(["--globalize-symbol=axon_main",
               &axon_obj.to_string_lossy(),
               &axon_global.to_string_lossy()])
        .status().unwrap();
    assert!(r.success(), "objcopy globalize failed");

    // Step 4: Compile C harness
    let pd_obj = build.join("axon_pd.o");
    let r = Command::new("aarch64-linux-gnu-gcc")
        .args(["-c", "-nostdlib", "-ffreestanding",
               &format!("-I{}/include", sdk_config),
               "-o", &pd_obj.to_string_lossy(),
               &c_harness])
        .status().unwrap();
    assert!(r.success(), "C harness compile failed");

    // Step 5: Link PD ELF
    let pd_elf = build.join("axon_pd.elf");
    let r = Command::new("aarch64-linux-gnu-gcc")
        .args(["-nostdlib",
               &format!("-T{}/lib/microkit.ld", sdk_config),
               &format!("-L{}/lib", sdk_config),
               &pd_obj.to_string_lossy(),
               &axon_global.to_string_lossy(),
               &format!("{}/lib/libmicrokit.a", sdk_config),
               "-o", &pd_elf.to_string_lossy()])
        .status().unwrap();
    assert!(r.success(), "PD ELF link failed");

    // Step 6: Build Microkit boot image
    let image = build.join("axon_image.img");
    let report = build.join("report.txt");
    let r = Command::new(format!("{}/bin/microkit", sdk))
        .args([&system,
               "--board", "qemu_virt_aarch64",
               "--config", "debug",
               "-o", &image.to_string_lossy(),
               "--report", &report.to_string_lossy(),
               "--search-path", &build.to_string_lossy()])
        .status().unwrap();
    assert!(r.success(), "Microkit image build failed");
    assert!(image.exists(), "boot image not produced");

    // Step 7: Boot on QEMU and verify output
    let output = Command::new("timeout")
        .args(["20", "qemu-system-aarch64",
               "-machine", "virt,virtualization=on,highmem=off",
               "-cpu", "cortex-a53",
               "-m", "2G",
               "-nographic",
               "-serial", "mon:stdio",
               "-device", &format!("loader,file={},addr=0x70000000,cpu-num=0",
                   image.to_string_lossy())])
        .output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    println!("=== QEMU output ===");
    println!("{}", combined);

    // Verify sovereign boot milestones
    assert!(combined.contains("AXON seL4-strict domain: ACTIVE"),
        "PD did not activate");
    assert!(combined.contains("axon_main() returned: 42"),
        "axon_main() did not return 42");
    assert!(combined.contains("AXON seL4 MILESTONE 2: PASSED"),
        "Milestone 2 not passed");

    println!("Phase 42 PASSED — AXON boots on aarch64-seL4, axon_main() = 42");

    // Cleanup build artifacts
    let _ = std::fs::remove_dir_all(build);
}
