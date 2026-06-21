// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P61.1 — ECHO sovereign WASM JIT: x86_64 native code generation
// Single-pass WASM bytecode → x86_64 machine code, no external JIT library.

use crate::error::WasmError;
use crate::module::{WasmModule, WasmFunc};
use crate::types::ValType;

// ── Executable code buffer ────────────────────────────────────────────────────

/// Wraps an mmap'd executable page. Filled with x86_64 machine bytes,
/// then called via a raw function pointer.
pub struct CodeBuffer {
    ptr: *mut u8,
    len: usize,
    pos: usize,
}

impl CodeBuffer {
    /// Allocate `size` bytes of RWX memory (mmap anonymous).
    pub fn new(size: usize) -> Result<Self, WasmError> {
        let size = size.max(4096);
        let ptr = unsafe {
            libc_mmap(size)
        };
        if ptr.is_null() {
            return Err(WasmError::Trap("jit: mmap failed".into()));
        }
        Ok(Self { ptr, len: size, pos: 0 })
    }

    /// Emit a single byte.
    #[inline]
    pub fn emit_u8(&mut self, b: u8) -> Result<(), WasmError> {
        if self.pos >= self.len {
            return Err(WasmError::Trap("jit: code buffer overflow".into()));
        }
        unsafe { self.ptr.add(self.pos).write(b); }
        self.pos += 1;
        Ok(())
    }

    /// Emit a slice of bytes.
    pub fn emit_bytes(&mut self, bs: &[u8]) -> Result<(), WasmError> {
        for &b in bs { self.emit_u8(b)?; }
        Ok(())
    }

    /// Emit a little-endian i32.
    pub fn emit_i32_le(&mut self, v: i32) -> Result<(), WasmError> {
        let b = v.to_le_bytes();
        self.emit_bytes(&b)
    }

    /// Emit a little-endian i64.
    pub fn emit_i64_le(&mut self, v: i64) -> Result<(), WasmError> {
        let b = v.to_le_bytes();
        self.emit_bytes(&b)
    }

    /// Return the entry point as a callable function pointer.
    /// Signature: fn(locals_ptr: *mut i64) -> i64
    /// The JIT calling convention passes a pointer to the locals array;
    /// the return value is in rax.
    pub fn entry_fn(&self) -> unsafe extern "C" fn(*mut i64) -> i64 {
        unsafe { std::mem::transmute(self.ptr) }
    }

    pub fn written(&self) -> usize { self.pos }
}

impl Drop for CodeBuffer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { libc_munmap(self.ptr, self.len); }
        }
    }
}

// ── mmap / munmap shims ───────────────────────────────────────────────────────
// Avoids pulling in the `libc` crate; calls Linux syscalls directly.

unsafe fn libc_mmap(size: usize) -> *mut u8 {
    // mmap(NULL, size, PROT_READ|PROT_WRITE|PROT_EXEC,
    //      MAP_PRIVATE|MAP_ANONYMOUS, -1, 0)
    // syscall NR_mmap = 9 on x86_64
    let addr: usize;
    std::arch::asm!(
        "syscall",
        inout("rax") 9usize => addr,
        in("rdi") 0usize,          // addr = NULL
        in("rsi") size,
        in("rdx") 0x7usize,        // PROT_READ|PROT_WRITE|PROT_EXEC
        in("r10") 0x22usize,       // MAP_PRIVATE|MAP_ANONYMOUS
        in("r8")  usize::MAX,      // fd = -1
        in("r9")  0usize,          // offset = 0
        out("rcx") _,
        out("r11") _,
        options(nostack),
    );
    if addr > usize::MAX - 4096 { return std::ptr::null_mut(); }
    addr as *mut u8
}

unsafe fn libc_munmap(ptr: *mut u8, size: usize) {
    // munmap syscall NR = 11 on x86_64
    std::arch::asm!(
        "syscall",
        inout("rax") 11usize => _,
        in("rdi") ptr as usize,
        in("rsi") size,
        out("rcx") _,
        out("r11") _,
        options(nostack),
    );
}

// ── x86_64 opcode emitter ─────────────────────────────────────────────────────

