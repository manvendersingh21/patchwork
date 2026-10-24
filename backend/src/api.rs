use crate::{config::Config, github::GithubClient, graph::{connect,query_rows,run_template,write_query}, llm, templates::{DayMention,T1_TALKS_FOR_MY_ISSUES,T3_PEOPLE_TO_MEET,T4_MEMORY_RECALL,T6_TALKS_BY_TOPIC,T7_CONFERENCE_DAYS, day_from_question, normalize_day, normalize_after, tokenize}};
use axum::{extract::{Query,Path,State}, http::StatusCode, response::IntoResponse, routing::{get,post}, Json, Router};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json,Value};
use std::{collections::HashMap,sync::Arc};
use uuid::Uuid;

#[derive(Clone)] pub struct AppState { pub config:Config, pub github:Option<GithubClient> }
pub fn router(state:AppState)->Router { Router::new()
    .route("/api/health",get(health)).route("/api/sync",post(sync)).route("/api/encounters",post(encounter))
    .route("/api/ask",post(ask)).route("/api/graph",get(graph_view)).route("/api/events",get(events))
    .route("/api/alerts",get(alerts)).route("/api/alerts/{bridge_uid}/seen",post(seen)).route("/api/alerts/{bridge_uid}/send",post(send_followup))
    .route("/api/topics",get(topics)).route("/api/demo/inject-issue",post(inject_issue)).with_state(Arc::new(state)) }

async fn health(State(s):State<Arc<AppState>>)->Json<Value>{
    let falkordb=connect(&s.config.falkordb_host,s.config.falkordb_port,&s.config.graph_name).await.is_ok();
    let openai=std::env::var("OPENAI_API_KEY").map(|v|!v.is_empty()).unwrap_or(false);
    let github=s.github.as_ref().map(GithubClient::authenticated).unwrap_or(false);
    let counts=if let Ok(mut g)=connect(&s.config.falkordb_host,s.config.falkordb_port,&s.config.graph_name).await{
        let rows=query_rows(&mut g,"MATCH (n) RETURN labels(n)[0] AS label,count(n) AS count",json!({})).await.unwrap_or_default();
        rows.into_iter().filter_map(|r|Some((r.get("label")?.as_str()?.to_string(),r.get("count")?.clone()))).collect::<serde_json::Map<String,Value>>()
    }else{serde_json::Map::new()};
    Json(json!({"falkordb":falkordb,"openai":openai,"github":github,"counts":counts}))
}

#[derive(Deserialize)]struct SyncBody{handle:String,kind:String}
// GitHub usernames: 1-39 chars of [A-Za-z0-9-], no leading/trailing hyphen.
fn valid_handle(h:&str)->bool{(1..=39).contains(&h.len())&&h.bytes().next().is_some_and(|c|c.is_ascii_alphanumeric())&&h.bytes().next_back().is_some_and(|c|c.is_ascii_alphanumeric())&&h.bytes().all(|c|c.is_ascii_alphanumeric()||c==b'-')}
async fn sync(State(s):State<Arc<AppState>>,Json(b):Json<SyncBody>)->impl IntoResponse{
    let handle=b.handle.trim_start_matches('@').to_string();
    if !valid_handle(&handle)||(b.kind!="self"&&b.kind!="contact"){return (StatusCode::BAD_REQUEST,Json(json!({"error":"invalid GitHub handle"})))}
    let uid=format!("person:gh:{handle}");
    match connect(&s.config.falkordb_host,s.config.falkordb_port,&s.config.graph_name).await{
        Ok(mut g)=>{let q="MERGE (p:Person {uid:$uid}) SET p.handle=$handle,p.kind=$kind,p.state='SYNC_REQUESTED'";
            match write_query(&mut g,q,json!({"uid":uid,"handle":handle,"kind":b.kind})).await{Ok(())=>(StatusCode::OK,Json(json!({"person_uid":uid}))),Err(e)=>err(e)}},Err(e)=>err(e)}
}

