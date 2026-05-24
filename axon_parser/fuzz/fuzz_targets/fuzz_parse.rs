#![no_main]
use libfuzzer_sys::fuzz_target;
use axon_parser::{parse, FileId};

fuzz_target!(|data: &[u8]| {
    // PARSER_INVARIANT: skip inputs > 1MB
    if data.len() > 1_048_576 {
        return;
    }
    let input = String::from_utf8_lossy(data);
    // parse() must NEVER panic on any input
    // ParseErrors are acceptable — panics are not
    let _result = parse(&input, FileId(0));
});
