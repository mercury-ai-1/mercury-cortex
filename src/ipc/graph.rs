use serde_json::Value;

use mercury_cortex_core::runtime::RuntimeContext;
use mercury_cortex_core::service::ServiceError;
use mercury_cortex_core::service::graph::GraphService;

pub(crate) async fn dispatch(
    ctx: &RuntimeContext,
    method: &str,
    params: Value,
) -> Result<Value, ServiceError> {
    match method {
        "graph/list" => {
            let results = GraphService::list_all(ctx).await?;
            Ok(serde_json::to_value(results)?)
        }
        "graph/project" => {
            let project_id = params
                .get("project_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServiceError::Validation("missing project_id".into()))?;
            let results = GraphService::list_by_project(ctx, project_id).await?;
            Ok(serde_json::to_value(results)?)
        }
        _ => Err(ServiceError::NotFound(format!("Unknown method: {method}"))),
    }
}
