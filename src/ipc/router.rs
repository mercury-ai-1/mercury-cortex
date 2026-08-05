use serde_json::{Value, json};

use mercury_cortex_core::runtime::Runtime;
use mercury_cortex_core::runtime::RuntimeContext;
use mercury_cortex_core::runtime::RwLockExt;
use mercury_cortex_core::runtime::status::HealthStatus;
use mercury_cortex_core::service::ServiceError;

use super::{engine, files, graph, profile, project};

/// Dispatch an IPC request to the appropriate domain router.
pub(crate) async fn dispatch(
    ctx: &RuntimeContext,
    method: &str,
    params: Value,
) -> Result<Value, ServiceError> {
    match method {
        m if m.starts_with("project/") => project::dispatch(ctx, method, params).await,
        m if m.starts_with("profile/") => profile::dispatch(ctx, method, params).await,
        m if m.starts_with("files/") => files::dispatch(ctx, method, params).await,
        m if m.starts_with("graph/") => graph::dispatch(ctx, method, params).await,
        m if m.starts_with("engine/") => engine::dispatch(ctx, method, params).await,
        "runtime/ping" => Ok(Value::Null),
        "runtime/status" => runtime_status(ctx),
        "runtime/health" => runtime_health(ctx),
        "runtime/diagnostics" => runtime_diagnostics(ctx),
        "runtime/shutdown" => {
            Runtime::trigger_shutdown(ctx).await;
            ctx.signal_shutdown();
            Ok(Value::Null)
        }
        _ => Err(ServiceError::NotFound(format!("Unknown method: {method}"))),
    }
}

fn runtime_status(ctx: &RuntimeContext) -> Result<Value, ServiceError> {
    let reactor = ctx.status.read_result()?;
    Ok(json!({
        "status": health_status_str(reactor.health),
        "phase": reactor.phase.to_string(),
        "started_at": reactor.started_at,
        "error": reactor.error.as_ref().map(|e| json!({
            "code": e.code.code_str(),
            "message": e.message,
            "recovery": e.recovery,
        })),
    }))
}

fn runtime_health(ctx: &RuntimeContext) -> Result<Value, ServiceError> {
    let reactor = ctx.status.read_result()?;
    let db_available = true;
    let engine_running = ctx.engine.read_result()?.is_some();
    Ok(json!({
        "status": health_status_str(reactor.health),
        "phase": reactor.phase.to_string(),
        "checks": {
            "database": if db_available { "ok" } else { "not_initialized" },
            "engine": if engine_running { "running" } else { "not_initialized" },
        },
        "error": reactor.error.as_ref().map(|e| json!({
            "code": e.code.code_str(),
            "message": e.message,
            "recovery": e.recovery,
        })),
    }))
}

fn runtime_diagnostics(ctx: &RuntimeContext) -> Result<Value, ServiceError> {
    let reactor = ctx.status.read_result()?;
    let trace: Vec<Value> = reactor
        .startup_trace
        .iter()
        .map(|entry| {
            json!({
                "phase": entry.phase.to_string(),
                "duration_ms": entry.duration_ms,
                "error": entry.error.as_ref().map(|e| json!({
                    "code": e.code.code_str(),
                    "message": e.message,
                    "recovery": e.recovery,
                    "source": e.source,
                })),
            })
        })
        .collect();
    Ok(json!({
        "status": health_status_str(reactor.health),
        "phase": reactor.phase.to_string(),
        "error": reactor.error.as_ref().map(|e| json!({
            "code": e.code.code_str(),
            "message": e.message,
            "recovery": e.recovery,
            "source": e.source,
        })),
        "config": {
            "data_dir": ctx.config.data_dir.to_string_lossy(),
            "socket_path": ctx.config.socket_path.to_string_lossy(),
        },
        "startup_trace": trace,
    }))
}

fn health_status_str(health: HealthStatus) -> &'static str {
    match health {
        HealthStatus::Healthy => "healthy",
        HealthStatus::Degraded => "degraded",
        HealthStatus::Unhealthy => "unhealthy",
    }
}
