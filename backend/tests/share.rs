//! Live regression tests for share policy lookup, validation, and redaction.
//!
//! These tests use the configured graph like `isolation.rs`. Shares are labeled
//! with `mcp-regtest-share-`. Each test's cleanup deletes only its own label,
//! so tests running in parallel cannot delete each other's shares.
use anyhow::{ensure, Result};
use patchwork::{
    graph::{connect, write_query},
    mcp::PatchworkMcp,
    share::{self, SharePolicy},
};

async fn open() -> Result<falkordb::AsyncGraph> {
    patchwork::config::load_dotenv();
    let cfg = patchwork::config::Config::from_env();
    Ok(connect(&cfg.falkordb_host, cfg.falkordb_port, &cfg.graph_name).await?)
}

async fn cleanup(label: &str) {
    let result = async {
        let mut graph = open().await?;
        write_query(
            &mut graph,
            "MATCH (s:Share {label: $label}) DETACH DELETE s",
            serde_json::json!({ "label": label }),
        )
        .await
    }
    .await;
    if let Err(error) = result {
        eprintln!("mcp-regtest share cleanup for {label} (best effort) failed: {error:#}");
    }
}

#[tokio::test]
#[ignore]
async fn share_policy_finds_created_share_and_returns_none_for_unknown_token() {
    let outcome = async {
        let mut graph = open().await?;
        let policy = SharePolicy::new(
            "mcp-regtest-share-lookup",
            &["memory_recall".into()],
            &["note".into()],
        )
        .map_err(anyhow::Error::msg)?;
        let token = share::create(&mut graph, &policy).await?;
        let cfg = patchwork::config::Config::from_env();
        let mcp = PatchworkMcp::new(&cfg.falkordb_host, cfg.falkordb_port, &cfg.graph_name).await?;

        let found = mcp.share_policy(token.clone()).await?;
        ensure!(found.is_some(), "created share token did not resolve");
        let found = found.unwrap();
        ensure!(found.label == policy.label, "share label did not round-trip");
        ensure!(found.tools == policy.tools, "share tools did not round-trip");
        ensure!(found.redact == policy.redact, "share redactions did not round-trip");
        ensure!(
            mcp.share_policy("mcp-regtest-share-unknown-token".into())
                .await?
                .is_none(),
            "unknown token unexpectedly resolved"
        );
        Ok::<_, anyhow::Error>(())
    }
    .await;
    cleanup("mcp-regtest-share-lookup").await;
    outcome.unwrap();
}

#[tokio::test]
#[ignore]
async fn share_policy_rejects_log_encounter() -> Result<()> {
    let outcome = SharePolicy::new(
        "mcp-regtest-share-invalid",
        &["log_encounter".into()],
        &[],
    );
    ensure!(outcome.is_err(), "owner-only log_encounter was accepted");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn share_redaction_removes_note_key_from_json_value() {
    let outcome = async {
        let mut graph = open().await?;
        let policy = SharePolicy::new(
            "mcp-regtest-share-redact",
            &["memory_recall".into()],
            &["note".into()],
        )
        .map_err(anyhow::Error::msg)?;
        let _token = share::create(&mut graph, &policy).await?;

        let mut value = serde_json::json!({
            "rows": [{"contact": "Ada", "note": "private note"}],
            "note": "private top-level note"
        });
        policy.redact_value(&mut value);
        ensure!(
            value == serde_json::json!({"rows": [{"contact": "Ada"}]}),
            "redacted JSON retained a note key: {value}"
        );
        Ok::<_, anyhow::Error>(())
    }
    .await;
    cleanup("mcp-regtest-share-redact").await;
    outcome.unwrap();
}
