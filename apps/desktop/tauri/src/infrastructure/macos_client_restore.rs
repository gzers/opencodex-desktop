//! Native reversible publication of the assessed artifacts and, when admitted,
//! originally absent first-use Codex config and Grok backup. All targets share the same
//! native-object review, conflict preservation and immutable outcome.
//! Ordinary current-session calls require borrowed original Stop/pair/lease
//! authority. Content assessment or private journals alone cannot mint it.
//! Descriptor checks followed by rename are not a kernel compare-and-swap on
//! an expected inode. The lifecycle factory must prove live official writer
//! exclusion across the mutation, not merely a formerly valid lease.
//! Whole-session success remains separate from artifact publication.
use super::*;
use crate::modules::runtime::client_artifact_policy as policy;
#[path = "macos_client_empty_config.rs"]
mod empty_config;
use empty_config::{EmptyConfig, History as ConfigHistory};
#[path = "macos_client_grok_backup.rs"]
mod grok_backup;
use grok_backup::{GrokBackup, History as GrokHistory};
struct Histories {
    config: ConfigHistory,
    grok: GrokHistory,
}

fn target_name(i: usize) -> io::Result<&'static str> {
    if i == 4 {
        Ok("config.toml.bak-opencodex")
    } else if i == 3 {
        Ok("config.toml")
    } else {
        NAMES
            .get(i)
            .copied()
            .ok_or_else(|| denied("invalid artifact index"))
    }
}

unsafe extern "C" {
    fn acl_copy_int(bytes: *const libc::c_void) -> *mut libc::c_void;
    fn acl_init(count: libc::c_int) -> *mut libc::c_void;
    fn acl_set_fd_np(fd: libc::c_int, acl: *mut libc::c_void, kind: libc::c_int) -> libc::c_int;
}

