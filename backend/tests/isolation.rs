//! Live regression tests for cross-user isolation in T4 (memory_recall) and
//! T5 (issues_for_encounter).
//!
//! Original bugs: T4 recalled encounters logged by *anyone* who discussed the
//! issue's topic, and T5 bridged an encounter to issues owned by *anyone* (or
//! by the presenter demo) — both leaked private data across users. The fixed
//! templates bind every row to the issue's owner / the encounter's logger, and
//! these tests rebuild each leak shape against the live graph to prove it.
//!
//! Same conventions as tests/templates.rs: `#[ignore]`d `#[tokio::test`]s that
//! load .env through patchwork::config, connect to the configured graph, and
//! read only through run_template(). The graph is shared with a live demo, so
//! every uid created here contains `mcp-regtest-` and exactly those uids are
//! detach-deleted afterwards — even when an assertion fails.
use anyhow::{ensure, Result};
use patchwork::graph::{connect, query_rows, run_template, write_query, Envelope};
use patchwork::templates::{T4_MEMORY_RECALL, T5_ISSUES_FOR_ENCOUNTER};

/// Connect exactly like tests/templates.rs: dotenv -> Config -> graph.
async fn open() -> Result<falkordb::AsyncGraph> {
    patchwork::config::load_dotenv();
    let cfg = patchwork::config::Config::from_env();
    Ok(connect(&cfg.falkordb_host, cfg.falkordb_port, &cfg.graph_name).await?)
}

/// run_template wrapper mirroring the `run` helper in tests/templates.rs.
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

/// The demo graph must already contain the conference Topic that issues and
/// encounters hang off; fail with a clear message instead of empty results.
async fn require_mcp_auth_topic(g: &mut falkordb::AsyncGraph) -> Result<()> {
    let rows = query_rows(
        g,
        "MATCH (t:Topic {slug: 'mcp-auth'}) RETURN t.uid AS uid",
        serde_json::json!({}),
    )
    .await?;
    ensure!(
        !rows.is_empty(),
        "precondition failed: no Topic {{slug:'mcp-auth'}} in the graph"
    );
    Ok(())
}

/// Give `mcp-regtest-<tag>` a notes repo with one open issue ABOUT mcp-auth —
/// exactly the ownership chain (Person)-[:MAINTAINS]->(Repo)-[:HAS_ISSUE]->(Issue)
/// that the fixed T4/T5 scope to. Returns the issue uid.
async fn seed_owner(g: &mut falkordb::AsyncGraph, tag: &str) -> Result<String> {
    let issue = format!("issue:notes/mcp-regtest-{tag}#1");
    write_query(
        g,
        "\
        MERGE (p:Person {uid: $person}) SET p.name = $person, p.kind = 'self', p.demo = false \
        MERGE (r:Repo {uid: $repo}) SET r.title = $repo \
        MERGE (p)-[:MAINTAINS]->(r) \
        MERGE (i:Issue {uid: $issue}) SET i.title = $issue, i.gh_state = 'open' \
        MERGE (r)-[:HAS_ISSUE]->(i) \
        WITH DISTINCT i \
        MATCH (t:Topic {slug: 'mcp-auth'}) \
        MERGE (i)-[:ABOUT]->(t)",
        serde_json::json!({
            "person": format!("mcp-regtest-{tag}"),
            "repo": format!("repo:notes/mcp-regtest-{tag}"),
            "issue": issue,
        }),
    )
    .await?;
    Ok(issue)
}

/// `mcp-regtest-<tag>` logs an encounter: LOGGED from the person, MET a fresh
/// contact, DISCUSSED the shared mcp-auth topic. Returns the encounter uid.
async fn seed_encounter(
    g: &mut falkordb::AsyncGraph,
    tag: &str,
    note: &str,
    place: &str,
    met_at: &str,
) -> Result<String> {
    let enc = format!("mcp-regtest-enc-{tag}");
    write_query(g, "\
        MERGE (p:Person {uid: $person}) \
        MERGE (e:Encounter {uid: $enc}) SET e.note = $note, e.place = $place, e.met_at = $met_at, e.state = 'PARSED' \
        MERGE (p)-[:LOGGED]->(e) \
        MERGE (c:Person {uid: $contact}) SET c.name = $contact, c.kind = 'contact', c.demo = false \
        MERGE (e)-[:MET]->(c) \
        WITH DISTINCT e \
        MATCH (t:Topic {slug: 'mcp-auth'}) \
        MERGE (e)-[d:DISCUSSED]->(t) SET d.confidence = '1.0', d.evidence = 'isolation regression test'",
        serde_json::json!({
            "person": format!("mcp-regtest-{tag}"),
            "contact": format!("mcp-regtest-contact-{tag}"),
            "enc": enc,
            "note": note,
            "place": place,
            "met_at": met_at,
        })).await?;
    Ok(enc)
}

