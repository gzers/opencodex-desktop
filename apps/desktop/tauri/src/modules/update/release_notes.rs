//! Exact-version release details. User-requested only; no scheduler/install side effects.
use crate::modules::runtime_defaults::ReleaseNotesDefaults;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    Manager,
    Official,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    Stable,
    Beta,
    Official,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotesState {
    Ready,
    Unavailable,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Details {
    pub target: Target,
    pub channel: Channel,
    pub candidate_version: String,
    pub notes_state: NotesState,
    pub notes: Option<String>,
    pub notes_source: Option<String>,
    pub release_url: Option<String>,
    pub published_at: Option<String>,
    pub fetched_at: i64,
    pub cached: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Source,
    Network,
    Size,
    Cache,
}

#[async_trait]
pub trait Source: Send + Sync {
    /// None is an actual HTTP 404; errors are not publisher absence.
    async fn get(&self, url: &str) -> Result<Option<Vec<u8>>, Error>;
}

pub struct HttpSource {
    client: tokio::sync::OnceCell<reqwest::Client>,
    policy: crate::modules::preferences::ProxyPolicy,
    total_seconds: u64,
    max_bytes: u64,
}
impl HttpSource {
    pub fn new(
        policy: crate::modules::preferences::ProxyPolicy,
        limits: &ReleaseNotesDefaults,
    ) -> Result<Self, Error> {
        if limits.total_seconds == 0 || limits.response_max_bytes == 0 {
            return Err(Error::Source);
        }
        Ok(Self {
            client: tokio::sync::OnceCell::new(),
            policy,
            total_seconds: limits.total_seconds,
            max_bytes: limits.response_max_bytes,
        })
    }
    async fn client(&self) -> Result<&reqwest::Client, Error> {
        self.client
            .get_or_try_init(|| async {
                let policy = self.policy.clone();
                let total_seconds = self.total_seconds;
                // Proxy/certificate discovery is not needed for offline cache hits,
                // and must not block the async executor on an actual cache miss.
                tokio::task::spawn_blocking(move || Self::build_client(policy, total_seconds))
                    .await
                    .map_err(|_| Error::Network)?
            })
            .await
    }
    fn build_client(
        policy: crate::modules::preferences::ProxyPolicy,
        total_seconds: u64,
    ) -> Result<reqwest::Client, Error> {
        use crate::modules::preferences::ProxyPolicy;
        let builder = reqwest::Client::builder()
            .user_agent(crate::modules::network_defaults::user_agent())
            .connect_timeout(crate::modules::network_defaults::connect_timeout())
            .timeout(Duration::from_secs(total_seconds))
            .redirect(reqwest::redirect::Policy::none());
        let builder = match policy {
            ProxyPolicy::None => builder.no_proxy(),
            ProxyPolicy::System => builder,
            ProxyPolicy::Manual(url) => {
                builder.proxy(reqwest::Proxy::all(url).map_err(|_| Error::Source)?)
            }
        };
        builder.build().map_err(|_| Error::Network)
    }
}
#[async_trait]
impl Source for HttpSource {
    async fn get(&self, url: &str) -> Result<Option<Vec<u8>>, Error> {
        let mut response = self
            .client()
            .await?
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|_| Error::Network)?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(Error::Network);
        }
        if response
            .content_length()
            .is_some_and(|n| n > self.max_bytes)
        {
            return Err(Error::Size);
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| Error::Network)? {
            if bytes.len() as u64 + chunk.len() as u64 > self.max_bytes {
                return Err(Error::Size);
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(Some(bytes))
    }
}

fn key(target: Target, channel: Channel) -> Result<&'static str, Error> {
    match (target, channel) {
        (Target::Manager, Channel::Stable) => Ok("manager-stable"),
        (Target::Manager, Channel::Beta) => Ok("manager-beta"),
        (Target::Official, Channel::Official) => Ok("panel-latest"),
        _ => Err(Error::Source),
    }
}
fn repository_valid(repo: &str) -> bool {
    let parts: Vec<_> = repo.split('/').collect();
    parts.len() == 2
        && parts.iter().all(|p| {
            !p.is_empty()
                && *p != "."
                && *p != ".."
                && p.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
        })
}
pub fn valid_request(target: Target, channel: Channel, version: &str) -> bool {
    key(target, channel).is_ok() && super::handoff::valid_version(version)
}
pub fn bounded_text(text: Option<&str>, limit: usize) -> Option<String> {
    text.filter(|value| !value.trim().is_empty()).map(|value| {
        let mut end = value.len().min(limit);
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        value[..end].to_owned()
    })
}

