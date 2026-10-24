//! MCP over Streamable HTTP, for ChatGPT connectors and other remote clients.
//! Run: cargo run -p patchwork --bin mcp_http   (listens on MCP_HTTP_ADDR, default 127.0.0.1:8765)
//! /mcp is the owner's full server, behind MCP_BEARER_TOKEN when set.
//! /share/<token>/mcp serves only the tools and fields that token was created with (see create_share).
//! Optional: MCP_ALLOWED_HOSTS (comma-separated) adds public hostnames, e.g. a tunnel domain.
use axum::{extract::Request, http::StatusCode, middleware::{self, Next}, response::{IntoResponse, Response}, routing::any_service, Router};
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

/// Resolve /share/<token>/mcp to its policy and attach it to the request. Unknown tokens get 401.
async fn share_gate(server: PatchworkMcp, mut req: Request, next: Next) -> Response {
    let token = req.uri().path().trim_start_matches("/share/").trim_end_matches("/mcp").to_string();
    match server.share_policy(token).await {
        Ok(Some(policy)) => {
            req.extensions_mut().insert(policy);
            next.run(req).await
        }
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
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
    // Separate session managers: a share session id can never be replayed against /mcp.
    let owner = server.clone();
    let owner_service = StreamableHttpService::new(move || Ok(owner.clone()), Arc::new(LocalSessionManager::default()), http_cfg.clone());
    let shared = server.clone();
    let share_service = StreamableHttpService::new(move || Ok(shared.clone()), Arc::new(LocalSessionManager::default()), http_cfg);
    let gate_server = server.clone();
    let share_route = any_service(share_service).layer(middleware::from_fn(move |req, next| share_gate(gate_server.clone(), req, next)));
    let app = Router::new()
        .route("/mcp", any_service(owner_service).layer(middleware::from_fn(bearer)))
        .route("/share/{token}/mcp", share_route);
    let addr = std::env::var("MCP_HTTP_ADDR").unwrap_or_else(|_| "127.0.0.1:8765".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("patchwork MCP (streamable HTTP) on http://{addr}/mcp and /share/<token>/mcp");
    axum::serve(listener, app).await?;
    Ok(())
}
