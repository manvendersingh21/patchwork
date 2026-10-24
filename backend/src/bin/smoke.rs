#![recursion_limit = "512"]
use anyhow::Result;
use patchwork::config::{load_dotenv, Config};
use patchwork::graph::{connect, query_rows, run_template};
use patchwork::templates::{
    T1_TALKS_FOR_MY_ISSUES, T2_TALKS_FOR_ISSUE, T3_PEOPLE_TO_MEET,
    T4_MEMORY_RECALL, T5_ISSUES_FOR_ENCOUNTER, T6_TALKS_BY_TOPIC,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    load_dotenv();
    let config = Config::from_env();
    let mut g = connect(&config.falkordb_host, config.falkordb_port, &config.graph_name).await?;

    // Determine sample params from the live graph, falling back to config / defaults
    let mut person_uid = if !config.presenter_handle.is_empty() {
        format!("person:gh:{}", config.presenter_handle)
    } else {
        "person:gh:manvendersingh21".to_string()
    };
    if let Ok(rows) = query_rows(&mut g, "MATCH (p:Person {kind: 'self'}) RETURN p.uid AS uid LIMIT 1", json!({})).await {
        if let Some(r) = rows.first() {
            if let Some(uid) = r.get("uid").and_then(|v| v.as_str()) {
                person_uid = uid.to_string();
            }
        }
    }

    let mut issue_uid = if !config.demo_repo.is_empty() {
        format!("issue:{}#1", config.demo_repo)
    } else {
        "issue:manvendersingh21/deepcore-n1#1".to_string()
    };
    if let Ok(rows) = query_rows(&mut g, "MATCH (i:Issue) RETURN i.uid AS uid LIMIT 1", json!({})).await {
        if let Some(r) = rows.first() {
            if let Some(uid) = r.get("uid").and_then(|v| v.as_str()) {
                issue_uid = uid.to_string();
            }
        }
    }

    let mut encounter_uid = "enc:test".to_string();
    if let Ok(rows) = query_rows(&mut g, "MATCH (e:Encounter) RETURN e.uid AS uid LIMIT 1", json!({})).await {
        if let Some(r) = rows.first() {
            if let Some(uid) = r.get("uid").and_then(|v| v.as_str()) {
                encounter_uid = uid.to_string();
            }
        }
    }

    let tests: Vec<(&str, &str, serde_json::Value)> = vec![
        (
            "T1_TALKS_FOR_MY_ISSUES",
            T1_TALKS_FOR_MY_ISSUES,
            json!({"person": person_uid, "day": "", "after": ""}),
        ),
        (
            "T2_TALKS_FOR_ISSUE",
            T2_TALKS_FOR_ISSUE,
            json!({"issue": issue_uid}),
        ),
        (
            "T3_PEOPLE_TO_MEET",
            T3_PEOPLE_TO_MEET,
            json!({"person": person_uid}),
        ),
        (
            "T4_MEMORY_RECALL",
            T4_MEMORY_RECALL,
            json!({"issue": issue_uid}),
        ),
        (
            "T5_ISSUES_FOR_ENCOUNTER",
            T5_ISSUES_FOR_ENCOUNTER,
            json!({"encounter": encounter_uid}),
        ),
        (
            "T6_TALKS_BY_TOPIC",
            T6_TALKS_BY_TOPIC,
            json!({"topics": "mcp-auth", "day": "", "after": ""}),
        ),
    ];

    let mut any_failed = false;
    let mut t1_passed = false;

    for (name, cypher, params) in tests {
        match run_template(&mut g, name, cypher, params).await {
            Ok(envelope) => {
                println!(
                    "{}: PASS (hops={}, rows={}, time={:.3}ms)",
                    name,
                    envelope.hops,
                    envelope.rows.len(),
                    envelope.run_time_ms.unwrap_or(0.0)
                );
                if name == "T1_TALKS_FOR_MY_ISSUES" {
                    t1_passed = true;
                }
            }
            Err(e) => {
                eprintln!("{}: FAIL - {}", name, e);
                any_failed = true;
            }
        }
    }

    if any_failed || !t1_passed {
        std::process::exit(1);
    }

    Ok(())
}
