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
