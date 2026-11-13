//! First-use directory custody, before the ordinary client-root-bound index.
//! This module is not wired to Start yet. Its returned in-process proof must
//! first be registered by the lifecycle owner; disk files cannot mint it.
//! It never captures an "original existing root", starts a service or removes
//! a directory. Interrupted evidence is preserved, never automatically adopted.
use super::*;
use session_index::observation::{action_pin, entry};
use std::time::Instant;
const REFERENCE: &str = "client-root-intent.json";
const PURPOSE: &str = "create-first-codex-root";
/// Mere creation evidence cannot initialize an ordinary baseline. Only an
/// independently registered root binding may validate these records.
pub(super) fn has_creation_evidence(manager: &PinnedDirectory) -> io::Result<bool> {
    Ok(entry(manager, REFERENCE)?.is_some()
        || entry(manager, "client-root-reservation")?.is_some()
        || entry(manager, "client-root-generations")?.is_some())
}
pub(super) fn require_no_pending_root(manager: &PinnedDirectory) -> io::Result<()> {
    if has_creation_evidence(manager)? {
        return Err(denied(
            "first client root is not registered with lifecycle; preserved",
        ));
    }
    Ok(())
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Intent {
    schema_version: u32,
    purpose: String,
    id: String,
    uid: u32,
    data_root: PathBuf,
    data_identity: Identity,
    profile: PathBuf,
    profile_identity: Identity,
    profile_protection: Protection,
    original_root_absent: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IntentReference<'a> {
    schema_version: u32,
    id: &'a str,
    intent_sha256: &'a str,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct References {
    pub id: String,
    pub intent_sha256: String,
    pub created_sha256: String,
    pub published_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_slot: Option<String>,
    // A transient exact namespace observation supplied by the owning index or
    // live creation transfer. Never serialized or reconstructed from a file.
    #[serde(skip)]
    namespace: Option<Vec<String>>,
}
impl PartialEq for References {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.intent_sha256 == other.intent_sha256
            && self.created_sha256 == other.created_sha256
            && self.published_sha256 == other.published_sha256
            && self.generation_slot == other.generation_slot
    }
}
impl Eq for References {}
impl References {
    pub(super) fn bind_indexed_slots(
        &mut self,
        namespace: &session_index::root_lineage::IndexedNamespace,
    ) {
        self.namespace = Some(namespace.names().to_vec());
    }
    pub(super) fn inherit_namespace(&mut self, source: &Self) -> io::Result<()> {
        if self != source {
            return Err(denied("namespace cannot rebind root provenance"));
        }
        self.namespace = source.namespace.clone();
        Ok(())
    }
    fn expected_slots(&self) -> Vec<String> {
        self.namespace
            .clone()
            .unwrap_or_else(|| self.generation_slot.iter().cloned().collect())
    }
    pub(super) fn verify_namespace(&self, manager: &PinnedDirectory) -> io::Result<()> {
        verify_generation_slots(manager, &self.expected_slots())
    }
}
/// Exact indexed names, or the live Reservation's known indexed names plus its
/// sole create-only pending slot. Enumeration never supplies new references.
pub(super) fn verify_generation_slots(
    manager: &PinnedDirectory,
    expected: &[String],
) -> io::Result<()> {
    if expected.len() > 511
        || expected.windows(2).any(|p| p[0] >= p[1])
        || expected.iter().any(|id| {
            uuid::Uuid::parse_str(id)
                .map(|u| u.to_string() != *id)
                .unwrap_or(true)
        })
    {
        return Err(denied("invalid indexed generation namespace"));
    }
    if entry(manager, "client-root-generations")?.is_none() {
        return if expected.is_empty() {
            Ok(())
        } else {
            Err(denied("indexed root generation slots missing"))
        };
    }
    let generations = manager.child_private("client-root-generations", false)?;
    let names = tree::names(generations.file())?;
    let wanted = expected
        .iter()
        .map(std::ffi::OsString::from)
        .collect::<Vec<_>>();
    if names != wanted {
        return Err(denied(
            "root generation namespace differs from independent index",
        ));
    }
    for id in expected {
        generations.child_private(id, false)?.private()?;
    }
    if tree::names(generations.file())? != names {
        return Err(denied("root generation namespace changed during review"));
    }
    generations.private()?;
    manager.recheck()
}

