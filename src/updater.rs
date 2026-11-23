use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::process::Command;
use tauri::{AppHandle, Emitter};

pub const GITHUB_REPO: &str = "Iwant2dye8976/roklootswatcher";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub release_notes: String,
    pub release_name: String,
    pub published_at: String,
    pub download_url: Option<String>,
    pub sha256_url: Option<String>,
    pub asset_name: Option<String>,
    pub is_installer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgress {
    pub percent: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub status: String,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    published_at: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    #[allow(dead_code)]
    size: u64,
}

/// Parses semantic version tags like "v2.21.0" or "2.21.0" into numeric triples
pub fn parse_semver(v: &str) -> Option<(u64, u64, u64)> {
    let cleaned = v.trim().trim_start_matches('v');
    let parts: Vec<&str> = cleaned.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    let major = parts[0].parse::<u64>().ok()?;
    let minor = parts[1].parse::<u64>().ok()?;
    // Handle suffix like "0-beta" or "0"
    let patch_str = parts[2].split('-').next().unwrap_or("0");
    let patch = patch_str.parse::<u64>().ok()?;
    Some((major, minor, patch))
}

pub fn is_newer_version(current: &str, latest: &str) -> bool {
    let cur = match parse_semver(current) {
        Some(v) => v,
        None => return false,
    };
    let lat = match parse_semver(latest) {
        Some(v) => v,
        None => return false,
    };

    lat > cur
}

/// Checks the latest GitHub release for roklootswatcher
pub fn check_for_updates(current_version: &str) -> Result<UpdateInfo, String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let resp = ureq::get(&url)
        .set("User-Agent", "roklootswatcher-updater")
        .set("Accept", "application/vnd.github.v3+json")
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| format!("Failed to connect to GitHub update server: {}", e))?;

    let release: GitHubRelease = resp
        .into_json()
        .map_err(|e| format!("Failed to parse release metadata: {}", e))?;

    let latest_version = release.tag_name.trim().trim_start_matches('v').to_string();
    let has_update = is_newer_version(current_version, &latest_version);

    // Find best asset:
    // If running from an installed path (e.g. Program Files) or setup asset exists, prefer setup or portable exe
    let mut selected_asset: Option<&GitHubAsset> = None;
    let mut sha256_url: Option<String> = None;

    // Check if we are running in an installed directory or portable
    let is_running_installer = if let Ok(exe) = env::current_exe() {
        let path_str = exe.to_string_lossy().to_lowercase();
        path_str.contains("program files")
    } else {
        false
    };

    for asset in &release.assets {
        let name_lower = asset.name.to_lowercase();
        if name_lower.ends_with(".sha256") || name_lower.contains("sha256sums") {
            sha256_url = Some(asset.browser_download_url.clone());
        }
    }

    if is_running_installer {
        // Look for setup installer asset first
        selected_asset = release.assets.iter().find(|a| {
            let n = a.name.to_lowercase();
            n.ends_with(".exe") && (n.contains("setup") || n.contains("installer"))
        });
    }

    // If no setup found or running portable, pick rok_watcher.exe or any .exe
    if selected_asset.is_none() {
        selected_asset = release.assets.iter().find(|a| {
            let n = a.name.to_lowercase();
            n == "rok_watcher.exe" || (n.ends_with(".exe") && !n.contains("setup") && !n.contains("installer"))
        });
    }

    // Fallback: any .exe asset
    if selected_asset.is_none() {
        selected_asset = release.assets.iter().find(|a| a.name.to_lowercase().ends_with(".exe"));
    }

    let (download_url, asset_name, is_installer) = match selected_asset {
        Some(a) => {
            let n = a.name.to_lowercase();
            let is_inst = n.contains("setup") || n.contains("installer");
            (Some(a.browser_download_url.clone()), Some(a.name.clone()), is_inst)
        }
        None => (None, None, false),
    };

    Ok(UpdateInfo {
        current_version: current_version.to_string(),
        latest_version,
        has_update,
        release_notes: release.body.unwrap_or_default(),
        release_name: release.name.unwrap_or_else(|| release.tag_name.clone()),
        published_at: release.published_at.unwrap_or_default(),
        download_url,
        sha256_url,
        asset_name,
        is_installer,
    })
}

