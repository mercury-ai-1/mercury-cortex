use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use mercury_cortex_core::client::CoreClient;

pub(super) fn prompt_name(default: Option<&str>) -> Result<String, anyhow::Error> {
    let theme = ColorfulTheme::default();
    let mut input = Input::<String>::with_theme(&theme)
        .with_prompt("Full Name")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Name cannot be empty")
            } else {
                Ok(())
            }
        });

    if let Some(val) = default {
        input = input.default(val.to_string());
    }

    Ok(input.interact_text()?.trim().to_string())
}

pub(super) async fn prompt_email(
    client: &CoreClient,
    current_email: Option<&str>,
) -> Result<String, anyhow::Error> {
    loop {
        let theme = ColorfulTheme::default();
        let mut input = Input::<String>::with_theme(&theme)
            .with_prompt("Email Address")
            .validate_with(|input: &String| -> Result<(), String> { validate_email(input) });

        if let Some(val) = current_email {
            input = input.default(val.to_string());
        }

        let email = input.interact_text()?.trim().to_string();

        let is_duplicate = client.profile().email_exists(&email, current_email).await?;
        if is_duplicate {
            eprintln!("  This email is already in use. Please use a different email.");
            continue;
        }

        return Ok(email);
    }
}

pub(super) fn prompt_email_nodb(current_email: Option<&str>) -> Result<String, anyhow::Error> {
    let theme = ColorfulTheme::default();
    let mut input = Input::<String>::with_theme(&theme)
        .with_prompt("Email Address")
        .validate_with(|input: &String| -> Result<(), String> { validate_email(input) });

    if let Some(val) = current_email {
        input = input.default(val.to_string());
    }

    let email = input.interact_text()?.trim().to_string();
    Ok(email)
}

pub(super) fn prompt_agent_name(default: Option<&str>) -> Result<String, anyhow::Error> {
    let theme = ColorfulTheme::default();
    let mut input = Input::<String>::with_theme(&theme)
        .with_prompt("Agent name (suffix only — e.g. one, tj, john123)")
        .validate_with(|input: &String| -> Result<(), String> {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                return Err("Agent suffix cannot be empty".into());
            }
            if !trimmed
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                return Err("Only lowercase letters, numbers, and hyphens allowed".into());
            }
            if !trimmed.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit()) {
                return Err("Suffix must start with a letter or number".into());
            }
            Ok(())
        });

    if let Some(val) = default {
        let suffix = val.strip_prefix("agent-").unwrap_or(val);
        input = input.default(suffix.to_string());
    }

    let suffix = input.interact_text()?.trim().to_string();
    Ok(format!("agent-{suffix}"))
}

pub(super) fn prompt_profile_type(default: Option<&str>) -> Result<String, anyhow::Error> {
    let options = &["personal", "organization"];
    let default_idx = default
        .and_then(|d| options.iter().position(|&o| o == d))
        .unwrap_or(0);

    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Profile Type")
        .default(default_idx)
        .items(options)
        .interact()?;

    Ok(options[idx].to_string())
}

pub(super) fn confirm_save(
    name: &str,
    email: &str,
    agent_name: &str,
    profile_type: &str,
) -> Result<bool, anyhow::Error> {
    println!();
    println!("Profile summary:");
    println!("  Name:      {name}");
    println!("  Email:     {email}");
    println!("  Agent:     {agent_name}");
    println!("  Type:      {profile_type}");
    println!();

    Ok(Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Save this profile?")
        .default(true)
        .interact()?)
}

pub fn validate_email(input: &str) -> Result<(), String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Email cannot be empty".into());
    }
    if !trimmed.contains('@') || trimmed.chars().filter(|&c| c == '@').count() != 1 {
        return Err("Email must contain exactly one '@' symbol".into());
    }
    let parts: Vec<&str> = trimmed.split('@').collect();
    if parts[0].is_empty() {
        return Err("Email must have a local part before '@'".into());
    }
    if !parts[1].contains('.') {
        return Err("Email domain must contain a '.'".into());
    }
    if trimmed.contains(' ') {
        return Err("Email must not contain spaces".into());
    }
    Ok(())
}
