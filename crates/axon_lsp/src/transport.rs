// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P64 — LSP transport: Content-Length framed JSON-RPC over stdin/stdout

use std::io::{self, BufRead, Write};
use crate::types::{RpcRequest, RpcResponse};

/// Read one LSP message from reader.
/// Format: "Content-Length: N\r\n\r\n{json}"
pub fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<RpcRequest>> {
    let mut content_length = 0usize;

    // Read headers
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 { return Ok(None); } // EOF
        let line = line.trim_end_matches('\n').trim_end_matches('\r');
        if line.is_empty() { break; } // blank line = end of headers
        if let Some(rest) = line.strip_prefix("Content-Length: ") {
            content_length = rest.trim().parse().unwrap_or(0);
        }
    }

    if content_length == 0 { return Ok(None); }

    // Read body
    let mut body = vec![0u8; content_length];
    let mut pos = 0;
    while pos < content_length {
        let slice = reader.fill_buf()?;
        if slice.is_empty() { return Ok(None); }
        let to_copy = slice.len().min(content_length - pos);
        body[pos..pos + to_copy].copy_from_slice(&slice[..to_copy]);
        pos += to_copy;
        reader.consume(to_copy);
    }

    match serde_json::from_slice(&body) {
        Ok(req) => Ok(Some(req)),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData,
            format!("JSON parse error: {}", e))),
    }
}

/// Write one LSP response to writer with Content-Length framing.
pub fn write_message<W: Write>(writer: &mut W, resp: &serde_json::Value) -> io::Result<()> {
    let body = serde_json::to_vec(resp)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

pub fn write_response<W: Write>(writer: &mut W, resp: &RpcResponse) -> io::Result<()> {
    let val = serde_json::to_value(resp).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, e)
    })?;
    write_message(writer, &val)
}
