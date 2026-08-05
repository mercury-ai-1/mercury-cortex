//! The `mercury-cortex version` command handler.
//!
//! Reports the installed Mercury Cortex version together with the binary
//! and data directory paths.  If the binary path or data directory cannot
//! be resolved, the error is reported instead of a version.

use std::path::Path;

use serde_json::json;

use mercury_cortex_core::client::CoreClient;

/// Run the version reporting flow.
pub fn run(json: bool) -> Result<(), anyhow::Error> {
    let version = env!("CARGO_PKG_VERSION");

    let binary_path = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Cannot determine executable path: {e}"))?;
    if !binary_path.exists() {
        anyhow::bail!(
            "Executable not found at its own reported path: {}",
            binary_path.display()
        );
    }

    let data_dir = CoreClient::paths()?.data_dir;

    if json {
        println!(
            "{}",
            json!({
                "version": version,
                "binary_path": binary_path.display().to_string(),
                "data_dir": data_dir.display().to_string(),
            })
        );
    } else {
        println!("Mercury Cortex v{version}");
        println!("  Binary:  {}", binary_dir_canonical(&binary_path));
        println!("  Data:    {}", data_dir.display());
    }

    Ok(())
}

/// Canonicalise the directory containing the binary, falling back to the
/// non-canonical form if resolution fails.
fn binary_dir_canonical(binary_path: &Path) -> String {
    binary_path
        .parent()
        .and_then(|p| std::fs::canonicalize(p).ok())
        .map_or_else(
            || {
                binary_path
                    .parent()
                    .map_or("<unknown>".into(), |p| p.display().to_string())
            },
            |p| p.display().to_string(),
        )
}
