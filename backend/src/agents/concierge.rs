use super::graph;
use crate::{config::Config, events::write_event, graph::{query_rows,write_query}, llm};
use anyhow::Result;
use serde_json::json;
use tokio::time::{sleep,Duration};

pub async fn run(config:Config){let _=recover(&config).await;loop{let worked=match process(&config).await{Ok(v)=>v,Err(e)=>{tracing::warn!(error=%e,"Concierge failed");false}};if !worked{sleep(Duration::from_millis(500)).await;}}}
async fn recover(c:&Config)->Result<()>{let mut g=graph(c).await?;write_query(&mut g,"MATCH ()-[b:BRIDGE {state:'DRAFTING'}]->() SET b.state='DISCOVERED'",json!({})).await?;Ok(())}
async fn process(c:&Config)->Result<bool>{
    let mut g=graph(c).await?;
    let rows=query_rows(&mut g,"MATCH (i:Issue)-[b:BRIDGE {state:'DISCOVERED'}]->(target) WITH i,b,target LIMIT 1 SET b.state='DRAFTING' RETURN i.uid AS issue_uid,i.title AS issue,b.uid AS bridge_uid,b.kind AS kind,b.note AS note,target.name AS target_name",json!({})).await?;
    let Some(r)=rows.first()else{return Ok(false)};let bridge=r["bridge_uid"].as_str().unwrap_or_default().to_string();let kind=r["kind"].as_str().unwrap_or("");let issue=r["issue"].as_str().unwrap_or("");let note=r["note"].as_str().unwrap_or("");
    write_event(&mut g,"concierge","CLAIMED",&bridge,"DISCOVERED → DRAFTING").await?;
    let draft=if kind=="contact"{llm::draft_followup(note,issue).await.unwrap_or_else(|_|format!("Hi! I remembered our conversation about this issue: {issue}. Would you be open to continuing the discussion?"))}else{String::new()};
    write_query(&mut g,"MATCH ()-[b:BRIDGE {uid:$uid}]->() SET b.draft=$draft,b.state='ACTION_READY',b.seen='false'",json!({"uid":bridge,"draft":draft})).await?;
    write_event(&mut g,"concierge","ACTION_READY",&bridge,"DRAFTING → ACTION_READY").await?;Ok(true)
}
