mod catalog;
mod gui;
mod ops;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "taria",
    version,
    about = "Taria — an open-source Linux package manager",
    long_about = "Taria combines a GTK desktop interface, an online software catalog, and Ubuntu APT/DPKG integration."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Search APT and the Taria catalog (alias: s)
    #[command(alias = "s")]
    Search { query: String },
    /// Show details about a package
    Info { package: String },
    /// Install a package through APT (alias: i)
    #[command(alias = "i")]
    Install { package: String },
    /// Remove a package through APT (alias: r)
    #[command(alias = "r")]
    Remove { package: String },
    /// Refresh APT and Taria catalog metadata (alias: u)
    #[command(alias = "u")]
    Update,
    /// Upgrade installed packages through APT
    #[command(alias = "up")]
    Upgrade,
    /// List installed Debian packages
    List,
    /// Run basic package-system diagnostics
    Doctor,
    /// Show the Taria version
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None => gui::run(),
        Some(Command::Search { query }) => {
            catalog::search_local_and_online(&query)?;
            ops::search_apt(&query)?;
            Ok(())
        }
        Some(Command::Info { package }) => {
            ops::show_info(&package)?;
            Ok(())
        }
        Some(Command::Install { package }) => ops::install(&package),
        Some(Command::Remove { package }) => ops::remove(&package),
        Some(Command::Update) => {
            ops::require_root("update")?;
            ops::run_apt(&["update"])?;
            catalog::refresh_online_catalog();
            Ok(())
        }
        Some(Command::Upgrade) => {
            ops::require_root("upgrade")?;
            ops::run_apt(&["upgrade", "-y"])
        }
        Some(Command::List) => ops::list_installed(),
        Some(Command::Doctor) => ops::doctor(),
        Some(Command::Version) => {
            println!("Taria {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
