//! Live regression test for recalling a LinkedIn DM encounter through T4.
//!
//! Like isolation.rs, this test runs against the configured live graph and
//! detach-deletes its uniquely tagged nodes even when setup or assertions fail.
use anyhow::{ensure, Result};
use patchwork::graph::{connect, query_rows, run_template, write_query, Envelope};
use patchwork::templates::T4_MEMORY_RECALL;

/// Connect exactly like tests/isolation.rs: dotenv -> Config -> graph.
async fn open() -> Result<falkordb::AsyncGraph> {
    patchwork::config::load_dotenv();
    let cfg = patchwork::config::Config::from_env();
    Ok(connect(&cfg.falkordb_host, cfg.falkordb_port, &cfg.graph_name).await?)
}

/// run_template wrapper mirroring tests/isolation.rs.
async fn run(
    g: &mut falkordb::AsyncGraph,
    cypher: &str,
    template: &str,
    params: serde_json::Value,
) -> Result<Envelope> {
    let env = run_template(g, template, cypher, params).await?;
    ensure!(
        env.run_time_ms.is_some(),
        "run_template returned no internal execution time"
    );
    Ok(env)
}

/// Detach-delete all nodes created by this regression test, best effort.
async fn cleanup() {
    let done = async {
        let mut g = open().await?;
        write_query(
            &mut g,
            "MATCH (n) WHERE n.uid CONTAINS 'mcp-regtest-' DETACH DELETE n",
            serde_json::json!({}),
        )
        .await
    }
    .await;
    if let Err(e) = done {
        eprintln!("mcp-regtest cleanup (best effort) failed: {e:#}");
    }
}

async fn linkedin_recall_body() -> Result<()> {
    let mut g = open().await?;
    let topics = query_rows(
        &mut g,
        "MATCH (t:Topic) RETURN t.uid AS uid, t.slug AS slug ORDER BY t.uid LIMIT 1",
        serde_json::json!({}),
    )
    .await?;
    let topic = topics
        .first()
        .ok_or_else(|| anyhow::anyhow!("precondition failed: no Topic in the graph"))?;
    let topic_uid = topic
        .get("uid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("first Topic has no uid"))?;
    let topic_slug = topic
        .get("slug")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("first Topic has no slug"))?;

    let owner = "mcp-regtest-li-owner";
    let repo = "repo:notes/mcp-regtest-linkedin";
    let issue = "issue:notes/mcp-regtest-linkedin#1";
    let encounter = "mcp-regtest-enc-linkedin";
    let contact = "person:li:mcp-regtest-li";
    write_query(
        &mut g,
        "\
        MERGE (p:Person {uid: $owner}) SET p.name = $owner, p.kind = 'self', p.demo = false \
        MERGE (r:Repo {uid: $repo}) SET r.title = $repo \
        MERGE (p)-[:MAINTAINS]->(r) \
        MERGE (i:Issue {uid: $issue}) SET i.title = $issue, i.gh_state = 'open' \
        MERGE (r)-[:HAS_ISSUE]->(i) \
        WITH p, i \
        MATCH (t:Topic {uid: $topic_uid}) \
        MERGE (i)-[:ABOUT]->(t) \
        WITH p, t \
        MERGE (e:Encounter {uid: $encounter}) \
        SET e.note = 'Summarized LinkedIn DM regression conversation', \
            e.place = 'LinkedIn DM', e.source_url = $source_url, \
            e.met_at = '2026-10-24T12:00:00Z', e.state = 'PARSED' \
        MERGE (p)-[:LOGGED]->(e) \
        MERGE (c:Person {uid: $contact}) \
        SET c.name = 'LinkedIn regression contact', c.kind = 'contact', \
            c.demo = false, c.linkedin = 'mcp-regtest-li' \
        MERGE (e)-[:MET]->(c) \
        MERGE (e)-[d:DISCUSSED]->(t) \
        SET d.confidence = '1.0', d.evidence = 'LinkedIn DM regression test'",
        serde_json::json!({
            "owner": owner,
            "repo": repo,
            "issue": issue,
            "encounter": encounter,
            "contact": contact,
            "topic_uid": topic_uid,
            "source_url": format!("https://www.linkedin.com/messaging/thread/mcp-regtest-li/"),
        }),
    )
    .await?;

    let recalled = run(
        &mut g,
        T4_MEMORY_RECALL,
        "T4",
        serde_json::json!({"issue": issue}),
    )
    .await?;
    ensure!(
        recalled.rows.iter().any(|row| {
            row.get("contact_uid").and_then(|v| v.as_str()) == Some(contact)
                && row.get("place").and_then(|v| v.as_str()) == Some("LinkedIn DM")
        }),
        "T4 did not return the LinkedIn contact and place for issue {issue}; rows: {:?}",
        recalled.rows
    );
    // Keep references explicit so the selected pre-existing Topic is evident
    // in test diagnostics and this test uses its actual graph identity.
    ensure!(!topic_slug.is_empty(), "selected Topic has an empty slug");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn linkedin_dm_encounter_is_recalled() {
    let outcome = linkedin_recall_body().await;
    cleanup().await;
    outcome.unwrap();
}
