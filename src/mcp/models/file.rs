use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for `file/metadata`: the relative file path to look up.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FileMetadataParams {
    pub path: String,
}
