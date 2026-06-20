# SOVEREIGNTY-LEDGER.md
**Copyright (c) 2026 Edison Lepiten / AIEONYX**

Clean-room implementation record.
Every entry documents external material studied before writing sovereign AXONYX code.
No code was copied. All implementations are original.

---

## Format
| Date | Studied | Purpose | Implementation | Clean Room |
|------|---------|---------|----------------|------------|

---

## Entries

| Date | Studied | Purpose | Implementation | Clean Room |
|------|---------|---------|----------------|------------|
| 2026-06-19 | IEEE 754 float spec — behavior of f32/f64 rounding | Understand why floats are unsafe for finance before writing Money<T> | `Money<currency, precision>` token + AST node in P55.5 — decimal-safe by design | Yes — no code copied |
| 2026-06-19 | libsodium API surface — secret key handling conventions | Understand zeroize-on-drop pattern before writing Secret<T> | `Secret<T>` token + AST node in P55.5 — Rust implementation writes own zeroize | Yes — no code copied |
| 2026-06-19 | GDPR Article 44-49 — data residency requirements | Understand jurisdiction law before writing Resident<T> | `Resident<T, jurisdiction>` token + AST node in P55.5 | Yes — legal text only, no code |
| 2026-06-19 | RFC 6962 Certificate Transparency — audit log chaining | Understand hash-chain audit patterns before writing Auditable<T> | `Auditable<T>` token + AST node in P55.5 — EdisonDB chain is independent design | Yes — no code copied |
| 2026-06-19 | COBOL COMP-3 packed decimal — BCD arithmetic for finance | Understand why banking uses fixed precision before writing Money<T> | `Money<currency, precision>` AST node — precision is a compile-time type parameter | Yes — concept only, no code |

---

*"We study the world to build better. We copy nothing."*
*— Edison Lepiten / AIEONYX, 2026*

---

## DEFERRED PHASES

### P55.7 — Finance CCP Profile + Constant-Time + Sealed Memory
**Deferred:** 2026-06-20
**Reason:** Not blocking P56. Features belong naturally in later phases.
**Return triggers:**
- `domain Finance` CCP profile → return here when writing first real Finance domain AXONYX app, or at P63 HANIEL domain profiles milestone
- `@constant_time` codegen → return here at P57 (axon_crypto) — mandatory before any crypto function ships
- `@sealed_memory` seL4 wiring → return here at P57 (axon_crypto + BASTION PD isolation)

**Contents when you return:**
1. Add `domain Finance` to CCP profile enum — rejects f32/f64 in Finance-tagged modules
2. `@constant_time` decorator enforcement in axon_codegen — rejects branches in marked functions
3. `@sealed_memory` runtime — mlock() on Linux, seL4 PD capability gate on BASTION

**Search tag:** DEFER-P55.7

| 2026-06-20 | RFC 7539 — ChaCha20 stream cipher specification | Understand quarter-round and block function before writing chacha20.rs | `chacha20.rs` sovereign implementation from scratch | Yes — spec only, no code copied |
| 2026-06-20 | RFC 8032 — Ed25519 Edwards-curve Digital Signature Algorithm | Understand seed-to-keypair and sign/verify structure before writing ed25519.rs | `ed25519.rs` P57.0 structural implementation; full curve at P57.1 | Yes — spec only, no code copied |
| 2026-06-20 | RFC 7748 — X25519 Elliptic Curve Diffie-Hellman | Understand key clamping and DH structure before writing x25519.rs | `x25519.rs` P57.0 approximation; Montgomery ladder at P57.1 | Yes — spec only, no code copied |
| 2026-06-20 | FIPS PUB 180-4 — SHA-256 specification | Understand compression function and padding before writing sha256 in identity.rs | `identity.rs` sovereign SHA-256 from scratch | Yes — spec only, no code copied |
| 2026-06-20 | RFC 3550 — RTP: A Transport Protocol for Real-Time Applications | Understand RTP header structure and packet framing before writing rtp.rs | `rtp.rs` sovereign implementation from scratch | Yes — spec only, no code copied |
| 2026-06-20 | ITU-T G.711 — Pulse Code Modulation of Voice Frequencies | Understand PCM audio format and telephony sample rates before writing pcm.rs | `pcm.rs` sovereign implementation from scratch | Yes — spec only, no code copied |
| 2026-06-20 | WebAssembly Core Specification 2.0 — binary format and execution semantics | Understand WASM section format, LEB128 encoding, and stack machine semantics before writing module.rs and runtime.rs | `module.rs`, `runtime.rs`, `validator.rs` sovereign implementations from scratch | Yes — spec only, no code copied |
| 2026-06-20 | Adobe BDF (Bitmap Distribution Format) specification | Understand glyph bitmap encoding before writing glyph.rs and builtin.rs | `glyph.rs`, `builtin.rs` sovereign implementations — pixel patterns original artwork | Yes — spec only, pixel patterns original |
