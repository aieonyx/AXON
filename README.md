# AXON — AI-Native Programming Language

**The first AI-native sovereign systems programming language designed with AI as a 
first-class compiler citizen.**

Write intent in plain English. The compiler verifies it at compile time.
Deploy to seL4 bare-metal.

```
@ai.intent("always returns a valid threat level between 0 and 2")
@ensures("result >= 0")
@ensures("result <= 2")
fn classify(severity : Int) -> Int:
    match severity:
        0 => return 0
        1 => return 1
        _ => return 2
```

## Overview
AXON is a general-purpose systems programming language 
combining memory safety, Python-like readability, and 
AI-assisted ownership inference baked into the compiler itself.

## What makes AXON different

- **AI as compiler phase** — not a linter. The verifier runs during compilation and can reject programs.
- **seL4 native** — `aarch64-unknown-none-elf` target. Built for the formally verified microkernel.
- **190M ops/sec** — LLVM 18 backend. Equivalent machine code quality to Rust.
- **Zero cloud** — all AI inference runs locally via Ollama.

## Status

| Phase | What | Status |
|---|---|---|
| 1 | Language design & spec | ✅ |
| 2 | Parser (105 tests) | ✅ |
| 3 | Rust transpiler (`axon run`) | ✅ |
| 4 | LLVM native backend (190M ops/sec) | ✅ |
| 5 | AI inference engine (`axon verify`, `axon suggest`) | ✅ |

## Benchmark Card

─────────────────────────────────────

classify()           100M calls    525ms            190 M/s

| Rust native (same conditions) |                224 M/s

|Rust with inlining  |               516 M/s



Machine code: AXON 26 bytes, Rust 20 bytes

Verdict: Equivalent quality. Different strategy.
##
## Built with

Rust · LLVM 18 · seL4 · Ollama · ARM64

## License

MIT · Copyright © 2026 Edison Lepiten - AIEONYX. All rights reserved.

## Author
Edison Lepiten — Founder & Project Director, AIEONYX

## Support

If AXON is useful or interesting to you, consider supporting its development:

[![Support on Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/aieonyx)

#rust, #programming-language, #sel4, #llvm, #formal-verification, #systems-programming, #compiler, #ai, #sovereign

