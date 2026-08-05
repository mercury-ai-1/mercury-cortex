use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for `search/code`: filters for indexed file metadata search.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchCodeParams {
    pub query: Option<String>,
    pub path: Option<String>,
    pub purpose: Option<String>,
    pub features: Option<Vec<String>>,
    pub language: Option<String>,
    pub framework: Option<String>,
    pub limit: Option<u32>,
}
