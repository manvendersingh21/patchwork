use super::graph;
use crate::{config::Config, events::write_event, github::GithubClient, graph::{query_rows, write_query}, llm};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use tokio::time::{sleep, Duration};

pub async fn run(config: Config) {
    let _ = recover(&config).await;
    let github = GithubClient::from_env().ok();
    loop {
        let worked = if let Some(client) = github.as_ref() { match process_sync(&config, client).await { Ok(v)=>v,Err(e)=>{tracing::warn!(error=%e,"Cartographer sync failed");false} } } else { false }
            || match process_encounter(&config).await { Ok(v)=>v,Err(e)=>{tracing::warn!(error=%e,"Cartographer encounter failed");false} }
            || match process_issue(&config).await { Ok(v)=>v,Err(e)=>{tracing::warn!(error=%e,"Cartographer issue failed");false} }
            || match watch(&config, github.as_ref()).await { Ok(v)=>v,Err(e)=>{tracing::warn!(error=%e,"Cartographer watcher failed");false} };
        if !worked { sleep(Duration::from_millis(500)).await; }
    }
}

async fn recover(c: &Config) -> Result<()> {
    let mut g=graph(c).await?;
    write_query(&mut g,"MATCH (n:Person {state:'SYNCING'}) SET n.state='SYNC_REQUESTED', n.claimed_by='', n.claimed_at=''",json!({})).await?;
    write_query(&mut g,"MATCH (n:Issue {state:'CLASSIFYING'}) SET n.state='NEW', n.claimed_by='', n.claimed_at=''",json!({})).await?;
    write_query(&mut g,"MATCH (n:Encounter {state:'PARSING'}) SET n.state='NEW', n.claimed_by='', n.claimed_at=''",json!({})).await?;
    Ok(())
}

async fn process_issue(c:&Config)->Result<bool>{
    let mut g=graph(c).await?;
    let rows=query_rows(&mut g,"MATCH (n:Issue {state:'NEW'}) WITH n LIMIT 1 SET n.state='CLASSIFYING', n.claimed_by='cartographer', n.claimed_at=timestamp() RETURN n.uid AS uid,n.title AS title,n.body AS body",json!({})).await?;
    let Some(row)=rows.first() else{return Ok(false)};
    let uid=row["uid"].as_str().unwrap_or_default().to_string(); let title=row["title"].as_str().unwrap_or_default(); let body=row["body"].as_str().unwrap_or_default();
    write_event(&mut g,"cartographer","CLAIMED",&uid,"NEW → CLASSIFYING").await?;
    let hits=llm::classify(&[format!("{title}\n{body}")]).await?.into_iter().next().unwrap_or_default();
    for hit in hits { write_query(&mut g,"MATCH (i:Issue {uid:$uid}) MERGE (t:Topic {uid:$topic_uid}) ON CREATE SET t.slug=$slug, t.name=$slug MERGE (i)-[r:ABOUT]->(t) SET r.confidence=$confidence, r.evidence=$evidence",json!({"uid":uid,"topic_uid":format!("topic:{}",hit.slug),"slug":hit.slug,"confidence":hit.confidence.to_string(),"evidence":hit.evidence})).await?; }
    write_query(&mut g,"MATCH (n:Issue {uid:$uid}) SET n.state='CLASSIFIED', n.claimed_by='', n.claimed_at=''",json!({"uid":uid})).await?;
    write_event(&mut g,"cartographer","CLASSIFIED",&uid,"CLASSIFYING → CLASSIFIED").await?; Ok(true)
}

async fn process_encounter(c:&Config)->Result<bool>{
    let mut g=graph(c).await?;
    let rows=query_rows(&mut g,"MATCH (n:Encounter {state:'NEW'}) WITH n LIMIT 1 SET n.state='PARSING',n.claimed_by='cartographer',n.claimed_at=timestamp() RETURN n.uid AS uid,n.note AS note",json!({})).await?;
    let Some(row)=rows.first() else{return Ok(false)}; let uid=row["uid"].as_str().unwrap_or_default().to_string(); let note=row["note"].as_str().unwrap_or_default();
    write_event(&mut g,"cartographer","CLAIMED",&uid,"NEW → PARSING").await?;
    let parsed=llm::parse_note(note).await?;
    for p in parsed["people"].as_array().into_iter().flatten(){
        let name=p["name"].as_str().unwrap_or(""); if name.is_empty(){continue}
        let handle=p["handle"].as_str().unwrap_or(""); let person_uid=if handle.is_empty(){format!("person:name:{}",name.to_lowercase().replace(' ',"-"))}else{format!("person:gh:{}",handle.trim_start_matches('@'))};
        write_query(&mut g,"MATCH (e:Encounter {uid:$uid}) MERGE (c:Person {uid:$person_uid}) ON CREATE SET c.name=$name,c.handle=$handle,c.company=$company,c.kind='contact' MERGE (e)-[:MET]->(c)",json!({"uid":uid,"person_uid":person_uid,"name":name,"handle":handle,"company":p["company"].as_str().unwrap_or("")})).await?;
    }
    for slug in parsed["topics"].as_array().into_iter().flatten().filter_map(Value::as_str){
        write_query(&mut g,"MATCH (e:Encounter {uid:$uid}) MERGE (t:Topic {uid:$topic_uid}) ON CREATE SET t.slug=$slug,t.name=$slug MERGE (e)-[r:DISCUSSED]->(t) SET r.confidence='0.8',r.evidence=$note",json!({"uid":uid,"topic_uid":format!("topic:{slug}"),"slug":slug,"note":note})).await?;
    }
    write_query(&mut g,"MATCH (e:Encounter {uid:$uid}) SET e.state='PARSED',e.claimed_by='',e.claimed_at=''",json!({"uid":uid})).await?;
    write_event(&mut g,"cartographer","PARSED",&uid,"PARSING → PARSED").await?; Ok(true)
}

