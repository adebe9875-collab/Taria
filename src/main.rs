mod backend;
mod github;
mod logging;
mod updater;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::process::ExitCode;

const GITHUB_REPOSITORY: &str = "adebe9875-collab/Taria";

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Source {
    Apt,
    Snap,
    Pacman,
    Github,
    All,
}

#[derive(Parser, Debug)]
#[command(
    name = "taria",
    version,
    about = "Taria — terminal-native package manager",
    long_about = "Taria manages Linux packages through available native package managers and searches GitHub for project information."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Search APT, Snap, Pacman and GitHub
    #[command(visible_alias = "s")]
    Search {
        query: String,
        #[arg(long, value_enum, default_value = "all")]
        source: Source,
    },
    /// Install a package (alias: i)
    #[command(visible_alias = "i")]
    Install {
        package: String,
        #[arg(long, value_enum)]
        source: Option<Source>,
    },
    /// Remove a package (alias: r)
    #[command(visible_alias = "r")]
    Remove {
        package: String,
        #[arg(long, value_enum)]
        source: Option<Source>,
        #[arg(long)]
        purge: bool,
    },
    /// Show package details
    Info {
        package: String,
        #[arg(long, value_enum, default_value = "apt")]
        source: Source,
    },
    /// List installed packages
    #[command(visible_alias = "ls")]
    List {
        #[arg(long, value_enum, default_value = "all")]
        source: Source,
    },
    /// Refresh package metadata
    #[command(visible_alias = "u")]
    Update {
        #[arg(long, value_enum, default_value = "all")]
        source: Source,
    },
    /// Upgrade installed packages
    #[command(visible_alias = "up")]
    Upgrade {
        #[arg(long, value_enum, default_value = "all")]
        source: Source,
        #[arg(long, help = "Use the distribution's full-upgrade operation")]
        full: bool,
    },
    /// Diagnose package managers and check Taria's GitHub release
    Doctor,
    /// Search the Taria GitHub issue tracker for a related report
    Report {
        message: String,
    },
    /// Check for or install a verified Taria update from GitHub Releases
    #[command(name = "self-update", visible_alias = "selfupdate")]
    SelfUpdate {
        #[arg(long, help = "Only check; do not replace the executable")]
        check: bool,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let message = format!("{error:#}");
            eprintln!("\nTaria error: {message}");
            logging::record_error(&message);

            // Search GitHub using a generic label only. Never upload local logs or paths.
            eprintln!("Checking the Taria GitHub issue tracker for known problems...");
            if let Err(remote_error) =
                github::search_issues(GITHUB_REPOSITORY, "Taria CLI command failure")
            {
                eprintln!("GitHub issue lookup unavailable: {remote_error:#}");
            }
            match updater::check_latest(GITHUB_REPOSITORY) {
                Ok(Some(version)) => eprintln!(
                    "A newer Taria release ({version}) is available. Review it with 'taria self-update --check', then apply with 'sudo taria self-update'."
                ),
                Ok(None) => eprintln!("No newer stable Taria release was found."),
                Err(update_error) => eprintln!("Could not check Taria updates: {update_error:#}"),
            }
            eprintln!("Local error log: {}", logging::log_path().display());
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let Some(command) = cli.command else {
        println!("Taria {} — terminal package manager", env!("CARGO_PKG_VERSION"));
        println!("Try 'taria help' or 'taria search firefox'.");
        return Ok(());
    };

    match command {
        Commands::Search { query, source } => search(&query, source),
        Commands::Install { package, source } => install(&package, source),
        Commands::Remove { package, source, purge } => remove(&package, source, purge),
        Commands::Info { package, source } => backend::show_info(source, &package),
        Commands::List { source } => backend::list_installed(source),
        Commands::Update { source } => backend::refresh(source),
        Commands::Upgrade { source, full } => backend::upgrade(source, full),
        Commands::Doctor => doctor(),
        Commands::Report { message } => github::search_issues(GITHUB_REPOSITORY, &message),
        Commands::SelfUpdate { check } => updater::self_update(GITHUB_REPOSITORY, check),
    }
}

