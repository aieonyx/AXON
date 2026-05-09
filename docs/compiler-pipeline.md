# AXON Compiler Pipeline v1.0

> Architecture and contract specification for the AXON compiler.
> Copyright © 2026 Edison Lepiten — AIEONYX

---

## Overview

The AXON compiler (`axon`) is a multi-phase pipeline implemented in Rust.
Each phase has a defined input contract, output contract, and failure mode.
The pipeline is designed so that no phase's output is trusted without
verification from the previous phase.

```
Source (.axon)
    ↓ axon_lexer
Token stream (with indentation injection)
    ↓ axon_parser
AST (Program)
    ↓ axon_ai (optional — @ensures verification)
Verified AST
    ↓ axon_codegen (Rust transpiler path)
    │   OR
    ↓ axon_llvm (native path)
Rust source / LLVM IR
    ↓ rustc / llc-18 + clang-18
Native binary / .o object
```

---

## Phase 1 — Lexer (`axon_lexer`)

**Input:** Raw AXON source text (`&str`)

**Output:** `Vec<Token>` with indentation tokens injected

**Key step:** `inject_indentation(raw_tokens)` transforms physical newlines
and spaces into logical `Indent` and `Dedent` tokens that the parser uses
to delimit blocks.

**Contracts:**
- Every source file produces exactly one `Eof` token at the end
- `Indent`/`Dedent` tokens are balanced — every Indent has a matching Dedent
- Lexer errors are recoverable — unknown characters produce error tokens
  rather than aborting

**Token types include:**
- `Ident` — identifiers and keywords
- `Int`, `Float`, `Str`, `Bool` — literals
- `At` — decorator marker
- `LetAt` — ephemeral binding (`let@`)
- `ProgramIntentDecl` — `@program_intent` marker
- `Indent`, `Dedent` — block delimiters
- `Newline`, `Eof`

---

## Phase 2 — Parser (`axon_parser`)

**Input:** `Vec<Token>` (with indentation tokens)

**Output:** `ParseResult { program: Program, errors: Vec<ParseError> }`

**Key property:** The parser is error-tolerant. It collects errors and
continues parsing to produce the most complete AST possible.
A partial AST alongside errors is preferable to no AST.

**Contracts:**
- `program.items` contains all successfully parsed top-level declarations
- `errors` is empty for a valid AXON source file
- The parser never panics — all errors are collected into `errors`
- Decorators are attached to the declaration that immediately follows them
- A blank line between a decorator and its target causes the decorator
  to be detached (parser limitation — always put decorators immediately
  above their target)

**AST top-level nodes:**
- `TopLevelItem::Fn(FnDecl)` — function declaration
- `TopLevelItem::Task(TaskDecl)` — async task declaration
- `TopLevelItem::Struct(StructDecl)`
- `TopLevelItem::Enum(EnumDecl)`
- `TopLevelItem::TypeAlias(TypeAlias)`
- `TopLevelItem::Const(ConstDecl)`
- `TopLevelItem::Impl(ImplBlock)`

---

## Phase 3 — AI Verification (`axon_ai`) — Optional

**Input:** `Program` AST + source text

**Output:** `Vec<VerificationResult>` — one per annotated function

**Triggered by:** `axon verify <file>` or `axon suggest <file>`

**This phase has two sub-components:**

### 3a — Intent Translator (`IntentTranslator`)

Translates `@ai.intent("...")` natural language strings into
proposed `FormalSpec` values.

- Calls local Ollama instance (temperature=0 for determinism)
- Falls back to rule-based analysis if Ollama is unavailable
- Output is **advisory** — never gates compilation directly
- The LLM is not in the Trusted Computing Base

### 3b — Constraint Verifier (`ConstraintVerifier`)

Deterministic formal verifier. Checks `@ensures` / `@requires` constraints
against the function body using abstract interpretation.

**Abstract domain:** Interval + constant + nullability

**Verification states:**
- `Verified` — proven on all paths
- `Unknown` — conservative, cannot determine
- `Violated` — definite counterexample found
- `NotVerifiable` — no constraints to check

**Contracts:**
- Verification is deterministic — same source always produces same result
- No LLM calls during verification
- `Violated` is only reported when a definite counterexample is found
- `Unknown` is never promoted to `Verified`
- Effect checking is advisory in v0.5 (becomes enforced in P5-05)

