// axon_parser/src/profile.rs
// AXON CCP Profile Enforcement — Stage 8E
// Copyright © 2026 Edison Lepiten — AIEONYX
//
// Implements Phase 7A CCP (Capability Control Profiles).
// Four profiles:
//   seL4Strict       — maximum isolation, BASTION production
//   SovereignOffline — no network, local sovereign node
//   MeshNode         — controlled network, mesh participant
//   DevMode          — all capabilities, development only
//
// ARCH INVARIANT: BASTION rejects DevMode by default.
// Profile is checked at compile time — violations are errors.

use crate::hir::{HirModule, HirItem, HirFn};

// ============================================================
// CAPABILITY DEFINITIONS
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    // Network capabilities
    NetworkConnect,
    NetworkListen,
    NetworkRaw,
    // File system
    FileRead,
    FileWrite,
    FileExecute,
    // Memory
    AllocHeap,
    AllocStack,
    MmapAnon,
    // IPC
    IpcSend,
    IpcReceive,
    // Process
    Spawn,
    Signal,
    // Hardware
    HwAccess,
    HwDma,
    // Crypto
    CryptoSign,
    CryptoVerify,
    CryptoRng,
    // Unsafe AXON
    UnsafeAxon,
    // Patching
    Patchable,
}

impl Capability {
    pub fn name(&self) -> &'static str {
        match self {
            Capability::NetworkConnect => "network_connect",
            Capability::NetworkListen  => "network_listen",
            Capability::NetworkRaw     => "network_raw",
            Capability::FileRead       => "file_read",
            Capability::FileWrite      => "file_write",
            Capability::FileExecute    => "file_execute",
            Capability::AllocHeap      => "alloc_heap",
            Capability::AllocStack     => "alloc_stack",
            Capability::MmapAnon       => "mmap_anon",
            Capability::IpcSend        => "ipc_send",
            Capability::IpcReceive     => "ipc_receive",
            Capability::Spawn          => "spawn",
            Capability::Signal         => "signal",
            Capability::HwAccess       => "hw_access",
            Capability::HwDma          => "hw_dma",
            Capability::CryptoSign     => "crypto_sign",
            Capability::CryptoVerify   => "crypto_verify",
            Capability::CryptoRng      => "crypto_rng",
            Capability::UnsafeAxon     => "unsafe_axon",
            Capability::Patchable      => "patchable",
        }
    }

    pub fn from_str(s: &str) -> Option<Capability> {
        match s {
            "network_connect" => Some(Capability::NetworkConnect),
            "network_listen"  => Some(Capability::NetworkListen),
            "network_raw"     => Some(Capability::NetworkRaw),
            "file_read"       => Some(Capability::FileRead),
            "file_write"      => Some(Capability::FileWrite),
            "file_execute"    => Some(Capability::FileExecute),
            "alloc_heap"      => Some(Capability::AllocHeap),
            "alloc_stack"     => Some(Capability::AllocStack),
            "mmap_anon"       => Some(Capability::MmapAnon),
            "ipc_send"        => Some(Capability::IpcSend),
            "ipc_receive"     => Some(Capability::IpcReceive),
            "spawn"           => Some(Capability::Spawn),
            "signal"          => Some(Capability::Signal),
            "hw_access"       => Some(Capability::HwAccess),
            "hw_dma"          => Some(Capability::HwDma),
            "crypto_sign"     => Some(Capability::CryptoSign),
            "crypto_verify"   => Some(Capability::CryptoVerify),
            "crypto_rng"      => Some(Capability::CryptoRng),
            "unsafe_axon"     => Some(Capability::UnsafeAxon),
            "patchable"       => Some(Capability::Patchable),
            _                 => None,
        }
    }
}

// ============================================================
// CCP PROFILES — Phase 7A
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Profile {
    SeL4Strict,
    SovereignOffline,
    MeshNode,
    DevMode,
}

impl Profile {
    pub fn from_str(s: &str) -> Option<Profile> {
        match s {
            "seL4-strict"       | "sel4_strict"       => Some(Profile::SeL4Strict),
            "sovereign-offline" | "sovereign_offline" => Some(Profile::SovereignOffline),
            "mesh-node"         | "mesh_node"         => Some(Profile::MeshNode),
            "dev-mode"          | "dev_mode"          => Some(Profile::DevMode),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Profile::SeL4Strict       => "seL4-strict",
            Profile::SovereignOffline => "sovereign-offline",
            Profile::MeshNode         => "mesh-node",
            Profile::DevMode          => "dev-mode",
        }
    }

    /// ARCH INVARIANT: BASTION rejects DevMode by default.
    pub fn is_bastion_safe(&self) -> bool {
        !matches!(self, Profile::DevMode)
    }

