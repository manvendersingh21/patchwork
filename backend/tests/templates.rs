use patchwork::graph::{connect, run_template};
use patchwork::templates::{T2_TALKS_FOR_ISSUE,T3_PEOPLE_TO_MEET,T4_MEMORY_RECALL,T5_ISSUES_FOR_ENCOUNTER,T6_TALKS_BY_TOPIC};

async fn run(cypher: &str, template: &str, params: serde_json::Value) {
    patchwork::config::load_dotenv();
    let cfg = patchwork::config::Config::from_env();
    let mut g = connect(&cfg.falkordb_host, cfg.falkordb_port, &cfg.graph_name).await.unwrap();
    let result = run_template(&mut g, template, cypher, params).await.unwrap();
    assert!(result.run_time_ms.is_some());
}
#[tokio::test]
#[ignore]
async fn t2_live() { run(T2_TALKS_FOR_ISSUE,"T2",serde_json::json!({"issue":"issue:manvendersingh21/deepcore-n1#1"})).await; }
#[tokio::test]
#[ignore]
async fn t3_live() { run(T3_PEOPLE_TO_MEET,"T3",serde_json::json!({"person":"person:gh:manvendersingh21"})).await; }
#[tokio::test]
#[ignore]
async fn t4_live() { run(T4_MEMORY_RECALL,"T4",serde_json::json!({"issue":"issue:manvendersingh21/deepcore-n1#1"})).await; }
#[tokio::test]
#[ignore]
async fn t5_live() { run(T5_ISSUES_FOR_ENCOUNTER,"T5",serde_json::json!({"encounter":"enc:test"})).await; }
#[tokio::test]
#[ignore]
async fn t6_live() { run(T6_TALKS_BY_TOPIC,"T6",serde_json::json!({"topics":"mcp-auth","day":"","after":""})).await; }
