pub mod daemon;
pub mod db;
mod dispatch;
pub mod help;
pub mod mcp;
pub mod profile;
pub mod project;
mod setup;
pub mod version;

pub use dispatch::{Commands, dispatch};
