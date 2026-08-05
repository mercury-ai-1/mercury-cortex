#[test]
fn test_validate_email_valid() {
    assert!(mercury_cortex::commands::profile::validate_email("user@example.com").is_ok());
}

#[test]
fn test_validate_email_empty() {
    assert!(mercury_cortex::commands::profile::validate_email("").is_err());
}

#[test]
fn test_validate_email_only_spaces() {
    assert!(mercury_cortex::commands::profile::validate_email("   ").is_err());
}

#[test]
fn test_validate_email_no_at() {
    assert!(mercury_cortex::commands::profile::validate_email("userexample.com").is_err());
}

#[test]
fn test_validate_email_no_local_part() {
    assert!(mercury_cortex::commands::profile::validate_email("@example.com").is_err());
}

#[test]
fn test_validate_email_no_domain() {
    assert!(mercury_cortex::commands::profile::validate_email("user@").is_err());
}

#[test]
fn test_validate_email_no_tld() {
    assert!(mercury_cortex::commands::profile::validate_email("user@example").is_err());
}

#[test]
fn test_validate_email_contains_spaces() {
    assert!(mercury_cortex::commands::profile::validate_email("user @ example.com").is_err());
}

#[test]
fn test_validate_email_multiple_at() {
    assert!(mercury_cortex::commands::profile::validate_email("user@domain@example.com").is_err());
}

#[test]
fn test_validate_email_trims_input() {
    assert!(mercury_cortex::commands::profile::validate_email("  user@example.com  ").is_ok());
}
