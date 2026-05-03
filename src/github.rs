use anyhow::{Context, Result};
use octocrab::Octocrab;
use serde_json::json;
use std::env;
use tracing::{info, instrument};

#[instrument(skip(body, labels))]
pub async fn create_issue(title: &str, body: &str, labels: Vec<String>) -> Result<String> {
    let token = env::var("GITHUB_PAT").context("GITHUB_PAT not set")?;
    let owner = env::var("GITHUB_OWNER").context("GITHUB_OWNER not set")?;
    let repo = env::var("GITHUB_REPO").context("GITHUB_REPO not set")?;
    let assignee = env::var("GITHUB_ASSIGNEE").context("GITHUB_ASSIGNEE not set")?;
    let project_org = env::var("GITHUB_PROJECT_ORG").unwrap_or_else(|_| owner.clone());
    let project_number = env::var("GITHUB_PROJECT_NUMBER").unwrap_or_default();

    info!(owner = %owner, repo = %repo, title = %title, assignee = %assignee, "Creating GitHub issue");

    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .context("Failed to build octocrab client")?;

    // Create the issue
    let issue = octocrab
        .issues(&owner, &repo)
        .create(title)
        .body(body)
        .labels(labels)
        .assignees(vec![assignee])
        .send()
        .await
        .context("Failed to create GitHub issue")?;

    let issue_url = issue.html_url.to_string();
    let node_id = issue.node_id;

    info!(url = %issue_url, node_id = %node_id, "GitHub issue created");

    // Add to Project V2 if configured
    if !project_number.is_empty() {
        if let Ok(num) = project_number.parse::<i32>() {
            match add_to_project_v2(&octocrab, &project_org, num, &node_id).await {
                Ok(_) => info!("Added to Project V2"),
                Err(e) => tracing::error!(error = ?e, "Failed to add to Project V2"),
            }
        }
    }

    Ok(issue_url)
}

async fn add_to_project_v2(octocrab: &Octocrab, org: &str, project_number: i32, content_id: &str) -> Result<()> {
    // 1. Get Project V2 ID
    let query_project = json!({
        "query": r#"
            query($org: String!, $number: Int!) {
                organization(login: $org) {
                    projectV2(number: $number) {
                        id
                    }
                }
            }
        "#,
        "variables": {
            "org": org,
            "number": project_number
        }
    });

    let resp_project: serde_json::Value = octocrab.graphql(&query_project).await?;
    let project_id = resp_project["data"]["organization"]["projectV2"]["id"]
        .as_str()
        .context("Project V2 not found")?;

    // 2. Add item to Project
    let mutation = json!({
        "query": r#"
            mutation($project: ID!, $content: ID!) {
                addProjectV2ItemById(input: {projectId: $project, contentId: $content}) {
                    item {
                        id
                    }
                }
            }
        "#,
        "variables": {
            "project": project_id,
            "content": content_id
        }
    });

    let _: serde_json::Value = octocrab.graphql(&mutation).await?;
    Ok(())
}

pub async fn get_labels() -> Result<Vec<String>> {
    let token = env::var("GITHUB_PAT").context("GITHUB_PAT not set")?;
    let owner = env::var("GITHUB_OWNER").context("GITHUB_OWNER not set")?;
    let repo = env::var("GITHUB_REPO").context("GITHUB_REPO not set")?;

    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .context("Failed to build octocrab client")?;

    let labels = octocrab
        .issues(owner, repo)
        .list_labels_for_repo()
        .send()
        .await
        .context("Failed to fetch labels")?;

    Ok(labels.items.into_iter().map(|l| l.name).collect())
}