/// Detach-delete every node these tests create: all their uids contain
/// 'mcp-regtest-', which covers both the `mcp-regtest-*` prefixed uids and the
/// `repo:notes/mcp-regtest-*` / `issue:notes/mcp-regtest-*` shapes. Best effort.
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

async fn t4_body() -> Result<()> {
    let mut g = open().await?;
    require_mcp_auth_topic(&mut g).await?;
    let issue_a = seed_owner(&mut g, "a").await?;
    // Person B — a stranger who neither maintains nor tracks A's repo — logs a
    // private encounter that happens to discuss A's issue topic.
    seed_encounter(
        &mut g,
        "b",
        "mcp-regtest-b private note",
        "mcp-regtest-b place",
        "2026-10-24T09:00:00Z",
    )
    .await?;

    // Original bug: T4 matched ANY encounter discussing the issue's topic, so
    // B's private note/place leaked into A's memory recall. The fix requires
    // the encounter's logger to own the issue, and A has logged nothing yet:
    // exactly zero rows must come back.
    let leaked = run(
        &mut g,
        T4_MEMORY_RECALL,
        "T4",
        serde_json::json!({"issue": issue_a}),
    )
    .await?;
    ensure!(
        leaked.rows.is_empty(),
        "T4 leaked a non-owner's encounter into A's memory recall: {:?}",
        leaked
            .rows
            .iter()
            .map(|r| (
                r.get("note").and_then(|v| v.as_str()),
                r.get("place").and_then(|v| v.as_str())
            ))
            .collect::<Vec<_>>()
    );

    // A logs its own encounter on the same topic; recall must now surface it.
    seed_encounter(
        &mut g,
        "a",
        "mcp-regtest-a private note",
        "mcp-regtest-a place",
        "2026-10-24T10:00:00Z",
    )
    .await?;
    let recalled = run(
        &mut g,
        T4_MEMORY_RECALL,
        "T4",
        serde_json::json!({"issue": issue_a}),
    )
    .await?;
    ensure!(
        !recalled.rows.is_empty(),
        "T4 returned nothing for the owner's own encounter on the issue topic"
    );
    for row in &recalled.rows {
        ensure!(
            row.get("note").and_then(|v| v.as_str()) == Some("mcp-regtest-a private note")
                && row.get("place").and_then(|v| v.as_str()) == Some("mcp-regtest-a place"),
            "T4 recalled a foreign encounter (note/place mismatch) for A's issue: {row:?}"
        );
    }
    ensure!(recalled.rows.iter().all(|r| r.get("contact_uid").and_then(|v| v.as_str()) == Some("mcp-regtest-contact-a")),
        "T4 recalled a contact A never met: {:?}",
        recalled.rows.iter().map(|r| r.get("contact_uid")).collect::<Vec<_>>());
    Ok(())
}

async fn t5_body() -> Result<()> {
    let mut g = open().await?;
    require_mcp_auth_topic(&mut g).await?;
    // A and B each own a notes repo with an open mcp-auth issue.
    seed_owner(&mut g, "a").await?;
    seed_owner(&mut g, "b").await?;
    // Only B logs an encounter about that topic.
    let enc_b = seed_encounter(
        &mut g,
        "b",
        "mcp-regtest-b private note",
        "mcp-regtest-b place",
        "2026-10-24T09:00:00Z",
    )
    .await?;

    // Original bug: T5 bridged an encounter to EVERY open issue about the
    // discussed topic, so B's bridge list surfaced A's issues and the
    // presenter demo's issues. The fix keeps bridges on the logger's own repos.
    let bridged = run(
        &mut g,
        T5_ISSUES_FOR_ENCOUNTER,
        "T5",
        serde_json::json!({"encounter": enc_b}),
    )
    .await?;
    ensure!(
        !bridged.rows.is_empty(),
        "T5 returned none of B's own issues for B's encounter"
    );
    let uids: Vec<&str> = bridged
        .rows
        .iter()
        .map(|r| r.get("issue_uid").and_then(|v| v.as_str()).unwrap_or(""))
        .collect();
    ensure!(
        !uids.iter().any(|u| u.contains("mcp-regtest-a")),
        "T5 leaked person A's issues into B's bridges: {uids:?}"
    );
    ensure!(
        !uids.iter().any(|u| u.contains("deepcore")),
        "T5 leaked presenter/demo repo issues into B's bridges: {uids:?}"
    );
    for uid in &uids {
        ensure!(
            uid.starts_with("issue:notes/mcp-regtest-b"),
            "T5 bridged B's encounter to a foreign issue {uid:?}: {uids:?}"
        );
    }
    Ok(())
}

#[tokio::test]
#[ignore]
async fn t4_memory_recall_isolation() {
    let outcome = t4_body().await;
    cleanup().await; // runs even on failure so the shared graph stays clean
    outcome.unwrap();
}

#[tokio::test]
#[ignore]
async fn t5_encounter_bridge_scoping() {
    let outcome = t5_body().await;
    cleanup().await;
    outcome.unwrap();
}
