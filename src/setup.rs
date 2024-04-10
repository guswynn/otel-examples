use std::io;
use std::time::Duration;

use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use opentelemetry::KeyValue;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::{trace, Resource};
use tonic::transport::Endpoint;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;

pub fn configure(fmt_debug: bool, otel_debug: bool) {
    // fmt
    let fmt_layer = fmt::layer().with_writer(io::stderr).with_ansi(true);

    // otel
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let channel = Endpoint::from_shared("http://localhost:4317")
        .unwrap()
        .timeout(Duration::from_secs(
            opentelemetry_otlp::OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT,
        ))
        // TODO(guswynn): investigate if this should be non-lazy.
        .connect_with_connector_lazy({
            let mut http = HttpConnector::new();
            http.enforce_http(false);
            HttpsConnector::from((
                http,
                // This is the same as the default, plus an h2 ALPN request.
                tokio_native_tls::TlsConnector::from(
                    native_tls::TlsConnector::builder()
                        .request_alpns(&["h2"])
                        .build()
                        .unwrap(),
                ),
            ))
        });
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_channel(channel);
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(trace::config().with_resource(
            // The latter resources win, so if the user specifies
            // `service.name` in the configuration, it will override the
            // `service.name` value we configure here.
            Resource::new([KeyValue::new("service.name", "tst".to_string())]),
        ))
        .with_exporter(exporter)
        .install_simple()
        .unwrap();

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let stack = tracing_subscriber::registry();
    let stack = stack.with(fmt_layer.with_filter(if fmt_debug {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    }));
    let stack = stack.with(otel_layer.with_filter(if otel_debug {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    }));
    stack.init();
}
