use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    let home = tempfile::tempdir().unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mercury-cortex"));
    cmd.args(args).env("HOME", home.path());
    // `dirs::home_dir()` reads USERPROFILE on Windows (not HOME), so point it
    // at the same hermetic dir for cross-platform consistency.
    #[cfg(windows)]
    cmd.env("USERPROFILE", home.path());
    cmd.output().expect("binary should run")
}

#[test]
fn failing_command_exits_nonzero() {
    // `db restore` with a nonexistent backup dir deterministically fails
    // before touching the (hermetic) data dir.
    let output = run(&["db", "restore", "/nonexistent/backup/dir"]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "a failing command must exit 1 (stderr: {})",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not exist"),
        "stderr should carry the error origin: {stderr}"
    );
}

#[test]
fn success_command_exits_zero() {
    let output = run(&["version"]);
    assert_eq!(output.status.code(), Some(0));
}
