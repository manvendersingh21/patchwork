//! Run with: cargo run -p patchwork --bin mcp   (speaks MCP over stdin/stdout)
use patchwork::{config::load_dotenv, config::Config, mcp::PatchworkMcp};
use rmcp::{transport::stdio, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_dotenv();
    let cfg = Config::from_env();
    let server = PatchworkMcp::new(&cfg.falkordb_host, cfg.falkordb_port, &cfg.graph_name).await?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
