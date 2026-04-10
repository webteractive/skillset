use std::fs;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const CHECK_INTERVAL: Duration = Duration::from_secs(86400); // 24 hours
const GITHUB_API_URL: &str = "https://api.github.com/repos/webteractive/skillset/releases/latest";

fn cache_path() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "skillset")?;
    Some(dirs.config_dir().join(".version_check"))
}

/// Read cached latest version and last-check timestamp.
fn read_cache() -> Option<(String, SystemTime)> {
    let path = cache_path()?;
    let content = fs::read_to_string(&path).ok()?;
    let mut lines = content.lines();
    let version = lines.next()?.to_string();
    let timestamp_secs: u64 = lines.next()?.parse().ok()?;
    let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp_secs);
    Some((version, timestamp))
}

/// Write latest version and current timestamp to cache.
fn write_cache(version: &str) {
    if let Some(path) = cache_path() {
        if let Some(dir) = path.parent() {
            if let Err(e) = fs::create_dir_all(dir) {
                eprintln!("Warning: could not create version cache directory: {}", e);
                return;
            }
        }
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if let Err(e) = fs::write(&path, format!("{}\n{}", version, now)) {
            eprintln!("Warning: could not write version cache: {}", e);
        }
    }
}

/// Fetch the latest release tag from GitHub. Returns version without 'v' prefix.
fn fetch_latest_version() -> Option<String> {
    let output = std::process::Command::new("curl")
        .args([
            "-sSL",
            "--max-time",
            "3",
            "-H",
            "Accept: application/vnd.github+json",
            GITHUB_API_URL,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let body = String::from_utf8(output.stdout).ok()?;

    // Simple JSON parse for "tag_name": "vX.Y.Z"
    let tag_key = "\"tag_name\"";
    let pos = body.find(tag_key)?;
    let after = &body[pos + tag_key.len()..];
    let colon = after.find(':')?;
    let after_colon = &after[colon + 1..];
    let quote_start = after_colon.find('"')? + 1;
    let rest = &after_colon[quote_start..];
    let quote_end = rest.find('"')?;
    let tag = &rest[..quote_end];

    Some(tag.strip_prefix('v').unwrap_or(tag).to_string())
}

/// Check for updates and print a notice if a newer version is available.
/// Skips if: not a TTY, checked recently (within 24h), or network fails.
pub fn check_and_notify() {
    // Only show in interactive terminals
    if !std::io::stderr().is_terminal() {
        return;
    }

    let current_version = env!("CARGO_PKG_VERSION");

    // Check cache first
    if let Some((cached_version, last_check)) = read_cache() {
        if let Ok(elapsed) = SystemTime::now().duration_since(last_check) {
            if elapsed < CHECK_INTERVAL {
                // Cache is fresh — show notice if outdated, then return
                if cached_version != current_version && cached_version.as_str() > current_version
                {
                    print_update_notice(current_version, &cached_version);
                }
                return;
            }
        }
    }

    // Cache expired or missing — fetch from GitHub
    if let Some(latest) = fetch_latest_version() {
        write_cache(&latest);
        if latest != current_version && latest.as_str() > current_version {
            print_update_notice(current_version, &latest);
        }
    }
}

fn print_update_notice(current: &str, latest: &str) {
    let yellow = "\x1b[33m";
    let bold = "\x1b[1m";
    let reset = "\x1b[0m";
    let cyan = "\x1b[36m";

    eprintln!();
    eprintln!(
        "{}╭─────────────────────────────────────────────╮{}",
        yellow, reset
    );
    eprintln!(
        "{}│                                             │{}",
        yellow, reset
    );
    eprintln!(
        "{}│  {}Update available: {} → {}{}{}{}  │{}",
        yellow, bold, current, cyan, latest, reset, yellow, reset
    );
    eprintln!(
        "{}│  Run {}skillset self-update{} to update     │{}",
        yellow, bold, reset, reset
    );
    eprintln!(
        "{}│                                             │{}",
        yellow, reset
    );
    eprintln!(
        "{}╰─────────────────────────────────────────────╯{}",
        yellow, reset
    );
}
