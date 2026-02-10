//! Auto-update from GitHub Releases.
//!
//! Requires feature `self-update`. Release assets should be named:
//! - `velocity-{version}-{target}.tar.gz` (e.g. velocity-0.1.0-x86_64-pc-windows-msvc.tar.gz)
//! - or `velocity-{version}-{target}.zip`

const GITHUB_OWNER: &str = "lonestill";
const GITHUB_REPO: &str = "velocity-client";
const BIN_NAME: &str = "velocity";

/// Check if a newer version is available on GitHub.
#[cfg(feature = "self-update")]
pub fn check_for_updates() -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    use self_update::version::bump_is_greater;
    let current = env!("CARGO_PKG_VERSION");
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner(GITHUB_OWNER)
        .repo_name(GITHUB_REPO)
        .build()?
        .fetch()?;

    let latest = releases.first();
    let Some(latest) = latest else {
        return Ok(None);
    };

    let latest_version = latest.version.trim_start_matches('v');
    if bump_is_greater(latest_version, current)? {
        Ok(Some(latest_version.to_string()))
    } else {
        Ok(None)
    }
}

#[cfg(not(feature = "self-update"))]
pub fn check_for_updates() -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(None)
}

/// Download and apply the latest update. Restarts the app after update.
#[cfg(feature = "self-update")]
pub fn perform_update() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner(GITHUB_OWNER)
        .repo_name(GITHUB_REPO)
        .bin_name(BIN_NAME)
        .current_version(env!("CARGO_PKG_VERSION"))
        .show_download_progress(true)
        .build()?
        .update()?;

    if let self_update::Status::Updated(_) = status {
        #[cfg(unix)]
        {
            std::process::Command::new(std::env::current_exe()?)
                .args(std::env::args().skip(1))
                .spawn()?;
            std::process::exit(0);
        }
        #[cfg(windows)]
        {
            std::process::Command::new("cmd")
                .args([
                    "/C",
                    "start",
                    "",
                    std::env::current_exe()?.to_str().unwrap_or("velocity.exe"),
                ])
                .spawn()?;
            std::process::exit(0);
        }
    }
    Ok(())
}

#[cfg(not(feature = "self-update"))]
pub fn perform_update() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err("self-update feature is disabled".into())
}
