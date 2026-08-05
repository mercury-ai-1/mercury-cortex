/// AI-detected language, framework, and technology stack for a project.
///
/// `project/update` should be called with a structured JSON object for
/// `metadata`.  Some AI clients double-encode nested arguments as JSON
/// strings; this type tolerates both forms so those calls still succeed.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct ProjectMetadata {
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub framework: Option<String>,
}

#[derive(serde::Deserialize)]
struct ProjectMetadataFields {
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    framework: Option<String>,
}

impl<'de> serde::Deserialize<'de> for ProjectMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;

        let value = serde_json::Value::deserialize(deserializer)?;
        let fields = match value {
            serde_json::Value::Object(_) => serde_json::from_value::<ProjectMetadataFields>(value),
            serde_json::Value::String(s) => serde_json::from_str::<ProjectMetadataFields>(&s),
            other => {
                return Err(D::Error::custom(format!(
                    "ProjectMetadata: expected an object or a JSON-encoded object string, got {other}"
                )));
            }
        };
        let fields = fields.map_err(|e| D::Error::custom(format!("ProjectMetadata: {e}")))?;
        Ok(ProjectMetadata {
            language: fields.language,
            framework: fields.framework,
        })
    }
}
