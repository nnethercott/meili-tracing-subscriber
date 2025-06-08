use axum::{Router, routing::get};
use meili_tracing_subscriber::{Credentials, MeiliWriter};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::net::TcpListener;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, MakeSpan, TraceLayer};
use tracing::{error, info, span, warn};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

// tip: don't do this in prod
const MEILI_HOST: &'static str = "http://localhost:7700/";
const MEILI_MASTER_KEY: &'static str = "password";

static COUNTER: AtomicU16 = AtomicU16::new(0);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    run_app().await?;
    Ok(())
}

fn init_tracing() {
    // this is `Fn` so we increment doc count in meilisearch with AtomicU16
    let meili_layer_builder = || {
        MeiliWriter::new(
            0,
            Credentials::new(MEILI_HOST, MEILI_MASTER_KEY),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        )
    };
    let meili_layer = fmt::layer()
        .json()
        .with_writer(meili_layer_builder)
        .with_span_list(false);

    let fmt_layer = fmt::layer().with_level(true);

    tracing_subscriber::Registry::default()
        .with(EnvFilter::from_default_env())
        .with(fmt_layer)
        .with(meili_layer)
        .init();
}

#[derive(Clone)]
struct CustomSpan;

impl<T> MakeSpan<T> for CustomSpan {
    fn make_span(&mut self, request: &axum::http::Request<T>) -> tracing::Span {
        span!(
            tracing::Level::INFO,
            "http",
            method= %request.method(),
            uri = %request.uri().path(),
            span_id = %Uuid::new_v4(),
        )
    }
}

async fn run_app() -> anyhow::Result<()> {
    let app = Router::new()
        // a route with info span
        .route(
            "/ok",
            get(async || {
                info!("hello, world");
                "hello, world"
            }),
        )
        // a route with a warning
        .route(
            "/warn",
            get(async || {
                warn!("ahhhhHH bad");
                "not good!"
            }),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(CustomSpan)
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Millis),
                ),
        );

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    if let Err(e) = axum::serve(listener, app).await {
        error!(error=?e);
    }
    Ok(())
}
