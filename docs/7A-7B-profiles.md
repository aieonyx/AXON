# AXON Phase 7A + 7B — CCP Profiles + ASP Patchable System

**Spec:** AXON-SPEC-P7A-v1.0 / AXON-SPEC-ASP-v1.0  
**Status:** LOCKED (foundational specs, not subject to revision)

---

## 7A — Compiler Capability Profiles (CCP)

### Overview

CCP defines four compilation profiles that gate which system capabilities are available to AXON code. The profile is selected at compile time and enforced by BASTION at commissioning.

### Profiles

| Profile | Description | BASTION Default |
|---------|-------------|-----------------|
| `seL4-strict` | Hardened sovereign node. Maximum capability restriction. All system calls gated through BASTION capability broker. | ✅ Accepted |
| `sovereign-offline` | Air-gapped sovereign operation. No network capabilities. Full local sovereignty. | ✅ Accepted |
| `mesh-node` | AWP mesh node operation. Network-capable but governed by AWP protocol, not IP. | ✅ Accepted |
| `dev-mode` | Development profile. Relaxed restrictions for local development. **BASTION rejects this profile in production.** | ❌ Rejected |

### Enforcement

CCP profiles are enforced at two levels:

1. **Compile-time:** The AXON compiler checks capability usage against the selected profile. Code that uses capabilities unavailable in the target profile fails to compile.
2. **Commissioning-time:** BASTION verifies the compiled image's profile signature before accepting it as a valid node.

### Profile Hierarchy

```
seL4-strict ⊂ sovereign-offline ⊂ mesh-node ⊂ dev-mode
```

A program that compiles under `seL4-strict` compiles under all profiles. A program that requires `dev-mode` capabilities cannot be deployed to production BASTION nodes.

### Key Rule

> **BASTION rejects dev-mode by default.**  
> No production commissioning key signs a dev-mode image. This is a hard architectural invariant, not a configuration option.

---

## 7B — AXON Secure Patchable System (ASP)

### Overview

ASP allows individual functions to be marked `#[patchable]`, enabling runtime hot-patch with cryptographic verification. Designed for zero-downtime sovereign node maintenance.

### Core Construct

```axon
#[patchable]
fn consensus_verify(block: &Block) -> bool {
    // ... implementation ...
}
```

### Three-Tier Verification

| Tier | Mechanism | Use Case |
|------|-----------|----------|
| Live IPC | Real-time patch via authenticated BASTION IPC channel | Standard operational patch |
| Scoped Token | Time-limited, scope-restricted patch authorization | Emergency operational patch |
| Emergency Token | Root Key signed, breaks glass | Critical security patch |

### Monotonic Counter Tokens

Every patch carries a monotonically-increasing counter value. BASTION maintains the counter per `#[patchable]` symbol. A patch with a counter value ≤ current counter is rejected. This prevents replay attacks.

### Patch-Class Rollback

Each `#[patchable]` function belongs to a patch class. If a patch in a class fails verification during commissioning, the entire class rolls back to the last verified state. Partial class application is not permitted.

### Policy PD Symbol Graph Signing

The set of `#[patchable]` symbols in a compiled image is represented as a directed graph (Policy PD). The entire graph is signed with the AXON Root Key during commissioning. Any modification to the patch graph (adding or removing patchable symbols) requires a new Root Key signature.

### Open Risks (Deferred to Production)

1. **Commissioning image integrity** — End-to-end verification of the full compiled image, not just the symbol graph
2. **Post-quantum signature rotation** — Current Ed25519; migration path to post-quantum signatures
3. **Root Key Ceremony** — Full ceremony with hardware security modules; currently Ed holds the key on USB-A/USB-B

---

## Integration with Phase 7 Compiler

The Phase 7 compiler enforces CCP profiles via the `CcpProfile` enum in `axon_codegen`. Profile selection flows from `run_phase7()` through all passes:

- **7E (Ownership):** `T: Send` bounds enforced per profile capability gates
- **7F (Dispatch):** dyn Trait dispatch restricted in `seL4-strict` (only object-safe traits)
- **7G (Verification):** Contract verification checks capability use in `@requires` clauses
- **7H (Emission):** Profile embedded in LLVM module metadata

---

*AXON-SPEC-P7A-v1.0 / AXON-SPEC-ASP-v1.0 — LOCKED*