/// JIT calling convention:
///   rdi = pointer to locals array (i64[])
///   rax = return value
///   rsp aligned to 16 bytes at call site (System V AMD64 ABI)
///
/// Value stack: implemented as a fixed shadow array on the real stack,
/// indexed by a compile-time stack depth counter (no heap allocation).
/// Stack slots are pushed/popped via rsp-relative addressing.
/// Max WASM stack depth: 64 values (512 bytes of shadow space).

const MAX_STACK: usize = 64;
const SLOT: i32 = 8; // bytes per slot (i64)

pub struct X64Emitter<'a> {
    pub buf: &'a mut CodeBuffer,
    pub stack_depth: usize,  // current JIT value stack depth
    pub locals_count: usize,
}

impl<'a> X64Emitter<'a> {
    pub fn new(buf: &'a mut CodeBuffer, locals_count: usize) -> Self {
        Self { buf, stack_depth: 0, locals_count }
    }

    // ── Prologue: push rbp, mov rbp rsp, sub rsp shadow, save rdi ───────────
    // rdi (locals ptr) saved at [rbp-8].
    // Shadow stack for values starts at [rbp-16] downward.
    pub fn prologue(&mut self) -> Result<(), WasmError> {
        // push rbp
        self.buf.emit_u8(0x55)?;
        // mov rbp, rsp
        self.buf.emit_bytes(&[0x48, 0x89, 0xE5])?;
        // sub rsp, 512+8 (64 slots * 8 bytes + locals ptr slot, 16-byte aligned)
        let frame = ((MAX_STACK * 8 + 8 + 15) / 16) * 16;
        self.buf.emit_bytes(&[0x48, 0x81, 0xEC])?;
        self.buf.emit_i32_le(frame as i32)?;
        // mov [rbp-8], rdi  (save locals ptr)
        self.buf.emit_bytes(&[0x48, 0x89, 0x7D, 0xF8])?;
        Ok(())
    }

    // ── Epilogue: pop value into rax, restore rsp, pop rbp, ret ─────────────
    pub fn epilogue(&mut self) -> Result<(), WasmError> {
        if self.stack_depth > 0 {
            // pop rax (return value)
            self.pop_rax()?;
        } else {
            // xor rax, rax (void / no return)
            self.buf.emit_bytes(&[0x48, 0x31, 0xC0])?;
        }
        // leave (mov rsp,rbp; pop rbp)
        self.buf.emit_u8(0xC9)?;
        // ret
        self.buf.emit_u8(0xC3)?;
        Ok(())
    }

    // ── Stack helpers ─────────────────────────────────────────────────────────

    /// Push rax onto the JIT value stack (store to next slot).
    fn push_rax(&mut self) -> Result<(), WasmError> {
        if self.stack_depth >= MAX_STACK {
            return Err(WasmError::Trap("jit: stack overflow".into()));
        }
        // slot offset from rbp: -(16 + depth*8)
        let off = -((16 + self.stack_depth as i32 * SLOT) as i32);
        // mov [rbp + off], rax
        self.buf.emit_bytes(&[0x48, 0x89, 0x85])?;
        self.buf.emit_i32_le(off)?;
        self.stack_depth += 1;
        Ok(())
    }

    /// Pop top of JIT value stack into rax.
    fn pop_rax(&mut self) -> Result<(), WasmError> {
        if self.stack_depth == 0 {
            return Err(WasmError::Trap("jit: stack underflow".into()));
        }
        self.stack_depth -= 1;
        let off = -((16 + self.stack_depth as i32 * SLOT) as i32);
        // mov rax, [rbp + off]
        self.buf.emit_bytes(&[0x48, 0x8B, 0x85])?;
        self.buf.emit_i32_le(off)?;
        Ok(())
    }

    /// Peek at top (rax) without decrementing depth.
    fn peek_rax(&mut self) -> Result<(), WasmError> {
        if self.stack_depth == 0 {
            return Err(WasmError::Trap("jit: peek on empty stack".into()));
        }
        let off = -((16 + (self.stack_depth as i32 - 1) * SLOT) as i32);
        self.buf.emit_bytes(&[0x48, 0x8B, 0x85])?;
        self.buf.emit_i32_le(off)?;
        Ok(())
    }

