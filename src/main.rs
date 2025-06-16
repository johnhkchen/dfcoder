use anyhow::Result;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::init();
    
    tracing::info!("Starting DFCoder...");
    
    dfcoder_tui::run().await?;
    
    Ok(())
}
