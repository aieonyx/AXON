// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// x86_64 byte encoding primitives — mirrors x86.ax.
// Target: x86_64 / System V AMD64 ABI.
// All encodings verified against Intel SDM Vol. 2.
// REX.W prefix (0x48) used throughout for 64-bit operands.

/// push rbp — saves frame pointer
pub fn push_rbp() -> Vec<u8> { vec![0x55] }

/// pop rbp — restores frame pointer
pub fn pop_rbp() -> Vec<u8> { vec![0x5d] }

/// mov rbp, rsp — establish stack frame
pub fn mov_rbp_rsp() -> Vec<u8> { vec![0x48, 0x89, 0xe5] }

/// mov rsp, rbp — collapse stack frame
pub fn mov_rsp_rbp() -> Vec<u8> { vec![0x48, 0x89, 0xec] }

/// ret — return from function
pub fn ret_byte() -> Vec<u8> { vec![0xc3] }

/// mov rax, imm32 (sign-extended to 64-bit)
pub fn mov_rax_imm32(n: i32) -> Vec<u8> {
    let b = n.to_le_bytes();
    vec![0x48, 0xc7, 0xc0, b[0], b[1], b[2], b[3]]
}

/// sub rsp, imm8 — allocate stack space for locals
pub fn sub_rsp_imm8(n: u8) -> Vec<u8> { vec![0x48, 0x83, 0xec, n] }

/// push rax — save accumulator to stack
pub fn push_rax() -> Vec<u8> { vec![0x50] }

/// pop rbx — restore into secondary register
pub fn pop_rbx() -> Vec<u8> { vec![0x5b] }

/// pop rdi — first param register
pub fn pop_rdi() -> Vec<u8> { vec![0x5f] }

/// pop rsi — second param register
pub fn pop_rsi() -> Vec<u8> { vec![0x5e] }

/// pop rdx — third param register
pub fn pop_rdx() -> Vec<u8> { vec![0x5a] }

/// add rax, rbx → rax = rax + rbx
pub fn add_rax_rbx() -> Vec<u8> { vec![0x48, 0x01, 0xd8] }

/// sub rbx, rax → rbx = rbx - rax  (= LHS - RHS)
pub fn sub_rbx_rax() -> Vec<u8> { vec![0x48, 0x29, 0xc3] }

/// mov rax, rbx — move result to accumulator
pub fn mov_rax_rbx() -> Vec<u8> { vec![0x48, 0x89, 0xd8] }

/// imul rax, rbx → rax = rax * rbx
pub fn imul_rax_rbx() -> Vec<u8> { vec![0x48, 0x0f, 0xaf, 0xc3] }

/// and rax, rbx
pub fn and_rax_rbx() -> Vec<u8> { vec![0x48, 0x21, 0xd8] }

/// or rax, rbx
pub fn or_rax_rbx() -> Vec<u8> { vec![0x48, 0x09, 0xd8] }

/// cmp rbx, rax — sets flags from (rbx - rax) = (LHS - RHS)
pub fn cmp_rbx_rax() -> Vec<u8> { vec![0x48, 0x39, 0xc3] }

/// sete al — al = 1 if ZF (LHS == RHS)
pub fn sete_al()  -> Vec<u8> { vec![0x0f, 0x94, 0xc0] }
/// setne al
pub fn setne_al() -> Vec<u8> { vec![0x0f, 0x95, 0xc0] }
/// setl al — al = 1 if LHS < RHS signed
pub fn setl_al()  -> Vec<u8> { vec![0x0f, 0x9c, 0xc0] }
/// setle al
pub fn setle_al() -> Vec<u8> { vec![0x0f, 0x9e, 0xc0] }
/// setg al — al = 1 if LHS > RHS signed
pub fn setg_al()  -> Vec<u8> { vec![0x0f, 0x9f, 0xc0] }
/// setge al
pub fn setge_al() -> Vec<u8> { vec![0x0f, 0x9d, 0xc0] }

/// movzx rax, al — zero-extend boolean result to 64-bit
pub fn movzx_rax_al() -> Vec<u8> { vec![0x48, 0x0f, 0xb6, 0xc0] }

/// test rax, rax — set ZF if rax == 0 (for if-condition)
pub fn test_rax_rax() -> Vec<u8> { vec![0x48, 0x85, 0xc0] }

/// je rel32 — jump if ZF=1 (condition false → skip then-block)
pub fn je_rel32(rel: i32) -> Vec<u8> {
    let b = rel.to_le_bytes();
    vec![0x0f, 0x84, b[0], b[1], b[2], b[3]]
}

/// jmp rel32 — unconditional jump (skip else-block)
pub fn jmp_rel32(rel: i32) -> Vec<u8> {
    let b = rel.to_le_bytes();
    vec![0xe9, b[0], b[1], b[2], b[3]]
}

/// call rel32 — call function (relative to next instruction)
pub fn call_rel32(rel: i32) -> Vec<u8> {
    let b = rel.to_le_bytes();
    vec![0xe8, b[0], b[1], b[2], b[3]]
}

/// mov rax, [rbp - (slot+1)*8] — load local variable from stack slot
pub fn load_rax_rbp_slot(slot: usize) -> Vec<u8> {
    let disp = -((slot as i32 + 1) * 8) as i8;
    vec![0x48, 0x8b, 0x45, disp as u8]
}

/// mov [rbp - (slot+1)*8], rax — store local variable to stack slot
pub fn store_rax_rbp_slot(slot: usize) -> Vec<u8> {
    let disp = -((slot as i32 + 1) * 8) as i8;
    vec![0x48, 0x89, 0x45, disp as u8]
}

/// mov rax, <param_reg>
/// reg_id: rdi=7, rsi=6, rdx=2
/// Encoding: REX.W 89 (C0 | reg_id<<3)
pub fn mov_rax_param(reg_id: u8) -> Vec<u8> {
    vec![0x48, 0x89, 0xc0 | (reg_id << 3)]
}

/// neg rax — two's complement negate
pub fn neg_rax() -> Vec<u8> { vec![0x48, 0xf7, 0xd8] }

/// xor rax, rax — zero rax efficiently
pub fn xor_rax_rax() -> Vec<u8> { vec![0x48, 0x31, 0xc0] }
