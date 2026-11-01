use anyhow::{Context, Result, anyhow};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

const REPO: &str = "Jorgvt/climg";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct SelfUpdater;

impl SelfUpdater {
    /// Detect target tuple (e.g. x86_64-unknown-linux-musl, aarch64-apple-darwin)
    pub fn detect_target() -> Result<&'static str> {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        match (os, arch) {
            ("linux", "x86_64") => Ok("x86_64-unknown-linux-musl"),
            ("linux", "aarch64") => Ok("aarch64-unknown-linux-musl"),
            ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
            ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
            _ => Err(anyhow!(
                "Unsupported operating system/architecture for self-update: {}/{}",
                os,
                arch
            )),
        }
    }

    /// Check and perform self-update to latest GitHub Release
    pub fn update() -> Result<()> {
        let current_exe = env::current_exe()
            .context("Failed to determine current executable path")?;

        println!("Current climg version: v{}", CURRENT_VERSION);
        println!("Checking for updates on GitHub ({})...", REPO);

        let target = Self::detect_target()?;

        // Query GitHub API or latest release redirect for newest version tag
        let latest_tag = Self::fetch_latest_tag()?;
        let clean_latest = latest_tag.trim_start_matches('v');

        if clean_latest == CURRENT_VERSION {
            println!("climg is already up to date (v{}).", CURRENT_VERSION);
            return Ok(());
        }

        println!("New version available: v{} -> {}", CURRENT_VERSION, latest_tag);

        let asset_url = format!(
            "https://github.com/{}/releases/download/{}/climg-{}.tar.gz",
            REPO, latest_tag, target
        );

        println!("Downloading {}...", asset_url);

        let temp_dir = tempfile::Builder::new()
            .prefix("climg_update_")
            .tempdir()
            .context("Failed to create temporary directory")?;

        let tar_gz_path = temp_dir.path().join("climg.tar.gz");

        // Download asset
        Self::download_file(&asset_url, &tar_gz_path)?;

        // Extract tar.gz
        Self::extract_tar_gz(&tar_gz_path, temp_dir.path())?;

        let new_binary = temp_dir.path().join("climg");
        if !new_binary.exists() {
            return Err(anyhow!("Extracted release archive did not contain 'climg' binary"));
        }

        // Set executable permissions on unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&new_binary)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&new_binary, perms)?;
        }

        // Atomically replace executable
        Self::replace_binary(&new_binary, &current_exe)?;

        println!("Successfully updated climg to {}!", latest_tag);
        println!("Installed at: {}", current_exe.display());

        Ok(())
    }

    fn fetch_latest_tag() -> Result<String> {
        let api_url = format!("https://api.github.com/repos/{}/releases/latest", REPO);

        // Try curl first
        let curl_output = Command::new("curl")
            .arg("-fsSL")
            .arg("-H")
            .arg("User-Agent: climg-updater")
            .arg(&api_url)
            .output();

        if let Ok(out) = curl_output {
            if out.status.success() {
                let json_str = String::from_utf8_lossy(&out.stdout);
                if let Some(tag) = Self::parse_tag_name_from_json(&json_str) {
                    return Ok(tag);
                }
            }
        }

        // Fallback: Query redirect location from releases/latest
        let redirect_url = format!("https://github.com/{}/releases/latest", REPO);
        let header_output = Command::new("curl")
            .arg("-sI")
            .arg(&redirect_url)
            .output();

        if let Ok(out) = header_output {
            let headers = String::from_utf8_lossy(&out.stdout);
            for line in headers.lines() {
                let lower = line.to_lowercase();
                if lower.starts_with("location:") {
                    if let Some(tag) = line.split("/tag/").nth(1) {
                        return Ok(tag.trim().to_string());
                    }
                }
            }
        }

        Err(anyhow!(
            "Failed to retrieve latest release information from GitHub. Please check your internet connection."
        ))
    }

    pub fn parse_tag_name_from_json(json_str: &str) -> Option<String> {
        let key = "\"tag_name\":";
        if let Some(pos) = json_str.find(key) {
            let rest = &json_str[pos + key.len()..];
            let rest = rest.trim_start();
            if let Some(stripped) = rest.strip_prefix('"') {
                if let Some(end_quote) = stripped.find('"') {
                    return Some(stripped[..end_quote].to_string());
                }
            }
        }
        None
    }

    fn download_file(url: &str, dest: &Path) -> Result<()> {
        let status = Command::new("curl")
            .arg("-fsSL")
            .arg("-o")
            .arg(dest)
            .arg(url)
            .status()
            .context("Failed to run curl for download. Make sure curl is installed.")?;

        if !status.success() {
            return Err(anyhow!("Failed to download update asset from: {}", url));
        }

        Ok(())
    }

    fn extract_tar_gz(archive_path: &Path, extract_to: &Path) -> Result<()> {
        let status = Command::new("tar")
            .arg("-xzf")
            .arg(archive_path)
            .arg("-C")
            .arg(extract_to)
            .status()
            .context("Failed to execute tar command to extract release archive")?;

        if !status.success() {
            return Err(anyhow!("tar extraction failed with non-zero exit status"));
        }

        Ok(())
    }

    fn replace_binary(new_bin: &Path, target: &Path) -> Result<()> {
        let backup = target.with_extension("old");

        // 1. Move old binary to .old (safely handles running executable file lock on unix and windows)
        if target.exists() {
            let _ = fs::rename(target, &backup);
        }

        // 2. Move new binary to target
        if let Err(e) = fs::copy(new_bin, target) {
            // Restore backup if copy failed
            if backup.exists() {
                let _ = fs::rename(&backup, target);
            }
            return Err(anyhow!(
                "Failed to replace binary at {}: {}. Do you need sudo / write permissions?",
                target.display(),
                e
            ));
        }

        // Remove backup
        if backup.exists() {
            let _ = fs::remove_file(backup);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tag_name() {
        let json = r#"{ "tag_name": "v0.2.0", "name": "Release v0.2.0" }"#;
        assert_eq!(SelfUpdater::parse_tag_name_from_json(json), Some("v0.2.0".to_string()));
    }

    #[test]
    fn test_detect_target() {
        let target = SelfUpdater::detect_target();
        assert!(target.is_ok());
    }
}
