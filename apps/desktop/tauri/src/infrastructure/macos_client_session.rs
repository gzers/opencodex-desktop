//! Bind startup originals to a genuinely authenticated ready generation and
//! the complete ordinary Stop result. No client publication/permit is issued.
use super::*;
use crate::{
    commands::runtime_recovery_v2::{ServiceOwnerProof, ServiceOwnerProofProvider},
    commands::runtime_recovery_v2_owner::{
        CompletedLifecycleStop, SystemServiceOwnerProofProvider,
    },
    errors::{AppError, AppResult},
    infrastructure::{
        macos_process_exit::Generation, official_client_coordination::native::NativeLease,
    },
    modules::runtime::program_selection::CurrentSelection,
    state::ProcessContext,
};
use std::time::Instant;
#[cfg(test)]
#[path = "macos_client_root_stop_binding_tests.rs"]
mod root_stop_binding_tests;

fn error<T>(_: T) -> AppError {
    AppError::RuntimeManaged {
        code: "client_session_unconfirmed".into(),
        detail: "启动原件与当前服务的绑定未确认；原件和现场保留，未恢复客户端文件".into(),
    }
}
fn time(deadline: Instant) -> AppResult<()> {
    if Instant::now() >= deadline {
        Err(AppError::Timeout)
    } else {
        Ok(())
    }
}
fn valid_hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Scope {
    schema_version: u32,
    attempt: String,
    original_sha256: String,
    uid: u32,
    data: PathBuf,
    data_identity: Identity,
    profile: PathBuf,
    profile_identity: Identity,
    home: PathBuf,
    home_identity: Identity,
    client: PathBuf,
    client_identity: Identity,
    program_root: PathBuf,
    selection: CurrentSelection,
    captured_at: chrono::DateTime<chrono::Utc>,
    declared_routes: Option<Vec<String>>,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Live {
    schema_version: u32,
    scope_sha256: String,
    ready_sha256: String,
    record_sha256: String,
    generation: [u64; 4],
    port: u16,
    parent_generation: [u64; 4],
}
impl Live {
    fn parent(&self) -> AppResult<Generation> {
        let [pid, uid, sec, micros] = self.parent_generation;
        let pid = u32::try_from(pid).map_err(error)?;
        let uid = u32::try_from(uid).map_err(error)?;
        if pid == 0
            || pid > i32::MAX as u32
            || uid != unsafe { libc::geteuid() }
            || sec == 0
            || sec > i64::MAX as u64
            || micros >= 1_000_000
            || pid as u64 == self.generation[0]
        {
            return Err(error(()));
        }
        Ok(Generation {
            pid,
            uid,
            start_seconds: sec,
            start_micros: micros,
        })
    }
    fn process(&self) -> AppResult<Generation> {
        let [pid, uid, start_seconds, start_micros] = self.generation;
        let pid = u32::try_from(pid).map_err(error)?;
        let uid = u32::try_from(uid).map_err(error)?;
        if pid == 0
            || pid > i32::MAX as u32
            || uid != unsafe { libc::geteuid() }
            || start_seconds == 0
            || start_seconds > i64::MAX as u64
            || start_micros >= 1_000_000
            || self.port == 0
            || !valid_hash(&self.record_sha256)
            || !valid_hash(&self.ready_sha256)
            || !valid_hash(&self.scope_sha256)
        {
            return Err(error(()));
        }
        Ok(Generation {
            pid,
            uid,
            start_seconds,
            start_micros,
        })
    }
}

/// These are references for the owning transaction to protect independently,
/// not serialized service authority. Opening with a new observed file digest
/// or scanning a directory to invent a replacement baseline is forbidden.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct References {
    pub attempt: String,
    pub original_sha256: String,
    pub scope_sha256: String,
    pub phases: Vec<(Phase, String)>,
    pub live_sha256: Option<String>,
}
pub struct StartupSession {
    originals: Originals,
    scope_sha256: String,
    live_sha256: Option<String>,
    profile: PinnedDirectory,
    home: PinnedDirectory,
}
/// Borrowed action inputs only; constructing this does not grant authority.
/// The normal-Stop factory independently validates every field.
pub struct RootStopRequest<'a> {
    pub context: &'a ProcessContext,
    pub stop: &'a CompletedLifecycleStop,
    pub pair: &'a crate::infrastructure::macos_process_pair::PairExitProof,
    pub lease: &'a NativeLease,
    pub head: &'a session_index::Head,
    pub deadline: Instant,
}
/// Full OS absence is admitted before a config writer helper is spawned.
/// Borrowed current Stop/pair/lease survive the operation; no serde or copied
/// status can recreate this admission after interruption.
pub struct RootStopAdmission<'a> {
    session: &'a StartupSession,
    context: &'a ProcessContext,
    stop: &'a CompletedLifecycleStop,
    pair: &'a crate::infrastructure::macos_process_pair::PairExitProof,
    lease: &'a NativeLease,
    head: session_index::Head,
    scope: Scope,
    live: Live,
    deadline: Instant,
}
#[derive(Clone, Copy)]
enum RootPhaseAuthority<'a> {
    Ready(&'a NativeLease),
    PreStop,
    Stopped(
        &'a NativeLease,
        &'a CompletedLifecycleStop,
        &'a crate::infrastructure::macos_process_pair::PairExitProof,
    ),
}
/// Current typed phase evidence. Disk frame hashes alone cannot construct it.
pub struct RootPhaseProof<'a> {
    session: &'a StartupSession,
    context: &'a ProcessContext,
    expected: session_index::Head,
    authority: RootPhaseAuthority<'a>,
    deadline: Instant,
}
impl RootPhaseProof<'_> {
    pub(super) fn source(&self) -> &session_index::Head {
        &self.expected
    }
    pub(super) fn state(&self) -> session_index::State {
        match self.authority {
            RootPhaseAuthority::Ready(_) => session_index::State::Ready,
            RootPhaseAuthority::PreStop => session_index::State::StopPrepared,
            RootPhaseAuthority::Stopped(..) => session_index::State::Stopped,
        }
    }
    pub(super) fn require(
        &self,
        data: &Path,
        client: &Path,
        profile: &Path,
    ) -> AppResult<References> {
        time(self.deadline)?;
        let origin = self
            .session
            .originals
            .original_root_absence
            .as_ref()
            .ok_or_else(|| error(()))?;
        if data != self.session.originals.data.path()
            || client != self.session.originals.client.path()
            || profile != self.session.profile.path()
            || self.expected.ticket != origin.id
        {
            return Err(error(()));
        }
        let refs = self.session.references()?;
        let before = self.expected.references.as_ref().ok_or_else(|| error(()))?;
        if before.attempt != refs.attempt
            || before.original_sha256 != refs.original_sha256
            || before.scope_sha256 != refs.scope_sha256
            || !refs.phases.starts_with(&before.phases)
            || before
                .live_sha256
                .as_ref()
                .is_some_and(|s| Some(s) != refs.live_sha256.as_ref())
        {
            return Err(error(()));
        }
        match self.authority {
            RootPhaseAuthority::Ready(lease) => {
                if self.expected.state != session_index::State::Captured || refs.phases.len() != 1 {
                    return Err(error(()));
                }
                lease.require_context(self.context, client)?;
                self.session
                    .verify_current_ready(self.context, self.deadline)?;
            }
            RootPhaseAuthority::PreStop => {
                if self.expected.state != session_index::State::Ready || refs.phases.len() != 2 {
                    return Err(error(()));
                }
                self.session
                    .originals
                    .frame(Phase::PreStop)
                    .map_err(error)?;
                self.session
                    .verify_current_ready(self.context, self.deadline)?;
            }
            RootPhaseAuthority::Stopped(lease, stop, pair) => {
                if self.expected.state != session_index::State::StopPrepared
                    || refs.phases.len() != 3
                {
                    return Err(error(()));
                }
                lease.require_context(self.context, client)?;
                let live = self.session.live(self.context)?;
                pair.require(live.process()?, live.parent()?, self.deadline)?;
                let (root, _) = source(self.context)?;
                let owner = SystemServiceOwnerProofProvider::new(root, self.context.clone())?;
                stop.require_original_absence(
                    data,
                    live.process()?,
                    live.parent()?,
                    &live.record_sha256,
                    live.port,
                    &owner,
                )?;
                self.session
                    .originals
                    .frame(Phase::Stopped)
                    .map_err(error)?;
            }
        }
        time(self.deadline)?;
        Ok(refs)
    }
}
impl RootStopAdmission<'_> {
    fn check_before_plan(&self, session: &StartupSession) -> AppResult<()> {
        time(self.deadline)?;
        if !std::ptr::eq(session, self.session)
            || self.head.references.as_ref() != Some(&session.references()?)
            || session.scope(self.context)? != self.scope
            || session.live(self.context)? != self.live
        {
            return Err(error(()));
        }
        self.lease
            .require_context(self.context, &session.originals.logical_client())?;
        self.pair
            .require(self.live.process()?, self.live.parent()?, self.deadline)?;
        let owner = SystemServiceOwnerProofProvider::new(
            self.scope.program_root.clone(),
            self.context.clone(),
        )?;
        self.stop.require_original_absence(
            session.originals.data.path(),
            self.live.process()?,
            self.live.parent()?,
            &self.live.record_sha256,
            self.live.port,
            &owner,
        )?;
        time(self.deadline)
    }
}
/// Current normal-Stop authority only. Values are never deserialized and no
/// public constructor accepts a callback, bool, PID list or copied receipt.
pub(super) struct RootRetirementAuthority<'a> {
    session: &'a StartupSession,
    context: &'a ProcessContext,
    stop: &'a CompletedLifecycleStop,
    pair: &'a crate::infrastructure::macos_process_pair::PairExitProof,
    lease: &'a NativeLease,
    owner: SystemServiceOwnerProofProvider,
    head: session_index::Head,
    plan: session_index::RestorePlan,
    scope: Scope,
    live: Live,
    deadline: Instant,
    registration: &'a session_index::RootRetirementRegistration<'a>,
}
impl RootRetirementAuthority<'_> {
    pub(super) fn deadline(&self) -> Instant {
        self.deadline
    }
    pub(super) fn require_pending(&self, pending: &root_retirement::Pending<'_>) -> AppResult<()> {
        if !pending.belongs_to(&self.session.originals)
            || pending.id() != self.plan.id
            || pending.sha256() != self.plan.sha256
        {
            return Err(error(()));
        }
        pending.require_source(&self.head).map_err(error)?;
        self.check()
    }
    pub(super) fn check(&self) -> AppResult<()> {
        time(self.deadline)?;
        self.registration
            .require(&self.head, &self.plan)
            .map_err(error)?;
        self.lease
            .require_context(self.context, &self.session.originals.logical_client())?;
        self.session.profile.recheck().map_err(error)?;
        self.session.home.recheck().map_err(error)?;
        self.session.verify_root_stop_binding(
            self.context,
            &self.head,
            &self.plan,
            &self.scope,
            &self.live,
        )?;
        let live = &self.live;
        self.pair
            .require(live.process()?, live.parent()?, self.deadline)?;
        self.stop.require_original_absence(
            self.session.originals.data.path(),
            live.process()?,
            live.parent()?,
            &live.record_sha256,
            live.port,
            &self.owner,
        )?;
        if capture_file(&self.session.home, "runtime-port.json", 4096).map_err(error)?
            != Image::Absent
            || capture_file(&self.session.home, "ocx.pid", 64).map_err(error)? != Image::Absent
        {
            return Err(error(()));
        }
        time(self.deadline)
    }
    pub(super) fn finish_check(&self) -> AppResult<()> {
        self.check()?;
        self.owner.require_finalization_absence(
            self.context,
            self.live.process()?,
            self.live.parent()?,
            self.live.port,
        )?;
        self.check()
    }
}
/// Borrow-scoped permission for this session's assessed Codex artifacts,
/// including an individually admitted originally absent first-use config.
/// The official K and L writers remain excluded by the native lease throughout
/// publication. This is not whole-family quiescence or complete client recovery.
pub(super) struct ArtifactRestoreAuthority<'a> {
    session: &'a StartupSession,
    context: &'a ProcessContext,
    stop: &'a CompletedLifecycleStop,
    pair: &'a crate::infrastructure::macos_process_pair::PairExitProof,
    lease: &'a NativeLease,
    deadline: Instant,
}
impl ArtifactRestoreAuthority<'_> {
    pub(super) fn grok_write_guard(
        &self,
    ) -> AppResult<crate::infrastructure::official_client_coordination::native::config::Guard<'_>>
    {
        self.lease.grok_write_guard()
    }

    pub(super) fn config_write_guard(
        &self,
    ) -> AppResult<crate::infrastructure::official_client_coordination::native::config::Guard<'_>>
    {
        self.lease.config_write_guard()
    }

    pub(super) fn check(&self, originals: &Originals) -> AppResult<()> {
        time(self.deadline)?;
        if !std::ptr::eq(originals, &self.session.originals) {
            return Err(error(()));
        }
        self.lease
            .require_context(self.context, &originals.client.path)?;
        let live = self.session.live(self.context)?;
        self.pair
            .require(live.process()?, live.parent()?, self.deadline)?;
        self.stop.require_original(
            self.context,
            &originals.data.path,
            live.process()?,
            &live.record_sha256,
        )?;
        if capture_file(&self.session.home, "runtime-port.json", 4096).map_err(error)?
            != Image::Absent
            || capture_file(&self.session.home, "ocx.pid", 64).map_err(error)? != Image::Absent
        {
            return Err(error(()));
        }
        time(self.deadline)
    }
}
/// A distinct user-confirmed action after interruption. This proves CURRENT
/// absence and live writer exclusion, never reconstructs normal exit history.
pub(super) struct ExplicitRestoreAuthority<'a> {
    session: &'a StartupSession,
    context: &'a ProcessContext,
    owner: SystemServiceOwnerProofProvider,
    lease: &'a NativeLease,
    deadline: Instant,
}
impl ExplicitRestoreAuthority<'_> {
    pub(super) fn grok_write_guard(
        &self,
    ) -> AppResult<crate::infrastructure::official_client_coordination::native::config::Guard<'_>>
    {
        self.lease.grok_write_guard()
    }

    pub(super) fn config_write_guard(
        &self,
    ) -> AppResult<crate::infrastructure::official_client_coordination::native::config::Guard<'_>>
    {
        self.lease.config_write_guard()
    }

    pub(super) fn check(&self, originals: &Originals) -> AppResult<()> {
        time(self.deadline)?;
        if !std::ptr::eq(originals, &self.session.originals) {
            return Err(error(()));
        }
        self.lease
            .require_context(self.context, &originals.client.path)?;
        let live = self.session.live(self.context)?;
        self.owner.require_excluded_recovery_absence(
            self.context,
            live.process()?,
            live.parent()?,
            live.port,
        )?;
        time(self.deadline)
    }
}
fn effective_client(context: &ProcessContext) -> AppResult<PathBuf> {
    // The production controlled runner clears CODEX_HOME and supplies the
    // startup user's HOME. On macOS official defaultCodexHome therefore uses
    // this profile's .codex, canonicalizing an existing explicit directory.
    // An independently configured coordinator path is not runtime evidence.
    let path = std::fs::canonicalize(context.working_directory.join(".codex")).map_err(error)?;
    PinnedDirectory::open(&path).map_err(error)?;
    Ok(path)
}
fn source(context: &ProcessContext) -> AppResult<(PathBuf, CurrentSelection)> {
    // Observation only: caller-side NativeLease owns the full before/after
    // image admission, and fresh_ready independently repeats the actual signed
    // process protocol. Do not hash Node again for each private reference read.
    if context.runtime.source_problem().is_some() {
        return Err(error(()));
    }
    let root = context
        .runtime
        .managed_installation_root()
        .ok_or_else(|| error(()))?;
    let identity = context.runtime.observed_program_identity()?;
    let path = root.join("runtime/current.json");
    crate::modules::backup::safety::check_path(&root, &path, false)?;
    let file = crate::modules::backup::safety::open_file(&path, false)?;
    crate::modules::backup::safety::require_single_link(&file)?;
    let mut bytes = Vec::new();
    file.take(65537).read_to_end(&mut bytes).map_err(error)?;
    if bytes.len() > 65536 {
        return Err(error(()));
    }
    let selection: CurrentSelection = parse(&bytes)?;
    if selection.schema_version != 1
        || selection.version != "2.99.0"
        || selection.platform != "darwin-arm64"
        || !valid_hash(&selection.image_sha256)
        || identity.installation_root != root
        || identity.package_version != selection.version
        || context.runtime.executable().as_ref() != Some(&identity.entry)
        || crate::modules::backup::safety::read_file(&path, 65536)? != bytes
    {
        return Err(error(()));
    }
    Ok((root, selection))
}
fn read_private(originals: &Originals, name: &str, sha: &str) -> AppResult<Vec<u8>> {
    if !valid_hash(sha) {
        return Err(error(()));
    }
    let image = capture_file(&originals.directory, name, 1024 * 1024).map_err(error)?;
    let Image::File {
        protection, bytes, ..
    } = image
    else {
        return Err(error(()));
    };
    require_private(&protection, false).map_err(error)?;
    if digest(&bytes) != sha {
        return Err(error(()));
    }
    Ok(bytes)
}
fn parse<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> AppResult<T> {
    serde_json::from_value(parse_strict_json_v2(bytes).map_err(error)?).map_err(error)
}
pub(super) fn verify_index_scope(
    originals: &Originals,
    refs: &References,
    home: &Path,
) -> AppResult<()> {
    let scope: Scope = parse(&read_private(
        originals,
        "startup-scope.json",
        &refs.scope_sha256,
    )?)?;
    let home = PinnedDirectory::open(home).map_err(error)?;
    if scope.schema_version != 1
        || scope.attempt != refs.attempt
        || scope.original_sha256 != refs.original_sha256
        || scope.uid != unsafe { libc::geteuid() }
        || scope.data_identity != *originals.data.identity()
        || scope.client_identity != *originals.client.identity()
        || scope.profile_identity != *originals.profile.identity()
        || scope.data != originals.data.path
        || scope.client != originals.logical_client()
        || scope.profile != originals.profile.path
        || scope.home != home.path
        || scope.home_identity != *home.identity()
    {
        return Err(error(()));
    }
    if refs.live_sha256.is_some() {
        verify_closed_reference_files(originals, refs)?;
    }
    Ok(())
}
pub(super) fn verify_closed_reference_files(
    originals: &Originals,
    refs: &References,
) -> AppResult<()> {
    let scope: Scope = parse(&read_private(
        originals,
        "startup-scope.json",
        &refs.scope_sha256,
    )?)?;
    let live: Live = parse(&read_private(
        originals,
        "startup-live.json",
        refs.live_sha256.as_deref().ok_or_else(|| error(()))?,
    )?)?;
    let (_, ready) = originals.frame(Phase::Ready).map_err(error)?;
    if scope.schema_version != 1
        || scope.attempt != refs.attempt
        || scope.original_sha256 != refs.original_sha256
        || scope.uid != unsafe { libc::geteuid() }
        || scope.data_identity != *originals.data.identity()
        || scope.client_identity != *originals.client.identity()
        || scope.profile_identity != *originals.profile.identity()
        || scope.data != originals.data.path
        || scope.client != originals.logical_client()
        || scope.profile != originals.profile.path
        || live.schema_version != 1
        || live.scope_sha256 != refs.scope_sha256
        || live.ready_sha256 != digest(&ready)
    {
        return Err(error(()));
    }
    live.process()?;
    live.parent()?;
    Ok(())
}
pub(super) fn verify_captured_reference_files(
    originals: &Originals,
    refs: &References,
) -> AppResult<()> {
    if refs.live_sha256.is_some() || !refs.phases.is_empty() {
        return Err(error(()));
    }
    let scope: Scope = parse(&read_private(
        originals,
        "startup-scope.json",
        &refs.scope_sha256,
    )?)?;
    if scope.schema_version != 1
        || scope.attempt != refs.attempt
        || scope.original_sha256 != refs.original_sha256
        || scope.uid != unsafe { libc::geteuid() }
        || scope.data_identity != *originals.data.identity()
        || scope.client_identity != *originals.client.identity()
        || scope.profile_identity != *originals.profile.identity()
        || scope.data != originals.data.path
        || scope.client != originals.logical_client()
        || scope.profile != originals.profile.path
    {
        return Err(error(()));
    }
    Ok(())
}
fn private_runtime(home: &PinnedDirectory) -> AppResult<(String, u32, u16)> {
    let image = capture_file(home, "runtime-port.json", 4096).map_err(error)?;
    let Image::File {
        protection, bytes, ..
    } = image
    else {
        return Err(error(()));
    };
    let bytes = zeroize::Zeroizing::new(bytes);
    if protection.uid != unsafe { libc::geteuid() }
        || protection.mode != 0o600
        || protection.acl.is_some()
        || protection.flags != 0
    {
        return Err(error(()));
    }
    let value = parse_strict_json_v2(&bytes).map_err(error)?;
    let pid = value["pid"]
        .as_u64()
        .and_then(|n| u32::try_from(n).ok())
        .filter(|n| *n > 0)
        .ok_or_else(|| error(()))?;
    let port = value["port"]
        .as_u64()
        .and_then(|n| u16::try_from(n).ok())
        .filter(|n| *n > 0)
        .ok_or_else(|| error(()))?;
    Ok((digest(&bytes), pid, port))
}
fn declared_routes(home: &PinnedDirectory) -> AppResult<Option<Vec<String>>> {
    let image = capture_file(home, "config.json", 2 * 1024 * 1024).map_err(error)?;
    let Image::File { bytes, .. } = image else {
        return Err(error(()));
    };
    declaration(&bytes)
}
fn declared_id_list(value: Option<&serde_json::Value>) -> Option<Vec<&str>> {
    let Some(value) = value else {
        return Some(Vec::new());
    };
    let rows = value.as_array()?;
    if rows.len() > 4096 {
        return None;
    }
    rows.iter()
        .map(|v| {
            v.as_str()
                .filter(|s| !s.is_empty() && s.len() <= 1024 && !s.chars().any(char::is_control))
        })
        .collect()
}
fn declaration_slug_key(slug: &str) -> String {
    match slug.split_once('/') {
        Some((provider, id)) if !provider.is_empty() => {
            format!("{provider}/{}", id.replace('/', "-"))
        }
        _ => slug.to_string(),
    }
}
fn combo_identifier(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.bytes().next().is_some_and(|v| v.is_ascii_alphanumeric())
        && id
            .bytes()
            .all(|v| v.is_ascii_alphanumeric() || b"._-".contains(&v))
}
fn add_combo_declarations(
    value: &serde_json::Value,
    providers: &serde_json::Map<String, serde_json::Value>,
    native_ids: &std::collections::BTreeSet<(String, String)>,
    routes: &mut std::collections::BTreeSet<String>,
) -> Option<()> {
    let Some(combos) = value.get("combos") else {
        return Some(());
    };
    let combos = combos.as_object()?;
    if combos.len() > 1024 || !combos.is_empty() && providers.contains_key("combo") {
        return None;
    }
    for (id, combo) in combos {
        if !combo_identifier(id)
            || providers.contains_key(id)
            || !combo.is_object()
            || combo
                .get("nativeAlias")
                .is_some_and(|v| !v.is_boolean() || v == &serde_json::Value::Bool(true))
        {
            return None;
        }
        let public = match combo.get("alias") {
            None | Some(serde_json::Value::Null) => format!("combo/{id}"),
            Some(serde_json::Value::String(alias)) => {
                let alias = alias.trim();
                if alias.is_empty() {
                    format!("combo/{id}")
                } else {
                    let segments: Vec<_> = alias.split('/').collect();
                    if segments.len() > 2
                        || !segments.iter().all(|s| combo_identifier(s))
                        || alias == "combo"
                        || alias.starts_with("combo/")
                        || segments.len() == 1
                            && ["gpt-", "o1-", "o3-", "o4-", "codex-"]
                                .iter()
                                .any(|p| alias.starts_with(p))
                    {
                        return None;
                    }
                    alias.to_string()
                }
            }
            _ => return None,
        };
        let targets = combo.get("targets")?.as_array()?;
        if targets.is_empty() || targets.len() > 256 {
            return None;
        }
        let mut seen = std::collections::BTreeSet::new();
        for target in targets {
            let provider = target.get("provider")?.as_str()?.trim();
            let model = target.get("model")?.as_str()?.trim();
            if !native_ids.contains(&(provider.into(), model.into()))
                || !seen.insert((provider, model))
            {
                return None;
            }
        }
        if !routes.insert(public) || routes.len() > 4096 {
            return None;
        }
    }
    Some(())
}
// Independent explicit selection declaration. An unbounded discovery graph
// stays unresolved; nonempty selectedModels is the official final allowlist.
// It is never learned from the generated catalog or a discovery response.
// The actual content policy subsequently requires EXACT ready route coverage.
fn declaration(bytes: &[u8]) -> AppResult<Option<Vec<String>>> {
    let bytes = zeroize::Zeroizing::new(bytes.to_vec());
    let value = parse_strict_json_v2(&bytes).map_err(error)?;
    let Some(providers) = value
        .get("providers")
        .and_then(serde_json::Value::as_object)
    else {
        return Ok(None);
    };
    // Official default/model aliases label rows; they do not change a provider
    // row's catalogModelSlug. Omitted global/provider flags inherit false.
    // Never promote a friendly display alias into independent route authority.
    if value
        .get("defaultModelAliases")
        .is_some_and(|v| !v.is_boolean())
    {
        return Ok(None);
    }
    // Official schema defaults omitted fastRows to true. This boolean
    // normalization does not change independently selected base routes. The
    // content policy STILL refuses undeclared Fast/effort siblings in Ready.
    if value.get("fastRows").is_some_and(|v| !v.is_boolean()) {
        return Ok(None);
    }
    let Some(disabled) = declared_id_list(value.get("disabledModels")) else {
        return Ok(None);
    };
    let disabled: std::collections::BTreeSet<_> =
        disabled.into_iter().map(declaration_slug_key).collect();
    let mut routes = std::collections::BTreeSet::new();
    let mut native_ids = std::collections::BTreeSet::new();
    let mut custom_by_provider: std::collections::BTreeMap<&str, Vec<&str>> =
        std::collections::BTreeMap::new();
    if let Some(custom) = value.get("customModels") {
        let Some(custom) = custom.as_array().filter(|rows| rows.len() <= 4096) else {
            return Ok(None);
        };
        let mut seen = std::collections::BTreeSet::new();
        for row in custom {
            let Some(provider) = row.get("provider").and_then(serde_json::Value::as_str) else {
                return Ok(None);
            };
            let Some(id) = row
                .get("modelId")
                .and_then(serde_json::Value::as_str)
                .filter(|s| !s.is_empty() && s.len() <= 1024 && !s.chars().any(char::is_control))
            else {
                return Ok(None);
            };
            if !providers.contains_key(provider) || !seen.insert((provider, id)) {
                return Ok(None);
            }
            custom_by_provider.entry(provider).or_default().push(id);
        }
    }
    for (provider, entry) in providers {
        if !entry.is_object() || entry.get("disabled").is_some_and(|v| !v.is_boolean()) {
            return Ok(None);
        }
        if entry.get("disabled") == Some(&serde_json::Value::Bool(true)) {
            continue;
        }
        if provider.is_empty()
            || provider.len() > 256
            || !provider
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || entry.get("liveModels").is_some_and(|v| !v.is_boolean())
            || entry.get("defaultAliases").is_some_and(|v| !v.is_boolean())
            || entry.get("modelAliases").is_some_and(|v| {
                !v.as_object().is_some_and(|o| {
                    o.iter().all(|(id, alias)| {
                        !id.is_empty()
                            && id.len() <= 1024
                            && !id.chars().any(char::is_control)
                            && alias.as_str().is_some_and(|s| {
                                s.len() <= 1024 && !s.chars().any(char::is_control)
                            })
                    })
                })
            })
        {
            return Ok(None);
        }
        let Some(selected) = declared_id_list(entry.get("selectedModels")) else {
            return Ok(None);
        };
        let models = if !selected.is_empty() {
            selected
        } else {
            // Empty/omitted selection means ALL models, not an empty allowlist.
            // Only a static provider supplies a complete independent roster.
            if entry.get("liveModels") != Some(&serde_json::Value::Bool(false)) {
                return Ok(None);
            }
            let Some(configured) = declared_id_list(entry.get("models")) else {
                return Ok(None);
            };
            let Some(retained) = declared_id_list(entry.get("retainModels")) else {
                return Ok(None);
            };
            let mut ids: std::collections::BTreeSet<_> =
                configured.iter().copied().chain(retained).collect();
            if configured.is_empty() {
                if let Some(default) = entry.get("defaultModel") {
                    let Some(id) = default
                        .as_str()
                        .filter(|s| s.len() <= 1024 && !s.chars().any(char::is_control))
                    else {
                        return Ok(None);
                    };
                    if !id.is_empty() {
                        ids.insert(id);
                    }
                }
            }
            ids.into_iter().collect()
        };
        // Same native model may be overlaid with custom metadata; different
        // native IDs that encode to one slug are still ambiguous and refuse.
        // Check this even for unselected/disabled custom rows, before filtering.
        let mut encoded_custom = std::collections::BTreeMap::new();
        for id in models.iter().copied().chain(
            custom_by_provider
                .get(provider.as_str())
                .into_iter()
                .flatten()
                .copied(),
        ) {
            let encoded = id.replace('/', "-");
            if encoded_custom
                .insert(encoded, id)
                .is_some_and(|prior| prior != id)
            {
                return Ok(None);
            }
        }
        let mut models = models;
        if entry
            .get("selectedModels")
            .is_none_or(|v| v.as_array().is_some_and(|s| s.is_empty()))
        {
            if let Some(custom) = custom_by_provider.get(provider.as_str()) {
                for id in custom {
                    if !models.contains(id) {
                        models.push(id);
                    }
                }
            }
        }
        for model in models {
            native_ids.insert((provider.clone(), model.to_string()));
            // Original 2.99 providers/slug-codec::routedSlug maps inner slash to
            // hyphen, NOT a blind unencoded provider/model concatenation.
            if !routes.insert(format!("{provider}/{}", model.replace('/', "-")))
                || routes.len() > 4096
            {
                return Ok(None);
            }
        }
    }
    if add_combo_declarations(&value, providers, &native_ids, &mut routes).is_none() {
        return Ok(None);
    }
    Ok(Some(
        routes
            .into_iter()
            .filter(|slug| !disabled.contains(&declaration_slug_key(slug)))
            .collect(),
    ))
}
fn audit_home(path: &Path) -> AppResult<String> {
    let raw = path.to_str().ok_or_else(|| error(()))?;
    // Match the official user-home projection. Refuse sensitive segments and
    // token-like inputs rather than implement an incomplete secret redactor.
    let lower = raw.to_ascii_lowercase();
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
        "eyJ",
    ]
    .iter()
    .any(|key| lower.contains(&key.to_ascii_lowercase()))
    {
        return Err(error(()));
    }
    let pattern = regex::Regex::new(r"(?i)(/(?:Users|home)/)[^/]+").map_err(error)?;
    let projected = pattern.replace_all(raw, "$1[USER]").into_owned();
    if projected.encode_utf16().count() > 256 {
        return Err(error(()));
    }
    Ok(projected)
}
fn fresh_ready(
    context: &ProcessContext,
    home: &PinnedDirectory,
    deadline: Instant,
) -> AppResult<(String, Generation, u16)> {
    time(deadline)?;
    let record = private_runtime(home)?;
    let before = Generation::observe(record.1)?;
    let root = context
        .runtime
        .managed_installation_root()
        .ok_or_else(|| error(()))?;
    let owner = SystemServiceOwnerProofProvider::new(root, context.clone())?;
    let original_owner = owner.query(context)?;
    if !matches!(original_owner,ServiceOwnerProof::Owned{pid,..} if pid==record.1) {
        return Err(error(()));
    }
    let payload = tauri::async_runtime::block_on(async {
        tokio::time::timeout(
            deadline.saturating_duration_since(Instant::now()),
            crate::infrastructure::attested_status::fetch(
                context.runtime.clone(),
                context.opencodex_home.clone(),
            ),
        )
        .await
        .map_err(error)?
        .map_err(error)
    })?
    .ok_or_else(|| error(()))?;
    if payload["status"] != "running"
        || payload["ready"] != true
        || payload["officialReadiness"] != "ready"
        || payload["pid"].as_str().and_then(|s| s.parse::<u32>().ok()) != Some(record.1)
        || payload["port"].as_u64() != Some(record.2.into())
        || payload["opencodexHome"] != context.opencodex_home.to_string_lossy().as_ref()
    {
        return Err(error(()));
    }
    if private_runtime(home)? != record
        || Generation::observe(before.pid)? != before
        || owner.query(context)? != original_owner
    {
        return Err(error(()));
    }
    time(deadline)?;
    Ok((record.0, before, record.2))
}
impl StartupSession {
    pub(super) fn missing_plan_fingerprint(
        &self,
        context: &ProcessContext,
        head: &session_index::Head,
        leased: bool,
    ) -> AppResult<Option<String>> {
        if !matches!(
            head.state,
            session_index::State::Stopped
                | session_index::State::StopPrepared
                | session_index::State::Ready
        ) || head.restore_plan.is_some()
            || head.references.as_ref() != Some(&self.references()?)
        {
            return Ok(None);
        }
        let evidence = self.restore_evidence(context)?;
        let decision = if head.state != session_index::State::Stopped {
            restore::Pending::observed_preparation_fingerprint(&self.originals, head, &evidence)
        } else {
            restore::Pending::preparation_fingerprint(&self.originals, &evidence)
        }
        .map_err(error)?;
        let original = self.originals.frame(Phase::Original).map_err(error)?.0;
        let stopped = self
            .originals
            .frame(match head.state {
                session_index::State::Ready => Phase::Ready,
                session_index::State::StopPrepared => Phase::PreStop,
                _ => Phase::Stopped,
            })
            .map_err(error)?
            .0;
        let codex = TreeSnapshot::read(&self.originals.client).map_err(error)?;
        if !(original.tree.preplan_remainder_matches(&codex, leased)
            || restore::Pending::empty_config_preplan_matches(
                &self.originals,
                head,
                &original.tree,
                &codex,
                leased,
            )
            .map_err(error)?)
            || {
                let grok = GrokSnapshot::read(&self.originals.profile).map_err(error)?;
                !(original.grok.restored_matches(&grok)
                    || restore::Pending::grok_preplan_matches(
                        &self.originals,
                        head,
                        &original.grok,
                        &grok,
                    )
                    .map_err(error)?)
            }
        {
            return Ok(None);
        }
        for name in [
            ".opencodex-native-main.claim.sqlite",
            ".opencodex-native-main.owner.sqlite",
        ] {
            let id = codex.stable_identity(name).map_err(error)?;
            if stopped.tree.stable_identity(name).map_err(error)? != id
                || !original.tree.original_stable_identity_matches(name, &id)
            {
                return Ok(None);
            }
        }
        Ok(Some(digest(
            &encode(&(
                head,
                decision,
                self.originals.data.identity(),
                self.originals.client.identity(),
                self.originals.profile.identity(),
            ))
            .map_err(error)?,
        )))
    }
    pub(super) fn prepare_missing_plan(
        &self,
        context: &ProcessContext,
        head: &session_index::Head,
        confirmation: &str,
        lease: &NativeLease,
        deadline: Instant,
    ) -> AppResult<(String, String)> {
        time(deadline)?;
        lease.require_context(context, &self.originals.client.path)?;
        if self
            .missing_plan_fingerprint(context, head, true)?
            .as_deref()
            != Some(confirmation)
        {
            return Err(error(()));
        }
        let live = self.live(context)?;
        let (root, _) = source(context)?;
        let owner = SystemServiceOwnerProofProvider::new(root, context.clone())?;
        owner.require_finalization_absence(context, live.process()?, live.parent()?, live.port)?;
        let pending = if head.state != session_index::State::Stopped {
            restore::Pending::prepare_observed(
                &self.originals,
                head,
                self.restore_evidence(context)?,
                lease,
            )
        } else {
            restore::Pending::prepare(&self.originals, self.restore_evidence(context)?, lease)
        }
        .map_err(error)?;
        if head.state != session_index::State::Stopped
            && pending
                .prepared_observation_confirmation(head)
                .map_err(error)?
                != confirmation
        {
            return Err(error(()));
        }
        lease.require_context(context, &self.originals.client.path)?;
        owner.require_finalization_absence(context, live.process()?, live.parent()?, live.port)?;
        time(deadline)?;
        Ok((pending.id().into(), pending.sha256().into()))
    }
    pub(super) fn require_early_absence(
        &self,
        context: &ProcessContext,
        deadline: Instant,
    ) -> AppResult<()> {
        time(deadline)?;
        self.scope(context)?;
        let (root, _) = source(context)?;
        let owner = SystemServiceOwnerProofProvider::new(root, context.clone())?;
        if self.live_sha256.is_some() {
            let live = self.live(context)?;
            owner.require_finalization_absence(
                context,
                live.process()?,
                live.parent()?,
                live.port,
            )?;
        } else if owner.query(context)? != ServiceOwnerProof::Stopped {
            return Err(error(()));
        }
        if capture_file(&self.home, "runtime-port.json", 4096).map_err(error)? != Image::Absent
            || capture_file(&self.home, "ocx.pid", 64).map_err(error)? != Image::Absent
            || owner.query(context)? != ServiceOwnerProof::Stopped
        {
            return Err(error(()));
        }
        time(deadline)
    }
    pub(super) fn rearm_fingerprint(
        &self,
        head: &session_index::Head,
        leased: bool,
    ) -> AppResult<Option<String>> {
        self.originals
            .rearm_fingerprint(head, leased)
            .map_err(error)
    }
    pub(super) fn rearm_rolled_back(
        &self,
        context: &ProcessContext,
        head: &session_index::Head,
        confirmation: &str,
        lease: &NativeLease,
        deadline: Instant,
    ) -> AppResult<(String, String)> {
        time(deadline)?;
        lease.require_context(context, &self.originals.client.path)?;
        if self.rearm_fingerprint(head, true)?.as_deref() != Some(confirmation) {
            return Err(error(()));
        }
        let live = self.live(context)?;
        let (root, _) = source(context)?;
        let owner = SystemServiceOwnerProofProvider::new(root, context.clone())?;
        owner.require_finalization_absence(context, live.process()?, live.parent()?, live.port)?;
        let plan = head.restore_plan.as_ref().ok_or_else(|| error(()))?;
        let parent =
            restore::Pending::open(&self.originals, &plan.id, &plan.sha256).map_err(error)?;
        let pending = parent.rearm_rolled_back().map_err(error)?;
        lease.require_context(context, &self.originals.client.path)?;
        owner.require_finalization_absence(context, live.process()?, live.parent()?, live.port)?;
        time(deadline)?;
        Ok((pending.id().into(), pending.sha256().into()))
    }
    pub(super) fn continue_interrupted_plan(
        &self,
        context: &ProcessContext,
        head: &session_index::Head,
        confirmation: &str,
        lease: &NativeLease,
        deadline: Instant,
    ) -> AppResult<()> {
        self.execute_interrupted_plan(
            context,
            head,
            confirmation,
            lease,
            deadline,
            restore::Decision::Complete,
        )
    }
    pub(super) fn recovery_fingerprint(
        &self,
        head: &session_index::Head,
        decision: restore::Decision,
    ) -> AppResult<Option<String>> {
        self.originals
            .recovery_fingerprint(head, false, decision)
            .map_err(error)
    }
    pub(super) fn execute_interrupted_plan(
        &self,
        context: &ProcessContext,
        head: &session_index::Head,
        confirmation: &str,
        lease: &NativeLease,
        deadline: Instant,
        decision: restore::Decision,
    ) -> AppResult<()> {
        time(deadline)?;
        if self
            .originals
            .recovery_fingerprint(head, true, decision)
            .map_err(error)?
            .as_deref()
            != Some(confirmation)
        {
            return Err(error(()));
        }
        let (root, _) = source(context)?;
        let authority = ExplicitRestoreAuthority {
            session: self,
            context,
            lease,
            deadline,
            owner: SystemServiceOwnerProofProvider::new(root, context.clone())?,
        };
        let live = self.live(context)?;
        authority.owner.require_finalization_absence(
            context,
            live.process()?,
            live.parent()?,
            live.port,
        )?;
        authority.check(&self.originals)?;
        let plan = head.restore_plan.as_ref().ok_or_else(|| error(()))?;
        let pending =
            restore::Pending::open(&self.originals, &plan.id, &plan.sha256).map_err(error)?;
        let permit = restore::MutationPermit::from_explicit(authority);
        pending.apply(decision, &permit).map_err(error)?;
        permit.check(&self.originals).map_err(error)?;
        // The full OS enumeration is a batch-boundary fact. Every native step
        // still rechecks the live native writer lease and exact original
        // process/port/records, just as ordinary Stop retains its exit proof.
        let (root, _) = source(context)?;
        SystemServiceOwnerProofProvider::new(root, context.clone())?.require_finalization_absence(
            context,
            live.process()?,
            live.parent()?,
            live.port,
        )?;
        time(deadline)
    }
    pub(super) fn continued_readback(
        &self,
        context: &ProcessContext,
        head: &session_index::Head,
        deadline: Instant,
    ) -> AppResult<completion::ReadbackProof> {
        self.explicit_readback(context, head, deadline, restore::Decision::Complete)
    }
    pub(super) fn explicit_readback(
        &self,
        context: &ProcessContext,
        head: &session_index::Head,
        deadline: Instant,
        decision: restore::Decision,
    ) -> AppResult<completion::ReadbackProof> {
        time(deadline)?;
        let live = self.live(context)?;
        let (root, _) = source(context)?;
        let owner = SystemServiceOwnerProofProvider::new(root, context.clone())?;
        let require_clear = || {
            owner.require_finalization_absence(context, live.process()?, live.parent()?, live.port)
        };
        require_clear()?;
        let refs = head.references.as_ref().ok_or_else(|| error(()))?.clone();
        let plan = head.restore_plan.as_ref().ok_or_else(|| error(()))?.clone();
        let proof = match decision {
            restore::Decision::Complete => self.originals.complete_readback(refs, plan, deadline),
            restore::Decision::Rollback => self.originals.rollback_readback(refs, plan, deadline),
        }
        .map_err(error)?;
        require_clear()?;
        time(deadline)?;
        Ok(proof)
    }
    /// Metadata-only explicit recovery. This NEVER reissues the old normal
    /// exit proof, enters Pending.apply, or writes any client artifact.
    pub fn finalize_existing_complete(
        &self,
        context: &ProcessContext,
        head: &session_index::Head,
        deadline: Instant,
    ) -> AppResult<completion::ReadbackProof> {
        time(deadline)?;
        if self
            .originals
            .finalization_fingerprint(head)
            .map_err(error)?
            .is_none()
        {
            return Err(error(()));
        }
        let live = self.live(context)?;
        let (root, _) = source(context)?;
        let owner = SystemServiceOwnerProofProvider::new(root, context.clone())?;
        let require_clear = || {
            owner.require_finalization_absence(context, live.process()?, live.parent()?, live.port)
        };
        require_clear()?;
        time(deadline)?;
        let result = self
            .originals
            .finalize_readback(
                head.references.as_ref().ok_or_else(|| error(()))?.clone(),
                head.restore_plan.as_ref().ok_or_else(|| error(()))?.clone(),
                deadline,
            )
            .map_err(error)?;
        require_clear()?;
        time(deadline)?;
        Ok(result)
    }
    pub fn watch_stop_pair(
        &self,
        context: &ProcessContext,
        deadline: Instant,
    ) -> AppResult<crate::infrastructure::macos_process_pair::PairWatch> {
        let live = self.live(context)?;
        crate::infrastructure::macos_process_pair::PairWatch::capture(
            live.process()?,
            live.parent()?,
            deadline,
        )
    }
    pub fn require_stopped_pair(
        &self,
        context: &ProcessContext,
        proof: &crate::infrastructure::macos_process_pair::PairExitProof,
        deadline: Instant,
    ) -> AppResult<()> {
        let live = self.live(context)?;
        proof.require(live.process()?, live.parent()?, deadline)
    }
    pub fn verify_current_ready(
        &self,
        context: &ProcessContext,
        deadline: Instant,
    ) -> AppResult<()> {
        let live = self.live(context)?;
        let generation = live.process()?;
        if fresh_ready(context, &self.home, deadline)?
            != (live.record_sha256, generation, live.port)
        {
            return Err(error(()));
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn begin(
        context: &ProcessContext,
        data_root: &Path,
        client_root: &Path,
        lease: &NativeLease,
        deadline: Instant,
    ) -> AppResult<Self> {
        Self::begin_capture(context, data_root, client_root, lease, deadline, None)
    }
    pub fn begin_reserved(
        context: &ProcessContext,
        data_root: &Path,
        client_root: &Path,
        lease: &NativeLease,
        deadline: Instant,
        reservation: &session_index::CaptureReservation<'_>,
    ) -> AppResult<Self> {
        Self::begin_capture(
            context,
            data_root,
            client_root,
            lease,
            deadline,
            Some(reservation),
        )
    }
    fn begin_capture(
        context: &ProcessContext,
        data_root: &Path,
        client_root: &Path,
        lease: &NativeLease,
        deadline: Instant,
        reservation: Option<&session_index::CaptureReservation<'_>>,
    ) -> AppResult<Self> {
        let phase = std::cell::Cell::new("deadline");
        let result = (|| {
            time(deadline)?;
            phase.set("effective-client");
            if client_root != effective_client(context)? {
                return Err(error(()));
            }
            phase.set("native-lease");
            lease.require_context(context, client_root)?;
            phase.set("program-source");
            let (program_root, selection) = source(context)?;
            phase.set("owner-observation");
            let owner =
                SystemServiceOwnerProofProvider::new(program_root.clone(), context.clone())?;
            if owner.query(context)? != ServiceOwnerProof::Stopped {
                return Err(error(()));
            }
            phase.set("capture-original");
            let originals = match reservation {
                Some(reservation) => Originals::begin_reserved_profile(
                    data_root,
                    client_root,
                    &context.working_directory,
                    reservation,
                ),
                None => {
                    Originals::begin_for_profile(data_root, client_root, &context.working_directory)
                }
            }
            .map_err(error)?;
            phase.set("check-original-lease");
            lease.check()?;
            time(deadline)?;
            phase.set("persist-scope");
            let result = Self::persist(context, originals, program_root, selection)?;
            phase.set("final-lease");
            lease.check()?;
            time(deadline)?;
            Ok(result)
        })();
        if result.is_err() {
            // Fixed stage only. Never expose source config, argv, paths,
            // inherited environment or arbitrary error strings in this trace.
            let _ = crate::infrastructure::runtime_log::RuntimeLog::new(data_root).append_event(
                &format!("client-start-original: refused phase={}", phase.get()),
            );
        }
        result
    }
    fn persist(
        context: &ProcessContext,
        originals: Originals,
        program_root: PathBuf,
        selection: CurrentSelection,
    ) -> AppResult<Self> {
        let profile = PinnedDirectory::open(&context.working_directory).map_err(error)?;
        let home = PinnedDirectory::open(&context.opencodex_home).map_err(error)?;
        let scope = Scope {
            schema_version: 1,
            attempt: originals.attempt.clone(),
            original_sha256: originals.original_sha256.clone(),
            uid: unsafe { libc::geteuid() },
            data: originals.data.path.clone(),
            data_identity: originals.data.identity().clone(),
            profile: profile.path.clone(),
            profile_identity: profile.identity().clone(),
            home: home.path.clone(),
            home_identity: home.identity().clone(),
            client: originals.client.path.clone(),
            client_identity: originals.client.identity().clone(),
            program_root,
            selection,
            captured_at: chrono::Utc::now(),
            declared_routes: declared_routes(&home)?,
        };
        let bytes = encode(&scope).map_err(error)?;
        create(&originals.directory, "startup-scope.json", &bytes).map_err(error)?;
        let result = Self {
            originals,
            scope_sha256: digest(&bytes),
            live_sha256: None,
            profile,
            home,
        };
        result.scope(context)?;
        Ok(result)
    }
    pub fn references(&self) -> AppResult<References> {
        let known = self.originals.known.lock().map_err(error)?;
        Ok(References {
            attempt: self.originals.attempt.clone(),
            original_sha256: self.originals.original_sha256.clone(),
            scope_sha256: self.scope_sha256.clone(),
            phases: [Phase::Ready, Phase::PreStop, Phase::Stopped]
                .into_iter()
                .filter_map(|phase| known[phase.index()].clone().map(|hash| (phase, hash)))
                .collect(),
            live_sha256: self.live_sha256.clone(),
        })
    }
    pub fn original_root_absent(&self) -> bool {
        self.originals.original_root_absence.is_some()
    }
    pub fn open_indexed(
        context: &ProcessContext,
        index: &session_index::Index,
        head: &session_index::Head,
    ) -> AppResult<Self> {
        if index.has_original_root_absence() {
            let originals = index.registered_root_originals(head).map_err(error)?;
            let refs = head.references.as_ref().ok_or_else(|| error(()))?;
            let result = Self {
                originals,
                scope_sha256: refs.scope_sha256.clone(),
                live_sha256: refs.live_sha256.clone(),
                profile: PinnedDirectory::open(&context.working_directory).map_err(error)?,
                home: PinnedDirectory::open(&context.opencodex_home).map_err(error)?,
            };
            if result.live_sha256.is_some() {
                result.live(context)?;
            } else {
                result.scope(context)?;
            }
            Ok(result)
        } else {
            Self::open(
                context,
                index.data_root(),
                &effective_client(context)?,
                head.references.as_ref().ok_or_else(|| error(()))?,
            )
        }
    }
    pub fn first_root_ready<'a>(
        &'a mut self,
        context: &'a ProcessContext,
        lease: &'a NativeLease,
        expected: &session_index::Head,
        deadline: Instant,
    ) -> AppResult<RootPhaseProof<'a>> {
        if !self.original_root_absent() {
            return Err(error(()));
        }
        self.bind_ready(context, lease, deadline)?;
        let proof = RootPhaseProof {
            session: self,
            context,
            expected: expected.clone(),
            authority: RootPhaseAuthority::Ready(lease),
            deadline,
        };
        Ok(proof)
    }
    pub fn first_root_pre_stop<'a>(
        &'a self,
        context: &'a ProcessContext,
        expected: &session_index::Head,
        deadline: Instant,
    ) -> AppResult<RootPhaseProof<'a>> {
        if !self.original_root_absent() {
            return Err(error(()));
        }
        self.seal_pre_stop(context, deadline)?;
        let proof = RootPhaseProof {
            session: self,
            context,
            expected: expected.clone(),
            authority: RootPhaseAuthority::PreStop,
            deadline,
        };
        Ok(proof)
    }
    pub fn first_root_stopped<'a>(
        &'a self,
        request: &RootStopRequest<'a>,
    ) -> AppResult<RootPhaseProof<'a>> {
        let RootStopRequest {
            context,
            stop,
            pair,
            lease,
            head,
            deadline,
        } = *request;
        let deadline = stop.bound_deadline(deadline);
        if !self.original_root_absent() {
            return Err(error(()));
        }
        self.seal_completed_stop(context, stop, pair, lease, deadline)?;
        let proof = RootPhaseProof {
            session: self,
            context,
            expected: head.clone(),
            authority: RootPhaseAuthority::Stopped(lease, stop, pair),
            deadline,
        };
        Ok(proof)
    }
    pub fn restore_evidence(
        &self,
        context: &ProcessContext,
    ) -> AppResult<crate::infrastructure::macos_client_originals::restore::Evidence> {
        let scope = self.scope(context)?;
        let live = self.live(context)?;
        let generation = live.process()?;
        if declared_routes(&self.home)? != scope.declared_routes {
            return Err(error(()));
        }
        Ok(
            crate::infrastructure::macos_client_originals::restore::Evidence {
                declared_routes: scope.declared_routes,
                retained_pids: vec![generation.pid, live.parent()?.pid],
                audit_home: audit_home(self.home.path())?,
                not_before: scope.captured_at,
                not_after: chrono::Utc::now(),
            },
        )
    }
    pub fn prepare_restore<'a>(
        &'a self,
        context: &ProcessContext,
        stop: &CompletedLifecycleStop,
        pair: &crate::infrastructure::macos_process_pair::PairExitProof,
        lease: &NativeLease,
        deadline: Instant,
    ) -> AppResult<crate::infrastructure::macos_client_originals::restore::Pending<'a>> {
        time(deadline)?;
        self.require_stopped_pair(context, pair, deadline)?;
        lease.require_context(context, &self.originals.client.path)?;
        let live = self.live(context)?;
        stop.require_original(
            context,
            &self.originals.data.path,
            live.process()?,
            &live.record_sha256,
        )?;
        self.originals.read(Phase::Stopped).map_err(error)?;
        if capture_file(&self.home, "runtime-port.json", 4096).map_err(error)? != Image::Absent {
            return Err(error(()));
        }
        let evidence = self.restore_evidence(context)?;
        let pending = crate::infrastructure::macos_client_originals::restore::Pending::prepare(
            &self.originals,
            evidence,
            lease,
        )
        .map_err(error)?;
        time(deadline)?;
        stop.require_original(
            context,
            &self.originals.data.path,
            live.process()?,
            &live.record_sha256,
        )?;
        if capture_file(&self.home, "runtime-port.json", 4096).map_err(error)? != Image::Absent {
            return Err(error(()));
        }
        Ok(pending)
    }
    pub fn validate_restore_plan(
        &self,
        context: &ProcessContext,
        id: &str,
        sha256: &str,
    ) -> AppResult<()> {
        self.scope(context)?;
        crate::infrastructure::macos_client_originals::restore::Pending::open(
            &self.originals,
            id,
            sha256,
        )
        .map_err(error)?;
        Ok(())
    }
    /// Ordinary current-session compensation only. A pending/crashed session
    /// cannot manufacture either Stop or pair authority from persisted bytes.
    pub fn restore_completed_stop(
        &self,
        context: &ProcessContext,
        stop: &CompletedLifecycleStop,
        pair: &crate::infrastructure::macos_process_pair::PairExitProof,
        lease: &NativeLease,
        deadline: Instant,
        plan: &session_index::RestorePlan,
    ) -> AppResult<()> {
        let phase = std::cell::Cell::new("authority-admission");
        let result = (|| {
            let authority = ArtifactRestoreAuthority {
                session: self,
                context,
                stop,
                pair,
                lease,
                deadline,
            };
            authority.check(&self.originals)?;
            phase.set("service-stopped");
            let (root, _) = source(context)?;
            let owner = SystemServiceOwnerProofProvider::new(root, context.clone())?;
            if owner.query(context)? != ServiceOwnerProof::Stopped {
                return Err(error(()));
            }
            let pending =
                restore::Pending::open(&self.originals, &plan.id, &plan.sha256).map_err(error)?;
            phase.set("apply-plan");
            let permit = restore::MutationPermit::from_authority(authority);
            pending
                .apply(restore::Decision::Complete, &permit)
                .map_err(|e| {
                    #[cfg(test)]
                    eprintln!(
                        "ROOT_ARTIFACT_RESTORE_REFUSAL phase=apply-plan kind={:?} reason={}",
                        e.kind(),
                        e
                    );
                    error(e)
                })?;
            phase.set("post-apply-authority");
            permit.check(&self.originals).map_err(error)?;
            phase.set("post-apply-stopped");
            if owner.query(context)? != ServiceOwnerProof::Stopped {
                return Err(error(()));
            }
            time(deadline)
        })();
        if result.is_err() {
            let _ = crate::infrastructure::runtime_log::RuntimeLog::new(self.originals.data.path())
                .append_event(&format!(
                    "client-artifact-restore: refused phase={}",
                    phase.get()
                ));
        }
        result
    }
    fn verify_root_stop_binding(
        &self,
        context: &ProcessContext,
        head: &session_index::Head,
        plan: &session_index::RestorePlan,
        expected_scope: &Scope,
        expected_live: &Live,
    ) -> AppResult<()> {
        if head.references.as_ref() != Some(&self.references()?) {
            return Err(error(()));
        }
        let pending = root_retirement::Pending::open(&self.originals, &plan.id, &plan.sha256)
            .map_err(error)?;
        pending.require_source(head).map_err(error)?;
        let view = pending.current_originals().map_err(error)?;
        let refs = head.references.as_ref().ok_or_else(|| error(()))?;
        verify_index_scope(&view, refs, &context.opencodex_home)?;
        let scope: Scope = parse(&read_private(
            &view,
            "startup-scope.json",
            &refs.scope_sha256,
        )?)?;
        let live: Live = parse(&read_private(
            &view,
            "startup-live.json",
            refs.live_sha256.as_deref().ok_or_else(|| error(()))?,
        )?)?;
        let (_, ready) = view.frame(Phase::Ready).map_err(error)?;
        view.frame(Phase::Stopped).map_err(error)?;
        if scope != *expected_scope
            || live != *expected_live
            || digest(&ready) != live.ready_sha256
            || live.scope_sha256 != refs.scope_sha256
        {
            return Err(error(()));
        }
        let (root, selection) = source(context)?;
        if root != scope.program_root || selection != scope.selection {
            return Err(error(()));
        }
        Ok(())
    }
    fn require_root_stop(
        &self,
        context: &ProcessContext,
        stop: &CompletedLifecycleStop,
        pair: &crate::infrastructure::macos_process_pair::PairExitProof,
        lease: &NativeLease,
        head: &session_index::Head,
        deadline: Instant,
    ) -> AppResult<(Scope, Live, SystemServiceOwnerProofProvider, Instant)> {
        let deadline = stop.bound_deadline(deadline);
        time(deadline)?;
        if self.originals.original_root_absence.is_none()
            || head.root_retirement.is_some()
            || !matches!(
                head.state,
                session_index::State::Stopped | session_index::State::RestorePrepared
            )
            || head.references.as_ref() != Some(&self.references()?)
        {
            return Err(error(()));
        }
        // Exact original normal Stop history is mandatory. A preexisting
        // Stopped observation, Ready-only recovery or disk receipt cannot join.
        self.originals.frame(Phase::Stopped).map_err(error)?;
        let scope = self.scope(context)?;
        let live = self.live(context)?;
        lease.require_context(context, &self.originals.logical_client())?;
        pair.require(live.process()?, live.parent()?, deadline)?;
        let owner =
            SystemServiceOwnerProofProvider::new(scope.program_root.clone(), context.clone())?;
        owner.require_finalization_absence(context, live.process()?, live.parent()?, live.port)?;
        stop.require_original_absence(
            self.originals.data.path(),
            live.process()?,
            live.parent()?,
            &live.record_sha256,
            live.port,
            &owner,
        )?;
        time(deadline)?;
        Ok((scope, live, owner, deadline))
    }
    /// Must run before creating the config Guard; its Bun holder is itself a
    /// current same-HOME runtime-shaped process and cannot pass a whole-scope
    /// unused scan. Held-phase checks below stay constrained by this admission.
    pub fn admit_root_completed_stop<'a>(
        &'a self,
        request: &RootStopRequest<'a>,
    ) -> AppResult<RootStopAdmission<'a>> {
        let RootStopRequest {
            context,
            stop,
            pair,
            lease,
            head,
            deadline,
        } = *request;
        let (scope, live, _, deadline) =
            self.require_root_stop(context, stop, pair, lease, head, deadline)?;
        Ok(RootStopAdmission {
            session: self,
            context,
            stop,
            pair,
            lease,
            head: head.clone(),
            scope,
            live,
            deadline,
        })
    }
    pub fn prepare_root_completed_stop<'a>(
        &'a self,
        admission: &RootStopAdmission<'_>,
        guard: &mut crate::infrastructure::official_client_coordination::native::config::Guard<'_>,
    ) -> AppResult<root_retirement::Pending<'a>> {
        admission.check_before_plan(self)?;
        let pending = root_retirement::Pending::prepare_guarded(
            admission.context,
            &self.originals,
            &admission.head,
            admission.lease,
            guard,
        )
        .map_err(error)?;
        admission.check_before_plan(self)?;
        Ok(pending)
    }
    pub fn retire_root_completed_stop(
        &self,
        admission: &RootStopAdmission<'_>,
        registration: &session_index::RootRetirementRegistration<'_>,
        guard: crate::infrastructure::official_client_coordination::native::config::Guard<'_>,
    ) -> AppResult<root_retirement::AuthorizedReadback> {
        admission.check_before_plan(self)?;
        let plan = registration.plan().map_err(error)?.clone();
        registration
            .require(&admission.head, &plan)
            .map_err(error)?;
        let pending = root_retirement::Pending::open(&self.originals, &plan.id, &plan.sha256)
            .map_err(error)?;
        let owner = SystemServiceOwnerProofProvider::new(
            admission.scope.program_root.clone(),
            admission.context.clone(),
        )?;
        let authority = RootRetirementAuthority {
            session: self,
            context: admission.context,
            stop: admission.stop,
            pair: admission.pair,
            lease: admission.lease,
            owner,
            head: admission.head.clone(),
            plan,
            scope: admission.scope.clone(),
            live: admission.live.clone(),
            deadline: admission.deadline,
            registration,
        };
        let proof = pending
            .apply_completed_stop(authority, admission.lease, guard, admission.deadline)
            .map_err(error)?;
        time(admission.deadline)?;
        Ok(proof)
    }
    pub fn completed_readback(
        &self,
        context: &ProcessContext,
        stop: &CompletedLifecycleStop,
        pair: &crate::infrastructure::macos_process_pair::PairExitProof,
        deadline: Instant,
        plan: &session_index::RestorePlan,
    ) -> AppResult<completion::ReadbackProof> {
        // Restart has a larger outer budget; it cannot lend that extra time
        // to completion of this Stop generation.
        let deadline = stop.bound_deadline(deadline);
        self.require_stopped_pair(context, pair, deadline)?;
        let live = self.live(context)?;
        stop.require_original(
            context,
            &self.originals.data.path,
            live.process()?,
            &live.record_sha256,
        )?;
        let (root, _) = source(context)?;
        let owner = SystemServiceOwnerProofProvider::new(root, context.clone())?;
        if owner.query(context)? != ServiceOwnerProof::Stopped {
            return Err(error(()));
        }
        let proof = self
            .originals
            .complete_readback(self.references()?, plan.clone(), deadline)
            .map_err(error)?;
        if owner.query(context)? != ServiceOwnerProof::Stopped {
            return Err(error(()));
        }
        self.require_stopped_pair(context, pair, deadline)?;
        time(deadline)?;
        Ok(proof)
    }
    pub fn open(
        context: &ProcessContext,
        data_root: &Path,
        client_root: &Path,
        refs: &References,
    ) -> AppResult<Self> {
        let originals = Originals::open_for_profile(
            data_root,
            client_root,
            &context.working_directory,
            &refs.attempt,
            &refs.original_sha256,
            &refs.phases,
        )
        .map_err(error)?;
        let result = Self {
            originals,
            scope_sha256: refs.scope_sha256.clone(),
            live_sha256: refs.live_sha256.clone(),
            profile: PinnedDirectory::open(&context.working_directory).map_err(error)?,
            home: PinnedDirectory::open(&context.opencodex_home).map_err(error)?,
        };
        if result.live_sha256.is_some() {
            result.live(context)?;
        } else {
            result.scope(context)?;
        }
        Ok(result)
    }
    fn scope(&self, context: &ProcessContext) -> AppResult<Scope> {
        self.originals.frame(Phase::Original).map_err(error)?;
        self.scope_binding(context)
    }
    /// The caller must independently validate the required fresh frame prefix
    /// in THIS observation. This checks every scope/root/program binding but
    /// does not redundantly reread Original before live() reads Original+Ready.
    fn scope_binding(&self, context: &ProcessContext) -> AppResult<Scope> {
        self.profile.recheck().map_err(error)?;
        self.home.recheck().map_err(error)?;
        let scope: Scope = parse(&read_private(
            &self.originals,
            "startup-scope.json",
            &self.scope_sha256,
        )?)?;
        let (program_root, selection) = source(context)?;
        if scope.schema_version != 1
            || scope.attempt != self.originals.attempt
            || scope.original_sha256 != self.originals.original_sha256
            || scope.uid != unsafe { libc::geteuid() }
            || scope.data != self.originals.data.path
            || scope.data_identity != *self.originals.data.identity()
            || scope.profile != context.working_directory
            || scope.profile != self.profile.path
            || scope.profile_identity != *self.profile.identity()
            || scope.home != context.opencodex_home
            || scope.home != self.home.path
            || scope.home_identity != *self.home.identity()
            || scope.client != self.originals.client.path
            || scope.client != effective_client(context)?
            || scope.client_identity != *self.originals.client.identity()
            || scope.program_root != program_root
            || scope.selection != selection
        {
            return Err(error(()));
        }
        Ok(scope)
    }
    fn live(&self, context: &ProcessContext) -> AppResult<Live> {
        self.scope_binding(context)?;
        let sha = self.live_sha256.as_deref().ok_or_else(|| error(()))?;
        let live: Live = parse(&read_private(&self.originals, "startup-live.json", sha)?)?;
        let (_, ready) = self.originals.frame(Phase::Ready).map_err(error)?;
        if live.schema_version != 1
            || live.scope_sha256 != self.scope_sha256
            || live.ready_sha256 != digest(&ready)
        {
            return Err(error(()));
        }
        live.process()?;
        live.parent()?;
        Ok(live)
    }
    /// Fresh signed health/ready/health and unchanged private record, surrounded
    /// by actual native UID/birth readback. No file-exists or cached PID shortcut.
    pub fn bind_ready(
        &mut self,
        context: &ProcessContext,
        lease: &NativeLease,
        deadline: Instant,
    ) -> AppResult<String> {
        time(deadline)?;
        self.scope(context)?;
        lease.require_context(context, &self.originals.client.path)?;
        if self.live_sha256.is_some() {
            return Err(error(()));
        }
        let (record_sha256, generation, port) = fresh_ready(context, &self.home, deadline)?;
        let parent = Generation::observe(crate::infrastructure::macos_process_exit::parent_pid(
            generation.pid,
        )?)?;
        let ready_sha256 = self.originals.seal(Phase::Ready).map_err(error)?;
        lease.check()?;
        if private_runtime(&self.home)? != (record_sha256.clone(), generation.pid, port)
            || Generation::observe(generation.pid)? != generation
            || crate::infrastructure::macos_process_exit::parent_pid(generation.pid)? != parent.pid
            || Generation::observe(parent.pid)? != parent
        {
            return Err(error(()));
        }
        let live = Live {
            schema_version: 1,
            scope_sha256: self.scope_sha256.clone(),
            ready_sha256,
            record_sha256,
            generation: [
                generation.pid.into(),
                generation.uid.into(),
                generation.start_seconds,
                generation.start_micros,
            ],
            port,
            parent_generation: [
                parent.pid.into(),
                parent.uid.into(),
                parent.start_seconds,
                parent.start_micros,
            ],
        };
        let bytes = encode(&live).map_err(error)?;
        time(deadline)?;
        create(&self.originals.directory, "startup-live.json", &bytes).map_err(error)?;
        let hash = digest(&bytes);
        self.live_sha256 = Some(hash.clone());
        self.live(context)?;
        time(deadline)?;
        Ok(hash)
    }
    /// Read-only observer cannot reseal ready or adopt a different runtime. This
    /// stage deliberately provides no additional client-mutation authority.
    pub fn seal_pre_stop(&self, context: &ProcessContext, deadline: Instant) -> AppResult<String> {
        let live = self.live(context)?;
        let generation = live.process()?;
        if fresh_ready(context, &self.home, deadline)?
            != (live.record_sha256.clone(), generation, live.port)
        {
            return Err(error(()));
        }
        let hash = self.originals.seal(Phase::PreStop).map_err(error)?;
        time(deadline)?;
        Ok(hash)
    }
    /// This binds the original real Stop proof to the persisted ready snapshot;
    /// it does NOT mint MutationPermit or claim complete family/listener proof.
    pub fn seal_completed_stop(
        &self,
        context: &ProcessContext,
        stop: &CompletedLifecycleStop,
        pair: &crate::infrastructure::macos_process_pair::PairExitProof,
        lease: &NativeLease,
        deadline: Instant,
    ) -> AppResult<String> {
        self.require_stopped_pair(context, pair, deadline)?;
        lease.require_context(context, &self.originals.client.path)?;
        let live = self.live(context)?;
        stop.require_original(
            context,
            &self.originals.data.path,
            live.process()?,
            &live.record_sha256,
        )?;
        if capture_file(&self.home, "runtime-port.json", 4096).map_err(error)? != Image::Absent
            || capture_file(&self.home, "ocx.pid", 64).map_err(error)? != Image::Absent
        {
            return Err(error(()));
        }
        let hash = self.originals.seal(Phase::Stopped).map_err(error)?;
        lease.check()?;
        stop.require_original(
            context,
            &self.originals.data.path,
            live.process()?,
            &live.record_sha256,
        )?;
        if capture_file(&self.home, "runtime-port.json", 4096).map_err(error)? != Image::Absent
            || capture_file(&self.home, "ocx.pid", 64).map_err(error)? != Image::Absent
        {
            return Err(error(()));
        }
        Ok(hash)
    }
}

#[cfg(test)]
#[path = "macos_client_session_tests.rs"]
mod tests;
