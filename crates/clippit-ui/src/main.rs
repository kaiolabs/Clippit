mod ui;

use anyhow::Result;
use tracing::info;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("clippit_ui=info")
        .init();

    info!("Starting Clippit UI...");

    ui::run_ui()
}
