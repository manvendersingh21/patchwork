//! Append-only transition log; agents never read events for coordination.
use anyhow::Result;
use falkordb::AsyncGraph;
use uuid::Uuid;

pub async fn write_event(g: &mut AsyncGraph, agent: &str, action: &str, target: &str, detail: &str) -> Result<()> {
    let cypher = "MERGE (e:AgentEvent {uid: $uid}) SET e.ts = timestamp(), e.agent = $agent, e.action = $action, e.target = $target, e.detail = $detail";
    g.query(cypher).with_params(vec![
        (String::from("uid"), format!("evt:{}", Uuid::new_v4())),
        (String::from("agent"), agent.to_string()),
        (String::from("action"), action.to_string()),
        (String::from("target"), target.to_string()),
        (String::from("detail"), detail.to_string()),
    ]).execute().await?;
    Ok(())
}
