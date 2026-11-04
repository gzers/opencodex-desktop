//! Candidate transaction only. It does not replace or patch the production mapper.
use opencodex_desktop_lib::{infrastructure::{atomic_write::atomic_write, locking::TargetFileLock}, modules::model_config::{self as model, config_file, store::sha256}};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::{BTreeMap, BTreeSet}, fs, path::{Path, PathBuf}};
use zeroize::Zeroizing;
use opencodex_desktop_lib::state::ProcessContext;

pub type Result<T> = std::result::Result<T, String>;
pub const SERVICE: &str = "opencodex.provider-api-key.v1";
pub const LAYOUT: &str = "manager-state/model-channel-layout.json";
pub const PREFS: &str = "manager-state/preferences.json";
pub const IDS: &str = "manager-state/model-execution-source-ids.json";
pub const GROUPS: &str = "manager-state/model-protocol-mappings.json";
pub fn read(path: &Path) -> Value { serde_json::from_slice(&fs::read(path).unwrap()).unwrap() }
pub fn write(path: &Path, value: &Value) { atomic_write(path, &serde_json::to_vec_pretty(value).unwrap(), 0o600).unwrap(); }
fn trace(value:Value) { if std::env::var_os("G0_TRACE").is_some() {eprintln!("{}",value);} }
fn error<T>(code: &str) -> Result<T> { Err(code.into()) }
fn optional_bytes(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {Ok(bytes)=>Ok(Some(bytes)),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(_)=>error("baseline_read")}
}
pub fn get(account: &str) -> Result<Option<Zeroizing<String>>> {
    match keyring::Entry::new(SERVICE, account).map_err(|_| "credential_entry")?.get_password() {
        Ok(v) => Ok(Some(Zeroizing::new(v))),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => error("credential_read"),
    }
}
pub fn put(account: &str, value: &str) -> Result<()> {
    keyring::Entry::new(SERVICE, account).map_err(|_| "credential_entry")?.set_password(value).map_err(|_| "credential_write".into())
}
fn remove(account: &str) -> Result<()> {
    match keyring::Entry::new(SERVICE, account).map_err(|_| "credential_entry")?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()), Err(_) => error("credential_delete"),
    }
}
fn digest(account: &str) -> Result<Option<String>> { Ok(get(account)?.as_deref().map(|v| sha256(v.as_bytes()))) }
fn owner(raw: &str, provider: &str) -> bool { raw == provider || raw.strip_prefix(provider).is_some_and(|tail| tail.starts_with('/')) }
fn routed(raw: &str, old: &str, new: &str) -> String {
    match raw.split_once('/') { Some((p, tail)) if p == old => format!("{new}/{tail}"), _ => raw.into() }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Rename { pub id: String, pub label: String, pub target: String }
#[derive(Clone)]
pub struct Image { pub path: PathBuf, pub before: Vec<u8>, pub after: Vec<u8> }
#[derive(Clone, Serialize, Deserialize)]
pub struct Move { from: String, to: String, digest: String }
pub struct Plan { pub images: Vec<Image>, pub guards: Vec<(PathBuf, Option<Vec<u8>>)>, pub moves: Vec<Move>, pub journal: PathBuf }
fn ref_edit(value: &mut Value, old: &str, new: &str, moves: &mut BTreeMap<String, Move>) -> Result<()> {
    match value {
        Value::String(s) if s.starts_with("keychain:") => {
            let account = s.strip_prefix("keychain:").unwrap();
            if !owner(account, old) { return error("credential_owner"); }
            let target = format!("{new}{}", &account[old.len()..]);
            let hash = digest(account)?.ok_or("credential_missing")?;
            if digest(&target)?.is_some() { return error("credential_target_occupied"); }
            moves.insert(account.into(), Move { from: account.into(), to: target.clone(), digest: hash });
            *s = format!("keychain:{target}");
        }
        Value::Array(a) => for v in a { ref_edit(v, old, new, moves)?; },
        Value::Object(o) => for v in o.values_mut() { ref_edit(v, old, new, moves)?; },
        _ => (),
    }
    Ok(())
}
fn protected(provider: &str, raw: &Value) -> bool {
    ["openai", "chatgpt", "openai-multi", "combo"].contains(&provider) || matches!(raw["authMode"].as_str(), Some("oauth" | "forward"))
}
pub fn prepare(root: &Path, home: &Path, requested: &[Rename], scope: Option<&str>, context: &ProcessContext) -> Result<Plan> {
    if context.opencodex_home != home {return error("context_home");}
    let snap = config_file::snapshot(root, home).map_err(|e| e.code)?;
    let raw_path = home.join("config.json");
    let mut raw = read(&raw_path);
    let mut layout = read(&root.join(LAYOUT));
    let mut prefs = if root.join(PREFS).exists() {read(&root.join(PREFS))} else {Value::Null};
    let mut ids = if root.join(IDS).exists() {read(&root.join(IDS))} else {Value::Null};
    let groups_path=root.join(GROUPS);
    let groups=if groups_path.exists(){read(&groups_path)}else{json!([])};
    let mut intended = snap.policy.clone();
    let edits = requested.iter().filter(|r| scope.is_none_or(|s| s == r.id)).collect::<Vec<_>>();
    if scope.is_some_and(|s| !edits.iter().any(|r| r.id == s)) { return error("save_scope"); }
    let mut targets = BTreeSet::new();
    let mut identities = BTreeSet::new();
    let mut moves = BTreeMap::new();
    for edit in &edits {
        if !identities.insert(&edit.id) || !targets.insert(&edit.target) { return error("rename_duplicate"); }
        let saved = snap.policy.channels.iter().find(|c| c.id == edit.id).ok_or("channel_missing")?;
        let old = &saved.connection.provider_id;
        let source = &raw["providers"][old];
        if old == &edit.target { return error("rename_noop"); }
        if protected(old, source) || protected(&edit.target, source) { return error("protected_provider"); }
        if raw["providers"].get(&edit.target).is_some() { return error("provider_occupied"); }
        // Protocol groups keep their own provider identity, even when invisible.
        if groups.as_array().ok_or("protocol_mapping")?.iter().any(|m| m["providerId"] == edit.target) { return error("derived_provider_occupied"); }
        let c = intended.channels.iter_mut().find(|c| c.id == edit.id).unwrap();
        c.label = edit.label.clone(); c.connection.provider_id = edit.target.clone();
        model::compiler::validate_channel(c).map_err(|e| e.code)?;
        if c.connection.imported_config.is_some() { return error("provider_import_scope"); }
        let mut row = raw["providers"].as_object_mut().unwrap().remove(old).unwrap();
        for field in model::provider_fields::PRIVATE {
            if let Some(value) = row.get_mut(*field) { ref_edit(value, old, &edit.target, &mut moves)?; }
        }
        if row.get("name").is_some() { row["name"] = json!(edit.label); }
        raw["providers"][&edit.target] = row;
        if raw["defaultProvider"] == *old { raw["defaultProvider"] = json!(edit.target); }
        for key in ["modelPickerOrder", "disabledModels"] {
            if let Some(values) = raw.get_mut(key).and_then(Value::as_array_mut) {
                for value in values { if let Some(s) = value.as_str() { *value = json!(routed(s, old, &edit.target)); } }
            }
        }
        for c in layout["channels"].as_array_mut().unwrap() {
            if c["id"] == edit.id { c["label"] = json!(edit.label); c["connection"]["providerId"] = json!(edit.target); }
        }
        if let Some(agent) = prefs.get_mut("modelTemplateAgent") {
            if agent["providerId"] == *old {
                if agent.get("channelId").is_some_and(|v| v != &json!(edit.id)) { return error("agent_binding"); }
                agent["providerId"] = json!(edit.target);
                // Temporary mirror of the existing PRIVATE pure helper in
                // execution_directory::connection_binding. A real change
                // should expose/reuse that helper. The final probe compares
                // this persisted value with production bound_directory/bind.
                let runtime=sha256(&serde_json::to_vec(&(context.executable().map_err(|_|"runtime_binding")?,&context.opencodex_home)).unwrap());
                let row=&raw["providers"][&edit.target];
                let binding=sha256(&serde_json::to_vec(&(runtime,&edit.target,&row["adapter"],&row["baseUrl"],&row["authMode"],&row["apiKeyTransport"],&row["responsesPath"])).unwrap());
                agent["connectionDigest"]=json!(binding);
            }
        }
        if let Some(id) = ids.as_object_mut().and_then(|ids|ids.remove(old)) {
            if id != edit.id { return error("execution_identity"); }
            if ids.get(&edit.target).is_some() { return error("execution_identity_target"); }
            ids[&edit.target] = id;
        }
    }
    // Reject foreign stale references in a sibling provider before deleting the
    // source account. Never silently retarget another channel's authorization.
    fn refs(v: &Value, out: &mut Vec<String>) {
        match v { Value::String(s) if s.starts_with("keychain:") => out.push(s[9..].into()), Value::Array(a) => for x in a {refs(x,out)}, Value::Object(o) => for x in o.values(){refs(x,out)}, _ => () }
    }
    let mut referenced = vec![]; refs(&raw, &mut referenced);
    if moves.values().any(|m| referenced.contains(&m.from)) { return error("credential_shared_reference"); }
    let mut images = vec![];
    for (path, next) in [(raw_path,raw),(root.join(LAYOUT),layout),(root.join(PREFS),prefs),(root.join(IDS),ids)] {
        if next.is_null() && !path.exists() {continue;}
        let before = fs::read(&path).map_err(|_| "read_baseline")?;
        let after = serde_json::to_vec_pretty(&next).unwrap();
        if serde_json::from_slice::<Value>(&before).unwrap() != next { images.push(Image {path,before,after}); }
    }
    let guards = [GROUPS,PREFS,IDS].into_iter().map(|p|root.join(p)).filter(|p|!images.iter().any(|i|&i.path==p)).map(|p|optional_bytes(&p).map(|bytes|(p,bytes))).collect::<Result<Vec<_>>>()?;
    Ok(Plan {images,guards,moves:moves.into_values().collect(),journal:root.join("manager-state/g0-rename-transaction/journal.json")})
}

#[derive(Serialize, Deserialize)]
struct Record { path: PathBuf, before: String, after: String, before_hash: String, after_hash: String }
#[derive(Serialize, Deserialize)]
struct Journal { decision: String, files: Vec<Record>, moves: Vec<Move> }
fn persist(path: &Path, j: &Journal) -> Result<()> {
    atomic_write(path,&serde_json::to_vec(j).unwrap(),0o600).map_err(|_| "journal_write".into())
}
fn load(path: &Path) -> Result<Journal> { serde_json::from_slice(&fs::read(path).map_err(|_| "journal_read")?).map_err(|_| "journal_parse".into()) }
fn journal_image(path: &Path, name: &str, hash: &str) -> Result<Vec<u8>> {
    let bytes=fs::read(path.parent().unwrap().join(name)).map_err(|_| "backup_read")?;
    if sha256(&bytes) != hash {return error("backup_digest");} Ok(bytes)
}
#[derive(Default)]
pub struct Fault { pub fail: Option<String>, pub crash: Option<String>, pub rollback_fail: bool, pub drift: Option<PathBuf>, pub real_write_fail: Option<usize> }
impl Fault {
    fn point(&self, name: &str) -> Result<()> {
        if self.crash.as_deref()==Some(name) { return error("simulated_process_loss"); }
        if self.fail.as_deref()==Some(name) { return error(&format!("injected:{name}")); } Ok(())
    }
}
#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum Outcome { Committed, RolledBack, RecoveryRequired, CleanupPending, SimulatedProcessLoss, NoPending }
fn locks(paths: impl Iterator<Item=PathBuf>) -> Result<Vec<TargetFileLock>> {
    // Keep config -> layout, the order used by actual snapshot/save readers.
    // Preferences' existing namespace guard is acquired outside these locks.
    let mut seen=BTreeSet::new();let paths=paths.filter(|p|seen.insert(p.clone())).collect::<Vec<_>>();
    paths.iter().map(|p|TargetFileLock::lock(p).map_err(|_| "lock".into())).collect()
}
pub fn execute(plan: &Plan, fault: &Fault) -> Result<Outcome> {
    let root=plan.journal.ancestors().nth(3).ok_or("journal_root")?;
    let _preferences=opencodex_desktop_lib::modules::backup::manager::acquire_preferences_transaction(root).map_err(|_|"preferences_lock")?;
    let _locks=locks(plan.images.iter().map(|i|i.path.clone()).chain(plan.guards.iter().map(|(p,_)|p.clone())))?;
    for image in &plan.images { if fs::read(&image.path).map_err(|_| "read_baseline")? != image.before {return error("file_changed");} }
    for (p,bytes) in &plan.guards {if optional_bytes(p)? != *bytes{return error("guard_changed");}}
    for m in &plan.moves {
        if digest(&m.from)?.as_deref()!=Some(&m.digest) {return error("credential_changed");}
        if digest(&m.to)?.is_some() {return error("credential_target_occupied");}
    }
    if plan.journal.exists() {return error("recovery_pending");}
    let dir=plan.journal.parent().unwrap(); fs::create_dir_all(dir).map_err(|_| "journal_dir")?;
    opencodex_desktop_lib::infrastructure::platform::set_mode(dir,0o700).map_err(|_|"journal_mode")?;
    let mut journal=Journal{decision:"prepared".into(),files:vec![],moves:plan.moves.clone()};
    for (index,image) in plan.images.iter().enumerate() {
        let before=format!("{index}.before"); let after=format!("{index}.after");
        atomic_write(&dir.join(&before),&image.before,0o600).map_err(|_| "backup_write")?;
        atomic_write(&dir.join(&after),&image.after,0o600).map_err(|_| "backup_write")?;
        journal.files.push(Record{path:image.path.clone(),before,after,before_hash:sha256(&image.before),after_hash:sha256(&image.after)});
    }
    persist(&plan.journal,&journal)?;
    trace(json!({"phase":"prepared","journal":plan.journal,"file_count":plan.images.len(),"credential_move_count":plan.moves.len()}));
    let result=(|| -> Result<()> {
        for (i,m) in plan.moves.iter().enumerate() {
            let secret=get(&m.from)?.ok_or("credential_missing")?;
            if sha256(secret.as_bytes())!=m.digest {return error("credential_changed");}
            if digest(&m.to)?.is_some() {return error("credential_target_occupied");}
            put(&m.to,&secret)?;
            if digest(&m.to)?.as_deref()!=Some(&m.digest) {return error("credential_readback");}
            trace(json!({"phase":"credential_staged_readback","from":m.from,"to":m.to}));
            fault.point(&format!("key:{i}"))?;
        }
        for (i,image) in plan.images.iter().enumerate() {
            if fs::read(&image.path).map_err(|_| "file_read")? != image.before {return error("file_changed");}
            if fault.real_write_fail==Some(i) {
                let blocked=dir.join("write-error-directory");fs::create_dir_all(&blocked).unwrap();
                atomic_write(&blocked,&image.after,0o600).map_err(|_| "actual_atomic_write_failure")?;
                return error("expected_write_failure_missing");
            }
            atomic_write(&image.path,&image.after,0o600).map_err(|_| "file_write")?;
            trace(json!({"phase":"file_published","path":image.path,"before_digest":sha256(&image.before),"after_digest":sha256(&image.after)}));
            fault.point(&format!("file:{i}"))?;
        }
        if let Some(p)=&fault.drift {write(p,&json!({"externalWriter":"newer-do-not-overwrite"}));}
        for image in &plan.images {if fs::read(&image.path).map_err(|_| "readback")? != image.after {return error("file_readback_drift");}}
        for (p,bytes) in &plan.guards {if optional_bytes(p)? != *bytes{return error("guard_changed");}}
        for m in &plan.moves {if digest(&m.to)?.as_deref()!=Some(&m.digest){return error("credential_readback");}}
        for m in &plan.moves {if digest(&m.from)?.as_deref()!=Some(&m.digest){return error("credential_changed");}}
        fault.point("readback")?;
        journal.decision="committed".into(); persist(&plan.journal,&journal)?;
        trace(json!({"phase":"commit_decision_durable","journal":plan.journal}));
        fault.point("commit")?;
        Ok(())
    })();
    if result.as_ref().err().map(String::as_str)==Some("simulated_process_loss") {return Ok(Outcome::SimulatedProcessLoss);}
    if result.is_err() && fault.rollback_fail {return Ok(Outcome::RecoveryRequired);}
    recover_locked(&plan.journal, fault.fail.as_deref()==Some("cleanup"))
}
pub fn recover(path: &Path) -> Result<Outcome> {
    if !path.exists() {return Ok(Outcome::NoPending);}
    let root=path.ancestors().nth(3).ok_or("journal_root")?;
    let _preferences=opencodex_desktop_lib::modules::backup::manager::acquire_preferences_transaction(root).map_err(|_|"preferences_lock")?;
    let journal=load(path)?;
    let _locks=locks(journal.files.iter().map(|r|r.path.clone()))?;
    recover_locked(path,false)
}
fn recover_locked(path: &Path, cleanup_failure: bool) -> Result<Outcome> {
    let journal=load(path)?;
    if journal.decision=="committed" {
        for file in &journal.files {
            if sha256(&fs::read(&file.path).map_err(|_| "readback")?) != file.after_hash {return Ok(Outcome::RecoveryRequired);}
        }
        for m in &journal.moves {
            if digest(&m.to)?.as_deref()!=Some(&m.digest) {return Ok(Outcome::CleanupPending);}
            if cleanup_failure {return Ok(Outcome::CleanupPending);}
            if let Some(hash)=digest(&m.from)? {
                if hash!=m.digest {return Ok(Outcome::CleanupPending);}
                if remove(&m.from).is_err() || digest(&m.from)?.is_some() {return Ok(Outcome::CleanupPending);}
                trace(json!({"phase":"old_credential_removed_readback","account":m.from}));
            }
        }
        fs::remove_file(path).map_err(|_| "journal_remove")?;
        return Ok(Outcome::Committed);
    }
    let mut failed=false;
    for file in journal.files.iter().rev() {
        let current=sha256(&fs::read(&file.path).map_err(|_| "rollback_read")?);
        if current==file.before_hash {continue;}
        if current!=file.after_hash {trace(json!({"phase":"rollback_refused_external_drift","path":file.path}));failed=true;continue;}
        let before=journal_image(path,&file.before,&file.before_hash)?;
        if atomic_write(&file.path,&before,0o600).is_err() || fs::read(&file.path).ok().as_deref()!=Some(before.as_slice()) {failed=true;}
        trace(json!({"phase":"file_rollback_readback","path":file.path,"restored_digest":sha256(&before)}));
    }
    // Keep staged accounts while any new file may still refer to them.
    if failed {return Ok(Outcome::RecoveryRequired);}
    for m in journal.moves.iter().rev() {
        if digest(&m.from)?.as_deref()!=Some(&m.digest) {return Ok(Outcome::RecoveryRequired);}
        if let Some(hash)=digest(&m.to)? {
            if hash!=m.digest {return Ok(Outcome::RecoveryRequired);}
            if remove(&m.to).is_err() || digest(&m.to)?.is_some() {return Ok(Outcome::RecoveryRequired);}
            trace(json!({"phase":"staged_credential_removed","account":m.to}));
        }
    }
    fs::remove_file(path).map_err(|_| "journal_remove")?;
    Ok(Outcome::RolledBack)
}
