use patchwork::{agents, api::{self,AppState}, config::{load_dotenv,Config}, github::GithubClient};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    load_dotenv();
    let config=Config::from_env();
    let agent_config=config.clone();
    tokio::spawn(async move { agents::run_all(agent_config).await; });
    let state=AppState{config,github:GithubClient::from_env().ok()};
    // Demo-local hardening: the Vite dev server proxies /api same-origin, so
    // CORS only needs the dev origins — this keeps arbitrary web pages from
    // issuing cross-origin JSON POSTs (preflighted) against the demo graph.
    let dev_origins=[
        axum::http::HeaderValue::from_static("http://localhost:5173"),
        axum::http::HeaderValue::from_static("http://127.0.0.1:5173"),
    ];
    let app=api::router(state).layer(CorsLayer::new().allow_origin(dev_origins));
    // Loopback by default; API_HOST/API_PORT override for a LAN demo.
    let host=std::env::var("API_HOST").unwrap_or_else(|_|"127.0.0.1".into());
    let port:u16=std::env::var("API_PORT").ok().and_then(|s|s.parse().ok()).unwrap_or(8000);
    let listener=tokio::net::TcpListener::bind((host.as_str(),port)).await?;
    tracing::info!("PatchWork API listening on {host}:{port}");
    axum::serve(listener,app).await?;
    Ok(())
}