/// Download an update file with progress reporting and apply it.
pub fn download_and_apply_update(
    app: &AppHandle,
    download_url: &str,
    is_installer: bool,
) -> Result<(), String> {
    let current_exe = env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    let exe_dir = current_exe
        .parent()
        .ok_or_else(|| "Failed to get executable parent directory".to_string())?;

    let resp = ureq::get(download_url)
        .set("User-Agent", "roklootswatcher-updater")
        .timeout(std::time::Duration::from_secs(120))
        .call()
        .map_err(|e| format!("Failed to download update: {}", e))?;

    let total_bytes: u64 = resp
        .header("Content-Length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let temp_download_path = if is_installer {
        exe_dir.join("rok_watcher_setup_update.exe")
    } else {
        exe_dir.join("rok_watcher_update.exe")
    };

    let mut out_file = File::create(&temp_download_path)
        .map_err(|e| format!("Failed to create temporary update file {:?}: {}", temp_download_path, e))?;

    let mut reader = resp.into_reader();
    let mut buffer = [0u8; 64 * 1024];
    let mut downloaded_bytes: u64 = 0;
    let mut last_emit = std::time::Instant::now();

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| format!("Error while downloading stream: {}", e))?;
        if bytes_read == 0 {
            break;
        }

        out_file
            .write_all(&buffer[..bytes_read])
            .map_err(|e| format!("Failed to write downloaded chunk: {}", e))?;

        downloaded_bytes += bytes_read as u64;

        if last_emit.elapsed() > std::time::Duration::from_millis(150) || downloaded_bytes == total_bytes {
            last_emit = std::time::Instant::now();
            let percent = if total_bytes > 0 {
                (downloaded_bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };

            let _ = app.emit(
                "updater://progress",
                UpdateProgress {
                    percent,
                    downloaded_bytes,
                    total_bytes,
                    status: "downloading".to_string(),
                },
            );
        }
    }

    out_file.flush().map_err(|e| format!("Flush failed: {}", e))?;
    drop(out_file);

    let _ = app.emit(
        "updater://progress",
        UpdateProgress {
            percent: 100.0,
            downloaded_bytes,
            total_bytes: downloaded_bytes,
            status: "ready_to_install".to_string(),
        },
    );

    // Apply update
    if is_installer {
        // Run installer and exit
        let _ = Command::new(&temp_download_path).spawn().map_err(|e| {
            format!("Failed to launch installer {:?}: {}", temp_download_path, e)
        })?;
        crate::tauri_app::shutdown_app_gracefully(app);
    } else {
        // Standalone EXE swap:
        // Create a batch script that waits for current process to terminate, replaces the exe, and starts it
        let current_pid = std::process::id();
        let bat_path = exe_dir.join("rok_update_runner.bat");

        let bat_script = format!(
            "@echo off\r\n\
             setlocal\r\n\
             :wait_loop\r\n\
             tasklist /FI \"PID eq {pid}\" 2>NUL | find /I \"{pid}\" >NUL\r\n\
             if not errorlevel 1 (\r\n\
                 timeout /T 1 /NOBREAK >NUL\r\n\
                 goto wait_loop\r\n\
             )\r\n\
             timeout /T 1 /NOBREAK >NUL\r\n\
             copy /Y \"{temp_exe}\" \"{target_exe}\" >NUL\r\n\
             if errorlevel 1 (\r\n\
                 timeout /T 2 /NOBREAK >NUL\r\n\
                 copy /Y \"{temp_exe}\" \"{target_exe}\" >NUL\r\n\
             )\r\n\
             del /F /Q \"{temp_exe}\" >NUL 2>&1\r\n\
             start \"\" \"{target_exe}\"\r\n\
             del /F /Q \"%~f0\" >NUL 2>&1\r\n\
             exit\r\n",
            pid = current_pid,
            temp_exe = temp_download_path.to_string_lossy(),
            target_exe = current_exe.to_string_lossy(),
        );

        fs::write(&bat_path, bat_script)
            .map_err(|e| format!("Failed to create updater helper script: {}", e))?;

        // Launch helper detached in background
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            Command::new("cmd")
                .args(["/C", &bat_path.to_string_lossy()])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .map_err(|e| format!("Failed to execute updater helper script: {}", e))?;
        }
        #[cfg(not(windows))]
        {
            Command::new("sh")
                .args(["-c", &bat_path.to_string_lossy()])
                .spawn()
                .map_err(|e| format!("Failed to execute updater script: {}", e))?;
        }

        crate::tauri_app::shutdown_app_gracefully(app);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver("v2.21.0"), Some((2, 21, 0)));
        assert_eq!(parse_semver("2.21.0"), Some((2, 21, 0)));
        assert_eq!(parse_semver("v1.0.5-beta"), Some((1, 0, 5)));
        assert_eq!(parse_semver("invalid"), None);
    }

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("2.21.0", "2.21.1"));
        assert!(is_newer_version("2.21.0", "2.22.0"));
        assert!(is_newer_version("2.21.0", "3.0.0"));
        assert!(!is_newer_version("2.21.0", "2.21.0"));
        assert!(!is_newer_version("2.21.1", "2.21.0"));
        assert!(!is_newer_version("3.0.0", "2.25.0"));
    }
}
