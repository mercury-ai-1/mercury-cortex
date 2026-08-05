use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for `project/update_mcignore`: root path and patterns to add.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateMcIgnoreParams {
    pub root: String,
    pub patterns: Vec<String>,
}
