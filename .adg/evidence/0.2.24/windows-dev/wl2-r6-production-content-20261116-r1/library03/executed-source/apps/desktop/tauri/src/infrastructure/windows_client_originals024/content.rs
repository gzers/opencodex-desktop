//! Fixed-source audit projection and actual original action/family context.
//! No serde/Clone/public fields or constructor from caller PID/home/time DATA.
use super::{
    bounded::{native_stamp, BoundedAccessV1},
    phase::invalid,
    source::Source,
    tree,
    v2::routes::RoutePin,
};
use crate::{
    errors::AppResult,
    infrastructure::{
        hash,
        windows_diagnostic_roles024::NormalExit,
        windows_official_lifecycle024::{NativeAction, OriginalActionKind, Owner},
        windows_official_runtime024::{PinnedDirectory, ProcessIdentity},
    },
    modules::runtime::windows_lifecycle024_contract::NativeSession,
};
use chrono::{DateTime, Utc};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    time::Instant,
};
pub(super) const AUDIT_SOURCES: [(&str, &str); 4] = [
    (
        "src/lib/redact.ts",
        "1f2804c159d0cbf670d701d83422ed0c9d0adfc9125691aa4d408dba4d847ffa",
    ),
    (
        "src/lib/redact-folding.ts",
        "7bcd5a96fc97523df2f60e5f547aeb2e392c95c8802020bee973c6b775c9b85f",
    ),
    (
        "src/codex/catalog/write-audit.ts",
        "09cd430d0337b49d4b400d2a256a9a82eaa977bed66749d64f6041ed73a405d9",
    ),
    (
        "src/codex/catalog/write-audit-contract.ts",
        "0b7ec87dfe50ab6d300c489b18d9c1a59c732939c92c085d46b4902117ec0ccd",
    ),
];
fn before(end: Instant) -> io::Result<()> {
    if Instant::now() >= end {
        Err(io::ErrorKind::TimedOut.into())
    } else {
        Ok(())
    }
}
fn ioe(e: impl std::fmt::Display) -> io::Error {
    io::Error::other(e.to_string())
}
fn created_at(filetime: u64) -> io::Result<DateTime<Utc>> {
    let ticks = filetime
        .checked_sub(116444736000000000)
        .ok_or_else(invalid)?;
    let seconds = i64::try_from(ticks / 10000000).map_err(|_| invalid())?;
    // Official audit timestamps have millisecond precision. Floor the native
    // creation time to the same precision, not a new runtime/mutation deadline.
    let nanos = ((ticks % 10000000) / 10000 * 1000000) as u32;
    DateTime::from_timestamp(seconds, nanos).ok_or_else(invalid)
}
pub(super) fn audit_home_projection(home: &Path) -> io::Result<String> {
    crate::infrastructure::windows_official_runtime024::validate_local_path(home)?;
    let raw = home.to_str().ok_or_else(invalid)?;
    let windows = regex::Regex::new(r"((?i-u:[A-Za-z]:\\Users\\))[^\\/]+").map_err(ioe)?;
    let posix = regex::Regex::new(r"((?i-u:/(?:Users|home)/))[^/]+").map_err(ioe)?;
    let projected = windows
        .replace_all(raw, |caps: &regex::Captures<'_>| {
            format!("{}[USER]", &caps[1])
        })
        .into_owned();
    let projected = posix
        .replace_all(&projected, |caps: &regex::Captures<'_>| {
            format!("{}[USER]", &caps[1])
        })
        .into_owned();
    // Only admit strings for which the remaining official sensitive-segment,
    // token/framing rules are provably identity transforms. Never implement a
    // partial secret redactor, guess a projection or truncate colliding HOMEs.
    let lower = projected.to_ascii_lowercase();
    if [
        "secret",
        "password",
        "passwd",
        "token",
        "api-key",
        "api_key",
        "apikey",
        "credential",
        "email",
        "sk-",
        "eyj",
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "ghr_",
        "github_pat_",
        "bearer",
    ]
    .iter()
    .any(|s| lower.contains(s))
        || projected.encode_utf16().count() > 256
        || projected
            .chars()
            .enumerate()
            .any(|(i, c)| !(c.is_alphanumeric() || "\\/._- []".contains(c) || c == ':' && i == 1))
    {
        return Err(invalid());
    }
    Ok(projected)
}
struct AuditSourcePin {
    file: File,
    parent: PinnedDirectory,
    security: BoundedAccessV1,
    path: PathBuf,
    sha: String,
}
impl AuditSourcePin {
    fn capture(path: PathBuf, sha: &str, end: Instant) -> io::Result<Self> {
        before(end)?;
        let parent = PinnedDirectory::open(path.parent().ok_or_else(invalid)?)?;
        let security = BoundedAccessV1::capture(&path, false, end).map_err(ioe)?;
        let file = OpenOptions::new()
            .access_mode(0x80020000)
            .share_mode(1)
            .custom_flags(0x200000)
            .open(&path)?;
        let mut pin = Self {
            file,
            parent,
            security,
            path,
            sha: sha.into(),
        };
        pin.check(end)?;
        Ok(pin)
    }
    fn check(&mut self, end: Instant) -> io::Result<()> {
        before(end)?;
        self.parent.recheck()?;
        self.security.recheck(end).map_err(ioe)?;
        tree::streams(&self.path, false)?;
        if native_stamp(&self.file, end).map_err(ioe)? != self.security.snapshot_data()? {
            return Err(invalid());
        }
        self.file.seek(SeekFrom::Start(0))?;
        let mut raw = Vec::new();
        (&mut self.file)
            .take(256 * 1024 + 1)
            .read_to_end(&mut raw)?;
        if raw.len() > 256 * 1024 || hash::sha256_hex(&raw) != self.sha {
            return Err(invalid());
        }
        before(end)
    }
}
pub(super) struct StartContentScope {
    action: NativeAction,
    root: ProcessIdentity,
    profile: PathBuf,
    home: PathBuf,
    audit_home: String,
    not_before: DateTime<Utc>,
    routes_digest: String,
    pins: Vec<AuditSourcePin>,
}
impl StartContentScope {
    pub(super) fn capture(
        source: &mut Source,
        owner: &mut Owner,
        routes: &mut RoutePin,
        end: Instant,
    ) -> AppResult<Self> {
        source.check(owner, end)?;
        owner.originals_require_unresumed(end)?;
        let result = (|| {
            let declared = routes.declared(owner.home_path(), end)?;
            let routes_digest =
                hash::sha256_hex(&serde_json::to_vec(declared).map_err(|_| invalid())?);
            let root = owner.original_native().binding().process.clone();
            let not_before = created_at(root.creation_filetime)?;
            let audit_home = audit_home_projection(owner.home_path())?;
            if not_before > Utc::now() {
                return Err(invalid());
            }
            let package = owner.selected_program().paths.package_root.clone();
            let mut pins = Vec::new();
            for (path, sha) in AUDIT_SOURCES {
                pins.push(AuditSourcePin::capture(package.join(path), sha, end)?);
            }
            Ok((root, not_before, audit_home, routes_digest, pins))
        })()
        .map_err(super::source::content_error)?;
        let action = owner.original_action(OriginalActionKind::Start, end)?;
        source.check(owner, end)?;
        owner.originals_require_unresumed(end)?;
        Ok(Self {
            action,
            root: result.0,
            profile: owner.profile_path().into(),
            home: owner.home_path().into(),
            not_before: result.1,
            audit_home: result.2,
            routes_digest: result.3,
            pins: result.4,
        })
    }
    pub(super) fn stopped(
        &mut self,
        source: &mut Source,
        owner: &mut Owner,
        normal: &NormalExit,
        routes: &mut RoutePin,
        end: Instant,
    ) -> io::Result<StoppedContentScope> {
        before(end)?;
        source.check(owner, end).map_err(ioe)?;
        normal.check(owner, end).map_err(ioe)?;
        let action = owner.original_stop_action(end).map_err(ioe)?;
        if !action.same_original_owner(&self.action)
            || owner.profile_path() != self.profile
            || owner.home_path() != self.home
            || owner.original_native().binding().process != self.root
        {
            return Err(invalid());
        }
        for pin in &mut self.pins {
            pin.check(end)?;
        }
        let declared = routes.declared(owner.home_path(), end)?.to_vec();
        if hash::sha256_hex(&serde_json::to_vec(&declared).map_err(|_| invalid())?)
            != self.routes_digest
        {
            return Err(invalid());
        }
        let processes = normal.original_content_processes(owner, end).map_err(ioe)?;
        let not_after = Utc::now();
        if self.not_before > not_after {
            return Err(invalid());
        }
        let scope = StoppedContentScope {
            action,
            root: self.root.clone(),
            profile: self.profile.clone(),
            home: self.home.clone(),
            processes,
            audit_home: self.audit_home.clone(),
            not_before: self.not_before,
            not_after,
            routes: declared,
        };
        scope.check(source, owner, normal, routes, end)?;
        Ok(scope)
    }
}
pub(super) struct StoppedContentScope {
    action: NativeAction,
    root: ProcessIdentity,
    profile: PathBuf,
    home: PathBuf,
    processes: Vec<ProcessIdentity>,
    audit_home: String,
    not_before: DateTime<Utc>,
    not_after: DateTime<Utc>,
    routes: Vec<String>,
}
impl StoppedContentScope {
    pub(super) fn check(
        &self,
        source: &mut Source,
        owner: &mut Owner,
        normal: &NormalExit,
        routes: &mut RoutePin,
        end: Instant,
    ) -> io::Result<()> {
        before(end)?;
        self.action.check(owner, end).map_err(ioe)?;
        source.check(owner, end).map_err(ioe)?;
        if self.action.deadline() != end
            || owner.profile_path() != self.profile
            || owner.home_path() != self.home
            || owner.original_native().binding().process != self.root
            || self.not_before > self.not_after
            || self.not_after > Utc::now()
            || normal.original_content_processes(owner, end).map_err(ioe)? != self.processes
            || routes.declared(owner.home_path(), end)? != self.routes.as_slice()
        {
            return Err(invalid());
        }
        before(end)
    }
    pub(super) fn deadline(&self) -> Instant {
        self.action.deadline()
    }
    pub(super) fn routes(&self) -> &[String] {
        &self.routes
    }
    pub(super) fn pids(&self) -> Vec<u32> {
        self.processes.iter().map(|p| p.pid).collect()
    }
    pub(super) fn audit_home(&self) -> &str {
        &self.audit_home
    }
    pub(super) fn interval(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        (self.not_before, self.not_after)
    }
    pub(super) fn digest(&self) -> io::Result<String> {
        Ok(hash::sha256_hex(&serde_json::to_vec(&serde_json::json!({"scope":self.action.scope_id(),"ownerRootPid":self.root.pid,
            "ownerRootCreated":self.root.creation_filetime.to_string(),"profile":self.profile,"home":self.home,
            "processes":self.processes.iter().map(|p|serde_json::json!({"pid":p.pid,"created":p.creation_filetime.to_string(),"imageVolume":p.executable_id.volume,"imageId":p.executable_id.index.to_string(),"tokenSid":p.token.sid,"tokenSession":p.token.session_id,"tokenElevated":p.token.elevated,"tokenElevationType":p.token.elevation_type})).collect::<Vec<_>>(),
            "routes":self.routes,"auditHome":self.audit_home,"auditNotBefore":self.not_before,"auditNotAfter":self.not_after})).map_err(|_|invalid())?))
    }
}
#[cfg(test)]
#[path = "content_tests.rs"]
mod tests;
