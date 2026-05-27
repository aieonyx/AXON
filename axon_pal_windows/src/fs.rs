#![allow(clippy::all, dead_code, unused_imports, unused_variables)]
use axon_core::prelude::*;
use axon_pal::{traits::PalFs, types::{AxonPath, FileStat, OpenFlags, RawFd}};
use crate::WindowsPal;

fn path_wide(path: &AxonPath) -> Vec<u16> {
    path.as_str().encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
extern "system" {
    fn CreateFileW(name: *const u16, access: u32, share: u32, sec: *mut (),
                   disposition: u32, flags: u32, tmpl: isize) -> isize;
    fn CloseHandle(h: isize) -> i32;
    fn GetFileAttributesExW(name: *const u16, level: i32, info: *mut FileAttrData) -> i32;
    fn CreateDirectoryW(name: *const u16, sec: *mut ()) -> i32;
    fn DeleteFileW(name: *const u16) -> i32;
    fn RemoveDirectoryW(name: *const u16) -> i32;
    fn MoveFileW(from: *const u16, to: *const u16) -> i32;
}

#[cfg(windows)]
#[repr(C)]
struct FileAttrData { attrs: u32, created_hi: u32, created_lo: u32,
    access_hi: u32, access_lo: u32, write_hi: u32, write_lo: u32,
    size_hi: u32, size_lo: u32 }

#[cfg(windows)]
const GENERIC_READ:    u32 = 0x80000000;
#[cfg(windows)]
const GENERIC_WRITE:   u32 = 0x40000000;
#[cfg(windows)]
const FILE_SHARE_READ: u32 = 0x00000001;
#[cfg(windows)]
const CREATE_ALWAYS:   u32 = 2;
#[cfg(windows)]
const OPEN_EXISTING:   u32 = 3;
#[cfg(windows)]
const OPEN_ALWAYS:     u32 = 4;
#[cfg(windows)]
const FILE_ATTRIBUTE_NORMAL:    u32 = 0x80;
#[cfg(windows)]
const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
#[cfg(windows)]
const INVALID_HANDLE_VALUE: isize = -1isize;

impl PalFs for WindowsPal {
    fn open(path: &AxonPath, flags: OpenFlags) -> AxonResult<RawFd> {
        #[cfg(windows)]
        {
            let access = if flags.contains(OpenFlags::WRITE) { GENERIC_READ | GENERIC_WRITE } else { GENERIC_READ };
            let disposition = if flags.contains(OpenFlags::CREATE) {
                if flags.contains(OpenFlags::TRUNCATE) { CREATE_ALWAYS } else { OPEN_ALWAYS }
            } else { OPEN_EXISTING };
            let wide = path_wide(path);
            let h = unsafe { CreateFileW(wide.as_ptr(), access, FILE_SHARE_READ,
                core::ptr::null_mut(), disposition, FILE_ATTRIBUTE_NORMAL, 0) };
            if h == INVALID_HANDLE_VALUE { AxonResult::Err(AxonError::io("CreateFileW failed")) }
            else { AxonResult::Ok(RawFd(h as u32)) }
        }
        #[cfg(not(windows))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows::open: not on Windows"))
    }
    fn close(fd: RawFd) -> AxonResult<()> {
        #[cfg(windows)]
        {
            let ok = unsafe { CloseHandle(fd.0 as isize) };
            if ok == 0 { AxonResult::Err(AxonError::io("CloseHandle failed")) } else { AxonResult::Ok(()) }
        }
        #[cfg(not(windows))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows::close: not on Windows"))
    }
    fn stat(path: &AxonPath) -> AxonResult<FileStat> {
        #[cfg(windows)]
        {
            let wide = path_wide(path);
            let mut data = FileAttrData { attrs:0, created_hi:0, created_lo:0,
                access_hi:0, access_lo:0, write_hi:0, write_lo:0, size_hi:0, size_lo:0 };
            let ok = unsafe { GetFileAttributesExW(wide.as_ptr(), 0, &mut data) };
            if ok == 0 { return AxonResult::Err(AxonError::not_found("file not found")); }
            let size = ((data.size_hi as u64) << 32) | data.size_lo as u64;
            let is_dir = (data.attrs & FILE_ATTRIBUTE_DIRECTORY) != 0;
            AxonResult::Ok(FileStat { size, is_dir, is_file: !is_dir, is_symlink: false, mode: 0 })
        }
        #[cfg(not(windows))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows::stat: not on Windows"))
    }
    fn mkdir(path: &AxonPath, _mode: u32) -> AxonResult<()> {
        #[cfg(windows)]
        {
            let wide = path_wide(path);
            let ok = unsafe { CreateDirectoryW(wide.as_ptr(), core::ptr::null_mut()) };
            if ok == 0 { AxonResult::Err(AxonError::io("CreateDirectoryW failed")) } else { AxonResult::Ok(()) }
        }
        #[cfg(not(windows))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows::mkdir: not on Windows"))
    }
    fn remove(path: &AxonPath) -> AxonResult<()> {
        #[cfg(windows)]
        {
            let wide = path_wide(path);
            if unsafe { DeleteFileW(wide.as_ptr()) } != 0 { return AxonResult::Ok(()); }
            if unsafe { RemoveDirectoryW(wide.as_ptr()) } != 0 { return AxonResult::Ok(()); }
            AxonResult::Err(AxonError::io("remove failed"))
        }
        #[cfg(not(windows))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows::remove: not on Windows"))
    }
    fn rename(from: &AxonPath, to: &AxonPath) -> AxonResult<()> {
        #[cfg(windows)]
        {
            let wf = path_wide(from); let wt = path_wide(to);
            let ok = unsafe { MoveFileW(wf.as_ptr(), wt.as_ptr()) };
            if ok == 0 { AxonResult::Err(AxonError::io("MoveFileW failed")) } else { AxonResult::Ok(()) }
        }
        #[cfg(not(windows))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows::rename: not on Windows"))
    }
}
