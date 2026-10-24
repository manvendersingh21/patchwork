//! OpenAI calls and a small disk cache for deterministic demo replay.
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TopicHit {
    pub slug: String,
    pub confidence: f32,
    pub evidence: String,
}

#[derive(Deserialize)]
struct Taxonomy { topics: Vec<Topic> }
#[derive(Deserialize)]
struct Topic { slug: String, name: String }

fn taxonomy() -> Result<Taxonomy> {
    serde_json::from_str(include_str!("../../data/topics.json")).context("read topics.json")
}

fn cache_path() -> PathBuf { PathBuf::from("data/cache/llm.json") }

fn cache_key(function: &str, model: &str, input: &Value) -> Result<String> {
    let mut hash = Sha256::new();
    hash.update(function.as_bytes());
    hash.update([0]);
    hash.update(model.as_bytes());
    hash.update([0]);
    hash.update(serde_json::to_vec(input)?);
    Ok(format!("{:x}", hash.finalize()))
}

fn read_cache() -> HashMap<String, Value> {
    fs::read(cache_path()).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

fn write_cache(cache: &HashMap<String, Value>) -> Result<()> {
    let path = cache_path();
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, serde_json::to_vec(cache)?)?;
    fs::rename(tmp, path)?;
    Ok(())
}

async fn chat_json(function: &str, input: Value, schema: Value, instructions: &str) -> Result<Value> {
    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into());
    let key = cache_key(function, &model, &input)?;
    let mut cache = read_cache();
    if let Some(v) = cache.get(&key) { return Ok(v.clone()); }
    let api_key = std::env::var("OPENAI_API_KEY").context("OPENAI_API_KEY is not set")?;
    let response = reqwest::Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "messages": [
                {"role":"system","content":instructions},
                {"role":"user","content":serde_json::to_string(&input)?}
            ],
            "response_format": {"type":"json_schema","json_schema":{"name":function,"strict":true,"schema":schema}}
        }))
        .send().await?;
    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() { return Err(anyhow!("OpenAI HTTP {status}: {body}")); }
    let response: Value = serde_json::from_str(&body).context("parse OpenAI response")?;
    let content = response["choices"][0]["message"]["content"].as_str().ok_or_else(|| anyhow!("OpenAI returned no JSON content"))?;
    let result: Value = serde_json::from_str(content).context("parse OpenAI JSON response")?;
    cache.insert(key, result.clone());
    write_cache(&cache)?;
    Ok(result)
}

fn hit_schema(topics: &[Topic]) -> Value {
    let mut slugs: Vec<String> = topics.iter().map(|t| t.slug.clone()).collect();
    slugs.push("none".into());
    json!({"type":"object","properties":{"items":{"type":"array","items":{"type":"array","items":{"type":"object","properties":{"slug":{"type":"string","enum":slugs},"confidence":{"type":"number"},"evidence":{"type":"string"}},"required":["slug","confidence","evidence"],"additionalProperties":false}}}},"required":["items"],"additionalProperties":false})
}

/// Classify text in batches of at most ten; low confidence and `none` hits are removed.
pub async fn classify(items: &[String]) -> Result<Vec<Vec<TopicHit>>> {
    let taxonomy = taxonomy()?;
    let schema = hit_schema(&taxonomy.topics);
    let mut output = Vec::with_capacity(items.len());
    for batch in items.chunks(10) {
        let value = chat_json("classify", json!({"items": batch}), schema.clone(),
            "Map each text to zero to three topics from the closed list. Return only exact copied evidence, at most 12 words. Use none when no topic fits.").await?;
        let mut classified: Vec<Vec<TopicHit>> = serde_json::from_value(value["items"].clone())?;
        classified.resize_with(batch.len(), Vec::new);
        output.extend(classified.into_iter().take(batch.len()).map(|hits| hits.into_iter()
            .filter(|h| h.slug != "none" && h.confidence >= 0.5 && h.evidence.split_whitespace().count() <= 12)
            .take(3).collect()));
    }
    Ok(output)
}

pub async fn parse_note(note: &str) -> Result<Value> {
    let tax = taxonomy()?;
    let slugs: Vec<String> = tax.topics.iter().map(|t| t.slug.clone()).collect();
    let schema = json!({"type":"object","properties":{"people":{"type":"array","items":{"type":"object","properties":{"name":{"type":"string"},"handle":{"type":"string"},"company":{"type":"string"}},"required":["name","handle","company"],"additionalProperties":false}},"topics":{"type":"array","items":{"type":"string","enum":slugs}},"repo_urls":{"type":"array","items":{"type":"string"}}},"required":["people","topics","repo_urls"],"additionalProperties":false});
    chat_json("parse_note", json!({"note":note,"topics":tax.topics.iter().map(|t|(&t.slug,&t.name)).collect::<Vec<_>>()}), schema,
        "Extract only people, closed-list topic slugs, and repository URLs explicitly mentioned in the note. Empty handle/company when unknown.").await
}

pub async fn draft_followup(note: &str, issue: &str) -> Result<String> {
    let schema = json!({"type":"object","properties":{"draft":{"type":"string"}},"required":["draft"],"additionalProperties":false});
    let v = chat_json("draft_followup", json!({"note":note,"issue":issue}), schema,
        "Write a concise, warm follow-up of 2 or 3 sentences grounded only in the supplied note and issue.").await?;
    Ok(v["draft"].as_str().unwrap_or_default().to_string())
}

pub async fn answer(question: &str, rows: &Value) -> Result<String> {
    let schema = json!({"type":"object","properties":{"answer":{"type":"string"}},"required":["answer"],"additionalProperties":false});
    let v = chat_json("answer", json!({"question":question,"rows":rows}), schema,
        "Answer in 1 to 3 sentences using only the supplied graph rows. If rows are empty, say the graph has no traced answer.").await?;
    Ok(v["answer"].as_str().unwrap_or_default().to_string())
}
