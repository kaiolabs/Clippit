use anyhow::Result;
use tracing::{error, info};

mod engine;
mod typing_buffer;

use engine::ClippitEngine;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("clippit_ibus=debug")
        .init();

    info!(
        "Starting Clippit IBus Engine v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Create and run the engine
    let mut engine = ClippitEngine::new().await?;

    info!("Clippit IBus Engine initialized successfully");
    info!("Waiting for IBus connections...");

    // Run the engine (blocks until shutdown)
    if let Err(e) = engine.run().await {
        error!("Engine error: {}", e);
        return Err(e);
    }

    info!("Clippit IBus Engine shut down gracefully");
    Ok(())
}
