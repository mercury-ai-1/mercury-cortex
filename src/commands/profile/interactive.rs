use crate::ipc::client::RuntimeClient;
use anyhow::bail;
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;
use mercury_cortex_core::client::{CoreClient, ProfileData, UpsertParams};
use serde_json::json;

use super::prompts;

pub async fn run(json: bool) -> Result<(), anyhow::Error> {
    let paths = CoreClient::paths()?;

    if !paths.db_path.exists() {
        if json {
            println!(
                "{}",
                json!({ "status": "error", "error": "No database found; run `mercury-cortex setup` first." })
            );
        }
        bail!("No database found. Run `mercury-cortex setup` first.");
    }

    if let Some(client) = RuntimeClient::try_connect().await {
        return run_with_ipc(&client, json).await;
    }

    let client = CoreClient::open()?;
    run_with_db(&client, json).await
}

async fn run_with_db(client: &CoreClient, json: bool) -> Result<(), anyhow::Error> {
    let existing = client.profile().get().await?;

    let result = if let Some(profile) = existing {
        update_profile_interactive(client, &profile, json).await
    } else {
        create_profile_interactive(client, json).await
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) if is_cancelled(&e) => {
            if json {
                println!("{}", json!({ "status": "cancelled" }));
            } else {
                println!("Cancelled.");
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

async fn run_with_ipc(client: &RuntimeClient, json: bool) -> Result<(), anyhow::Error> {
    let existing = client
        .profile_get()
        .await
        .map_err(|e| anyhow::anyhow!("IPC error: {e}"))?;

    let result = if let Some(user) = existing {
        update_profile_via_ipc(client, &user, json).await
    } else {
        create_profile_via_ipc(client, json).await
    };

    match result {
        Ok(()) => Ok(()),
        Err(e) if is_cancelled(&e) => {
            if json {
                println!("{}", json!({ "status": "cancelled" }));
            } else {
                println!("Cancelled.");
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

async fn create_profile_via_ipc(client: &RuntimeClient, json: bool) -> Result<(), anyhow::Error> {
    if !json {
        println!("Create your Mercury Cortex profile");
        println!();
    }

    let name = prompts::prompt_name(None)?;
    let email = prompts::prompt_email_nodb(None)?;
    let agent_name = prompts::prompt_agent_name(None)?;
    let profile_type = prompts::prompt_profile_type(None)?;

    if prompts::confirm_save(&name, &email, &agent_name, &profile_type)? {
        let params = UpsertParams {
            id: None,
            name: name.clone(),
            email: email.clone(),
            agent_name: agent_name.clone(),
            profile_type: profile_type.clone(),
        };
        client
            .profile_upsert(params)
            .await
            .map_err(|e| anyhow::anyhow!("IPC error: {e}"))?;
        if json {
            println!(
                "{}",
                json!({ "status": "created", "name": &name, "email": &email, "agent_name": &agent_name, "profile_type": &profile_type })
            );
        } else {
            println!("✓ Profile created successfully.");
        }
    } else if json {
        println!("{}", json!({ "status": "cancelled" }));
    } else {
        println!("Profile creation cancelled.");
    }

    Ok(())
}

async fn update_profile_via_ipc(
    client: &RuntimeClient,
    record: &ProfileData,
    json: bool,
) -> Result<(), anyhow::Error> {
    let record_id_str = record.id.as_deref().unwrap_or("");
    let current_name = &record.name;
    let current_email = &record.email;
    let current_agent_name = &record.agent_name;
    let current_type = &record.profile_type;
    let created_at = record.created_at.as_deref().unwrap_or_default();

    if !json {
        println!();
        println!("Existing profile:");
        println!("  Name:       {current_name}");
        println!("  Email:      {current_email}");
        println!("  Agent:      {current_agent_name}");
        println!("  Type:       {current_type}");
        println!("  Created:    {created_at}");
        println!();
    }

    if !Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Would you like to update this profile?")
        .default(false)
        .interact()?
    {
        if json {
            println!("{}", json!({ "status": "no_changes" }));
        } else {
            println!("No changes made.");
        }
        return Ok(());
    }

    let name = prompts::prompt_name(Some(current_name))?;
    let email = prompts::prompt_email_nodb(Some(current_email))?;
    let agent_name = prompts::prompt_agent_name(Some(current_agent_name))?;
    let profile_type = prompts::prompt_profile_type(Some(current_type))?;

    if prompts::confirm_save(&name, &email, &agent_name, &profile_type)? {
        let id = if record_id_str.is_empty() {
            None
        } else {
            Some(record_id_str.to_string())
        };
        let params = UpsertParams {
            id,
            name: name.clone(),
            email: email.clone(),
            agent_name: agent_name.clone(),
            profile_type: profile_type.clone(),
        };
        client
            .profile_upsert(params)
            .await
            .map_err(|e| anyhow::anyhow!("IPC error: {e}"))?;
        if json {
            println!(
                "{}",
                json!({ "status": "updated", "name": &name, "email": &email, "agent_name": &agent_name, "profile_type": &profile_type })
            );
        } else {
            println!("✓ Profile updated successfully.");
        }
    } else if json {
        println!("{}", json!({ "status": "no_changes" }));
    } else {
        println!("No changes made.");
    }

    Ok(())
}

async fn create_profile_interactive(client: &CoreClient, json: bool) -> Result<(), anyhow::Error> {
    if !json {
        println!("Create your Mercury Cortex profile");
        println!();
    }

    let name = prompts::prompt_name(None)?;
    let email = prompts::prompt_email(client, None).await?;
    let agent_name = prompts::prompt_agent_name(None)?;
    let profile_type = prompts::prompt_profile_type(None)?;

    if prompts::confirm_save(&name, &email, &agent_name, &profile_type)? {
        client
            .profile()
            .upsert(UpsertParams {
                id: None,
                name: name.clone(),
                email: email.clone(),
                agent_name: agent_name.clone(),
                profile_type: profile_type.clone(),
            })
            .await?;
        if json {
            println!(
                "{}",
                json!({ "status": "created", "name": &name, "email": &email, "agent_name": &agent_name, "profile_type": &profile_type })
            );
        } else {
            println!("✓ Profile created successfully.");
        }
    } else if json {
        println!("{}", json!({ "status": "cancelled" }));
    } else {
        println!("Profile creation cancelled.");
    }

    Ok(())
}

async fn update_profile_interactive(
    client: &CoreClient,
    record: &ProfileData,
    json: bool,
) -> Result<(), anyhow::Error> {
    let record_id_str = record.id.as_deref().unwrap_or("");
    let current_name = &record.name;
    let current_email = &record.email;
    let current_agent_name = &record.agent_name;
    let current_type = &record.profile_type;
    let created_at = record.created_at.as_deref().unwrap_or_default();

    if !json {
        println!();
        println!("Existing profile:");
        println!("  Name:       {current_name}");
        println!("  Email:      {current_email}");
        println!("  Agent:      {current_agent_name}");
        println!("  Type:       {current_type}");
        println!("  Created:    {created_at}");
        println!();
    }

    if !Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Would you like to update this profile?")
        .default(false)
        .interact()?
    {
        if json {
            println!("{}", json!({ "status": "no_changes" }));
        } else {
            println!("No changes made.");
        }
        return Ok(());
    }

    let name = prompts::prompt_name(Some(current_name))?;
    let email = prompts::prompt_email(client, Some(current_email)).await?;
    let agent_name = prompts::prompt_agent_name(Some(current_agent_name))?;
    let profile_type = prompts::prompt_profile_type(Some(current_type))?;

    if prompts::confirm_save(&name, &email, &agent_name, &profile_type)? {
        let id = if record_id_str.is_empty() {
            None
        } else {
            Some(record_id_str.to_string())
        };
        client
            .profile()
            .upsert(UpsertParams {
                id,
                name: name.clone(),
                email: email.clone(),
                agent_name: agent_name.clone(),
                profile_type: profile_type.clone(),
            })
            .await?;
        if json {
            println!(
                "{}",
                json!({ "status": "updated", "name": &name, "email": &email, "agent_name": &agent_name, "profile_type": &profile_type })
            );
        } else {
            println!("✓ Profile updated successfully.");
        }
    } else if json {
        println!("{}", json!({ "status": "no_changes" }));
    } else {
        println!("No changes made.");
    }

    Ok(())
}

fn is_cancelled(err: &anyhow::Error) -> bool {
    for cause in err.chain() {
        if let Some(io_err) = cause.downcast_ref::<std::io::Error>()
            && io_err.kind() == std::io::ErrorKind::Interrupted
        {
            return true;
        }
    }
    false
}
