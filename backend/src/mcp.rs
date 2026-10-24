//! MCP server (stdio) exposing the verified read templates as typed tools.
//! Clients pass typed parameters only. The server never accepts Cypher from a client.
use crate::graph::{connect, query_rows, run_template, write_query};
use crate::templates::*;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig},
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use falkordb::AsyncGraph;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TalksForMyIssuesArgs {
    /// Person uid, e.g. person:gh:<handle>. Defaults to the presenter.
    pub person: String,
    /// Day filter as YYYY-MM-DD (2026-10-28 or 2026-10-29; "day 1"/"day 2" map too), or empty for any day. Other values are rejected.
    #[serde(default)]
    pub day: String,
    /// Earliest start time HH:MM (24h, e.g. 10:30; "9am" maps too), or empty for any time. Invalid times are rejected.
    #[serde(default)]
    pub after: String,
    /// Latest start time HH:MM (24h), or empty for any time. Invalid times are rejected.
    #[serde(default)]
    pub before: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueArgs {
    /// Issue uid, e.g. issue:<owner>/<repo>#<number>.
    pub issue: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TalksByTopicArgs {
    /// Comma-separated topic slugs from list_topics, e.g. "mcp-auth,agentic-security". Unknown slugs are rejected.
    pub topics: String,
    #[serde(default)]
    pub day: String,
    #[serde(default)]
    pub after: String,
    #[serde(default)]
    pub before: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PeopleToMeetArgs {
    pub person: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LogEncounterArgs {
    /// Person uid of the user logging the conversation, e.g. person:gh:<handle>.
    pub person: String,
    /// The note as the user wrote it.
    pub note: String,
    /// Where it happened, e.g. "snack table".
    #[serde(default)]
    pub place: String,
    /// Name of the person met.
    pub met_name: String,
    /// Their GitHub handle, if the user knows it. Leave empty otherwise; never guess.
    #[serde(default)]
    pub met_handle: String,
    /// Topic slugs discussed, chosen from list_topics. Unknown slugs are rejected.
    pub topics: Vec<String>,
    /// Where the conversation lives, e.g. a LinkedIn messaging thread URL. Empty for in-person chats.
    #[serde(default)]
    pub source_url: String,
    /// Their LinkedIn profile slug (linkedin.com/in/<slug>), if known from the thread. Leave empty otherwise; never guess.
    #[serde(default)]
    pub met_linkedin: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LogLearningArgs {
    /// Who learned it: their GitHub handle (e.g. "octocat") or plain name (e.g. "Ada Lovelace").
    pub person: String,
    /// The lesson itself, one or two sentences the user wants to keep.
    pub learned: String,
    /// Topic slugs the lesson is about, chosen from list_topics. Unknown slugs are rejected.
    pub topics: Vec<String>,
    /// Where it came from, e.g. "photo of a slide", "hallway chat".
    #[serde(default)]
    pub source: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LogIssueArgs {
    /// Whose notes repo this lands on: GitHub handle or plain name.
    pub person: String,
    /// One-line summary of the open problem or question.
    pub title: String,
    /// The full question or problem as the user described it.
    pub body: String,
    /// Topic slugs from list_topics. Unknown slugs are rejected.
    pub topics: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecallLearningsArgs {
    /// Who to recall for: GitHub handle or plain name.
    pub person: String,
    /// Optional topic slug from list_topics; empty for all topics.
    #[serde(default)]
    pub topic: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NoArgs {}

#[derive(Clone)]
pub struct PatchworkMcp {
    graph: Arc<Mutex<AsyncGraph>>,
    /// Set when this connection came in on a /share/<token>/mcp URL; None for the owner.
    policy: Option<crate::share::SharePolicy>,
    #[allow(dead_code)]
    tool_router: ToolRouter<PatchworkMcp>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateShareArgs {
    /// Short name for the teammate or group, e.g. "team-berlin".
    pub label: String,
    /// Tools the share may call, from: talks_for_my_issues, talks_for_issue, people_to_meet, memory_recall, talks_by_topic, list_topics.
    pub tools: Vec<String>,
    /// Fields to hide from results, from: note, place.
    #[serde(default)]
    pub redact: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RevokeShareArgs {
    /// Label of the share to revoke, as shown by list_shares.
    pub label: String,
}

/// Full URL a teammate pastes into their MCP client.
fn share_url(token: &str) -> String {
    let base = match std::env::var("MCP_PUBLIC_URL") {
        Ok(u) if !u.trim().is_empty() => u.trim().trim_end_matches('/').to_string(),
        _ => format!("http://{}", std::env::var("MCP_HTTP_ADDR").unwrap_or_else(|_| "127.0.0.1:8765".into())),
    };
    format!("{base}/share/{token}/mcp")
}

impl PatchworkMcp {
    fn owner_only(&self) -> Result<(), McpError> {
        if self.policy.is_some() {
            return Err(McpError::invalid_request("owner-only tool", None));
        }
        Ok(())
    }

    pub async fn new(host: &str, port: u16, graph_name: &str) -> anyhow::Result<Self> {
        let g = connect(host, port, graph_name).await?;
        Ok(Self { graph: Arc::new(Mutex::new(g)), policy: None, tool_router: Self::tool_router() })
    }

    /// Same graph, restricted to one share's policy.
    pub fn scoped(&self, policy: crate::share::SharePolicy) -> Self {
        Self { policy: Some(policy), ..self.clone() }
    }

    /// Look up a share token. Used by the HTTP gate before any tool runs.
    pub fn share_policy(&self, token: String) -> futures_util::future::BoxFuture<'static, anyhow::Result<Option<crate::share::SharePolicy>>> {
        let graph = self.graph.clone();
        Box::pin(async move {
            let mut g = graph.lock().await;
            crate::share::load(&mut g, &token).await
        })
    }

    async fn run(&self, name: &str, cypher: &str, params: serde_json::Value) -> Result<CallToolResult, McpError> {
        let mut g = self.graph.lock().await;
        let env = run_template(&mut g, name, cypher, params)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        drop(g);
        let mut value = serde_json::to_value(&env).map_err(|e| McpError::internal_error(e.to_string(), None))?;
        if let Some(p) = &self.policy {
            p.redact_value(&mut value);
        }
        let text = serde_json::to_string_pretty(&value).map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
    }
}

fn text_result<T: serde::Serialize>(v: &T) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(v).map_err(|e| McpError::internal_error(e.to_string(), None))?;
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

/// Resolve the caller's `person` argument (GitHub handle, plain name, or an existing
/// person: uid) to the Person uid convention used by log_encounter and the templates.
fn person_uid(person: &str) -> String {
    let p = person.trim().trim_start_matches('@');
    if p.starts_with("person:") { return p.to_string(); }
    if p.split_whitespace().count() > 1 {
        format!("person:name:{}", p.to_lowercase().split_whitespace().collect::<Vec<_>>().join("-"))
    } else {
        format!("person:gh:{p}")
    }
}

/// The graph's closed set of topic slugs — the same set list_topics shows.
async fn known_topics(g: &mut AsyncGraph) -> Result<std::collections::HashSet<String>, McpError> {
    let rows = query_rows(g, "MATCH (t:Topic) RETURN t.slug AS slug", serde_json::json!({}))
        .await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
    Ok(rows.iter().filter_map(|r| r["slug"].as_str().map(str::to_string)).collect())
}

/// Topic slugs are a closed set; reject anything list_topics would not show (as log_encounter does).
async fn require_topics(g: &mut AsyncGraph, topics: &[String]) -> Result<(), McpError> {
    let known = known_topics(g).await?;
    let unknown: Vec<&String> = topics.iter().filter(|t| !known.contains(t.as_str())).collect();
    if topics.is_empty() || !unknown.is_empty() {
        return Err(McpError::invalid_params(format!("topics must be non-empty slugs from list_topics; unknown: {unknown:?}"), None));
    }
    Ok(())
}

/// Strict `day` param: empty means no filter; anything else must map to a real
/// conference day, or the call is rejected instead of silently widening the query.
fn strict_day(raw: &str) -> Result<String, String> {
    if raw.trim().is_empty() { return Ok(String::new()); }
    let day = normalize_day(raw);
    if day.is_empty() {
        Err(format!("invalid day '{}': the conference days are 2026-10-28 and 2026-10-29, e.g. \"2026-10-29\" or \"day 2\"", raw.trim()))
    } else {
        Ok(day)
    }
}

/// Strict `after`/`before` param: empty means no filter; anything else must
/// parse as a 24h HH:MM time, or the call is rejected.
fn strict_time(raw: &str) -> Result<String, String> {
    if raw.trim().is_empty() { return Ok(String::new()); }
    let t = normalize_after(raw);
    if t.is_empty() {
        Err(format!("invalid time '{}': use 24h HH:MM, e.g. 10:30", raw.trim()))
    } else {
        Ok(t)
    }
}

/// The strict_* helpers speak String errors; tools turn them into invalid_params.
fn linkedin_slug(raw: &str) -> String {
    let r = raw.trim().split(['?', '#']).next().unwrap_or("").trim_end_matches('/');
    r.rsplit("/in/").next().unwrap_or(r).trim_end_matches('/').to_lowercase()
}

/// Contact uid precedence: GitHub handle, then LinkedIn slug, then plain name.
fn contact_uid(handle: &str, linkedin: &str, name: &str) -> String {
    if !handle.trim().is_empty() {
        format!("person:gh:{}", handle.trim().trim_start_matches('@'))
    } else if !linkedin_slug(linkedin).is_empty() {
        format!("person:li:{}", linkedin_slug(linkedin))
    } else {
        format!("person:name:{}", name.trim().to_lowercase().split_whitespace().collect::<Vec<_>>().join("-"))
    }
}

fn bad_param(e: String) -> McpError {
    McpError::invalid_params(e, None)
}

#[tool_router]
impl PatchworkMcp {
    #[tool(description = "Talks that cover topics of the person's open issues, as a 5-hop path (Person→Repo→Issue→Topic→Talk→Speaker). Optional day and earliest start time.")]
    async fn talks_for_my_issues(&self, Parameters(a): Parameters<TalksForMyIssuesArgs>) -> Result<CallToolResult, McpError> {
        let person = a.person.trim();
        if person.is_empty() {
            return Err(McpError::invalid_params("person is required, e.g. person:gh:<handle>", None));
        }
        let params = serde_json::json!({"person": person, "day": strict_day(&a.day).map_err(bad_param)?, "after": strict_time(&a.after).map_err(bad_param)?, "before": strict_time(&a.before).map_err(bad_param)?});
        self.run("T1", T1_TALKS_FOR_MY_ISSUES, params).await
    }

    #[tool(description = "Talks that cover the topics of one issue, with the path that connects them.")]
    async fn talks_for_issue(&self, Parameters(a): Parameters<IssueArgs>) -> Result<CallToolResult, McpError> {
        let issue = a.issue.trim();
        if issue.is_empty() || !issue.contains(':') || !issue.contains('#') {
            return Err(McpError::invalid_params("issue must be an issue uid like issue:<owner>/<repo>#<number> or issue:notes/<handle>#<id>", None));
        }
        self.run("T2", T2_TALKS_FOR_ISSUE, serde_json::json!({"issue": issue})).await
    }

    #[tool(description = "Speakers relevant to the person's open issues whom they have not logged meeting yet.")]
    async fn people_to_meet(&self, Parameters(a): Parameters<PeopleToMeetArgs>) -> Result<CallToolResult, McpError> {
        let person = a.person.trim();
        if person.is_empty() {
            return Err(McpError::invalid_params("person is required, e.g. person:gh:<handle>", None));
        }
        self.run("T3", T3_PEOPLE_TO_MEET, serde_json::json!({"person": person, "limit": 5})).await
    }

    #[tool(description = "People the user met who discussed the topics of one issue (memory recall), newest first.")]
    async fn memory_recall(&self, Parameters(a): Parameters<IssueArgs>) -> Result<CallToolResult, McpError> {
        let issue = a.issue.trim();
        if issue.is_empty() {
            return Err(McpError::invalid_params("issue is required, e.g. issue:<owner>/<repo>#<number>", None));
        }
        self.run("T4", T4_MEMORY_RECALL, serde_json::json!({"issue": issue})).await
    }

    #[tool(description = "The closed topic list (slug and name). Use these slugs for talks_by_topic and log_encounter.")]
    async fn list_topics(&self, Parameters(_): Parameters<NoArgs>) -> Result<CallToolResult, McpError> {
        let mut g = self.graph.lock().await;
        let rows = query_rows(&mut g, "MATCH (t:Topic) RETURN t.slug AS slug, t.name AS name ORDER BY slug", serde_json::json!({}))
            .await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        text_result(&rows)
    }

    #[tool(description = "Create a scoped MCP link for a teammate. Before calling, ask the user which tools to share (talks_for_my_issues, talks_for_issue, people_to_meet, memory_recall, talks_by_topic, list_topics) and whether to hide the private fields note and place. Then show the user the returned `url` to paste into the teammate's MCP client, and list `hidden_tools` as \"hidden from teammates\". The link can never call write or owner tools.")]
    async fn create_share(&self, Parameters(a): Parameters<CreateShareArgs>) -> Result<CallToolResult, McpError> {
        self.create_share_impl(a).await
    }

    #[tool(description = "List the share links you created (label, tools, hidden fields, created_at, first 8 characters of the token), newest first. Owner only.")]
    async fn list_shares(&self, Parameters(_a): Parameters<NoArgs>) -> Result<CallToolResult, McpError> {
        self.owner_only()?;
        let mut g = self.graph.lock().await;
        let shares = crate::share::list(&mut g).await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        text_result(&serde_json::json!({"shares": shares}))
    }

    #[tool(description = "Revoke every share link with this label; its URL stops working immediately. Returns how many were removed. Owner only.")]
    async fn revoke_share(&self, Parameters(a): Parameters<RevokeShareArgs>) -> Result<CallToolResult, McpError> {
        self.owner_only()?;
        let mut g = self.graph.lock().await;
        let removed = crate::share::revoke(&mut g, a.label.trim()).await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        text_result(&serde_json::json!({"label": a.label.trim(), "removed": removed}))
    }

    #[tool(description = "Record a conversation: a hallway chat, or a LinkedIn DM thread read with the LinkedIn MCP's get_conversation (then set place='LinkedIn DM', source_url and met_linkedin). Your own model extracts who was met and which topics (slugs from list_topics) were discussed, and writes a one- or two-sentence summary into `note`; never paste the raw transcript. The server stores it and PatchWork's graph agents link it to open issues. No LLM runs on the server.")]
    async fn log_encounter(&self, Parameters(a): Parameters<LogEncounterArgs>) -> Result<CallToolResult, McpError> {
        if a.note.trim().is_empty() || a.met_name.trim().is_empty() {
            return Err(McpError::invalid_params("note and met_name are required", None));
        }
        let mut g = self.graph.lock().await;
        require_topics(&mut g, &a.topics).await?;
        let uid = format!("enc:{}", uuid::Uuid::new_v4());
        let contact = contact_uid(&a.met_handle, &a.met_linkedin, &a.met_name);
        let err = |e: anyhow::Error| McpError::internal_error(e.to_string(), None);
        write_query(&mut g, "MATCH (p:Person {uid: $person}) MERGE (e:Encounter {uid: $uid}) SET e.note = $note, e.place = $place, e.source_url = $source_url, e.met_at = $met_at, e.state = 'NEW' MERGE (p)-[:LOGGED]->(e)",
            serde_json::json!({"person": a.person, "uid": uid, "note": a.note, "place": a.place, "source_url": a.source_url.trim(), "met_at": chrono::Utc::now().to_rfc3339()})).await.map_err(err)?;
        write_query(&mut g, "MATCH (e:Encounter {uid: $uid}) MERGE (c:Person {uid: $contact}) ON CREATE SET c.name = $name, c.handle = $handle, c.kind = 'contact', c.demo = false SET c.linkedin = CASE WHEN $linkedin = '' THEN c.linkedin ELSE $linkedin END MERGE (e)-[:MET]->(c)",
            serde_json::json!({"uid": uid, "contact": contact, "name": a.met_name.trim(), "handle": a.met_handle.trim(), "linkedin": linkedin_slug(&a.met_linkedin)})).await.map_err(err)?;
        for t in &a.topics {
            write_query(&mut g, "MATCH (e:Encounter {uid: $uid}), (t:Topic {slug: $slug}) MERGE (e)-[r:DISCUSSED]->(t) SET r.confidence = '1.0', r.evidence = 'set by client'",
                serde_json::json!({"uid": uid, "slug": t})).await.map_err(err)?;
        }
        // Parsed by the client, so hand straight to Pathfinder (PARSED); the server makes no LLM call.
        write_query(&mut g, "MATCH (e:Encounter {uid: $uid}) SET e.state = 'PARSED'", serde_json::json!({"uid": uid})).await.map_err(err)?;
        text_result(&serde_json::json!({"encounter_uid": uid, "contact_uid": contact, "state": "PARSED"}))
    }

    #[tool(description = "Log something the user learned at the conference. Works from a sentence the user says OR a photo they share (slide, whiteboard, poster, badge, notes): read the photo, extract the key lesson into `learned`, pick topic slugs with list_topics, and describe where it came from in `source`.")]
    async fn log_learning(&self, Parameters(a): Parameters<LogLearningArgs>) -> Result<CallToolResult, McpError> {
        if a.person.trim().is_empty() || a.learned.trim().is_empty() {
            return Err(McpError::invalid_params("person and learned are required", None));
        }
        let mut g = self.graph.lock().await;
        require_topics(&mut g, &a.topics).await?;
        let uid = format!("learning:{}", uuid::Uuid::new_v4());
        let person = person_uid(&a.person);
        let handle = if person.starts_with("person:gh:") { a.person.trim().trim_start_matches('@').to_string() } else { String::new() };
        let err = |e: anyhow::Error| McpError::internal_error(e.to_string(), None);
        write_query(&mut g, "MERGE (p:Person {uid: $person}) ON CREATE SET p.name = $name, p.handle = $handle, p.kind = 'contact', p.demo = false",
            serde_json::json!({"person": person, "name": a.person.trim(), "handle": handle})).await.map_err(err)?;
        write_query(&mut g, "MATCH (p:Person {uid: $person}) CREATE (l:Learning {uid: $uid}) SET l.text = $learned, l.source = $source, l.learned_at = $now, l.state = 'LOGGED' MERGE (p)-[:LOGGED]->(l)",
            serde_json::json!({"person": person, "uid": uid, "learned": a.learned, "source": a.source, "now": chrono::Utc::now().to_rfc3339()})).await.map_err(err)?;
        for t in &a.topics {
            write_query(&mut g, "MATCH (l:Learning {uid: $uid}), (t:Topic {slug: $slug}) MERGE (l)-[:ABOUT]->(t)",
                serde_json::json!({"uid": uid, "slug": t})).await.map_err(err)?;
        }
        // Params are string-typed, so fan the topic IN-list out per slug and merge here.
        let mut talks = Vec::new();
        for t in &a.topics {
            talks.extend(query_rows(&mut g, "MATCH (t:Topic {slug: $slug})<-[:COVERS]-(k:Talk)<-[:SPEAKS_AT]-(s:Person) RETURN k.title AS talk, k.day AS day, k.start_time AS start_time, k.room AS room, s.name AS speaker",
                serde_json::json!({"slug": t})).await.map_err(|e| McpError::internal_error(e.to_string(), None))?);
        }
        talks.sort_by_key(|r| (r["day"].as_str().unwrap_or("").to_string(), r["start_time"].as_str().unwrap_or("").to_string()));
        talks.dedup_by(|x, y| x == y);
        talks.truncate(5);
        text_result(&serde_json::json!({"learning_uid": uid, "person_uid": person, "state": "LOGGED", "topics": a.topics, "related_talks": talks}))
    }

    #[tool(description = "Log an open problem or question the user describes (verbally or from a photo of notes/a slide). It becomes an open issue on their personal notes repo in the graph, auto-classified under the given topics; the agents then find talks and people that address it.")]
    async fn log_issue(&self, Parameters(a): Parameters<LogIssueArgs>) -> Result<CallToolResult, McpError> {
        if a.person.trim().is_empty() || a.title.trim().is_empty() || a.body.trim().is_empty() {
            return Err(McpError::invalid_params("person, title and body are required", None));
        }
        let mut g = self.graph.lock().await;
        require_topics(&mut g, &a.topics).await?;
        let who = a.person.trim().trim_start_matches('@');
        let person = person_uid(&a.person);
        let handle = if person.starts_with("person:gh:") { who.to_string() } else { String::new() };
        let repo = format!("repo:notes/{who}");
        let issue = format!("issue:notes/{who}#{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
        let err = |e: anyhow::Error| McpError::internal_error(e.to_string(), None);
        // MERGE the notes repo so T1 (Person→Repo→Issue→Topic→Talk) reaches issues logged here.
        write_query(&mut g, "MERGE (p:Person {uid: $person}) ON CREATE SET p.name = $name, p.handle = $handle, p.kind = 'contact', p.demo = false MERGE (r:Repo {uid: $repo}) ON CREATE SET r.full_name = $full_name MERGE (p)-[:MAINTAINS]->(r)",
            serde_json::json!({"person": person, "name": a.person.trim(), "handle": handle, "repo": repo, "full_name": format!("{who}'s conference notes")})).await.map_err(err)?;
        // state='CLASSIFIED' hands the issue straight to Pathfinder (T2 talks, T4 contacts) within seconds.
        write_query(&mut g, "MATCH (r:Repo {uid: $repo}) CREATE (i:Issue {uid: $issue}) SET i.title = $title, i.body = $body, i.gh_state = 'open', i.state = 'CLASSIFIED', i.created_at = $now MERGE (r)-[:HAS_ISSUE]->(i)",
            serde_json::json!({"repo": repo, "issue": issue, "title": a.title, "body": a.body, "now": chrono::Utc::now().to_rfc3339()})).await.map_err(err)?;
        for t in &a.topics {
            write_query(&mut g, "MATCH (i:Issue {uid: $issue}), (t:Topic {slug: $slug}) MERGE (i)-[:ABOUT]->(t)",
                serde_json::json!({"issue": issue, "slug": t})).await.map_err(err)?;
        }
        text_result(&serde_json::json!({"issue_uid": issue, "repo_uid": repo, "person_uid": person, "state": "CLASSIFIED", "next": "Pathfinder claims CLASSIFIED issues within seconds and bridges them to talks (T2) and useful contacts (T4); watch check_alerts."}))
    }

    #[tool(description = "Recall what the user logged as learned, newest first, optionally filtered to one topic slug from list_topics (empty = all). Summarize these lessons back to the user.")]
    async fn recall_learnings(&self, Parameters(a): Parameters<RecallLearningsArgs>) -> Result<CallToolResult, McpError> {
        if a.person.trim().is_empty() {
            return Err(McpError::invalid_params("person is required, e.g. a GitHub handle or plain name", None));
        }
        let mut g = self.graph.lock().await;
        let person = person_uid(&a.person);
        let topic = a.topic.trim();
        if !topic.is_empty() && !known_topics(&mut g).await?.contains(topic) {
            return Err(McpError::invalid_params(format!("topic must be a slug from list_topics; unknown: [\"{topic}\"]"), None));
        }
        // Results come back column-typed (no arrays), so fan out one row per learning×topic and group here.
        // No topic predicate in the query: filtering the OPTIONAL MATCH would drop only the non-matching
        // topic rows, so every learning would still come back (untagged ones) and multi-topic lessons
        // would lose their other tags. The filter is applied to the grouped learnings below instead.
        let rows = query_rows(&mut g, "MATCH (p:Person {uid: $person})-[:LOGGED]->(l:Learning) OPTIONAL MATCH (l)-[:ABOUT]->(t:Topic) RETURN l.uid AS uid, l.text AS text, l.source AS source, l.learned_at AS learned_at, t.slug AS topic ORDER BY l.learned_at DESC",
            serde_json::json!({"person": person})).await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let mut learnings: Vec<serde_json::Value> = Vec::new();
        let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for r in rows {
            let uid = r["uid"].as_str().unwrap_or_default().to_string();
            let slug = r["topic"].as_str().unwrap_or_default().to_string();
            if let Some(&i) = seen.get(&uid) {
                if !slug.is_empty() { if let Some(ts) = learnings[i]["topics"].as_array_mut() { ts.push(serde_json::Value::String(slug)); } }
            } else {
                let topics = if slug.is_empty() { Vec::new() } else { vec![serde_json::Value::String(slug)] };
                seen.insert(uid.clone(), learnings.len());
                learnings.push(serde_json::json!({"uid": uid, "text": r["text"], "source": r["source"], "learned_at": r["learned_at"], "topics": topics}));
            }
        }
        if !topic.is_empty() {
            learnings.retain(|l| l["topics"].as_array().map_or(false, |ts| ts.iter().any(|t| t.as_str() == Some(topic))));
        }
        learnings.truncate(20);
        text_result(&serde_json::json!({"person_uid": person, "learnings": learnings}))
    }

    #[tool(description = "Bridges PatchWork found between your open issues and people you met (newest first). Draft any follow-up message with your own model.")]
    async fn check_alerts(&self, Parameters(_): Parameters<NoArgs>) -> Result<CallToolResult, McpError> {
        let mut g = self.graph.lock().await;
        let rows = query_rows(&mut g, "MATCH (i:Issue)-[b:BRIDGE {kind: 'contact'}]->(c:Person) RETURN b.uid AS bridge_uid, b.state AS state, i.uid AS issue_uid, i.title AS issue, c.name AS contact, b.via_topic AS topic, b.hops AS hops ORDER BY b.created_at DESC LIMIT 10", serde_json::json!({}))
            .await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        text_result(&rows)
    }

    #[tool(description = "Talks about given topics, filtered by day and earliest start time, with their path.")]
    async fn talks_by_topic(&self, Parameters(a): Parameters<TalksByTopicArgs>) -> Result<CallToolResult, McpError> {
        let slugs: Vec<String> = a.topics.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
        if slugs.is_empty() {
            return Err(McpError::invalid_params("topics must be a non-empty comma-separated list of slugs from list_topics, e.g. \"mcp-auth,agentic-security\"", None));
        }
        {
            // Same closed-set check as the write tools: an unknown slug errors instead of matching zero talks.
            let mut g = self.graph.lock().await;
            let known = known_topics(&mut g).await?;
            let unknown: Vec<&String> = slugs.iter().filter(|t| !known.contains(t.as_str())).collect();
            if !unknown.is_empty() {
                return Err(McpError::invalid_params(format!("topics must be non-empty slugs from list_topics; unknown: {unknown:?}"), None));
            }
        }
        let params = serde_json::json!({"topics": slugs.join(","), "day": strict_day(&a.day).map_err(bad_param)?, "after": strict_time(&a.after).map_err(bad_param)?, "before": strict_time(&a.before).map_err(bad_param)?});
        self.run("T6", T6_TALKS_BY_TOPIC, params).await
    }
}

#[tool_handler]
impl PatchworkMcp {
    /// Create a share token and return its MCP path. Owner only.
    async fn create_share_impl(&self, a: CreateShareArgs) -> Result<CallToolResult, McpError> {
        if self.policy.is_some() {
            return Err(McpError::invalid_request("shares cannot create shares", None));
        }
        let policy = crate::share::SharePolicy::new(&a.label, &a.tools, &a.redact).map_err(|e| McpError::invalid_params(e, None))?;
        let mut g = self.graph.lock().await;
        let token = crate::share::create(&mut g, &policy).await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        text_result(&serde_json::json!({"label": policy.label, "token": token, "url": share_url(&token), "mcp_path": format!("/share/{token}/mcp"), "tools": policy.tools, "redact": policy.redact, "hidden_tools": crate::share::withheld(&policy)}))
    }
}

impl ServerHandler for PatchworkMcp {
    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::CallToolResponse, McpError> {
        // The HTTP gate stores the share policy in the request extensions; the owner's /mcp has none.
        let policy = context.extensions.get::<axum::http::request::Parts>().and_then(|p| p.extensions.get::<crate::share::SharePolicy>()).cloned();
        let scoped = match policy {
            Some(p) => {
                if !p.allows(&request.name) {
                    return Err(McpError::invalid_request(format!("tool '{}' is not shared with this link", request.name), None));
                }
                self.scoped(p)
            }
            None => self.clone(),
        };
        let tcc = rmcp::handler::server::tool::ToolCallContext::new(&scoped, request, context);
        scoped.tool_router.call(tcc).await
    }

    async fn list_tools(
        &self,
        request: Option<rmcp::model::PaginatedRequestParams>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, McpError> {
        let policy = context.extensions.get::<axum::http::request::Parts>().and_then(|p| p.extensions.get::<crate::share::SharePolicy>()).cloned();
        let mut result = rmcp::model::ListToolsResult::default();
        result.tools = self.tool_router.list_all().into_iter().filter(|t| policy.as_ref().map_or(true, |p| p.allows(&t.name))).collect();
        let _ = request;
        Ok(result)
    }

    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("patchwork", env!("CARGO_PKG_VERSION")))
            .with_instructions("PatchWork graph tools. Each tool runs a fixed, verified Cypher template on FalkorDB. Arguments are typed values only; no Cypher is accepted.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contact_uid_precedence() {
        assert_eq!(contact_uid("@octo", "ada", "Ada L"), "person:gh:octo");
        assert_eq!(contact_uid("", "https://www.linkedin.com/in/Ada-L/", "Ada L"), "person:li:ada-l");
        assert_eq!(contact_uid("", "linkedin.com/in/ada-l?miniProfileUrn=x#y", "Ada L"), "person:li:ada-l");
        assert_eq!(contact_uid("", "", "Ada  L"), "person:name:ada-l");
    }

    #[test]
    fn strict_day_forms() {
        assert_eq!(strict_day("2026-10-28"), Ok("2026-10-28".to_string()));
        assert_eq!(strict_day("day 2"), Ok("2026-10-29".to_string()));
        assert_eq!(strict_day(""), Ok(String::new()));
        assert_eq!(strict_day("   "), Ok(String::new()));
        let err = strict_day("2030-01-01").unwrap_err();
        assert!(err.contains("2026-10-28") && err.contains("2026-10-29") && err.contains("2030-01-01"));
    }

    #[test]
    fn strict_time_forms() {
        assert_eq!(strict_time("10:30"), Ok("10:30".to_string()));
        assert_eq!(strict_time("9am"), Ok("09:00".to_string()));
        assert_eq!(strict_time(""), Ok(String::new()));
        assert_eq!(strict_time("99:99"), Err("invalid time '99:99': use 24h HH:MM, e.g. 10:30".to_string()));
    }
}
