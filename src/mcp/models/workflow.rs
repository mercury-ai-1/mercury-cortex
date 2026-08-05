use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for `workflow/session`: the workflow mode (dev or init).
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowSessionParams {
    pub mode: String,
}

/// Parameters for `workflow/step`: the workflow mode and step number.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowStepParams {
    pub mode: String,
    pub step: u32,
}
