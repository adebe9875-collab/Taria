use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;

const API: &str = "https://api.github.com";
const MAX_BINARY_BYTES: u64 = 64 * 1024 * 1024;
const REPOSITORY: &str = "adebe9875-collab/Taria";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

fn http() -> Result<Client> {
    Client::builder()
        .user_agent(concat!("Taria/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("could not create GitHub update client")
}

fn latest_release(repository: &str) -> Result<Option<Release>> {
    if repository != REPOSITORY {
        bail!("Taria self-updates are restricted to its official repository.");
    }
    let response = http()?
        .get(format!("{API}/repos/{repository}/releases/latest"))
        .send()
        .context("could not contact the Taria GitHub release service")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    let release: Release = response
        .error_for_status()
        .context("GitHub release lookup failed")?
        .json()
        .context("GitHub returned invalid release metadata")?;
    if release.prerelease {
        return Ok(None);
    }
    Ok(Some(release))
}

fn local_asset_names() -> Result<(&'static str, &'static str)> {
    match std::env::consts::ARCH {
        "x86_64" => Ok(("taria-linux-x86_64", "taria-linux-x86_64.sha256")),
        "aarch64" => Ok(("taria-linux-aarch64", "taria-linux-aarch64.sha256")),
        other => bail!("self-update is not yet published for architecture {other}"),
    }
}

fn parse_release_version(tag: &str) -> Result<semver::Version> {
    let version = tag.trim_start_matches('v');
    semver::Version::parse(version)
        .with_context(|| format!("GitHub release tag '{tag}' is not a valid semantic version"))
}

pub fn check_latest(repository: &str) -> Result<Option<String>> {
    let Some(release) = latest_release(repository)? else {
        return Ok(None);
    };
    let latest = parse_release_version(&release.tag_name)?;
    let current = semver::Version::parse(env!("CARGO_PKG_VERSION"))?;
    if latest > current {
        Ok(Some(latest.to_string()))
    } else {
        Ok(None)
    }
}

pub fn self_update(repository: &str, check_only: bool) -> Result<()> {
    let Some(release) = latest_release(repository)? else {
        println!("No published stable Taria release was found yet.");
        return Ok(());
    };
    let latest = parse_release_version(&release.tag_name)?;
    let current = semver::Version::parse(env!("CARGO_PKG_VERSION"))?;

    if latest <= current {
        println!("Taria is already up to date ({current}).");
        return Ok(());
    }
    println!("Taria update available: {current} → {latest}");
    if check_only {
        println!("Check only: no files were changed.");
        return Ok(());
    }
    ensure_root()?;

    let (binary_name, checksum_name) = local_asset_names()?;
    let binary_url = release
        .assets
        .iter()
        .find(|asset| asset.name == binary_name)
        .map(|asset| asset.browser_download_url.as_str())
        .context("this release does not contain a binary for your CPU architecture")?;
    let checksum_url = release
        .assets
        .iter()
        .find(|asset| asset.name == checksum_name)
        .map(|asset| asset.browser_download_url.as_str())
        .context("release is missing its SHA-256 checksum file; refusing an unverified update")?;

    let expected_prefix = format!(
        "https://github.com/{repository}/releases/download/{}/",
        release.tag_name
    );
    if !binary_url.starts_with(&expected_prefix) || !checksum_url.starts_with(&expected_prefix) {
        bail!(
            "release assets point outside the official Taria GitHub release; refusing the update"
        );
    }

    let client = http()?;
    let binary_response = client.get(binary_url).send()?.error_for_status()?;
    if let Some(length) = binary_response.content_length() {
        if length > MAX_BINARY_BYTES {
            bail!("release binary is larger than the 64 MiB safety limit");
        }
    }
    let binary = binary_response.bytes()?;
    if binary.is_empty() || binary.len() as u64 > MAX_BINARY_BYTES {
        bail!("release binary is empty or larger than the 64 MiB safety limit");
    }
    validate_linux_binary(&binary)?;

    let checksum_text = client
        .get(checksum_url)
        .send()?
        .error_for_status()?
        .text()?;
    let expected = checksum_text
        .split_whitespace()
        .next()
        .context("checksum file is empty")?
        .to_ascii_lowercase();
    if expected.len() != 64
        || !expected
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        bail!("release SHA-256 file is malformed");
    }

    let actual = hex::encode(Sha256::digest(&binary));
    if expected != actual {
        bail!("download checksum mismatch; the current Taria binary was not changed");
    }

    let current_exe =
        std::env::current_exe().context("could not locate the running Taria executable")?;
    let parent = current_exe
        .parent()
        .context("Taria executable has no parent directory")?;
    let temp = parent.join(format!(".taria-update-{}.tmp", std::process::id()));

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .context("could not create a temporary update file beside the current executable")?;

    if let Err(error) = file.write_all(&binary).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(&temp);
        return Err(error).context("could not write the verified update");
    }
    drop(file);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(error) = fs::set_permissions(&temp, fs::Permissions::from_mode(0o755)) {
            let _ = fs::remove_file(&temp);
            return Err(error).context("could not set executable permissions on the update");
        }
    }

    if let Err(error) = fs::rename(&temp, &current_exe) {
        let _ = fs::remove_file(&temp);
        return Err(error).context("could not replace Taria; the existing executable was kept");
    }

    println!("Taria was updated to {latest}. Start a new terminal command to use it.");
    Ok(())
}

fn validate_linux_binary(binary: &[u8]) -> Result<()> {
    if binary.len() < 20 || &binary[0..4] != b"\x7fELF" {
        bail!("release asset is not a valid ELF executable");
    }
    let machine = u16::from_le_bytes([binary[18], binary[19]]);
    let expected_machine = match std::env::consts::ARCH {
        "x86_64" => 62,   // EM_X86_64
        "aarch64" => 183, // EM_AARCH64
        other => bail!("unsupported self-update architecture: {other}"),
    };
    if machine != expected_machine {
        bail!("release binary architecture does not match this machine");
    }
    Ok(())
}

fn ensure_root() -> Result<()> {
    #[cfg(target_os = "linux")]
    let root = unsafe { libc::geteuid() == 0 };
    #[cfg(not(target_os = "linux"))]
    let root = false;

    if !root {
        bail!(
            "updating /usr/local/bin/taria needs administrator permissions; run 'sudo taria self-update'"
        );
    }
    Ok(())
}