/// A move of the original flock/created fd, never a release plus path reopen.
pub(super) struct Transfer {
    pub(super) action: File,
    pub(super) action_identity: Identity,
    pub(super) action_protection: Protection,
    pub(super) created: File,
    pub(super) references: References,
    pub(super) deadline: Instant,
    _sealed: (),
}
impl Transfer {
    pub(super) fn bind_predecessor_namespace(
        &self,
        origin: &mut References,
        indexed: &[String],
    ) -> io::Result<()> {
        let mut expected = indexed.to_vec();
        if expected.contains(&self.references.id) {
            return Err(denied("pending generation reuses indexed slot"));
        }
        expected.push(self.references.id.clone());
        expected.sort();
        if self.references.generation_slot.as_deref() != Some(self.references.id.as_str())
            || self.references.namespace.as_ref() != Some(&expected)
        {
            return Err(denied(
                "pending generation namespace not held by creation transfer",
            ));
        }
        origin.namespace = Some(expected);
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreatedRecord {
    schema_version: u32,
    intent_sha256: String,
    root_identity: Identity,
    root_protection: Protection,
    root_empty: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PublishedRecord {
    schema_version: u32,
    intent_sha256: String,
    created_sha256: String,
    root_identity: Identity,
    root_protection: Protection,
}
/// Read-only validation of facts already referenced by the independent index.
/// This function cannot construct PublishedRoot/Transfer or a service permit.
pub(super) fn verify_indexed(
    manager: &PinnedDirectory,
    data: &PinnedDirectory,
    profile: &Path,
    client: &PinnedDirectory,
    refs: &References,
) -> io::Result<()> {
    verify_indexed_anchor(
        manager,
        data,
        profile,
        client,
        refs,
        &profile.join(".codex"),
    )
}
/// Only an owning immutable retirement plan may supply an alternative physical
/// anchor. The logical user root in Intent remains profile/.codex.
pub(super) fn verify_indexed_anchor(
    manager: &PinnedDirectory,
    data: &PinnedDirectory,
    profile: &Path,
    client: &PinnedDirectory,
    refs: &References,
    expected_anchor: &Path,
) -> io::Result<()> {
    #[cfg(test)]
    let _metric = observation_metrics::span("root_provenance");
    if uuid::Uuid::parse_str(&refs.id)
        .map(|v| v.to_string() != refs.id)
        .unwrap_or(true)
        || [
            &refs.intent_sha256,
            &refs.created_sha256,
            &refs.published_sha256,
        ]
        .iter()
        .any(|s| {
            s.len() != 64
                || !s
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
    {
        return Err(denied("invalid indexed root references"));
    }
    let profile = PinnedDirectory::open(profile)?;
    verify_anchor_metadata(&profile, data, client)?;
    verify_generation_slots(manager, &refs.expected_slots())?;
    let vault = if let Some(id) = &refs.generation_slot {
        if id != &refs.id {
            return Err(denied("root generation slot differs from origin"));
        }
        manager.descendant_private(&["client-root-generations", id])?
    } else {
        manager.child_private("client-root-reservation", false)?
    };
    let read = |name: &str, hash: &str| -> io::Result<Vec<u8>> {
        match capture_file(&vault, name, 65536)? {
            Image::File {
                bytes, protection, ..
            } if digest(&bytes) == hash => {
                require_private(&protection, false)?;
                Ok(bytes)
            }
            _ => Err(denied("indexed root provenance is missing or changed")),
        }
    };
    let intent_bytes = read("intent.json", &refs.intent_sha256)?;
    let intent: Intent = serde_json::from_value(
        crate::modules::model_config::schema_v2::parse_strict_json_v2(&intent_bytes)
            .map_err(|_| denied("invalid root intent"))?,
    )
    .map_err(|_| denied("invalid root intent fields"))?;
    let created: CreatedRecord =
        serde_json::from_slice(&read("created.json", &refs.created_sha256)?)
            .map_err(|_| denied("invalid root creation fields"))?;
    let published: PublishedRecord =
        serde_json::from_slice(&read("published.json", &refs.published_sha256)?)
            .map_err(|_| denied("invalid root publication fields"))?;
    if intent.schema_version != 1
        || intent.id != refs.id
        || intent.purpose != PURPOSE
        || !intent.original_root_absent
        || intent.uid != unsafe { libc::geteuid() }
        || intent.data_root != data.path()
        || intent.data_identity != *data.identity()
        || intent.profile != profile.path()
        || intent.profile_identity != *profile.identity()
        || intent.profile_protection != protection(profile.file(), &stat(profile.file())?)?
        || client.path() != expected_anchor
        || created.schema_version != 1
        || !created.root_empty
        || created.intent_sha256 != refs.intent_sha256
        || published.schema_version != 1
        || published.intent_sha256 != refs.intent_sha256
        || published.created_sha256 != refs.created_sha256
        || published.root_identity != created.root_identity
        || published.root_protection != created.root_protection
        || created.root_identity != *client.identity()
        || created.root_protection != protection(client.file(), &stat(client.file())?)?
    {
        return Err(denied(
            "indexed root provenance does not bind this original absence and created directory",
        ));
    }
    let expected = encode(&IntentReference {
        schema_version: 1,
        id: &refs.id,
        intent_sha256: &refs.intent_sha256,
    })?;
    let reference_root = if refs.generation_slot.is_some() {
        &vault
    } else {
        manager
    };
    match capture_file(reference_root, REFERENCE, 65536)? {
        Image::File {
            bytes, protection, ..
        } if bytes == expected => require_private(&protection, false),
        _ => Err(denied("root intent reference changed")),
    }?;
    refs.verify_namespace(manager)
}
/// This lease uses the SAME action flock as the ordinary independent index.
/// Index integration must transfer this custody, not acquire a second lock.
pub struct Reservation {
    data: PinnedDirectory,
    profile: PinnedDirectory,
    manager: PinnedDirectory,
    vault: PinnedDirectory,
    action: File,
    action_identity: Identity,
    action_protection: Protection,
    intent: Intent,
    intent_sha256: String,
    deadline: Instant,
    generation_slot: Option<String>,
    generation_slots: Vec<String>,
}
pub struct PublishedRoot {
    reservation: Reservation,
    created: File,
    root: PinnedDirectory,
    identity: Identity,
    protection: Protection,
    refs: References,
}
fn time(deadline: Instant) -> io::Result<()> {
    if Instant::now() >= deadline {
        Err(denied("first client root creation expired"))
    } else {
        Ok(())
    }
}
fn absent(profile: &PinnedDirectory) -> io::Result<()> {
    profile.recheck()?;
    if entry(profile, ".codex")?.is_some() {
        return Err(denied("first client root already exists; not adopted"));
    }
    profile.recheck()
}
fn safe_profile(profile: &PinnedDirectory) -> io::Result<()> {
    profile.recheck()?;
    safe_profile_leaf(profile)
}
fn safe_profile_leaf(profile: &PinnedDirectory) -> io::Result<()> {
    let s = stat(profile.file())?;
    let p = protection(profile.file(), &s)?;
    if p.uid != unsafe { libc::geteuid() } || p.mode & 0o022 != 0 || p.flags != 0 {
        return Err(denied("first client profile write access unknown"));
    }
    Ok(())
}
/// No record/namespace contents are read before every original leaf policy
/// and two complete fresh path/fd/security passes have succeeded. The caller
/// still performs its full per-frame provenance reads and dynamic bindings.
fn verify_anchor_metadata(
    profile: &PinnedDirectory,
    data: &PinnedDirectory,
    client: &PinnedDirectory,
) -> io::Result<()> {
    #[cfg(test)]
    if observation_metrics::root_metadata_reference_enabled() {
        safe_profile(profile)?;
        data.private()?;
        return client.private();
    }
    verify_anchor_metadata_checked(profile, data, client, &|| Ok(()))
}
fn verify_anchor_metadata_checked(
    profile: &PinnedDirectory,
    data: &PinnedDirectory,
    client: &PinnedDirectory,
    checkpoint: &impl Fn() -> io::Result<()>,
) -> io::Result<()> {
    safe_profile_leaf(profile)?;
    require_private(&protection(data.file(), &stat(data.file())?)?, true)?;
    require_private(&protection(client.file(), &stat(client.file())?)?, true)?;
    checkpoint()?;
    PinnedDirectory::recheck_set(&[profile, data, client])
}
#[cfg(test)]
#[path = "macos_client_root_metadata_tests.rs"]
mod metadata_tests;
fn require_uninitialized(manager: &PinnedDirectory) -> io::Result<()> {
    for n in [
        "client-session-index.sqlite",
        "client-session-index.sqlite-journal",
        "client-session-index.sqlite-wal",
        "client-session-index.sqlite-shm",
        "client-originals",
        REFERENCE,
        "client-root-reservation",
        "client-root-generations",
    ] {
        if entry(manager, n)?.is_some() {
            return Err(denied(
                "prior client initialization evidence exists; not rebased",
            ));
        }
    }
    Ok(())
}
/// Filesystem custody only. Caller still needs full official/runtime/native
/// admission and an independent index registration before a service can start.
pub fn reserve(
    data_root: &Path,
    profile_root: &Path,
    deadline: Instant,
) -> io::Result<Reservation> {
    reserve_checked(data_root, profile_root, deadline, &|_| Ok(()))
}
/// Only the held Index can construct this authority. A copied RootAbsent or
/// on-disk reservation cannot provide the original action flock/context.
pub(super) fn reserve_generation(
    authority: &session_index::generation::CreationAuthority<'_>,
    deadline: Instant,
) -> io::Result<Reservation> {
    authority.check()?;
    time(deadline)?;
    let (data_root, profile_root, manager_root) = authority.paths();
    let data = PinnedDirectory::open(data_root)?;
    let profile = PinnedDirectory::open(profile_root)?;
    let manager = PinnedDirectory::open(manager_root)?;
    data.private()?;
    safe_profile(&profile)?;
    absent(&profile)?;
    let indexed = authority.slots()?;
    if indexed.len() >= 511 {
        return Err(denied("root generation count exhausted before creation"));
    }
    verify_generation_slots(&manager, &indexed)?;
    // Clone while the namespace still matches the predecessor. Later checks
    // are bound to the live Reservation's exact indexed-plus-pending set.
    let action = authority.clone_action()?;
    let (action_identity, action_protection) = action_pin(&manager, &action)?;
    let generations = manager.child_private("client-root-generations", true)?;
    let id = uuid::Uuid::new_v4().to_string();
    let mut generation_slots = indexed;
    if generation_slots.contains(&id) {
        return Err(denied("new generation id already indexed"));
    }
    generation_slots.push(id.clone());
    generation_slots.sort();
    let slot = name(OsStr::new(&id))?;
    if unsafe { libc::mkdirat(generations.file().as_raw_fd(), slot.as_ptr(), 0o700) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let vault = generations.child_private(&id, false)?;
    // Persist both create-only directory entries before a later SQLite commit
    // can refer to them. Syncing files inside slot alone does not sync parents.
    generations.sync()?;
    manager.sync()?;
    let intent = Intent {
        schema_version: 1,
        purpose: PURPOSE.into(),
        id: id.clone(),
        uid: unsafe { libc::geteuid() },
        data_root: data.path().into(),
        data_identity: data.identity().clone(),
        profile: profile.path().into(),
        profile_identity: profile.identity().clone(),
        profile_protection: protection(profile.file(), &stat(profile.file())?)?,
        original_root_absent: true,
    };
    let bytes = encode(&intent)?;
    let intent_sha256 = digest(&bytes);
    create(&vault, "intent.json", &bytes)?;
    create(
        &vault,
        REFERENCE,
        &encode(&IntentReference {
            schema_version: 1,
            id: &id,
            intent_sha256: &intent_sha256,
        })?,
    )?;
    vault.sync()?;
    generations.sync()?;
    manager.sync()?;
    let result = Reservation {
        data,
        profile,
        manager,
        vault,
        action,
        action_identity,
        action_protection,
        intent,
        intent_sha256,
        deadline,
        generation_slot: Some(id),
        generation_slots,
    };
    result.check()?;
    time(deadline)?;
    Ok(result)
}
fn reserve_checked(
    data_root: &Path,
    profile_root: &Path,
    deadline: Instant,
    checkpoint: &impl Fn(&str) -> io::Result<()>,
) -> io::Result<Reservation> {
    time(deadline)?;
    let data = PinnedDirectory::open(data_root)?;
    data.private()?;
    let profile = PinnedDirectory::open(profile_root)?;
    safe_profile(&profile)?;
    absent(&profile)?;
    let target = profile.path().join(".codex");
    if data_root.starts_with(&target) || target.starts_with(data_root) {
        return Err(denied("first client root overlaps transaction data"));
    }
    let manager = data.child_private("manager-state", true)?;
    let n = name(OsStr::new("client-session-action.lock"))?;
    let fd = unsafe {
        libc::openat(
            manager.file().as_raw_fd(),
            n.as_ptr(),
            libc::O_RDWR | libc::O_CREAT | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
            0o600,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let action = unsafe { File::from_raw_fd(fd) };
    let (action_identity, action_protection) = action_pin(&manager, &action)?;
    action
        .try_lock()
        .map_err(|_| denied("another first-client or lifecycle action is active"))?;
    if action_pin(&manager, &action)? != (action_identity.clone(), action_protection.clone()) {
        return Err(denied("first client action lock changed"));
    }
    require_uninitialized(&manager)?;
    absent(&profile)?;
    checkpoint("absent")?;
    time(deadline)?;
    absent(&profile)?;
    let id = uuid::Uuid::new_v4().to_string();
    // Fixed, create-only slot refuses even unreferenced leftovers next time.
    let slot = name(OsStr::new("client-root-reservation"))?;
    if unsafe { libc::mkdirat(manager.file().as_raw_fd(), slot.as_ptr(), 0o700) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let vault = manager.child_private("client-root-reservation", false)?;
    if !tree::names(vault.file())?.is_empty() {
        return Err(denied("unregistered root reservation remains"));
    }
    let intent = Intent {
        schema_version: 1,
        purpose: PURPOSE.into(),
        id,
        uid: unsafe { libc::geteuid() },
        data_root: data.path().into(),
        data_identity: data.identity().clone(),
        profile: profile.path().into(),
        profile_identity: profile.identity().clone(),
        profile_protection: protection(profile.file(), &stat(profile.file())?)?,
        original_root_absent: true,
    };
    let bytes = encode(&intent)?;
    let intent_sha256 = digest(&bytes);
    create(&vault, "intent.json", &bytes)?;
    create(
        &manager,
        REFERENCE,
        &encode(&IntentReference {
            schema_version: 1,
            id: &intent.id,
            intent_sha256: &intent_sha256,
        })?,
    )?;
    checkpoint("intent_published")?;
    let result = Reservation {
        data,
        profile,
        manager,
        vault,
        action,
        action_identity,
        action_protection,
        intent,
        intent_sha256,
        deadline,
        generation_slot: None,
        generation_slots: Vec::new(),
    };
    result.check()?;
    Ok(result)
}
impl Reservation {
    fn check(&self) -> io::Result<()> {
        time(self.deadline)?;
        self.data.private()?;
        safe_profile(&self.profile)?;
        self.vault.private()?;
        if let Some(id) = &self.generation_slot {
            if !self.generation_slots.contains(id) {
                return Err(denied("pending generation absent from owned namespace"));
            }
            verify_generation_slots(&self.manager, &self.generation_slots)?;
        } else {
            verify_generation_slots(&self.manager, &[])?;
        }
        if action_pin(&self.manager, &self.action)?
            != (self.action_identity.clone(), self.action_protection.clone())
        {
            return Err(denied("root reservation lost action identity"));
        }
        let expected = encode(&IntentReference {
            schema_version: 1,
            id: &self.intent.id,
            intent_sha256: &self.intent_sha256,
        })?;
        let reference_root = if self.generation_slot.is_some() {
            &self.vault
        } else {
            &self.manager
        };
        match capture_file(reference_root, REFERENCE, 65536)? {
            Image::File {
                bytes, protection, ..
            } if bytes == expected => require_private(&protection, false)?,
            _ => return Err(denied("root intent reference changed")),
        }
        match capture_file(&self.vault, "intent.json", 65536)? {
            Image::File {
                bytes, protection, ..
            } if digest(&bytes) == self.intent_sha256 && bytes == encode(&self.intent)? => {
                require_private(&protection, false)?
            }
            _ => return Err(denied("original root absence evidence changed")),
        }
        if self.profile.identity() != &self.intent.profile_identity
            || self.data.identity() != &self.intent.data_identity
            || protection(self.profile.file(), &stat(self.profile.file())?)?
                != self.intent.profile_protection
        {
            return Err(denied("first client root context changed"));
        }
        Ok(())
    }
    pub fn publish(self) -> io::Result<PublishedRoot> {
        self.publish_checked(&|_| Ok(()))
    }
    fn publish_checked(
        self,
        checkpoint: &impl Fn(&str) -> io::Result<()>,
    ) -> io::Result<PublishedRoot> {
        self.check()?;
        absent(&self.profile)?;
        let n = name(OsStr::new("candidate"))?;
        if unsafe { libc::mkdirat(self.vault.file().as_raw_fd(), n.as_ptr(), 0o700) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let root = self.vault.child_private("candidate", false)?;
        if root.identity().device != self.profile.identity().device
            || !tree::names(root.file())?.is_empty()
        {
            return Err(denied("first client root is nonempty or cross-volume"));
        }
        let created = root.file().try_clone()?;
        let identity = root.identity().clone();
        let protection = protection(root.file(), &stat(root.file())?)?;
        let bytes = encode(
            &serde_json::json!({"schemaVersion":1,"intentSha256":self.intent_sha256,"rootIdentity":identity,"rootProtection":protection,"rootEmpty":true}),
        )?;
        let created_sha256 = digest(&bytes);
        create(&self.vault, "created.json", &bytes)?;
        checkpoint("created")?;
        self.check()?;
        root.recheck()?;
        absent(&self.profile)?;
        checkpoint("before_publish")?;
        let target = name(OsStr::new(".codex"))?;
        if unsafe {
            libc::renameatx_np(
                self.vault.file().as_raw_fd(),
                n.as_ptr(),
                self.profile.file().as_raw_fd(),
                target.as_ptr(),
                libc::RENAME_EXCL,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
        self.profile.sync()?;
        self.vault.sync()?;
        checkpoint("root_published")?;
        let published = PinnedDirectory::open(&self.profile.path().join(".codex"))?;
        published.private()?;
        if published.identity() != &identity
            || super::identity(&stat(&created)?) != identity
            || super::protection(published.file(), &stat(published.file())?)? != protection
            || !tree::names(published.file())?.is_empty()
        {
            return Err(denied("published first root changed; evidence retained"));
        }
        let bytes = encode(
            &serde_json::json!({"schemaVersion":1,"intentSha256":self.intent_sha256,"createdSha256":created_sha256,"rootIdentity":identity,"rootProtection":protection}),
        )?;
        let published_sha256 = digest(&bytes);
        create(&self.vault, "published.json", &bytes)?;
        checkpoint("publication_recorded")?;
        let refs = References {
            id: self.intent.id.clone(),
            intent_sha256: self.intent_sha256.clone(),
            created_sha256,
            published_sha256,
            generation_slot: self.generation_slot.clone(),
            namespace: Some(self.generation_slots.clone()),
        };
        let result = PublishedRoot {
            reservation: self,
            created,
            root: published,
            identity,
            protection,
            refs,
        };
        result.check()?;
        Ok(result)
    }
}
impl PublishedRoot {
    pub(super) fn into_generation_transfer(
        self,
        data: &PinnedDirectory,
        profile: &Path,
        client: &PinnedDirectory,
    ) -> io::Result<Transfer> {
        self.check()?;
        if self.refs.generation_slot.as_deref() != Some(self.refs.id.as_str())
            || self.reservation.data.path() != data.path()
            || self.reservation.data.identity() != data.identity()
            || self.reservation.profile.path() != profile
            || self.root.path() != client.path()
            || self.root.identity() != client.identity()
        {
            return Err(denied("successor root creation transfer context changed"));
        }
        let Self {
            reservation,
            created,
            refs,
            ..
        } = self;
        Ok(Transfer {
            action: reservation.action,
            action_identity: reservation.action_identity,
            action_protection: reservation.action_protection,
            created,
            references: refs,
            deadline: reservation.deadline,
            _sealed: (),
        })
    }
    pub(super) fn into_transfer(
        self,
        data: &PinnedDirectory,
        profile: &Path,
        client: &PinnedDirectory,
    ) -> io::Result<Transfer> {
        self.check()?;
        if self.reservation.data.path() != data.path()
            || self.reservation.data.identity() != data.identity()
            || self.reservation.profile.path() != profile
            || self.root.path() != client.path()
            || self.root.identity() != client.identity()
        {
            return Err(denied("root creation transfer context changed"));
        }
        for name in [
            "client-session-index.sqlite",
            "client-session-index.sqlite-journal",
            "client-session-index.sqlite-wal",
            "client-session-index.sqlite-shm",
            "client-originals",
            "client-root-generations",
        ] {
            if entry(&self.reservation.manager, name)?.is_some() {
                return Err(denied(
                    "root creation cannot replace an existing lifecycle index",
                ));
            }
        }
        let Self {
            reservation,
            created,
            refs,
            ..
        } = self;
        Ok(Transfer {
            action: reservation.action,
            action_identity: reservation.action_identity,
            action_protection: reservation.action_protection,
            created,
            references: refs,
            deadline: reservation.deadline,
            _sealed: (),
        })
    }
    pub fn references(&self) -> io::Result<References> {
        self.check()?;
        Ok(self.refs.clone())
    }
    pub fn path(&self) -> &Path {
        self.root.path()
    }
    pub fn check(&self) -> io::Result<()> {
        self.reservation.check()?;
        self.root.private()?;
        if self.root.identity() != &self.identity
            || super::identity(&stat(&self.created)?) != self.identity
            || super::protection(&self.created, &stat(&self.created)?)? != self.protection
        {
            return Err(denied("published root lost created object identity"));
        }
        for (name, hash) in [
            ("created.json", &self.refs.created_sha256),
            ("published.json", &self.refs.published_sha256),
        ] {
            match capture_file(&self.reservation.vault, name, 65536)? {
                Image::File {
                    bytes, protection, ..
                } if digest(&bytes) == *hash => require_private(&protection, false)?,
                _ => return Err(denied("root publication evidence changed")),
            }
        }
        if !tree::names(self.root.file())?.is_empty() {
            return Err(denied("first root already changed before index transfer"));
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "macos_client_root_creation_tests.rs"]
mod tests;