fn verify_package(raw: &[u8], version: &str, repo: &str) -> Result<(), Error> {
    let value: serde_json::Value = serde_json::from_slice(raw).map_err(|_| Error::Source)?;
    let expected_repo = format!("git+https://github.com/{repo}.git");
    if value["name"] != crate::modules::runtime::OFFICIAL_PACKAGE
        || value["version"] != version
        || value["repository"]["url"] != expected_repo
    {
        return Err(Error::Source);
    }
    Ok(())
}
fn read_release(
    raw: &[u8],
    details: &mut Details,
    repo: &str,
    limits: &ReleaseNotesDefaults,
) -> Result<(), Error> {
    let value: serde_json::Value = serde_json::from_slice(raw).map_err(|_| Error::Source)?;
    let tag = format!("v{}", details.candidate_version);
    let page = format!("https://github.com/{repo}/releases/tag/{tag}");
    if value["tag_name"] != tag
        || value["draft"] != false
        || value["html_url"] != page
        || !value["prerelease"].is_boolean()
        || details.channel == Channel::Stable && value["prerelease"] != false
    {
        return Err(Error::Source);
    }
    details.release_url = Some(page);
    details.published_at = value["published_at"]
        .as_str()
        .filter(|s| chrono::DateTime::parse_from_rfc3339(s).is_ok())
        .map(str::to_owned);
    if details.notes.is_none() {
        details.notes = bounded_text(value["body"].as_str(), limits.notes_max_bytes);
        details.notes_source = details.notes.as_ref().map(|_| "github_release".to_owned());
    }
    details.notes_state = if details.notes.is_some() {
        NotesState::Ready
    } else {
        NotesState::Unavailable
    };
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Cache {
    schema: u8,
    repository: String,
    details: Details,
}

fn cache_file(root: &Path, name: &str, create: bool) -> Result<PathBuf, Error> {
    let mut path = root.to_path_buf();
    for dir in ["cache", "updates", "release-notes"] {
        path.push(dir);
        match std::fs::symlink_metadata(&path) {
            Ok(meta) if meta.file_type().is_dir() => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && create => {
                if let Err(error) = std::fs::create_dir(&path) {
                    if error.kind() != std::io::ErrorKind::AlreadyExists {
                        return Err(Error::Cache);
                    }
                }
                if !std::fs::symlink_metadata(&path)
                    .map_err(|_| Error::Cache)?
                    .file_type()
                    .is_dir()
                {
                    return Err(Error::Cache);
                }
            }
            _ => return Err(Error::Cache),
        }
    }
    path.push(format!("{name}.json"));
    match std::fs::symlink_metadata(&path) {
        Ok(meta) if !meta.file_type().is_file() => Err(Error::Cache),
        _ => Ok(path),
    }
}
fn cached(
    root: &Path,
    name: &str,
    repo: &str,
    request: &Details,
    limits: &ReleaseNotesDefaults,
) -> Option<Details> {
    let path = cache_file(root, name, false).ok()?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(512 * 1024 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 512 * 1024 {
        return None;
    }
    let cache: Cache = serde_json::from_slice(&bytes).ok()?;
    let d = &cache.details;
    let ttl = if d.notes_state == NotesState::Ready {
        limits.cache_seconds
    } else {
        limits.missing_cache_seconds
    };
    if cache.schema != 1
        || cache.repository != repo
        || d.target != request.target
        || d.channel != request.channel
        || d.candidate_version != request.candidate_version
        || d.fetched_at < 0
        || d.fetched_at > request.fetched_at
        || request.fetched_at - d.fetched_at >= ttl
        || d.notes_state == NotesState::Failed
        || !consistent_notes(d)
        || d.notes
            .as_ref()
            .is_some_and(|s| s.len() > limits.notes_max_bytes)
        || d.release_url.as_ref().is_some_and(|s| {
            s != &format!(
                "https://github.com/{repo}/releases/tag/v{}",
                d.candidate_version
            )
        })
    {
        return None;
    }
    let mut details = cache.details;
    details.cached = true;
    Some(details)
}

fn consistent_notes(details: &Details) -> bool {
    let content = details
        .notes
        .as_deref()
        .is_some_and(|text| !text.trim().is_empty());
    let source = match details.notes_source.as_deref() {
        Some("github_release") => details.release_url.is_some(),
        Some("updater") => details.target == Target::Manager,
        _ => false,
    };
    let state = match details.notes_state {
        NotesState::Ready => content && source,
        NotesState::Unavailable => details.notes.is_none() && details.notes_source.is_none(),
        NotesState::Failed => false,
    };
    state
        && details
            .published_at
            .as_deref()
            .is_none_or(|time| chrono::DateTime::parse_from_rfc3339(time).is_ok())
}

pub struct Request<'a> {
    pub target: Target,
    pub channel: Channel,
    pub version: &'a str,
    pub updater_notes: Option<&'a str>,
    pub force: bool,
    pub now: i64,
}

pub async fn query<S: Source + ?Sized>(
    source: &S,
    root: &Path,
    request: Request<'_>,
    limits: &ReleaseNotesDefaults,
) -> Result<Details, Error> {
    let Request {
        target,
        channel,
        version,
        updater_notes,
        force,
        now,
    } = request;
    if !valid_request(target, channel, version) || now < 0 {
        return Err(Error::Source);
    }
    let name = key(target, channel)?;
    let repo = if target == Target::Manager {
        &limits.manager_repository
    } else {
        &limits.panel_repository
    };
    if !repository_valid(repo) {
        return Err(Error::Source);
    }
    let notes = if target == Target::Manager {
        bounded_text(updater_notes, limits.notes_max_bytes)
    } else {
        None
    };
    let mut details = Details {
        target,
        channel,
        candidate_version: version.into(),
        notes_state: if notes.is_some() {
            NotesState::Ready
        } else {
            NotesState::Unavailable
        },
        notes_source: notes.as_ref().map(|_| "updater".into()),
        notes,
        release_url: None,
        published_at: None,
        fetched_at: now,
        cached: false,
    };
    if !force {
        let cache_root = root.to_path_buf();
        let cache_repo = repo.clone();
        let cache_request = details.clone();
        let cache_limits = limits.clone();
        let hit = tokio::task::spawn_blocking(move || {
            cached(
                &cache_root,
                name,
                &cache_repo,
                &cache_request,
                &cache_limits,
            )
        })
        .await
        .map_err(|_| Error::Cache)?;
        if let Some(mut hit) = hit {
            // Prefer the current updater's body to a previous fallback for this candidate.
            if details.notes.is_some() {
                hit.notes = details.notes;
                hit.notes_source = details.notes_source;
                hit.notes_state = NotesState::Ready;
            }
            return Ok(hit);
        }
    }
    let result = tokio::time::timeout(Duration::from_secs(limits.total_seconds), async {
        if target == Target::Official {
            let url = format!("https://registry.npmjs.org/@bitkyc08%2fopencodex/{version}");
            let raw = source.get(&url).await?.ok_or(Error::Source)?;
            verify_package(&raw, version, repo)?;
        }
        let url = format!("https://api.github.com/repos/{repo}/releases/tags/v{version}");
        match source.get(&url).await? {
            Some(raw) => read_release(&raw, &mut details, repo, limits),
            None => Ok(()),
        }
    })
    .await
    .unwrap_or(Err(Error::Network));
    if result.is_err() {
        details.notes_state = NotesState::Failed;
        details.release_url = None;
        return Ok(details);
    }
    let cache = Cache {
        schema: 1,
        repository: repo.clone(),
        details: details.clone(),
    };
    let cache_root = root.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let path = cache_file(&cache_root, name, true)?;
        let bytes = serde_json::to_vec(&cache).map_err(|_| Error::Cache)?;
        crate::infrastructure::atomic_write::atomic_write(&path, &bytes, 0o600)
            .map_err(|_| Error::Cache)
    })
    .await
    .map_err(|_| Error::Cache)??;
    Ok(details)
}

#[cfg(test)]
#[path = "release_notes_tests.rs"]
mod tests;
