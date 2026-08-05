//! The `mercury-cortex profile` command — create or update the user profile.
//!
//! Delegates to interactive and prompts sub-modules.

mod interactive;
mod prompts;

pub use interactive::run;
pub use prompts::validate_email;
