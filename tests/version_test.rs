use mercury_cortex::commands::version;

#[test]
fn test_version_text_output() -> anyhow::Result<()> {
    let result = version::run(false);
    assert!(
        result.is_ok(),
        "version command should succeed in text mode"
    );
    Ok(())
}

#[test]
fn test_version_json_output() -> anyhow::Result<()> {
    let result = version::run(true);
    assert!(
        result.is_ok(),
        "version command should succeed in JSON mode"
    );
    Ok(())
}

#[test]
fn test_current_exe_is_valid() {
    let exe = std::env::current_exe().expect("should resolve current exe");
    assert!(exe.exists(), "executable should exist at its own path");
}

#[test]
fn test_data_dir_resolves() -> anyhow::Result<()> {
    let dir = mercury_cortex_core::db::data_dir()?;
    assert!(
        dir.to_string_lossy().contains(".mercury/cortex"),
        "data dir should contain .mercury/context"
    );
    Ok(())
}