#[derive(Deserialize)]struct EncounterBody{note:String,place:String}
async fn encounter(State(s):State<Arc<AppState>>,Json(b):Json<EncounterBody>)->impl IntoResponse{
    let uid=format!("enc:{}",Uuid::new_v4());let self_uid=format!("person:gh:{}",s.config.presenter_handle.trim_start_matches('@'));
    match connect(&s.config.falkordb_host,s.config.falkordb_port,&s.config.graph_name).await{Ok(mut g)=>{
        let q="MERGE (p:Person {uid:$person}) ON CREATE SET p.kind='self' MERGE (e:Encounter {uid:$uid}) SET e.note=$note,e.place=$place,e.met_at=$met_at,e.state='NEW' MERGE (p)-[:LOGGED]->(e)";
        match write_query(&mut g,q,json!({"person":self_uid,"uid":uid,"note":b.note,"place":b.place,"met_at":Utc::now().to_rfc3339()})).await{Ok(())=>(StatusCode::OK,Json(json!({"encounter_uid":uid}))),Err(e)=>err(e)}},Err(e)=>err(e)}
}

#[derive(Deserialize)]struct AskBody{question:String,person_uid:Option<String>}
async fn ask(State(s):State<Arc<AppState>>,Json(b):Json<AskBody>)->impl IntoResponse{
    // L4 fix: read the day mention from the question itself, not from the model.
    let mention=day_from_question(&b.question);
    if let DayMention::Invalid(raw)=&mention{
        let Ok(mut g)=connect(&s.config.falkordb_host,s.config.falkordb_port,&s.config.graph_name).await else{return Json(json!({"error":"FalkorDB unavailable"})).into_response()};
        let Ok(mut envelope)=run_template(&mut g,"T7",T7_CONFERENCE_DAYS,json!({})).await else{return Json(json!({"error":"Graph query failed"})).into_response()};
        let days=envelope.rows.iter().filter_map(|r|r.get("day").and_then(Value::as_str).map(str::to_string)).collect::<Vec<_>>().join(" and ");
        envelope.params=json!({"mentioned":raw});
        let mut output=serde_json::to_value(envelope).unwrap_or(json!({}));output["answer"]=json!(format!("The conference covers only {days} — there is no {raw}."));
        return Json(output).into_response();
    }
    let key=match std::env::var("OPENAI_API_KEY"){Ok(k)=>k,Err(_)=>return (StatusCode::SERVICE_UNAVAILABLE,Json(json!({"error":"OPENAI_API_KEY is not configured"}))).into_response()};
    let person=b.person_uid.unwrap_or_else(||format!("person:gh:{}",s.config.presenter_handle));
    let model=std::env::var("OPENAI_MODEL").unwrap_or_else(|_|"gpt-4o-mini".into());
    let tools=json!([
        {"type":"function","function":{"name":"talks_for_my_issues","description":"Find talks relevant to the presenter's open repo issues","parameters":{"type":"object","properties":{"day":{"type":"string","description":"Conference day as ISO date: 2026-10-28 (day 1, Wed) or 2026-10-29 (day 2, Thu). Empty means any day."},"after":{"type":"string","description":"Earliest talk start time, 24h HH:MM, e.g. 10:00. Empty means any time."}},"required":[],"additionalProperties":false}}},
        {"type":"function","function":{"name":"people_to_meet","description":"Find relevant speakers not met yet","parameters":{"type":"object","properties":{},"required":[],"additionalProperties":false}}},
        {"type":"function","function":{"name":"memory_recall","description":"Find a person met who can help with an issue","parameters":{"type":"object","properties":{"issue":{"type":"string"}},"required":["issue"],"additionalProperties":false}}},
        {"type":"function","function":{"name":"talks_by_topic","description":"Find talks by topics in the question","parameters":{"type":"object","properties":{"day":{"type":"string","description":"Conference day as ISO date: 2026-10-28 or 2026-10-29. Empty means any day."},"after":{"type":"string","description":"Earliest talk start time, 24h HH:MM, e.g. 10:00. Empty means any time."}},"required":[],"additionalProperties":false}}}
    ]);
    let response=reqwest::Client::new().post("https://api.openai.com/v1/chat/completions").bearer_auth(key).json(&json!({"model":model,"messages":[{"role":"system","content":"Choose exactly one graph tool if it can answer. Never answer directly."},{"role":"user","content":b.question}],"tools":tools,"tool_choice":"auto"})).send().await;
    let Ok(response)=response else{return (StatusCode::BAD_GATEWAY,Json(json!({"error":"OpenAI request failed"}))).into_response()};
    let Ok(resp)=response.error_for_status() else{return (StatusCode::BAD_GATEWAY,Json(json!({"error":"OpenAI request failed"}))).into_response()};
    let Ok(resp)=resp.json::<Value>().await else{return (StatusCode::BAD_GATEWAY,Json(json!({"error":"OpenAI response invalid"}))).into_response()};
    let call=&resp["choices"][0]["message"]["tool_calls"][0];let name=call["function"]["name"].as_str().unwrap_or("");
    if name.is_empty(){return Json(json!({"answer":"I can only answer what the graph can trace","quick_questions":["Which talks after lunch help with my open issues?","Who should I meet?"],"rows":[],"path":{"nodes":[],"edges":[]}})).into_response()}
    let args:Value=serde_json::from_str(call["function"]["arguments"].as_str().unwrap_or("{}")).unwrap_or(json!({}));
    // A valid day mention in the question overrides whatever day the model passed.
    let day_arg=match &mention{DayMention::Valid(iso)=>iso.clone(),_=>args["day"].as_str().unwrap_or("").to_string()};
    // Time-of-day words become hard bounds; zero-padded HH:MM compares lexically.
    let toks=tokenize(&b.question);let mut after=normalize_after(args["after"].as_str().unwrap_or(""));let mut before=String::new();
    if toks.iter().any(|t|t=="morning"){before="12:00".into();if after.as_str()>="12:00"{after=String::new()}}
    if toks.iter().any(|t|t=="afternoon"){after=after.max("12:00".into())}
    if toks.iter().any(|t|t=="evening"){after=after.max("17:00".into())}
    let params=match name{
        "talks_for_my_issues"=>json!({"person":person,"day":normalize_day(&day_arg),"after":after,"before":before}),
        "people_to_meet"=>json!({"person":person}),
        "memory_recall"=>json!({"issue":args["issue"].as_str().unwrap_or("")}),
        "talks_by_topic"=>{let hits=llm::classify(&[b.question.clone()]).await.unwrap_or_default().into_iter().next().unwrap_or_default();json!({"topics":hits.into_iter().map(|h|h.slug).collect::<Vec<_>>().join(","),"day":normalize_day(&day_arg),"after":after,"before":before})},
        _=>return Json(json!({"answer":"I can only answer what the graph can trace","rows":[],"path":{"nodes":[],"edges":[]}})).into_response()
    };
    let (template,cypher)=match name{"talks_for_my_issues"=>("T1",T1_TALKS_FOR_MY_ISSUES),"people_to_meet"=>("T3",T3_PEOPLE_TO_MEET),"memory_recall"=>("T4",T4_MEMORY_RECALL),_=>("T6",T6_TALKS_BY_TOPIC)};
    let Ok(mut g)=connect(&s.config.falkordb_host,s.config.falkordb_port,&s.config.graph_name).await else{return Json(json!({"error":"FalkorDB unavailable"})).into_response()};
    let Ok(mut envelope)=run_template(&mut g,template,cypher,params.clone()).await else{return Json(json!({"error":"Graph query failed"})).into_response()};
    if name=="memory_recall"{envelope.params["issue"] = args["issue"].clone();}
    let rows=serde_json::to_value(&envelope.rows).unwrap_or(Value::Null);let answer=llm::answer(&b.question,&rows).await.unwrap_or_else(|_|"I can only answer what the graph can trace".into());
    let mut output=serde_json::to_value(envelope).unwrap_or(json!({}));output["answer"]=json!(answer);Json(output).into_response()
}

