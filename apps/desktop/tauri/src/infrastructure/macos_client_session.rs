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
}
impl Live {
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
#[derive(Clone, Serialize, Deserialize)]
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
    pub fn begin(
        context: &ProcessContext,
        data_root: &Path,
        client_root: &Path,
        lease: &NativeLease,
        deadline: Instant,
    ) -> AppResult<Self> {
        time(deadline)?;
        if client_root != effective_client(context)? {
            return Err(error(()));
        }
        lease.require_context(context, client_root)?;
        let (program_root, selection) = source(context)?;
        let owner = SystemServiceOwnerProofProvider::new(program_root.clone(), context.clone())?;
        if owner.query(context)? != ServiceOwnerProof::Stopped {
            return Err(error(()));
        }
        let originals = Originals::begin(data_root, client_root).map_err(error)?;
        lease.check()?;
        time(deadline)?;
        let result = Self::persist(context, originals, program_root, selection)?;
        lease.check()?;
        time(deadline)?;
        Ok(result)
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
    pub fn open(
        context: &ProcessContext,
        data_root: &Path,
        client_root: &Path,
        refs: &References,
    ) -> AppResult<Self> {
        let originals = Originals::open(
            data_root,
            client_root,
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
        result.scope(context)?;
        if result.live_sha256.is_some() {
            result.live(context)?;
        }
        Ok(result)
    }
    fn scope(&self, context: &ProcessContext) -> AppResult<Scope> {
        self.profile.recheck().map_err(error)?;
        self.home.recheck().map_err(error)?;
        self.originals.frame(Phase::Original).map_err(error)?;
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
        self.scope(context)?;
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
        let ready_sha256 = self.originals.seal(Phase::Ready).map_err(error)?;
        lease.check()?;
        if private_runtime(&self.home)? != (record_sha256.clone(), generation.pid, port)
            || Generation::observe(generation.pid)? != generation
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
        lease: &NativeLease,
    ) -> AppResult<String> {
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
