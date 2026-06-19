// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P45 QA — 9/9 required before P46 begins.

use axon_std_io::{
    create, flush, open, path_exists, path_is_dir, path_is_file,
    read_to_end, stderr_write, stdout_write, write_all, IoError,
};
use tempfile::tempdir;

#[test]
fn test_file_read() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("read.txt");
    std::fs::write(&path, b"sovereign").unwrap();
    let file = open(path.to_str().unwrap()).unwrap();
    assert_eq!(read_to_end(&file).unwrap(), b"sovereign");
}

#[test]
fn test_file_write() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("write.txt");
    let file = create(path.to_str().unwrap()).unwrap();
    write_all(&file, b"axon_std::io").unwrap();
    flush(&file).unwrap();
    drop(file);
    assert_eq!(std::fs::read(&path).unwrap(), b"axon_std::io");
}

#[test]
fn test_create_overwrite() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("overwrite.txt");
    std::fs::write(&path, b"old").unwrap();
    let file = create(path.to_str().unwrap()).unwrap();
    write_all(&file, b"new").unwrap();
    drop(file);
    assert_eq!(std::fs::read(&path).unwrap(), b"new");
}

#[test]
fn test_not_found() {
    assert!(matches!(
        open("/tmp/axon_p45_no_such_file_xyzzy"),
        Err(IoError::NotFound)
    ));
}

#[test]
fn test_permission_denied() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("locked.txt");
    std::fs::write(&path, b"locked").unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
    let result = open(path.to_str().unwrap());
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(matches!(result, Err(IoError::PermissionDenied)));
}

#[test]
fn test_stdin_stub() {
    let _fn: fn() -> _ = axon_std_io::stdin_read_line;
}

#[test]
fn test_stdout_write() {
    assert!(stdout_write(b"[P45 stdout ok]
").is_ok());
}

#[test]
fn test_stderr_write() {
    assert!(stderr_write(b"[P45 stderr ok]
").is_ok());
}

#[test]
fn test_path_utils() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("pathtest.txt");
    std::fs::write(&file_path, b"x").unwrap();
    assert!(path_exists(file_path.to_str().unwrap()));
    assert!(path_is_file(file_path.to_str().unwrap()));
    assert!(!path_is_dir(file_path.to_str().unwrap()));
    assert!(path_exists(dir.path().to_str().unwrap()));
    assert!(path_is_dir(dir.path().to_str().unwrap()));
    assert!(!path_is_file(dir.path().to_str().unwrap()));
    assert!(!path_exists("/tmp/axon_p45_no_path_xyzzy"));
}
