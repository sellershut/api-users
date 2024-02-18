use std::collections::HashMap;

use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{self, Tracer},
    Resource,
};
use opentelemetry_semantic_conventions::resource::{
    DEPLOYMENT_ENVIRONMENT, SERVICE_NAME, SERVICE_VERSION,
};
use sentry::{ClientOptions, IntoDsn};
use tokio::sync::oneshot::Sender;
use tracing::{error, level_filters::LevelFilter, trace};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt, Layer};

use crate::state::env::extract_variable;

pub fn init_tracer() -> anyhow::Result<sentry::ClientInitGuard> {
    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_ver = env!("CARGO_PKG_VERSION");

    let (tx, rx) = tokio::sync::oneshot::channel();
    let tracer = tracer(pkg_name, pkg_ver, tx)?;

    let filter = Targets::new()
        .with_target("api_users", LevelFilter::TRACE)
        .with_default(LevelFilter::TRACE);

    let log_levels = log_levels(pkg_name);

    let sentry_guard = init_sentry()?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| log_levels.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_opentelemetry::layer()
                .with_tracer(tracer)
                .with_filter(filter),
        )
        .with(sentry::integrations::tracing::layer())
        .init();

    tokio::spawn(async move {
        match rx.await {
            Ok(e) => {
                trace!(collector = e, "opentelemetry enabled");
            }
            Err(e) => {
                error!("{e}");
            }
        }
    });

    Ok(sentry_guard)
}

fn init_sentry() -> anyhow::Result<sentry::ClientInitGuard> {
    let dsn = extract_variable("SENTRY_DSN", "http://localhost:9000/1");

    Ok(sentry::init(ClientOptions {
        dsn: dsn.into_dsn()?,
        traces_sample_rate: 0.2,
        release: sentry::release_name!(),
        ..Default::default()
    }))
}

fn tracer(pkg_name: &str, pkg_ver: &str, tx: Sender<String>) -> anyhow::Result<Tracer> {
    let collector_endpoint =
        extract_variable("OPENTELEMETRY_COLLECTOR_HOST", "http://localhost:4317");

    let deployment_env = extract_variable("ENV", "develop");

    let _ = tx.send(collector_endpoint.clone());

    Ok(opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(collector_endpoint),
        )
        .with_trace_config(trace::config().with_resource(Resource::new([
            KeyValue::new(SERVICE_NAME, pkg_name.to_owned()),
            KeyValue::new(SERVICE_VERSION, pkg_ver.to_owned()),
            KeyValue::new(DEPLOYMENT_ENVIRONMENT, deployment_env),
        ])))
        .install_batch(runtime::Tokio)?)
}

fn log_levels(pkg_name: &str) -> String {
    let mut log_levels = HashMap::new();
    log_levels.insert(pkg_name, "trace");
    log_levels.insert("api_interface", "trace");
    log_levels.insert("reqwest", "info");
    log_levels.insert("api_database", "trace");
    log_levels.insert("tower", "warn");
    log_levels.insert("h2", "warn");
    log_levels.insert("hyper", "warn");
    log_levels.insert("tungstenite", "warn");
    log_levels.insert("", "debug"); // Default to empty string for unspecified crate

    log_levels
        .iter()
        .map(|(crate_name, log_level)| {
            if crate_name.is_empty() {
                log_level.to_string()
            } else {
                format!("{}={}", crate_name, log_level).replace('-', "_")
            }
        })
        .collect::<Vec<String>>()
        .join(",")
}
