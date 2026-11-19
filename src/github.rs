use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;

const API: &str = "https://api.github.com";

#[derive(Debug, Deserialize)]
struct RepositorySearch {
    items: Vec<Repository>,
}

#[derive(Debug, Deserialize)]
struct Repository {
    full_name: String,
    description: Option<String>,
    html_url: String,
    stargazers_count: u64,
}

#[derive(Debug, Deserialize)]
struct IssueSearch {
    items: Vec<Issue>,
}

#[derive(Debug, Deserialize)]
struct Issue {
    title: String,
    html_url: String,
}

fn client() -> Result<Client> {
    Client::builder()
        .user_agent(concat!("Taria/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("could not create GitHub HTTP client")
}

pub fn search_repositories(query: &str) -> Result<usize> {
    let response: RepositorySearch = client()?
        .get(format!("{API}/search/repositories"))
        .query(&[
            ("q", format!("{query} in:name,description")),
            ("per_page", "10".to_string()),
        ])
        .send()
        .context("could not reach GitHub repository search")?
        .error_for_status()
        .context("GitHub repository search returned an error")?
        .json()
        .context("GitHub returned an unexpected search response")?;

    println!("GitHub repositories:");
    for repo in &response.items {
        println!("  {}  ★ {}", repo.full_name, repo.stargazers_count);
        if let Some(description) = &repo.description {
            if !description.is_empty() {
                println!("    {description}");
            }
        }
        println!("    {}", repo.html_url);
    }
    Ok(response.items.len())
}

pub fn search_issues(repository: &str, message: &str) -> Result<()> {
    let response: IssueSearch = client()?
        .get(format!("{API}/search/issues"))
        .query(&[
            ("q", format!("repo:{repository} is:issue {message}")),
            ("per_page", "10".to_string()),
        ])
        .send()
        .context("could not reach GitHub issue search")?
        .error_for_status()
        .context("GitHub issue search returned an error")?
        .json()
        .context("GitHub returned an unexpected issue-search response")?;

    if response.items.is_empty() {
        println!("No matching issue was found in https://github.com/{repository}/issues.");
    } else {
        println!("Related issues in {repository}:");
        for issue in response.items {
            println!("  {}\n    {}", issue.title, issue.html_url);
        }
    }
    println!("Taria does not upload local logs automatically. Review them before sharing any report.");
    Ok(())
}
