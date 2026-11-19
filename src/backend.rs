use anyhow::{Context, Result, bail};
use std::process::{Command, Output};

use crate::Source;

#[derive(Debug, Clone, Copy)]
pub enum Manager {
    Apt,
    Snap,
    Pacman,
}

impl Manager {
    pub fn label(self) -> &'static str {
        match self {
            Self::Apt => "APT / DPKG",
            Self::Snap => "Snap",
            Self::Pacman => "Pacman",
        }
    }

    fn executable(self) -> &'static str {
        match self {
            Self::Apt => "apt-cache",
            Self::Snap => "snap",
            Self::Pacman => "pacman",
        }
    }
}

pub fn available(manager: Manager) -> bool {
    executable_exists(manager.executable())
}

fn executable_exists(program: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|directory| directory.join(program).is_file())
}

fn output(program: &str, args: &[&str]) -> Result<Output> {
    Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("could not start {program}; check that it is installed"))
}

fn run_privileged(program: &str, args: &[String]) -> Result<()> {
    ensure_root()?;
    println!("→ {program} {}", args.join(" "));
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("could not start {program}"))?;
    if !status.success() {
        bail!("{program} exited with status {status}");
    }
    Ok(())
}

fn ensure_root() -> Result<()> {
    // Taria deliberately does not silently elevate itself. The caller must use sudo.
    #[cfg(target_os = "linux")]
    let is_root = unsafe { libc::geteuid() == 0 };
    #[cfg(not(target_os = "linux"))]
    let is_root = false;

    if !is_root {
        bail!("this operation needs administrator permissions; run it as 'sudo taria ...'");
    }
    Ok(())
}

pub fn os_description() -> String {
    let content = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    let pretty = content
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .unwrap_or("Linux");
    pretty.trim_matches('"').to_string()
}

pub fn search(manager: Manager, query: &str) -> Result<usize> {
    let result = match manager {
        Manager::Apt => apt_search(query)?,
        Manager::Snap => snap_search(query)?,
        Manager::Pacman => pacman_search(query)?,
    };

    println!("{} results:", manager.label());
    let count = result
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    if result.trim().is_empty() {
        println!("  No results.");
        return Ok(0);
    }

    for line in result.lines().take(30) {
        println!("  {line}");
    }
    if result.lines().count() > 30 {
        println!("  … output shortened; use the native package manager for the full list.");
    }
    Ok(count)
}

fn apt_search(query: &str) -> Result<String> {
    let out = output("apt-cache", &["search", "--names-only", query])?;
    ensure_success("apt-cache search", &out)?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn snap_search(query: &str) -> Result<String> {
    let out = output("snap", &["find", query])?;
    ensure_success("snap find", &out)?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn pacman_search(query: &str) -> Result<String> {
    let out = output("pacman", &["-Ss", query])?;
    ensure_success("pacman -Ss", &out)?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn ensure_success(label: &str, out: &Output) -> Result<()> {
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!("{label} failed: {}", stderr.trim());
    }
    Ok(())
}

pub fn show_info(source: Source, package: &str) -> Result<()> {
    match source {
        Source::Apt => print_output(output("apt-cache", &["show", package])?, "apt-cache show")?,
        Source::Snap => print_output(output("snap", &["info", package])?, "snap info")?,
        Source::Pacman => print_output(output("pacman", &["-Si", package])?, "pacman -Si")?,
        Source::Github => {
            crate::github::search_repositories(package)?;
        }
        Source::All => {
            println!("Choose a source with --source apt|snap|pacman|github.");
        }
    }
    Ok(())
}

fn print_output(out: Output, label: &str) -> Result<()> {
    ensure_success(label, &out)?;
    print!("{}", String::from_utf8_lossy(&out.stdout));
    Ok(())
}

pub fn list_installed(source: Source) -> Result<()> {
    let mut ran = false;

    if matches!(source, Source::All | Source::Apt) && available(Manager::Apt) {
        let out = output(
            "dpkg-query",
            &[
                "-W",
                "-f=${binary:Package}\t${Version}\t${db:Status-Abbrev}\n",
            ],
        )?;
        ensure_success("dpkg-query", &out)?;
        println!("Installed APT/DPKG packages:");
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let status = line.split('\t').nth(2).unwrap_or("").trim();
            if !line.trim().is_empty() && status.starts_with("ii") {
                println!("  {line}");
            }
        }
        ran = true;
    }

    if matches!(source, Source::All | Source::Snap) && available(Manager::Snap) {
        let out = output("snap", &["list"])?;
        ensure_success("snap list", &out)?;
        println!("\nInstalled Snap packages:");
        print!("{}", String::from_utf8_lossy(&out.stdout));
        ran = true;
    }

    if matches!(source, Source::All | Source::Pacman) && available(Manager::Pacman) {
        let out = output("pacman", &["-Q"])?;
        ensure_success("pacman -Q", &out)?;
        println!("\nInstalled Pacman packages:");
        print!("{}", String::from_utf8_lossy(&out.stdout));
        ran = true;
    }

    if !ran {
        println!("No supported package manager was detected.");
    }
    Ok(())
}

pub fn choose_install_manager(package: &str) -> Result<Manager> {
    let os = std::fs::read_to_string("/etc/os-release")
        .unwrap_or_default()
        .to_lowercase();

    if (os.contains("id=arch") || os.contains("id=manjaro"))
        && available(Manager::Pacman)
        && pacman_has_package(package)
    {
        return Ok(Manager::Pacman);
    }

    if available(Manager::Apt) && apt_has_candidate(package) {
        return Ok(Manager::Apt);
    }
    if available(Manager::Snap) && snap_has_package(package) {
        return Ok(Manager::Snap);
    }
    if available(Manager::Pacman) && pacman_has_package(package) {
        return Ok(Manager::Pacman);
    }

    bail!(
        "could not find '{package}' in the enabled APT, Snap or Pacman sources; use 'taria search {package}'"
    )
}

pub fn detect_installed_manager(package: &str) -> Option<Manager> {
    if available(Manager::Apt) {
        if let Ok(out) = output("dpkg-query", &["-W", "-f=${db:Status-Abbrev}", package]) {
            if out.status.success()
                && String::from_utf8_lossy(&out.stdout)
                    .trim_start()
                    .starts_with("ii")
            {
                return Some(Manager::Apt);
            }
        }
    }

    if available(Manager::Snap) {
        if let Ok(out) = output("snap", &["list", package]) {
            if out.status.success() {
                return Some(Manager::Snap);
            }
        }
    }

    if available(Manager::Pacman) {
        if let Ok(out) = output("pacman", &["-Q", package]) {
            if out.status.success() {
                return Some(Manager::Pacman);
            }
        }
    }
    None
}

fn apt_has_candidate(package: &str) -> bool {
    output("apt-cache", &["policy", package])
        .ok()
        .filter(|out| out.status.success())
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .any(|line| line.trim_start().starts_with("Candidate:") && !line.contains("(none)"))
        })
        .unwrap_or(false)
}

