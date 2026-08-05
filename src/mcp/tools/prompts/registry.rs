use serde_json::Value;

use crate::mcp::error::{McpError, McpResult};
use crate::mcp::models::prompt::{Prompt, PromptResult};

use super::{dev, init};

#[must_use]
pub fn list_prompts() -> Vec<Prompt> {
    vec![init::definition(), dev::definition()]
}

pub fn get_prompt(name: &str, _args: Option<&Value>) -> McpResult<PromptResult> {
    match name {
        "mercury-cortex:init" => Ok(init::content()),
        "mercury-cortex:dev" => Ok(dev::content()),
        _ => Err(McpError::InvalidParams(format!("unknown prompt: {name}"))),
    }
}

/// Return the list of available step titles for a given workflow mode.
#[must_use]
pub fn list_steps(mode: &str) -> Vec<Value> {
    match mode {
        "dev" => dev::steps()
            .iter()
            .map(|s| serde_json::json!({ "number": s.number, "title": s.title }))
            .collect(),
        "init" => init::steps()
            .iter()
            .map(|s| serde_json::json!({ "number": s.number, "title": s.title }))
            .collect(),
        _ => vec![],
    }
}

/// Return the (title, content) for a specific workflow step, or None.
#[must_use]
pub fn get_step(mode: &str, number: usize) -> Option<(&'static str, &'static str)> {
    match mode {
        "dev" => {
            let step = dev::steps().iter().find(|s| s.number == number)?;
            let content = dev::step_content(number)?;
            Some((step.title, content))
        }
        "init" => {
            let step = init::steps().iter().find(|s| s.number == number)?;
            let content = init::step_content(number)?;
            Some((step.title, content))
        }
        _ => None,
    }
}
