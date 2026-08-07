//! Project initialization orchestration: entry point, DB and IPC flows.

use std::path::Path;

use anyhow::{Context, Result, bail};

use mercury_cortex_core::client::{CoreClient, ProjectAction, RegisterParams};

use crate::ipc::client::RuntimeClient;

/// Run the `mercury-cortex project` command against the current directory.
pub async fn run() -> Result<()> {
    let paths = CoreClient::paths()?;
    if !paths.db_path.exists() {
        bail!("No database found. Run `mercury-cortex setup` first.");
    }

    let project_root =
        std::env::current_dir().context("Failed to determine current working directory")?;

    if let Some(client) = RuntimeClient::try_connect().await {
        return initialize_project_via_ipc(&project_root, &client).await;
    }

    let client = CoreClient::open()?;
    initialize_project(&project_root, &client).await
}

/// Initialize a Mercury Cortex project at `project_root`.
pub async fn initialize_project(project_root: &Path, client: &CoreClient) -> Result<()> {
    if !project_root.exists() {
        bail!(
            "Project root path does not exist: {}",
            project_root.display()
        );
    }

    let project_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .context("Could not determine project name from directory")?
        .to_string();

    let project = client.project();
    let slug = project.slugify(&project_name);
    let project_root_str = project_root.to_string_lossy().to_string();

    let mc_dir = project_root.join(".mercury-cortex");
    std::fs::create_dir_all(&mc_dir)
        .with_context(|| format!("Failed to create {}", mc_dir.display()))?;

    let config_path = mc_dir.join("config.json");
    let mcignore_path = mc_dir.join(".mcignore");
    project.create_or_update_mcignore(&mcignore_path)?;
    project.create_or_update_agents_md(project_root)?;
    project.create_or_update_instructions_md(&mc_dir)?;

    let config_project_id = project.read_config_project_id(&config_path)?;

    let params = RegisterParams {
        config_project_id,
        name: project_name.clone(),
        slug,
        root_path: project_root_str,
    };

    let result = project.register(params).await?;

    print_register_progress(&result.action, result.duplicates_removed);

    project.write_config(&config_path, &result.project_id)?;

    println!("  Project:     {project_name}");
    println!("  Project ID:  {}", result.project_id);
    println!("  Root:        {}", project_root.display());
    println!("✓ Project successfully registered with Mercury Cortex.");

    Ok(())
}

/// Initialize a project via IPC when the runtime daemon is running.
///
/// Scaffolding runs locally through the facade; registration is delegated to
/// the daemon over IPC. Output matches the pre-refactor IPC flow (no progress
/// lines; the daemon-side register produces none).
pub(super) async fn initialize_project_via_ipc(
    project_root: &Path,
    client: &RuntimeClient,
) -> Result<()> {
    if !project_root.exists() {
        bail!(
            "Project root path does not exist: {}",
            project_root.display()
        );
    }

    let project_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .context("Could not determine project name from directory")?
        .to_string();

    let core = CoreClient::open()?;
    let project = core.project();
    let slug = project.slugify(&project_name);
    let project_root_str = project_root.to_string_lossy().to_string();

    let mc_dir = project_root.join(".mercury-cortex");
    std::fs::create_dir_all(&mc_dir)
        .with_context(|| format!("Failed to create {}", mc_dir.display()))?;

    let config_path = mc_dir.join("config.json");
    let mcignore_path = mc_dir.join(".mcignore");
    project.create_or_update_mcignore(&mcignore_path)?;
    project.create_or_update_agents_md(project_root)?;
    project.create_or_update_instructions_md(&mc_dir)?;

    let config_project_id = project.read_config_project_id(&config_path)?;

    let params = RegisterParams {
        config_project_id,
        name: project_name.clone(),
        slug,
        root_path: project_root_str,
    };

    let result = client
        .project_register(params)
        .await
        .map_err(|e| anyhow::anyhow!("IPC error: {e}"))?;

    project.write_config(&config_path, &result.project_id)?;

    println!("  Project:     {project_name}");
    println!("  Project ID:  {}", result.project_id);
    println!("  Root:        {}", project_root.display());
    println!("✓ Project successfully registered with Mercury Cortex.");

    Ok(())
}

/// Print the registration decision lines previously emitted by `register.rs`.
fn print_register_progress(action: &ProjectAction, duplicates_removed: usize) {
    match action {
        ProjectAction::Created => println!("  Creating new project record"),
        ProjectAction::CreatedStaleConfig => {
            println!("  Creating new project record (config had stale project_id)")
        }
        ProjectAction::Reused => println!("  Reusing existing project record"),
        ProjectAction::ReusedStaleConfig => {
            println!("  Reusing existing project record (config was stale)")
        }
        ProjectAction::Moved => println!("  Project location updated (was moved)"),
    }
    if duplicates_removed > 0 {
        println!("  Removed {duplicates_removed} duplicate project record(s) at this root");
    }
}
