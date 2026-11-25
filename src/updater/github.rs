//! Published GitHub installers are offered as downloads, never executed or
//! silently installed. Platform signature verification for self-replacement
//! remains in the original updater; unsigned previews cannot use that path.
use super::{Shared, State, lock, wait};
use color_eyre::eyre::{Result, bail};
use serde::Deserialize;
use std::io::Read;
use std::time::Duration;

const REPOSITORY: &str = "Sudeepnt/Utter";
const API: &str = "https://api.github.com/repos/Sudeepnt/Utter/releases?per_page=10";
const INTERVAL: Duration = Duration::from_secs(15 * 60);

pub(super) fn enabled() -> bool {
    option_env!("UTTER_GITHUB_UPDATES") == Some("1")
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    state: String,
    size: u64,
}
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

fn version(value: &str) -> Option<(u64, u64, u64)> {
    let core = value.strip_prefix('v').unwrap_or(value).split('-').next()?;
    let parts = core.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}
fn trusted_url(url: &str, tag: &str) -> bool {
    let Ok(url) = url::Url::parse(url) else {
        return false;
    };
    url.scheme() == "https"
        && url.host_str() == Some("github.com")
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && url
            .path()
            .starts_with(&format!("/{REPOSITORY}/releases/download/{tag}/"))
}
fn select(releases: &[Release], target: &str, running: &str, previews: bool) -> Option<String> {
    let running = version(running)?;
    releases
        .iter()
        .filter(|r| !r.draft && (previews || !r.prerelease))
        .filter_map(|r| Some((version(&r.tag_name)?, r)))
        .filter(|(v, _)| *v > running)
        .filter_map(|(v, r)| {
            let core = format!("{}.{}.{}", v.0, v.1, v.2);
            let names = match target {
                "macos-arm64" => vec![
                    format!("utter-{core}-macos-arm64.dmg"),
                    format!("utter-{core}-macos-arm64-local-unsigned.dmg"),
                ],
                "windows-x64" => vec![
                    format!("utter-setup-{core}-windows-x64.exe"),
                    "UtterSetup-x64.exe".into(),
                ],
                "windows-arm64" => vec!["UtterSetup-arm64.exe".into()],
                _ => return None,
            };
            let valid = |asset: &&Asset| {
                asset.state == "uploaded"
                    && asset.size > 0
                    && trusted_url(&asset.browser_download_url, &r.tag_name)
            };
            let file = r.assets.iter().filter(valid).find(|a| {
                names.contains(&a.name) && (r.prerelease || !a.name.contains("local-unsigned"))
            })?;
            r.assets.iter().filter(valid).find(|a| {
                a.name == "SHA256SUMS.txt" || a.name == format!("{}.sha256", file.name)
            })?;
            Some((v, file.browser_download_url.clone()))
        })
        .max_by_key(|(v, _)| *v)
        .map(|(_, url)| url)
}
fn fetch() -> Result<Vec<Release>> {
    let agent = crate::net::agent(API, |config| {
        config
            .https_only(true)
            .timeout_global(Some(Duration::from_secs(15)))
            .http_status_as_error(false)
    });
    let mut response = agent
        .get(API)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "Utter-update-check")
        .call()?;
    if response.status().as_u16() == 404 {
        return Ok(Vec::new());
    }
    if response.status().as_u16() != 200 {
        bail!("GitHub update check returned {}", response.status());
    }
    let mut body = String::new();
    response
        .body_mut()
        .with_config()
        .limit(256 * 1024 + 1)
        .reader()
        .take(256 * 1024 + 1)
        .read_to_string(&mut body)?;
    if body.len() > 256 * 1024 {
        bail!("GitHub release response is too large");
    }
    Ok(serde_json::from_str(&body)?)
}
pub(super) fn run(shared: &Shared) {
    if wait(shared, Duration::from_secs(10)) {
        return;
    }
    loop {
        let before = lock(&shared.state).clone();
        let previews = super::BUILD_CHANNEL == Some("beta");
        *lock(&shared.last) = Some(("github".into(), std::time::Instant::now()));
        match fetch() {
            Ok(releases) => {
                let target = super::platform::TARGET;
                let next = match select(&releases, target, crate::VERSION, previews) {
                    Some(url) => State::Download { url },
                    None => State::Current,
                };
                *lock(&shared.state) = next;
            }
            Err(error) => {
                tracing::warn!(%error,"could not check GitHub for updates");
                *lock(&shared.state) = if matches!(before, State::Download { .. }) {
                    before
                } else {
                    State::Failed
                };
            }
        }
        if wait(shared, INTERVAL) {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn release(v: &str) -> Release {
        let tag = format!("v{v}");
        let asset = |name: String| Asset {
            browser_download_url: format!(
                "https://github.com/{REPOSITORY}/releases/download/{tag}/{name}"
            ),
            name,
            state: "uploaded".into(),
            size: 100,
        };
        Release {
            tag_name: tag,
            draft: false,
            prerelease: false,
            assets: vec![
                asset(format!("utter-{v}-macos-arm64.dmg")),
                asset("SHA256SUMS.txt".into()),
            ],
        }
    }
    #[test]
    fn only_newer_versions_are_offered() {
        assert!(select(&[release("3.4.2")], "macos-arm64", "3.4.1", false).is_some());
        assert!(
            select(
                &[release("3.4.1"), release("3.3.9")],
                "macos-arm64",
                "3.4.1",
                false
            )
            .is_none()
        );
        assert!(version("unexpected").is_none());
    }
    #[test]
    fn missing_checksums_and_foreign_urls_are_rejected() {
        let mut r = release("3.4.2");
        r.assets.pop();
        assert!(select(&[r], "macos-arm64", "3.4.1", false).is_none());
        let mut r = release("3.4.2");
        r.assets[0].browser_download_url = "https://example.com/installer.dmg".into();
        assert!(select(&[r], "macos-arm64", "3.4.1", false).is_none());
    }
    #[test]
    fn previews_are_opt_in_and_drafts_are_never_offered() {
        let mut r = release("3.4.2");
        r.prerelease = true;
        assert!(select(&[r], "macos-arm64", "3.4.1", false).is_none());
        let mut r = release("3.4.2");
        r.draft = true;
        assert!(select(&[r], "macos-arm64", "3.4.1", true).is_none());
    }
}
