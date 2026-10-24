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
    let app=api::router(state).layer(CorsLayer::permissive());
    let listener=tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    tracing::info!("PatchWork API listening on :8000");
    axum::serve(listener,app).await?;
    Ok(())
}
