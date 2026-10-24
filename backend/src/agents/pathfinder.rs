use super::graph;
use crate::{config::Config, events::write_event, graph::{query_rows, run_template, write_query}, templates::{T2_TALKS_FOR_ISSUE,T4_MEMORY_RECALL,T5_ISSUES_FOR_ENCOUNTER}};
use anyhow::Result;
use serde_json::{json, Value};
use tokio::time::{sleep,Duration};
use uuid::Uuid;

pub async fn run(config:Config){let _=recover(&config).await;loop{let worked=match process_issue(&config).await{Ok(v)=>v,Err(e)=>{tracing::warn!(error=%e,"Pathfinder issue failed");false}}||match process_encounter(&config).await{Ok(v)=>v,Err(e)=>{tracing::warn!(error=%e,"Pathfinder encounter failed");false}};if !worked{sleep(Duration::from_millis(500)).await;}}}
async fn recover(c:&Config)->Result<()>{let mut g=graph(c).await?;write_query(&mut g,"MATCH (n:Issue {state:'LINKING'}) SET n.state='CLASSIFIED',n.claimed_by='',n.claimed_at=''",json!({})).await?;write_query(&mut g,"MATCH (n:Encounter {state:'LINKING'}) SET n.state='PARSED',n.claimed_by='',n.claimed_at=''",json!({})).await?;Ok(())}

async fn process_issue(c:&Config)->Result<bool>{
    let mut g=graph(c).await?;
    let rows=query_rows(&mut g,"MATCH (n:Issue {state:'CLASSIFIED'}) WITH n LIMIT 1 SET n.state='LINKING',n.claimed_by='pathfinder',n.claimed_at=timestamp() RETURN n.uid AS uid,n.title AS title",json!({})).await?;
    let Some(row)=rows.first() else{return Ok(false)};let uid=row["uid"].as_str().unwrap_or_default().to_string();let title=row["title"].as_str().unwrap_or_default().to_string();
    write_event(&mut g,"pathfinder","CLAIMED",&uid,"CLASSIFIED → LINKING").await?;
    let t2=run_template(&mut g,"T2",T2_TALKS_FOR_ISSUE,json!({"issue":uid})).await?;
    for r in &t2.rows { let talk=r.get("talk_uid").and_then(Value::as_str).unwrap_or(""); if talk.is_empty(){continue;} let buid=format!("bridge:{}",Uuid::new_v4()); let path=serde_json::to_string(&t2.path.nodes.iter().map(|n|n.uid.clone()).collect::<Vec<_>>())?;
        write_query(&mut g,"MATCH (i:Issue {uid:$issue}),(k:Talk {uid:$talk}) MERGE (i)-[b:BRIDGE {uid:$bridge_uid}]->(k) SET b.kind='talk',b.via_topic=$topic,b.hops=$hops,b.path_uids=$path,b.score='0.8',b.state='DISCOVERED',b.seen='false',b.created_at=timestamp()",json!({"issue":uid,"talk":talk,"bridge_uid":buid,"topic":r.get("topic").and_then(Value::as_str).unwrap_or(""),"hops":t2.hops.to_string(),"path":path})).await?;
    }
    let t4=run_template(&mut g,"T4",T4_MEMORY_RECALL,json!({"issue":uid})).await?;
    for r in &t4.rows {let contact=r.get("contact_uid").and_then(Value::as_str).unwrap_or("");if contact.is_empty(){continue;}let buid=format!("bridge:{}",Uuid::new_v4());let path=serde_json::to_string(&t4.path.nodes.iter().map(|n|n.uid.clone()).collect::<Vec<_>>())?;
        write_query(&mut g,"MATCH (i:Issue {uid:$issue}),(c:Person {uid:$contact}) MERGE (i)-[b:BRIDGE {uid:$bridge_uid}]->(c) SET b.kind='contact',b.via_topic=$topic,b.hops=$hops,b.path_uids=$path,b.score='0.8',b.state='DISCOVERED',b.seen='false',b.created_at=timestamp(),b.note=$note,b.place=$place,b.met_at=$met_at,b.issue_title=$issue_title",json!({"issue":uid,"contact":contact,"bridge_uid":buid,"topic":r.get("topic").and_then(Value::as_str).unwrap_or(""),"hops":t4.hops.to_string(),"path":path,"note":r.get("note").and_then(Value::as_str).unwrap_or(""),"place":r.get("place").and_then(Value::as_str).unwrap_or(""),"met_at":r.get("met_at").and_then(Value::as_str).unwrap_or(""),"issue_title":title})).await?;
    }
    write_query(&mut g,"MATCH (n:Issue {uid:$uid}) SET n.state='LINKED',n.claimed_by='',n.claimed_at=''",json!({"uid":uid})).await?;
    write_event(&mut g,"pathfinder","LINKED",&uid,"LINKING → LINKED").await?;Ok(true)
}

async fn process_encounter(c:&Config)->Result<bool>{
    let mut g=graph(c).await?;let rows=query_rows(&mut g,"MATCH (n:Encounter {state:'PARSED'}) WITH n LIMIT 1 SET n.state='LINKING',n.claimed_by='pathfinder',n.claimed_at=timestamp() RETURN n.uid AS uid",json!({})).await?;
    let Some(row)=rows.first()else{return Ok(false)};let uid=row["uid"].as_str().unwrap_or_default().to_string();write_event(&mut g,"pathfinder","CLAIMED",&uid,"PARSED → LINKING").await?;
    let env=run_template(&mut g,"T5",T5_ISSUES_FOR_ENCOUNTER,json!({"encounter":uid})).await?;
    for r in &env.rows{let issue=r.get("issue_uid").and_then(Value::as_str).unwrap_or("");if issue.is_empty(){continue;}let contact=r.get("contact_uid").and_then(Value::as_str).unwrap_or("");let buid=format!("bridge:{}",Uuid::new_v4());let path=serde_json::to_string(&env.path.nodes.iter().map(|n|n.uid.clone()).collect::<Vec<_>>())?;
        write_query(&mut g,"MATCH (i:Issue {uid:$issue}),(c:Person {uid:$contact}) MERGE (i)-[b:BRIDGE {uid:$bridge_uid}]->(c) SET b.kind='contact',b.via_topic=$topic,b.hops=$hops,b.path_uids=$path,b.score='0.8',b.state='DISCOVERED',b.seen='false',b.created_at=timestamp(),b.note=$note,b.place=$place,b.met_at=$met_at,b.issue_title=$issue_title",json!({"issue":issue,"contact":contact,"bridge_uid":buid,"topic":r.get("topic").and_then(Value::as_str).unwrap_or(""),"hops":env.hops.to_string(),"path":path,"note":"","place":"","met_at":"","issue_title":r.get("issue").and_then(Value::as_str).unwrap_or("")})).await?;
    }
    write_query(&mut g,"MATCH (n:Encounter {uid:$uid}) SET n.state='LINKED',n.claimed_by='',n.claimed_at=''",json!({"uid":uid})).await?;write_event(&mut g,"pathfinder","LINKED",&uid,"LINKING → LINKED").await?;Ok(true)
}
