//! Scoped share links: a token that exposes a subset of PatchWork's tools, with
//! fields redacted from the results. The policy lives in FalkorDB (`:Share` nodes),
//! so it survives a restart. The owner's own /mcp is never restricted.
use crate::graph::query_rows;
use falkordb::AsyncGraph;
use serde_json::Value;

/// Tools a share may expose. Owner-only tools (writes, alerts, learnings) are never
/// listed here, so a teammate can read the graph but not change it.
pub const SHARABLE_TOOLS: &[&str] = &["talks_for_my_issues", "talks_for_issue", "people_to_meet", "memory_recall", "talks_by_topic", "list_topics"];

/// Fields that never leave the owner's graph through a share.
pub const REDACTED_FIELDS: &[&str] = &["note", "met_linkedin", "source_url", "handle", "place"];

#[derive(Debug, Clone)]
pub struct SharePolicy {
    pub label: String,
    pub tools: Vec<String>,
    pub redact: Vec<String>,
}

impl SharePolicy {
    /// Validate a requested policy: only known sharable tools and known redactable fields.
    pub fn new(label: &str, tools: &[String], redact: &[String]) -> Result<Self, String> {
        if label.trim().is_empty() {
            return Err("label is required".into());
        }
        if tools.is_empty() {
            return Err("share at least one tool".into());
        }
        if let Some(bad) = tools.iter().find(|t| !SHARABLE_TOOLS.contains(&t.as_str())) {
            return Err(format!("tool '{bad}' cannot be shared; allowed: {}", SHARABLE_TOOLS.join(", ")));
        }
        if let Some(bad) = redact.iter().find(|f| !REDACTED_FIELDS.contains(&f.as_str())) {
            return Err(format!("field '{bad}' is not redactable; allowed: {}", REDACTED_FIELDS.join(", ")));
        }
        Ok(Self { label: label.trim().into(), tools: tools.to_vec(), redact: redact.to_vec() })
    }

    pub fn allows(&self, tool: &str) -> bool {
        self.tools.iter().any(|t| t == tool)
    }

    /// Remove every redacted key from a tool result, recursively.
    pub fn redact_value(&self, v: &mut Value) {
        match v {
            Value::Object(m) => {
                for f in &self.redact {
                    m.remove(f);
                }
                m.values_mut().for_each(|c| self.redact_value(c));
            }
            Value::Array(a) => a.iter_mut().for_each(|c| self.redact_value(c)),
            _ => {}
        }
    }
}

/// Mint a random token and persist its policy. Returns the token.
pub async fn create(g: &mut AsyncGraph, policy: &SharePolicy) -> anyhow::Result<String> {
    let token = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
    crate::graph::write_query(
        g,
        "MERGE (s:Share {token: $token}) SET s.label = $label, s.tools = $tools, s.redact = $redact, s.created_at = $created_at",
        serde_json::json!({
            "token": token,
            "label": policy.label,
            "tools": policy.tools.join(","),
            "redact": policy.redact.join(","),
            "created_at": chrono::Utc::now().to_rfc3339(),
        }),
    )
    .await?;
    Ok(token)
}

/// Load a policy by token. Unknown tokens return None (the caller answers 401).
pub async fn load(g: &mut AsyncGraph, token: &str) -> anyhow::Result<Option<SharePolicy>> {
    let rows = query_rows(
        g,
        "MATCH (s:Share {token: $token}) RETURN s.label AS label, s.tools AS tools, s.redact AS redact",
        serde_json::json!({"token": token}),
    )
    .await?;
    let Some(row) = rows.first() else { return Ok(None) };
    let split = |k: &str| row[k].as_str().unwrap_or("").split(',').filter(|s| !s.is_empty()).map(str::to_string).collect::<Vec<_>>();
    Ok(Some(SharePolicy {
        label: row["label"].as_str().unwrap_or("").to_string(),
        tools: split("tools"),
        redact: split("redact"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_owner_only_tools_and_unknown_fields() {
        assert!(SharePolicy::new("x", &["log_encounter".into()], &[]).is_err());
        assert!(SharePolicy::new("x", &["talks_for_issue".into()], &["secret".into()]).is_err());
        assert!(SharePolicy::new("", &["talks_for_issue".into()], &[]).is_err());
        assert!(SharePolicy::new("x", &["talks_for_issue".into()], &["note".into()]).is_ok());
    }

    #[test]
    fn redacts_fields_recursively() {
        let p = SharePolicy::new("x", &["memory_recall".into()], &["note".into(), "place".into()]).unwrap();
        let mut v = serde_json::json!({"rows": [{"contact": "Ada", "note": "private", "place": "snack table"}], "note": "top"});
        p.redact_value(&mut v);
        assert_eq!(v, serde_json::json!({"rows": [{"contact": "Ada"}]}));
    }
}