async fn process_sync(c:&Config,gh:&GithubClient)->Result<bool>{
    let mut g=graph(c).await?;
    let rows=query_rows(&mut g,"MATCH (n:Person {state:'SYNC_REQUESTED'}) WITH n LIMIT 1 SET n.state='SYNCING',n.claimed_by='cartographer',n.claimed_at=timestamp() RETURN n.uid AS uid,n.handle AS handle,n.kind AS kind",json!({})).await?;
    let Some(row)=rows.first() else{return Ok(false)}; let uid=row["uid"].as_str().unwrap_or_default().to_string(); let handle=row["handle"].as_str().unwrap_or_default().to_string();
    write_event(&mut g,"cartographer","CLAIMED",&uid,"SYNC_REQUESTED → SYNCING").await?;
    let demo=if c.demo_repo.is_empty(){None}else{Some(c.demo_repo.as_str())};
    let repos=gh.list_top_repos(&handle,demo).await?;
    for repo in repos {
        write_query(&mut g,"MATCH (p:Person {uid:$person}) MERGE (r:Repo {uid:$repo_uid}) SET r.full_name=$full_name,r.url=$url,r.description=$description,r.stars=$stars MERGE (p)-[:MAINTAINS]->(r)",json!({"person":uid,"repo_uid":format!("repo:{}",repo.full_name),"full_name":repo.full_name,"url":repo.html_url,"description":repo.description.unwrap_or_default(),"stars":repo.stargazers_count.to_string()})).await?;
        for issue in gh.open_issues(&repo.full_name).await? {
            write_query(&mut g,"MATCH (r:Repo {uid:$repo_uid}) MERGE (i:Issue {uid:$issue_uid}) ON CREATE SET i.number=$number,i.title=$title,i.body=$body,i.url=$url,i.gh_state='open',i.created_at=$created,i.state='NEW' MERGE (r)-[:HAS_ISSUE]->(i)",json!({"repo_uid":format!("repo:{}",repo.full_name),"issue_uid":format!("issue:{}#{}",repo.full_name,issue.number),"number":issue.number.to_string(),"title":issue.title,"body":issue.body.unwrap_or_default().chars().take(2000).collect::<String>(),"url":issue.html_url,"created":issue.created_at})).await?;
        }
    }
    for repo in &c.tracked_repos { write_query(&mut g,"MATCH (p:Person {uid:$person}) MERGE (r:Repo {uid:$repo_uid}) SET r.full_name=$full_name MERGE (p)-[:TRACKS]->(r)",json!({"person":uid,"repo_uid":format!("repo:{repo}"),"full_name":repo})).await?; }
    write_query(&mut g,"MATCH (p:Person {uid:$uid}) SET p.state='SYNCED',p.claimed_by='',p.claimed_at=''",json!({"uid":uid})).await?;
    write_event(&mut g,"cartographer","SYNCED",&uid,"SYNCING → SYNCED").await?; Ok(true)
}

async fn watch(c:&Config,gh:Option<&GithubClient>)->Result<bool>{
    let Some(gh)=gh else{return Ok(false)};
    let mut g=graph(c).await?; let repo_uid=format!("repo:{}",c.demo_repo);
    let rows=query_rows(&mut g,"MATCH (r:Repo {uid:$uid}) RETURN r.watch_since AS cursor",json!({"uid":repo_uid})).await?;
    let Some(v)=rows.first().and_then(|r|r.get("cursor")).and_then(Value::as_str) else{return Ok(false)};
    let cursor=DateTime::parse_from_rfc3339(v).map(|x|x.with_timezone(&Utc)).unwrap_or_else(|_|Utc::now());
    let issues=gh.issues_created_after(&c.demo_repo,cursor).await?;
    let found = !issues.is_empty();
    for i in issues { write_query(&mut g,"MERGE (r:Repo {uid:$repo_uid}) MERGE (i:Issue {uid:$issue_uid}) ON CREATE SET i.number=$number,i.title=$title,i.body=$body,i.url=$url,i.gh_state=$gh_state,i.created_at=$created,i.state='NEW' MERGE (r)-[:HAS_ISSUE]->(i)",json!({"repo_uid":repo_uid,"issue_uid":format!("issue:{}#{}",c.demo_repo,i.number),"number":i.number.to_string(),"title":i.title,"body":i.body.unwrap_or_default().chars().take(2000).collect::<String>(),"url":i.html_url,"gh_state":i.state,"created":i.created_at})).await?; }
    write_query(&mut g,"MATCH (r:Repo {uid:$uid}) SET r.watch_since=$cursor",json!({"uid":repo_uid,"cursor":Utc::now().to_rfc3339()})).await?;
    Ok(found)
}