    // ── WASM opcode emitters ──────────────────────────────────────────────────

    /// i32.const N  — push sign-extended 32-bit immediate as i64
    pub fn emit_i32_const(&mut self, v: i32) -> Result<(), WasmError> {
        // mov rax, sign_extend(v)
        // Using movsx rax, imm32 via mov eax, imm32; movsxd rax, eax
        self.buf.emit_bytes(&[0xB8])?;   // mov eax, imm32
        self.buf.emit_i32_le(v)?;
        // movsxd rax, eax
        self.buf.emit_bytes(&[0x48, 0x63, 0xC0])?;
        self.push_rax()
    }

    /// i64.const N
    pub fn emit_i64_const(&mut self, v: i64) -> Result<(), WasmError> {
        // mov rax, imm64
        self.buf.emit_bytes(&[0x48, 0xB8])?;
        self.buf.emit_i64_le(v)?;
        self.push_rax()
    }

    /// i32.add / i64.add
    pub fn emit_add(&mut self) -> Result<(), WasmError> {
        self.pop_rax()?;                       // rax = b
        // mov rcx, [rbp + top_slot]           (rcx = a)
        let off = -((16 + self.stack_depth as i32 * SLOT - SLOT) as i32);
        self.buf.emit_bytes(&[0x48, 0x8B, 0x8D])?;
        self.buf.emit_i32_le(off)?;
        // add rax, rcx
        self.buf.emit_bytes(&[0x48, 0x01, 0xC8])?;
        // overwrite top slot with result
        self.stack_depth -= 1;
        self.push_rax()
    }

    /// i32.sub / i64.sub  (a - b, a is deeper)
    pub fn emit_sub(&mut self) -> Result<(), WasmError> {
        self.pop_rax()?;                       // rax = b
        let off = -((16 + self.stack_depth as i32 * SLOT - SLOT) as i32);
        self.buf.emit_bytes(&[0x48, 0x8B, 0x8D])?;
        self.buf.emit_i32_le(off)?;
        // sub rcx, rax  (a - b → rcx)
        self.buf.emit_bytes(&[0x48, 0x29, 0xC1])?;
        // mov rax, rcx
        self.buf.emit_bytes(&[0x48, 0x89, 0xC8])?;
        self.stack_depth -= 1;
        self.push_rax()
    }

    /// i32.mul / i64.mul
    pub fn emit_mul(&mut self) -> Result<(), WasmError> {
        self.pop_rax()?;                       // rax = b
        let off = -((16 + self.stack_depth as i32 * SLOT - SLOT) as i32);
        self.buf.emit_bytes(&[0x48, 0x8B, 0x8D])?;
        self.buf.emit_i32_le(off)?;
        // imul rax, rcx
        self.buf.emit_bytes(&[0x48, 0x0F, 0xAF, 0xC1])?;
        self.stack_depth -= 1;
        self.push_rax()
    }

    /// i32.and / i64.and
    pub fn emit_and(&mut self) -> Result<(), WasmError> {
        self.pop_rax()?;
        let off = -((16 + self.stack_depth as i32 * SLOT - SLOT) as i32);
        self.buf.emit_bytes(&[0x48, 0x8B, 0x8D])?;
        self.buf.emit_i32_le(off)?;
        // and rax, rcx
        self.buf.emit_bytes(&[0x48, 0x21, 0xC8])?;
        self.stack_depth -= 1;
        self.push_rax()
    }

    /// i32.or / i64.or
    pub fn emit_or(&mut self) -> Result<(), WasmError> {
        self.pop_rax()?;
        let off = -((16 + self.stack_depth as i32 * SLOT - SLOT) as i32);
        self.buf.emit_bytes(&[0x48, 0x8B, 0x8D])?;
        self.buf.emit_i32_le(off)?;
        // or rax, rcx
        self.buf.emit_bytes(&[0x48, 0x09, 0xC8])?;
        self.stack_depth -= 1;
        self.push_rax()
    }

