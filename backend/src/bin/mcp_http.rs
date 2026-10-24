//! MCP over Streamable HTTP, for ChatGPT connectors and other remote clients.
//! Run: cargo run -p patchwork --bin mcp_http   (listens on MCP_HTTP_ADDR, default 127.0.0.1:8765, path /mcp)
//! Optional: MCP_BEARER_TOKEN requires `Authorization: Bearer <token>` on every request.
//! Optional: MCP_ALLOWED_HOSTS (comma-separated) adds public hostnames, e.g. a tunnel domain.
use axum::{extract::Request, http::StatusCode, middleware::{self, Next}, response::Response, Router};
use patchwork::{config::{load_dotenv, Config}, mcp::PatchworkMcp};
use rmcp::transport::streamable_http_server::{session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService};
use std::sync::Arc;

async fn bearer(req: Request, next: Next) -> Result<Response, StatusCode> {
    match std::env::var("MCP_BEARER_TOKEN") {
        Ok(token) if !token.is_empty() => {
            let ok = req.headers().get("authorization").and_then(|v| v.to_str().ok()) == Some(format!("Bearer {token}").as_str());
            if ok { Ok(next.run(req).await) } else { Err(StatusCode::UNAUTHORIZED) }
        }
        _ => Ok(next.run(req).await),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_dotenv();
    let cfg = Config::from_env();
    let server = PatchworkMcp::new(&cfg.falkordb_host, cfg.falkordb_port, &cfg.graph_name).await?;
    let mut http_cfg = StreamableHttpServerConfig::default();
    for host in std::env::var("MCP_ALLOWED_HOSTS").unwrap_or_default().split(',').map(str::trim).filter(|h| !h.is_empty()) {
        http_cfg.allowed_hosts.push(host.to_string());
    }
    let service = StreamableHttpService::new(move || Ok(server.clone()), Arc::new(LocalSessionManager::default()), http_cfg);
    let app = Router::new().nest_service("/mcp", service).layer(middleware::from_fn(bearer));
    let addr = std::env::var("MCP_HTTP_ADDR").unwrap_or_else(|_| "127.0.0.1:8765".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("patchwork MCP (streamable HTTP) on http://{addr}/mcp");
    axum::serve(listener, app).await?;
    Ok(())
}
