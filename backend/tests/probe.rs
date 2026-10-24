// Live checks against FalkorDB on localhost:6379. Run with `cargo test -p patchwork -- --ignored`.
use patchwork::graph::{connect, run_template};
use patchwork::templates::T1_TALKS_FOR_MY_ISSUES;

#[tokio::test]
#[ignore]
async fn act1_day2_morning_returns_five_hop_mcp_talk() {
    let mut g = connect("localhost", 6379, "patchwork").await.unwrap();
    let params = serde_json::json!({"person": "person:gh:manvendersingh21", "day": "2026-10-29", "after": "10:00"});
    let env = run_template(&mut g, "T1", T1_TALKS_FOR_MY_ISSUES, params).await.unwrap();
    println!("hops={} run_time_ms={:?} rows={:?}", env.hops, env.run_time_ms, env.rows);
    assert_eq!(env.hops, 5);
    assert!(env.run_time_ms.is_some());
    assert!(env.rows.iter().any(|r| r["talk"] == "Open pull requests, don't merge them: Fine-grained authorization for hosted MCP servers"));
}
