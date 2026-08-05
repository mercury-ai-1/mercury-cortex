//! CLI help text, externalized from the command definitions.
//!
//! One file per command area. Each exports `&'static str` consts that clap
//! renders: `{CMD}_ABOUT` (short `-h` + parent listings), `{CMD}_LONG`
//! (`--help` prose), `{CMD}_EXAMPLES` (an `Examples:` section in `--help`),
//! and `{CMD}_{ARG}_LONG` (arg-level `--help` detail).

mod daemon;
mod db;
mod mcp;
mod profile;
mod project;
mod root;
mod setup;
mod version;

pub use daemon::*;
pub use db::*;
pub use mcp::*;
pub use profile::*;
pub use project::*;
pub use root::*;
pub use setup::*;
pub use version::*;
