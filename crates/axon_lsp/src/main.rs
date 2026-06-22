// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P64 — axon_lsp entry point: stdin/stdout LSP server

use std::io::{BufReader, BufWriter, stdout};
use axon_lsp::handler::LspHandler;
use axon_lsp::transport::{read_message, write_response};

fn main() {
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let mut handler = LspHandler::new();

    loop {
        match read_message(&mut reader) {
            Ok(Some(req)) => {
                let is_exit = req.method == "exit";
                if let Some(resp) = handler.handle(&req) {
                    let _ = write_response(&mut writer, &resp);
                }
                if is_exit { break; }
                if handler.shutdown && req.method == "exit" { break; }
            }
            Ok(None) => break, // EOF
            Err(_e) => {
                // Invalid message — continue
                continue;
            }
        }
    }
}
