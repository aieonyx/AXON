// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P63.1 — FFI bridge: C-ABI symbols called from .ax attention module

use std::alloc::{alloc, Layout};

#[no_mangle]
pub extern "C" fn ai_alloc_f32(len: i64) -> i64 {
    if len <= 0 { return 0; }
    let layout = Layout::array::<f32>(len as usize).unwrap();
    unsafe { alloc(layout) } as i64
}

#[no_mangle]
pub extern "C" fn ai_free_f32(ptr: i64) -> i64 {
    let _ = ptr;
    0
}

#[no_mangle]
pub extern "C" fn ai_set_f32(ptr: i64, idx: i64, val: f32) -> i64 {
    if ptr == 0 { return -1; }
    unsafe { *(ptr as *mut f32).add(idx as usize) = val; }
    0
}

#[no_mangle]
pub extern "C" fn ai_get_f32(ptr: i64, idx: i64) -> f32 {
    if ptr == 0 { return 0.0; }
    unsafe { *(ptr as *const f32).add(idx as usize) }
}

#[no_mangle]
pub extern "C" fn ai_dot(a_ptr: i64, b_ptr: i64, len: i64) -> f32 {
    if a_ptr == 0 || b_ptr == 0 || len <= 0 { return 0.0; }
    let mut acc = 0.0f32;
    unsafe {
        let a = a_ptr as *const f32;
        let b = b_ptr as *const f32;
        for i in 0..(len as usize) { acc += (*a.add(i)) * (*b.add(i)); }
    }
    acc
}

#[no_mangle]
pub extern "C" fn ai_softmax_inplace(ptr: i64, len: i64) -> i64 {
    if ptr == 0 || len <= 0 { return -1; }
    unsafe {
        let s = std::slice::from_raw_parts_mut(ptr as *mut f32, len as usize);
        let max = s.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let mut sum = 0.0f32;
        for v in s.iter_mut() { *v = (*v - max).exp(); sum += *v; }
        for v in s.iter_mut() { *v /= sum; }
    }
    0
}

#[no_mangle]
pub extern "C" fn ai_matmul_flat(
    a_ptr: i64, b_ptr: i64, c_ptr: i64,
    m: i64, n: i64, k: i64,
) -> i64 {
    if a_ptr == 0 || b_ptr == 0 || c_ptr == 0 { return -1; }
    unsafe {
        let a = a_ptr as *const f32;
        let b = b_ptr as *const f32;
        let c = c_ptr as *mut f32;
        for i in 0..(m as usize) {
            for j in 0..(n as usize) {
                let mut acc = 0.0f32;
                for p in 0..(k as usize) {
                    acc += (*a.add(i * k as usize + p)) * (*b.add(p * n as usize + j));
                }
                *c.add(i * n as usize + j) = acc;
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn ai_scale_inplace(ptr: i64, len: i64, scale: f32) -> i64 {
    if ptr == 0 || len <= 0 { return -1; }
    unsafe {
        let s = std::slice::from_raw_parts_mut(ptr as *mut f32, len as usize);
        for v in s.iter_mut() { *v *= scale; }
    }
    0
}

#[no_mangle]
pub extern "C" fn ai_add_inplace(dst_ptr: i64, src_ptr: i64, len: i64) -> i64 {
    if dst_ptr == 0 || src_ptr == 0 || len <= 0 { return -1; }
    unsafe {
        let dst = dst_ptr as *mut f32;
        let src = src_ptr as *const f32;
        for i in 0..(len as usize) { *dst.add(i) += *src.add(i); }
    }
    0
}

#[no_mangle]
pub extern "C" fn ai_relu_inplace(ptr: i64, len: i64) -> i64 {
    if ptr == 0 || len <= 0 { return -1; }
    unsafe {
        let s = std::slice::from_raw_parts_mut(ptr as *mut f32, len as usize);
        for v in s.iter_mut() { if *v < 0.0 { *v = 0.0; } }
    }
    0
}