async fn graph_view(State(s):State<Arc<AppState>>,Query(q):Query<HashMap<String,String>>)->Json<Value>{
    let Ok(mut g)=connect(&s.config.falkordb_host,s.config.falkordb_port,&s.config.graph_name).await else{return Json(json!({"nodes":[],"edges":[]}))};
    let depth=q.get("depth").and_then(|v|v.parse::<u8>().ok()).unwrap_or(2).clamp(1,3);
    let cypher=if q.get("scope").map(String::as_str)==Some("around")&&q.contains_key("uid") {format!("MATCH (start {{uid:$uid}})-[*0..{depth}]-(n) WITH DISTINCT n LIMIT 400 OPTIONAL MATCH (n)-[r]-(m) RETURN n.uid AS uid,labels(n)[0] AS label,coalesce(n.name,n.title,n.full_name,'') AS name,coalesce(n.state,'') AS state,type(r) AS rel,m.uid AS other")}else{"MATCH (n) WITH n LIMIT 400 OPTIONAL MATCH (n)-[r]-(m) RETURN n.uid AS uid,labels(n)[0] AS label,coalesce(n.name,n.title,n.full_name,'') AS name,coalesce(n.state,'') AS state,type(r) AS rel,m.uid AS other".to_string()};
    let rows=query_rows(&mut g,&cypher,json!({"uid":q.get("uid").cloned().unwrap_or_default()})).await.unwrap_or_default();let mut nodes=serde_json::Map::new();let mut edges=Vec::new();
    for r in rows{let uid=r.get("uid").and_then(Value::as_str).unwrap_or("");if uid.is_empty(){continue}nodes.entry(uid.to_string()).or_insert(json!({"uid":uid,"label":r.get("label"),"name":r.get("name"),"state":r.get("state")}));if let(Some(rel),Some(other))=(r.get("rel").and_then(Value::as_str),r.get("other").and_then(Value::as_str)){edges.push(json!({"type":rel,"src_uid":uid,"dst_uid":other}));}}
    Json(json!({"nodes":nodes.into_values().collect::<Vec<_>>(),"edges":edges}))
}

