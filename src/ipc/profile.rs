use serde_json::Value;

use mercury_cortex_core::runtime::RuntimeContext;
use mercury_cortex_core::service::ServiceError;
use mercury_cortex_core::service::profile::{ProfileService, UpsertParams};

pub(crate) async fn dispatch(
    ctx: &RuntimeContext,
    method: &str,
    params: Value,
) -> Result<Value, ServiceError> {
    match method {
        "profile/get" => {
            let r = ProfileService::get(ctx).await?;
            Ok(serde_json::to_value(r)?)
        }
        "profile/upsert" => {
            let p: UpsertParams = serde_json::from_value(params)?;
            let r = ProfileService::upsert(ctx, p).await?;
            Ok(serde_json::to_value(r)?)
        }
        _ => Err(ServiceError::NotFound(format!("Unknown method: {method}"))),
    }
}
