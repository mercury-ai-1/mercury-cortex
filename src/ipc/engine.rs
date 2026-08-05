use serde_json::Value;

use mercury_cortex_core::runtime::RuntimeContext;
use mercury_cortex_core::service::ServiceError;

pub(crate) async fn dispatch(
    ctx: &RuntimeContext,
    method: &str,
    _params: Value,
) -> Result<Value, ServiceError> {
    match method {
        "engine/status" => {
            let engine = ctx.engine()?;
            let info = engine.info().await;
            Ok(serde_json::to_value(info)?)
        }
        "engine/events" => {
            let engine = ctx.engine()?;
            let entries = engine.event_log().recent_entries(100).await;
            let result: Vec<Value> = entries
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "timestamp": format!("{:?}", e.timestamp),
                        "event_type": e.event_type,
                        "details": e.details,
                    })
                })
                .collect();
            Ok(serde_json::to_value(result)?)
        }
        _ => Err(ServiceError::NotFound(format!("Unknown method: {method}"))),
    }
}