async fn events(State(s):State<Arc<AppState>>,Query(q):Query<HashMap<String,String>>)->Json<Value>{let since=q.get("since").and_then(|v|v.parse::<i64>().ok()).unwrap_or(0).to_string();let Ok(mut g)=connect(&s.config.falkordb_host,s.config.falkordb_port,&s.config.graph_name).await else{return Json(json!([]))};let rows=query_rows(&mut g,"MATCH (e:AgentEvent) WHERE e.ts >= toInteger($since) RETURN e.uid AS uid,e.ts AS ts,e.agent AS agent,e.action AS action,e.target AS target,e.detail AS detail ORDER BY ts LIMIT 100",json!({"since":since})).await.unwrap_or_default();Json(json!(rows))}
async fn alerts(State(s):State<Arc<AppState>>)->Json<Value>{let Ok(mut g)=connect(&s.config.falkordb_host,s.config.falkordb_port,&s.config.graph_name).await else{return Json(json!([]))};let rows=query_rows(&mut g,"MATCH (i:Issue)-[b:BRIDGE {kind:'contact',state:'ACTION_READY',seen:'false'}]->(c:Person) RETURN b.uid AS bridge_uid,i.title AS issue,c.name AS contact,b.note AS note,b.place AS place,b.met_at AS met_at,b.draft AS draft,b.path_uids AS path,b.hops AS hops ORDER BY b.created_at DESC LIMIT 20",json!({})).await.unwrap_or_default();Json(json!(rows))}
#[derive(Deserialize)] struct SendBody { message:String, #[serde(default)] confirm:bool }

