// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P66 — AWP FFI: C-ABI exports for Onyxia and BASTION integration
// These symbols are callable from Onyxia's Rust FFI bridge (axon_awp_ffi crate)

use crate::parser::{parse, is_awp};

/// Check if a URI is an AWP address.
/// Returns 1 if valid awp://, 0 otherwise.
/// # Safety: uri_ptr must be a valid null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn awp_is_awp(uri_ptr: *const u8, uri_len: usize) -> i32 {
    if uri_ptr.is_null() { return 0; }
    let bytes = std::slice::from_raw_parts(uri_ptr, uri_len);
    match std::str::from_utf8(bytes) {
        Ok(s) => if is_awp(s) { 1 } else { 0 },
        Err(_) => 0,
    }
}

/// Parse and validate an AWP URI.
/// Returns 1 if valid, 0 if invalid.
/// Writes null-terminated error message to err_buf (max err_len bytes) on failure.
/// # Safety: pointers must be valid for their respective lengths.
#[no_mangle]
pub unsafe extern "C" fn awp_validate(
    uri_ptr: *const u8, uri_len: usize,
    err_buf: *mut u8, err_len: usize,
) -> i32 {
    if uri_ptr.is_null() { return 0; }
    let bytes = std::slice::from_raw_parts(uri_ptr, uri_len);
    let uri = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => {
            write_err(err_buf, err_len, "invalid UTF-8");
            return 0;
        }
    };
    match parse(uri) {
        Ok(_) => 1,
        Err(e) => {
            write_err(err_buf, err_len, &e.to_string());
            0
        }
    }
}

/// Extract the category from an AWP URI into out_buf.
/// Returns number of bytes written, or 0 on error.
/// # Safety: pointers must be valid for their respective lengths.
#[no_mangle]
pub unsafe extern "C" fn awp_category(
    uri_ptr: *const u8, uri_len: usize,
    out_buf: *mut u8, out_len: usize,
) -> usize {
    if uri_ptr.is_null() || out_buf.is_null() { return 0; }
    let bytes = std::slice::from_raw_parts(uri_ptr, uri_len);
    let uri = match std::str::from_utf8(bytes) { Ok(s) => s, Err(_) => return 0 };
    match parse(uri) {
        Ok(addr) => {
            let cat = addr.category.as_bytes();
            let n = cat.len().min(out_len);
            std::ptr::copy_nonoverlapping(cat.as_ptr(), out_buf, n);
            n
        }
        Err(_) => 0,
    }
}

/// Extract the node name from an AWP URI into out_buf.
/// Returns number of bytes written, or 0 on error.
/// # Safety: pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn awp_name(
    uri_ptr: *const u8, uri_len: usize,
    out_buf: *mut u8, out_len: usize,
) -> usize {
    if uri_ptr.is_null() || out_buf.is_null() { return 0; }
    let bytes = std::slice::from_raw_parts(uri_ptr, uri_len);
    let uri = match std::str::from_utf8(bytes) { Ok(s) => s, Err(_) => return 0 };
    match parse(uri) {
        Ok(addr) => {
            let name = addr.name.as_bytes();
            let n = name.len().min(out_len);
            std::ptr::copy_nonoverlapping(name.as_ptr(), out_buf, n);
            n
        }
        Err(_) => 0,
    }
}

// Internal helper: write error string to C buffer
unsafe fn write_err(buf: *mut u8, len: usize, msg: &str) {
    if buf.is_null() || len == 0 { return; }
    let bytes = msg.as_bytes();
    let n = bytes.len().min(len - 1);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, n);
    *buf.add(n) = 0; // null terminate
}