    /// local.get idx — load local[idx] onto value stack
    pub fn emit_local_get(&mut self, idx: u32) -> Result<(), WasmError> {
        if idx as usize >= self.locals_count {
            return Err(WasmError::Trap(format!("jit: local.get {} out of range", idx)));
        }
        // mov rdi, [rbp-8]   (reload locals ptr)
        self.buf.emit_bytes(&[0x48, 0x8B, 0x7D, 0xF8])?;
        // mov rax, [rdi + idx*8]
        let off = idx as i32 * 8;
        self.buf.emit_bytes(&[0x48, 0x8B, 0x87])?;
        self.buf.emit_i32_le(off)?;
        self.push_rax()
    }

    /// local.set idx — pop value stack into local[idx]
    pub fn emit_local_set(&mut self, idx: u32) -> Result<(), WasmError> {
        if idx as usize >= self.locals_count {
            return Err(WasmError::Trap(format!("jit: local.set {} out of range", idx)));
        }
        self.pop_rax()?;
        // mov rdi, [rbp-8]
        self.buf.emit_bytes(&[0x48, 0x8B, 0x7D, 0xF8])?;
        // mov [rdi + idx*8], rax
        let off = idx as i32 * 8;
        self.buf.emit_bytes(&[0x48, 0x89, 0x87])?;
        self.buf.emit_i32_le(off)?;
        Ok(())
    }

    /// local.tee idx — peek + set (value stays on stack)
    pub fn emit_local_tee(&mut self, idx: u32) -> Result<(), WasmError> {
        if idx as usize >= self.locals_count {
            return Err(WasmError::Trap(format!("jit: local.tee {} out of range", idx)));
        }
        self.peek_rax()?;
        // mov rdi, [rbp-8]
        self.buf.emit_bytes(&[0x48, 0x8B, 0x7D, 0xF8])?;
        let off = idx as i32 * 8;
        self.buf.emit_bytes(&[0x48, 0x89, 0x87])?;
        self.buf.emit_i32_le(off)?;
        Ok(())
    }

    /// drop — discard top of value stack
    pub fn emit_drop(&mut self) -> Result<(), WasmError> {
        if self.stack_depth == 0 {
            return Err(WasmError::Trap("jit: drop on empty stack".into()));
        }
        self.stack_depth -= 1;
        Ok(())
    }

    /// nop
    pub fn emit_nop(&mut self) -> Result<(), WasmError> {
        self.buf.emit_u8(0x90)?;
        Ok(())
    }

    /// return — emit epilogue inline (early return)
    pub fn emit_return(&mut self) -> Result<(), WasmError> {
        self.epilogue()
    }

    /// unreachable — emit ud2 (illegal instruction → SIGILL)
    pub fn emit_unreachable(&mut self) -> Result<(), WasmError> {
        self.buf.emit_bytes(&[0x0F, 0x0B])?;
        Ok(())
    }
}

// ── JIT compiler ──────────────────────────────────────────────────────────────

/// A compiled WASM function — owns the executable code buffer.
pub struct JitFunction {
    pub buf: CodeBuffer,
    pub locals_count: usize,
}

impl JitFunction {
    /// Call the JIT-compiled function with a locals array.
    /// `locals` must have at least `self.locals_count` elements.
    pub fn call(&self, locals: &mut [i64]) -> i64 {
        assert!(locals.len() >= self.locals_count,
            "jit: locals array too small: {} < {}", locals.len(), self.locals_count);
        let f = self.buf.entry_fn();
        unsafe { f(locals.as_mut_ptr()) }
    }
}

