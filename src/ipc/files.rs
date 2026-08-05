use serde_json::Value;

use mercury_cortex_core::engine::SearchQuery;
use mercury_cortex_core::runtime::RuntimeContext;
use mercury_cortex_core::service::ServiceError;
use mercury_cortex_core::service::file_data::{FileDataFilterParams, FileDataService};

pub(crate) async fn dispatch(
    ctx: &RuntimeContext,
    method: &str,
    params: Value,
) -> Result<Value, ServiceError> {
    match method {
        "files/list" => {
            let filter: FileDataFilterParams = serde_json::from_value(params)?;
            let results = FileDataService::list(ctx, &filter).await?;
            Ok(serde_json::to_value(results)?)
        }
        "files/get" => {
            let id = params
                .as_str()
                .ok_or_else(|| ServiceError::Validation("expected string id".into()))?;
            let result = FileDataService::get_by_id(ctx, id).await?;
            Ok(serde_json::to_value(result)?)
        }
        "files/search" => {
            let query: SearchQuery = serde_json::from_value(params)?;
            let engine = ctx.engine()?;
            let results = engine.search(&query).await?;
            Ok(serde_json::to_value(results)?)
        }
        _ => Err(ServiceError::NotFound(format!("Unknown method: {method}"))),
    }
}
