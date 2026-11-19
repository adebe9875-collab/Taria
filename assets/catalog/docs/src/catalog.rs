use anyhow::{Context, Result};
use serde::Deserialize;

const CATALOG_URL: &str = "https://raw.githubusercontent.com/adebe9875-collab/Taria/main/catalog/apps.json";

#[derive(Debug, Deserialize)]
struct CatalogApp {
    id: String,
    name: String,
    description: String,
    source: String,
    package: Option<String>,
    homepage: Option<String>,
}

fn read_local_catalog() -> Result<Vec<CatalogApp>> {
    serde_json::from_str(include_str!("../catalog/apps.json")).context("Invalid local catalog/apps.json")
}

fn read_online_catalog() -> Result<Vec<CatalogApp>> {
    let response = reqwest::blocking::get(CATALOG_URL)
        .context("Could not reach the Taria GitHub catalog")?;
    let response = response.error_for_status().context("GitHub catalog returned an error")?;
    response.json().context("GitHub catalog JSON was invalid")
}

pub fn search_local_and_online(query: &str) -> Result<()> {
    let local = read_local_catalog()?;
    let online = read_online_catalog().unwrap_or_default();
    let q = query.to_lowercase();
    let mut found = false;
    for app in local.iter().chain(online.iter()) {
        let haystack = format!("{} {} {}", app.id, app.name, app.description).to_lowercase();
        if haystack.contains(&q) {
            println!("{} — {} [{}]", app.name, app.description, app.source);
            if let Some(pkg) = &app.package {
                println!("  package: {pkg}");
            }
            if let Some(url) = &app.homepage {
                println!("  website: {url}");
            }
            found = true;
        }
    }
    if !found {
        println!("No matching app found in the Taria catalog. Searching APT next...");
    }
    Ok(())
}

pub fn refresh_online_catalog() {
    match read_online_catalog() {
        Ok(apps) => println!("Taria catalog reachable ({} entries).", apps.len()),
        Err(err) => eprintln!("Taria catalog refresh skipped: {err}"),
    }
}