    /// Capabilities allowed under this profile.
    pub fn allowed_capabilities(&self) -> Vec<Capability> {
        match self {
            // PEF4-WARN: SeL4Strict allows CryptoSign/CryptoVerify but not CryptoRng.
            // A deterministic nonce strategy must be documented before production.
            // Consider adding CryptoRng or mandating hardware RNG via seL4 capability.
            Profile::SeL4Strict => vec![
                Capability::AllocStack,
                Capability::IpcSend,
                Capability::IpcReceive,
                Capability::CryptoVerify,
                Capability::CryptoSign,
            ],
            Profile::SovereignOffline => vec![
                Capability::AllocHeap,
                Capability::AllocStack,
                Capability::FileRead,
                Capability::FileWrite,
                Capability::IpcSend,
                Capability::IpcReceive,
                Capability::CryptoSign,
                Capability::CryptoVerify,
                Capability::CryptoRng,
            ],
            Profile::MeshNode => vec![
                Capability::AllocHeap,
                Capability::AllocStack,
                Capability::FileRead,
                Capability::FileWrite,
                Capability::NetworkConnect,
                Capability::NetworkListen,
                Capability::IpcSend,
                Capability::IpcReceive,
                Capability::CryptoSign,
                Capability::CryptoVerify,
                Capability::CryptoRng,
            ],
            Profile::DevMode => vec![
                // All capabilities — development only
                Capability::NetworkConnect,
                Capability::NetworkListen,
                Capability::NetworkRaw,
                Capability::FileRead,
                Capability::FileWrite,
                Capability::FileExecute,
                Capability::AllocHeap,
                Capability::AllocStack,
                Capability::MmapAnon,
                Capability::IpcSend,
                Capability::IpcReceive,
                Capability::Spawn,
                Capability::Signal,
                Capability::HwAccess,
                Capability::HwDma,
                Capability::CryptoSign,
                Capability::CryptoVerify,
                Capability::CryptoRng,
                Capability::UnsafeAxon,
                Capability::Patchable,
            ],
        }
    }

    pub fn allows(&self, cap: &Capability) -> bool {
        self.allowed_capabilities().contains(cap)
    }

    pub fn forbidden_capabilities(&self) -> Vec<Capability> {
        let all = vec![
            Capability::NetworkConnect, Capability::NetworkListen,
            Capability::NetworkRaw, Capability::FileRead,
            Capability::FileWrite, Capability::FileExecute,
            Capability::AllocHeap, Capability::AllocStack,
            Capability::MmapAnon, Capability::IpcSend,
            Capability::IpcReceive, Capability::Spawn,
            Capability::Signal, Capability::HwAccess,
            Capability::HwDma, Capability::CryptoSign,
            Capability::CryptoVerify, Capability::CryptoRng,
            Capability::UnsafeAxon, Capability::Patchable,
        ];
        all.into_iter().filter(|c| !self.allows(c)).collect()
    }
}

// ============================================================
// PROFILE VIOLATION
// ============================================================

#[derive(Debug, Clone)]
pub struct ProfileViolation {
    pub capability: Capability,
    pub profile: Profile,
    pub location: String,
    pub msg: String,
}

impl ProfileViolation {
    pub fn new(cap: Capability, profile: &Profile, location: impl Into<String>) -> Self {
        let msg = format!(
            "capability '{}' is forbidden under profile '{}'",
            cap.name(), profile.name()
        );
        ProfileViolation {
            capability: cap,
            profile: profile.clone(),
            location: location.into(),
            msg,
        }
    }
}

impl std::fmt::Display for ProfileViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ProfileViolation at {}: {}", self.location, self.msg)
    }
}

// ============================================================
// PROFILE CHECKER
// ============================================================

pub struct ProfileChecker {
    pub profile: Profile,
    pub violations: Vec<ProfileViolation>,
}

impl ProfileChecker {
    pub fn new(profile: Profile) -> Self {
        ProfileChecker { profile, violations: Vec::new() }
    }

    pub fn check_module(&mut self, module: &HirModule) {
        for item in &module.items {
            self.check_item(item);
        }
    }

    fn check_item(&mut self, item: &HirItem) {
        match item {
            HirItem::Fn(f) => self.check_fn(f),
            HirItem::Impl(i) => {
                for m in &i.methods { self.check_fn(m); }
            }
            HirItem::Trait(t) => {
                for m in &t.methods { self.check_fn(m); }
            }
            _ => {}
        }
    }

