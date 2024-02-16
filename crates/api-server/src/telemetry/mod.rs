use sentry::ClientInitGuard;

pub mod metrics;
mod tracing;

pub fn initialise() -> anyhow::Result<ClientInitGuard> {
    tracing::init_tracer()
}
