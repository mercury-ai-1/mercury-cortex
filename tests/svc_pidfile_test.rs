use std::fs;

use tempfile::TempDir;

use mercury_cortex::svc::{PidFile, PidFileGuard};

#[test]
fn pidfile_write_read_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let pf = PidFile::new(tmp.path(), "test");
    pf.write(4242).unwrap();
    assert_eq!(pf.read().unwrap(), Some(4242));
}

#[test]
fn pidfile_read_missing_returns_none() {
    let tmp = TempDir::new().unwrap();
    let pf = PidFile::new(tmp.path(), "test");
    assert_eq!(pf.read().unwrap(), None);
}

#[test]
fn pidfile_read_unparseable_returns_none() {
    let tmp = TempDir::new().unwrap();
    let pf = PidFile::new(tmp.path(), "test");
    pf.write(1).unwrap();
    fs::write(pf.path(), "not-a-number").unwrap();
    assert_eq!(pf.read().unwrap(), None);
}

#[test]
fn pidfile_remove_removes_file() {
    let tmp = TempDir::new().unwrap();
    let pf = PidFile::new(tmp.path(), "test");
    pf.write(1).unwrap();
    pf.remove().unwrap();
    assert!(!pf.path().exists());
}

#[test]
fn pidfile_raii_guard_cleans_up_on_drop() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("test.pid");
    {
        let _guard = PidFileGuard::acquire(tmp.path(), "test").unwrap();
        assert!(path.exists());
    }
    assert!(
        !path.exists(),
        "PID file must be removed when the guard is dropped"
    );
}

#[test]
fn pidfile_remove_missing_is_not_an_error() {
    let tmp = TempDir::new().unwrap();
    let pf = PidFile::new(tmp.path(), "test");
    pf.remove().unwrap();
}
