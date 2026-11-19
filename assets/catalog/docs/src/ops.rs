use anyhow::{bail, Context, Result};
use std::process::{Command, ExitStatus};

/// Accept only ordinary Debian package-name characters. This blocks option and shell injection.
pub fn validate_package_name(name: &str) -> Result<()> {
    let valid = !name.is_empty()
        && name.len() <= 200
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".+:_-".contains(&b));
    if !valid {
        bail!("Invalid package name: {name:?}");
    }
    Ok(())
}

pub fn require_root(action: &str) -> Result<()> {
    // SAFETY: geteuid has no pointer arguments and is safe to call on Linux.
    if unsafe { libc::geteuid() } != 0 {
        bail!(
            "This action needs administrator permission. Run: sudo taria {action}"
        );
    }
    Ok(())
}

pub fn run_apt(args: &[&str]) -> Result<()> {
    let status = Command::new("apt-get")
        .args(args)
        .status()
        .context("Could not start apt-get. Check that APT is installed.")?;
    ensure_success(status, "apt-get")
}

fn ensure_success(status: ExitStatus, program: &str) -> Result<()> {
    if !status.success() {
        bail!("{program} exited with status: {status}");
    }
    Ok(())
}

pub fn install(package: &str) -> Result<()> {
    validate_package_name(package)?;
    require_root(&format!("install {package}"))?;
    println!("Installing {package} with APT...");
    run_apt(&["install", "-y", "--", package])
}

pub fn remove(package: &str) -> Result<()> {
    validate_package_name(package)?;
    require_root(&format!("remove {package}"))?;
    println!("Removing {package} with APT...");
    run_apt(&["remove", "-y", "--", package])
}

pub fn search_apt(query: &str) -> Result<()> {
    let status = Command::new("apt-cache")
        .args(["search", "--", query])
        .status()
        .context("Could not start apt-cache")?;
    ensure_success(status, "apt-cache")
}

pub fn show_info(package: &str) -> Result<()> {
    validate_package_name(package)?;
    let status = Command::new("apt-cache")
        .args(["show", "--", package])
        .status()
        .context("Could not start apt-cache")?;
    ensure_success(status, "apt-cache")
}

pub fn list_installed() -> Result<()> {
    let status = Command::new("dpkg-query")
        .args(["-W", "-f=${binary:Package}\t${Version}\t${db:Status-Status}\n"])
        .status()
        .context("Could not start dpkg-query")?;
    ensure_success(status, "dpkg-query")
}

pub fn doctor() -> Result<()> {
    println!("Taria system check\n-----------------");
    for program in ["apt-get", "apt-cache", "dpkg-query", "pkexec"] {
        let found = Command::new("sh")
            .args(["-c", "command -v \"$1\" >/dev/null 2>&1", "taria-doctor", program])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        println!("{program}: {}", if found { "found" } else { "not found" });
    }
    let audit = Command::new("dpkg").arg("--audit").status();
    match audit {
        Ok(s) if s.success() => println!("dpkg audit: completed"),
        Ok(s) => println!("dpkg audit: reported issues (exit {s})"),
        Err(e) => println!("dpkg audit: could not run ({e})"),
    }
    Ok(())
}
