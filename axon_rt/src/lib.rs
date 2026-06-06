#![cfg_attr(feature = "standalone", no_std)]
#![allow(clippy::module_name_repetitions)]

#[repr(C)]
pub struct AxonIterResult {
    pub tag: i8,
    pub value: i64,
}

#[repr(C)]
pub struct RangeIterator {
    pub current: i64,
    pub end: i64,
}

static mut RANGE_BUF: RangeIterator = RangeIterator { current: 0, end: 0 };

#[no_mangle]
pub extern "C" fn axon_range_new(start: i64, end: i64) -> *mut RangeIterator {
    unsafe {
        // SAFETY: Single-threaded MVP; sole writer to static RANGE_BUF.
        let ptr = core::ptr::addr_of_mut!(RANGE_BUF);
        (*ptr).current = start;
        (*ptr).end = end;
        ptr
    }
}

/// # Safety
/// `iter` must be a non-null pointer returned by `axon_range_new`.
// P13-M2-UNSAFE-FIX
#[no_mangle]
pub unsafe extern "C" fn axon_iter_next(iter: *mut RangeIterator) -> AxonIterResult {
    unsafe {
        // SAFETY: iter is a non-null pointer returned by axon_range_new.
        let r = &mut *iter;
        if r.current < r.end {
            let val = r.current;
            r.current += 1;
            AxonIterResult { tag: 1, value: val }
        } else {
            AxonIterResult { tag: 0, value: 0 }
        }
    }
}

/// # Safety
/// `iter` must be a non-null pointer returned by `axon_range_new`.
#[no_mangle]
pub unsafe extern "C" fn axon_iter_drop(iter: *mut RangeIterator) {
    unsafe {
        // SAFETY: iter is a non-null pointer returned by axon_range_new.
        let r = &mut *iter;
        r.current = r.end;
    }
}

/// Write `count` bytes from `buf` to file descriptor `fd` via Linux syscall.
///
/// # Safety
/// `buf` must be valid for `count` bytes. `fd` must be a valid open descriptor.
unsafe fn sys_write(fd: i32, buf: *const u8, count: usize) -> isize {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") 1u64,
        in("rdi") fd as u64,
        in("rsi") buf as u64,
        in("rdx") count as u64,
        out("rcx") _,
        out("r11") _,
        lateout("rax") ret,
    );
    ret as isize
}

#[no_mangle]
pub extern "C" fn axon_print_int(val: i64) {
    let mut buf = [0u8; 21];
    let mut pos = 0usize;
    let mut v = val;

    if v < 0 {
        buf[pos] = b'-';
        pos += 1;
        if v == i64::MIN {
            let min_str = b"-9223372036854775808\n";
            unsafe {
                // SAFETY: min_str is a valid static byte slice; fd 1 is stdout.
                sys_write(1, min_str.as_ptr(), min_str.len());
            }
            return;
        }
        v = -v;
    }

    if v == 0 {
        buf[pos] = b'0';
        pos += 1;
    } else {
        let mut tmp = [0u8; 20];
        let mut tpos = 0usize;
        while v > 0 {
            tmp[tpos] = b'0' + (v % 10) as u8;
            tpos += 1;
            v /= 10;
        }
        for i in (0..tpos).rev() {
            buf[pos] = tmp[i];
            pos += 1;
        }
    }
    buf[pos] = b'\n';
    pos += 1;

    unsafe {
        // SAFETY: buf is valid for pos bytes; fd 1 is stdout.
        sys_write(1, buf.as_ptr(), pos);
    }
}

/// Minimal panic handler — active in no_std standalone builds only.
/// Suppressed when std is available (std provides its own panic_impl).
#[cfg(all(not(test), feature = "standalone"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
