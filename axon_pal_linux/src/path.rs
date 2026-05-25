use axon_core::prelude::*;
use axon_pal::types::AxonPath;
pub(crate) const PATH_MAX: usize = 4096;
pub(crate) struct CPathBuf([u8; PATH_MAX]);
impl CPathBuf {
    pub(crate) fn from_axon_path(path: &AxonPath) -> AxonResult<Self> {
        let s = path.as_str().as_bytes();
        if s.len() >= PATH_MAX { return AxonResult::Err(AxonError::invalid_input("path exceeds PATH_MAX")); }
        let mut buf = [0u8; PATH_MAX];
        buf[..s.len()].copy_from_slice(s);
        AxonResult::Ok(CPathBuf(buf))
    }
    pub(crate) fn as_ptr(&self) -> *const libc::c_char { self.0.as_ptr() as *const libc::c_char }
}
