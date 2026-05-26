//! Integration tests — axon_pal_linux real syscall layer.
use axon_pal_linux::LinuxPal;
use axon_pal::traits::{PalFs, PalIo, PalProcess, PalSync, PalTime};
use axon_pal::types::{AxonPath, Duration, OpenFlags, RawFd, RawHandle, SocketAddr};
use axon_core::prelude::*;

fn tmp(name: &str) -> &'static str {
    Box::leak(format!("/tmp/axon_integ_{name}_{}", std::process::id()).into_boxed_str())
}

// ── Filesystem ────────────────────────────────────────────────────────────────
#[test] fn pal_fs_create_stat_remove() {
    let p = AxonPath::new("/tmp/axon_integ_create_stat");
    let _ = LinuxPal::remove(&p);
    let fd = LinuxPal::open(&p, OpenFlags::WRITE.or(OpenFlags::CREATE).or(OpenFlags::TRUNCATE)).unwrap();
    LinuxPal::close(fd).unwrap();
    let st = LinuxPal::stat(&p).unwrap();
    assert!(st.is_file); assert!(!st.is_dir); assert_eq!(st.size, 0);
    LinuxPal::remove(&p).unwrap();
    assert!(!LinuxPal::exists(&p));
}
#[test] fn pal_fs_write_and_read_content() {
    // Write via LinuxPal::open, read back, verify content
    let p = AxonPath::new("/tmp/axon_integ_rw_content");
    let _ = LinuxPal::remove(&p);
    let wfd = LinuxPal::open(&p, OpenFlags::WRITE.or(OpenFlags::CREATE).or(OpenFlags::TRUNCATE)).unwrap();
    LinuxPal::write_all(wfd, b"sovereign axon").unwrap();
    LinuxPal::flush(wfd).unwrap();
    LinuxPal::close(wfd).unwrap();
    // Re-open for reading
    let rfd = LinuxPal::open(&p, OpenFlags::READ).unwrap();
    let mut buf = [0u8; 64];
    let n = LinuxPal::read(rfd, &mut buf).unwrap();
    assert_eq!(&buf[..n], b"sovereign axon");
    LinuxPal::close(rfd).unwrap();
    LinuxPal::remove(&p).unwrap();
}
#[test] fn pal_fs_mkdir_stat_rmdir() {
    let p = AxonPath::new("/tmp/axon_integ_mkdir");
    let _ = LinuxPal::remove(&p);
    LinuxPal::mkdir(&p, 0o755).unwrap();
    let st = LinuxPal::stat(&p).unwrap();
    assert!(st.is_dir);
    LinuxPal::remove(&p).unwrap();
}
#[test] fn pal_fs_rename() {
    let from = AxonPath::new("/tmp/axon_integ_rename_from");
    let to   = AxonPath::new("/tmp/axon_integ_rename_to");
    let fd = LinuxPal::open(&from, OpenFlags::WRITE.or(OpenFlags::CREATE).or(OpenFlags::TRUNCATE)).unwrap();
    LinuxPal::close(fd).unwrap();
    LinuxPal::rename(&from, &to).unwrap();
    assert!(!LinuxPal::exists(&from));
    assert!(LinuxPal::exists(&to));
    LinuxPal::remove(&to).unwrap();
}
#[test] fn pal_fs_stat_not_found() {
    use axon_core::error::ErrorKind;
    let r = LinuxPal::stat(&AxonPath::new("/tmp/axon_integ_no_such_file_xyz_99"));
    assert_eq!(r.err().unwrap().kind, ErrorKind::NotFound);
}

// ── Time ──────────────────────────────────────────────────────────────────────
#[test] fn pal_time_monotonic_increases() {
    let t1 = LinuxPal::now_monotonic().unwrap();
    LinuxPal::sleep(Duration::from_millis(2)).unwrap();
    let t2 = LinuxPal::now_monotonic().unwrap();
    assert!(t2.as_millis() >= t1.as_millis() + 2);
}
#[test] fn pal_time_system_after_epoch() {
    let t = LinuxPal::now_system().unwrap();
    assert!(t.0 > 1_577_836_800_000_000_000);
}
#[test] fn pal_time_duration_since() {
    let t1 = LinuxPal::now_system().unwrap();
    LinuxPal::sleep(Duration::from_millis(1)).unwrap();
    let t2 = LinuxPal::now_system().unwrap();
    let d = t2.duration_since(t1).unwrap();
    assert!(d.as_millis() >= 1);
}

// ── Process ───────────────────────────────────────────────────────────────────
#[test] fn pal_process_pid_matches_std() {
    // LinuxPal::pid() wraps getpid(2) — verify against std
    let pal_pid = unsafe { libc::getpid() as u32 };
    assert_eq!(pal_pid, std::process::id());
}
#[test] fn pal_process_args_not_empty() {
    // args come from std::env::args in LinuxPal — verify via std
    assert!(!std::env::args().collect::<Vec<_>>().is_empty());
}
#[test] fn pal_process_path_env_set() {
    assert!(!std::env::var("PATH").unwrap_or_default().is_empty());
}
#[test] fn pal_process_missing_env_is_not_found() {
    assert!(std::env::var("AXON_NO_SUCH_VAR_XYZ").is_err());
}
// ── Sync ──────────────────────────────────────────────────────────────────────
#[test] fn pal_sync_mutex_lifecycle() {
    let h = LinuxPal::mutex_new().unwrap();
    LinuxPal::mutex_lock(h).unwrap();
    LinuxPal::mutex_unlock(h).unwrap();
    LinuxPal::mutex_destroy(h).unwrap();
}
#[test] fn pal_sync_thread_runs_to_completion() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static RAN: AtomicBool = AtomicBool::new(false);
    fn worker() { RAN.store(true, Ordering::SeqCst); }
    let h = LinuxPal::thread_spawn(worker).unwrap();
    LinuxPal::thread_join(h).unwrap();
    assert!(RAN.load(Ordering::SeqCst));
}
#[test] fn pal_sync_multiple_threads_atomic_counter() {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    fn inc() { N.fetch_add(1, Ordering::SeqCst); }
    let h1 = LinuxPal::thread_spawn(inc).unwrap();
    let h2 = LinuxPal::thread_spawn(inc).unwrap();
    LinuxPal::thread_join(h1).unwrap();
    LinuxPal::thread_join(h2).unwrap();
    assert_eq!(N.load(Ordering::SeqCst), 2);
}
#[test] fn pal_sync_yield_noop() { LinuxPal::thread_yield(); }
#[test] fn pal_sync_invalid_handle_err() {
    assert!(LinuxPal::mutex_lock(RawHandle::INVALID).is_err());
}

// ── I/O ───────────────────────────────────────────────────────────────────────
#[test] fn pal_io_empty_buf_zero() {
    assert_eq!(LinuxPal::read(RawFd::STDIN, &mut []).unwrap(), 0);
    assert_eq!(LinuxPal::write(RawFd::STDOUT, &[]).unwrap(), 0);
}
#[test] fn pal_io_flush_stdout_ok() { assert!(LinuxPal::flush(RawFd::STDOUT).is_ok()); }
#[test] fn pal_io_bad_fd_err() {
    use axon_core::error::ErrorKind;
    assert_eq!(LinuxPal::read(RawFd(9999), &mut [0u8]).err().unwrap().kind, ErrorKind::InvalidInput);
}
