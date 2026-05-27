#![allow(clippy::all, dead_code, unused_imports, unused_variables)]
use axon_core::prelude::*;
use axon_pal::{traits::PalFs, types::{AxonPath, FileStat, OpenFlags, RawFd}};
use crate::{MacOsPal, error::last_os_axon_error};

// Returns AxonResult so axon_try! works correctly
fn path_cstr(path: &AxonPath) -> AxonResult<std::ffi::CString> {
    match std::ffi::CString::new(path.as_str()) {
        Ok(c)  => AxonResult::Ok(c),
        Err(_) => AxonResult::Err(AxonError::invalid_input("path contains null byte")),
    }
}

fn to_darwin_flags(flags: OpenFlags) -> libc::c_int {
    let mut f = 0i32;
    let rdwr = flags.contains(OpenFlags::READ) && flags.contains(OpenFlags::WRITE);
    if rdwr { f |= libc::O_RDWR; } else if flags.contains(OpenFlags::WRITE) { f |= libc::O_WRONLY; } else { f |= libc::O_RDONLY; }
    if flags.contains(OpenFlags::CREATE)   { f |= libc::O_CREAT; }
    if flags.contains(OpenFlags::TRUNCATE) { f |= libc::O_TRUNC; }
    if flags.contains(OpenFlags::APPEND)   { f |= libc::O_APPEND; }
    f
}

impl PalFs for MacOsPal {
    fn open(path: &AxonPath, flags: OpenFlags) -> AxonResult<RawFd> {
        #[cfg(target_os = "macos")]
        {
            let c = axon_try!(path_cstr(path));
            let fd = unsafe { libc::open(c.as_ptr(), to_darwin_flags(flags), 0o644u32) };
            if fd < 0 { AxonResult::Err(last_os_axon_error()) } else { AxonResult::Ok(RawFd(fd as u32)) }
        }
        #[cfg(not(target_os = "macos"))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos::open: not on macOS"))
    }
    fn close(fd: RawFd) -> AxonResult<()> {
        #[cfg(target_os = "macos")]
        {
            let ret = unsafe { libc::close(fd.0 as libc::c_int) };
            if ret < 0 { AxonResult::Err(last_os_axon_error()) } else { AxonResult::Ok(()) }
        }
        #[cfg(not(target_os = "macos"))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos::close: not on macOS"))
    }
    fn stat(path: &AxonPath) -> AxonResult<FileStat> {
        #[cfg(target_os = "macos")]
        {
            let c = axon_try!(path_cstr(path));
            let mut st: libc::stat = unsafe { core::mem::zeroed() };
            let ret = unsafe { libc::stat(c.as_ptr(), &mut st) };
            if ret < 0 { return AxonResult::Err(last_os_axon_error()); }
            AxonResult::Ok(FileStat {
                size: st.st_size as u64,
                is_dir:  (st.st_mode & libc::S_IFMT) == libc::S_IFDIR,
                is_file: (st.st_mode & libc::S_IFMT) == libc::S_IFREG,
                is_symlink: false, mode: st.st_mode as u32,
            })
        }
        #[cfg(not(target_os = "macos"))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos::stat: not on macOS"))
    }
    fn mkdir(path: &AxonPath, mode: u32) -> AxonResult<()> {
        #[cfg(target_os = "macos")]
        {
            let c = axon_try!(path_cstr(path));
            let ret = unsafe { libc::mkdir(c.as_ptr(), mode as libc::mode_t) };
            if ret < 0 { AxonResult::Err(last_os_axon_error()) } else { AxonResult::Ok(()) }
        }
        #[cfg(not(target_os = "macos"))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos::mkdir: not on macOS"))
    }
    fn remove(path: &AxonPath) -> AxonResult<()> {
        #[cfg(target_os = "macos")]
        {
            let c = axon_try!(path_cstr(path));
            if unsafe { libc::unlink(c.as_ptr()) } == 0 { return AxonResult::Ok(()); }
            let ret = unsafe { libc::rmdir(c.as_ptr()) };
            if ret < 0 { AxonResult::Err(last_os_axon_error()) } else { AxonResult::Ok(()) }
        }
        #[cfg(not(target_os = "macos"))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos::remove: not on macOS"))
    }
    fn rename(from: &AxonPath, to: &AxonPath) -> AxonResult<()> {
        #[cfg(target_os = "macos")]
        {
            let cf = axon_try!(path_cstr(from));
            let ct = axon_try!(path_cstr(to));
            let ret = unsafe { libc::rename(cf.as_ptr(), ct.as_ptr()) };
            if ret < 0 { AxonResult::Err(last_os_axon_error()) } else { AxonResult::Ok(()) }
        }
        #[cfg(not(target_os = "macos"))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos::rename: not on macOS"))
    }
}
