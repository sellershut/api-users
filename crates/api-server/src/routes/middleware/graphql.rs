use std::{sync::Arc, time::Instant};

use async_graphql::{
    extensions::{Extension, ExtensionContext, ExtensionFactory, NextResolve, ResolveInfo},
    ServerResult, Value,
};
use axum::async_trait;

pub struct Metrics;

impl ExtensionFactory for Metrics {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(MetricsExtension)
    }
}

struct MetricsExtension;

#[async_trait]
impl Extension for MetricsExtension {
    /// Called at resolve field.
    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        let start = Instant::now();
        let field = info.name;

        let result = next.run(ctx, info).await;

        let latency = start.elapsed().as_secs_f64();
        let labels = [("field", field.to_string())];

        metrics::counter!("graphql_field_calls_total", &labels).increment(1);
        metrics::histogram!("graphql_field_resolve_seconds", &labels).record(latency);

        result
    }
}
