//! Where a marketplace's files come from, and reading them.
//!
//! Every kind of source ends up as a `Root`: an HTTP base URL (GitHub and HTTPS sources) or a
//! directory (local folders, and git sources after a shallow clone).

use std::path::{Path, PathBuf};
use std::time::Duration;

use sha2::{Digest, Sha256};
use url::Url;

use crate::catalog::{self, Catalog, CatalogModule};
use crate::MarketError;

pub const MAX_CATALOG: usize = 1024 * 1024;
pub const MAX_MANIFEST: usize = 64 * 1024;
pub const MAX_ENTRY: usize = 16 * 1024 * 1024;

/// What the user typed when adding a marketplace, understood.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Location {
    /// The catalog file's URL.
    Http(Url),
    Git(String),
    Dir(PathBuf),
}

impl Location {
    /// `owner/repo`, `https://github.com/owner/repo`, a catalog URL, a git URL, or an absolute
    /// folder path.
    pub fn parse(input: &str) -> Result<Self, MarketError> {
        let input = input.trim();
        let github = |owner: &str, repo: &str| {
            let raw = format!("https://raw.githubusercontent.com/{owner}/{repo}/HEAD/{}", catalog::FILE);
            Ok(Location::Http(Url::parse(&raw).expect("valid URL")))
        };
        if Path::new(input).is_absolute() {
            return Ok(Location::Dir(PathBuf::from(input)));
        }
        if ["git@", "ssh://", "git://", "file://"].iter().any(|p| input.starts_with(p)) {
            return Ok(Location::Git(input.into()));
        }
        if let Ok(url) = Url::parse(input) {
            if !matches!(url.scheme(), "http" | "https") {
                return Err(MarketError::Invalid(format!("{input:?}: unsupported scheme {}", url.scheme())));
            }
            if url.path().ends_with(".git") {
                return Ok(Location::Git(input.into()));
            }
            if url.host_str() == Some("github.com") {
                let parts: Vec<&str> = url.path().trim_matches('/').split('/').collect();
                if let [owner, repo] = parts.as_slice() {
                    return github(owner, repo);
                }
            }
            check_http(&url)?;
            return Ok(Location::Http(url));
        }
        let simple = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b));
        if let Some((owner, repo)) = input.split_once('/').filter(|(o, r)| simple(o) && simple(r)) {
            return github(owner, repo);
        }
        Err(MarketError::Invalid(format!(
            "{input:?} is not a marketplace: use owner/repo (GitHub), an https:// catalog URL, a git URL or a folder path"
        )))
    }

    /// Local folders are read in place; everything else is copied on install and must pin
    /// its modules with sha256.
    pub fn is_local(&self) -> bool {
        matches!(self, Location::Dir(_))
    }
}

/// Remote files only over HTTPS, except from this machine (development servers).
fn check_http(url: &Url) -> Result<(), MarketError> {
    let loopback = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if url.scheme() == "https" || (url.scheme() == "http" && loopback) {
        Ok(())
    } else {
        Err(MarketError::Invalid(format!("{url}: marketplaces must use https")))
    }
}

pub enum Root {
    /// Files are relative to this URL (the catalog's).
    Http(Url),
    Dir(PathBuf),
}

impl Root {
    pub async fn read(&self, http: &reqwest::Client, path: &str, limit: usize) -> Result<Vec<u8>, MarketError> {
        match self {
            Root::Http(base) => {
                let url = base.join(path).map_err(|e| MarketError::Invalid(format!("{path}: {e}")))?;
                check_http(&url)?;
                fetch(http, &url, limit).await
            }
            Root::Dir(dir) => {
                let file = dir.join(path);
                let bytes = tokio::fs::read(&file).await.map_err(|e| match e.kind() {
                    std::io::ErrorKind::NotFound => MarketError::NotFound(file.display().to_string()),
                    _ => MarketError::Io(format!("{}: {e}", file.display())),
                })?;
                if bytes.len() > limit {
                    return Err(MarketError::Invalid(format!("{} is larger than {limit} bytes", file.display())));
                }
                Ok(bytes)
            }
        }
    }

    /// The catalog, or for a folder that is a single module, a catalog of just that module.
    pub async fn catalog(&self, http: &reqwest::Client) -> Result<Catalog, MarketError> {
        let bytes = match self.read(http, catalog::FILE, MAX_CATALOG).await {
            Err(MarketError::NotFound(_)) if matches!(self, Root::Dir(_)) => {
                let manifest = self.read(http, "manifest.json", MAX_MANIFEST).await.map_err(|_| {
                    MarketError::NotFound(format!("no {} or manifest.json in this folder", catalog::FILE))
                })?;
                let manifest = zoo_module_client::manifest::Manifest::parse(&String::from_utf8_lossy(&manifest))
                    .map_err(MarketError::Invalid)?;
                return Ok(Catalog {
                    name: format!("dev-{}", manifest.id),
                    owner: manifest.author.clone(),
                    modules: vec![CatalogModule {
                        id: manifest.id,
                        version: manifest.version,
                        source: ".".into(),
                        description: manifest.description,
                        sha256: None,
                        api_version: Some(manifest.api_version),
                        needs_newer_app: false,
                    }],
                });
            }
            other => other?,
        };
        Catalog::parse(&bytes).map_err(MarketError::Invalid)
    }
}

