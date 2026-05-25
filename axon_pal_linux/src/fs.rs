use axon_core::prelude::*;
use axon_pal::{traits::PalFs, types::{AxonPath, FileStat, OpenFlags, RawFd}};
use crate::{LinuxPal, error::errno_to_axon_error, path::CPathBuf};

fn to_linux_flags(flags: OpenFlags) -> libc::c_int {
    let mut f = 0i32;
    let rdwr = flags.contains(OpenFlags::READ) && flags.contains(OpenFlags::WRITE);
    if rdwr { f |= libc::O_RDWR; } else if flags.contains(OpenFlags::WRITE) { f |= libc::O_WRONLY; } else { f |= libc::O_RDONLY; }
    if flags.contains(OpenFlags::CREATE)   { f |= libc::O_CREAT; }
    if flags.contains(OpenFlags::TRUNCATE) { f |= libc::O_TRUNC; }
    if flags.contains(OpenFlags::APPEND)   { f |= libc::O_APPEND; }
    f
}

impl PalFs for LinuxPal {
    fn open(path: &AxonPath, flags: OpenFlags) -> AxonResult<RawFd> {
        let cpath = axon_try!(CPathBuf::from_axon_path(path));
        let fd = unsafe { libc::open(cpath.as_ptr(), to_linux_flags(flags), 0o644u32) };
        if fd < 0 { AxonResult::Err(errno_to_axon_error()) } else { AxonResult::Ok(RawFd(fd as u32)) }
    }
    fn close(fd: RawFd) -> AxonResult<()> {
        let ret = unsafe { libc::close(fd.0 as libc::c_int) };
        if ret < 0 { AxonResult::Err(errno_to_axon_error()) } else { AxonResult::Ok(()) }
    }
    fn stat(path: &AxonPath) -> AxonResult<FileStat> {
        let cpath = axon_try!(CPathBuf::from_axon_path(path));
        let mut st: libc::stat = unsafe { core::mem::zeroed() };
        let ret = unsafe { libc::stat(cpath.as_ptr(), &mut st) };
        if ret < 0 { return AxonResult::Err(errno_to_axon_error()); }
        AxonResult::Ok(FileStat {
            size: st.st_size as u64,
            is_dir:  (st.st_mode & libc::S_IFMT) == libc::S_IFDIR,
            is_file: (st.st_mode & libc::S_IFMT) == libc::S_IFREG,
            is_symlink: false,
            mode: st.st_mode as u32,
        })
    }
    fn mkdir(path: &AxonPath, mode: u32) -> AxonResult<()> {
        let cpath = axon_try!(CPathBuf::from_axon_path(path));
        let ret = unsafe { libc::mkdir(cpath.as_ptr(), mode as libc::mode_t) };
        if ret < 0 { AxonResult::Err(errno_to_axon_error()) } else { AxonResult::Ok(()) }
    }
    fn remove(path: &AxonPath) -> AxonResult<()> {
        let cpath = axon_try!(CPathBuf::from_axon_path(path));
        let ret = unsafe { libc::unlink(cpath.as_ptr()) };
        if ret == 0 { return AxonResult::Ok(()); }
        let ret = unsafe { libc::rmdir(cpath.as_ptr()) };
        if ret < 0 { AxonResult::Err(errno_to_axon_error()) } else { AxonResult::Ok(()) }
    }
    fn rename(from: &AxonPath, to: &AxonPath) -> AxonResult<()> {
        let cf = axon_try!(CPathBuf::from_axon_path(from));
        let ct = axon_try!(CPathBuf::from_axon_path(to));
        let ret = unsafe { libc::rename(cf.as_ptr(), ct.as_ptr()) };
        if ret < 0 { AxonResult::Err(errno_to_axon_error()) } else { AxonResult::Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axon_pal::traits::PalFs;
    #[test] fn fs_open_write_close() {
        let p = AxonPath::new("/tmp/axon_pal_fs_open_test");
        let fd = LinuxPal::open(&p, OpenFlags::WRITE.or(OpenFlags::CREATE).or(OpenFlags::TRUNCATE)).unwrap();
        assert!(!fd.is_invalid()); LinuxPal::close(fd).unwrap(); LinuxPal::remove(&p).unwrap();
    }
    #[test] fn fs_stat_existing_file() {
        let p = AxonPath::new("/tmp/axon_pal_fs_stat_test");
        let fd = LinuxPal::open(&p, OpenFlags::WRITE.or(OpenFlags::CREATE).or(OpenFlags::TRUNCATE)).unwrap();
        LinuxPal::close(fd).unwrap();
        let st = LinuxPal::stat(&p).unwrap(); assert!(st.is_file); assert!(!st.is_dir);
        LinuxPal::remove(&p).unwrap();
    }
    #[test] fn fs_stat_missing_file_returns_not_found() {
        use axon_core::error::ErrorKind;
        let r = LinuxPal::stat(&AxonPath::new("/tmp/axon_pal_fs_no_such_file_xyz"));
        assert_eq!(r.err().unwrap().kind, ErrorKind::NotFound);
    }
    #[test] fn fs_exists_true_and_false() {
        let p = AxonPath::new("/tmp/axon_pal_fs_exists_test");
        let fd = LinuxPal::open(&p, OpenFlags::WRITE.or(OpenFlags::CREATE).or(OpenFlags::TRUNCATE)).unwrap();
        LinuxPal::close(fd).unwrap(); assert!(LinuxPal::exists(&p));
        LinuxPal::remove(&p).unwrap(); assert!(!LinuxPal::exists(&p));
    }
    #[test] fn fs_mkdir_and_remove() {
        let p = AxonPath::new("/tmp/axon_pal_fs_mkdir_test");
        let _ = LinuxPal::remove(&p);
        LinuxPal::mkdir(&p, 0o755).unwrap();
        assert!(LinuxPal::stat(&p).unwrap().is_dir);
        LinuxPal::remove(&p).unwrap();
    }
    #[test] fn fs_rename() {
        let from = AxonPath::new("/tmp/axon_pal_fs_rename_from");
        let to   = AxonPath::new("/tmp/axon_pal_fs_rename_to");
        let fd = LinuxPal::open(&from, OpenFlags::WRITE.or(OpenFlags::CREATE).or(OpenFlags::TRUNCATE)).unwrap();
        LinuxPal::close(fd).unwrap(); LinuxPal::rename(&from, &to).unwrap();
        assert!(!LinuxPal::exists(&from)); assert!(LinuxPal::exists(&to));
        LinuxPal::remove(&to).unwrap();
    }
    #[test] fn fs_stat_directory() { let st = LinuxPal::stat(&AxonPath::new("/tmp")).unwrap(); assert!(st.is_dir); }
}
