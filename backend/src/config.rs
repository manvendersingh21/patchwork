//! Environment configuration. Values from .env are loaded only when not already set.
use std::{fs, path::Path};

#[derive(Clone, Debug)]
pub struct Config {
    pub falkordb_host: String,
    pub falkordb_port: u16,
    pub graph_name: String,
    pub presenter_handle: String,
    pub demo_repo: String,
    pub tracked_repos: Vec<String>,
    pub watch_interval_sec: u64,
}

/// Reads `.env` from the current directory, falling back to the repo root, so binaries
/// launched by MCP clients from any directory still reach the configured graph.
pub fn load_dotenv() {
    let repo_env = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env");
    let Ok(content) = fs::read_to_string(".env").or_else(|_| fs::read_to_string(&repo_env)) else { return; };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        let Some((key, value)) = line.split_once('=') else { continue; };
        let key = key.trim();
        if std::env::var_os(key).is_none() {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            // SAFETY: startup initialization occurs before worker threads are spawned.
            unsafe { std::env::set_var(key, value); }
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let tracked_repos = std::env::var("TRACKED_REPOS").unwrap_or_default().split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
        Self {
            falkordb_host: std::env::var("FALKORDB_HOST").unwrap_or_else(|_| "localhost".into()),
            falkordb_port: std::env::var("FALKORDB_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(6379),
            graph_name: std::env::var("GRAPH_NAME").unwrap_or_else(|_| "patchwork".into()),
            presenter_handle: std::env::var("PRESENTER_HANDLE").unwrap_or_default(),
            demo_repo: std::env::var("DEMO_REPO").unwrap_or_default(),
            tracked_repos,
            watch_interval_sec: std::env::var("WATCH_INTERVAL_SEC").ok().and_then(|s| s.parse().ok()).unwrap_or(5),
        }
    }
}
