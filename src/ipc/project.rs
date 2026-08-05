use serde_json::Value;

use mercury_cortex_core::runtime::RuntimeContext;
use mercury_cortex_core::service::ServiceError;
use mercury_cortex_core::service::project::{ProjectService, RegisterParams};

pub(crate) async fn dispatch(
    ctx: &RuntimeContext,
    method: &str,
    params: Value,
) -> Result<Value, ServiceError> {
    match method {
        "project/register" => {
            let p: RegisterParams = serde_json::from_value(params)?;
            let r = ProjectService::register(ctx, p).await?;
            Ok(serde_json::to_value(r)?)
        }
        _ => Err(ServiceError::NotFound(format!("Unknown method: {method}"))),
    }
}