    fn check_fn(&mut self, f: &HirFn) {
        let loc = format!("fn {}", f.name);
        // Check capability annotations on function signature
        // Functions named with capability prefixes are checked against profile
        // Full annotation checking awaits HirFn carrying cap attrs (Phase 9)
        // Current: check known forbidden capability name patterns in fn name
        // and check is_pure / is_ghost attributes
        let forbidden = self.profile.forbidden_capabilities();
        for cap in &forbidden {
            let cap_name = cap.name();
            // If fn name contains a forbidden capability name, flag it
            // e.g. fn network_connect_handler() under seL4-strict
            if f.name.contains(cap_name) && cap_name.len() > 4 {
                self.violations.push(ProfileViolation::new(
                    cap.clone(), &self.profile,
                    format!("{} (function references forbidden capability '{}')", loc, cap_name)
                ));
            }
        }
        // Check: pure functions cannot use network or file caps
        if f.is_pure {
            for cap in &[
                Capability::NetworkConnect, Capability::NetworkListen,
                Capability::FileRead, Capability::FileWrite,
            ] {
                if !self.profile.allows(cap) {
                    // Already forbidden by profile — no double-report
                } else {
                    // Pure fn cannot use I/O caps even if profile allows
                    self.violations.push(ProfileViolation::new(
                        cap.clone(), &self.profile,
                        format!("{} (pure fn cannot use I/O capabilities)", loc)
                    ));
                }
            }
        }
        // Check: unsafe_axon requires DevMode or explicit unsafe_axon cap
        if f.is_ghost && !self.profile.allows(&Capability::UnsafeAxon) {
            self.violations.push(ProfileViolation::new(
                Capability::UnsafeAxon, &self.profile, &loc
            ));
        }
        // Note: patchable attr checking requires attrs on HirFn — deferred to 8F
        // when HIR lowerer is updated to carry attrs through.
    }

    pub fn is_clean(&self) -> bool { self.violations.is_empty() }

    pub fn violation_count(&self) -> usize { self.violations.len() }
}

// ============================================================
// CLI ARG PARSING
// ============================================================

#[derive(Debug, Clone)]
pub struct CompilerArgs {
    pub input: Option<String>,
    pub output: Option<String>,
    pub profile: Profile,
    pub emit_ir: bool,
    pub verbose: bool,
}

impl CompilerArgs {
    pub fn default() -> Self {
        CompilerArgs {
            input: None,
            output: None,
            profile: Profile::SovereignOffline,
            emit_ir: false,
            verbose: false,
        }
    }

    /// Parse CLI args: axon build --profile seL4-strict input.axon -o output
    pub fn parse(args: &[String]) -> Result<CompilerArgs, String> {
        let mut result = CompilerArgs::default();
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--profile" | "-p" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("--profile requires a value".into());
                    }
                    result.profile = Profile::from_str(&args[i])
                        .ok_or_else(|| format!("unknown profile: {}", args[i]))?;
                }
                "--output" | "-o" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("--output requires a value".into());
                    }
                    result.output = Some(args[i].clone());
                }
                "--emit-ir" => { result.emit_ir = true; }
                "--verbose" | "-v" => { result.verbose = true; }
                arg if arg.ends_with(".axon") => {
                    result.input = Some(arg.to_string());
                }
                arg => {
                    return Err(format!("unknown argument: {}", arg));
                }
            }
            i += 1;
        }
        Ok(result)
    }
}

// ============================================================
// FULL PROFILE-GATED COMPILATION
// ============================================================

pub struct ProfiledCompileResult {
    pub violations: Vec<ProfileViolation>,
    pub ll_source: Option<String>,
    pub errors: Vec<String>,
}

/// Run profile check on a HirModule before codegen.
/// Returns violations — caller decides whether to abort.
pub fn check_profile(module: &HirModule, profile: Profile) -> Vec<ProfileViolation> {
    let mut checker = ProfileChecker::new(profile);
    checker.check_module(module);
    checker.violations
}

