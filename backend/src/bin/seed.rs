#![recursion_limit = "512"]
//! Seeds the graph from data/*.json and .env. Every write is a MERGE, so re-running is safe.
use anyhow::{Context, Result};
use falkordb::AsyncGraph;
use patchwork::graph::connect;
use serde_json::Value;

async fn write(g: &mut AsyncGraph, q: &str, pairs: &[(&str, &str)]) -> Result<()> {
    let owned: Vec<(String, String)> = pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    g.query(q).with_params(owned).execute().await?;
    Ok(())
}

fn read_json(path: &str) -> Result<Value> {
    let text = std::fs::read_to_string(path).with_context(|| path.to_string())?;
    Ok(serde_json::from_str(&text)?)
}

fn env_value(key: &str) -> Option<String> {
    std::fs::read_to_string(".env").ok()?.lines().find_map(|line| {
        let (k, v) = line.split_once('=')?;
        (k.trim() == key).then(|| v.trim().to_string())
    }).filter(|v| !v.is_empty())
}

fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}

fn slug(name: &str) -> String {
    name.to_lowercase().split_whitespace().collect::<Vec<_>>().join("-")
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut g = connect("localhost", 6379, "patchwork").await?;
    let sessions = read_json("data/sessions.json")?;
    let topics = read_json("data/topics.json")?;
    let demo = read_json("data/demo_issues.json")?;

    let presenter = env_value("PRESENTER_HANDLE").context("PRESENTER_HANDLE missing in .env")?;
    let me = format!("person:gh:{presenter}");
    write(&mut g, "MERGE (p:Person {uid: $uid}) SET p.name = $name, p.handle = $name, p.kind = 'self', p.demo = false, p.state = 'SYNCED'",
        &[("uid", me.as_str()), ("name", presenter.as_str())]).await?;

    for t in topics["topics"].as_array().context("topics")? {
        let uid = format!("topic:{}", s(t, "slug"));
        write(&mut g, "MERGE (t:Topic {uid: $uid}) SET t.slug = $slug, t.name = $name, t.description = $desc",
            &[("uid", uid.as_str()), ("slug", s(t, "slug")), ("name", s(t, "name")), ("desc", s(t, "description"))]).await?;
    }

    for sess in sessions["sessions"].as_array().context("sessions")? {
        let talk = format!("talk:{}", s(sess, "sid"));
        write(&mut g, "MERGE (k:Talk {uid: $uid}) SET k.title = $title, k.day = $day, k.start_time = $start, k.end_time = $end, k.room = $room",
            &[("uid", talk.as_str()), ("title", s(sess, "title")), ("day", s(sess, "day")),
              ("start", s(sess, "start_time")), ("end", s(sess, "end_time")), ("room", s(sess, "room"))]).await?;
        for topic in sess["topics"].as_array().into_iter().flatten() {
            let t = format!("topic:{}", topic.as_str().unwrap_or(""));
            write(&mut g, "MATCH (k:Talk {uid: $k}), (t:Topic {uid: $t}) MERGE (k)-[:COVERS]->(t)",
                &[("k", talk.as_str()), ("t", t.as_str())]).await?;
        }
        for sp in sess["speakers"].as_array().into_iter().flatten() {
            if sp["link"] == Value::Bool(false) { continue; }
            let name = s(sp, "name");
            let person = format!("person:name:{}", slug(name));
            write(&mut g, "MERGE (p:Person {uid: $uid}) SET p.name = $name, p.company = $company, p.kind = 'speaker', p.demo = false",
                &[("uid", person.as_str()), ("name", name), ("company", s(sp, "company"))]).await?;
            write(&mut g, "MATCH (p:Person {uid: $p}), (k:Talk {uid: $k}) MERGE (p)-[:SPEAKS_AT]->(k)",
                &[("p", person.as_str()), ("k", talk.as_str())]).await?;
        }
    }

    let repo_name = env_value("DEMO_REPO").context("DEMO_REPO missing in .env")?;
    let repo = format!("repo:{repo_name}");
    let url = format!("https://github.com/{repo_name}");
    write(&mut g, "MERGE (r:Repo {uid: $uid}) SET r.full_name = $name, r.url = $url, r.demo = false",
        &[("uid", repo.as_str()), ("name", repo_name.as_str()), ("url", url.as_str())]).await?;
    write(&mut g, "MATCH (p:Person {uid: $p}), (r:Repo {uid: $r}) MERGE (p)-[:MAINTAINS]->(r)",
        &[("p", me.as_str()), ("r", repo.as_str())]).await?;

    for iss in demo["issues"].as_array().context("issues")? {
        let number = iss["number"].as_i64().unwrap_or(0).to_string();
        let uid = format!("issue:{repo_name}#{number}");
        let issue_url = format!("https://github.com/{repo_name}/issues/{number}");
        write(&mut g, "MERGE (i:Issue {uid: $uid}) SET i.number = $n, i.title = $title, i.gh_state = 'open', i.state = 'LINKED', i.demo = false, i.url = $url",
            &[("uid", uid.as_str()), ("n", number.as_str()), ("title", s(iss, "title")), ("url", issue_url.as_str())]).await?;
        write(&mut g, "MATCH (r:Repo {uid: $r}), (i:Issue {uid: $i}) MERGE (r)-[:HAS_ISSUE]->(i)",
            &[("r", repo.as_str()), ("i", uid.as_str())]).await?;
        for topic in iss["topics"].as_array().into_iter().flatten() {
            let t = format!("topic:{}", topic.as_str().unwrap_or(""));
            write(&mut g, "MATCH (i:Issue {uid: $i}), (t:Topic {uid: $t}) MERGE (i)-[:ABOUT]->(t)",
                &[("i", uid.as_str()), ("t", t.as_str())]).await?;
        }
    }

    for tracked in env_value("TRACKED_REPOS").unwrap_or_default().split(',').map(str::trim).filter(|r| !r.is_empty()) {
        let uid = format!("repo:{tracked}");
        let url = format!("https://github.com/{tracked}");
        write(&mut g, "MERGE (r:Repo {uid: $uid}) SET r.full_name = $name, r.url = $url",
            &[("uid", uid.as_str()), ("name", tracked), ("url", url.as_str())]).await?;
        write(&mut g, "MATCH (p:Person {uid: $p}), (r:Repo {uid: $r}) MERGE (p)-[:TRACKS]->(r)",
            &[("p", me.as_str()), ("r", uid.as_str())]).await?;
    }

    println!("seeded: talks={} topics={} issues={}",
        sessions["sessions"].as_array().map_or(0, |a| a.len()),
        topics["topics"].as_array().map_or(0, |a| a.len()),
        demo["issues"].as_array().map_or(0, |a| a.len()));
    Ok(())
}
