use rimuru_core::{RimuruWorker, DEFAULT_ENGINE_URL};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("rimuru=info".parse()?)
                .add_directive("tower_http=info".parse()?),
        )
        .init();

    let engine_url =
        std::env::var("RIMURU_ENGINE_URL").unwrap_or_else(|_| DEFAULT_ENGINE_URL.to_string());
    let api_port: u16 = std::env::var("RIMURU_API_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3100);

    let worker = RimuruWorker::new(&engine_url);
    worker.start().await?;

    let kv = worker.kv().clone();
    tokio::spawn(async move {
        if let Err(e) = rimuru_core::http::serve(kv, api_port).await {
            tracing::error!("HTTP server error: {}", e);
        }
    });

    println!("rimuru-worker running (API on port {}). Press Ctrl+C to stop.", api_port);
    tokio::signal::ctrl_c().await?;

    worker.shutdown().await;
    Ok(())
}