/// Send a bridge's follow-up as a LinkedIn DM. The recipient is the contact stored on the bridge,
/// never caller-supplied; `confirm` must be true; a bridge is sent at most once.
async fn send_followup(State(s):State<Arc<AppState>>,Path(uid):Path<String>,Json(body):Json<SendBody>)->(StatusCode,Json<Value>){
    let fail=|code:StatusCode,msg:&str|(code,Json(json!({"ok":false,"error":msg})));
    if !body.confirm{return fail(StatusCode::BAD_REQUEST,"confirm must be true");}
    let message=body.message.trim().to_string();
    if message.is_empty()||message.chars().count()>2000{return fail(StatusCode::BAD_REQUEST,"message must be 1-2000 characters");}
    let Some(li)=crate::linkedin::LinkedIn::from_env() else{return fail(StatusCode::SERVICE_UNAVAILABLE,"LINKEDIN_MCP_URL is not set");};
    let Ok(mut g)=connect(&s.config.falkordb_host,s.config.falkordb_port,&s.config.graph_name).await else{return fail(StatusCode::SERVICE_UNAVAILABLE,"graph unavailable");};
    let rows=query_rows(&mut g,"MATCH (:Issue)-[b:BRIDGE {uid:$uid, kind:'contact'}]->(c:Person) RETURN c.linkedin AS linkedin, c.name AS name, coalesce(b.sent,'false') AS sent",json!({"uid":uid})).await.unwrap_or_default();
    let Some(row)=rows.first() else{return fail(StatusCode::NOT_FOUND,"no contact bridge with that uid");};
    if row["sent"].as_str()==Some("true"){return fail(StatusCode::CONFLICT,"this follow-up was already sent");}
    let target=row["linkedin"].as_str().unwrap_or("").to_string();
    if target.is_empty(){return fail(StatusCode::UNPROCESSABLE_ENTITY,"this contact has no LinkedIn profile on file");}
    // Claim before sending so a double click cannot send twice.
    if write_query(&mut g,"MATCH ()-[b:BRIDGE {uid:$uid}]->() SET b.sent='true'",json!({"uid":uid})).await.is_err(){return fail(StatusCode::SERVICE_UNAVAILABLE,"graph unavailable");}
    match li.call("send_message",json!({"linkedin_username":target,"message":message,"confirm_send":true})).await{
        Ok(result)=>{
            let _=write_query(&mut g,"MATCH ()-[b:BRIDGE {uid:$uid}]->() SET b.seen='true', b.state='SENT', b.sent_at=$at, b.sent_text=$text",json!({"uid":uid,"at":Utc::now().to_rfc3339(),"text":message})).await;
            (StatusCode::OK,Json(json!({"ok":true,"to":row["name"],"result":result})))
        }
        Err(e)=>{
            let _=write_query(&mut g,"MATCH ()-[b:BRIDGE {uid:$uid}]->() SET b.sent='false'",json!({"uid":uid})).await;
            fail(StatusCode::BAD_GATEWAY,&e.to_string())
        }
    }
}

async fn seen(State(s):State<Arc<AppState>>,Path(uid):Path<String>)->Json<Value>{if let Ok(mut g)=connect(&s.config.falkordb_host,s.config.falkordb_port,&s.config.graph_name).await{let _=write_query(&mut g,"MATCH ()-[b:BRIDGE {uid:$uid}]->() SET b.seen='true'",json!({"uid":uid})).await;}Json(json!({"ok":true}))}
async fn topics()->Json<Value>{let v:Value=serde_json::from_str(include_str!("../../data/topics.json")).unwrap_or(json!({"topics":[]}));Json(Value::Array(v["topics"].as_array().into_iter().flatten().map(|t|json!({"slug":t["slug"],"name":t["name"]})).collect()))}
#[derive(Deserialize)]struct InjectBody{title:String,body:String}
async fn inject_issue(State(s):State<Arc<AppState>>,Json(b):Json<InjectBody>)->impl IntoResponse{let uid=format!("issue:{}#demo-{}",s.config.demo_repo,Uuid::new_v4());let repo=format!("repo:{}",s.config.demo_repo);match connect(&s.config.falkordb_host,s.config.falkordb_port,&s.config.graph_name).await{Ok(mut g)=>match write_query(&mut g,"MERGE (r:Repo {uid:$repo}) MERGE (i:Issue {uid:$uid}) SET i.title=$title,i.body=$body,i.gh_state='open',i.state='NEW',i.created_at=$created MERGE (r)-[:HAS_ISSUE]->(i)",json!({"repo":repo,"uid":uid,"title":b.title,"body":b.body,"created":Utc::now().to_rfc3339()})).await{Ok(())=>(StatusCode::OK,Json(json!({"issue_uid":uid}))),Err(e)=>err(e)},Err(e)=>err(e)}}

fn err(e:anyhow::Error)->(StatusCode,Json<Value>){(StatusCode::INTERNAL_SERVER_ERROR,Json(json!({"error":e.to_string()})))}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_rules() {
        assert!(valid_handle("a"));
        assert!(valid_handle("Ada-Lovelace-99"));
        assert!(valid_handle(&"a".repeat(39)));
        assert!(!valid_handle(""));
        assert!(!valid_handle(&"a".repeat(40)));
        assert!(!valid_handle("-ada"));
        assert!(!valid_handle("ada-"));
        assert!(!valid_handle("not_a_handle"));
        assert!(!valid_handle("uniçode"));
        assert!(!valid_handle("x}) DETACH DELETE n//repos"));
    }
}
