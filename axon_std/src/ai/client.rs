//! Sovereign Ollama HTTP client — TcpStream, no external HTTP crate.
//!
//! AXON owns its Ollama connection. No reqwest, no hyper.
//! Pure std::net::TcpStream with manual HTTP/1.1.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use super::{AiError, AiResult};

/// Ollama default endpoint.
pub const OLLAMA_HOST: &str = "127.0.0.1";
pub const OLLAMA_PORT: u16  = 11434;
pub const CONNECT_TIMEOUT_MS: u64 = 2_000;
pub const READ_TIMEOUT_MS:    u64 = 120_000;

/// Send a POST request to Ollama and return the response body.
pub fn ollama_post(path: &str, body: &str) -> AiResult<String> {
    let addr = format!("{}:{}", OLLAMA_HOST, OLLAMA_PORT);

    let mut stream = TcpStream::connect(&addr).map_err(|e| {
        AiError::OllamaUnavailable(format!("connect failed: {e}"))
    })?;

    stream.set_read_timeout(Some(Duration::from_millis(READ_TIMEOUT_MS)))
        .map_err(|e| AiError::IoError(e.to_string()))?;
    stream.set_write_timeout(Some(Duration::from_millis(CONNECT_TIMEOUT_MS)))
        .map_err(|e| AiError::IoError(e.to_string()))?;

    let request = format!(
        "POST {path} HTTP/1.1\r\n\
         Host: {OLLAMA_HOST}:{OLLAMA_PORT}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        body.len()
    );

    stream.write_all(request.as_bytes())
        .map_err(|e| AiError::IoError(format!("write failed: {e}")))?;

    let mut response = String::new();
    stream.read_to_string(&mut response)
        .map_err(|e| AiError::IoError(format!("read failed: {e}")))?;

    extract_body(response)
}

/// Send a GET request to Ollama and return the response body.
pub fn ollama_get(path: &str) -> AiResult<String> {
    let addr = format!("{}:{}", OLLAMA_HOST, OLLAMA_PORT);
    let mut stream = TcpStream::connect(&addr).map_err(|e| {
        AiError::OllamaUnavailable(format!("connect failed: {e}"))
    })?;
    stream.set_read_timeout(Some(Duration::from_millis(READ_TIMEOUT_MS)))
        .map_err(|e| AiError::IoError(e.to_string()))?;

    let request = format!(
        "GET {path} HTTP/1.1\r\n\
         Host: {OLLAMA_HOST}:{OLLAMA_PORT}\r\n\
         Connection: close\r\n\
         \r\n"
    );
    stream.write_all(request.as_bytes())
        .map_err(|e| AiError::IoError(format!("write failed: {e}")))?;

    let mut response = String::new();
    stream.read_to_string(&mut response)
        .map_err(|e| AiError::IoError(format!("read failed: {e}")))?;

    extract_body(response)
}

/// Extract the HTTP response body (after the blank line).
fn extract_body(response: String) -> AiResult<String> {
    // Handle HTTP/1.1 chunked encoding and plain responses
    if let Some(pos) = response.find("\r\n\r\n") {
        let body = &response[pos + 4..];
        // Strip chunked encoding markers if present
        let cleaned = strip_chunked(body);
        Ok(cleaned.trim().to_string())
    } else {
        Err(AiError::MalformedResponse("no HTTP header/body separator".into()))
    }
}

/// Strip HTTP chunked transfer encoding markers.
fn strip_chunked(body: &str) -> String {
    // Chunked format: size_hex\r\ndata\r\n0\r\n\r\n
    // If no chunk markers, return as-is
    if !body.contains("\r\n") {
        return body.to_string();
    }
    let mut result = String::new();
    let mut lines = body.split("\r\n");
    while let Some(line) = lines.next() {
        if line.is_empty() { continue; }
        // If line is a hex number, it's a chunk size — skip it
        if u64::from_str_radix(line.trim(), 16).is_ok() { continue; }
        result.push_str(line);
    }
    result
}

/// Check if Ollama is reachable at localhost:11434.
pub fn ollama_available() -> bool {
    TcpStream::connect_timeout(
        &format!("{}:{}", OLLAMA_HOST, OLLAMA_PORT).parse().unwrap(),
        Duration::from_millis(500),
    ).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ollama_available_returns_bool() {
        // Just verify it doesn't panic — result depends on environment
        let _ = ollama_available();
    }

    #[test]
    fn extract_body_works() {
        let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"ok\":true}";
        let body = extract_body(response.to_string()).unwrap();
        assert_eq!(body, "{\"ok\":true}");
    }

    #[test]
    fn extract_body_missing_separator() {
        let r = extract_body("no separator here".to_string());
        assert!(r.is_err());
    }
}