async fn fetch(http: &reqwest::Client, url: &Url, limit: usize) -> Result<Vec<u8>, MarketError> {
    let network = |e: reqwest::Error| MarketError::Network(format!("{url}: {e}"));
    let mut response = http.get(url.clone()).send().await.map_err(network)?;
    match response.status().as_u16() {
        200..=299 => {}
        404 => return Err(MarketError::NotFound(url.to_string())),
        status => return Err(MarketError::Network(format!("{url}: HTTP {status}"))),
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(network)? {
        body.extend_from_slice(&chunk);
        if body.len() > limit {
            return Err(MarketError::Invalid(format!("{url} is larger than {limit} bytes")));
        }
    }
    Ok(body)
}

pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(120))
        .user_agent(concat!("zoo-downloader/", env!("CARGO_PKG_VERSION")))
        // A redirect must not downgrade to plain http.
        .redirect(reqwest::redirect::Policy::custom(|attempt| match check_http(attempt.url()) {
            Ok(()) if attempt.previous().len() < 10 => attempt.follow(),
            Ok(()) => attempt.error("too many redirects"),
            Err(e) => attempt.error(e),
        }))
        .build()
        .expect("HTTP client builds")
}

/// Turn a location into a readable root. A git source is cloned (or updated) under `cache`;
/// returns its commit.
pub async fn materialize(location: &Location, cache: &Path) -> Result<(Root, Option<String>), MarketError> {
    match location {
        Location::Http(url) => Ok((Root::Http(url.clone()), None)),
        Location::Dir(dir) => Ok((Root::Dir(dir.clone()), None)),
        Location::Git(url) => {
            let dir = git_dir(cache, url);
            if dir.join(".git").is_dir() {
                git(Some(&dir), &["fetch", "--depth", "1", "origin", "HEAD"]).await?;
                git(Some(&dir), &["reset", "--hard", "FETCH_HEAD"]).await?;
            } else {
                let _ = std::fs::remove_dir_all(&dir);
                std::fs::create_dir_all(cache).map_err(|e| MarketError::Io(e.to_string()))?;
                git(None, &["clone", "--depth", "1", "--", url, &dir.to_string_lossy()]).await?;
            }
            let commit = git(Some(&dir), &["rev-parse", "HEAD"]).await?;
            Ok((Root::Dir(dir), Some(commit)))
        }
    }
}

/// Where a git source is checked out.
pub fn git_dir(cache: &Path, url: &str) -> PathBuf {
    let hash = Sha256::digest(url.as_bytes());
    cache.join(hex(&hash[..8]))
}

async fn git(dir: Option<&Path>, args: &[&str]) -> Result<String, MarketError> {
    let mut command = tokio::process::Command::new("git");
    if let Some(dir) = dir {
        command.arg("-C").arg(dir);
    }
    // Never prompt for credentials: the app has no terminal.
    command.args(args).env("GIT_TERMINAL_PROMPT", "0").stdin(std::process::Stdio::null());
    let output = command.output().await.map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => MarketError::Git("git is not installed; install Git to add git marketplaces".into()),
        _ => MarketError::Git(format!("cannot run git: {e}")),
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MarketError::Git(format!("git {} failed: {}", args[0], stderr.trim())));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn understands_every_kind_of_location() {
        let raw = "https://raw.githubusercontent.com/zoo/mods/HEAD/zoo-marketplace.json";
        assert_eq!(Location::parse("zoo/mods").unwrap(), Location::Http(Url::parse(raw).unwrap()));
        assert_eq!(Location::parse("https://github.com/zoo/mods").unwrap(), Location::Http(Url::parse(raw).unwrap()));
        let catalog = "https://example.com/m/zoo-marketplace.json";
        assert_eq!(Location::parse(catalog).unwrap(), Location::Http(Url::parse(catalog).unwrap()));
        assert!(matches!(Location::parse("https://gitlab.com/a/b.git").unwrap(), Location::Git(_)));
        assert!(matches!(Location::parse("git@github.com:a/b.git").unwrap(), Location::Git(_)));
        let dir = if cfg!(windows) { r"C:\modules\dev" } else { "/modules/dev" };
        assert_eq!(Location::parse(dir).unwrap(), Location::Dir(dir.into()));
        assert!(Location::parse("http://example.com/c.json").is_err(), "plain http is refused");
        assert!(Location::parse("http://127.0.0.1:8080/c.json").is_ok(), "except locally");
        assert!(Location::parse("ftp://x/y").is_err());
        assert!(Location::parse("just words").is_err());
    }
}
