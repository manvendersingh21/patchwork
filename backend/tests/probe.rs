// Live checks against the FalkorDB instance configured in .env. Run with `cargo test -p patchwork -- --ignored`.
use patchwork::graph::{connect, run_template};
use patchwork::templates::T1_TALKS_FOR_MY_ISSUES;

#[tokio::test]
#[ignore]
async fn act1_day2_morning_returns_five_hop_mcp_talk() {
    patchwork::config::load_dotenv();
    let cfg = patchwork::config::Config::from_env();
    let mut g = connect(&cfg.falkordb_host, cfg.falkordb_port, &cfg.graph_name).await.unwrap();
    let params = serde_json::json!({"person": "person:gh:manvendersingh21", "day": "2026-10-29", "after": "10:00", "before": ""});
    let env = run_template(&mut g, "T1", T1_TALKS_FOR_MY_ISSUES, params).await.unwrap();
    println!("hops={} run_time_ms={:?} rows={:?}", env.hops, env.run_time_ms, env.rows);
    assert_eq!(env.hops, 5);
    assert!(env.run_time_ms.is_some());
    assert!(env.rows.iter().any(|r| r["talk"] == "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers"));
}
