//! Minimal client for the LinkedIn MCP server: JSON-RPC over streamable HTTP with reqwest.
//! Configured by LINKEDIN_MCP_URL and LINKEDIN_BEARER_TOKEN. Only the API's send endpoint uses it.
use anyhow::{anyhow, bail, Result};
use serde_json::{json, Value};

pub struct LinkedIn {
    url: String,
    token: String,
    http: reqwest::Client,
}

impl LinkedIn {
    pub fn from_env() -> Option<Self> {
        let url = std::env::var("LINKEDIN_MCP_URL").ok().filter(|s| !s.is_empty())?;
        let token = std::env::var("LINKEDIN_BEARER_TOKEN").unwrap_or_default();
        Some(Self { url, token, http: reqwest::Client::new() })
    }

    async fn post(&self, session: Option<&str>, body: Value) -> Result<(Option<String>, Option<Value>)> {
        let mut req = self
            .http
            .post(&self.url)
            .header("Accept", "application/json, text/event-stream")
            .bearer_auth(&self.token)
            .json(&body);
        if let Some(s) = session {
            req = req.header("mcp-session-id", s);
        }
        let res = req.send().await?;
        if !res.status().is_success() {
            bail!("LinkedIn MCP returned HTTP {}", res.status());
        }
        let sid = res.headers().get("mcp-session-id").and_then(|v| v.to_str().ok()).map(str::to_string);
        let text = res.text().await?;
        // Either a plain JSON body or SSE frames whose data line carries the JSON-RPC message.
        let msg = if text.trim_start().starts_with('{') {
            serde_json::from_str(&text).ok()
        } else {
            text.lines()
                .filter_map(|l| l.strip_prefix("data:"))
                .map(str::trim)
                .filter(|l| l.starts_with('{'))
                .last()
                .and_then(|l| serde_json::from_str(l).ok())
        };
        Ok((sid, msg))
    }

    /// Call one tool and return its text content. Tool errors come back as Err.
    pub async fn call(&self, tool: &str, args: Value) -> Result<String> {
        let init = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": "patchwork", "version": env!("CARGO_PKG_VERSION")}}});
        let (sid, _) = self.post(None, init).await?;
        let sid = sid.ok_or_else(|| anyhow!("LinkedIn MCP returned no session id"))?;
        self.post(Some(&sid), json!({"jsonrpc": "2.0", "method": "notifications/initialized"})).await?;
        let call = json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": tool, "arguments": args}});
        let (_, msg) = self.post(Some(&sid), call).await?;
        let msg = msg.ok_or_else(|| anyhow!("LinkedIn MCP returned no result"))?;
        if let Some(e) = msg.get("error") {
            bail!("{}", e["message"].as_str().unwrap_or("LinkedIn MCP error"));
        }
        let text = msg["result"]["content"][0]["text"].as_str().unwrap_or("").to_string();
        if msg["result"]["isError"].as_bool() == Some(true) {
            bail!("{text}");
        }
        Ok(text)
    }
}
