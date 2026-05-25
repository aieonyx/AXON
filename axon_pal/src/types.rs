use axon_core::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RawFd(pub u32);
impl RawFd {
    pub const INVALID: RawFd = RawFd(u32::MAX);
    pub const STDIN:   RawFd = RawFd(0);
    pub const STDOUT:  RawFd = RawFd(1);
    pub const STDERR:  RawFd = RawFd(2);
    pub const fn is_invalid(self) -> bool { self.0 == u32::MAX }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RawHandle(pub u64);
impl RawHandle {
    pub const INVALID: RawHandle = RawHandle(u64::MAX);
    pub const fn is_invalid(self) -> bool { self.0 == u64::MAX }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketAddr {
    V4 { ip: [u8; 4], port: u16 },
    V6 { ip: [u8; 16], port: u16 },
}
impl SocketAddr {
    pub const fn v4(a: u8, b: u8, c: u8, d: u8, port: u16) -> Self { Self::V4 { ip: [a,b,c,d], port } }
    pub const fn loopback(port: u16) -> Self { Self::v4(127,0,0,1,port) }
    pub const fn port(&self) -> u16 { match self { Self::V4{port,..}|Self::V6{port,..} => *port } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenFlags(pub u32);
impl OpenFlags {
    pub const READ:     OpenFlags = OpenFlags(0b0001);
    pub const WRITE:    OpenFlags = OpenFlags(0b0010);
    pub const RDWR:     OpenFlags = OpenFlags(0b0011);
    pub const CREATE:   OpenFlags = OpenFlags(0b0100);
    pub const TRUNCATE: OpenFlags = OpenFlags(0b1000);
    pub const APPEND:   OpenFlags = OpenFlags(0b1_0000);
    pub const fn or(self, other: OpenFlags) -> OpenFlags { OpenFlags(self.0 | other.0) }
    pub const fn contains(self, flag: OpenFlags) -> bool { (self.0 & flag.0) != 0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileStat { pub size: u64, pub is_dir: bool, pub is_file: bool, pub is_symlink: bool, pub mode: u32 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration { pub secs: u64, pub nanos: u32 }
impl Duration {
    pub const ZERO: Duration = Duration { secs: 0, nanos: 0 };
    pub const fn new(secs: u64, nanos: u32) -> Self {
        let extra = nanos / 1_000_000_000;
        Self { secs: secs + extra as u64, nanos: nanos % 1_000_000_000 }
    }
    pub const fn from_millis(ms: u64) -> Self { Self::new(ms/1_000, ((ms%1_000)*1_000_000) as u32) }
    pub const fn from_secs(s: u64) -> Self { Self::new(s, 0) }
    pub const fn as_millis(&self) -> u64 { self.secs*1_000 + self.nanos as u64/1_000_000 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemTime(pub u64);
impl SystemTime {
    pub const UNIX_EPOCH: SystemTime = SystemTime(0);
    pub fn duration_since(self, earlier: SystemTime) -> AxonResult<Duration> {
        if self.0 >= earlier.0 {
            let diff = self.0 - earlier.0;
            AxonResult::Ok(Duration::new(diff/1_000_000_000, (diff%1_000_000_000) as u32))
        } else {
            AxonResult::Err(AxonError::invalid_input("time is before earlier"))
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AxonPath(pub &'static str);
impl AxonPath {
    pub const fn new(s: &'static str) -> Self { Self(s) }
    pub const fn as_str(&self) -> &str { self.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn rawfd_constants() { assert_eq!(RawFd::STDIN.0,0); assert!(RawFd::INVALID.is_invalid()); }
    #[test] fn rawhandle_invalid() { assert!(RawHandle::INVALID.is_invalid()); assert!(!RawHandle(0).is_invalid()); }
    #[test] fn socket_addr_v4_port() { assert_eq!(SocketAddr::loopback(3000).port(), 3000); }
    #[test] fn open_flags_combine() {
        let f = OpenFlags::WRITE.or(OpenFlags::CREATE);
        assert!(f.contains(OpenFlags::WRITE)); assert!(!f.contains(OpenFlags::READ));
    }
    #[test] fn duration_from_millis() {
        let d = Duration::from_millis(1500); assert_eq!(d.secs,1); assert_eq!(d.nanos,500_000_000);
    }
    #[test] fn duration_ordering() { assert!(Duration::from_secs(2)>Duration::from_secs(1)); }
    #[test] fn system_time_duration_since() {
        let d = SystemTime(2_000_000_000).duration_since(SystemTime(1_000_000_000)).unwrap();
        assert_eq!(d.secs,1);
    }
    #[test] fn system_time_duration_since_err() {
        assert!(SystemTime(1_000_000_000).duration_since(SystemTime(2_000_000_000)).is_err());
    }
    #[test] fn axon_path_str() { assert_eq!(AxonPath::new("/etc/axon.conf").as_str(), "/etc/axon.conf"); }
}