fn search(query: &str, source: Source) -> Result<()> {
    if query.trim().is_empty() {
        bail!("search text cannot be empty");
    }

    println!("Searching for '{query}'...\n");
    let mut found = 0usize;
    let strict = !matches!(source, Source::All);

    if matches!(source, Source::All | Source::Apt) && backend::available(backend::Manager::Apt) {
        collect_search(&mut found, "APT", backend::search(backend::Manager::Apt, query), strict)?;
    }
    if matches!(source, Source::All | Source::Snap) && backend::available(backend::Manager::Snap) {
        collect_search(&mut found, "Snap", backend::search(backend::Manager::Snap, query), strict)?;
    }
    if matches!(source, Source::All | Source::Pacman) && backend::available(backend::Manager::Pacman) {
        collect_search(&mut found, "Pacman", backend::search(backend::Manager::Pacman, query), strict)?;
    }
    if matches!(source, Source::All | Source::Github) {
        collect_search(&mut found, "GitHub", github::search_repositories(query), strict)?;
    }

    if found == 0 {
        println!("No matching packages or GitHub repositories were found.");
    } else {
        println!("\n{found} result(s).");
    }
    Ok(())
}

fn collect_search(
    total: &mut usize,
    label: &str,
    result: Result<usize>,
    strict: bool,
) -> Result<()> {
    match result {
        Ok(count) => *total += count,
        Err(error) if strict => return Err(error),
        Err(error) => eprintln!("{label} search unavailable: {error:#}"),
    }
    Ok(())
}

fn install(package: &str, source: Option<Source>) -> Result<()> {
    validate_package_name(package)?;
    let manager = match source {
        Some(Source::Apt) => backend::Manager::Apt,
        Some(Source::Snap) => backend::Manager::Snap,
        Some(Source::Pacman) => backend::Manager::Pacman,
        Some(Source::Github) => bail!(
            "GitHub repositories are searchable, but arbitrary repository code is not installed as a system package. A verified Taria catalog entry is required."
        ),
        Some(Source::All) | None => backend::choose_install_manager(package)?,
    };
    backend::install(manager, package)
}

fn remove(package: &str, source: Option<Source>, purge: bool) -> Result<()> {
    validate_package_name(package)?;
    let manager = match source {
        Some(Source::Apt) => backend::Manager::Apt,
        Some(Source::Snap) => backend::Manager::Snap,
        Some(Source::Pacman) => backend::Manager::Pacman,
        Some(Source::Github) => bail!("GitHub source is not a system package manager."),
        Some(Source::All) | None => backend::detect_installed_manager(package)
            .context("package is not installed by a supported manager; use --source to select one")?,
    };
    backend::remove(manager, package, purge)
}

fn validate_package_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 200 {
        bail!("package name is empty or too long");
    }
    if name.starts_with('-') || name.chars().any(|c| c.is_control() || c.is_whitespace()) {
        bail!("invalid package name: {name:?}");
    }
    Ok(())
}

fn doctor() -> Result<()> {
    println!("Taria system diagnosis\n======================");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("OS: {}", backend::os_description());
    for manager in [
        backend::Manager::Apt,
        backend::Manager::Snap,
        backend::Manager::Pacman,
    ] {
        let status = if backend::available(manager) {
            "available"
        } else {
            "not available"
        };
        println!("{}: {status}", manager.label());
    }
    println!("Logs: {}", logging::log_path().display());
    println!();

    match updater::check_latest(GITHUB_REPOSITORY) {
        Ok(Some(version)) => println!(
            "GitHub: Taria {version} is available (current: {}). Run 'sudo taria self-update' after reviewing the release.",
            env!("CARGO_PKG_VERSION")
        ),
        Ok(None) => println!("GitHub: no newer stable release found."),
        Err(error) => println!("GitHub: update check unavailable: {error:#}"),
    }
    Ok(())
}