/// SEC3 FIX: Profile violations are fatal by default.
/// Call this from any CLI entry point after check_profile().
/// Any non-empty violations list aborts compilation with exit code 1.
/// API consumers MUST enforce the same behaviour.
pub fn enforce_profile(violations: &[ProfileViolation]) {
    if !violations.is_empty() {
        eprintln!("AXON profile enforcement: {} violation(s) found:", violations.len());
        for v in violations {
            eprintln!("  [VIOLATION] {}", v);
        }
        eprintln!("Compilation aborted. Profile violations are fatal.");
        std::process::exit(1);
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::hir::lower;

    fn check_src(src: &str, profile: Profile) -> Vec<ProfileViolation> {
        let items = parse(src).expect("parse failed");
        let module = lower(items);
        check_profile(&module, profile)
    }

    #[test]
    fn te1_profile_from_str() {
        assert_eq!(Profile::from_str("seL4-strict"),       Some(Profile::SeL4Strict));
        assert_eq!(Profile::from_str("sovereign-offline"), Some(Profile::SovereignOffline));
        assert_eq!(Profile::from_str("mesh-node"),         Some(Profile::MeshNode));
        assert_eq!(Profile::from_str("dev-mode"),          Some(Profile::DevMode));
        assert_eq!(Profile::from_str("unknown"),           None);
    }

    #[test]
    fn te2_sel4_strict_allows_ipc() {
        assert!(Profile::SeL4Strict.allows(&Capability::IpcSend));
        assert!(Profile::SeL4Strict.allows(&Capability::IpcReceive));
    }

    #[test]
    fn te3_sel4_strict_forbids_network() {
        assert!(!Profile::SeL4Strict.allows(&Capability::NetworkConnect));
        assert!(!Profile::SeL4Strict.allows(&Capability::NetworkListen));
        assert!(!Profile::SeL4Strict.allows(&Capability::NetworkRaw));
    }

    #[test]
    fn te4_dev_mode_allows_all() {
        assert!(Profile::DevMode.allows(&Capability::NetworkRaw));
        assert!(Profile::DevMode.allows(&Capability::UnsafeAxon));
        assert!(Profile::DevMode.allows(&Capability::HwDma));
        assert!(Profile::DevMode.allows(&Capability::Patchable));
    }

    #[test]
    fn te5_bastion_rejects_dev_mode() {
        // ARCH INVARIANT: BASTION rejects DevMode by default
        assert!(!Profile::DevMode.is_bastion_safe());
        assert!(Profile::SeL4Strict.is_bastion_safe());
        assert!(Profile::SovereignOffline.is_bastion_safe());
        assert!(Profile::MeshNode.is_bastion_safe());
    }

    #[test]
    fn te6_mesh_node_allows_network() {
        assert!(Profile::MeshNode.allows(&Capability::NetworkConnect));
        assert!(Profile::MeshNode.allows(&Capability::NetworkListen));
        assert!(!Profile::MeshNode.allows(&Capability::NetworkRaw));
    }

    #[test]
    fn te7_sovereign_offline_no_network() {
        assert!(!Profile::SovereignOffline.allows(&Capability::NetworkConnect));
        assert!(!Profile::SovereignOffline.allows(&Capability::NetworkListen));
        assert!(!Profile::SovereignOffline.allows(&Capability::NetworkRaw));
    }

    #[test]
    fn te8_patchable_attr_forbidden_in_sel4() {
        // DEFERRED: patchable attr checking awaits HirFn carrying attrs (8F)
        // For now verify clean fn has no violations
        let src = "fn update(x: i32) -> i32 { return x; }";
        let violations = check_src(src, Profile::SeL4Strict);
        assert!(violations.is_empty());
    }

    #[test]
    fn te9_patchable_allowed_in_dev_mode() {
        // DEFERRED: patchable attr checking awaits HirFn carrying attrs (8F)
        // Verify dev-mode has no violations on clean fn
        let src = "fn update(x: i32) -> i32 { return x; }";
        let violations = check_src(src, Profile::DevMode);
        assert!(violations.is_empty());
    }

    #[test]
    fn te10_clean_fn_no_violations() {
        let src = "fn add(x: i32, y: i32) -> i32 { return x; }";
        let violations = check_src(src, Profile::SeL4Strict);
        assert!(violations.is_empty(), "clean fn should have no violations");
    }

    #[test]
    fn te11_capability_from_str() {
        assert_eq!(Capability::from_str("network_connect"), Some(Capability::NetworkConnect));
        assert_eq!(Capability::from_str("patchable"),       Some(Capability::Patchable));
        assert_eq!(Capability::from_str("unsafe_axon"),     Some(Capability::UnsafeAxon));
        assert_eq!(Capability::from_str("unknown"),         None);
    }

    #[test]
    fn te12_forbidden_capabilities_sel4() {
        let forbidden = Profile::SeL4Strict.forbidden_capabilities();
        assert!(forbidden.contains(&Capability::NetworkConnect));
        assert!(forbidden.contains(&Capability::FileWrite));
        assert!(forbidden.contains(&Capability::UnsafeAxon));
        assert!(!forbidden.contains(&Capability::IpcSend));
    }

    #[test]
    fn te13_compiler_args_default_profile() {
        let args = CompilerArgs::default();
        assert_eq!(args.profile, Profile::SovereignOffline);
    }

    #[test]
    fn te14_compiler_args_parse_profile() {
        let args: Vec<String> = vec![
            "--profile".into(), "seL4-strict".into(), "main.axon".into()
        ];
        let parsed = CompilerArgs::parse(&args).unwrap();
        assert_eq!(parsed.profile, Profile::SeL4Strict);
        assert_eq!(parsed.input, Some("main.axon".into()));
    }

    #[test]
    fn te15_compiler_args_unknown_profile_error() {
        let args: Vec<String> = vec![
            "--profile".into(), "nonexistent".into()
        ];
        let result = CompilerArgs::parse(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown profile"));
    }
}
