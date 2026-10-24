//! GitHub REST client. Authentication tokens are kept in memory and never logged.
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Clone)]
pub struct GithubClient { http: reqwest::Client, token: Option<String> }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Repo {
    pub full_name: String,
    pub html_url: String,
    pub description: Option<String>,
    pub stargazers_count: u64,
    pub pushed_at: Option<String>,
    pub fork: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub html_url: String,
    pub state: String,
    pub created_at: String,
    pub pull_request: Option<serde_json::Value>,
}

impl GithubClient {
    /// Read GITHUB_TOKEN, or resolve `gh auth token` once during startup.
    pub fn from_env() -> Result<Self> {
        let token = std::env::var("GITHUB_TOKEN").ok().filter(|v| !v.trim().is_empty()).or_else(|| {
            Command::new("gh").arg("auth").arg("token").output().ok()
                .filter(|o| o.status.success()).and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
        });
        Ok(Self { http: reqwest::Client::builder().user_agent("PatchWork/0.1").build()?, token })
    }

    pub fn authenticated(&self) -> bool { self.token.is_some() }

    fn request(&self, url: &str) -> reqwest::RequestBuilder {
        let req = self.http.get(url).header("Accept", "application/vnd.github+json").header("X-GitHub-Api-Version", "2022-11-28");
        match &self.token { Some(token) => req.bearer_auth(token), None => req }
    }

    pub async fn list_top_repos(&self, handle: &str, demo_repo: Option<&str>) -> Result<Vec<Repo>> {
        let url = format!("https://api.github.com/users/{handle}/repos?sort=pushed&direction=desc&per_page=100");
        let mut repos: Vec<Repo> = self.request(&url).send().await?.error_for_status()?.json().await?;
        repos.retain(|r| !r.fork);
        repos.sort_by(|a,b| b.pushed_at.cmp(&a.pushed_at));
        let mut chosen: Vec<Repo> = repos.iter().take(3).cloned().collect();
        if let Some(demo) = demo_repo {
            if !chosen.iter().any(|r| r.full_name.eq_ignore_ascii_case(demo)) {
                let r = repos.into_iter().find(|r| r.full_name.eq_ignore_ascii_case(demo))
                    .or_else(|| Some(Repo { full_name: demo.to_string(), html_url: format!("https://github.com/{demo}"), description: None, stargazers_count: 0, pushed_at: None, fork: false }))
                    .ok_or_else(|| anyhow!("DEMO_REPO not found"))?;
                chosen.push(r);
            }
        }
        Ok(chosen)
    }

    pub async fn open_issues(&self, repo: &str) -> Result<Vec<Issue>> {
        let url = format!("https://api.github.com/repos/{repo}/issues?state=open&per_page=100");
        let items: Vec<Issue> = self.request(&url).send().await?.error_for_status()?.json().await?;
        Ok(items.into_iter().filter(|i| i.pull_request.is_none()).take(10).collect())
    }

    /// Watcher query: GitHub's `since` filters updated time, then created_at enforces cursor semantics.
    pub async fn issues_created_after(&self, repo: &str, cursor: DateTime<Utc>) -> Result<Vec<Issue>> {
        let since = cursor.to_rfc3339();
        let url = format!("https://api.github.com/repos/{repo}/issues?state=all&since={since}&per_page=100");
        let items: Vec<Issue> = self.request(&url).send().await?.error_for_status()?.json().await?;
        Ok(items.into_iter().filter(|i| i.pull_request.is_none() && DateTime::parse_from_rfc3339(&i.created_at).map(|t| t.with_timezone(&Utc) > cursor).unwrap_or(false)).collect())
    }
}

pub fn parse_repo_name(name: &str) -> Result<(&str, &str)> {
    let (owner, repo) = name.split_once('/').context("repository must be owner/name")?;
    if owner.is_empty() || repo.is_empty() || repo.contains('/') { anyhow::bail!("repository must be owner/name"); }
    Ok((owner, repo))
}
