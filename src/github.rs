use anyhow::{Context, Result};
use octocrab::Octocrab;
use std::env;

pub async fn create_issue(title: &str, body: &str) -> Result<String> {
    let token = env::var("GITHUB_PAT").context("GITHUB_PAT not set")?;
    let owner = env::var("GITHUB_OWNER").context("GITHUB_OWNER not set")?;
    let repo = env::var("GITHUB_REPO").context("GITHUB_REPO not set")?;

    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .context("Failed to build octocrab client")?;

    let issue = octocrab
        .issues(owner, repo)
        .create(title)
        .body(body)
        .send()
        .await
        .context("Failed to create GitHub issue")?;

    Ok(issue.html_url.to_string())
}
