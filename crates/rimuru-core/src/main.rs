use rimuru_core::{DEFAULT_ENGINE_URL, RimuruWorker};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("rimuru=info".parse()?))
        .init();

    let engine_url =
        std::env::var("RIMURU_ENGINE_URL").unwrap_or_else(|_| DEFAULT_ENGINE_URL.to_string());

    let worker = RimuruWorker::new(&engine_url);
    worker.start().await?;

    println!("rimuru-worker running (API via iii-http on engine port). Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;

    worker.shutdown().await;
    Ok(())
}
