use anyhow::Result;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    tracing::info!("Starting DFCoder...");
    
    dfcoder_tui::run().await?;
    
    Ok(())
}
