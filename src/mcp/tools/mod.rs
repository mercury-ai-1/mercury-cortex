//! MCP tool handler implementations, organized by domain.
//!
//! Each subdirectory mirrors a two-level MCP tool namespace (`domain/action`).
//! For example, `project/open` → `tools/project/open.rs`.

pub mod cortex;
pub mod file;
pub mod index;
pub mod metadata;
pub mod project;
pub mod search;
pub mod workflow;

pub mod prompts;
