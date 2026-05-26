use axon_core::prelude::*;
use axon_core::types::U32;
use axon_pal::traits::PalProcess;
use crate::WindowsPal;

impl PalProcess for WindowsPal {
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
    fn exit(code: U32) -> ! { std::process::exit(code as i32) }
    fn pid() -> AxonResult<U32> { AxonResult::Ok(std::process::id()) }
}
