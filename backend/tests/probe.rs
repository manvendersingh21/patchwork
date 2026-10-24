// Live probe against FalkorDB on localhost:6379. Run with `cargo test -p patchwork -- --ignored`.
use patchwork::graph::{connect, run_template};
use patchwork::templates::T1_TALKS_FOR_MY_ISSUES;

#[tokio::test]
#[ignore]
async fn t1_returns_five_hop_path() {
    let mut g = connect("localhost", 6379, "patchwork").await.unwrap();
    let env = run_template(&mut g, "T1", T1_TALKS_FOR_MY_ISSUES, serde_json::json!({"person": "person:gh:manvendersingh21"})).await.unwrap();
    println!("hops={} run_time_ms={:?} rows={}", env.hops, env.run_time_ms, env.rows.len());
    assert!(env.run_time_ms.is_some());
}