/// Compile a single WASM function body to native x86_64.
/// `func.body` contains raw WASM bytecode (expression body, no section header).
pub fn jit_compile(func: &WasmFunc, locals_count: usize) -> Result<JitFunction, WasmError> {
    let mut buf = CodeBuffer::new(4096)?;
    {
        let mut emit = X64Emitter::new(&mut buf, locals_count);
        emit.prologue()?;

        let code = &func.body;
        let mut ip = 0usize;

        while ip < code.len() {
            let op = code[ip];
            ip += 1;
            match op {
                0x00 => emit.emit_unreachable()?,          // unreachable
                0x01 => emit.emit_nop()?,                  // nop
                0x0F => emit.emit_return()?,               // return
                0x1A => emit.emit_drop()?,                 // drop

                // local.get u32
                0x20 => {
                    let (idx, n) = leb128_u32(&code[ip..])?;
                    ip += n;
                    emit.emit_local_get(idx)?;
                }
                // local.set u32
                0x21 => {
                    let (idx, n) = leb128_u32(&code[ip..])?;
                    ip += n;
                    emit.emit_local_set(idx)?;
                }
                // local.tee u32
                0x22 => {
                    let (idx, n) = leb128_u32(&code[ip..])?;
                    ip += n;
                    emit.emit_local_tee(idx)?;
                }

                // i32.const s32
                0x41 => {
                    let (v, n) = leb128_i32(&code[ip..])?;
                    ip += n;
                    emit.emit_i32_const(v)?;
                }
                // i64.const s64
                0x42 => {
                    let (v, n) = leb128_i64(&code[ip..])?;
                    ip += n;
                    emit.emit_i64_const(v)?;
                }

                0x6A => emit.emit_add()?,   // i32.add
                0x6B => emit.emit_sub()?,   // i32.sub
                0x6C => emit.emit_mul()?,   // i32.mul
                0x71 => emit.emit_and()?,   // i32.and
                0x72 => emit.emit_or()?,    // i32.or
                0x7C => emit.emit_add()?,   // i64.add
                0x7D => emit.emit_sub()?,   // i64.sub
                0x7E => emit.emit_mul()?,   // i64.mul
                0x83 => emit.emit_and()?,   // i64.and
                0x84 => emit.emit_or()?,    // i64.or

                0x0B => {
                    // end — function end marker
                    break;
                }

                _ => {
                    return Err(WasmError::Trap(
                        format!("jit: unsupported opcode 0x{:02X} at ip={}", op, ip - 1)
                    ));
                }
            }
        }

        emit.epilogue()?;
    }
    Ok(JitFunction { buf, locals_count })
}

// ── LEB128 decoders ───────────────────────────────────────────────────────────

pub fn leb128_u32(bytes: &[u8]) -> Result<(u32, usize), WasmError> {
    let mut val = 0u32;
    let mut shift = 0u32;
    let mut i = 0;
    loop {
        if i >= bytes.len() {
            return Err(WasmError::ParseError("leb128_u32: unexpected end".into()));
        }
        let b = bytes[i]; i += 1;
        val |= ((b & 0x7F) as u32) << shift;
        shift += 7;
        if b & 0x80 == 0 { break; }
        if shift >= 35 {
            return Err(WasmError::ParseError("leb128_u32: overflow".into()));
        }
    }
    Ok((val, i))
}

pub fn leb128_i32(bytes: &[u8]) -> Result<(i32, usize), WasmError> {
    let mut val = 0i32;
    let mut shift = 0u32;
    let mut i = 0;
    loop {
        if i >= bytes.len() {
            return Err(WasmError::ParseError("leb128_i32: unexpected end".into()));
        }
        let b = bytes[i]; i += 1;
        val |= ((b & 0x7F) as i32) << shift;
        shift += 7;
        if b & 0x80 == 0 {
            if shift < 32 && (b & 0x40) != 0 {
                val |= !0i32 << shift;  // sign extend
            }
            break;
        }
        if shift >= 35 {
            return Err(WasmError::ParseError("leb128_i32: overflow".into()));
        }
    }
    Ok((val, i))
}

pub fn leb128_i64(bytes: &[u8]) -> Result<(i64, usize), WasmError> {
    let mut val = 0i64;
    let mut shift = 0u32;
    let mut i = 0;
    loop {
        if i >= bytes.len() {
            return Err(WasmError::ParseError("leb128_i64: unexpected end".into()));
        }
        let b = bytes[i]; i += 1;
        val |= ((b & 0x7F) as i64) << shift;
        shift += 7;
        if b & 0x80 == 0 {
            if shift < 64 && (b & 0x40) != 0 {
                val |= !0i64 << shift;
            }
            break;
        }
        if shift >= 70 {
            return Err(WasmError::ParseError("leb128_i64: overflow".into()));
        }
    }
    Ok((val, i))
}