/// No public constructor/deserializer/default and no transferable stored
/// success boolean. The session owner is the only authority constructor.
pub struct MutationPermit<'a> {
    authority: Authority<'a>,
}
enum Authority<'a> {
    Current(session::ArtifactRestoreAuthority<'a>),
    Explicit(session::ExplicitRestoreAuthority<'a>),
}
#[inline]
fn writer_refusal(
    error: crate::errors::AppError,
    stage: &'static str,
    message: &'static str,
) -> io::Error {
    #[cfg(test)]
    {
        let code = match &error {
            crate::errors::AppError::Timeout => "timeout",
            crate::errors::AppError::RuntimeManaged { code, .. } => code.as_str(),
            _ => "other-error",
        };
        eprintln!("ROOT_STOP_WRITER_REFUSAL stage={stage} code={code} mapped_to=permission-denied");
    }
    #[cfg(not(test))]
    let _ = (error, stage);
    denied(message)
}
impl<'a> MutationPermit<'a> {
    pub(super) fn from_authority(authority: session::ArtifactRestoreAuthority<'a>) -> Self {
        Self {
            authority: Authority::Current(authority),
        }
    }
    pub(super) fn from_explicit(authority: session::ExplicitRestoreAuthority<'a>) -> Self {
        Self {
            authority: Authority::Explicit(authority),
        }
    }
    fn grok_write_guard(
        &self,
    ) -> io::Result<crate::infrastructure::official_client_coordination::native::config::Guard<'_>>
    {
        match &self.authority {
            Authority::Current(a) => a.grok_write_guard(),
            Authority::Explicit(a) => a.grok_write_guard(),
        }
        .map_err(|_| denied("official Grok writer exclusion unavailable"))
    }
    fn config_write_guard(
        &self,
    ) -> io::Result<crate::infrastructure::official_client_coordination::native::config::Guard<'_>>
    {
        match &self.authority {
            Authority::Current(a) => a.config_write_guard(),
            Authority::Explicit(a) => a.config_write_guard(),
        }
        .map_err(|_| denied("official config write exclusion unavailable"))
    }
    pub(super) fn check(&self, originals: &Originals) -> io::Result<()> {
        let checked = match &self.authority {
            Authority::Current(a) => a.check(originals),
            Authority::Explicit(a) => a.check(originals),
        };
        #[cfg(test)]
        if let Err(error) = &checked {
            let code = match error {
                crate::errors::AppError::Timeout => "timeout",
                crate::errors::AppError::RuntimeManaged { code, .. } => code.as_str(),
                _ => "other-error",
            };
            eprintln!("ROOT_STOP_AUTHORITY_REFUSAL code={code} mapped_to=permission-denied");
        }
        checked.map_err(|_| denied("current session/original/coordination authority changed"))
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Evidence {
    pub declared_routes: Option<Vec<String>>,
    pub retained_pids: Vec<u32>,
    pub audit_home: String,
    pub not_before: chrono::DateTime<chrono::Utc>,
    pub not_after: chrono::DateTime<chrono::Utc>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Complete,
    Rollback,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    Keep,
    Swap,
    Remove,
    Publish,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Plan {
    #[serde(skip)]
    object_id: String,
    #[serde(skip)]
    rearmed: bool,
    schema_version: u32,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    journal_stages: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recovery_observation: Option<RecoveryObservation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    empty_config: Option<EmptyConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    grok_backup: Option<GrokBackup>,
    id: String,
    attempt: String,
    phases: Vec<String>,
    kinds: [Kind; 3],
    staged: [Option<Identity>; 3],
    evidence: Evidence,
}
impl Plan {
    #[cfg(test)]
    fn count(&self) -> usize {
        3 + usize::from(self.empty_config.is_some()) + usize::from(self.grok_backup.is_some())
    }
    fn indices(&self) -> impl Iterator<Item = usize> {
        (0..3)
            .chain(self.empty_config.as_ref().map(|_| 3))
            .chain(self.grok_backup.as_ref().map(|_| 4))
    }
    fn kind(&self, i: usize) -> Kind {
        if i >= 3 {
            Kind::Remove
        } else {
            self.kinds[i]
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RearmedPlan {
    schema_version: u32,
    id: String,
    attempt: String,
    parent: session_index::RestorePlan,
}
/// An observation made NOW in a separate confirmed recovery action. It is not
/// the missing historical Stopped frame or a persisted normal-exit certificate.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RecoveryObservation {
    schema_version: u32,
    source_head: session_index::Head,
    observed_at: chrono::DateTime<chrono::Utc>,
    capture: Capture,
    tree: TreeSnapshot,
    grok: GrokSnapshot,
}

struct Projection {
    generation: [String; 3],
    protection: [String; 3],
}
impl Projection {
    fn new(c: &Capture) -> io::Result<Self> {
        let mut generation = std::array::from_fn(|_| String::new());
        let mut protections = generation.clone();
        for (i, image) in c.images.iter().enumerate() {
            if let Image::File {
                identity,
                protection,
                change,
                ..
            } = image
            {
                generation[i] = digest(&encode(&(identity, change))?);
                protections[i] = digest(&encode(protection)?);
            }
        }
        Ok(Self {
            generation,
            protection: protections,
        })
    }
    fn stage<'a>(&'a self, c: &'a Capture) -> policy::Stage<'a> {
        let one = |i: usize| match &c.images[i] {
            Image::Absent => policy::Snapshot::Absent,
            Image::File { bytes, .. } => policy::Snapshot::File {
                bytes,
                generation: self.generation[i].as_str(),
                protection: self.protection[i].as_str(),
            },
        };
        policy::Stage {
            catalog: one(0),
            cache: one(1),
            audit: one(2),
        }
    }
}
fn kinds(c: &[Capture; 4], e: &Evidence) -> io::Result<[Kind; 3]> {
    let p = [
        Projection::new(&c[0])?,
        Projection::new(&c[1])?,
        Projection::new(&c[2])?,
        Projection::new(&c[3])?,
    ];
    let assessed = policy::assess(&policy::Observations {
        before: p[0].stage(&c[0]),
        ready: p[1].stage(&c[1]),
        pre_stop: p[2].stage(&c[2]),
        stopped: p[3].stage(&c[3]),
        declared_routes: e.declared_routes.as_deref(),
        retained_pids: &e.retained_pids,
        audit_home: &e.audit_home,
        audit_not_before: e.not_before,
        audit_not_after: e.not_after,
    })
    .map_err(|_| denied("client content not eligible for compensation"))?;
    Ok(kind_actions(c, &assessed))
}
fn kinds_current(c: &[Capture; 4], e: &Evidence) -> io::Result<[Kind; 3]> {
    let before = Projection::new(&c[0])?;
    let ready = Projection::new(&c[1])?;
    let current = Projection::new(&c[3])?;
    let assessed = policy::assess_current_recovery(&policy::CurrentRecoveryObservations {
        before: before.stage(&c[0]),
        ready: ready.stage(&c[1]),
        current: current.stage(&c[3]),
        declared_routes: e.declared_routes.as_deref(),
        retained_pids: &e.retained_pids,
        audit_home: &e.audit_home,
        audit_not_before: e.not_before,
        audit_not_after: e.not_after,
    })
    .map_err(|_| denied("current recovery content not eligible"))?;
    Ok(kind_actions(c, &assessed))
}
fn kind_actions(c: &[Capture; 4], assessed: &policy::Assessment) -> [Kind; 3] {
    std::array::from_fn(|i| {
        if assessed.0[i].1 == policy::Action::AlreadyOriginal {
            Kind::Keep
        } else {
            match (&c[3].images[i], &c[0].images[i]) {
                (Image::File { .. }, Image::File { .. }) => Kind::Swap,
                (Image::File { .. }, Image::Absent) => Kind::Remove,
                (Image::Absent, Image::File { .. }) => Kind::Publish,
                _ => Kind::Keep,
            }
        }
    })
}
fn same(left: &Image, right: &Image, exact_time: bool) -> bool {
    if exact_time {
        return left == right;
    }
    match (left, right) {
        (Image::Absent, Image::Absent) => true,
        (
            Image::File {
                identity: a,
                protection: ap,
                bytes: ab,
                ..
            },
            Image::File {
                identity: b,
                protection: bp,
                bytes: bb,
                ..
            },
        ) => a == b && ap == bp && ab == bb,
        _ => false,
    }
}
fn original_content(left: &Image, right: &Image) -> bool {
    match (left, right) {
        (Image::Absent, Image::Absent) => true,
        (
            Image::File {
                protection: a,
                bytes: ab,
                ..
            },
            Image::File {
                protection: b,
                bytes: bb,
                ..
            },
        ) => a == b && ab == bb,
        _ => false,
    }
}
fn stat_image(original: &Image, id: Option<&Identity>) -> Image {
    match (original, id) {
        (
            Image::File {
                protection, bytes, ..
            },
            Some(identity),
        ) => Image::File {
            identity: identity.clone(),
            protection: protection.clone(),
            bytes: bytes.clone(),
            change: [0; 4],
        },
        _ => Image::Absent,
    }
}
fn rename(
    from: &PinnedDirectory,
    a: &str,
    to: &PinnedDirectory,
    b: &str,
    flags: u32,
) -> io::Result<()> {
    from.recheck()?;
    to.recheck()?;
    let a = name(OsStr::new(a))?;
    let b = name(OsStr::new(b))?;
    if unsafe {
        libc::renameatx_np(
            from.file().as_raw_fd(),
            a.as_ptr(),
            to.file().as_raw_fd(),
            b.as_ptr(),
            flags,
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    from.sync()?;
    if from.identity() != to.identity() {
        to.sync()?;
    }
    Ok(())
}
fn metadata(file: &File, p: &Protection) -> io::Result<()> {
    if p.uid != unsafe { libc::geteuid() } || p.flags != 0 {
        return Err(denied(
            "unsupported original owner or flags; not normalized",
        ));
    }
    if unsafe { libc::fchown(file.as_raw_fd(), p.uid, p.gid) } != 0
        || unsafe { libc::fchmod(file.as_raw_fd(), p.mode) } != 0
    {
        return Err(io::Error::last_os_error());
    }
    let acl = if let Some(bytes) = &p.acl {
        unsafe { acl_copy_int(bytes.as_ptr().cast()) }
    } else {
        unsafe { acl_init(0) }
    };
    if acl.is_null() {
        return Err(denied("original ACL cannot be materialized"));
    }
    let result = unsafe { acl_set_fd_np(file.as_raw_fd(), acl, 0x100) };
    unsafe {
        acl_free(acl);
    }
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    let desired: BTreeMap<_, _> = p.xattrs.iter().cloned().collect();
    for key in xattrs(file)?.keys() {
        if !desired.contains_key(key) {
            let c = CString::new(key.clone()).map_err(|_| denied("invalid original xattr"))?;
            if unsafe { libc::fremovexattr(file.as_raw_fd(), c.as_ptr(), 0) } != 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }
    for (key, bytes) in &desired {
        let c = CString::new(key.clone()).map_err(|_| denied("invalid original xattr"))?;
        if unsafe {
            libc::fsetxattr(
                file.as_raw_fd(),
                c.as_ptr(),
                bytes.as_ptr().cast(),
                bytes.len(),
                0,
                0,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    if protection(file, &stat(file)?)? != *p {
        return Err(denied("staged original metadata differs"));
    }
    Ok(())
}
fn stage(
    client: &PinnedDirectory,
    filename: &str,
    original: &Image,
    checkpoint: &impl Fn(&str) -> io::Result<()>,
) -> io::Result<Option<Identity>> {
    let Image::File {
        protection, bytes, ..
    } = original
    else {
        return Ok(None);
    };
    client.recheck()?;
    let n = name(OsStr::new(filename))?;
    let fd = unsafe {
        libc::openat(
            client.file().as_raw_fd(),
            n.as_ptr(),
            libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut file = unsafe { File::from_raw_fd(fd) };
    checkpoint("stage_created")?;
    file.write_all(bytes)?;
    checkpoint("stage_bytes")?;
    metadata(&file, protection)?;
    checkpoint("stage_protected")?;
    file.sync_all()?;
    client.sync()?;
    checkpoint("stage_synced")?;
    let staged_stat = stat(&file)?;
    if staged_stat.st_nlink != 1 {
        return Err(denied("created stage lost its original file identity"));
    }
    let captured = capture_file(client, filename, MAX_CONTENT)?;
    if !original_content(&captured, original) {
        return Err(denied("original staging readback differs"));
    }
    if let Image::File { identity, .. } = captured {
        if identity != super::identity(&staged_stat)
            || super::identity(&stat(&file)?) != identity
            || stat(&file)?.st_nlink != 1
        {
            return Err(denied("stage path no longer names the created file"));
        }
        Ok(Some(identity))
    } else {
        Err(denied("stage vanished"))
    }
}
fn read_plan(originals: &Originals, id: &str, expected: &str) -> io::Result<Plan> {
    // This parses the immutable plan lineage only. Every Pending admission
    // finishes with load(), whose one fresh four-frame prefix validates the
    // Original and all independent archives. Do not validate Original twice
    // within that same load boundary.
    read_plan_depth(originals, id, expected, 0)
}
fn read_plan_depth(
    originals: &Originals,
    id: &str,
    expected: &str,
    depth: usize,
) -> io::Result<Plan> {
    if depth > 16 {
        return Err(denied("restore plan lineage exceeds bound"));
    }
    if uuid::Uuid::parse_str(id)
        .map(|x| x.to_string() != id)
        .unwrap_or(true)
    {
        return Err(denied("invalid original restore plan identity"));
    }
    let file = capture_file(
        &originals.directory,
        &format!("restore-{id}.json"),
        MAX_FRAME,
    )?;
    let Image::File {
        protection, bytes, ..
    } = file
    else {
        return Err(denied("restore plan missing"));
    };
    require_private(&protection, false)?;
    if digest(&bytes) != expected {
        return Err(denied("independent restore plan digest differs"));
    }
    let value = parse_strict_json_v2(&bytes).map_err(|_| denied("invalid restore plan JSON"))?;
    if value["schemaVersion"] == 2 {
        let link: RearmedPlan =
            serde_json::from_value(value).map_err(|_| denied("invalid rearmed plan fields"))?;
        if link.schema_version != 2 || link.id != id || link.attempt != originals.attempt {
            return Err(denied("rearmed plan belongs to another attempt"));
        }
        let mut parent =
            read_plan_depth(originals, &link.parent.id, &link.parent.sha256, depth + 1)?;
        let Image::File {
            bytes, protection, ..
        } = capture_file(
            &originals.directory,
            &format!("restore-{}-outcome.json", link.parent.id),
            MAX_FRAME,
        )?
        else {
            return Err(denied("rearmed parent outcome absent"));
        };
        require_private(&protection, false)?;
        if bytes
            != encode(
                &serde_json::json!({"schemaVersion":1,"planSha256":link.parent.sha256,"decision":"rollback"}),
            )?
        {
            return Err(denied("rearmed parent is not immutable rollback"));
        }
        parent.id = id.into();
        parent.rearmed = true;
        return Ok(parent);
    }
    let mut plan: Plan =
        serde_json::from_value(value).map_err(|_| denied("invalid restore plan fields"))?;
    if !matches!(
        (
            plan.schema_version,
            plan.journal_stages,
            plan.recovery_observation.is_some(),
            plan.empty_config.is_some(),
            plan.grok_backup.is_some()
        ),
        (1, false, false, false, false)
            | (3, true, false, false, false)
            | (4, true, true, false, false)
            | (5, true, _, true, false)
            | (6, true, _, _, true)
    ) || plan.id != id
        || plan.attempt != originals.attempt
    {
        return Err(denied("restore plan belongs to another attempt"));
    }
    plan.object_id = id.into();
    Ok(plan)
}

pub struct Pending<'a> {
    originals: &'a Originals,
    id: String,
    sha256: String,
    object_id: String,
    journal_stages: bool,
    empty_config: bool,
    grok_root: Option<PinnedDirectory>,
}
/// Read-only assessment; this never grants replay authority.
pub(super) struct RecoveryReview {
    pub terminal: Option<Decision>,
    pub fingerprint: String,
}
impl<'a> Pending<'a> {
    /// Read-only compensation decision over all four independently registered
    /// frames. No current bytes become a replacement Ready/Stopped baseline.
    pub(super) fn preparation_fingerprint(
        originals: &Originals,
        evidence: &Evidence,
    ) -> io::Result<String> {
        let (captures, phases, history) = Self::captures(originals)?;
        let decisions = kinds(&captures, evidence)?;
        let empty_config = history.config.capture(&originals.client)?;
        let grok_backup = history.grok.capture(&originals.profile)?;
        let current = Capture::read(&originals.client)?;
        if current != captures[3] {
            return Err(denied(
                "stopped postimage changed before missing-plan preparation",
            ));
        }
        // not_after is freshly checked by the assessor, not a source fact to
        // freeze across UI time. Every audit entry must still pass bounds.
        Ok(digest(&encode(&(
            phases,
            decisions,
            empty_config,
            grok_backup,
            current,
            &evidence.declared_routes,
            &evidence.retained_pids,
            &evidence.audit_home,
            evidence.not_before,
        ))?))
    }
    #[cfg(test)]
    pub(super) fn prepare_fixture(
        originals: &'a Originals,
        evidence: Evidence,
    ) -> io::Result<Self> {
        Self::prepare_layout(originals, evidence, true, &|| Ok(()), &|_| Ok(()))
    }
    #[cfg(test)]
    pub(super) fn prepare_observed_fixture(
        originals: &'a Originals,
        head: &session_index::Head,
        evidence: Evidence,
    ) -> io::Result<Self> {
        let observation = Self::observe_current(originals, head)?;
        observation
            .tree
            .archive(&originals.client, &originals.directory)?;
        observation
            .grok
            .archive(&originals.profile, &originals.directory)?;
        Self::prepare_observation_layout(
            originals,
            evidence,
            true,
            Some(observation),
            &|| Ok(()),
            &|_| Ok(()),
        )
    }
    #[cfg(test)]
    pub(super) fn apply_fixture(
        &self,
        decision: Decision,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<()> {
        self.apply_checked(decision, &|| Ok(()), checkpoint)
    }
    /// A check then rename is not kernel CAS. On a detected race, preserve the
    /// complete actual pair in an immutable conflict receipt BEFORE attempting
    /// one reverse move. A reverse operation never grants a successful restore
    /// result. A further race remains pending; no retry learns a new baseline.
    fn preserve_publication_conflict(
        &self,
        plan: &Plan,
        c: &[Capture; 4],
        i: usize,
        wanted_after: bool,
        check: &impl Fn() -> io::Result<()>,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<()> {
        let actual = self.pair(i)?;
        let expected = Self::expected(plan, c, i, wanted_after);
        let receipt = encode(
            &serde_json::json!({"schemaVersion":1,"planSha256":self.sha256,
            "artifact":i,"wantedAfter":wanted_after,"target":actual.0,"stage":actual.1}),
        )?;
        create(
            &self.originals.directory,
            &format!("restore-{}-conflict-{}.json", self.id, uuid::Uuid::new_v4()),
            &receipt,
        )?;
        check()?;
        checkpoint("compensation_gap")?;
        if self.pair(i)? != actual {
            return Err(denied(
                "late conflict changed again; all observed objects retained",
            ));
        }
        let stage = format!(".ocxd-restore-{}-{i}", self.object_id);
        match plan.kind(i) {
            // Only our known original was installed into target. The other
            // complete regular object can be returned by another lossless swap.
            Kind::Swap
                if same(&actual.0, &expected.0, false)
                    && matches!(actual.1, Image::File { .. }) =>
            {
                rename(
                    self.staging(),
                    &stage,
                    self.target(i)?,
                    target_name(i)?,
                    libc::RENAME_SWAP,
                )?;
            }
            // A vanished original means target is now absent. EXCL guarantees
            // a newly created target cannot be overwritten by compensation.
            Kind::Remove
                if wanted_after
                    && actual.0 == Image::Absent
                    && matches!(actual.1, Image::File { .. }) =>
            {
                rename(
                    self.staging(),
                    &stage,
                    self.target(i)?,
                    target_name(i)?,
                    libc::RENAME_EXCL,
                )?;
            }
            Kind::Publish
                if !wanted_after
                    && actual.0 == Image::Absent
                    && matches!(actual.1, Image::File { .. }) =>
            {
                rename(
                    self.staging(),
                    &stage,
                    self.target(i)?,
                    target_name(i)?,
                    libc::RENAME_EXCL,
                )?;
            }
            _ => {
                return Err(denied(
                    "publication conflict cannot be reversed without guessing",
                ))
            }
        }
        let returned = self.pair(i)?;
        if !same(&returned.0, &actual.1, false) || !same(&returned.1, &actual.0, false) {
            return Err(denied(
                "reverse publication changed; no further mutation or success",
            ));
        }
        Err(denied(
            "late external object returned; original operation remains failed",
        ))
    }
    fn preserve_archive_conflict(
        &self,
        i: usize,
        archive_name: &str,
        check: &impl Fn() -> io::Result<()>,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<()> {
        let archived = capture_file(&self.originals.directory, archive_name, MAX_CONTENT)?;
        let stage_name = format!(".ocxd-restore-{}-{i}", self.object_id);
        let staged = capture_file(self.staging(), &stage_name, MAX_CONTENT)?;
        let receipt = encode(
            &serde_json::json!({"schemaVersion":1,"planSha256":self.sha256,
            "artifact":i,"archive":archived,"stage":staged}),
        )?;
        create(
            &self.originals.directory,
            &format!("restore-{}-conflict-{}.json", self.id, uuid::Uuid::new_v4()),
            &receipt,
        )?;
        if staged != Image::Absent || !matches!(archived, Image::File { .. }) {
            return Err(denied(
                "archive conflict retained; no safe vacant return path",
            ));
        }
        check()?;
        checkpoint("compensation_gap")?;
        if capture_file(&self.originals.directory, archive_name, MAX_CONTENT)? != archived
            || capture_file(self.staging(), &stage_name, MAX_CONTENT)? != Image::Absent
        {
            return Err(denied("archive conflict changed again; retained"));
        }
        rename(
            &self.originals.directory,
            archive_name,
            self.staging(),
            &stage_name,
            libc::RENAME_EXCL,
        )?;
        if !same(
            &capture_file(self.staging(), &stage_name, MAX_CONTENT)?,
            &archived,
            false,
        ) {
            return Err(denied("archive return changed; retained without retry"));
        }
        Err(denied(
            "late archive source returned to stage; transaction remains failed",
        ))
    }
    fn staging(&self) -> &PinnedDirectory {
        if self.journal_stages {
            &self.originals.directory
        } else {
            &self.originals.client
        }
    }
    pub(super) fn stages_in_journal(&self) -> bool {
        self.journal_stages
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    fn grok_root_for(originals: &Originals, plan: &Plan) -> io::Result<Option<PinnedDirectory>> {
        plan.grok_backup
            .as_ref()
            .map(|backup| {
                originals.profile.recheck()?;
                let root = PinnedDirectory::open(&originals.profile.path().join(".grok"))?;
                if root.identity() != &backup.root
                    || root.identity().device != originals.directory.identity().device
                {
                    return Err(denied("backup root changed or cross-volume"));
                }
                Ok(root)
            })
            .transpose()
    }
    fn target(&self, i: usize) -> io::Result<&PinnedDirectory> {
        if i == 4 {
            let root = self
                .grok_root
                .as_ref()
                .ok_or_else(|| denied("backup target unavailable"))?;
            self.originals.profile.recheck()?;
            root.recheck()?;
            Ok(root)
        } else {
            Ok(&self.originals.client)
        }
    }
    pub(super) fn grok_remainder_matches(
        &self,
        original: &GrokSnapshot,
        current: &GrokSnapshot,
    ) -> bool {
        if self.grok_root.is_some() {
            original.backup_remainder_matches(current)
        } else {
            original.restored_matches(current)
        }
    }
    pub(super) fn grok_preplan_matches(
        originals: &Originals,
        head: &session_index::Head,
        original: &GrokSnapshot,
        current: &GrokSnapshot,
    ) -> io::Result<bool> {
        let history = if head.state == session_index::State::Stopped {
            Self::captures(originals)?.2.grok
        } else {
            let missing = head.state == session_index::State::Ready;
            if !missing && head.state != session_index::State::StopPrepared {
                return Ok(false);
            }
            let frames = originals.frames_through(if missing {
                Phase::Ready
            } else {
                Phase::PreStop
            })?;
            GrokHistory::new(
                &frames[0].0.grok,
                &frames[1].0.grok,
                if missing {
                    None
                } else {
                    Some(&frames[2].0.grok)
                },
                current,
            )?
        };
        Ok(history.capture(&originals.profile)?.is_some()
            && original.backup_remainder_matches(current))
    }
    pub(super) fn object_id(&self) -> &str {
        &self.object_id
    }
    /// Called only on a Pending whose immutable plan/source was admitted by
    /// open()/load(). This is a content projection, not native permission.
    pub(super) fn remainder_matches(
        &self,
        original: &TreeSnapshot,
        current: &TreeSnapshot,
        leased: bool,
    ) -> bool {
        if self.empty_config {
            original.remainder_with_empty_config(current, leased)
        } else if self.stages_in_journal() {
            original.preplan_remainder_matches(current, leased)
        } else {
            original.recovery_remainder_matches(current, self.object_id(), leased)
        }
    }
    pub(super) fn empty_config_preplan_matches(
        originals: &Originals,
        head: &session_index::Head,
        original: &TreeSnapshot,
        current: &TreeSnapshot,
        leased: bool,
    ) -> io::Result<bool> {
        if original.recorded_file("config.toml")?.is_some()
            || current.recorded_file("config.toml")?.is_none()
        {
            return Ok(false);
        }
        let config = if head.state == session_index::State::Stopped {
            Self::captures(originals)?.2.config
        } else {
            let missing_pre = head.state == session_index::State::Ready;
            if !missing_pre && head.state != session_index::State::StopPrepared {
                return Ok(false);
            }
            let frames = originals.frames_through(if missing_pre {
                Phase::Ready
            } else {
                Phase::PreStop
            })?;
            ConfigHistory::new(
                &frames[0].0.tree,
                &frames[1].0.tree,
                if missing_pre {
                    None
                } else {
                    Some(&frames[2].0.tree)
                },
                current,
            )?
        };
        Ok(config.capture(&originals.client)?.is_some()
            && original.remainder_with_empty_config(current, leased))
    }
    pub(super) fn readback_name(&self, decision: Decision) -> String {
        let base = if decision == Decision::Complete {
            "completion-readback"
        } else {
            "rollback-readback"
        };
        if self.id == self.object_id {
            format!("{base}.json")
        } else {
            format!("{base}-{}.json", self.id)
        }
    }
    pub(super) fn require_parent(&self, parent: &session_index::RestorePlan) -> io::Result<()> {
        let Image::File { bytes, .. } = capture_file(
            &self.originals.directory,
            &format!("restore-{}.json", self.id),
            MAX_FRAME,
        )?
        else {
            return Err(denied("rearmed plan missing"));
        };
        let link: RearmedPlan = serde_json::from_value(
            parse_strict_json_v2(&bytes).map_err(|_| denied("rearmed JSON invalid"))?,
        )
        .map_err(|_| denied("rearmed link invalid"))?;
        if digest(&bytes) != self.sha256 || link.parent != *parent {
            return Err(denied("rearmed plan parent differs"));
        }
        self.load()?;
        Ok(())
    }
    /// A NEW logical decision over the SAME exact native objects. No stage is
    /// copied, moved or learned here; a crash before index publication leaves
    /// only an unreferenced immutable plan, never an unindexed client mutation.
    pub(super) fn rearm_rolled_back(&self) -> io::Result<Self> {
        if self.review_recovery()?.terminal != Some(Decision::Rollback) {
            return Err(denied("only exact rolled-back objects may be rearmed"));
        }
        let id = uuid::Uuid::new_v4().to_string();
        let bytes = encode(&RearmedPlan {
            schema_version: 2,
            id: id.clone(),
            attempt: self.originals.attempt.clone(),
            parent: session_index::RestorePlan {
                id: self.id.clone(),
                sha256: self.sha256.clone(),
            },
        })?;
        create(
            &self.originals.directory,
            &format!("restore-{id}.json"),
            &bytes,
        )?;
        let result = Self::open(self.originals, &id, &digest(&bytes))?;
        if result.review_recovery()?.terminal.is_some() {
            return Err(denied("rearmed plan already has outcome"));
        }
        Ok(result)
    }
    pub fn open(originals: &'a Originals, id: &str, sha256: &str) -> io::Result<Self> {
        let plan = read_plan(originals, id, sha256)?;
        let result = Self {
            originals,
            id: id.into(),
            sha256: sha256.into(),
            object_id: plan.object_id.clone(),
            journal_stages: plan.journal_stages,
            empty_config: plan.empty_config.is_some(),
            grok_root: Self::grok_root_for(originals, &plan)?,
        };
        result.load()?;
        Ok(result)
    }
    /// Creates only a plan and known original stages. No source artifact is
    /// replaced until apply has an independent native lifecycle permit.
    pub fn prepare(
        originals: &'a Originals,
        evidence: Evidence,
        lease: &crate::infrastructure::official_client_coordination::native::NativeLease,
    ) -> io::Result<Self> {
        Self::prepare_layout(
            originals,
            evidence,
            true,
            &|| {
                lease
                    .check()
                    .map_err(|_| denied("official coordination expired"))
            },
            &|point| {
                #[cfg(test)]
                if std::env::var("OCXD024_PLAN_PREPARE_INTERRUPT")
                    .ok()
                    .as_deref()
                    == Some(point)
                {
                    if !originals.data.path.starts_with("/private/tmp")
                        || !originals.data.path.ancestors().any(|p| {
                            p.file_name().is_some_and(|n| {
                                n.to_string_lossy()
                                    .starts_with("ocxd024-versioned-lifecycle-")
                            })
                        })
                    {
                        return Err(denied("plan interruption fixture refused"));
                    }
                    return Err(denied("owned journal staging interruption"));
                }
                let _ = point;
                Ok(())
            },
        )
    }
    fn observe_current(
        originals: &Originals,
        head: &session_index::Head,
    ) -> io::Result<RecoveryObservation> {
        session_index::validate(head)?;
        if !matches!(
            head.state,
            session_index::State::Ready | session_index::State::StopPrepared
        ) || head.restore_plan.is_some()
            || head.completion_sha256.is_some()
        {
            return Err(denied(
                "current observation requires interrupted StopPrepared",
            ));
        }
        let refs = head
            .references
            .as_ref()
            .ok_or_else(|| denied("recovery original references absent"))?;
        let known = originals
            .known
            .lock()
            .map_err(|_| denied("original references unavailable"))?;
        if refs.attempt != originals.attempt
            || refs.original_sha256 != originals.original_sha256
            || refs.phases.len()
                != if head.state == session_index::State::Ready {
                    1
                } else {
                    2
                }
            || refs
                .phases
                .iter()
                .any(|(phase, hash)| known[phase.index()].as_ref() != Some(hash))
            || refs.live_sha256.is_none()
        {
            return Err(denied("recovery observation references differ"));
        }
        drop(known);
        Ok(RecoveryObservation {
            schema_version: 1,
            source_head: head.clone(),
            observed_at: chrono::Utc::now(),
            capture: Capture::read(&originals.client)?,
            tree: TreeSnapshot::read(&originals.client)?,
            grok: GrokSnapshot::read(&originals.profile)?,
        })
    }
    fn captures_with_observation(
        originals: &Originals,
        observation: Option<&RecoveryObservation>,
    ) -> io::Result<([Capture; 4], Vec<String>, Histories)> {
        let Some(observation) = observation else {
            let (c, h, config) = Self::captures(originals)?;
            return Ok((c, h.to_vec(), config));
        };
        session_index::validate(&observation.source_head)?;
        let refs = observation
            .source_head
            .references
            .as_ref()
            .ok_or_else(|| denied("observation source absent"))?;
        if observation.schema_version != 1
            || !matches!(
                observation.source_head.state,
                session_index::State::Ready | session_index::State::StopPrepared
            )
            || observation.source_head.restore_plan.is_some()
            || refs.attempt != originals.attempt
            || refs.original_sha256 != originals.original_sha256
            || refs.phases.len()
                != if observation.source_head.state == session_index::State::Ready {
                    1
                } else {
                    2
                }
            || refs.live_sha256.is_none()
            || !observation.capture.valid()
            || !observation.tree.valid()
            || !observation.tree.coordinated()
            || observation.capture.root != *originals.client.identity()
            || observation.tree.root != *originals.client.identity()
            || !observation.tree.matches_artifacts(&observation.capture)
            || observation.tree.root_protection != observation.capture.root_protection
        {
            return Err(denied(
                "current recovery observation is not bound to original context",
            ));
        }
        let frames = originals.frames_through(if refs.phases.len() == 1 {
            Phase::Ready
        } else {
            Phase::PreStop
        })?;
        for (index, (phase, hash)) in refs.phases.iter().enumerate() {
            if frames[index + 1].0.phase != *phase || digest(&frames[index + 1].1) != *hash {
                return Err(denied("observation original prefix changed"));
            }
        }
        let config = ConfigHistory::new(
            &frames[0].0.tree,
            &frames[1].0.tree,
            if refs.phases.len() == 2 {
                Some(&frames[2].0.tree)
            } else {
                None
            },
            &observation.tree,
        )?;
        let grok = GrokHistory::new(
            &frames[0].0.grok,
            &frames[1].0.grok,
            if refs.phases.len() == 2 {
                Some(&frames[2].0.grok)
            } else {
                None
            },
            &observation.grok,
        )?;
        observation.tree.verify_archive(&originals.directory)?;
        observation.grok.verify_archive(&originals.directory)?;
        let mut captures = frames
            .iter()
            .map(|(f, _)| f.capture.clone())
            .collect::<Vec<_>>();
        // Slot2 is unused by the fresh current-content classifier when no
        // PreStop exists. It is never hashed or serialized as a historical frame.
        if refs.phases.len() == 1 {
            captures.push(frames[1].0.capture.clone());
        }
        captures.push(observation.capture.clone());
        let mut hashes = frames.iter().map(|(_, b)| digest(b)).collect::<Vec<_>>();
        hashes.push(digest(&encode(observation)?));
        Ok((
            captures
                .try_into()
                .map_err(|_| denied("original prefix incomplete"))?,
            hashes,
            Histories { config, grok },
        ))
    }
    pub(super) fn observed_preparation_fingerprint(
        originals: &Originals,
        head: &session_index::Head,
        evidence: &Evidence,
    ) -> io::Result<String> {
        let observation = Self::observe_current(originals, head)?;
        // Read-only inspection does not require an archive for new current
        // bytes yet. Assess from the original3-frame prefix and current image.
        Self::observation_fingerprint(originals, head, evidence, &observation)
    }
    fn observation_fingerprint(
        originals: &Originals,
        head: &session_index::Head,
        evidence: &Evidence,
        observation: &RecoveryObservation,
    ) -> io::Result<String> {
        let missing_pre = head.state == session_index::State::Ready;
        let frames = originals.frames_through(if missing_pre {
            Phase::Ready
        } else {
            Phase::PreStop
        })?;
        let c = [
            frames[0].0.capture.clone(),
            frames[1].0.capture.clone(),
            if missing_pre {
                frames[1].0.capture.clone()
            } else {
                frames[2].0.capture.clone()
            },
            observation.capture.clone(),
        ];
        let config = ConfigHistory::new(
            &frames[0].0.tree,
            &frames[1].0.tree,
            if missing_pre {
                None
            } else {
                Some(&frames[2].0.tree)
            },
            &observation.tree,
        )?;
        let empty_config = config.capture(&originals.client)?;
        let grok = GrokHistory::new(
            &frames[0].0.grok,
            &frames[1].0.grok,
            if missing_pre {
                None
            } else {
                Some(&frames[2].0.grok)
            },
            &observation.grok,
        )?;
        let grok_backup = grok.capture(&originals.profile)?;
        let decisions = if missing_pre {
            kinds_current(&c, evidence)?
        } else {
            kinds(&c, evidence)?
        };
        Ok(digest(&encode(&(
            head,
            &frames.iter().map(|(_, b)| digest(b)).collect::<Vec<_>>(),
            decisions,
            empty_config,
            grok_backup,
            c[3].clone(),
            &evidence.declared_routes,
            &evidence.retained_pids,
            &evidence.audit_home,
            evidence.not_before,
        ))?))
    }
    pub(super) fn prepared_observation_confirmation(
        &self,
        head: &session_index::Head,
    ) -> io::Result<String> {
        let (plan, _) = self.load()?;
        let observation = plan
            .recovery_observation
            .as_ref()
            .ok_or_else(|| denied("current recovery observation absent"))?;
        if observation.source_head != *head {
            return Err(denied("prepared observation source head changed"));
        }
        let original = self.originals.frame(Phase::Original)?.0;
        if !self.remainder_matches(&original.tree, &observation.tree, true)
            || !self.grok_remainder_matches(&original.grok, &observation.grok)
        {
            return Err(denied(
                "prepared observation contains changed unowned client content",
            ));
        }
        let pre = self
            .originals
            .frame(if head.state == session_index::State::Ready {
                Phase::Ready
            } else {
                Phase::PreStop
            })?
            .0;
        for name in [
            ".opencodex-native-main.claim.sqlite",
            ".opencodex-native-main.owner.sqlite",
        ] {
            let id = observation.tree.stable_identity(name)?;
            if pre.tree.stable_identity(name)? != id
                || !original.tree.original_stable_identity_matches(name, &id)
            {
                return Err(denied("prepared current observation rebound stable locks"));
            }
        }
        let decision =
            Self::observation_fingerprint(self.originals, head, &plan.evidence, observation)?;
        Ok(digest(&encode(&(
            head,
            decision,
            self.originals.data.identity(),
            self.originals.client.identity(),
            self.originals.profile.identity(),
        ))?))
    }
    pub(super) fn prepare_observed(
        originals: &'a Originals,
        head: &session_index::Head,
        evidence: Evidence,
        lease: &crate::infrastructure::official_client_coordination::native::NativeLease,
    ) -> io::Result<Self> {
        let observation = Self::observe_current(originals, head)?;
        observation
            .tree
            .archive(&originals.client, &originals.directory)?;
        observation
            .grok
            .archive(&originals.profile, &originals.directory)?;
        if Capture::read(&originals.client)? != observation.capture
            || TreeSnapshot::read(&originals.client)? != observation.tree
            || GrokSnapshot::read(&originals.profile)? != observation.grok
        {
            return Err(denied("current recovery observation changed before plan"));
        }
        Self::prepare_observation_layout(
            originals,
            evidence,
            true,
            Some(observation),
            &|| {
                lease
                    .check()
                    .map_err(|_| denied("current recovery writer lease expired"))
            },
            &|_| Ok(()),
        )
    }
    pub(super) fn stopped_tree(&self) -> io::Result<TreeSnapshot> {
        let (plan, _) = self.load()?;
        match plan.recovery_observation {
            Some(observation) => Ok(observation.tree),
            None => Ok(self.originals.frame(Phase::Stopped)?.0.tree),
        }
    }
    pub(super) fn require_indexed_references(&self, refs: &session::References) -> io::Result<()> {
        let (plan, _) = self.load()?;
        if let Some(observation) = plan.recovery_observation {
            if observation.source_head.references.as_ref() != Some(refs) {
                return Err(denied("recovery plan rebound source references"));
            }
        } else if refs.phases.len() != 3 {
            return Err(denied("normal plan requires complete historical prefix"));
        }
        Ok(())
    }
    pub(super) fn require_recovery_source(&self, head: &session_index::Head) -> io::Result<()> {
        let (plan, _) = self.load()?;
        if plan
            .recovery_observation
            .as_ref()
            .is_none_or(|o| o.source_head != *head)
        {
            return Err(denied(
                "recovery observation does not bind original interrupted head",
            ));
        }
        Ok(())
    }
    fn captures(originals: &Originals) -> io::Result<([Capture; 4], [String; 4], Histories)> {
        let frames = originals.frames_through(Phase::Stopped)?;
        let config = ConfigHistory::new(
            &frames[0].0.tree,
            &frames[1].0.tree,
            Some(&frames[2].0.tree),
            &frames[3].0.tree,
        )?;
        let grok = GrokHistory::new(
            &frames[0].0.grok,
            &frames[1].0.grok,
            Some(&frames[2].0.grok),
            &frames[3].0.grok,
        )?;
        let mut captures = Vec::new();
        let mut hashes = Vec::new();
        for (frame, raw) in frames {
            captures.push(frame.capture);
            hashes.push(digest(&raw));
        }
        Ok((
            captures
                .try_into()
                .map_err(|_| denied("phase set incomplete"))?,
            hashes
                .try_into()
                .map_err(|_| denied("phase digests incomplete"))?,
            Histories { config, grok },
        ))
    }
    #[cfg(test)]
    fn prepare_checked(
        originals: &'a Originals,
        evidence: Evidence,
        check: &impl Fn() -> io::Result<()>,
    ) -> io::Result<Self> {
        Self::prepare_layout(originals, evidence, false, check, &|_| Ok(()))
    }
    fn prepare_layout(
        originals: &'a Originals,
        evidence: Evidence,
        journal_stages: bool,
        check: &impl Fn() -> io::Result<()>,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<Self> {
        Self::prepare_observation_layout(
            originals,
            evidence,
            journal_stages,
            None,
            check,
            checkpoint,
        )
    }
    fn prepare_observation_layout(
        originals: &'a Originals,
        evidence: Evidence,
        journal_stages: bool,
        recovery_observation: Option<RecoveryObservation>,
        check: &impl Fn() -> io::Result<()>,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<Self> {
        check()?;
        if journal_stages
            && originals.directory.identity().device != originals.client.identity().device
        {
            return Err(denied(
                "restore staging and client are on different volumes",
            ));
        }
        let (captures, phases, history) =
            Self::captures_with_observation(originals, recovery_observation.as_ref())?;
        let empty_config = history.config.capture(&originals.client)?;
        let grok_backup = history.grok.capture(&originals.profile)?;
        if (empty_config.is_some() || grok_backup.is_some()) && !journal_stages {
            return Err(denied("empty config requires private journal stages"));
        }
        let ks = if recovery_observation
            .as_ref()
            .is_some_and(|o| o.source_head.state == session_index::State::Ready)
        {
            kinds_current(&captures, &evidence)?
        } else {
            kinds(&captures, &evidence)?
        };
        if Capture::read(&originals.client)? != captures[3] {
            return Err(denied("stopped postimage changed; no new baseline"));
        }
        let id = uuid::Uuid::new_v4().to_string();
        create(
            &originals.directory,
            &format!("restore-{id}-preparing.json"),
            &encode(
                &serde_json::json!({"schemaVersion":1,"attempt":originals.attempt,"id":id,"phases":phases,"kinds":ks}),
            )?,
        )?;
        checkpoint("preparing")?;
        let staging = if journal_stages {
            &originals.directory
        } else {
            &originals.client
        };
        let mut staged = std::array::from_fn(|_| None);
        // Every target is checked before staging. Failed staging is retained,
        // never cleaned by guessed filename or used as a replacement original.
        for i in 0..3 {
            check()?;
            if matches!(ks[i], Kind::Swap | Kind::Publish) {
                staged[i] = stage(
                    staging,
                    &format!(".ocxd-restore-{id}-{i}"),
                    &captures[0].images[i],
                    checkpoint,
                )?;
            }
            checkpoint("staged")?;
        }
        if Capture::read(&originals.client)? != captures[3]
            || history.config.capture(&originals.client)? != empty_config
            || history.grok.capture(&originals.profile)? != grok_backup
        {
            return Err(denied("source changed while preparing original stages"));
        }
        check()?;
        let plan = Plan {
            object_id: id.clone(),
            rearmed: false,
            schema_version: if grok_backup.is_some() {
                6
            } else if empty_config.is_some() {
                5
            } else if recovery_observation.is_some() {
                4
            } else if journal_stages {
                3
            } else {
                1
            },
            journal_stages,
            recovery_observation,
            empty_config,
            grok_backup,
            id: id.clone(),
            attempt: originals.attempt.clone(),
            phases,
            kinds: ks,
            staged,
            evidence,
        };
        let bytes = encode(&plan)?;
        create(&originals.directory, &format!("restore-{id}.json"), &bytes)?;
        checkpoint("plan_published")?;
        let result = Self {
            originals,
            object_id: id.clone(),
            journal_stages,
            empty_config: plan.empty_config.is_some(),
            grok_root: Self::grok_root_for(originals, &plan)?,
            id,
            sha256: digest(&bytes),
        };
        result.load()?;
        Ok(result)
    }
    fn load(&self) -> io::Result<(Plan, [Capture; 4])> {
        let plan = read_plan(self.originals, &self.id, &self.sha256)?;
        if plan.object_id != self.object_id
            || plan.journal_stages != self.journal_stages
            || plan.empty_config.is_some() != self.empty_config
        {
            return Err(denied("restore object lineage changed"));
        }
        let (captures, hashes, history) =
            Self::captures_with_observation(self.originals, plan.recovery_observation.as_ref())?;
        history.config.validate(plan.empty_config.as_ref())?;
        history.grok.validate(plan.grok_backup.as_ref())?;
        if self.grok_root.is_some() != plan.grok_backup.is_some() {
            return Err(denied("backup plan scope changed"));
        }
        if let Some(root) = &self.grok_root {
            root.recheck()?;
            if plan
                .grok_backup
                .as_ref()
                .is_none_or(|b| root.identity() != &b.root)
            {
                return Err(denied("backup root identity changed"));
            }
        }
        let current = plan
            .recovery_observation
            .as_ref()
            .is_some_and(|o| o.source_head.state == session_index::State::Ready);
        if plan.phases != hashes
            || plan.kinds
                != if current {
                    kinds_current(&captures, &plan.evidence)?
                } else {
                    kinds(&captures, &plan.evidence)?
                }
        {
            return Err(denied("original phases or restore decisions changed"));
        }
        for i in 0..3 {
            if matches!(plan.kinds[i], Kind::Swap | Kind::Publish) != plan.staged[i].is_some() {
                return Err(denied("staged identity missing or unexpected"));
            }
        }
        Ok((plan, captures))
    }
    /// One current normal-Stop transaction may keep its Codex writer through
    /// artifact restoration and whole-root retirement. The session owner must
    /// bind the supplied Guard to its lease; this method cannot issue release
    /// or root authority. Grok still holds/releases its own separate writer.
    pub(super) fn apply_retaining_codex(
        &self,
        permit: &MutationPermit<'_>,
        codex: &mut crate::infrastructure::official_client_coordination::native::config::Guard<'_>,
    ) -> io::Result<()> {
        permit.check(self.originals)?;
        codex
            .check()
            .map_err(|e| writer_refusal(e, "codex-before", "retained Codex writer unconfirmed"))?;
        let grok = if self.grok_root.is_some() {
            Some(std::cell::RefCell::new(permit.grok_write_guard()?))
        } else {
            None
        };
        let codex = std::cell::RefCell::new(codex);
        let result = self.apply_checked(
            Decision::Complete,
            &|| {
                permit.check(self.originals)?;
                codex.borrow_mut().check().map_err(|e| {
                    writer_refusal(e, "codex-current", "retained Codex writer changed")
                })?;
                if let Some(grok) = &grok {
                    grok.borrow_mut()
                        .check()
                        .map_err(|e| writer_refusal(e, "grok-current", "Grok writer changed"))?;
                }
                Ok(())
            },
            &|_| Ok(()),
        );
        let released = if let Some(grok) = grok {
            grok.into_inner()
                .finish()
                .map_err(|e| writer_refusal(e, "grok-release", "Grok writer release unconfirmed"))
        } else {
            Ok(())
        };
        result?;
        released?;
        permit.check(self.originals)?;
        let current = codex.borrow_mut().check().map_err(|e| {
            writer_refusal(
                e,
                "codex-after",
                "retained Codex writer unconfirmed after restore",
            )
        });
        current
    }
    pub fn apply(&self, decision: Decision, permit: &MutationPermit<'_>) -> io::Result<()> {
        if self.empty_config || self.grok_root.is_some() {
            let codex = if self.empty_config {
                Some(std::cell::RefCell::new(permit.config_write_guard()?))
            } else {
                None
            };
            let grok = if self.grok_root.is_some() {
                Some(std::cell::RefCell::new(permit.grok_write_guard()?))
            } else {
                None
            };
            let result = self.apply_checked(
                decision,
                &|| {
                    permit.check(self.originals)?;
                    for guard in [codex.as_ref(), grok.as_ref()].into_iter().flatten() {
                        guard.borrow_mut().check().map_err(|_| {
                            denied("official configuration writer exclusion changed")
                        })?;
                    }
                    Ok(())
                },
                &|_| Ok(()),
            );
            let mut released = Ok(());
            for guard in [grok, codex].into_iter().flatten() {
                if guard.into_inner().finish().is_err() {
                    released = Err(denied("official configuration writer release unconfirmed"));
                }
            }
            result?;
            released?;
            return Ok(());
        }
        #[cfg(test)]
        if std::env::var_os("OCXD024_INTERRUPT_ARTIFACT").as_deref() == Some(OsStr::new("1")) {
            if !self.originals.data.path.starts_with("/private/tmp")
                || !self.originals.data.path.ancestors().any(|p| {
                    p.file_name().is_some_and(|n| {
                        n.to_string_lossy()
                            .starts_with("ocxd024-versioned-lifecycle-")
                    })
                })
            {
                return Err(denied("interruption fixture root refused"));
            }
            return self.apply_checked(decision, &|| permit.check(self.originals), &|point| {
                if point == "after_native" {
                    Err(denied(
                        "owned fixture interrupted after first native publication",
                    ))
                } else {
                    Ok(())
                }
            });
        }
        self.apply_checked(decision, &|| permit.check(self.originals), &|_| Ok(()))
    }
    /// Verify the entire batch and its journals without learning new objects.
    /// Known partial states are evidence, never proof of writer exclusion.
    pub(super) fn review_recovery(&self) -> io::Result<RecoveryReview> {
        let (plan, captures) = self.load()?;
        let journal = |suffix: &str| {
            capture_file(
                &self.originals.directory,
                &format!("restore-{}-{suffix}.json", self.id),
                MAX_FRAME,
            )
        };
        let intent = journal("intent")?;
        let expected_intent =
            encode(&serde_json::json!({"schemaVersion":1,"planSha256":self.sha256}))?;
        match &intent {
            Image::Absent => {}
            Image::File {
                bytes, protection, ..
            } if *bytes == expected_intent => {
                require_private(protection, false)?;
            }
            _ => return Err(denied("recovery intent changed; no replacement baseline")),
        }
        let terminal_image = journal("outcome")?;
        let terminal = match &terminal_image {
            Image::Absent => None,
            Image::File {
                bytes, protection, ..
            } => {
                require_private(protection, false)?;
                let mut found = None;
                for decision in [Decision::Complete, Decision::Rollback] {
                    if *bytes
                        == encode(
                            &serde_json::json!({"schemaVersion":1,"planSha256":self.sha256,"decision":decision}),
                        )?
                    {
                        found = Some(decision);
                    }
                }
                Some(found.ok_or_else(|| denied("recovery terminal changed"))?)
            }
        };
        let mut observed = Vec::new();
        for i in plan.indices() {
            let pair = self.pair(i)?;
            let archive = self.archive(i)?;
            let after = Self::state_of(&plan, &captures, i, &pair, &archive)?;
            if intent == Image::Absent
                && (terminal.is_some() || after || !plan.rearmed && archive != Image::Absent)
            {
                return Err(denied("published recovery objects lack original intent"));
            }
            if let Some(decision) = terminal {
                let expected = if decision == Decision::Complete {
                    &captures[0]
                } else {
                    &captures[3]
                };
                let expected_image = if i >= 3 {
                    if decision == Decision::Complete {
                        &Image::Absent
                    } else {
                        if i == 3 {
                            &plan.empty_config.as_ref().unwrap().stopped
                        } else {
                            &plan.grok_backup.as_ref().unwrap().stopped
                        }
                    }
                } else {
                    &expected.images[i]
                };
                if !original_content(&pair.0, expected_image) {
                    return Err(denied("terminal client bytes or protection differ"));
                }
                if (plan.kind(i) != Kind::Keep && after != (decision == Decision::Complete))
                    || pair.1 != Image::Absent
                {
                    return Err(denied("recovery terminal disagrees with native objects"));
                }
            }
            observed.push((i, pair, archive, after));
        }
        self.load()?;
        if journal("intent")? != intent || journal("outcome")? != terminal_image {
            return Err(denied("recovery journal changed during review"));
        }
        for (i, pair, archive, _) in &observed {
            if self.pair(*i)? != *pair || self.archive(*i)? != *archive {
                return Err(denied("recovery objects changed during review"));
            }
        }
        Ok(RecoveryReview {
            terminal,
            fingerprint: digest(&encode(&(
                &self.id,
                &self.sha256,
                intent,
                terminal_image,
                observed,
                self.originals.data.identity(),
                self.originals.client.identity(),
                self.originals.directory.identity(),
            ))?),
        })
    }
    fn archive(&self, i: usize) -> io::Result<Image> {
        capture_file(
            &self.originals.directory,
            &format!("restore-{}-old-{i}", self.object_id),
            MAX_CONTENT,
        )
    }
    fn pair(&self, i: usize) -> io::Result<(Image, Image)> {
        Ok((
            capture_file(self.target(i)?, target_name(i)?, MAX_CONTENT)?,
            capture_file(
                self.staging(),
                &format!(".ocxd-restore-{}-{i}", self.object_id),
                MAX_CONTENT,
            )?,
        ))
    }
    fn expected(plan: &Plan, c: &[Capture; 4], i: usize, after: bool) -> (Image, Image) {
        let old = if i == 4 {
            plan.grok_backup.as_ref().unwrap().stopped.clone()
        } else if i == 3 {
            plan.empty_config.as_ref().unwrap().stopped.clone()
        } else {
            c[3].images[i].clone()
        };
        let desired = if i >= 3 {
            Image::Absent
        } else {
            stat_image(&c[0].images[i], plan.staged[i].as_ref())
        };
        match (plan.kind(i), after) {
            (Kind::Keep, _) => (old, Image::Absent),
            (Kind::Swap, false) => (old, desired),
            (Kind::Swap, true) => (desired, old),
            (Kind::Remove, false) => (old, Image::Absent),
            (Kind::Remove, true) => (Image::Absent, old),
            (Kind::Publish, false) => (Image::Absent, desired),
            (Kind::Publish, true) => (desired, Image::Absent),
        }
    }
    fn state(&self, plan: &Plan, c: &[Capture; 4], i: usize) -> io::Result<bool> {
        let actual = self.pair(i)?;
        let archive = self.archive(i)?;
        Self::state_of(plan, c, i, &actual, &archive)
    }
    fn state_of(
        plan: &Plan,
        c: &[Capture; 4],
        i: usize,
        actual: &(Image, Image),
        archive: &Image,
    ) -> io::Result<bool> {
        for after in [false, true] {
            let wanted = Self::expected(plan, c, i, after);
            if same(&actual.0, &wanted.0, false)
                && same(&actual.1, &wanted.1, false)
                && *archive == Image::Absent
            {
                return Ok(after);
            }
        }
        // Already archived, completed operations are recognized only by both
        // the exact published original inode and exact original postimage inode.
        for after in [false, true] {
            let wanted = Self::expected(plan, c, i, after);
            if same(&actual.0, &wanted.0, false)
                && actual.1 == Image::Absent
                && same(archive, &wanted.1, false)
            {
                return Ok(after);
            }
        }
        Err(denied(
            "unknown target or stage generation; all originals retained",
        ))
    }
    fn apply_checked(
        &self,
        decision: Decision,
        check: &impl Fn() -> io::Result<()>,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<()> {
        check()?;
        let (plan, c) = self.load()?;
        let terminal = format!("restore-{}-outcome.json", self.id);
        let existing = capture_file(&self.originals.directory, &terminal, MAX_FRAME)?;
        if let Image::File {
            bytes, protection, ..
        } = &existing
        {
            require_private(protection, false)?;
            let expected = encode(
                &serde_json::json!({"schemaVersion":1,"planSha256":self.sha256,"decision":decision}),
            )?;
            if bytes != &expected {
                return Err(denied("terminal restore decision differs"));
            }
        }
        // Audit the whole batch before first mutation. No partial authorization
        // is created just because one catalog object happens to be eligible.
        for i in plan.indices() {
            self.state(&plan, &c, i)?;
        }
        let intent = format!("restore-{}-intent.json", self.id);
        let intent_bytes =
            encode(&serde_json::json!({"schemaVersion":1,"planSha256":self.sha256}))?;
        match capture_file(&self.originals.directory, &intent, MAX_FRAME)? {
            Image::Absent => create(&self.originals.directory, &intent, &intent_bytes)?,
            Image::File {
                bytes, protection, ..
            } if bytes == intent_bytes => {
                require_private(&protection, false)?;
            }
            _ => return Err(denied("restore intent changed")),
        }
        checkpoint("intent")?;
        for i in plan.indices() {
            let target_name = target_name(i)?;
            check()?;
            self.load()?;
            let after = self.state(&plan, &c, i)?;
            let want = decision == Decision::Complete;
            if after != want && plan.kind(i) != Kind::Keep {
                let stage = format!(".ocxd-restore-{}-{i}", self.object_id);
                // An interrupted archive can be undone only by moving back the
                // exact retained object into a still-absent original stage.
                let expected_stage = Self::expected(&plan, &c, i, after).1;
                if self.pair(i)?.1 == Image::Absent && expected_stage != Image::Absent {
                    let archive = format!("restore-{}-old-{i}", self.object_id);
                    if !same(
                        &capture_file(&self.originals.directory, &archive, MAX_CONTENT)?,
                        &expected_stage,
                        false,
                    ) {
                        return Err(denied("archived source no longer matches original plan"));
                    }
                    rename(
                        &self.originals.directory,
                        &archive,
                        self.staging(),
                        &stage,
                        libc::RENAME_EXCL,
                    )?;
                }
                // Clock/time fields legitimately change during our own rename;
                // byte/protection/dev/inode/birth identity must match the plan.
                checkpoint("before_native")?;
                if self.state(&plan, &c, i)? != after {
                    return Err(denied("target changed before native publication"));
                }
                check()?;
                checkpoint("native_gap")?;
                match plan.kind(i) {
                    Kind::Swap => rename(
                        self.staging(),
                        &stage,
                        self.target(i)?,
                        target_name,
                        libc::RENAME_SWAP,
                    )?,
                    Kind::Remove if want => rename(
                        self.target(i)?,
                        target_name,
                        self.staging(),
                        &stage,
                        libc::RENAME_EXCL,
                    )?,
                    Kind::Remove => rename(
                        self.staging(),
                        &stage,
                        self.target(i)?,
                        target_name,
                        libc::RENAME_EXCL,
                    )?,
                    Kind::Publish if want => rename(
                        self.staging(),
                        &stage,
                        self.target(i)?,
                        target_name,
                        libc::RENAME_EXCL,
                    )?,
                    Kind::Publish => rename(
                        self.target(i)?,
                        target_name,
                        self.staging(),
                        &stage,
                        libc::RENAME_EXCL,
                    )?,
                    Kind::Keep => {}
                }
                checkpoint("after_native")?;
                check()?;
                if self.state(&plan, &c, i).ok() != Some(want) {
                    return self
                        .preserve_publication_conflict(&plan, &c, i, want, check, checkpoint);
                }
            }
            checkpoint("step")?;
        }
        check()?;
        for i in plan.indices() {
            if plan.kind(i) != Kind::Keep
                && self.state(&plan, &c, i)? != (decision == Decision::Complete)
            {
                return Err(denied("restore final state differs"));
            }
        }
        // Move obsolete known objects into the protected journal, never delete
        // a guessed pathname. EXDEV leaves the transaction pending; no copy /
        // unsafe unlink fallback is performed for a different filesystem.
        for i in plan.indices() {
            check()?;
            let (_, staged) = self.pair(i)?;
            if staged != Image::Absent {
                let expected = Self::expected(&plan, &c, i, decision == Decision::Complete).1;
                if !same(&staged, &expected, false) {
                    return Err(denied("unknown stage cannot be archived"));
                }
                let to = format!("restore-{}-old-{i}", self.object_id);
                check()?;
                checkpoint("archive_gap")?;
                rename(
                    self.staging(),
                    &format!(".ocxd-restore-{}-{i}", self.object_id),
                    &self.originals.directory,
                    &to,
                    libc::RENAME_EXCL,
                )?;
                let archived = capture_file(&self.originals.directory, &to, MAX_CONTENT)?;
                if !same(&archived, &expected, false) {
                    return self.preserve_archive_conflict(i, &to, check, checkpoint);
                }
            }
            checkpoint("archive")?;
        }
        let final_capture = Capture::read(&self.originals.client)?;
        let expected = if decision == Decision::Complete {
            &c[0]
        } else {
            &c[3]
        };
        if final_capture
            .images
            .iter()
            .zip(&expected.images)
            .any(|(a, b)| !original_content(a, b))
        {
            return Err(denied("full original bytes or protection not restored"));
        }
        if let Some(config) = &plan.empty_config {
            let actual = capture_file(&self.originals.client, "config.toml", 4096)?;
            let expected = if decision == Decision::Complete {
                &Image::Absent
            } else {
                &config.stopped
            };
            if !original_content(&actual, expected) {
                return Err(denied("optional config target differs at terminal"));
            }
        }
        if let Some(backup) = &plan.grok_backup {
            let actual = capture_file(self.target(4)?, target_name(4)?, MAX_CONTENT)?;
            let expected = if decision == Decision::Complete {
                &Image::Absent
            } else {
                &backup.stopped
            };
            if !original_content(&actual, expected) {
                return Err(denied("backup target differs at terminal"));
            }
        }
        check()?;
        let outcome = encode(
            &serde_json::json!({"schemaVersion":1,"planSha256":self.sha256,"decision":decision}),
        )?;
        if existing == Image::Absent {
            create(&self.originals.directory, &terminal, &outcome)?;
        }
        checkpoint("terminal")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "macos_client_restore_tests.rs"]
mod tests;
