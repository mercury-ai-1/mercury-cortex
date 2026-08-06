#![cfg(unix)]

//! Graceful `daemon stop` semantics: SIGTERM-based stop, PID-file removal on
//! Drop, and a clean (success) exit. Windows termination is a hard
//! `TerminateProcess` with no graceful analogue, so these assertions are
//! unix-only.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use fs2::FileExt;
use mercury_cortex_core::db;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn daemon_stop_terminates_serve_and_releases_db_lock() -> anyhow::Result<()> {
    let tmp = tempfile::TempDir::new()?;
    let home = tmp.path();

    // Hermetic data dir: everything (child + our own db::connect) lives under
    // $HOME/.mercury/cortex so the real ~/.mercury is never touched.
    unsafe { std::env::set_var("HOME", home) };

    let bin = env!("CARGO_BIN_EXE_mercury-cortex");

    // Spawn the daemon with stdin closed (it does not read stdin).
    let mut child = Command::new(bin)
        .args(["daemon"])
        .env("HOME", home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // Wait for the PID file, proving the daemon fully started.
    let pidfile = home.join(".mercury/cortex/daemon.pid");
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    while !pidfile.exists() && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        pidfile.exists(),
        "daemon did not write its PID file in time"
    );

    let mut pid_contents = String::new();
    std::fs::File::open(&pidfile)?.read_to_string(&mut pid_contents)?;
    let daemon_pid: u32 = pid_contents.trim().parse()?;
    assert_eq!(
        daemon_pid,
        child.id(),
        "PID file must match the spawned child"
    );

    // Run the real CLI stop path.
    let output = Command::new(bin)
        .args(["daemon", "stop"])
        .env("HOME", home)
        .output()?;
    assert!(
        output.status.success(),
        "daemon stop failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    // 1. Process exits. Bound the wait so an identity mismatch (stop returns
    //    exit 0 without killing) fails cleanly instead of hanging.
    let status = tokio::time::timeout(STARTUP_TIMEOUT, async { child.wait() })
        .await
        .map_err(|_| anyhow::anyhow!("daemon did not exit within {STARTUP_TIMEOUT:?}"))??;
    assert!(status.success(), "daemon should exit cleanly");

    // 2. PID file removed.
    assert!(!pidfile.exists(), "PID file must be removed after stop");

    // 3. DB lock released. SurrealKV never deletes the LOCK file (its PID is
    //    informational; the lock lives in the OS-level flock), so probe the
    //    flock directly. Scoped so our probe lock is released before connect.
    let lock = home.join(".mercury/cortex/mercury_cortex_global_knowledge.db/LOCK");
    {
        let probe = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&lock)?;
        let lock_free = probe.try_lock_exclusive().is_ok();
        assert!(
            lock_free,
            "LOCK flock must be free after stop: server drop chain leaked the lock"
        );
    }

    // 4. DB still usable: connecting must succeed immediately.
    let (_path, conn) = db::connect().await?;
    let _rows: Vec<serde_json::Value> = conn
        .query("SELECT count() AS n FROM users GROUP ALL")
        .await?
        .take(0)?;

    Ok(())
}
