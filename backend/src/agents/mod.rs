//! Agent loops share graph state and helpers, never one another's modules.
pub mod cartographer;
pub mod pathfinder;
pub mod concierge;

use crate::{config::Config, graph::connect};
use anyhow::Result;

pub async fn run_all(config: Config) {
    let a = config.clone(); let b = config.clone(); let c = config.clone();
    tokio::join!(cartographer::run(a), pathfinder::run(b), concierge::run(c));
}

pub(crate) async fn graph(config: &Config) -> Result<falkordb::AsyncGraph> {
    connect(&config.falkordb_host, config.falkordb_port, &config.graph_name).await
}
