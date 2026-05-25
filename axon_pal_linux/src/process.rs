use axon_core::prelude::*;
use axon_core::types::U32;
use axon_pal::traits::PalProcess;
use crate::LinuxPal;

impl PalProcess for LinuxPal {
    fn args() -> AxonResult<&'static [&'static str]> {
        use std::sync::OnceLock;
        static ARGS: OnceLock<Vec<&'static str>> = OnceLock::new();
        let args = ARGS.get_or_init(|| {
            std::env::args().map(|s| Box::leak(s.into_boxed_str()) as &'static str).collect()
        });
        AxonResult::Ok(args.as_slice())
    }
    fn env_var(key: &str) -> AxonResult<&'static str> {
        match std::env::var(key) {
            Ok(v)  => AxonResult::Ok(Box::leak(v.into_boxed_str())),
            Err(_) => AxonResult::Err(AxonError::not_found("environment variable not set")),
        }
    }
    fn exit(code: U32) -> ! { unsafe { libc::exit(code as libc::c_int) } }
    fn pid() -> AxonResult<U32> { AxonResult::Ok(unsafe { libc::getpid() } as U32) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axon_pal::traits::PalProcess;
    #[test] fn process_pid_is_positive()    { assert!(LinuxPal::pid().unwrap() > 0); }
    #[test] fn process_args_returns_slice() { assert!(!LinuxPal::args().unwrap().is_empty()); }
    #[test] fn process_env_var_path_exists(){ assert!(!LinuxPal::env_var("PATH").unwrap().is_empty()); }
    #[test] fn process_env_var_missing_returns_not_found() {
        use axon_core::error::ErrorKind;
        assert_eq!(LinuxPal::env_var("AXON_PAL_NO_SUCH_VAR").err().unwrap().kind, ErrorKind::NotFound);
    }
    #[test] fn process_pid_matches_std() { assert_eq!(LinuxPal::pid().unwrap(), std::process::id()); }
}
