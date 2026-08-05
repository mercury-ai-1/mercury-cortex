use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ProjectMetadata;

/// Parameters for `project/open`: identifies the project to open.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ProjectOpenParams {
    pub project_id: String,
    pub root: String,
}

/// Parameters for `project/register`: the root directory to initialize.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ProjectRegisterParams {
    pub root: String,
}

/// Parameters for `project/update`: project ID and AI-detected metadata.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ProjectUpdateParams {
    pub project_id: String,
    pub metadata: ProjectMetadata,
}
