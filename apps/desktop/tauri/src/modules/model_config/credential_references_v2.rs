//! Save-only namespace copies. Caller holds admission -> rename gate -> config
//! -> Store/version locks, also during recovery. Never writes configuration,
//! resolves ENV, deletes source credentials, or scans historical roots.
use super::{provider_auth, provider_fields, schema::*, schema_v2, store::sha256};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{File, Metadata, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};
use zeroize::Zeroizing;
const PENDING: &str = "credential-references-v2.pending.json";
const OPERATIONS: &str = "credential-references-v2";
const JOURNAL: &str = "journal.json";
const MAX_IMAGE: usize = 1024 * 1024;
const MAX_MARKER: usize = 4096;
const MAX_REFS: usize = 512;
const MAX_SECRET: usize = 8192;
const MAX_TOTAL: usize = 4 * 1024 * 1024;
type Result<T> = ModelResult<T>;
type Bytes = Zeroizing<Vec<u8>>;
pub struct ReferenceSourceV2 {
    pub source_provider_id: String,
    pub target_provider_id: String,
}
/// create refuses existing entries. Success is an ownership receipt; an error
/// may have created an unknown entry. Implementations must never log secrets.
pub trait CredentialVaultV2 {
    fn read(&self, account: &str) -> ModelResult<Option<Zeroizing<String>>>;
    fn create(&self, account: &str, value: &str) -> ModelResult<()>;
    fn remove_if_digest(&self, account: &str, digest: &str) -> ModelResult<()>;
}
pub struct SystemVault;
impl SystemVault {
    fn entry(account: &str) -> Result<keyring::Entry> {
        // Separate sandbox identity, never fall back to the daily service.
        let service = if crate::modules::test_sandbox::enabled() {
            crate::infrastructure::keychain::keychain_service_name()
        } else {
            "opencodex.provider-api-key.v1"
        };
        keyring::Entry::new(service, account).map_err(|_| fault("credential_references_vault"))
    }
}
impl CredentialVaultV2 for SystemVault {
    fn read(&self, account: &str) -> Result<Option<Zeroizing<String>>> {
        match Self::entry(account)?.get_password() {
            Ok(s) => Ok(Some(Zeroizing::new(s))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(fault("credential_references_vault")),
        }
    }
    fn create(&self, account: &str, value: &str) -> Result<()> {
        // System keyring has no create-only CAS. App writers are serialized by
        // the caller, matching channel_rename's existing read-before-put policy.
        if self.read(account)?.is_some() {
            return Err(fault("credential_references_collision"));
        }
        Self::entry(account)?
            .set_password(value)
            .map_err(|_| fault("credential_references_vault"))
    }
    fn remove_if_digest(&self, account: &str, digest: &str) -> Result<()> {
        if let Some(v) = self.read(account)? {
            if sha256(v.as_bytes()) != digest {
                return Err(fault("credential_references_collision"));
            }
            Self::entry(account)?
                .delete_credential()
                .map_err(|_| fault("credential_references_vault"))?;
            if self.read(account)?.is_some() {
                return Err(fault("credential_references_vault"));
            }
        }
        Ok(())
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    root: String,
    home: String,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Copy {
    source_provider: String,
    target_provider: String,
    from: String,
    to: String,
    digest: String,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Spec {
    id: String,
    binding: Binding,
    before: String,
    after: String,
    copies: Vec<Copy>,
}
/// Immutable operation identity and metadata only; intentionally no Debug/IPC.
pub struct ReferencePlanV2 {
    spec: Spec,
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Action {
    Absent,
    Creating,
    Removing,
    Unowned,
    Created,
    Reused,
    Removed,
}
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Phase {
    Preparing,
    Prepared,
    Aborting,
    Aborted,
    Finished,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    version: u32,
    spec: Spec,
    phase: Phase,
    actions: Vec<Action>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Marker {
    version: u32,
    id: String,
    binding: Binding,
    spec_digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope<T> {
    value: T,
    checksum: String,
}
fn fault(code: &'static str) -> ModelError {
    invalid(
        code,
        "凭据引用事务未完成，原账户和保护记录已保留，请核对后重试",
    )
}
fn io<T>(r: std::io::Result<T>) -> Result<T> {
    r.map_err(|_| fault("credential_references_io"))
}
fn encoded<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    schema_v2::canonical_bytes(v).map_err(|_| fault("credential_references_metadata"))
}
fn checksum<T: Serialize>(v: &T) -> Result<String> {
    Ok(sha256(&encoded(v)?))
}
fn envelope<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    let b = encoded(&Envelope {
        value: v,
        checksum: checksum(v)?,
    })?;
    if b.len() > MAX_IMAGE - MAX_MARKER {
        return Err(fault("credential_references_limit"));
    }
    Ok(b)
}
fn decode<T: DeserializeOwned + Serialize>(b: &[u8]) -> Result<T> {
    let v =
        schema_v2::parse_strict_json_v2(b).map_err(|_| fault("credential_references_metadata"))?;
    let e: Envelope<T> =
        serde_json::from_value(v).map_err(|_| fault("credential_references_metadata"))?;
    if checksum(&e.value)? != e.checksum {
        return Err(fault("credential_references_metadata"));
    }
    Ok(e.value)
}
fn value_digest(value: &Value) -> Result<String> {
    // Stream canonical JSON into the hash: no clone or plaintext byte buffer.
    use sha2::{Digest, Sha256};
    struct Sink(Sha256);
    impl Write for Sink {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.update(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    fn visit(v: &Value, out: &mut Sink) -> Result<()> {
        match v {
            Value::Object(m) => {
                io(out.write_all(b"{"))?;
                for (i, (k, v)) in m.iter().collect::<BTreeMap<_, _>>().into_iter().enumerate() {
                    if i > 0 {
                        io(out.write_all(b","))?;
                    }
                    serde_json::to_writer(&mut *out, k)
                        .map_err(|_| fault("credential_references_config"))?;
                    io(out.write_all(b":"))?;
                    visit(v, out)?;
                }
                io(out.write_all(b"}"))?;
            }
            Value::Array(a) => {
                io(out.write_all(b"["))?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        io(out.write_all(b","))?;
                    }
                    visit(v, out)?;
                }
                io(out.write_all(b"]"))?;
            }
            _ => {
                serde_json::to_writer(out, v).map_err(|_| fault("credential_references_config"))?
            }
        }
        Ok(())
    }
    let mut out = Sink(Sha256::new());
    visit(value, &mut out)?;
    Ok(format!("sha256:{:x}", out.0.finalize()))
}
fn uuid4(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok_and(|u| {
        u.to_string() == id && u.get_version_num() == 4 && u.get_variant() == uuid::Variant::RFC4122
    })
}
fn provider_id(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 256
        || id.trim() != id
        || id.contains(['/', '\\'])
        || id.chars().any(char::is_control)
        || ["openai", "chatgpt", "openai-multi", "combo", "native"]
            .contains(&id.to_ascii_lowercase().as_str())
    {
        return Err(fault("credential_references_provider"));
    }
    Ok(())
}
fn ordinary<'a>(doc: &'a Value, id: &str) -> Result<&'a Value> {
    provider_id(id)?;
    let row = doc
        .get("providers")
        .and_then(|p| p.get(id))
        .filter(|v| v.is_object())
        .ok_or_else(|| fault("credential_references_provider"))?;
    if !matches!(row.get("authMode"), None | Some(Value::Null))
        && row["authMode"].as_str() != Some("key")
    {
        return Err(fault("credential_references_provider"));
    }
    Ok(row)
}
fn walk(v: &Value, count: &mut usize, f: &mut impl FnMut(&str) -> Result<()>) -> Result<()> {
    match v {
        Value::String(s) if s.starts_with("keychain:") => {
            *count += 1;
            if *count > MAX_REFS {
                return Err(fault("credential_references_limit"));
            }
            f(s)?;
        }
        Value::Array(a) => {
            for v in a {
                walk(v, count, f)?;
            }
        }
        Value::Object(o) => {
            for v in o.values() {
                walk(v, count, f)?;
            }
        }
        _ => (),
    }
    Ok(())
}
fn private_refs(
    row: &Value,
    count: &mut usize,
    f: &mut impl FnMut(&str) -> Result<()>,
) -> Result<()> {
    for field in provider_fields::PRIVATE {
        if let Some(v) = row.get(field) {
            walk(v, count, f)?;
        }
    }
    Ok(())
}
fn whole_references(v: &Value, account: &str) -> bool {
    match v {
        Value::String(s) => s.strip_prefix("keychain:") == Some(account),
        Value::Array(a) => a.iter().any(|v| whole_references(v, account)),
        Value::Object(o) => o.values().any(|v| whole_references(v, account)),
        _ => false,
    }
}
fn vault_read(vault: &dyn CredentialVaultV2, account: &str) -> Result<Option<Zeroizing<String>>> {
    let v = vault
        .read(account)
        .map_err(|_| fault("credential_references_vault"))?;
    if v.as_ref()
        .is_some_and(|s| s.is_empty() || s.len() > MAX_SECRET)
    {
        return Err(fault("credential_references_limit"));
    }
    Ok(v)
}
fn vault_digest(vault: &dyn CredentialVaultV2, account: &str) -> Result<Option<String>> {
    Ok(vault_read(vault, account)?.map(|v| sha256(v.as_bytes())))
}
struct Context {
    root: Dir,
    home: Dir,
    manager: Dir,
}
impl Context {
    fn open(root: &Path, home: &Path) -> Result<Self> {
        let root = Dir::anchor(root)?;
        let home = Dir::anchor(home)?;
        let manager = root
            .child("manager-state", true, false)?
            .ok_or_else(|| fault("credential_references_path"))?;
        Ok(Self {
            root,
            home,
            manager,
        })
    }
    fn binding(&self) -> Result<Binding> {
        Ok(Binding {
            root: self.root.binding()?,
            home: self.home.binding()?,
        })
    }
    fn operation(&self, id: &str, create: bool) -> Result<Dir> {
        if !uuid4(id) {
            return Err(fault("credential_references_metadata"));
        }
        self.root.live()?;
        self.home.live()?;
        let ops = self
            .manager
            .child(OPERATIONS, create, true)?
            .ok_or_else(|| fault("credential_references_metadata"))?;
        ops.child(id, create, true)?
            .ok_or_else(|| fault("credential_references_metadata"))
    }
    fn marker(&self) -> Result<Option<Marker>> {
        self.manager
            .read(PENDING, MAX_MARKER, true)?
            .map(|b| decode(&b))
            .transpose()
    }
    fn check_marker(&self, spec: &Spec, required: bool) -> Result<()> {
        if let Some(m) = self.marker()? {
            if m.version != 2
                || m.id != spec.id
                || m.binding != spec.binding
                || m.spec_digest != checksum(spec)?
            {
                return Err(fault("credential_references_pending"));
            }
        } else if required {
            return Err(fault("credential_references_pending"));
        }
        Ok(())
    }
    fn load(&self, spec: &Spec) -> Result<Journal> {
        if spec.binding != self.binding()? {
            return Err(fault("credential_references_binding"));
        }
        self.check_marker(spec, false)?;
        let dir = self.operation(&spec.id, false)?;
        let j: Journal = decode(
            &dir.read(JOURNAL, MAX_IMAGE, true)?
                .ok_or_else(|| fault("credential_references_metadata"))?,
        )?;
        validate(&j, &self.binding()?)?;
        if j.spec != *spec {
            return Err(fault("credential_references_metadata"));
        }
        self.check_marker(spec, !matches!(j.phase, Phase::Finished | Phase::Aborted))?;
        Ok(j)
    }
    fn save(&self, j: &Journal) -> Result<()> {
        validate(j, &self.binding()?)?;
        let dir = self.operation(&j.spec.id, false)?;
        let before = dir.read(JOURNAL, MAX_IMAGE, true)?;
        dir.replace(
            JOURNAL,
            before.as_deref().map(Vec::as_slice),
            Some(&envelope(j)?),
            true,
        )
    }
    fn clear(&self, spec: &Spec) -> Result<()> {
        self.check_marker(spec, false)?;
        if let Some(b) = self.manager.read(PENDING, MAX_IMAGE, true)? {
            self.manager.replace(PENDING, Some(&b), None, true)?;
        }
        Ok(())
    }
}
fn validate(j: &Journal, binding: &Binding) -> Result<()> {
    if j.version != 2
        || !uuid4(&j.spec.id)
        || j.spec.binding != *binding
        || !schema_v2::valid_digest(&j.spec.before)
        || !schema_v2::valid_digest(&j.spec.after)
        || j.spec.copies.len() > MAX_REFS
        || j.actions.len() != j.spec.copies.len()
    {
        return Err(fault("credential_references_metadata"));
    }
    let mut providers = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for (c, a) in j.spec.copies.iter().zip(&j.actions) {
        provider_id(&c.source_provider)?;
        provider_id(&c.target_provider)?;
        providers.insert(&c.source_provider);
        providers.insert(&c.target_provider);
        provider_auth::owned_account(&format!("keychain:{}", c.from), &c.source_provider)?;
        provider_auth::owned_account(&format!("keychain:{}", c.to), &c.target_provider)?;
        if c.to
            != format!(
                "{}{}",
                c.target_provider,
                &c.from[c.source_provider.len()..]
            )
            || !targets.insert(&c.to)
            || !schema_v2::valid_digest(&c.digest)
            || c.from == c.to && *a != Action::Reused
        {
            return Err(fault("credential_references_metadata"));
        }
    }
    if providers.len() > 32 {
        return Err(fault("credential_references_limit"));
    }
    Ok(())
}
/// Preflights all ownership and direction checks before the first vault read.
/// On error, next stays unchanged and only receipted creations are compensated.
pub fn prepare(
    root: &Path,
    home: &Path,
    original: &Value,
    next: &mut Value,
    sources: &[ReferenceSourceV2],
    vault: &dyn CredentialVaultV2,
) -> ModelResult<ReferencePlanV2> {
    if sources.len() > 32 {
        return Err(fault("credential_references_limit"));
    }
    let ctx = Context::open(root, home)?;
    if ctx.marker()?.is_some() {
        return Err(fault("credential_references_pending"));
    }
    let mut providers = BTreeSet::new();
    let mut targets = BTreeSet::new();
    let mut pairs: BTreeMap<String, (String, String, String)> = BTreeMap::new();
    let mut original_count = 0;
    let mut next_count = 0;
    let mut source_accounts = BTreeMap::new();
    // Complete ALL selected-source and target validation before reading secrets.
    for s in sources {
        let from = &s.source_provider_id;
        let to = &s.target_provider_id;
        let old = ordinary(original, from)?;
        let new = ordinary(next, to)?;
        providers.insert(from);
        providers.insert(to);
        if providers.len() > 32 || !targets.insert(to) {
            return Err(fault("credential_references_provider"));
        }
        if !source_accounts.contains_key(from) {
            let mut owned = BTreeSet::new();
            private_refs(old, &mut original_count, &mut |raw| {
                owned.insert(provider_auth::owned_account(raw, from)?.unwrap().to_owned());
                Ok(())
            })?;
            source_accounts.insert(from, owned);
        }
        let owned = &source_accounts[from];
        private_refs(new, &mut next_count, &mut |raw| {
            let account = provider_auth::owned_account(raw, from)?.unwrap();
            if !owned.contains(account) {
                return Err(fault("credential_references_reference"));
            }
            // Adapter changes are expected for derived protocol groups. This
            // API cannot authorize a changed upstream URL/auth direction.
            if old["baseUrl"].as_str().is_none_or(|s| s.is_empty())
                || old["baseUrl"] != new["baseUrl"]
                || old["authMode"] != new["authMode"]
                || old["apiKeyTransport"] != new["apiKeyTransport"]
            {
                return Err(fault("credential_references_direction"));
            }
            let target = format!("{to}{}", &account[from.len()..]);
            provider_auth::owned_account(&format!("keychain:{target}"), to)?;
            if let Some((existing, _, _)) = pairs.get(&target) {
                if existing != account {
                    return Err(fault("credential_references_collision"));
                }
            } else {
                pairs.insert(target, (account.to_owned(), from.clone(), to.clone()));
            }
            Ok(())
        })?;
    }
    // Temporary reference-only transform; restore before any fallible vault IO.
    rewrite(next, sources, false)?;
    let after = value_digest(next);
    rewrite(next, sources, true)?;
    let after = after?;
    let mut secrets = BTreeMap::new();
    let mut total = 0usize;
    let mut copies = Vec::new();
    let mut actions = Vec::new();
    for (to, (from, source_provider, target_provider)) in pairs {
        if !secrets.contains_key(&from) {
            let v =
                vault_read(vault, &from)?.ok_or_else(|| fault("credential_references_missing"))?;
            total += v.len();
            if total > MAX_TOTAL {
                return Err(fault("credential_references_limit"));
            }
            secrets.insert(from.clone(), v);
        }
        let digest = sha256(secrets[&from].as_bytes());
        let action = match vault_digest(vault, &to)? {
            Some(d) if d == digest => Action::Reused,
            Some(_) => return Err(fault("credential_references_collision")),
            None => Action::Absent,
        };
        copies.push(Copy {
            source_provider,
            target_provider,
            from,
            to,
            digest,
        });
        actions.push(action);
    }
    let mut j = Journal {
        version: 2,
        spec: Spec {
            id: uuid::Uuid::new_v4().to_string(),
            binding: ctx.binding()?,
            before: value_digest(original)?,
            after,
            copies,
        },
        phase: Phase::Preparing,
        actions,
    };
    let op = ctx.operation(&j.spec.id, true)?;
    op.create(JOURNAL, &envelope(&j)?)?;
    ctx.manager.create(
        PENDING,
        &envelope(&Marker {
            version: 2,
            id: j.spec.id.clone(),
            binding: j.spec.binding.clone(),
            spec_digest: checksum(&j.spec)?,
        })?,
    )?;
    let plan = ReferencePlanV2 {
        spec: j.spec.clone(),
    };
    let result = (|| {
        checkpoint("prepared")?;
        for i in 0..j.spec.copies.len() {
            if j.actions[i] == Action::Reused {
                continue;
            }
            let c = &j.spec.copies[i];
            if let Some(d) = vault_digest(vault, &c.to)? {
                if d != c.digest {
                    return Err(fault("credential_references_collision"));
                }
                j.actions[i] = Action::Reused;
                ctx.save(&j)?;
                continue;
            }
            j.actions[i] = Action::Creating;
            ctx.save(&j)?;
            checkpoint("before_create")?;
            vault
                .create(&c.to, &secrets[&c.from])
                .map_err(|_| fault("credential_references_vault"))?;
            checkpoint("after_create")?;
            if vault_digest(vault, &c.to)?.as_deref() != Some(&c.digest) {
                return Err(fault("credential_references_readback"));
            }
            j.actions[i] = Action::Created;
            ctx.save(&j)?;
            checkpoint("receipt")?;
        }
        for c in &j.spec.copies {
            if vault_digest(vault, &c.to)?.as_deref() != Some(&c.digest) {
                return Err(fault("credential_references_readback"));
            }
        }
        j.phase = Phase::Prepared;
        ctx.save(&j)?;
        checkpoint("ready")?;
        rewrite(next, sources, false)?;
        Ok(())
    })();
    if let Err(e) = result {
        if e.code != "credential_references_test_interrupted" {
            // An external editor need not honor app locks. Never compensate
            // against a stale original after an uncertain vault write.
            let actual = read_actual(&ctx)?;
            abort(root, home, &plan, &actual.0, vault)?;
        }
        return Err(e);
    }
    Ok(plan)
}
struct Actual(Value);
impl Drop for Actual {
    fn drop(&mut self) {
        provider_auth::erase_value(&mut self.0);
    }
}
fn read_actual(ctx: &Context) -> Result<Actual> {
    let bytes = ctx
        .home
        .read("config.json", 32 * 1024 * 1024, false)?
        .ok_or_else(|| fault("credential_references_actual"))?;
    let value = Actual(
        schema_v2::parse_strict_json_v2(&bytes)
            .map_err(|_| fault("credential_references_actual"))?,
    );
    if !value.0.is_object() {
        return Err(fault("credential_references_actual"));
    }
    Ok(value)
}
fn rewrite(next: &mut Value, sources: &[ReferenceSourceV2], reverse: bool) -> Result<()> {
    for s in sources {
        let (from, to) = if reverse {
            (&s.target_provider_id, &s.source_provider_id)
        } else {
            (&s.source_provider_id, &s.target_provider_id)
        };
        for field in provider_fields::PRIVATE {
            if let Some(v) = next["providers"][&s.target_provider_id].get_mut(*field) {
                provider_auth::rename_owned_references(v, from, to)?;
            }
        }
    }
    Ok(())
}
// Only used with the exact issued after-image. Resume the original UUID and
// accounts; verify every existing destination before any recovery copy.
fn complete_copies(ctx: &Context, j: &mut Journal, vault: &dyn CredentialVaultV2) -> Result<()> {
    let mut jobs = Vec::new();
    let mut total = 0usize;
    for i in 0..j.spec.copies.len() {
        let c = &j.spec.copies[i];
        let current = vault_digest(vault, &c.to)?;
        if current.as_ref().is_some_and(|d| d != &c.digest) {
            return Err(fault("credential_references_collision"));
        }
        match j.actions[i] {
            Action::Absent | Action::Creating if j.phase != Phase::Finished => {
                if current.is_some() {
                    // A recovered existing value is reusable, never an ownership
                    // receipt, even when its digest matches the original copy.
                    j.actions[i] = Action::Reused;
                    ctx.save(j)?;
                } else {
                    let secret = vault_read(vault, &c.from)?
                        .ok_or_else(|| fault("credential_references_missing"))?;
                    if sha256(secret.as_bytes()) != c.digest {
                        return Err(fault("credential_references_collision"));
                    }
                    total += secret.len();
                    if total > MAX_TOTAL {
                        return Err(fault("credential_references_limit"));
                    }
                    jobs.push((i, secret));
                }
            }
            Action::Created | Action::Reused if current.is_some() => (),
            _ => return Err(fault("credential_references_readback")),
        }
    }
    for (i, secret) in jobs {
        let c = &j.spec.copies[i];
        if let Some(d) = vault_digest(vault, &c.to)? {
            if d != c.digest {
                return Err(fault("credential_references_collision"));
            }
            j.actions[i] = Action::Reused;
            ctx.save(j)?;
            continue;
        }
        j.actions[i] = Action::Creating;
        ctx.save(j)?;
        checkpoint("before_create")?;
        vault
            .create(&c.to, &secret)
            .map_err(|_| fault("credential_references_vault"))?;
        checkpoint("after_create")?;
        if vault_digest(vault, &c.to)?.as_deref() != Some(&c.digest) {
            return Err(fault("credential_references_readback"));
        }
        j.actions[i] = Action::Created;
        ctx.save(j)?;
        checkpoint("receipt")?;
    }
    Ok(())
}
/// actual is the caller's locked, strictly parsed config after file readback.
pub fn finish(
    root: &Path,
    home: &Path,
    plan: &ReferencePlanV2,
    actual: &Value,
    vault: &dyn CredentialVaultV2,
) -> ModelResult<()> {
    let ctx = Context::open(root, home)?;
    let mut j = ctx.load(&plan.spec)?;
    if matches!(j.phase, Phase::Aborting | Phase::Aborted) || value_digest(actual)? != j.spec.after
    {
        return Err(fault("credential_references_actual"));
    }
    complete_copies(&ctx, &mut j, vault)?;
    for c in &j.spec.copies {
        if !whole_references(&actual["providers"][&c.target_provider], &c.to)
            || vault_digest(vault, &c.to)?.as_deref() != Some(&c.digest)
        {
            return Err(fault("credential_references_readback"));
        }
    }
    j.phase = Phase::Finished;
    ctx.save(&j)?;
    checkpoint("finished")?;
    ctx.clear(&j.spec)
}
/// Removes only receipted creations unreferenced across the ENTIRE config.
pub fn abort(
    root: &Path,
    home: &Path,
    plan: &ReferencePlanV2,
    actual: &Value,
    vault: &dyn CredentialVaultV2,
) -> ModelResult<()> {
    let ctx = Context::open(root, home)?;
    let mut j = ctx.load(&plan.spec)?;
    if j.phase == Phase::Finished {
        return Err(fault("credential_references_actual"));
    }
    if j.phase == Phase::Aborted {
        return ctx.clear(&j.spec);
    }
    j.phase = Phase::Aborting;
    ctx.save(&j)?;
    let mut pending = false;
    for i in 0..j.spec.copies.len() {
        let c = &j.spec.copies[i];
        match j.actions[i] {
            Action::Created => {
                if whole_references(actual, &c.to) {
                    pending = true;
                    continue;
                }
                if vault_digest(vault, &c.to)?
                    .as_ref()
                    .is_some_and(|d| d != &c.digest)
                {
                    j.actions[i] = Action::Unowned;
                    ctx.save(&j)?;
                    pending = true;
                    continue;
                }
                // The deletion itself also needs a write-ahead uncertainty
                // state: a crash can lose its receipt before another writer
                // recreates an identical credential in this account.
                j.actions[i] = Action::Removing;
                ctx.save(&j)?;
                checkpoint("before_remove")?;
                vault
                    .remove_if_digest(&c.to, &c.digest)
                    .map_err(|_| fault("credential_references_cleanup"))?;
                checkpoint("after_remove")?;
                if vault_digest(vault, &c.to)?.is_some() {
                    pending = true;
                    continue;
                }
                j.actions[i] = Action::Removed;
                ctx.save(&j)?;
            }
            Action::Creating | Action::Removing | Action::Unowned => {
                // A matching digest cannot prove ownership after a lost receipt
                // or after observing an external replacement.
                if vault_digest(vault, &c.to)?.is_some() {
                    pending = true;
                } else {
                    j.actions[i] = Action::Removed;
                    ctx.save(&j)?;
                }
            }
            _ => (),
        }
    }
    if pending {
        return Err(fault("credential_references_cleanup"));
    }
    j.phase = Phase::Aborted;
    ctx.save(&j)?;
    ctx.clear(&j.spec)
}
/// Only the fixed marker and its bound UUID record are read. Exact expected
/// actual may finish the original plan; other images allow compensation only.
/// Run model_transaction_v2 recovery first, then reread actual under the same
/// outer gates. An error retains this marker and must not start a new plan.
pub fn recover(
    root: &Path,
    home: &Path,
    actual: &Value,
    vault: &dyn CredentialVaultV2,
) -> ModelResult<bool> {
    let ctx = Context::open(root, home)?;
    let Some(m) = ctx.marker()? else {
        return Ok(false);
    };
    if m.version != 2 || m.binding != ctx.binding()? || !uuid4(&m.id) {
        return Err(fault("credential_references_binding"));
    }
    let dir = ctx.operation(&m.id, false)?;
    let j: Journal = decode(
        &dir.read(JOURNAL, MAX_IMAGE, true)?
            .ok_or_else(|| fault("credential_references_metadata"))?,
    )?;
    validate(&j, &ctx.binding()?)?;
    ctx.check_marker(&j.spec, true)?;
    let plan = ReferencePlanV2 { spec: j.spec };
    if value_digest(actual)? == plan.spec.after
        && !matches!(j.phase, Phase::Aborting | Phase::Aborted)
    {
        finish(root, home, &plan, actual, vault)?;
    } else {
        abort(root, home, &plan, actual, vault)?;
    }
    Ok(true)
}
#[cfg(test)]
type ReferenceCheckpointV2 = fn(&str) -> Result<()>;
#[cfg(test)]
thread_local! { static HOOK: std::cell::RefCell<Option<ReferenceCheckpointV2>> = const { std::cell::RefCell::new(None) }; }
fn checkpoint(_point: &str) -> Result<()> {
    #[cfg(test)]
    HOOK.with(|h| h.borrow().map(|f| f(_point)).unwrap_or(Ok(())))?;
    Ok(())
}
#[cfg(test)]
#[path = "credential_references_v2_tests.rs"]
mod tests;

// Pinned directory/security adapter kept identical to model_transaction_v2
// (private there; no registration/shared-file edits in this module).
// Unix child operations are relative to pinned directory descriptors. Windows
// directory handles exclude FILE_SHARE_DELETE and reject every reparse point.
// root/home are explicit anchors; OS ancestors may contain /var -> /private/var.
struct Dir {
    path: PathBuf,
    handle: File,
}
fn plain_name(name: &str) -> Result<()> {
    if name.is_empty() || name.contains(['/', '\\', '\0']) || name == "." || name == ".." {
        return Err(fault("credential_references_path_invalid"));
    }
    Ok(())
}
fn supported(m: &Metadata) -> Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err(fault("credential_references_path_invalid"));
        }
    }
    if m.file_type().is_symlink() {
        return Err(fault("credential_references_path_invalid"));
    }
    Ok(())
}
fn private(m: &Metadata, directory: bool) -> Result<()> {
    supported(m)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        // SAFETY: geteuid has no arguments or memory preconditions.
        if m.uid() != unsafe { libc::geteuid() }
            || m.mode() & 0o7777 != if directory { 0o700 } else { 0o600 }
        {
            return Err(fault("credential_references_permissions"));
        }
    }
    #[cfg(not(unix))]
    let _ = directory;
    Ok(())
}
fn regular(m: &Metadata, secret: bool) -> Result<()> {
    supported(m)?;
    if !m.is_file() {
        return Err(fault("credential_references_path_invalid"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if m.nlink() != 1 {
            return Err(fault("credential_references_path_invalid"));
        }
    }
    if secret {
        private(m, false)?;
    }
    Ok(())
}
fn same_file(a: &Metadata, b: &Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        a.dev() == b.dev() && a.ino() == b.ino()
    }
    #[cfg(not(unix))]
    {
        a.is_dir() == b.is_dir() && a.created().ok() == b.created().ok()
    }
}
impl Dir {
    fn anchor(path: &Path) -> Result<Self> {
        Self::anchor_access(path, false)
    }
    fn anchor_access(path: &Path, security_write: bool) -> Result<Self> {
        #[cfg(not(windows))]
        let _ = security_write;
        if !path.is_absolute()
            || path
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
        {
            return Err(fault("credential_references_path_invalid"));
        }
        let m = io(std::fs::symlink_metadata(path))?;
        supported(&m)?;
        if !m.is_dir() {
            return Err(fault("credential_references_path_invalid"));
        }
        let path = io(std::fs::canonicalize(path))?;
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.custom_flags(0x02200000).share_mode(3);
            if security_write {
                options.access_mode(0x80000000 | 0x00040000);
            }
        }
        let handle = io(options.open(&path))?;
        if !same_file(&m, &io(handle.metadata())?) {
            return Err(fault("credential_references_binding"));
        }
        Ok(Self { path, handle })
    }
    fn live(&self) -> Result<()> {
        let m = io(std::fs::symlink_metadata(&self.path))?;
        supported(&m)?;
        if !m.is_dir() || !same_file(&m, &io(self.handle.metadata())?) {
            return Err(fault("credential_references_binding"));
        }
        Ok(())
    }
    fn binding(&self) -> Result<String> {
        let m = io(self.handle.metadata())?;
        #[cfg(unix)]
        {
            use std::os::unix::{ffi::OsStrExt, fs::MetadataExt};
            let mut b = self.path.as_os_str().as_bytes().to_vec();
            b.extend_from_slice(&m.dev().to_le_bytes());
            b.extend_from_slice(&m.ino().to_le_bytes());
            Ok(sha256(&b))
        }
        #[cfg(not(unix))]
        {
            Ok(sha256(
                format!("{}:{:?}", self.path.display(), m.created().ok()).as_bytes(),
            ))
        }
    }
    fn sync(&self) -> Result<()> {
        #[cfg(unix)]
        io(self.handle.sync_all())?;
        Ok(())
    }
    fn child(&self, name: &str, create: bool, secret: bool) -> Result<Option<Self>> {
        plain_name(name)?;
        self.live()?;
        #[cfg(unix)]
        {
            use std::{
                ffi::CString,
                os::fd::{AsRawFd, FromRawFd},
            };
            let c = CString::new(name).map_err(|_| fault("credential_references_path_invalid"))?;
            if create {
                // SAFETY: owned descriptor and a validated NUL-terminated name.
                let rc = unsafe { libc::mkdirat(self.handle.as_raw_fd(), c.as_ptr(), 0o700) };
                if rc != 0
                    && std::io::Error::last_os_error().kind() != std::io::ErrorKind::AlreadyExists
                {
                    return Err(fault("credential_references_io"));
                }
                self.sync()?;
            }
            // SAFETY: openat does not follow the final component.
            let fd = unsafe {
                libc::openat(
                    self.handle.as_raw_fd(),
                    c.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            };
            if fd < 0 {
                if std::io::Error::last_os_error().kind() == std::io::ErrorKind::NotFound {
                    return Ok(None);
                }
                return Err(fault("credential_references_path_invalid"));
            }
            // SAFETY: successful openat gives a new owned descriptor.
            let handle = unsafe { File::from_raw_fd(fd) };
            if secret {
                private(&io(handle.metadata())?, true)?;
            }
            let d = Self {
                path: self.path.join(name),
                handle,
            };
            d.live()?;
            Ok(Some(d))
        }
        #[cfg(not(unix))]
        {
            let path = self.path.join(name);
            let mut created = false;
            if create {
                #[cfg(windows)]
                let made = if secret {
                    crate::infrastructure::windows_private_create::create_private_dir_all(
                        &path, 0o700,
                    )
                } else {
                    std::fs::create_dir(&path)
                };
                #[cfg(not(windows))]
                let made = std::fs::create_dir(&path);
                match made {
                    Ok(()) => created = true,
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
                    Err(_) => return Err(fault("credential_references_io")),
                }
            }
            match std::fs::symlink_metadata(&path) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(_) => Err(fault("credential_references_io")),
                Ok(m) => {
                    if secret {
                        private(&m, true)?;
                    }
                    let d = Self::anchor_access(&path, created && secret)?;
                    #[cfg(windows)]
                    if secret {
                        if created {
                            win_private::protect(&d.handle, true)?;
                        }
                        win_private::verify(&d.handle, true)?;
                    }
                    Ok(Some(d))
                }
            }
        }
    }

    fn open(&self, name: &str, create: bool) -> Result<Option<File>> {
        plain_name(name)?;
        self.live()?;
        #[cfg(unix)]
        let opened = {
            use std::{
                ffi::CString,
                os::fd::{AsRawFd, FromRawFd},
            };
            let name =
                CString::new(name).map_err(|_| fault("credential_references_path_invalid"))?;
            let flags = libc::O_NOFOLLOW
                | libc::O_NONBLOCK
                | libc::O_CLOEXEC
                | if create {
                    libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL
                } else {
                    libc::O_RDONLY
                };
            // SAFETY: validated component relative to owned directory descriptor.
            let fd = unsafe { libc::openat(self.handle.as_raw_fd(), name.as_ptr(), flags, 0o600) };
            if fd < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                // SAFETY: successful openat returns a new owned descriptor.
                Ok(unsafe { File::from_raw_fd(fd) })
            }
        };
        #[cfg(not(unix))]
        let opened = {
            let mut options = OpenOptions::new();
            options.read(!create).write(create).create_new(create);
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                options.custom_flags(0x00200000);
                if create {
                    options.access_mode(0x40000000 | 0x00040000 | 0x00020000);
                }
            }
            #[cfg(windows)]
            {
                if create {
                    crate::infrastructure::platform::create_new_private_file(
                        &self.path.join(name),
                        0o600,
                    )
                } else {
                    options.open(self.path.join(name))
                }
            }
            #[cfg(not(windows))]
            {
                options.open(self.path.join(name))
            }
        };
        match opened {
            Ok(f) => {
                #[cfg(windows)]
                {
                    win_private::single_link(&f)?;
                    if create {
                        win_private::protect(&f, false)?;
                    }
                }
                regular(&io(f.metadata())?, create)?;
                Ok(Some(f))
            }
            Err(e) if !create && e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err(fault("credential_references_io")),
        }
    }
    fn read(&self, name: &str, limit: usize, secret: bool) -> Result<Option<Bytes>> {
        let Some(mut f) = self.open(name, false)? else {
            return Ok(None);
        };
        let m = io(f.metadata())?;
        regular(&m, secret)?;
        #[cfg(windows)]
        if secret {
            win_private::verify(&f, false)?;
        }
        if m.len() > limit as u64 {
            return Err(fault("credential_references_limit"));
        }
        // One allocation plus a growth sentinel: image buffers are never reallocated.
        let size = m.len() as usize;
        let mut bytes = Zeroizing::new(vec![0; size + 1]);
        let mut count = 0;
        while count < bytes.len() {
            let got = io(f.read(&mut bytes[count..]))?;
            if got == 0 {
                break;
            }
            count += got;
        }
        if count != size || io(f.metadata())?.len() != size as u64 {
            return Err(fault("credential_references_read_changed"));
        }
        bytes.truncate(size);
        Ok(Some(bytes))
    }
    fn unlink(&self, name: &str) -> Result<()> {
        plain_name(name)?;
        self.live()?;
        #[cfg(unix)]
        {
            use std::{ffi::CString, os::fd::AsRawFd};
            let c = CString::new(name).map_err(|_| fault("credential_references_path_invalid"))?;
            // SAFETY: validated component, owned directory descriptor.
            if unsafe { libc::unlinkat(self.handle.as_raw_fd(), c.as_ptr(), 0) } != 0 {
                return Err(fault("credential_references_io"));
            }
        }
        #[cfg(not(unix))]
        io(std::fs::remove_file(self.path.join(name)))?;
        self.sync()
    }
    fn rename(&self, from: &str, to: &str, new: bool) -> Result<()> {
        plain_name(from)?;
        plain_name(to)?;
        self.live()?;
        #[cfg(unix)]
        {
            use std::{ffi::CString, os::fd::AsRawFd};
            let from =
                CString::new(from).map_err(|_| fault("credential_references_path_invalid"))?;
            let to = CString::new(to).map_err(|_| fault("credential_references_path_invalid"))?;
            let fd = self.handle.as_raw_fd();
            // An exclusive rename publishes a new object in one step. A
            // link/unlink pair would leave two links if interrupted, preventing
            // safe recovery under our single-link file policy.
            let rc = if new {
                #[cfg(target_vendor = "apple")]
                // SAFETY: both names and the pinned directory fd are valid.
                {
                    unsafe {
                        libc::renameatx_np(fd, from.as_ptr(), fd, to.as_ptr(), libc::RENAME_EXCL)
                    }
                }
                #[cfg(any(target_os = "linux", target_os = "android"))]
                // SAFETY: both names and the pinned directory fd are valid.
                {
                    unsafe {
                        libc::renameat2(
                            fd,
                            from.as_ptr(),
                            fd,
                            to.as_ptr(),
                            libc::RENAME_NOREPLACE as _,
                        )
                    }
                }
                #[cfg(not(any(
                    target_vendor = "apple",
                    target_os = "linux",
                    target_os = "android"
                )))]
                {
                    return Err(fault("credential_references_platform_unsupported"));
                }
            } else {
                // SAFETY: atomic replacement within the same owned directory.
                unsafe { libc::renameat(fd, from.as_ptr(), fd, to.as_ptr()) }
            };
            if rc != 0 {
                return Err(fault("credential_references_io"));
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::Storage::FileSystem::{
                MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
            };
            let from: Vec<u16> = self
                .path
                .join(from)
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
            let to: Vec<u16> = self
                .path
                .join(to)
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
            // SAFETY: both UTF-16 buffers are terminated and valid for this call.
            if unsafe {
                MoveFileExW(
                    from.as_ptr(),
                    to.as_ptr(),
                    MOVEFILE_WRITE_THROUGH | if new { 0 } else { MOVEFILE_REPLACE_EXISTING },
                )
            } == 0
            {
                return Err(fault("credential_references_io"));
            }
        }
        #[cfg(not(any(unix, windows)))]
        return Err(fault("credential_references_platform_unsupported"));
        self.sync()
    }
    fn replace(
        &self,
        name: &str,
        before: Option<&[u8]>,
        after: Option<&[u8]>,
        secret: bool,
    ) -> Result<()> {
        if self
            .read(name, MAX_IMAGE, secret)?
            .as_ref()
            .map(|v| v.as_slice())
            != before
        {
            return Err(fault("credential_references_cas"));
        }
        if before == after {
            return Ok(());
        }
        if let Some(after) = after {
            let temp = format!(".credential-references-{}.tmp", uuid::Uuid::new_v4());
            let result = (|| {
                let mut f = self
                    .open(&temp, true)?
                    .ok_or_else(|| fault("credential_references_io"))?;
                io(f.write_all(after))?;
                io(f.sync_all())?;
                drop(f);
                if self
                    .read(name, MAX_IMAGE, secret)?
                    .as_ref()
                    .map(|v| v.as_slice())
                    != before
                {
                    return Err(fault("credential_references_cas"));
                }
                self.rename(&temp, name, before.is_none())
            })();
            if result.is_err() {
                let _ = self.unlink(&temp);
            }
            result?;
        } else {
            self.unlink(name)?;
        }
        if self
            .read(name, MAX_IMAGE, secret)?
            .as_ref()
            .map(|v| v.as_slice())
            != after
        {
            return Err(fault("credential_references_readback"));
        }
        Ok(())
    }
    fn create(&self, name: &str, bytes: &[u8]) -> Result<()> {
        self.replace(name, None, Some(bytes), true)
    }
}
// POSIX 0600/0700 equivalent on Windows: a protected owner-only DACL. Apply it
// before writing any bytes, and verify existing private objects without changing
// their permissions. The three authorization declarations avoid adding a Cargo
// feature solely for this adapter; all types are from existing windows-sys.
#[cfg(windows)]
mod win_private {
    use super::*;
    use std::{
        ffi::c_void,
        os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::{
            EqualSid, GetSecurityDescriptorControl, GetSecurityDescriptorDacl, GetTokenInformation,
            TokenUser, ACL, TOKEN_QUERY, TOKEN_USER,
        },
        Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION},
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    #[link(name = "advapi32")]
    extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text: *const u16,
            revision: u32,
            descriptor: *mut *mut c_void,
            length: *mut u32,
        ) -> i32;
        fn GetSecurityInfo(
            handle: *mut c_void,
            kind: i32,
            information: u32,
            owner: *mut *mut c_void,
            group: *mut *mut c_void,
            dacl: *mut *mut ACL,
            sacl: *mut *mut ACL,
            descriptor: *mut *mut c_void,
        ) -> u32;
        fn SetSecurityInfo(
            handle: *mut c_void,
            kind: i32,
            information: u32,
            owner: *mut c_void,
            group: *mut c_void,
            dacl: *const ACL,
            sacl: *const ACL,
        ) -> u32;
    }
    struct Descriptor(*mut c_void);
    impl Drop for Descriptor {
        fn drop(&mut self) {
            // SAFETY: Windows returned this LocalAlloc-owned descriptor.
            unsafe {
                LocalFree(self.0);
            }
        }
    }
    fn descriptor(directory: bool) -> Result<Descriptor> {
        let text = if directory {
            "D:P(A;OICI;FA;;;OW)"
        } else {
            "D:P(A;;FA;;;OW)"
        };
        let text: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        let mut sd = ptr::null_mut();
        // SAFETY: terminated input and a live output pointer; revision is SDDL 1.
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                1,
                &mut sd,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(fault("credential_references_permissions"));
        }
        Ok(Descriptor(sd))
    }
    fn dacl(sd: &Descriptor) -> Result<*mut ACL> {
        let mut present = 0;
        let mut defaulted = 0;
        let mut acl = ptr::null_mut();
        // SAFETY: descriptor is owned and valid; outputs are initialized by Win32.
        if unsafe { GetSecurityDescriptorDacl(sd.0, &mut present, &mut acl, &mut defaulted) } == 0
            || present == 0
            || acl.is_null()
        {
            return Err(fault("credential_references_permissions"));
        }
        Ok(acl)
    }
    pub(super) fn protect(file: &File, directory: bool) -> Result<()> {
        let expected = descriptor(directory)?;
        let acl = dacl(&expected)?;
        // SAFETY: live file handle and ACL; protect the DACL against inheritance.
        if unsafe {
            SetSecurityInfo(
                file.as_raw_handle(),
                1,
                0x80000004,
                ptr::null_mut(),
                ptr::null_mut(),
                acl,
                ptr::null(),
            )
        } != 0
        {
            return Err(fault("credential_references_permissions"));
        }
        verify(file, directory)
    }
    fn current_owner(owner: *mut c_void) -> Result<()> {
        let mut token = ptr::null_mut();
        // SAFETY: pseudo process handle and a live output pointer.
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(fault("credential_references_permissions"));
        }
        // SAFETY: successful OpenProcessToken transfers a new owned handle.
        let token = unsafe { OwnedHandle::from_raw_handle(token) };
        let mut size = 0;
        // SAFETY: the first call only queries the required buffer length.
        unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                ptr::null_mut(),
                0,
                &mut size,
            );
        }
        if size < std::mem::size_of::<TOKEN_USER>() as u32 || size > 65536 {
            return Err(fault("credential_references_permissions"));
        }
        let mut buffer = vec![0usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
        // SAFETY: buffer is aligned for TOKEN_USER and has at least size bytes.
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                buffer.as_mut_ptr().cast(),
                size,
                &mut size,
            )
        } == 0
        {
            return Err(fault("credential_references_permissions"));
        }
        // SAFETY: successful TokenUser query initialized this aligned structure.
        let sid = unsafe { (*(buffer.as_ptr().cast::<TOKEN_USER>())).User.Sid };
        // SAFETY: both SIDs originate from successful Windows security queries.
        if owner.is_null() || unsafe { EqualSid(owner, sid) } == 0 {
            return Err(fault("credential_references_permissions"));
        }
        Ok(())
    }
    pub(super) fn verify(file: &File, directory: bool) -> Result<()> {
        let expected = descriptor(directory)?;
        let expected_acl = dacl(&expected)?;
        let mut owner = ptr::null_mut();
        let mut acl = ptr::null_mut();
        let mut sd = ptr::null_mut();
        // SAFETY: live handle and output pointers; request owner and DACL only.
        if unsafe {
            GetSecurityInfo(
                file.as_raw_handle(),
                1,
                5,
                &mut owner,
                ptr::null_mut(),
                &mut acl,
                ptr::null_mut(),
                &mut sd,
            )
        } != 0
        {
            return Err(fault("credential_references_permissions"));
        }
        let sd = Descriptor(sd);
        current_owner(owner)?;
        let mut control = 0;
        let mut revision = 0;
        // SAFETY: live security descriptor and valid output pointers.
        if unsafe { GetSecurityDescriptorControl(sd.0, &mut control, &mut revision) } == 0
            || control & 0x1000 == 0
            || acl.is_null()
        {
            return Err(fault("credential_references_permissions"));
        }
        // SAFETY: successful security queries guarantee valid ACL headers and
        // AclSize-byte buffers, both kept alive by their descriptor guards.
        let equal = unsafe {
            let a = std::slice::from_raw_parts(acl.cast::<u8>(), (*acl).AclSize as usize);
            let b = std::slice::from_raw_parts(
                expected_acl.cast::<u8>(),
                (*expected_acl).AclSize as usize,
            );
            a == b
        };
        if !equal {
            return Err(fault("credential_references_permissions"));
        }
        Ok(())
    }
    pub(super) fn single_link(file: &File) -> Result<()> {
        let mut info = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
        // SAFETY: live file handle and writable output storage.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), info.as_mut_ptr()) } == 0 {
            return Err(fault("credential_references_io"));
        }
        // SAFETY: successful query initialized the output.
        if unsafe { info.assume_init() }.nNumberOfLinks != 1 {
            return Err(fault("credential_references_path_invalid"));
        }
        Ok(())
    }
}
