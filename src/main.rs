use std::sync::atomic::{AtomicU16, Ordering};

use axum::{Router, routing::get};
use meili_tracing_subscriber::{Credentials, MeiliWriter};
use tokio::net::TcpListener;
use tracing::{debug, error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

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

/// initializes BOTH logging to stdout AND persistent logs in meilisearch
fn init_tracing() {
    // this is `Fn` so we increment doc count in meilisearch with AtomicU16
    let meili_layer_builder = || {
        MeiliWriter::new(
            0,
            Credentials::new(MEILI_HOST, MEILI_MASTER_KEY),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        )
    };

    // TODO: make it clearer what each of these layers are
    // TODO: make sure each message has its own span (right now error! and info! in router resolve
    // to same spane)
    tracing_subscriber::Registry::default()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().with_level(true))
        .with(fmt::layer().json().with_writer(meili_layer_builder))
        .init();
}

async fn run_app() -> anyhow::Result<()> {
    debug!("about to launch app");

    let app = Router::new().route(
        "/",
        get(async || {
            info!("inside handler");
            error!("some error now");
            "hello, world"
        }),
    );

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    if let Err(e) = axum::serve(listener, app).await {
        error!(error=?e);
    }
    Ok(())
}
