use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn log_path() -> PathBuf {
    if let Some(state) = std::env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(state).join("taria").join("errors.log");
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".local")
        .join("state")
        .join("taria")
        .join("errors.log")
}

pub fn record_error(message: &str) {
    let path = log_path();
    if let Some(parent) = path.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return;
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let safe_message = message.replace(['\n', '\r'], " ");
        let _ = writeln!(
            file,
            "[{now}] Taria {}: {safe_message}",
            env!("CARGO_PKG_VERSION")
        );
    }
}
