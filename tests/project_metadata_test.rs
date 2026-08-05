use serde_json::json;

use mercury_cortex::mcp::models::{ProjectMetadata, ProjectUpdateParams};

#[test]
fn project_metadata_deserializes_from_object() {
    let m: ProjectMetadata =
        serde_json::from_value(json!({"language": "Dart", "framework": "Flutter"})).unwrap();
    assert_eq!(m.language.as_deref(), Some("Dart"));
    assert_eq!(m.framework.as_deref(), Some("Flutter"));
}

#[test]
fn project_metadata_deserializes_from_json_string() {
    let m: ProjectMetadata = serde_json::from_value(json!(
        "{\"language\": \"Dart\", \"framework\": \"Flutter\"}"
    ))
    .unwrap();
    assert_eq!(m.language.as_deref(), Some("Dart"));
    assert_eq!(m.framework.as_deref(), Some("Flutter"));
}

#[test]
fn project_update_params_accepts_string_encoded_metadata() {
    let p: ProjectUpdateParams = serde_json::from_value(json!({
        "project_id": "projects:test",
        "metadata": "{\"language\": \"Dart\", \"framework\": \"Flutter\"}"
    }))
    .unwrap();
    assert_eq!(p.project_id, "projects:test");
    assert_eq!(p.metadata.language.as_deref(), Some("Dart"));
    assert_eq!(p.metadata.framework.as_deref(), Some("Flutter"));
}

#[test]
fn project_metadata_rejects_non_json_string() {
    let err = serde_json::from_value::<ProjectMetadata>(json!("not an object")).unwrap_err();
    assert!(err.to_string().contains("ProjectMetadata"));
}

#[test]
fn project_metadata_rejects_non_object_non_string() {
    let err = serde_json::from_value::<ProjectMetadata>(json!(42)).unwrap_err();
    assert!(err.to_string().contains("ProjectMetadata"));
}