**Constraint extractor (`constraint_parser`):**

Reads decorator AST nodes and maps them to `Constraint` enum values.

```
@ensures("result >= 0")  →  Constraint::ResultNonNegative
@ensures("result <= 2")  →  Constraint::ResultAtMost(2)
@ensures("result >= 1")  →  Constraint::ResultAtLeast(1)
@effect("pure")          →  Effect::Pure
```

Note: decorator arguments must be string literals.
Expression form `@ensures(result >= 0)` is parsed as three
separate arguments and will not produce a formal constraint.
Always use string form: `@ensures("result >= 0")`.

---

## Phase 4a — Rust Transpiler (`axon_codegen`)

**Input:** `Program` AST

**Output:** Rust source code (`String`)

**Used by:** `axon run` and `axon build` (without `--native`)

**Contracts:**
- Generated Rust code must compile with `rustc` without errors
- Generated Rust preserves AXON semantics for all supported constructs
- Defer statements are emitted at end of block (LIFO order preserved)
- Enum constructors use named fields matching AXON field names
- Module imports map to `axon_std::` namespace

**Runtime dependencies:**
- `axon_rt` — runtime primitives (defer guards, capability stubs)
- `axon_std` — standard library stubs (seL4 IPC, collective mesh)

---

## Phase 4b — LLVM Native Backend (`axon_llvm`)

**Input:** `Program` AST

**Output:** LLVM IR text (`.ll` file) + optionally `.o` object + binary

**Used by:** `axon build --native`

**Pipeline:**

```
Program AST
    ↓ LlvmCodegen::emit_program()
LLVM IR text (.ll)
    ↓ llc-18 -filetype=obj
Native object (.o)
    ↓ clang-18 (x86-64) or aarch64-linux-gnu-gcc (ARM64)
Executable binary
```

**Target triples:**

| Flag | Triple |
|---|---|
| (default) | `x86_64-unknown-linux-gnu` |
| `--target arm64` | `aarch64-unknown-linux-gnu` |
| `--target aarch64-sel4` | `aarch64-unknown-none-elf` |

**Contracts:**
- Generated LLVM IR must pass `llvm-as-18` verification
- Every function block must end with a terminator instruction
- All basic blocks must be reachable or explicitly marked unreachable
- Match statements compile to LLVM `switch` instructions
- All SSA values are uniquely named within their function scope

**Performance:**
- `classify()` benchmark: 190M calls/sec on Ryzen 7 (LLVM 18.1.3)
- Machine code quality equivalent to `rustc -O` for the same algorithm

---

## Phase 5 — CLI (`axon_cli`)

The `axon` command-line tool orchestrates all phases.

### Command Routing

```
axon version                         → print version
axon check <file>                    → Phase 1 + 2 only
axon verify <file>                   → Phase 1 + 2 + 3b
axon suggest <file>                  → Phase 1 + 2 + 3a
axon run <file>                      → Phase 1 + 2 + 4a + rustc + exec
axon build <file>                    → Phase 1 + 2 + 4a
axon build --native <file>           → Phase 1 + 2 + 4b + llc-18
axon build --native --target T <file>→ Phase 1 + 2 + 4b (cross-compile)
```

### Exit Codes

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | Parse error, verification violation, or build failure |

---

## Crate Structure

```
axon/                      Cargo workspace
  axon_lexer/              Tokenizer + indentation injection
  axon_parser/             AST builder
  axon_codegen/            Rust transpiler
  axon_rt/                 Runtime (defer guards, capability stubs)
  axon_std/                Standard library stubs
  axon_llvm/               LLVM IR native backend
  axon_ai/                 AI assistance engine + formal verifier
  axon_cli/                CLI entry point
```

---

## Error Codes

| Code | Phase | Meaning |
|---|---|---|
| `E001`–`E099` | Lexer | Tokenization errors |
| `E100`–`E299` | Parser | Syntax errors |
| `E300`–`E399` | Type | Type errors (future) |
| `E400`–`E499` | Verification | Formal constraint violations |
| `E411` | Verification | `@ensures` constraint violated |
| `E500`–`E599` | Codegen | Code generation errors |

---

*Version 1.0 — May 2026*
*Copyright © 2026 Edison Lepiten — AIEONYX*
