//! The `mercury-cortex project` command — init a directory as a Mercury Cortex workspace.
//!
//! Delegates to the orchestrate sub-module, which drives the `CoreClient` facade.

mod orchestrate;

pub use orchestrate::initialize_project;
pub use orchestrate::run;
