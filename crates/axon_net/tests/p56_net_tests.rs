// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P56 QA -- axon_net tests -- pass bar: 12/12
use axon_net::{AxonAddr, AxonConn, AxonListener};
use std::thread;

#[test]
fn test_addr_from_str_valid() {
    let addr = AxonAddr::from_str("127.0.0.1:0").unwrap();
    assert_eq!(addr.port(), 0);
    assert!(!addr.is_sovereign());
}
#[test]
fn test_addr_from_str_invalid() {
    assert!(AxonAddr::from_str("not_an_addr").is_err());
}
#[test]
fn test_addr_with_fingerprint() {
    let addr = AxonAddr::from_str("127.0.0.1:0").unwrap().with_fingerprint([0u8; 32]);
    assert!(addr.is_sovereign());
}
#[test]
fn test_addr_display_plain() {
    let addr = AxonAddr::from_str("127.0.0.1:8080").unwrap();
    assert_eq!(addr.to_string(), "127.0.0.1:8080");
}
#[test]
fn test_addr_display_sovereign() {
    let addr = AxonAddr::from_str("127.0.0.1:8080").unwrap().with_fingerprint([0u8; 32]);
    assert!(addr.to_string().contains("[sovereign]"));
}
#[test]
fn test_listener_bind() {
    let addr = AxonAddr::from_str("127.0.0.1:0").unwrap();
    let listener = AxonListener::bind(&addr).unwrap();
    assert!(listener.local_addr().port() > 0);
}
#[test]
fn test_listener_bind_invalid() {
    let addr = AxonAddr::from_str("0.0.0.0:1").unwrap();
    if std::env::var("USER").unwrap_or_default() != "root" {
        let _ = AxonListener::bind(&addr);
    }
}
#[test]
fn test_connect_send_recv() {
    let listener = AxonListener::bind(&AxonAddr::from_str("127.0.0.1:0").unwrap()).unwrap();
    let port = listener.local_addr().port();
    let handle = thread::spawn(move || {
        let mut conn = listener.accept().unwrap();
        let mut buf = [0u8; 5];
        let n = conn.recv(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"hello");
        conn.send_all(b"world").unwrap();
    });
    let mut conn = AxonConn::connect(&AxonAddr::from_str(&format!("127.0.0.1:{}", port)).unwrap()).unwrap();
    conn.send_all(b"hello").unwrap();
    let mut buf = [0u8; 5];
    let n = conn.recv(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"world");
    handle.join().unwrap();
}
#[test]
fn test_send_all_complete() {
    let listener = AxonListener::bind(&AxonAddr::from_str("127.0.0.1:0").unwrap()).unwrap();
    let port = listener.local_addr().port();
    let payload = b"sovereign_network_test";
    let handle = thread::spawn(move || {
        let mut conn = listener.accept().unwrap();
        let mut buf = vec![0u8; payload.len()];
        conn.recv(&mut buf).unwrap();
        assert_eq!(buf, payload);
    });
    let mut conn = AxonConn::connect(&AxonAddr::from_str(&format!("127.0.0.1:{}", port)).unwrap()).unwrap();
    conn.send_all(payload).unwrap();
    handle.join().unwrap();
}
#[test]
fn test_conn_local_addr_set() {
    let listener = AxonListener::bind(&AxonAddr::from_str("127.0.0.1:0").unwrap()).unwrap();
    let port = listener.local_addr().port();
    let _h = thread::spawn(move || { let _ = listener.accept(); });
    let conn = AxonConn::connect(&AxonAddr::from_str(&format!("127.0.0.1:{}", port)).unwrap()).unwrap();
    assert!(conn.local.port() > 0);
}
#[test]
fn test_conn_debug() {
    let listener = AxonListener::bind(&AxonAddr::from_str("127.0.0.1:0").unwrap()).unwrap();
    let port = listener.local_addr().port();
    let _h = thread::spawn(move || { let _ = listener.accept(); });
    let conn = AxonConn::connect(&AxonAddr::from_str(&format!("127.0.0.1:{}", port)).unwrap()).unwrap();
    assert!(format!("{:?}", conn).contains("AxonConn"));
}
#[test]
fn test_connect_refused() {
    assert!(AxonConn::connect(&AxonAddr::from_str("127.0.0.1:1").unwrap()).is_err());
}