fn snap_has_package(package: &str) -> bool {
    output("snap", &["info", package])
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn pacman_has_package(package: &str) -> bool {
    output("pacman", &["-Si", package])
        .map(|out| out.status.success())
        .unwrap_or(false)
}

pub fn install(manager: Manager, package: &str) -> Result<()> {
    match manager {
        Manager::Apt => run_privileged(
            "apt-get",
            &["install".into(), "-y".into(), "--".into(), package.into()],
        ),
        Manager::Snap => run_privileged("snap", &["install".into(), package.into()]),
        Manager::Pacman => run_privileged(
            "pacman",
            &[
                "-S".into(),
                "--noconfirm".into(),
                "--needed".into(),
                package.into(),
            ],
        ),
    }
}

pub fn remove(manager: Manager, package: &str, purge: bool) -> Result<()> {
    match manager {
        Manager::Apt => {
            let verb = if purge { "purge" } else { "remove" };
            run_privileged(
                "apt-get",
                &[verb.into(), "-y".into(), "--".into(), package.into()],
            )
        }
        Manager::Snap => {
            if purge {
                run_privileged("snap", &["remove".into(), "--purge".into(), package.into()])
            } else {
                run_privileged("snap", &["remove".into(), package.into()])
            }
        }
        Manager::Pacman => run_privileged(
            "pacman",
            &["-Rns".into(), "--noconfirm".into(), package.into()],
        ),
    }
}

pub fn refresh(source: Source) -> Result<()> {
    ensure_root()?;
    let mut ran = false;

    if matches!(source, Source::All | Source::Apt) && available(Manager::Apt) {
        run_privileged("apt-get", &["update".into()])?;
        ran = true;
    }

    // On Arch, a sync-only update can leave the system in a partial-upgrade state.
    // Therefore Taria's update command uses the safe full sync-and-upgrade operation.
    if matches!(source, Source::All | Source::Pacman) && available(Manager::Pacman) {
        run_privileged("pacman", &["-Syu".into(), "--noconfirm".into()])?;
        ran = true;
    }

    if matches!(source, Source::All | Source::Snap) && available(Manager::Snap) {
        println!(
            "Snap refreshes automatically; use 'taria upgrade --source snap' to request an immediate refresh."
        );
        ran = true;
    }

    if !ran {
        println!("No supported package manager was detected.");
    }
    Ok(())
}

pub fn upgrade(source: Source, full: bool) -> Result<()> {
    ensure_root()?;
    let mut ran = false;

    if matches!(source, Source::All | Source::Apt) && available(Manager::Apt) {
        let verb = if full { "full-upgrade" } else { "upgrade" };
        run_privileged("apt-get", &[verb.into(), "-y".into()])?;
        ran = true;
    }

    if matches!(source, Source::All | Source::Snap) && available(Manager::Snap) {
        run_privileged("snap", &["refresh".into()])?;
        ran = true;
    }

    if matches!(source, Source::All | Source::Pacman) && available(Manager::Pacman) {
        run_privileged("pacman", &["-Syu".into(), "--noconfirm".into()])?;
        ran = true;
    }

    if !ran {
        println!("No supported package manager was detected.");
    }
    Ok(())
}
