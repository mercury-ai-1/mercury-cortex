use std::io::Write;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use mercury_cortex_core::engine::McIgnore;

/// Writes `content` to a unique temp file and returns its path.
///
/// Uses a per-call unique name so tests can run in parallel threads without
/// racing on a shared `.mcignore` file (the test harness runs tests
/// concurrently; a shared path meant one test could `remove_file` the other's
/// file, yielding an empty pattern set and a spurious failure).
fn write_mcignore(content: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "mc_ignore_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    path
}

/// `*.ext` patterns must not infinite-loop when a path does not match.
///
/// Regression test: `wildcard_match` used to spin forever once its star
/// backtracking advanced past the end of the value, which made ignore
/// evaluation hang during `metadata/import`, when each staged path is
/// checked against `.mcignore`.
#[test]
fn wildcard_pattern_non_match_terminates() {
    let path = write_mcignore("*.tmp\n*.log\n*.swp\n");
    let mci = McIgnore::load(&path).unwrap();
    std::fs::remove_file(&path).ok();

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let start = Instant::now();
        let ignored = mci.is_ignored("lib/main.dart", false);
        let _ = tx.send((ignored, start.elapsed()));
    });

    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok((ignored, elapsed)) => {
            assert!(
                !ignored,
                "lib/main.dart should not be ignored by *.tmp/*.log/*.swp"
            );
            assert!(
                elapsed < Duration::from_secs(2),
                "wildcard_match took too long: {elapsed:?}"
            );
        }
        Err(_) => {
            panic!("is_ignored did not terminate within 2s — infinite loop in wildcard_match")
        }
    }
}

/// `*.ext` patterns must still match a path that does end with the ext.
#[test]
fn wildcard_pattern_match_still_works() {
    let path = write_mcignore("*.tmp\n");
    let mci = McIgnore::load(&path).unwrap();
    std::fs::remove_file(&path).ok();

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let start = Instant::now();
        let ignored = mci.is_ignored("foo.tmp", false);
        let _ = tx.send((ignored, start.elapsed()));
    });

    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok((ignored, elapsed)) => {
            assert!(ignored, "foo.tmp should be ignored by *.tmp");
            assert!(
                elapsed < Duration::from_secs(2),
                "wildcard match took too long: {elapsed:?}"
            );
        }
        Err(_) => {
            panic!("is_ignored did not terminate within 2s — infinite loop in wildcard_match")
        }
    }
}
